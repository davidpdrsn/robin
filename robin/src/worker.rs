use config::Config;
use connection::*;
use job::*;
use queue_adapters::{JobQueue, NoJobDequeued, QueueIdentifier, RetryCount};
use serde_json;
use std::sync::mpsc::*;
use std::thread::{self, JoinHandle};

/// Boot the worker.
///
/// **NOTE:** You normally wouldn't need to call this. Instead use the
/// [`robin_boot_worker!`](../macro.robin_boot_worker.html) macro in the `macros` module.
///
/// This will spawn the numbers of workers set by
/// [`config.worker_count`](../config/struct.Config.html#structfield.worker_count) plus one for
/// handling retries.
///
/// Make sure the config you're using here is the same config you use to establish the connection
/// in [`robin_establish_connection!`](../macro.robin_establish_connection.html).
pub fn boot<Q, T, K>(config: &Config, queue_config: K, lookup_job: T)
where
    K: 'static + Clone + Send,
    Q: JobQueue<Config = K>,
    T: 'static + LookupJob<Q> + Send + Clone,
{
    spawn_workers(config, queue_config, lookup_job).job_loop()
}

/// Spawn the workers and return the [`WorkerManager`](struct.WorkerManager) which enables
/// communication with the workers.
///
/// You should only need this method during tests.
///
/// If you don't need the `WorkerManager` but just want to keep performing jobs in an infinite
/// loop, use [`boot`](fn.boot.html) instead.
pub fn spawn_workers<Q, T, K>(config: &Config, queue_config: K, lookup_job: T) -> WorkerManager
where
    K: 'static + Clone + Send,
    Q: JobQueue<Config = K>,
    T: 'static + LookupJob<Q> + Send + Clone,
{
    let mut channel: MultiplexChannel<WorkerMessage> = MultiplexChannel::new();

    let mut handles: Vec<JoinHandle<()>> = config
        .worker_count
        .times()
        .map(|_| {
            spawn_worker(
                channel.new_receiver(),
                &config,
                &lookup_job,
                QueueIdentifier::Main,
                queue_config.clone(),
            )
        })
        .collect();

    handles.push(spawn_worker(
        channel.new_receiver(),
        &config,
        &lookup_job,
        QueueIdentifier::Retry,
        queue_config.clone(),
    ));

    WorkerManager { handles, channel }
}

fn spawn_worker<T, Q, K>(
    receiver: Receiver<WorkerMessage>,
    config: &Config,
    lookup_job: &T,
    queue_iden: QueueIdentifier,
    queue_config: K,
) -> JoinHandle<()>
where
    K: 'static + Clone + Send,
    Q: JobQueue<Config = K>,
    T: 'static + LookupJob<Q> + Send + Clone,
{
    let config = config.clone();
    let queue_config = queue_config.clone();
    let lookup_job = lookup_job.clone();
    thread::spawn(move || worker_loop(receiver, config, lookup_job, queue_iden, queue_config))
}

/// Struct the allows you to communicate with the running workers.
#[allow(missing_debug_implementations)]
pub struct WorkerManager {
    handles: Vec<JoinHandle<()>>,
    channel: MultiplexChannel<WorkerMessage>,
}

impl WorkerManager {
    /// Kill the workers once there are no more jobs left to perform.
    pub fn perform_all_jobs_and_die(self) {
        self.channel.send(WorkerMessage::PerformJobsAndDie);
        self.join_threads();
    }

    /// Kill the workers once they've performed the job they're currently working on.
    pub fn die(self) {
        self.channel.send(WorkerMessage::Die);
        self.join_threads();
    }

    fn job_loop(self) {
        self.join_threads()
    }

    fn join_threads(self) {
        self.handles.into_iter().for_each(|h| h.join().unwrap());
    }
}

#[derive(Debug, Clone, Copy)]
enum WorkerMessage {
    Die,
    PerformJobsAndDie,
}

fn worker_loop<Q, T, K>(
    receiver: Receiver<WorkerMessage>,
    config: Config,
    lookup_job: T,
    queue_iden: QueueIdentifier,
    queue_config: K,
) where
    Q: JobQueue<Config = K>,
    T: 'static + LookupJob<Q>,
{
    let con = establish(config, queue_config, lookup_job).expect("failed to establish connection");
    let mut received_perform_jobs_and_die = false;

    loop {
        let job = con.dequeue_from(queue_iden);
        let output = perform_job(job, &con);

        match output {
            PerformJobOutput::JobPerformed => {}
            PerformJobOutput::JobRetried => {}
            PerformJobOutput::NoJobPerformed(reason) => match reason {
                NoJobPerformedReason::HitTimeout => if received_perform_jobs_and_die {
                    break;
                },
                NoJobPerformedReason::RetryLimitReached => {
                    debug!("retry limit reached");
                }
            },
        }

        if let Ok(msg) = receiver.try_recv() {
            match msg {
                WorkerMessage::Die => break,
                WorkerMessage::PerformJobsAndDie => received_perform_jobs_and_die = true,
            }
        }
    }
}

type DequeuedJob<Q> = Result<(Box<Job<Q> + Send + 'static>, String, RetryCount), NoJobDequeued>;

#[derive(Debug)]
enum PerformJobOutput {
    JobPerformed,
    JobRetried,
    NoJobPerformed(NoJobPerformedReason),
}

#[derive(Debug)]
enum NoJobPerformedReason {
    HitTimeout,
    RetryLimitReached,
}

fn perform_job<Q>(job: DequeuedJob<Q>, con: &Connection<Q>) -> PerformJobOutput
where
    Q: JobQueue,
{
    match job {
        Ok((job, args, retry_count)) => perform_or_retry(con, job, args, retry_count),

        Err(NoJobDequeued::BecauseTimeout) => {
            PerformJobOutput::NoJobPerformed(NoJobPerformedReason::HitTimeout)
        }

        Err(NoJobDequeued::BecauseError(err)) => {
            panic!(format!("Failed to dequeue job with error\n{:?}", err))
        }
    }
}

fn perform_or_retry<Q: JobQueue>(
    con: &Connection<Q>,
    job: Box<Job<Q> + Send>,
    args: String,
    retry_count: RetryCount,
) -> PerformJobOutput {
    let retry_count = retry_count.increment();

    if retry_count.limit_reached(con.config()) {
        PerformJobOutput::NoJobPerformed(NoJobPerformedReason::RetryLimitReached)
    } else {
        // TODO: Handle this error
        let args = serde_json::from_str(&args).expect("TODO");
        let job_result = job.perform(&args, &con);

        match job_result {
            Ok(()) => PerformJobOutput::JobPerformed,
            Err(_) => {
                con.retry(job.name(), &args, retry_count)
                    .expect("Failed to enqueue job into retry queue");
                PerformJobOutput::JobRetried
            }
        }
    }
}

struct MultiplexChannel<T> {
    senders: Vec<Sender<T>>,
}

impl<T> MultiplexChannel<T>
where
    T: Send + Clone,
{
    fn new() -> Self {
        MultiplexChannel { senders: vec![] }
    }

    fn new_receiver(&mut self) -> Receiver<T> {
        let (send, recv) = channel();
        self.senders.push(send);
        recv
    }

    fn send(&self, t: T) -> Vec<Result<(), SendError<T>>> {
        self.senders
            .iter()
            .map(|sender| sender.send(t.clone()))
            .collect()
    }
}

trait Times {
    fn times(self) -> Box<Iterator<Item = Self>>;
}

impl Times for usize {
    /// Repeat something `self` times. Inspired by Ruby `n.times { ... }`.
    fn times(self) -> Box<Iterator<Item = Self>> {
        Box::new((0..self).into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_times() {
        let v = 3.times().map(|i| i).collect::<Vec<_>>();
        assert_eq!(vec![0, 1, 2], v);
    }

    #[test]
    fn test_multiplex_channel() {
        let mut channel: MultiplexChannel<()> = MultiplexChannel::new();

        let receivers = (0..3)
            .into_iter()
            .map(|_| channel.new_receiver())
            .collect::<Vec<_>>();

        channel.send(());

        for receiver in receivers {
            let value = receiver.recv();
            assert_eq!(value.unwrap(), ());
        }
    }
}

use connection::*;
use job::*;
use queue_adapters::{DequeueTimeout, NoJobDequeued, QueueIdentifier, RetryCount};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::sync::Arc;
use config::Config;
use ticker::*;
use serde_json;

/// Boot the worker.
///
/// **NOTE:** You normally wouldn't need to call this. Instead use the
/// [`robin_boot_worker!`](../macro.robin_boot_worker.html) macro in the `macros` module.
///
/// This will spawn the numbers of threads set by [`config.worker_count`](../config/struct.Config.html#structfield.worker_count). Each thread
/// will dequeue a job, perform it, and repeat.
///
/// Make sure the config you're using here is the same config you use to establish the connection
/// in [`robin_establish_connection!`](../macro.robin_establish_connection.html).
///
/// This will also print some metrics every few seconds. The output look like "Robin worker metric: Jobs per second 11000".
pub fn boot<T>(config: &Config, lookup_job: T)
where
    T: 'static,
    T: LookupJob,
    T: Send + Clone,
{
    let worker_count = config.worker_count;

    let mut handles = vec![];

    let ticker = Arc::new(Ticker::new());
    spawn_metrics_printer(Arc::clone(&ticker));

    for i in 0..worker_count {
        let worker_number = WorkerNumber {
            number: i + 1,
            total_worker_count: worker_count,
        };

        println!(
            "Robin worker {}/{} started",
            worker_number.number, worker_count
        );

        handles.push(spawn_worker(
            worker_number,
            config.clone(),
            lookup_job.clone(),
            Arc::clone(&ticker),
        ));
    }

    for handle in handles {
        handle.join().expect("failed to end worker thread");
    }
}

fn spawn_metrics_printer(ticker: Arc<Ticker>) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(5));
        let jobs_per_second = ticker.ticks_per_second();
        println!("Robin worker metric: Jobs per second {}", jobs_per_second);

        if ticker.elapsed() > Duration::from_secs(10) {
            println!("Resetting worker ticker");
            ticker.reset();
        }
    });
}

fn spawn_worker<T>(
    worker_number: WorkerNumber,
    config: Config,
    lookup_job: T,
    ticker: Arc<Ticker>,
) -> JoinHandle<()>
where
    T: 'static + LookupJob + Send,
{
    thread::spawn(move || {
        let con = establish(config, lookup_job).expect("failed to establish connection");

        loop {
            let result = perform_job(
                &con,
                QueueIdentifier::Main,
                worker_number,
                Arc::clone(&ticker),
            );

            match result {
                LoopControl::Break => break,
                LoopControl::Continue => {}
            }
        }
    })
}

fn perform_job(
    con: &WorkerConnection,
    iden: QueueIdentifier,
    worker_number: WorkerNumber,
    ticker: Arc<Ticker>,
) -> LoopControl {
    let dequeued = con.dequeue_from(iden, DequeueTimeout(con.config().timeout));

    match dequeued {
        Ok((job, args, retry_count)) => {
            let prev_count = retry_count;
            let retry_count = prev_count.increment();

            if retry_count.limit_reached(con.config()) {
                println!(
                    "Not retrying {} anymore. Retry count was {:?}",
                    job.name().0,
                    prev_count,
                );

                if con.config().repeat_on_timeout {
                    LoopControl::Continue
                } else {
                    LoopControl::Break
                }
            } else {
                match iden {
                    QueueIdentifier::Main => println!(
                        "Performing {} on worker {}",
                        job.name().0,
                        worker_number.description()
                    ),
                    QueueIdentifier::Retry => println!(
                        "Retying {} on worker {}. Retry count is {:?}",
                        job.name().0,
                        worker_number.description(),
                        retry_count
                    ),
                };

                perform_or_retry(con, job, &args, retry_count, worker_number, ticker);

                LoopControl::Continue
            }
        }

        Err(NoJobDequeued::BecauseTimeout) => match iden {
            QueueIdentifier::Main => {
                perform_job(con, QueueIdentifier::Retry, worker_number, ticker)
            }
            QueueIdentifier::Retry => {
                if con.config().repeat_on_timeout {
                    LoopControl::Continue
                } else {
                    LoopControl::Break
                }
            }
        },

        Err(NoJobDequeued::BecauseError(err)) => {
            panic!(format!("Failed to dequeue job with error\n{:?}", err));
        }
    }
}

enum LoopControl {
    Break,
    Continue,
}

fn perform_or_retry(
    con: &WorkerConnection,
    job: Box<Job + Send>,
    args: &str,
    retry_count: RetryCount,
    worker_number: WorkerNumber,
    ticker: Arc<Ticker>,
) {
    let args = serde_json::from_str(args).expect("todo");
    let job_result = job.perform(&args, &con);
    ticker.tick();

    match job_result {
        Ok(()) => println!(
            "Performed {} on worker {}",
            job.name().0,
            worker_number.description()
        ),
        Err(_) => {
            con.retry(job.name(), &args, retry_count)
                .expect("Failed to enqueue job into retry queue");
        }
    }
}

#[derive(Copy, Clone)]
struct WorkerNumber {
    number: usize,
    total_worker_count: usize,
}

impl WorkerNumber {
    fn description(&self) -> String {
        format!("{}/{}", self.number, self.total_worker_count)
    }
}

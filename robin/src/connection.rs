use config::Config;
use error::*;
use job::*;
use queue_adapters::{DequeueTimeout, EnqueuedJob, JobQueue, NoJobDequeued, QueueIdentifier,
                     RetryCount, redis_queue::RedisQueue};

/// Create a new connection.
///
/// **NOTE:** You normally wouldn't need to call this. Instead use the
/// [`robin_establish_connection!`](../macro.robin_establish_connection.html) macro in the `macros` module.
///
/// The lookup function is necessary for parsing the `String` we get from Redis
/// into a job type.
///
/// Make sure the config you're using here is the same config you use to boot the worker in
/// [`robin_boot_worker!`](../macro.robin_boot_worker.html).
pub fn establish<L, Q, K>(
    config: Config,
    queue_config: K,
    lookup_job: L,
) -> RobinResult<Connection<Q>>
where
    L: 'static + LookupJob<Q>,
    Q: JobQueue<Config = K>,
{
    JobQueue::new(&queue_config).map(|queue| Connection {
        queue: queue,
        config: config,
        lookup_job: Box::new(lookup_job),
    })
}

/// The connection to the queue backend. Required to enqueue and dequeue jobs.
#[allow(missing_debug_implementations)]
pub struct Connection<Q> {
    config: Config,
    queue: Q,
    lookup_job: Box<LookupJob<Q>>,
}

/// A connection that uses Redis as backend.
pub type RedisConnection = Connection<RedisQueue>;

impl<Q> Connection<Q>
where
    Q: JobQueue,
{
    /// Returns the connections config
    pub fn config(&self) -> &Config {
        &self.config
    }

    #[doc(hidden)]
    pub fn enqueue_to(
        &self,
        iden: QueueIdentifier,
        name: JobName,
        args: &Args,
        retry_count: RetryCount,
    ) -> RobinResult<()> {
        let enq_job = EnqueuedJob::build()
            .name(name.0.clone())
            .args(args.to_json()?)
            .retry_count(retry_count)
            .done();

        match iden {
            QueueIdentifier::Main => {
                debug!("Enqueued \"{}\" with {}", name.0, args.json());
                self.queue.enqueue(enq_job, iden)
            }
            QueueIdentifier::Retry => self.queue.enqueue(enq_job, iden),
        }
    }

    #[doc(hidden)]
    pub fn retry(&self, name: JobName, args: &Args, retry_count: RetryCount) -> RobinResult<()> {
        self.enqueue_to(QueueIdentifier::Retry, name, args, retry_count)
    }

    #[doc(hidden)]
    pub fn dequeue_from<'a>(
        &'a self,
        iden: QueueIdentifier,
        timeout: DequeueTimeout,
    ) -> Result<(Box<Job<Q> + Send>, String, RetryCount), NoJobDequeued> {
        let enq_job = self.queue.dequeue(&timeout, iden)?;

        let args = enq_job.args().to_string();
        let name = enq_job.name().to_string();

        let job = self.lookup_job(&JobName::from(name.clone()))
            .ok_or_else(move || Error::UnknownJob(name))
            .map_err(NoJobDequeued::from)?;

        Ok((job, args, enq_job.retry_count().clone()))
    }

    /// Delete all jobs from all queues
    pub fn delete_all(&self) -> RobinResult<()> {
        for iden in QueueIdentifier::all_variants() {
            self.queue.delete_all(iden)?;
        }
        Ok(())
    }

    /// The number of jobs in the queue
    pub fn size(&self, iden: QueueIdentifier) -> RobinResult<usize> {
        self.queue.size(iden)
    }

    /// The number of jobs in the main queue
    pub fn main_queue_size(&self) -> RobinResult<usize> {
        self.size(QueueIdentifier::Main)
    }

    /// The number of jobs in the retry queue
    pub fn retry_queue_size(&self) -> RobinResult<usize> {
        self.size(QueueIdentifier::Retry)
    }

    /// `true` if there are 0 jobs in the queue, `false` otherwise
    pub fn is_queue_empty(&self, iden: QueueIdentifier) -> RobinResult<bool> {
        self.queue.size(iden).map(|size| size == 0)
    }

    fn lookup_job(&self, name: &JobName) -> Option<Box<Job<Q> + Send>> {
        self.lookup_job.lookup(name)
    }
}

/// Trait that maps a `String` given to Robin by Redis to an actual job type.
pub trait LookupJob<Q> {
    /// Perform the lookup.
    fn lookup(&self, name: &JobName) -> Option<Box<Job<Q> + Send>>;
}

impl<F, Q> LookupJob<Q> for F
where
    F: Clone,
    F: Fn(&JobName) -> Option<Box<Job<Q> + Send>>,
    Q: JobQueue,
{
    fn lookup(&self, name: &JobName) -> Option<Box<Job<Q> + Send>> {
        self(name)
    }
}

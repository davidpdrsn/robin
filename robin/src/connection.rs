use config::Config;
use error::*;
use job::*;
use queue_adapters::{redis_queue::RedisQueue, EnqueuedJob, JobQueue, NoJobDequeued,
                     QueueIdentifier, RetryCount};

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
    JobQueue::new(&queue_config)
        .map(|(main_queue, retry_queue)| Connection {
            main_queue: main_queue,
            retry_queue: retry_queue,
            config: config,
            lookup_job: Box::new(lookup_job),
        })
        .map_err(Error::from)
}

/// The connection to the queue backend. Required to enqueue and dequeue jobs.
#[allow(missing_debug_implementations)]
pub struct Connection<Q> {
    config: Config,
    main_queue: Q,
    retry_queue: Q,
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
                self.main_queue.enqueue(enq_job).map_err(Error::from)
            }
            QueueIdentifier::Retry => {
                debug!("Re-enqueued \"{}\" with {}", name.0, args.json());
                self.retry_queue.enqueue(enq_job).map_err(Error::from)
            }
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
    ) -> Result<(Box<Job<Q> + Send>, String, RetryCount), NoJobDequeued> {
        let enq_job = match iden {
            QueueIdentifier::Main => self.main_queue.dequeue(),
            QueueIdentifier::Retry => self.retry_queue.dequeue(),
        }?;

        let args = enq_job.args().to_string();
        let name = enq_job.name().to_string();

        let job = self.lookup_job(&JobName::from(name.clone()))
            .ok_or_else(move || NoJobDequeued::BecauseUnknownJob(JobName(name)))?;

        Ok((job, args, enq_job.retry_count().clone()))
    }

    /// Delete all jobs from main queue
    pub fn delete_all_from_main(&self) -> RobinResult<()> {
        self.main_queue.delete_all()?;
        Ok(())
    }

    /// Delete all jobs from retry queue
    pub fn delete_all_from_retry(&self) -> RobinResult<()> {
        self.retry_queue.delete_all()?;
        Ok(())
    }

    /// Delete all jobs from all queues
    pub fn delete_all(&self) -> RobinResult<()> {
        self.delete_all_from_main()?;
        self.delete_all_from_retry()?;
        Ok(())
    }

    /// The number of jobs in the main queue
    pub fn main_queue_size(&self) -> RobinResult<usize> {
        self.size(QueueIdentifier::Main).map_err(Error::from)
    }

    /// The number of jobs in the main queue
    pub fn retry_queue_size(&self) -> RobinResult<usize> {
        self.size(QueueIdentifier::Retry).map_err(Error::from)
    }

    /// `true` if there are 0 jobs in the main queue, `false` otherwise
    pub fn is_main_queue_empty(&self) -> RobinResult<bool> {
        self.main_queue
            .size()
            .map_err(Error::from)
            .map(|size| size == 0)
    }

    /// `true` if there are 0 jobs in the retry queue, `false` otherwise
    pub fn is_retry_queue_empty(&self) -> RobinResult<bool> {
        self.retry_queue
            .size()
            .map_err(Error::from)
            .map(|size| size == 0)
    }

    fn lookup_job(&self, name: &JobName) -> Option<Box<Job<Q> + Send>> {
        self.lookup_job.lookup(name)
    }

    /// The number of jobs in the queue
    fn size(&self, iden: QueueIdentifier) -> RobinResult<usize> {
        match iden {
            QueueIdentifier::Main => self.main_queue.size(),
            QueueIdentifier::Retry => self.retry_queue.size(),
        }.map_err(Error::from)
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

pub mod queue_adapters;

use config::Config;
use error::*;
use job::*;
use self::queue_adapters::{DequeueTimeout, EnqueuedJob, NoJobDequeued, QueueIdentifier,
                           RetryCount, redis_queue::RedisQueue};
use std::fmt;

pub struct WorkerConnection {
    queue: RedisQueue,
    pub config: Config,
    lookup_job: Box<LookupJob>,
}

impl fmt::Debug for WorkerConnection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WorkerConnection {{ config: {:?} }}", self.config)
    }
}

impl WorkerConnection {
    pub fn enqueue_to(
        &self,
        iden: QueueIdentifier,
        name: JobName,
        args: &Args,
        retry_count: RetryCount,
    ) -> RobinResult<()> {
        let enq_job = EnqueuedJob {
            name: name.0.to_string(),
            args: args.to_json().expect("todo"),
            retry_count: retry_count,
        };

        match iden {
            QueueIdentifier::Main => {
                println!("Enqueued \"{}\" with {}", name.0, args.json);
                self.queue.enqueue(enq_job, iden)
            }
            QueueIdentifier::Retry => self.queue.enqueue(enq_job, iden),
        }
    }

    pub fn retry(&self, name: JobName, args: &Args, retry_count: RetryCount) -> RobinResult<()> {
        self.enqueue_to(QueueIdentifier::Retry, name, args, retry_count)
    }

    pub fn dequeue_from<'a>(
        &'a self,
        iden: QueueIdentifier,
        timeout: DequeueTimeout,
    ) -> Result<(Box<Job + Send>, String, RetryCount), NoJobDequeued> {
        let enq_job = self.queue.dequeue(&timeout, iden)?;

        let args = enq_job.args;
        let name = enq_job.name;

        let job = self.lookup_job(&JobName::from(name.clone()))
            .ok_or_else(move || Error::UnknownJob(name))
            .map_err(NoJobDequeued::from)?;

        Ok((job, args, enq_job.retry_count))
    }

    pub fn delete_all(&self) -> RobinResult<()> {
        for iden in QueueIdentifier::all_variants() {
            self.queue.delete_all(iden)?;
        }
        Ok(())
    }

    pub fn size(&self, iden: QueueIdentifier) -> RobinResult<usize> {
        self.queue.size(iden)
    }

    pub fn is_queue_empty(&self, iden: QueueIdentifier) -> RobinResult<bool> {
        self.queue.size(iden).map(|size| size == 0)
    }

    fn lookup_job(&self, name: &JobName) -> Option<Box<Job + Send>> {
        self.lookup_job.lookup(name)
    }
}

pub fn establish<T: 'static + LookupJob>(
    config: Config,
    lookup_job: T,
) -> RobinResult<WorkerConnection> {
    RedisQueue::new_with_namespace(&config.redis_namespace).map(|redis_queue| WorkerConnection {
        queue: redis_queue,
        config: config,
        lookup_job: Box::new(lookup_job),
    })
}

pub trait LookupJob {
    fn lookup(&self, name: &JobName) -> Option<Box<Job + Send>>;
}

impl<F> LookupJob for F
where
    F: Clone,
    F: Fn(&JobName) -> Option<Box<Job + Send>>,
{
    fn lookup(&self, name: &JobName) -> Option<Box<Job + Send>> {
        self(name)
    }
}

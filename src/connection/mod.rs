pub mod queue_adapters;

use config::Config;
use error::*;
use job::*;
use self::queue_adapters::redis_queue::RedisQueue;
use self::queue_adapters::{DequeueTimeout, EnqueuedJob, NoJobDequeued, QueueIdentifier, RetryCount};
use std::collections::HashMap;

pub struct WorkerConnection {
    queue: RedisQueue,
    jobs: HashMap<JobName, Box<Job + Send>>,
    pub config: Config,
}

impl WorkerConnection {
    pub fn register<T>(&mut self, job: T) -> RobinResult<()>
    where
        T: 'static + Job + Send,
    {
        let name = job.name();

        if self.jobs.contains_key(&name) {
            Err(Error::JobAlreadyRegistered(name))
        } else {
            self.jobs.insert(name, Box::new(job));
            Ok(())
        }
    }

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
    ) -> Result<(&'a Box<Job + Send>, String, RetryCount), NoJobDequeued> {
        let enq_job = self.queue.dequeue(&timeout, iden)?;

        let args = enq_job.args;
        let name = enq_job.name;

        let job = self.jobs
            .get(&JobName::from(name.clone()))
            .ok_or_else(move || Error::JobNotRegistered(name))
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
}

pub fn establish(config: Config) -> RobinResult<WorkerConnection> {
    RedisQueue::new_with_namespace(&config.redis_namespace).map(|redis_queue| WorkerConnection {
        queue: redis_queue,
        jobs: HashMap::new(),
        config: config,
    })
}

pub trait ConnectionProducer {
    fn new_connection(&self) -> RobinResult<WorkerConnection>;
}

impl<T> ConnectionProducer for T
where
    T: Fn() -> RobinResult<WorkerConnection>,
{
    fn new_connection(&self) -> RobinResult<WorkerConnection> {
        self()
    }
}

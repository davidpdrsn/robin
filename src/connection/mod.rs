pub mod queue_adapters;

use error::*;
use job::*;
use config::Config;
use self::queue_adapters::redis_queue::RedisQueue;
use self::queue_adapters::{NoJobDequeued, EnqueuedJob, DequeueTimeout, RetryCount};
use std::collections::HashMap;

pub struct WorkerConnection {
    main_queue: RedisQueue,
    retry_queue: RedisQueue,
    jobs: HashMap<JobName, Box<Job>>,
    pub config: Config,
}

impl WorkerConnection {
    pub fn register<T>(&mut self, job: T) -> RobinResult<()>
    where
        T: 'static + Job,
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
        args: &str,
        retry_count: RetryCount,
    ) -> RobinResult<()> {
        let enq_job = EnqueuedJob {
            name: name.0.to_string(),
            args: args.to_string(),
            retry_count: retry_count,
        };

        match iden {
            QueueIdentifier::Main => {
                println!("Enqueued \"{}\" with {}", name.0, args);
                self.main_queue.enqueue(enq_job)
            }
            QueueIdentifier::Retry => self.retry_queue.enqueue(enq_job),
        }
    }

    pub fn retry(&self, name: JobName, args: &str, retry_count: RetryCount) -> RobinResult<()> {
        self.enqueue_to(QueueIdentifier::Retry, name, args, retry_count)
    }

    pub fn dequeue_from<'a>(
        &'a self,
        iden: QueueIdentifier,
        timeout: DequeueTimeout,
    ) -> Result<(&'a Box<Job>, String, RetryCount), NoJobDequeued> {
        let enq_job = match iden {
            QueueIdentifier::Main => self.main_queue.dequeue(&timeout),
            QueueIdentifier::Retry => self.retry_queue.dequeue(&timeout),
        }?;

        let args = enq_job.args;
        let name = enq_job.name;

        let job = self.jobs
            .get(&JobName::from(name.clone()))
            .ok_or_else(move || Error::JobNotRegistered(name))
            .map_err(NoJobDequeued::from)?;

        Ok((job, args, enq_job.retry_count))
    }

    pub fn delete_all(&self) -> RobinResult<()> {
        self.main_queue.delete_all().and_then(|_| {
            self.retry_queue.delete_all()
        })
    }

    pub fn size(&self, iden: QueueIdentifier) -> RobinResult<usize> {
        match iden {
            QueueIdentifier::Main => self.main_queue.size(),
            QueueIdentifier::Retry => self.retry_queue.size(),
        }
    }

    pub fn is_queue_empty(&self, iden: QueueIdentifier) -> RobinResult<bool> {
        match iden {
            QueueIdentifier::Main => self.main_queue.size().map(|size| size == 0),
            QueueIdentifier::Retry => self.retry_queue.size().map(|size| size == 0),
        }
    }
}

pub fn establish(config: Config) -> RobinResult<WorkerConnection> {
    // I would like one >>= pls
    RedisQueue::new_with_namespace(&QueueIdentifier::Main.redis_queue_name(
        &config.redis_namespace,
    )).and_then(|main_redis_queue| {
        RedisQueue::new_with_namespace(&QueueIdentifier::Retry.redis_queue_name(
            &config.redis_namespace,
        )).map(|retry_redis_queue| {
            WorkerConnection {
                main_queue: main_redis_queue,
                retry_queue: retry_redis_queue,
                jobs: HashMap::new(),
                config: config,
            }
        })
    })
}

#[derive(Copy, Clone)]
pub enum QueueIdentifier {
    Main,
    Retry,
}

impl QueueIdentifier {
    fn redis_queue_name(&self, namespace: &str) -> String {
        match *self {
            QueueIdentifier::Main => format!("main_{}", namespace),
            QueueIdentifier::Retry => format!("retry_{}", namespace),
        }
    }
}

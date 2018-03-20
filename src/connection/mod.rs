mod queue_adapters;

use error::*;
use job::*;
use worker::Config;
use self::queue_adapters::redis_queue::RedisQueue;
use std::collections::HashMap;

pub struct WorkerConnection {
    queue: RedisQueue,
    jobs: HashMap<JobName, Box<Job>>,
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

    pub fn enqueue(&self, name: JobName, args: &str) -> RobinResult<()> {
        println!("Enqueued \"{}\" with {}", name.0, args);
        self.queue.enqueue(name, args)
    }

    pub fn dequeue<'a>(&'a self, config: &Config) -> RobinResult<(&'a Box<Job>, String)> {
        self.queue.dequeue(&self.jobs, config)
    }
}

pub fn establish() -> RobinResult<WorkerConnection> {
    RedisQueue::new().map(|redis| {
        WorkerConnection {
            queue: redis,
            jobs: HashMap::new(),
        }
    })
}

use error::*;
use job::*;
use worker::Config;
use redis::{Client, Commands};
use serde_json;
use std::collections::HashMap;
use super::EnqueuedJob;
use redis;

pub struct RedisQueue {
    redis: redis::Connection,
    key: String,
}

impl RedisQueue {
    pub fn new() -> RobinResult<Self> {
        let client = Client::open("redis://127.0.0.1/")?;
        let con = client.get_connection()?;
        Ok(RedisQueue {
            redis: con,
            key: "__robin_queue__".to_string(),
        })
    }

    pub fn enqueue(&self, name: JobName, args: &str) -> RobinResult<()> {
        let enq_job = EnqueuedJob {
            name: name.0.to_string(),
            args: args.to_string(),
        };
        let data: String = json!(enq_job).to_string();
        let _: () = self.redis.rpush(&self.key, data)?;

        Ok(())
    }

    pub fn dequeue<'a>(
        &self,
        jobs: &'a HashMap<JobName, Box<Job>>,
        config: &Config,
    ) -> RobinResult<(&'a Box<Job>, String)> {
        let timeout_in_seconds = config.timeout;
        let bulk: Vec<redis::Value> = self.redis.blpop(&self.key, timeout_in_seconds)?;

        match bulk.get(1) {
            Some(&redis::Value::Data(ref data)) => {
                let data = String::from_utf8(data.to_vec()).unwrap();
                let enq_job: EnqueuedJob = serde_json::from_str(&data).map_err(Error::from)?;

                let args = enq_job.args;
                let job = jobs.get(&JobName::from(enq_job.name.clone())).ok_or(
                    Error::JobNotRegistered(enq_job.name.clone()),
                )?;

                Ok((job, args))
            }

            // we hit the timeout
            None => self.dequeue(jobs, config),

            _ => Err(Error::UnknownRedisError(
                "List didn't hold what we were expecting".to_string(),
            )),
        }
    }
}

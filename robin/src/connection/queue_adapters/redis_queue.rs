use error::*;
use redis::{Client, Commands};
use serde_json;
use super::{DequeueTimeout, EnqueuedJob, NoJobDequeued, QueueIdentifier};
use redis;
use std::fmt;
use config::*;

/// A wrapper around an actual `redis::Connection`.
pub struct RedisQueue {
    redis: redis::Connection,
    redis_url: String,
    key: String,
}

impl RedisQueue {
    /// Create a new `RedisQueue` using the given config
    pub fn new(config: &Config) -> RobinResult<Self> {
        let redis_url = config.redis_url.to_string();
        let client = Client::open(redis_url.as_ref())?;
        let con = client.get_connection()?;
        Ok(RedisQueue {
            redis: con,
            redis_url: redis_url.to_string(),
            key: config.redis_namespace.to_string(),
        })
    }

    /// Put a job into a queue
    pub fn enqueue(&self, enq_job: EnqueuedJob, iden: QueueIdentifier) -> RobinResult<()> {
        let data: String = json!(enq_job).to_string();
        let _: () = self.redis.rpush(&self.key(iden), data)?;

        Ok(())
    }

    /// Pull a job out of the queue. This will block for `timeout` seconds if the queue is empty.
    pub fn dequeue<'a>(
        &self,
        timeout: &DequeueTimeout,
        iden: QueueIdentifier,
    ) -> Result<EnqueuedJob, NoJobDequeued> {
        let timeout_in_seconds = timeout.0;
        let bulk: Vec<redis::Value> = self.redis.blpop(&self.key(iden), timeout_in_seconds)?;

        match bulk.get(1) {
            Some(&redis::Value::Data(ref data)) => {
                let data =
                    String::from_utf8(data.to_vec()).expect("Didn't get valid UTF-8 from Redis");
                serde_json::from_str(&data).map_err(NoJobDequeued::from)
            }

            None => Err(NoJobDequeued::BecauseTimeout),

            _ => Err(NoJobDequeued::from(Error::UnknownRedisError(
                "List didn't contain what we were expecting".to_string(),
            ))),
        }
    }

    /// Delete everything in the queue.
    pub fn delete_all(&self, iden: QueueIdentifier) -> RobinResult<()> {
        let _: () = self.redis.del(&self.key(iden))?;
        Ok(())
    }

    /// The number of jobs in the queue.
    pub fn size(&self, iden: QueueIdentifier) -> RobinResult<usize> {
        let size: usize = self.redis.llen(&self.key(iden)).map_err(Error::from)?;
        Ok(size)
    }

    fn key(&self, iden: QueueIdentifier) -> String {
        format!("{}_{}", self.key, iden.redis_queue_name())
    }
}

impl fmt::Debug for RedisQueue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "RedisQueue {{ key: {:?}, redis_url: {:?} }}",
            self.key, self.redis_url
        )
    }
}

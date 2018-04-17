use serde_json;
use super::*;
use std::fmt;
use std::default::Default;
use redis;
use redis::{Client, Commands};

/// A queue backend the persists the jobs in Redis.
pub struct RedisQueue {
    redis: redis::Connection,
    redis_url: String,
    key: String,
    timeout: usize,
}

impl RedisQueue {
    fn key(&self, iden: QueueIdentifier) -> String {
        format!("{}_{}", self.key, iden.redis_queue_name())
    }
}

/// The arguments required to create a new `RedisQueue`
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RedisConfig {
    /// The URL used to connect to Redis.
    ///
    /// Default is "redis://127.0.0.1/"
    pub url: String,

    /// The key that will be prepended all Robin related Redis keys. Effectively working as a
    /// namespace.
    pub namespace: String,

    /// The number of seconds the worker will block while waiting for a new job to be enqueued.
    pub timeout: usize,
}

impl Default for RedisConfig {
    fn default() -> RedisConfig {
        RedisConfig {
            timeout: 30,
            namespace: "robin_".to_string(),
            url: "redis://127.0.0.1/".to_string(),
        }
    }
}

impl JobQueue for RedisQueue {
    type Config = RedisConfig;

    /// Create a new `RedisQueue` using the given config
    fn new(init: &RedisConfig) -> JobQueueResult<Self> {
        let client = Client::open(init.url.as_ref()).map_err(|e| (e, ErrorOrigin::Initialization))?;

        let con = client
            .get_connection()
            .map_err(|e| (e, ErrorOrigin::Initialization))?;

        Ok(RedisQueue {
            redis: con,
            redis_url: init.url.to_string(),
            key: init.namespace.to_string(),
            timeout: init.timeout,
        })
    }

    /// Put a job into a queue
    fn enqueue(&self, enq_job: EnqueuedJob, iden: QueueIdentifier) -> JobQueueResult<()> {
        let data: String = json!(enq_job).to_string();
        let _: () = self.redis
            .rpush(&self.key(iden), data)
            .map_err(|e| (e, ErrorOrigin::Enqueue))?;

        Ok(())
    }

    /// Pull a job out of the queue. This will block for `timeout` seconds if the queue is empty.
    fn dequeue(&self, iden: QueueIdentifier) -> Result<EnqueuedJob, NoJobDequeued> {
        let timeout_in_seconds = self.timeout;
        let bulk: Vec<redis::Value> = self.redis
            .blpop(&self.key(iden), timeout_in_seconds)
            .map_err(|e| NoJobDequeued::from((e, ErrorOrigin::Dequeue)))?;

        match bulk.get(1) {
            Some(&redis::Value::Data(ref data)) => {
                let data =
                    String::from_utf8(data.to_vec()).expect("Didn't get valid UTF-8 from Redis");
                serde_json::from_str(&data)
                    .map_err(|e| NoJobDequeued::from((e, ErrorOrigin::Dequeue)))
            }

            None => Err(NoJobDequeued::BecauseTimeout),

            _ => panic!("TODO"),
        }
    }

    /// Delete everything in the queue.
    fn delete_all(&self, iden: QueueIdentifier) -> JobQueueResult<()> {
        let _: () = self.redis
            .del(&self.key(iden))
            .map_err(|e| (e, ErrorOrigin::DeleteAll))?;
        Ok(())
    }

    /// The number of jobs in the queue.
    fn size(&self, iden: QueueIdentifier) -> JobQueueResult<usize> {
        let size: usize = self.redis
            .llen(&self.key(iden))
            .map_err(|e| (e, ErrorOrigin::Size))?;
        Ok(size)
    }
}

impl Debug for RedisQueue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "RedisQueue {{ key: {:?}, redis_url: {:?} }}",
            self.key, self.redis_url
        )
    }
}

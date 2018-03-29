use error::*;
use redis::{Client, Commands};
use serde_json;
use super::{DequeueTimeout, EnqueuedJob, NoJobDequeued};
use redis;

pub struct RedisQueue {
    redis: redis::Connection,
    key: String,
}

impl RedisQueue {
    pub fn new_with_namespace(name: &str) -> RobinResult<Self> {
        let client = Client::open("redis://127.0.0.1/")?;
        let con = client.get_connection()?;
        Ok(RedisQueue {
            redis: con,
            key: format!("__{}__", name),
        })
    }

    pub fn enqueue(&self, enq_job: EnqueuedJob) -> RobinResult<()> {
        let data: String = json!(enq_job).to_string();
        let _: () = self.redis.rpush(&self.key, data)?;

        Ok(())
    }

    pub fn dequeue<'a>(&self, timeout: &DequeueTimeout) -> Result<EnqueuedJob, NoJobDequeued> {
        let timeout_in_seconds = timeout.0;
        let bulk: Vec<redis::Value> = self.redis.blpop(&self.key, timeout_in_seconds)?;

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

    pub fn delete_all(&self) -> RobinResult<()> {
        let _: () = self.redis.del(&self.key)?;
        Ok(())
    }

    pub fn size(&self) -> RobinResult<usize> {
        let size: usize = self.redis.llen(&self.key).map_err(Error::from)?;
        Ok(size)
    }
}

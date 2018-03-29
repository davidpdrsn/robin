use error::*;
use redis::{Client, Commands};
use serde_json;
use super::{DequeueTimeout, EnqueuedJob, NoJobDequeued, QueueIdentifier};
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
            key: name.to_string(),
        })
    }

    pub fn enqueue(&self, enq_job: EnqueuedJob, iden: QueueIdentifier) -> RobinResult<()> {
        let data: String = json!(enq_job).to_string();
        let _: () = self.redis.rpush(&self.key(iden), data)?;

        Ok(())
    }

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

    pub fn delete_all(&self, iden: QueueIdentifier) -> RobinResult<()> {
        let _: () = self.redis.del(&self.key(iden))?;
        Ok(())
    }

    pub fn size(&self, iden: QueueIdentifier) -> RobinResult<usize> {
        let size: usize = self.redis.llen(&self.key(iden)).map_err(Error::from)?;
        Ok(size)
    }

    fn key(&self, iden: QueueIdentifier) -> String {
        format!("{}_{}", self.key, iden.redis_queue_name())
    }
}

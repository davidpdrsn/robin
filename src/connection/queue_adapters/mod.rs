pub mod redis_queue;

use serde_json;
use redis;
use error::*;
use config::Config;

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
pub enum RetryCount {
    NeverRetried,
    Count(u32),
}

impl RetryCount {
    pub fn increment(&self) -> RetryCount {
        match *self {
            RetryCount::NeverRetried => RetryCount::Count(1),
            RetryCount::Count(n) => RetryCount::Count(n + 1),
        }
    }

    pub fn limit_reached(&self, config: &Config) -> bool {
        match *self {
            RetryCount::NeverRetried => false,
            RetryCount::Count(n) => n > config.retry_count_limit,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EnqueuedJob {
    pub name: String,
    pub args: String,
    pub retry_count: RetryCount,
}

#[derive(Debug)]
pub enum NoJobDequeued {
    BecauseTimeout,
    BecauseError(Error),
}

impl From<redis::RedisError> for NoJobDequeued {
    fn from(error: redis::RedisError) -> NoJobDequeued {
        NoJobDequeued::BecauseError(Error::from(error))
    }
}

impl From<serde_json::Error> for NoJobDequeued {
    fn from(error: serde_json::Error) -> NoJobDequeued {
        NoJobDequeued::BecauseError(Error::from(error))
    }
}

impl From<Error> for NoJobDequeued {
    fn from(error: Error) -> NoJobDequeued {
        NoJobDequeued::BecauseError(error)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DequeueTimeout(pub usize);

#[derive(Debug, Copy, Clone)]
pub enum QueueIdentifier {
    Main,
    Retry,
}

impl QueueIdentifier {
    pub fn redis_queue_name(&self, namespace: &str) -> String {
        match *self {
            QueueIdentifier::Main => format!("main_{}", namespace),
            QueueIdentifier::Retry => format!("retry_{}", namespace),
        }
    }
}

use serde_json;
use redis;
use std::{error, fmt};
use job::JobName;
use queue_adapters::JobQueueError;

/// The result type used throughout Robin.
pub type RobinResult<T> = Result<T, Error>;

/// The different types of errors that might happen.
#[derive(Debug)]
pub enum Error {
    /// The job we got from the queue isn't known.
    UnknownJob(JobName),

    /// The job failed to perform and might be retried.
    JobFailed(Box<error::Error>),

    /// The job failed to perform and might be retried.
    JobQueueError(JobQueueError),

    /// Some serialization/deserialization failed
    SerdeError(serde_json::Error),

    /// Something related to Redis failed.
    RedisError(redis::RedisError),

    /// A Redis error that isn't included in `redis::RedisError`
    UnknownRedisError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::UnknownJob(ref name) => &name.0,
            &Error::JobFailed(ref err) => err.description(),
            &Error::JobQueueError(ref err) => err.description(),
            &Error::SerdeError(ref err) => err.description(),
            &Error::RedisError(ref err) => err.description(),
            &Error::UnknownRedisError(ref msg) => msg,
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Error {
        Error::SerdeError(error)
    }
}

impl From<redis::RedisError> for Error {
    fn from(error: redis::RedisError) -> Error {
        Error::RedisError(error)
    }
}

impl From<JobQueueError> for Error {
    fn from(error: JobQueueError) -> Error {
        Error::JobQueueError(error)
    }
}

use serde_json;
use redis;
use std::{self, fmt};

/// The result type used throughout Robin.
pub type RobinResult<T> = Result<T, Error>;

/// The different types of errors that might happen.
#[derive(Debug)]
pub enum Error {
    /// The job we got from the queue isn't known.
    UnknownJob(String),

    /// The job failed to perform and might be retried.
    JobFailed(Box<std::error::Error>),

    /// Some serialization/deserialization failed
    SerdeJsonError(serde_json::Error),

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

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::UnknownJob(ref name) => name,
            &Error::JobFailed(ref err) => err.description(),
            &Error::SerdeJsonError(ref err) => err.description(),
            &Error::RedisError(ref err) => err.description(),
            &Error::UnknownRedisError(ref msg) => msg,
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Error {
        Error::SerdeJsonError(error)
    }
}

impl From<redis::RedisError> for Error {
    fn from(error: redis::RedisError) -> Error {
        Error::RedisError(error)
    }
}

use serde_json;
use redis;
use std::io;

/// The result type used throughout Robin.
pub type RobinResult<T> = Result<T, Error>;

/// The different types of errors that might happen.
#[derive(Debug)]
pub enum Error {
    /// The job we got from the queue isn't known.
    UnknownJob(String),

    /// The job failed to perform and might be retried.
    JobFailed(String),

    /// Some serialization/deserialization failed
    SerdeJsonError(serde_json::Error),

    /// Something related to Redis failed.
    RedisError(redis::RedisError),

    /// A Redis error that isn't included in `redis::RedisError`
    UnknownRedisError(String),
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

impl From<Error> for String {
    fn from(error: Error) -> String {
        match error {
            Error::UnknownJob(name) => format!("{} is unknown", name),
            Error::JobFailed(msg) => format!("Job failed with message: {}", msg),
            Error::SerdeJsonError(err) => format!("Error::SerdeJsonError : {}", err),
            Error::RedisError(err) => format!("Error::RedisError : {}", err),
            Error::UnknownRedisError(err) => format!("Error::UnknownRedisError : {}", err),
        }
    }
}

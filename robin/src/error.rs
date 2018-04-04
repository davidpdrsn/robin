use serde_json;
use redis;
use std::fmt;
use std::error;

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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::UnknownJob(ref name) => name,
            &Error::JobFailed(ref msg) => msg,
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

impl<'a> From<&'a Error> for String {
    fn from(error: &'a Error) -> String {
        match error {
            &Error::UnknownJob(ref name) => format!(
                "The job named {} is unknown and cannot be performed or enqueued",
                name
            ),
            &Error::JobFailed(ref msg) => format!("Job failed with message: {}", msg),

            &Error::SerdeJsonError(ref err) => format!("Error::SerdeJsonError : {}", err),
            &Error::RedisError(ref err) => format!("Error::RedisError : {}", err),
            &Error::UnknownRedisError(ref err) => format!("Error::UnknownRedisError : {}", err),
        }
    }
}

impl From<Error> for String {
    fn from(error: Error) -> String {
        String::from(&error)
    }
}

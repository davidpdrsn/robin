use serde_json;
use redis;
use job::JobName;
use std::io;

pub type RobinResult<T> = Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    JobAlreadyRegistered(JobName),
    JobNotRegistered(String),

    JobFailed(String),

    IoError(io::Error),

    SerdeJsonError(serde_json::Error),

    RedisError(redis::RedisError),
    UnknownRedisError(String),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::IoError(error)
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

impl From<Error> for String {
    fn from(error: Error) -> String {
        match error {
            Error::JobAlreadyRegistered(name) => format!("{:?} already registered", name),
            Error::JobNotRegistered(name) => format!("{} not registered", name),
            Error::JobFailed(msg) => format!("Job failed with message: {}", msg),
            Error::IoError(err) => format!("Error::IoError : {}", err),
            Error::SerdeJsonError(err) => format!("Error::SerdeJsonError : {}", err),
            Error::RedisError(err) => format!("Error::RedisError : {}", err),
            Error::UnknownRedisError(err) => format!("Error::UnknownRedisError : {}", err),
        }
    }
}

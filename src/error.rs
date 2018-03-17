extern crate serde_json;

use job::JobName;
use std::io;

pub type RobinResult<T> = Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ConnectionError,
    JobAlreadyRegistered(JobName),
    IoError(io::Error),
    SerdeJsonError(serde_json::Error),
    EnqueueError(String),
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

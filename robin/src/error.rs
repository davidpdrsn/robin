use queue_adapters::{JobQueueError, JobQueueErrorInformation};
use serde_json;
use std::{error, fmt};

/// The result type used throughout Robin.
pub type RobinResult<T> = Result<T, Error>;

/// The different types of errors that might happen.
#[derive(Debug)]
pub enum Error {
    /// The job failed to perform and might be retried.
    JobFailed(Box<error::Error>),

    /// The job failed to perform and might be retried.
    JobQueueError(JobQueueError),

    /// Some serialization/deserialization failed
    SerdeError(serde_json::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::JobFailed(ref err) => err.description(),
            &Error::JobQueueError(ref err) => err.description(),
            &Error::SerdeError(ref err) => err.description(),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Error {
        Error::SerdeError(error)
    }
}

impl From<Box<JobQueueErrorInformation>> for Error {
    fn from(e: Box<JobQueueErrorInformation>) -> Error {
        Error::JobQueueError(e)
    }
}

use job::JobName;

#[derive(Debug)]
pub enum Error {
    ConnectionError,
    JobAlreadyRegistered(JobName),
}

pub type RobinResult<T> = Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ConnectionError,
    JobAlreadyRegistered(String),
}

pub type RobinResult<T> = Result<T, Error>;

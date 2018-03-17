#[derive(Debug)]
pub enum Error {
    ConnectionError,
}

pub type RobinResult<T> = Result<T, Error>;

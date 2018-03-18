extern crate serde_json;

use serde::{Serialize, Deserialize};
use connection::*;
use error::*;

pub trait Job {
    fn perform(&self, con: &WorkerConnection, args: &str) -> RobinResult<()>;
    fn name(&self) -> JobName;
}

pub trait PerformJob {
    fn perform_now<A: Serialize>(&self, con: &WorkerConnection, args: A) -> RobinResult<()>;
    fn perform_later<A: Serialize>(&self, con: &WorkerConnection, args: A) -> RobinResult<()>;
}

impl<T> PerformJob for T
where
    T: Job,
{
    fn perform_now<A: Serialize>(&self, con: &WorkerConnection, args: A) -> RobinResult<()> {
        self.perform(con, &serialize_arg(args)?)
    }

    fn perform_later<A: Serialize>(&self, con: &WorkerConnection, args: A) -> RobinResult<()> {
        con.enqueue(self.name(), &serialize_arg(args)?)
    }
}

fn serialize_arg<T: Serialize>(value: T) -> RobinResult<String> {
    serde_json::to_string(&value).map_err(Error::from)
}

pub fn deserialize_arg<'a, T: Deserialize<'a>>(args: &'a str) -> RobinResult<T> {
    serde_json::from_str(args).map_err(Error::from)
}

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct JobName(pub String);

impl<T: Into<String>> From<T> for JobName {
    fn from(t: T) -> JobName {
        JobName(t.into())
    }
}

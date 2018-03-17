extern crate serde_json;

use serde::{Serialize, Deserialize};
use connection::*;
use error::*;

pub trait Job {
    fn perform(&self, arg: &str) -> RobinResult<()>;
    fn name(&self) -> JobName;
}

pub trait PerformJob {
    fn perform_now<A: Serialize>(&self, arg: A) -> RobinResult<()>;
    fn perform_later<A: Serialize>(&self, _con: &WorkerConnection, arg: A) -> RobinResult<()>;
}

impl<T> PerformJob for T
where
    T: Job,
{
    fn perform_now<A: Serialize>(&self, arg: A) -> RobinResult<()> {
        self.perform(&serialize_arg(arg)?)
    }

    fn perform_later<A: Serialize>(&self, con: &WorkerConnection, arg: A) -> RobinResult<()> {
        con.enqueue(self.name(), &serialize_arg(arg)?)
    }
}

fn serialize_arg<T: Serialize>(value: T) -> RobinResult<String> {
    serde_json::to_string(&value).map_err(Error::from)
}

pub fn deserialize_arg<'a, T: Deserialize<'a>>(arg: &'a str) -> RobinResult<T> {
    serde_json::from_str(arg).map_err(Error::from)
}

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct JobName(pub String);

impl<T: Into<String>> From<T> for JobName {
    fn from(t: T) -> JobName {
        JobName(t.into())
    }
}

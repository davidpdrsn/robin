extern crate serde_json;

use serde::{Serialize, Deserialize};
use connection::*;
use error::*;
use connection::queue_adapters::RetryCount;

pub type JobResult<'a> = Result<(), String>;

pub trait Job {
    fn perform(&self, con: &WorkerConnection, args: &str) -> JobResult;
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
        let job_result: JobResult = self.perform(con, &serialize_arg(args)?);

        match job_result {
            Ok(_) => Ok(()),
            Err(msg) => Err(Error::JobFailed(msg.to_string())),
        }
    }

    fn perform_later<A: Serialize>(&self, con: &WorkerConnection, args: A) -> RobinResult<()> {
        con.enqueue_to(
            QueueIdentifier::Main,
            self.name(),
            &serialize_arg(args)?,
            RetryCount::NeverRetried,
        )
    }
}

fn serialize_arg<T: Serialize>(value: T) -> RobinResult<String> {
    serde_json::to_string(&value).map_err(Error::from)
}

pub fn deserialize_arg<'a, T: Deserialize<'a>>(args: &'a str) -> RobinResult<T> {
    match serde_json::from_str(args) {
        Ok(v) => Ok(v),
        Err(e) => {
            let msg = format!("Failed deserializing {:?}\nSerde error: {:?}", args, e);
            Err(Error::JobFailed(msg))
        }
    }
}

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct JobName(pub String);

impl<T: Into<String>> From<T> for JobName {
    fn from(t: T) -> JobName {
        JobName(t.into())
    }
}

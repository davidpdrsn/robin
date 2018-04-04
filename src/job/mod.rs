extern crate serde_json;

use serde::{Deserialize, Serialize};
use connection::WorkerConnection;
use connection::queue_adapters::{QueueIdentifier, RetryCount};
use error::{Error, RobinResult};

pub type JobResult<'a> = Result<(), String>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Args {
    pub json: String,
}

impl Args {
    pub fn to_json(&self) -> RobinResult<String> {
        serde_json::to_string(&self).map_err(Error::from)
    }

    pub fn deserialize<'a, T: Deserialize<'a>>(&'a self) -> RobinResult<T> {
        match serde_json::from_str(&self.json) {
            Ok(v) => Ok(v),
            Err(e) => {
                let msg = format!("Failed deserializing {:?}\nSerde error: {:?}", self.json, e);
                Err(Error::JobFailed(msg))
            }
        }
    }
}

pub trait Job {
    fn name(&self) -> JobName;
    fn perform(&self, con: &WorkerConnection, args: &Args) -> JobResult;
}

pub trait PerformJob {
    // TODO: Implement perform_now
    // fn perform_now<A: Serialize>(&self, con: &WorkerConnection, args: A) -> RobinResult<()>;

    fn perform_later<A: Serialize>(&self, con: &WorkerConnection, args: A) -> RobinResult<()>;
}

impl<T> PerformJob for T
where
    T: Job,
{
    fn perform_later<A: Serialize>(&self, con: &WorkerConnection, args: A) -> RobinResult<()> {
        con.enqueue_to(
            QueueIdentifier::Main,
            self.name(),
            &serialize_arg(args)?,
            RetryCount::NeverRetried,
        )
    }
}

fn serialize_arg<T: Serialize>(value: T) -> RobinResult<Args> {
    let json = serde_json::to_string(&value).map_err(Error::from)?;
    Ok(Args { json })
}

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct JobName(pub String);

impl<T: Into<String>> From<T> for JobName {
    fn from(t: T) -> JobName {
        JobName(t.into())
    }
}

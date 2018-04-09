extern crate serde_json;

use serde::{Deserialize, Serialize};
use connection::WorkerConnection;
use queue_adapters::{QueueIdentifier, RetryCount};
use error::{Error, RobinResult};
use std;

/// The result type returned when performing jobs
pub type JobResult = Result<(), Box<std::error::Error>>;

/// A type that holds serialized job arguments.
#[derive(Serialize, Deserialize, Debug)]
pub struct Args {
    /// The serialized arguments.
    json: String,
}

impl Args {
    /// Get the JSON
    pub fn json(&self) -> &str {
        &self.json
    }

    /// Convert into string encoded JSON.
    pub fn to_json(&self) -> RobinResult<String> {
        serde_json::to_string(&self).map_err(Error::from)
    }

    /// Generic function for deserializing the encoded arguments into the type
    /// required by the job.
    ///
    /// Will return `Err(Error::SerdeJsonError(_))` if deserialization fails.
    /// This will most likely happen if a given job doesn't support the arguments type you're
    /// trying to deserialize into.
    pub fn deserialize<'a, T: Deserialize<'a>>(&'a self) -> RobinResult<T> {
        match serde_json::from_str(&self.json) {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::SerdeJsonError(e)),
        }
    }
}

/// The trait that defines what a particular job should does.
///
/// **NOTE:** You normally wouldn't need to implement this. The [`jobs!`](../macro.jobs.html) macro
/// will implement it for you.
pub trait Job {
    /// The name of the job. Required to put the job into Redis.
    fn name(&self) -> JobName;

    /// What the job actually does.
    fn perform(&self, args: &Args, con: &WorkerConnection) -> JobResult;
}

/// Trait for either performing immediately, or more commonly, later.
/// This trait is automatically implemented for types that implement [`Job`](trait.Job.html)
/// so you shouldn't ever need to implement this manually.
pub trait PerformJob {
    /// Perform the job right now without blocking.
    fn perform_now<A: Serialize>(&self, args: A, con: &WorkerConnection) -> RobinResult<()>;

    /// Put the job into the queue for processing at a later point.
    fn perform_later<A: Serialize>(&self, args: A, con: &WorkerConnection) -> RobinResult<()>;
}

impl<T> PerformJob for T
where
    T: Job,
{
    fn perform_now<A: Serialize>(&self, args: A, con: &WorkerConnection) -> RobinResult<()> {
        self.perform(&serialize_arg(args)?, con)
            .map_err(|e| Error::JobFailed(e))
    }

    fn perform_later<A: Serialize>(&self, args: A, con: &WorkerConnection) -> RobinResult<()> {
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

/// A simple new type wrapper around strings.
#[derive(Eq, PartialEq, Hash, Debug)]
pub struct JobName(pub String);

impl<T> From<T> for JobName
where
    T: Into<String>,
{
    fn from(t: T) -> JobName {
        JobName(t.into())
    }
}

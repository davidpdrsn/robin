/// Contains a queue implementation using Redis.
pub mod redis_queue;

/// Contains an in-memory queue. This queue stores the jobs in-memory of the running Rust process
/// and therefore wont work across processes. Normally you'd only use this during testing.
pub mod memory_queue;

use crate::config::Config;
use crate::job::JobName;
use std::marker::Sized;
use std::{error,
          fmt::{self, Debug}};

/// Trait that represents a backend that can be used to store jobs.
pub trait JobQueue
where
    Self: Sized,
{
    /// The type required to configure the queue.
    type Config;

    /// Create a new queue with the given config.
    fn new(init: &Self::Config) -> JobQueueResult<(Self, Self)>;

    /// Push a job into the queue.
    fn enqueue(&self, enq_job: EnqueuedJob) -> JobQueueResult<()>;

    /// Pull a job from the queue.
    fn dequeue(&self) -> Result<EnqueuedJob, NoJobDequeued>;

    /// Delete all jobs from the queue.
    fn delete_all(&self) -> JobQueueResult<()>;

    /// Get the number of jobs in the queue.
    fn size(&self) -> JobQueueResult<usize>;
}

/// The result type returned by job backends.
pub type JobQueueResult<T> = Result<T, JobQueueError>;

/// The error type used by `JobQueue` implementation.
pub type JobQueueError = Box<JobQueueErrorInformation>;

/// Information about an error that happened in a queue backend.
pub trait JobQueueErrorInformation: Debug {
    /// The primary human-readable error message. Typically one line.
    fn description(&self) -> &str;

    /// An optional secondary error message providing more details about the
    /// problem.
    fn details(&self) -> Option<&str> {
        None
    }

    /// The underlaying error that caused the problem.
    fn underlaying_error(&self) -> &error::Error;

    /// The place the error originated.
    fn origin(&self) -> ErrorOrigin;
}

impl<E> JobQueueErrorInformation for (E, ErrorOrigin)
where
    E: error::Error,
{
    fn description(&self) -> &str {
        self.0.description()
    }

    fn details(&self) -> Option<&str> {
        None
    }

    fn underlaying_error(&self) -> &error::Error {
        &self.0
    }

    fn origin(&self) -> ErrorOrigin {
        self.1
    }
}

impl<T> From<T> for Box<JobQueueErrorInformation>
where
    T: 'static + JobQueueErrorInformation,
{
    fn from(value: T) -> Box<JobQueueErrorInformation> {
        Box::new(value)
    }
}

impl fmt::Display for JobQueueErrorInformation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for JobQueueErrorInformation {
    fn description(&self) -> &str {
        self.description()
    }
}

/// The places where errors can originate in job queues.
/// These should correspond 1-to-1 with the methods.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ErrorOrigin {
    /// The error originated in the `new` method.
    Initialization,

    /// The error originated in the `enqueue` method.
    Enqueue,

    /// The error originated in the `dequeue` method.
    Dequeue,

    /// The error originated in the `delete_all` method.
    DeleteAll,

    /// The error originated in the `size` method.
    Size,
}

/// The number of times a job has been retried, if ever.
#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
pub enum RetryCount {
    /// The job has never been retried,
    NeverRetried,

    /// The job has retried given number of times.
    Count(u32),
}

impl RetryCount {
    /// Increment the retry counter by one
    pub fn increment(&self) -> RetryCount {
        match *self {
            RetryCount::NeverRetried => RetryCount::Count(1),
            RetryCount::Count(n) => RetryCount::Count(n + 1),
        }
    }

    /// `true` if the retry limit in the config has been reached, `false` otherwise
    pub fn limit_reached(&self, config: &Config) -> bool {
        match *self {
            RetryCount::NeverRetried => false,
            RetryCount::Count(n) => n > config.retry_count_limit,
        }
    }
}

/// The data structure that gets serialized and put into Redis.
#[derive(Deserialize, Serialize, Debug, Builder)]
pub struct EnqueuedJob {
    name: String,
    args: String,
    retry_count: RetryCount,
}

impl EnqueuedJob {
    /// Create a new `EnqueuedJob`
    pub fn new(name: &str, args: &str, retry_count: RetryCount) -> Self {
        EnqueuedJob {
            name: name.to_string(),
            args: args.to_string(),
            retry_count: retry_count,
        }
    }

    /// Get the name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the arguments
    pub fn args(&self) -> &str {
        &self.args
    }

    /// Get the retry count
    pub fn retry_count(&self) -> &RetryCount {
        &self.retry_count
    }
}

/// Reasons why attempting to dequeue a job didn't yield a job.
#[derive(Debug)]
pub enum NoJobDequeued {
    /// The timeout was hit. This will most likely retry dequeueing a job
    BecauseTimeout,

    /// Because there some error.
    BecauseError(JobQueueError),

    /// The job name wasn't known by the lookup function.
    BecauseUnknownJob(JobName),
}

impl<T: 'static + JobQueueErrorInformation> From<T> for NoJobDequeued {
    fn from(e: T) -> NoJobDequeued {
        NoJobDequeued::BecauseError(Box::new(e))
    }
}

/// The different queues supported by Robin.
#[derive(EachVariant, Debug, Copy, Clone)]
pub enum QueueIdentifier {
    /// The main queue all new jobs are put into.
    Main,

    /// If a job from the main queue fails it gets put into the retry queue
    /// and retried later.
    Retry,
}

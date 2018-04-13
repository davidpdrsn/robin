use error::*;
use super::{EnqueuedJob, JobQueue, NoJobDequeued, QueueIdentifier};
use std::{sync::{Mutex, mpsc::{channel, Receiver, Sender}}, time::Duration};
use std::default::Default;

/// A queue backend the stores the jobs in-memory. Normally only used during testing.
#[allow(missing_debug_implementations)]
pub struct MemoryQueue {
    send: Mutex<Sender<EnqueuedJob>>,
    recv: Mutex<Receiver<EnqueuedJob>>,
    config: Mutex<MemoryQueueConfig>,
}

/// The type used to configure an in-memory queue.
#[derive(Clone, Debug, Copy)]
pub struct MemoryQueueConfig {
    /// The time duration the worker will block while waiting for a new job to be enqueued.
    pub timeout: Duration,
}

impl Default for MemoryQueueConfig {
    fn default() -> MemoryQueueConfig {
        MemoryQueueConfig {
            timeout: Duration::from_secs(1),
        }
    }
}

impl JobQueue for MemoryQueue {
    type Config = MemoryQueueConfig;

    fn new(config: &MemoryQueueConfig) -> RobinResult<Self> {
        let (send, recv) = channel();

        Ok(MemoryQueue {
            send: Mutex::new(send),
            recv: Mutex::new(recv),
            config: Mutex::new(config.clone()),
        })
    }

    fn enqueue(&self, enq_job: EnqueuedJob, iden: QueueIdentifier) -> RobinResult<()> {
        self.send.lock().unwrap().send(enq_job);
        Ok(())
    }

    fn dequeue(&self, iden: QueueIdentifier) -> Result<EnqueuedJob, NoJobDequeued> {
        Ok(self.recv.lock().unwrap().recv().unwrap())
    }

    /// Delete all jobs from the queue.
    ///
    /// ```
    /// # extern crate robin;
    /// # use robin::prelude::*;
    /// use robin::memory_queue::*;
    /// use robin::queue_adapters::{EnqueuedJob, QueueIdentifier, RetryCount};
    ///
    /// # use std::error::Error;
    /// # fn main() {
    /// # try_main().unwrap();
    /// # }
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// #
    /// let config = MemoryQueueConfig::default();
    /// let q = MemoryQueue::new(&config)?;
    ///
    /// let job = EnqueuedJob::new("name", "args", RetryCount::NeverRetried);
    /// q.enqueue(job, QueueIdentifier::Main);
    ///
    /// assert_eq!(q.size(QueueIdentifier::Main)?, 1);
    ///
    /// q.delete_all(QueueIdentifier::Main);
    ///
    /// assert_eq!(q.size(QueueIdentifier::Main)?, 0);
    /// # Ok(())
    /// # }
    /// ```
    fn delete_all(&self, iden: QueueIdentifier) -> RobinResult<()> {
        let recv = self.recv.lock().unwrap();
        loop {
            if let Err(_) = recv.try_recv() {
                break;
            }
        }
        Ok(())
    }

    /// Get the number of jobs in the queue.
    ///
    /// ```
    /// # extern crate robin;
    /// # use robin::prelude::*;
    /// use robin::memory_queue::*;
    /// use robin::queue_adapters::{EnqueuedJob, QueueIdentifier, RetryCount};
    ///
    /// # use std::error::Error;
    /// # fn main() {
    /// # try_main().unwrap();
    /// # }
    /// # fn try_main() -> Result<(), Box<Error>> {
    /// #
    /// let config = MemoryQueueConfig::default();
    /// let q = MemoryQueue::new(&config)?;
    ///
    /// assert_eq!(q.size(QueueIdentifier::Main)?, 0);
    ///
    /// let job = EnqueuedJob::new("name", "args", RetryCount::NeverRetried);
    /// q.enqueue(job, QueueIdentifier::Main);
    ///
    /// assert_eq!(q.size(QueueIdentifier::Main)?, 1);
    /// # Ok(())
    /// # }
    /// ```
    fn size(&self, iden: QueueIdentifier) -> RobinResult<usize> {
        let mut jobs = vec![];
        let mut count = 0;

        let recv = self.recv.lock().unwrap();
        loop {
            if let Ok(job) = recv.try_recv() {
                count += 1;
                jobs.push(job);
            } else {
                break;
            }
        }

        let send = self.send.lock().unwrap();
        for job in jobs {
            send.send(job);
        }

        Ok(count)
    }
}

test_type_impls!(memory_queue_impls_send, MemoryQueue, Send);
test_type_impls!(memory_queue_impls_sync, MemoryQueue, Sync);

#![allow(unused_imports)]
use super::*;
use error::*;
use std::default::Default;
use std::{sync::{mpsc::{channel, Receiver, SendError, Sender},
                 Arc,
                 Mutex},
          time::Duration};

/// A queue backend the stores the jobs in-memory. Normally only used during testing.
#[allow(missing_debug_implementations)]
pub struct MemoryQueue {
    config: MemoryQueueConfig,
}

/// The type used to configure an in-memory queue.
#[derive(Debug)]
pub struct MemoryQueueConfig {
    timeout: Duration,
    send: Arc<Mutex<Sender<EnqueuedJob>>>,
    recv: Arc<Mutex<Receiver<EnqueuedJob>>>,
}

impl MemoryQueueConfig {
    /// Create a new `MemoryQueueConfig`
    pub fn new(timeout: Duration) -> MemoryQueueConfig {
        let (send, recv) = channel();

        MemoryQueueConfig {
            timeout,
            send: Arc::new(Mutex::new(send)),
            recv: Arc::new(Mutex::new(recv)),
        }
    }
}

impl MemoryQueueConfig {
    fn enqueue(&self, enq_job: EnqueuedJob) -> JobQueueResult<()> {
        self.send
            .lock()
            .expect("mutex was poisoned")
            .send(enq_job)
            .map_err(|e| (e, ErrorOrigin::Enqueue))?;
        Ok(())
    }

    fn dequeue(&self) -> Result<EnqueuedJob, NoJobDequeued> {
        self.recv
            .lock()
            .expect("mutex was poisoned")
            .recv_timeout(self.timeout)
            .map_err(|_| NoJobDequeued::BecauseTimeout)
    }

    fn delete_all(&self) -> JobQueueResult<()> {
        let recv = self.recv.lock().expect("mutex was poisoned");
        loop {
            if let Err(_) = recv.try_recv() {
                break;
            }
        }
        Ok(())
    }

    fn size(&self) -> JobQueueResult<usize> {
        let mut jobs = vec![];
        let mut count = 0;

        let recv = self.recv.lock().expect("mutex was poisoned");
        loop {
            if let Ok(job) = recv.try_recv() {
                count += 1;
                jobs.push(job);
            } else {
                break;
            }
        }

        let send = self.send.lock().expect("mutex was poisoned");
        for job in jobs {
            send.send(job).expect("Failed to job back in queue");
        }

        Ok(count)
    }
}

impl Default for MemoryQueueConfig {
    fn default() -> MemoryQueueConfig {
        MemoryQueueConfig::new(Duration::from_millis(100))
    }
}

impl Clone for MemoryQueueConfig {
    fn clone(&self) -> MemoryQueueConfig {
        MemoryQueueConfig {
            timeout: self.timeout.clone(),
            send: Arc::clone(&self.send),
            recv: Arc::clone(&self.recv),
        }
    }
}

fn new_queue(config: &MemoryQueueConfig) -> MemoryQueue {
    MemoryQueue {
        config: config.clone(),
    }
}

impl JobQueue for MemoryQueue {
    type Config = MemoryQueueConfig;
    type DeadSet = MemoryQueueDeadSet;

    fn new(config: &MemoryQueueConfig) -> JobQueueResult<(Self, Self, MemoryQueueDeadSet)> {
        let dead_set = MemoryQueueDeadSet {
            jobs: Arc::new(Mutex::new(vec![])),
        };

        Ok((
            new_queue(&config.clone()),
            new_queue(&config.clone()),
            dead_set,
        ))
    }

    fn enqueue(&self, enq_job: EnqueuedJob) -> JobQueueResult<()> {
        self.config.enqueue(enq_job)
    }

    fn dequeue(&self) -> Result<EnqueuedJob, NoJobDequeued> {
        self.config.dequeue()
    }

    /// Delete all jobs from the queue.
    ///
    /// ```
    /// # extern crate robin;
    /// # use robin::prelude::*;
    /// use robin::memory_queue::*;
    /// use robin::queue_adapters::{JobQueueResult, EnqueuedJob, RetryCount};
    ///
    /// # use std::error::Error;
    /// # fn main() {
    /// # try_main().unwrap();
    /// # }
    /// # fn try_main() -> JobQueueResult<()> {
    /// #
    /// let config = MemoryQueueConfig::default();
    /// let (q, _retry_q, _dead_q) = MemoryQueue::new(&config)?;
    ///
    /// let job = EnqueuedJob::new("name", "args", RetryCount::NeverRetried);
    /// q.enqueue(job);
    ///
    /// assert_eq!(q.size()?, 1);
    ///
    /// q.delete_all();
    ///
    /// assert_eq!(q.size()?, 0);
    /// # Ok(())
    /// # }
    /// ```
    fn delete_all(&self) -> JobQueueResult<()> {
        self.config.delete_all()
    }

    /// Get the number of jobs in the queue.
    ///
    /// ```
    /// # extern crate robin;
    /// # use robin::prelude::*;
    /// use robin::memory_queue::*;
    /// use robin::queue_adapters::{JobQueueResult, EnqueuedJob, RetryCount};
    ///
    /// # use std::error::Error;
    /// # fn main() {
    /// # try_main().unwrap();
    /// # }
    /// # fn try_main() -> JobQueueResult<()> {
    /// #
    /// let config = MemoryQueueConfig::default();
    /// let (q, _retry_q, _dead_q) = MemoryQueue::new(&config)?;
    ///
    /// assert_eq!(q.size()?, 0);
    ///
    /// let job = EnqueuedJob::new("name", "args", RetryCount::NeverRetried);
    /// q.enqueue(job);
    ///
    /// assert_eq!(q.size()?, 1);
    /// # Ok(())
    /// # }
    /// ```
    fn size(&self) -> JobQueueResult<usize> {
        self.config.size()
    }
}

test_type_impls!(memory_queue_impls_send, MemoryQueue, Send);
test_type_impls!(memory_queue_impls_sync, MemoryQueue, Sync);

/// The dead set type used with in-memory queues.
#[derive(Debug)]
pub struct MemoryQueueDeadSet {
    jobs: Arc<Mutex<Vec<EnqueuedJob>>>,
}

impl DeadSet for MemoryQueueDeadSet {
    fn push(&self, enq_job: EnqueuedJob) -> JobQueueResult<()> {
        self.jobs.lock().expect("mutex was poisoned").push(enq_job);
        Ok(())
    }

    fn size(&self) -> JobQueueResult<usize> {
        let size = self.jobs.lock().expect("mutex was poisoned").len();
        Ok(size)
    }

    fn iter(&self) -> Result<Box<Iterator<Item = EnqueuedJob>>, NoJobDequeued> {
        panic!()
        // Box::new(self.jobs.lock().expect("mutex was poisoned").iter())
    }
}

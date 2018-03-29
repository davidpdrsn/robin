use connection::*;
use job::*;
use connection::queue_adapters::{DequeueTimeout, NoJobDequeued, RetryCount};
use connection::ConnectionProducer;
use std::thread::{self, JoinHandle};
use config::Config;
use serde_json;

#[derive(Copy, Clone)]
struct WorkerNumber {
    number: usize,
    total_worker_count: usize,
}

impl WorkerNumber {
    fn description(&self) -> String {
        format!("{}/{}", self.number, self.total_worker_count)
    }
}

pub fn boot<T>(config: &Config, connection_producer: T)
where
    T: ConnectionProducer + Send,
{
    let worker_count = config.worker_count;

    let mut handles = vec![];

    for i in 0..worker_count {
        let worker_number = WorkerNumber {
            number: i + 1,
            total_worker_count: worker_count,
        };

        println!(
            "Robin worker {}/{} started",
            worker_number.number, worker_count
        );

        let con = connection_producer
            .new_connection()
            .expect("Failed to create new connection");

        handles.push(spawn_worker(con, worker_number));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

fn spawn_worker(con: WorkerConnection, worker_number: WorkerNumber) -> JoinHandle<()> {
    thread::spawn(move || loop {
        match perform_job(&con, QueueIdentifier::Main, worker_number) {
            LoopControl::Break => break,
            LoopControl::Continue => {}
        }
    })
}

fn perform_job(
    con: &WorkerConnection,
    iden: QueueIdentifier,
    worker_number: WorkerNumber,
) -> LoopControl {
    match con.dequeue_from(iden, DequeueTimeout(con.config.timeout)) {
        Ok((job, args, retry_count)) => {
            let prev_count = retry_count;
            let retry_count = prev_count.increment();

            if retry_count.limit_reached(&con.config) {
                println!(
                    "Not retrying {} anymore. Retry count was {:?}",
                    job.name().0,
                    prev_count,
                );

                if con.config.repeat_on_timeout {
                    LoopControl::Continue
                } else {
                    LoopControl::Break
                }
            } else {
                match iden {
                    QueueIdentifier::Main => println!(
                        "Performing {} on worker {}",
                        job.name().0,
                        worker_number.description()
                    ),
                    QueueIdentifier::Retry => println!(
                        "Retying {} on worker {}. Retry count is {:?}",
                        job.name().0,
                        worker_number.description(),
                        retry_count
                    ),
                };

                perform_or_retry(con, job, &args, retry_count, worker_number);

                LoopControl::Continue
            }
        }

        Err(NoJobDequeued::BecauseTimeout) => match iden {
            QueueIdentifier::Main => perform_job(con, QueueIdentifier::Retry, worker_number),
            QueueIdentifier::Retry => {
                if con.config.repeat_on_timeout {
                    LoopControl::Continue
                } else {
                    LoopControl::Break
                }
            }
        },

        Err(NoJobDequeued::BecauseError(err)) => {
            panic!(format!("Failed to dequeue job with error\n{:?}", err));
        }
    }
}

enum LoopControl {
    Break,
    Continue,
}

fn perform_or_retry(
    con: &WorkerConnection,
    job: &Box<Job + Send>,
    args: &str,
    retry_count: RetryCount,
    worker_number: WorkerNumber,
) {
    let args = serde_json::from_str(args).expect("todo");
    let job_result = job.perform(&con, &args);

    match job_result {
        Ok(()) => println!(
            "Performed {} on worker {}",
            job.name().0,
            worker_number.description()
        ),
        Err(_) => {
            con.retry(job.name(), &args, retry_count)
                .expect("Failed to enqueue job into retry queue");
        }
    }
}

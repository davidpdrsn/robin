use connection::*;
use job::*;
use connection::queue_adapters::{NoJobDequeued, DequeueTimeout, RetryCount};

pub fn boot(con: WorkerConnection) {
    println!("Robin worker started!");

    loop {
        match perform_job(&con, QueueIdentifier::Main) {
            LoopControl::Break => break,
            LoopControl::Continue => {}
        }
    }
}

fn perform_job(con: &WorkerConnection, iden: QueueIdentifier) -> LoopControl {
    match con.dequeue_from(iden, DequeueTimeout(con.config.timeout)) {
        Ok((job, args, retry_count)) => {
            match iden {
                QueueIdentifier::Main => println!("Performing {}", job.name().0),
                QueueIdentifier::Retry => {
                    println!("Retying {}. Retry count is {:?}", job.name().0, retry_count)
                }
            };

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
                perform_or_retry(con, job, &args, retry_count);
                LoopControl::Continue
            }
        }

        Err(NoJobDequeued::BecauseTimeout) => {
            match iden {
                QueueIdentifier::Main => perform_job(con, QueueIdentifier::Retry),
                QueueIdentifier::Retry => {
                    if con.config.repeat_on_timeout {
                        LoopControl::Continue
                    } else {
                        LoopControl::Break
                    }
                }
            }
        }

        Err(NoJobDequeued::BecauseError(err)) => {
            panic!(format!("Failed to dequeue job with error\n{:?}", err));
        }
    }
}

enum LoopControl {
    Break,
    Continue,
}

fn perform_or_retry(con: &WorkerConnection, job: &Box<Job>, args: &str, retry_count: RetryCount) {
    use serde_json;
    let args = serde_json::from_str(args).expect("todo");
    let job_result = job.perform(&con, &args);

    match job_result {
        Ok(()) => {}
        Err(_) => {
            con.retry(job.name(), &args, retry_count).expect(
                "Failed to enqueue job into retry queue",
            );
        }
    }
}

#[macro_use]
extern crate robin;
#[macro_use]
extern crate serde_derive;

use robin::prelude::*;
use std::env;

fn main() {
    let mut config = Config::default();
    config.worker_count = 10;

    if run_as_client() {
        client(config.clone());
    } else if run_as_worker() {
        worker(config.clone());
    } else {
        println!("Call with either --client or --worker");
    }
}

fn worker(config: Config) {
    let clone = config.clone();
    robin::worker::boot(&config, Jobs::lookup_job);
}

fn client(config: Config) {
    let con = establish(config, Jobs::lookup_job).expect("Failed to connect");

    let n = 100;

    for i in 0..n {
        println!("{}/{}", i + 1, n);
        Jobs::MyJob.perform_later(&con, &JobArgs).unwrap();
    }
}

enum Jobs {
    MyJob,
}

impl Jobs {
    fn lookup_job(name: &JobName) -> Option<Box<Job + Send>> {
        match name.0.as_ref() {
            "Jobs::MyJob" => Some(Box::new(Jobs::MyJob)),
            _ => None,
        }
    }
}

impl Job for Jobs {
    fn name(&self) -> JobName {
        match *self {
            Jobs::MyJob => JobName::from("Jobs::MyJob"),
        }
    }

    fn perform(&self, con: &WorkerConnection, args: &Args) -> JobResult {
        match *self {
            Jobs::MyJob => perform_my_job(con, args.deserialize()?),
        }
    }
}

fn perform_my_job(_con: &WorkerConnection, args: JobArgs) -> JobResult {
    println!("Job performed with {:?}", args);
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct JobArgs;

fn run_as_client() -> bool {
    let args = cmd_args();
    args.len() == 2 && args.get(1) == Some(&"--client".to_string())
}

fn run_as_worker() -> bool {
    let args = cmd_args();
    args.len() == 2 && args.get(1) == Some(&"--worker".to_string())
}

fn cmd_args() -> Vec<String> {
    env::args().collect::<Vec<_>>()
}

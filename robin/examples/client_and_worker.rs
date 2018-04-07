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
    robin::worker::boot(&config, Jobs::lookup_job);
}

fn client(config: Config) {
    let con = establish(config, Jobs::lookup_job).expect("Failed to connect");

    let n = 100;

    for i in 0..n {
        println!("{}/{}", i + 1, n);
        Jobs::MyJob.perform_later(&JobArgs, &con).unwrap();
    }
}

#[derive(Job)]
enum Jobs {
    #[perform_with(perform_my_job)]
    MyJob,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct JobArgs;

fn perform_my_job(args: JobArgs, _con: &WorkerConnection) -> JobResult {
    println!("Job performed with {:?}", args);
    Ok(())
}

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

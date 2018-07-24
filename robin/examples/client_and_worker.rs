#[macro_use]
extern crate robin;
#[macro_use]
extern crate serde_derive;

use robin::prelude::*;
use robin::redis_queue::*;
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
    let queue_config = RedisConfig::default();
    robin_boot_worker!(RedisQueue, &config, queue_config);
}

fn client(config: Config) {
    let queue_config = RedisConfig::default();
    let con =
        robin_establish_connection!(RedisQueue, config, queue_config).expect("Failed to connect");

    let n = 10;

    for i in 0..n {
        println!("{}/{}", i + 1, n);
        MyJob::perform_later(&JobArgs, &con).unwrap();
    }
}

jobs! {
    MyJob(JobArgs),
}

impl MyJob {
    fn perform<Q: JobQueue>(args: JobArgs, _con: &Connection<Q>) -> JobResult {
        println!("Job performed with {:?}", args);
        Ok(())
    }
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

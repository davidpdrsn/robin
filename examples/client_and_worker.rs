#[macro_use]
extern crate serde_derive;
extern crate robin;

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
    robin::worker::boot(&config, || establish_connection_to_worker(config.clone()))
}

fn client(config: Config) {
    let con = establish_connection_to_worker(config).unwrap();

    let n = 100;

    for i in 0..n {
        println!("{}/{}", i + 1, n);
        MyJob.perform_later(&con, &JobArgs).unwrap();
    }
}

fn establish_connection_to_worker(config: Config) -> RobinResult<WorkerConnection> {
    let mut con: WorkerConnection = establish(config)?;
    con.register(MyJob)?;
    Ok(con)
}

struct MyJob;

impl Job for MyJob {
    fn name(&self) -> JobName {
        JobName::from("MyJob")
    }

    fn perform(&self, _con: &WorkerConnection, args: &Args) -> JobResult {
        let args: JobArgs = args.deserialize()?;
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

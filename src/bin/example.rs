extern crate robin;
#[macro_use]
extern crate serde_derive;

use std::env::args;

use robin::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: u32,
}

struct NotifyUser;

impl Job for NotifyUser {
    fn perform(&self, _con: &WorkerConnection, args: &str) -> RobinResult<()> {
        let user: User = deserialize_arg(args)?;
        println!("{:?} has been notified!", user);
        Ok(())
    }

    fn name(&self) -> JobName {
        JobName::from("NotifyUser")
    }
}

fn main() {
    let args = args().collect::<Vec<_>>();

    if args.get(1) == Some(&"--worker".to_string()) {
        run_worker();
    } else {
        run_client();
    }
}

fn run_client() {
    use std::io::{self, BufRead};

    println!("Starting client. Press <enter> to enqueue job");

    let con = establish_connection_to_worker().expect("Failed to connect");
    let bob = User { id: 10 };

    let stdin = io::stdin();
    for _ in stdin.lock().lines() {
        NotifyUser.perform_later(&con, &bob).expect(
            "Failed to perform_later",
        );
    }
}

fn run_worker() {
    let con = establish_connection_to_worker().expect("Failed to connect");
    robin::worker::boot(con);
}

fn establish_connection_to_worker() -> RobinResult<WorkerConnection> {
    let mut con: WorkerConnection = robin::connection::establish()?;
    con.register(NotifyUser)?;
    Ok(con)
}

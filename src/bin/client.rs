extern crate robin;
#[macro_use]
extern crate serde_derive;

use std::thread;
use std::time::Duration;

use robin::prelude::*;

#[derive(Serialize, Deserialize)]
struct User {
    id: u32,
}

struct NotifyUser;

impl Job for NotifyUser {
    fn perform(&self, arg: &str) {
        let user: User = deserialize_arg(arg);

        thread::sleep(Duration::from_secs(2));
        println!("User {} has been notified!", user.id);
    }

    fn name(&self) -> JobName {
        JobName::from("NotifyUser")
    }
}

fn main() {
    let con = establish_connection_to_worker().expect("Failed to connect");

    let bob = User { id: 1 };
    NotifyUser.perform_later(&con, &bob);
}

fn establish_connection_to_worker() -> RobinResult<WorkerConnection> {
    let mut con: WorkerConnection = establish()?;
    con.register(NotifyUser)?;
    Ok(con)
}

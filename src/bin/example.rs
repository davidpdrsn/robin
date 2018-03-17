extern crate robin;
#[macro_use]
extern crate serde_derive;

use std::thread::JoinHandle;
use std::thread;
use std::time::Duration;

use robin::prelude::*;

#[derive(Serialize, Deserialize)]
struct User {
    id: u32,
}

struct NotifyUser;

impl Job for NotifyUser {
    fn perform(&self, arg: &str) -> RobinResult<()> {
        let user: User = deserialize_arg(arg)?;

        thread::sleep(Duration::from_secs(2));
        println!("User {} has been notified!", user.id);

        Ok(())
    }

    fn name(&self) -> JobName {
        JobName::from("NotifyUser")
    }
}

fn main() {
    let client = spawn_client();
    let worker = spawn_worker();

    client.join().expect("client join failed");
    worker.join().expect("worker join failed");
}

fn spawn_client() -> JoinHandle<()> {
    thread::spawn(|| {
        let con = establish_connection_to_worker().expect("Failed to connect");
        let bob = User { id: 1 };
        NotifyUser.perform_later(&con, &bob).expect(
            "Failed to perform_later",
        );
    })
}

fn spawn_worker() -> JoinHandle<()> {
    thread::spawn(|| {
        let con = establish_connection_to_worker().expect("Failed to connect");
        boot(con);
    })
}

// TODO: Duplicated between client and worker
fn establish_connection_to_worker() -> RobinResult<WorkerConnection> {
    let mut con: WorkerConnection = establish()?;
    con.register(NotifyUser)?;
    Ok(con)
}

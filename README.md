# Robin

Background jobs for Rust inspired by ActiveJob and Sidekiq :heart:

*Very* much just a proof of concept.

## Example

Some shared module:

```rust
pub mod app {
  use robin::prelude::*;

  #[derive(Serialize, Deserialize)]
  pub struct User {
      pub id: u32,
  }

  pub struct NotifyUser;

  impl Job for NotifyUser {
      fn perform(&self, args: &str) -> RobinResult<()> {
          let user: User = deserialize_arg(args)?;
          println!("{:?} has been notified!", user);
          Ok(())
      }

      fn name(&self) -> JobName {
          JobName::from("NotifyUser")
      }
  }

  pub fn establish_connection_to_worker() -> RobinResult<WorkerConnection> {
      let mut con = robin::connection::establish()?;
      con.register(NotifyUser)?;
      Ok(con)
  }
}
```

Main binary, could be where your web app also boots:

```rust
use app::*;
use robin::prelude::*;

fn main() {
    let con = establish_connection_to_worker().expect("Failed to connect");

    let bob = User { id: 10 };

    NotifyUser.perform_later(&con, &bob).expect(
        "Failed to perform_later",
    );
}
```

Worker binary:
```rust
use app::*;
use robin::prelude::*;

fn main() {
    let con = establish_connection_to_worker().expect("Failed to connect");

    robin::worker::boot(con);
}
```

#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate redis;

pub mod connection;
pub mod error;
pub mod job;
pub mod worker;

pub mod prelude {
    pub use job::{Job, PerformJob, deserialize_arg, JobName};
    pub use error::RobinResult;
    pub use connection::{WorkerConnection, establish};
    pub use worker::{boot, boot_with_config, Config};
}

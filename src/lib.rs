#![deny(warnings)]

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
pub mod config;

pub mod prelude {
    pub use job::{Job, PerformJob, deserialize_arg, JobName, JobResult};
    pub use error::RobinResult;
    pub use connection::{WorkerConnection, establish};
    pub use worker::boot;
    pub use config::Config;
}

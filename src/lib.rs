extern crate serde;
extern crate serde_json;

pub mod connection;
pub mod error;
pub mod job;
pub mod worker;

pub mod prelude {
    pub use job::{Job, PerformJob, deserialize_arg, JobName};
    pub use error::RobinResult;
    pub use connection::{WorkerConnection, establish};
    pub use worker::boot;
}

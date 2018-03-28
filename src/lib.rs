#![deny(missing_copy_implementations,
        trivial_casts,
        trivial_numeric_casts,
        unsafe_code,
        unstable_features,
        unused_import_braces,
        unused_qualifications)]

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
    pub use job::{Args, Job, PerformJob, JobName, JobResult};
    pub use error::RobinResult;
    pub use connection::{WorkerConnection, establish, ConnectionProducer};
    pub use worker::boot;
    pub use config::Config;
}

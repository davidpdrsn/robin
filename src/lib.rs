#![deny(missing_copy_implementations, trivial_casts, trivial_numeric_casts, unsafe_code,
        unstable_features, unused_import_braces, unused_qualifications)]

extern crate redis;
#[macro_use]
extern crate robin_derives;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

pub mod connection;
pub mod error;
pub mod job;
pub mod worker;
pub mod config;

pub mod prelude {
    pub use job::{Args, Enqueueable, Job, JobName, JobResult, PerformJob};
    pub use error::RobinResult;
    pub use connection::{establish, ConnectionProducer, WorkerConnection};
    pub use worker::boot;
    pub use config::Config;
    pub use robin_derives::*;
}

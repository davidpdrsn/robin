#![deny(missing_docs, unused_imports, missing_debug_implementations, missing_copy_implementations,
        trivial_casts, trivial_numeric_casts, unsafe_code, unstable_features,
        unused_import_braces, unused_qualifications)]
#![doc(html_root_url = "https://docs.rs/robin/0.2.0")]

//! # Robin
//!
//! Robin lets you run jobs in the background. This could be payment processing or sending emails.
//! Inspired by ActiveJob from Ruby on Rails.
//!
//! ## Getting started
//!
//! The standard way to use Robin is through the `#[derive(Job)]` macro. It works on
//! enum and will generate all the boilerplate needed to perform jobs.
//!
//! Here is a full example:
//!
//! ```rust
//! extern crate robin;
//! #[macro_use]
//! extern crate serde_derive;
//! #
//! # use std::error::Error;
//! #
//! # fn main() { try_main().expect("try_main failed") }
//! #
//! # fn try_main() -> Result<(), Box<Error>> {
//! use robin::prelude::*;
//!
//! #[derive(Job)]
//! enum Jobs {
//!     #[perform_with(perform_my_job)]
//!     MyJob,
//! }
//!
//! #[derive(Serialize, Deserialize, Debug, Copy, Clone)]
//! pub struct JobArgs;
//!
//! fn perform_my_job(args: JobArgs, _con: &WorkerConnection) -> JobResult {
//!     println!("Job performed with {:?}", args);
//!     Ok(())
//! }
//!
//! let config = Config::default();
//! # let mut config = Config::default();
//! # config.timeout = 1;
//! # config.redis_namespace = "doc_tests_for_crate".to_string();
//! # config.repeat_on_timeout = false;
//! # config.retry_count_limit = 1;
//! # config.worker_count = 1;
//! let worker_config = config.clone();
//!
//! let con = robin::connection::establish(config, Jobs::lookup_job)?;
//! # con.delete_all();
//!
//! assert_eq!(con.main_queue_size()?, 0);
//! assert_eq!(con.retry_queue_size()?, 0);
//!
//! for i in 0..5 {
//!     Jobs::MyJob.perform_later(&JobArgs, &con)?;
//! }
//!
//! assert_eq!(con.main_queue_size()?, 5);
//! assert_eq!(con.retry_queue_size()?, 0);
//!
//! robin::worker::boot(&worker_config, Jobs::lookup_job);
//!
//! assert_eq!(con.main_queue_size()?, 0);
//! assert_eq!(con.retry_queue_size()?, 0);
//! # Ok(())
//! # }
//! ```
//!
//! Normally the code the enqueues jobs and the code the boots the worker would be in separate
//! binaries.
//!
//! ## The prelude
//!
//! Robin provides a prelude module which exports all the commonly used types and traits.
//! Code using Robin is expected to have
//!
//! ```rust
//! use robin::prelude::*;
//! ```

extern crate num_cpus;
extern crate redis;
#[macro_use]
extern crate robin_derives;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate typesafe_derive_builder;

/// Contains the connection type and functions for establishing connections.
pub mod connection;

/// Contains the error and result types used throughout Robin.
pub mod error;

/// Contains traits for enqueueing and performing jobs.
pub mod job;

/// Contains functions for booting and running workers which perform jobs.
pub mod worker;

/// Contains the config type used to configure Robin.
pub mod config;

mod ticker;

pub mod prelude {
    //! Reexports the most commonly used types and traits from the other modules.
    //! As long as you're doing standard things this is the only `use` you'll need.

    pub use job::{Args, Job, JobName, JobResult, PerformJob};
    pub use error::RobinResult;
    pub use connection::{establish, LookupJob, WorkerConnection};
    pub use worker::boot;
    pub use config::Config;
    pub use robin_derives::*;
}

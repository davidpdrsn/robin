#![deny(
    missing_docs, unused_imports, missing_debug_implementations, missing_copy_implementations,
    trivial_casts, trivial_numeric_casts, unsafe_code, unstable_features, unused_import_braces,
    unused_qualifications
)]
#![doc(html_root_url = "https://docs.rs/robin/0.3.0")]

//! # Robin
//!
//! Robin lets you run jobs in the background. This could for example be payment processing or
//! sending emails.
//!
//! If you've used ActiveJob from Ruby on Rails you'll feel right at home.
//!
//! ## Getting started
//!
//! The standard way to use Robin is through the [`jobs!`](macro.jobs.html) macro. It takes a comma separated list of
//! job names with the argument type, and generates all the boilerplate for you. Just you have to
//! define a static method named `perform` on each of your jobs.
//!
//! Here is a full example:
//!
//! ```rust
//! #[macro_use]
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
//! use robin::redis_queue::*;
//!
//! jobs! {
//!     MyJob(JobArgs),
//! }
//!
//! impl MyJob {
//!     fn perform<Q>(args: JobArgs, _con: &Connection<Q>) -> JobResult {
//!         println!("Job performed with {:?}", args.value);
//!         Ok(())
//!     }
//! }
//!
//! #[derive(Serialize, Deserialize, Debug)]
//! pub struct JobArgs {
//!     value: String
//! }
//!
//! let config = Config::default();
//! #
//! let queue_config = RedisConfig::default();
//! # let mut queue_config = RedisConfig::default();
//! # queue_config.namespace = "doc_tests_for_crate".to_string();
//! # queue_config.timeout = 1;
//!
//! let con = robin_establish_connection!(RedisQueue, config, queue_config)?;
//! # con.delete_all();
//!
//! assert_eq!(con.main_queue_size()?, 0);
//! assert_eq!(con.retry_queue_size()?, 0);
//!
//! for i in 0..5 {
//!     MyJob::perform_later(&JobArgs { value: "foo".to_string() }, &con)?;
//! }
//!
//! assert_eq!(con.main_queue_size()?, 5);
//! assert_eq!(con.retry_queue_size()?, 0);
//!
//! # if true {
//! # robin::worker::spawn_workers::<RedisQueue, _, _>(
//! #     &config.clone(),
//! #     queue_config.clone(),
//! #     __robin_lookup_job,
//! # ).perform_all_jobs_and_die();
//! # } else {
//! robin_boot_worker!(RedisQueue, config, queue_config);
//! # }
//!
//! assert_eq!(con.main_queue_size()?, 0);
//! assert_eq!(con.retry_queue_size()?, 0);
//! # Ok(())
//! # }
//! ```
//!
//! Normally the code that enqueues jobs and the code the boots the worker would be in separate
//! binaries.
//!
//! For more details see the documentation for [each of the macros](#macros).
//!
//! ## The prelude
//!
//! Robin provides a prelude module which exports all the commonly used types and traits.
//! Code using Robin is expected to have:
//!
//! ```rust
//! use robin::prelude::*;
//! ```

#[macro_use]
extern crate log;
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

#[doc(hidden)]
#[macro_use]
#[cfg(not(release))]
mod internal_macros;

/// Contains the connection type and functions for establishing connections.
pub mod connection;

/// Contains the error and result types used throughout Robin.
pub mod error;

/// Contains traits for enqueueing and performing jobs.
///
/// **NOTE:** If you're using the [`jobs!`](../macro.jobs.html) macro you normally only need to know about the [`JobResult`](type.JobResult.html) type from this module. Everything else will be handled for you by [`jobs!`](../macro.jobs.html).
pub mod job;

/// Contains functions for booting and running workers which perform jobs.
pub mod worker;

/// Contains the config type used to configure Robin.
pub mod config;

pub mod macros;

/// Contains the different types of queue backends supplied by Robin.
pub mod queue_adapters;

mod ticker;

pub mod prelude {
    //! Reexports the most commonly used types and traits from the other modules.
    //! As long as you're doing standard things this is the only `use` you'll need.

    pub use crate::config::Config;
    pub use crate::connection::{establish, Connection, LookupJob};
    pub use crate::error::RobinResult;
    pub use crate::job::{Args, Job, JobName, JobResult, PerformJob};
    pub use crate::queue_adapters::JobQueue;
    pub use crate::worker::{boot, spawn_workers};
}

/// Contains the types you'll need if you wish to use Redis as your backend.
pub mod redis_queue {
    pub use crate::queue_adapters::redis_queue::{RedisConfig, RedisQueue};
}

/// Contains the types you'll need if you wish to use Robins in-memory queue. Usually only used for
/// testing.
pub mod memory_queue {
    pub use crate::queue_adapters::memory_queue::{MemoryQueue, MemoryQueueConfig};
}

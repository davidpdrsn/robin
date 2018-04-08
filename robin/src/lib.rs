#![deny(missing_docs, unused_imports, missing_debug_implementations, missing_copy_implementations,
        trivial_casts, trivial_numeric_casts, unsafe_code, unstable_features,
        unused_import_braces, unused_qualifications)]
#![doc(html_root_url = "https://docs.rs/robin/0.2.0")]

//! # Robin
//!
//! Robin lets you run jobs in the background. This could for example be payment processing or
//! sending emails.
//!
//! If you've used ActiveJob from Ruby on Rails you'll feel right at home.
//!
//! ## Getting started
//!
//! The standard way to use Robin is through the `jobs!` macro. It takes a comma separated list of
//! job names, and generates all the boilerplate for you. Just you have to define a static method
//! named `perform` on each of your jobs.
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
//!
//! jobs! {
//!     MyJob,
//! }
//!
//! impl MyJob {
//!     fn perform(args: JobArgs, _con: &WorkerConnection) -> JobResult {
//!         println!("Job performed with {:?}", args);
//!         Ok(())
//!     }
//! }
//!
//! #[derive(Serialize, Deserialize, Debug)]
//! pub struct JobArgs;
//!
//! let config = Config::default();
//! # let mut config = Config::default();
//! # config.timeout = 1;
//! # config.redis_namespace = "doc_tests_for_crate".to_string();
//! # config.repeat_on_timeout = false;
//! # config.retry_count_limit = 1;
//! # config.worker_count = 1;
//!
//! let con = robin_establish_connection!(config)?;
//! # con.delete_all();
//!
//! assert_eq!(con.main_queue_size()?, 0);
//! assert_eq!(con.retry_queue_size()?, 0);
//!
//! for i in 0..5 {
//!     MyJob::perform_later(&JobArgs, &con)?;
//! }
//!
//! assert_eq!(con.main_queue_size()?, 5);
//! assert_eq!(con.retry_queue_size()?, 0);
//!
//! robin_boot_worker!(config);
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
//! For more details see the `robin::macros` module documentation.
//!
//! ## The prelude
//!
//! Robin provides a prelude module which exports all the commonly used types and traits.
//! Code using Robin is expected to have:
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

/// Contains the macros exported by Robin.
///
/// # `jobs!`
///
/// Takes a comma separate list of struct names. Each struct will become a job that you can call
/// `perform_now` or `perform_later` on. You just have to implement a static method named `perform`
/// on each struct that does the actual work.
///
/// ## Example
///
/// ```rust
/// #[macro_use]
/// extern crate robin;
/// #[macro_use]
/// extern crate serde_derive;
/// #
/// use robin::prelude::*;
///
/// # fn main() {
/// jobs! {
///     SendPushNotification,
/// }
///
/// impl SendPushNotification {
///     fn perform(args: SendPushNotificationArgs, _con: &WorkerConnection) -> JobResult {
///         // Code for actually sending push notifications
///         Ok(())
///     }
/// }
///
/// #[derive(Serialize, Deserialize, Debug)]
/// pub struct SendPushNotificationArgs {
///     device_id: String,
///     platform: DevicePlatform,
/// }
///
/// #[derive(Serialize, Deserialize, Debug)]
/// pub enum DevicePlatform {
///     Ios,
///     Android,
/// }
/// # }
/// ```
///
/// # `robin_establish_connection!`
///
/// Creates a new connection used to enqueued jobs, using the given config.
///
/// ## Example
///
/// ```rust
/// #[macro_use]
/// extern crate robin;
/// #[macro_use]
/// extern crate serde_derive;
///
/// use robin::prelude::*;
///
/// # use std::error::Error;
/// # fn main() { try_main().unwrap() }
/// # fn try_main() -> Result<(), Box<Error>> {
/// # jobs! {
/// #     SendPushNotification,
/// # }
/// #
/// # impl SendPushNotification {
/// #     fn perform(args: SendPushNotificationArgs, _con: &WorkerConnection) -> JobResult {
/// #         Ok(())
/// #     }
/// # }
/// #
/// # #[derive(Serialize, Deserialize, Debug)]
/// # pub struct SendPushNotificationArgs {
/// #     device_id: String,
/// #     platform: DevicePlatform,
/// # }
/// #
/// # #[derive(Serialize, Deserialize, Debug)]
/// # pub enum DevicePlatform {
/// #     Ios,
/// #     Android,
/// # }
/// let config = Config::default();
///
/// let con = robin_establish_connection!(config)?;
///
/// let args = SendPushNotificationArgs {
///     device_id: "123".to_string(),
///     platform: DevicePlatform::Android,
/// };
///
/// SendPushNotification::perform_later(&args, &con)?;
/// # Ok(())
/// # }
/// ```
///
/// # `robin_boot_worker!`
///
/// Boots the worker which performs the jobs.
///
/// ## Example
/// ```rust
/// #[macro_use]
/// extern crate robin;
/// #[macro_use]
/// extern crate serde_derive;
///
/// use robin::prelude::*;
///
/// # fn main() {
/// # jobs! {
/// #     MyJob,
/// # }
/// # impl MyJob {
/// #     fn perform(args: (), _con: &WorkerConnection) -> JobResult {
/// #         Ok(())
/// #     }
/// # }
/// let config = Config::default();
/// # let mut config = Config::default();
/// # config.timeout = 1;
/// # config.redis_namespace = "doc_tests_for_boot_worker_macro".to_string();
/// # config.repeat_on_timeout = false;
/// # config.retry_count_limit = 1;
/// # config.worker_count = 1;
///
/// robin_boot_worker!(config);
/// # }
/// ```
pub mod macros;

mod ticker;

pub mod prelude {
    //! Reexports the most commonly used types and traits from the other modules.
    //! As long as you're doing standard things this is the only `use` you'll need.

    pub use serde::Serialize;
    pub use job::{Args, Job, JobName, JobResult, PerformJob};
    pub use error::RobinResult;
    pub use connection::{establish, LookupJob, WorkerConnection};
    pub use worker::boot;
    pub use config::Config;
    pub use robin_derives::*;
    pub use macros::*;
}

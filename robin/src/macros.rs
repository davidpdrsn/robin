//! Contains the macros exported by Robin.
//!
//! See the top level module documentation for a list of the macros.

/// Generate the boilerplate for different types of jobs.
///
/// Takes a comma separate list of struct names. Each struct will become a job that you can call
/// `::perform_now` or `::perform_later` on. The type in the parenthesis is the argument type your job
/// expects. Make sure that type implements `serde::Serialize` and `serde::Deserialize`.
/// You also have to implement a static method named `perform` on each struct that does the actual
/// work.
///
/// Make sure that you're always calling `::perform_(now|later)` and never `.perform_(now|later)`.
/// The `.` version works with any type that is `Serialize` so you might enqueue you job with the
/// wrong type of arguments.
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
///     SendPushNotification(SendPushNotificationArgs),
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
/// If you call `perform_later` or `perform_now` with an argument of the wrong type, you'll get a type error:
///
/// ```compile_fail
/// # #[macro_use]
/// # extern crate robin;
/// # #[macro_use]
/// # extern crate serde_derive;
/// #
/// # use robin::prelude::*;
/// # use std::error::Error;
/// #
/// # fn main() { try_main().unwrap() }
/// #
/// # fn try_main() -> Result<(), Box<Error>> {
/// # jobs! {
/// #     SendPushNotification(SendPushNotificationArgs),
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
/// # let config = Config::default();
/// # let con = robin_establish_connection!(config)?;
/// #
/// SendPushNotification::perform_later(&(), &con)?;
/// #
/// # Ok(())
/// # }
/// ```
///
/// **Note** that is only the case when you call `YourJob::perform_(later|now)` **not** when you
/// call `YourJob.perform_(later|now)`. So you always want to call the `::` version. See the
/// [Expansion](macro.jobs.html#expansion) section below for an example of why that is.
///
/// ## Expansion
///
/// Here is what the [`jobs!`](macro.jobs.html) macro expands into.
///
/// This:
///
/// ```rust
/// # #[macro_use]
/// # extern crate robin;
/// # #[macro_use]
/// # extern crate serde_derive;
/// #
/// # use robin::prelude::*;
/// #
/// jobs! {
///     SendPushNotification(SendPushNotificationArgs),
/// }
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
/// # }
/// #
/// # fn main() {}
/// ```
///
/// Will expand into this.
///
/// ```rust
/// # #[macro_use]
/// # extern crate robin;
/// # #[macro_use]
/// # extern crate serde_derive;
/// #
/// # use robin::prelude::*;
/// #
/// pub struct SendPushNotification;
///
/// impl Job for SendPushNotification {
///     #[inline]
///     fn name(&self) -> JobName {
///         JobName::from("SendPushNotification")
///     }
///
///     #[inline]
///     fn perform(&self, args: &Args, con: &WorkerConnection) -> JobResult {
///         SendPushNotification::perform(args.deserialize()?, con)
///     }
/// }
///
/// impl SendPushNotification {
///     #[allow(dead_code)]
///     #[inline]
///     pub fn perform_now(
///         args: &SendPushNotificationArgs,
///         con: &WorkerConnection,
///     ) -> RobinResult<()> {
///         SendPushNotification.perform_now(args, con)
///     }
///
///     #[allow(dead_code)]
///     #[inline]
///     pub fn perform_later(
///         args: &SendPushNotificationArgs,
///         con: &WorkerConnection,
///     ) -> RobinResult<()> {
///         SendPushNotification.perform_later(args, con)
///     }
/// }
///
/// pub fn __robin_lookup_job(name: &JobName) -> Option<Box<Job + Send>> {
///     match name.0.as_ref() {
///         "SendPushNotification" => Some(Box::new(SendPushNotification)),
///         _ => None,
///     }
/// }
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
/// # }
/// #
/// # fn main() {}
/// ```
///
/// `__robin_lookup_job` is a special function that implements
/// [`LookupJob`](connection/trait.LookupJob.html).
/// [`robin_establish_connection!`](macro.robin_establish_connection.html) and
/// [`robin_boot_worker!`](macro.robin_boot_worker.html) knows this name and will call it for you.
#[macro_export]
macro_rules! jobs {
    (
        $($id:ident($arg_type:ty),)*
    ) => {
        $(
            pub struct $id;

            impl Job for $id {
                #[inline]
                fn name(&self) -> JobName {
                    JobName::from(stringify!($id))
                }

                #[inline]
                fn perform(&self, args: &Args, con: &WorkerConnection) -> JobResult {
                    $id::perform(args.deserialize()?, con)
                }
            }

            impl $id {
                #[allow(dead_code)]
                #[inline]
                pub fn perform_now(
                    args: &$arg_type,
                    con: &WorkerConnection,
                ) -> RobinResult<()> {
                    $id.perform_now(args, con)
                }

                #[allow(dead_code)]
                #[inline]
                pub fn perform_later(
                    args: &$arg_type,
                    con: &WorkerConnection,
                ) -> RobinResult<()> {
                    $id.perform_later(args, con)
                }
            }
        )*

        pub fn __robin_lookup_job(name: &JobName) -> Option<Box<Job + Send>> {
            match name.0.as_ref() {
                $(
                    stringify!($id) => Some(Box::new($id)),
                )*
                _ => None,
            }
        }
    }
}

/// Creates a new connection used to enqueued jobs, using the given config.
///
/// This macro requires that you're also using [`jobs!`](macro.jobs.html) to define your jobs.
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
/// #     SendPushNotification(SendPushNotificationArgs),
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
#[macro_export]
macro_rules! robin_establish_connection {
    ($config:expr) => (
        robin::connection::establish($config.clone(), __robin_lookup_job)
    )
}

/// Boots the worker which performs the jobs.
///
/// This macro requires that you're also using [`jobs!`](macro.jobs.html) to define your jobs.
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
/// #     MyJob(()),
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
#[macro_export]
macro_rules! robin_boot_worker {
    ($config:expr) => (
        robin::worker::boot(&$config.clone(), __robin_lookup_job);
    )
}

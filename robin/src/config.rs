use std::default::Default;
use num_cpus;

/// Configuration options used throughout Robin.
///
/// The normal way to construct a `Config` is through its [`Default`](https://doc.rust-lang.org/std/default/trait.Default.html) implementation.
/// Afterwards you can tweak the values you need.
///
/// ```rust
/// # use robin::prelude::*;
/// # fn main() {
/// let mut config = Config::default();
/// config.worker_count = 10;
///
/// assert_eq!(config.worker_count, 10);
/// assert_eq!(config.timeout, 30);
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Config {
    /// The number of seconds the worker will block while waiting for a new job
    /// to be enqueued. By default workers will retry after the timeout is hit,
    /// so you shouldn't need to configure this.
    pub timeout: usize,

    /// Namespace used for all Redis values.
    pub redis_namespace: String,

    /// Whether or not to repeat looking for jobs when the timeout is hit. This
    /// defaults to `true` and should probably remain that way.
    /// This is used when testing Robin internally.
    pub repeat_on_timeout: bool,

    /// The maximum number of times a job will be retried. After that it will discarded.
    pub retry_count_limit: u32,

    /// The number of worker threads to spawn. Each thread will have its own Redis
    /// connection, so make sure you have enough connections available.
    /// Defaults to the number of CPUs your machine has.
    pub worker_count: usize,

    /// The URL that will be used to connect to Redis.
    pub redis_url: String,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            timeout: 30,
            redis_namespace: "robin_".to_string(),
            repeat_on_timeout: true,
            retry_count_limit: 10,
            worker_count: num_cpus::get(),
            redis_url: "redis://127.0.0.1/".to_string(),
        }
    }
}

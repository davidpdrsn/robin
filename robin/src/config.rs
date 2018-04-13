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
/// # }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Config {
    /// The maximum number of times a job will be retried. After that it will discarded.
    pub retry_count_limit: u32,

    /// The number of worker threads to spawn. Each thread will have its own Redis
    /// connection, so make sure you have enough connections available.
    /// Defaults to the number of CPUs your machine has.
    pub worker_count: usize,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            retry_count_limit: 10,
            worker_count: num_cpus::get(),
        }
    }
}

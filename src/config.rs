use std::default::Default;

#[derive(Debug, Clone)]
pub struct Config {
    pub timeout: usize,
    pub redis_namespace: String,
    pub repeat_on_timeout: bool,
    pub retry_count_limit: u32,
    pub worker_count: usize,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            timeout: 30,
            redis_namespace: "robin_".to_string(),
            repeat_on_timeout: true,
            retry_count_limit: 10,
            worker_count: 4,
        }
    }
}

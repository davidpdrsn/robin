use std::{error, fmt, io, fs::{self, File}, io::{BufWriter, Write, prelude::*}};

use robin::prelude::*;
use robin::redis_queue::*;

pub fn setup() {
    fs::create_dir("tests/tmp").ok();
}

pub fn teardown() {}

pub fn test_config() -> Config {
    Config::default()
}

pub fn test_redis_init() -> RedisConfig {
    let mut config = RedisConfig::default();
    config.namespace = uuid();
    config.timeout = 1;
    config
}

pub fn uuid() -> String {
    use uuid::Uuid;
    Uuid::new_v4().hyphenated().to_string()
}

#[derive(Debug)]
pub struct TestError(pub &'static str);

impl TestError {
    pub fn into_job_result(self) -> JobResult {
        Err(Box::new(TestError(self.0)))
    }
}

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for TestError {
    fn description(&self) -> &str {
        self.0
    }
}

pub fn write_tmp_test_file<S: ToString>(file: S, data: S) {
    let file = file.to_string();
    let file = format!("tests/tmp/{}", file);
    let data = data.to_string();

    let f = File::create(&file).expect(format!("Couldn't create file {}", &file).as_ref());
    let mut f = BufWriter::new(f);
    f.write_all(data.as_bytes())
        .expect(format!("Couldn't write to {}", &file,).as_ref());
}

pub fn read_tmp_test_file<S: ToString>(file: S) -> Result<String, io::Error> {
    let file = file.to_string();
    let file = format!("tests/tmp/{}", file);

    let mut f = File::open(&file)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    Ok(contents)
}

#[allow(dead_code)]
pub fn delete_tmp_test_file<S: ToString>(file: S) {
    let file = file.to_string();
    let file = format!("tests/tmp/{}", file);
    fs::remove_file(&file).ok();
}

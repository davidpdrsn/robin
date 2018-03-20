pub use robin::prelude::*;

use std::thread::{self, JoinHandle};
use std::fs::{self, File};
use std::io::{Write, BufWriter};
use std::io::prelude::*;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct VerifyableJobArgs<'a> {
    pub file: &'a str,
}

pub struct VerifyableJob;

impl VerifyableJob {
    pub fn assert_performed_with(args: &VerifyableJobArgs) {
        let contents: String = read_tmp_test_file(args.file);
        assert_eq!(contents, args.file);
    }
}

impl Job for VerifyableJob {
    fn name(&self) -> JobName {
        JobName::from("VerifyableJob")
    }

    fn perform(&self, _con: &WorkerConnection, args: &str) -> RobinResult<()> {
        let args: VerifyableJobArgs = deserialize_arg(args)?;
        write_tmp_test_file(args.file, args.file);
        Ok(())
    }
}

fn write_tmp_test_file<S: ToString>(file: S, data: S) {
    let file = file.to_string();
    let file = format!("tests/tmp/{}", file);
    let data = data.to_string();

    let f = File::create(file).expect("Unable to create file");
    let mut f = BufWriter::new(f);
    f.write_all(data.as_bytes()).expect("Unable to write data");
}

fn read_tmp_test_file<S: ToString>(file: S) -> String {
    let file = file.to_string();
    let file = format!("tests/tmp/{}", file);

    let mut f = File::open(file).expect("file not found");
    let mut contents = String::new();
    f.read_to_string(&mut contents).expect(
        "something went wrong reading the file",
    );
    contents
}

fn delete_tmp_test_file<S: ToString>(file: S) {
    let file = file.to_string();
    let file = format!("tests/tmp/{}", file);
    fs::remove_file(file).ok();
}

fn establish_connection_to_worker() -> RobinResult<WorkerConnection> {
    let mut con: WorkerConnection = establish()?;
    con.register(VerifyableJob)?;
    Ok(con)
}

pub struct TestHelper;

impl TestHelper {
    pub fn new() -> TestHelper {
        TestHelper
    }

    pub fn setup(&self, args: &VerifyableJobArgs) {
        delete_tmp_test_file(args.file);
    }

    pub fn teardown(&self, args: &VerifyableJobArgs) {
        delete_tmp_test_file(args.file);
    }

    pub fn spawn<F>(&mut self, f: F) -> JoinHandle<()>
    where
        F: 'static + FnOnce(WorkerConnection) + Send,
    {
        thread::spawn(move || {
            let con = establish_connection_to_worker().expect("Failed to connect");
            f(con)
        })
    }
}

pub trait TestConfig {
    fn test_config() -> Self;
}

impl TestConfig for Config {
    fn test_config() -> Config {
        Config {
            timeout: 2,
            loop_forever: false,
        }
    }
}

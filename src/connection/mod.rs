extern crate serde_json;

use error::*;
use job::*;
use std::collections::HashMap;

use job::JobName;
use serde_json::Value;

pub struct WorkerConnection {
    jobs: HashMap<JobName, Box<Job>>,
}

impl WorkerConnection {
    pub fn register<T>(&mut self, job: T) -> RobinResult<()>
    where
        T: 'static + Job,
    {
        let name = job.name();

        if self.jobs.contains_key(&name) {
            Err(Error::JobAlreadyRegistered(name))
        } else {
            self.jobs.insert(name, Box::new(job));
            Ok(())
        }
    }

    pub fn enqueue(&self, name: JobName, args: &str) -> RobinResult<()> {
        let filename = "jobs.robin";

        let data: String = if let Ok(data) = read_file(filename) {
            data
        } else {
            "[]".to_string()
        };

        let mut enqueued_jobs: Vec<Value> = serde_json::from_str::<Value>(&data)
            .map_err(Error::from)?
            .as_array()
            .map(Clone::clone)
            .ok_or(Error::EnqueueError(
                "Failed to load JSON as array".to_string(),
            ))?;

        let new_entry: Value = json!({ "name": name.0, "args": args });
        enqueued_jobs.push(new_entry);

        let json = json!(enqueued_jobs).to_string();
        file_write(&filename, &json)?;

        println!("Enqueued \"{}\" with {}", name.0, args);

        Ok(())
    }

    pub fn dequeue(&self) -> (Box<Job>, String) {
        use std::{time, thread};
        thread::sleep(time::Duration::from_secs(10));

        panic!()
    }
}

pub fn establish() -> RobinResult<WorkerConnection> {
    let con = WorkerConnection { jobs: HashMap::new() };
    Ok(con)
}

use std::io;
use std::fs::File;
use std::io::prelude::*;

fn file_write(filename: &str, contents: &str) -> Result<(), io::Error> {
    let mut file = File::create(filename)?;
    file.write_all(contents.as_bytes())?;
    file.flush()?;
    Ok(())
}

fn read_file(filename: &str) -> Result<String, io::Error> {
    let mut f = File::open(filename)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    Ok(contents)
}

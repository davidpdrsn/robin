use error::*;
use job::*;
use std::collections::HashMap;

use job::JobName;

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
}

pub fn establish() -> RobinResult<WorkerConnection> {
    let con = WorkerConnection { jobs: HashMap::new() };
    Ok(con)
}

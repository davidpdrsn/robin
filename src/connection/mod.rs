use error::*;
use job::*;

pub struct WorkerConnection;

impl WorkerConnection {
    pub fn register(&mut self, _name: &str, job: &Job) -> RobinResult<()> {
        Ok(())
    }
}

pub fn establish() -> RobinResult<WorkerConnection> {
    Ok(WorkerConnection)
}

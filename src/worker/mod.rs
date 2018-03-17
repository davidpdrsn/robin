use connection::*;

pub fn boot(con: WorkerConnection) {
    loop {
        let (job, arg) = con.dequeue();
        job.perform(&arg);
    }
}

use connection::*;

pub fn boot(con: WorkerConnection) {
    println!("Robin worker started!");

    loop {
        match con.dequeue() {
            Ok((job, args)) => job.perform(&args).expect("Job failed"),
            Err(err) => {
                println!("Failed to dequeue job with error\n{:?}", err);
                panic!()
            }
        }
    }
}

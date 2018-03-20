use connection::*;
use std::default::Default;

pub fn boot(con: WorkerConnection) {
    let config = Config::default();
    boot_with_config(config, con);
}

pub fn boot_with_config(config: Config, con: WorkerConnection) {
    println!("Robin worker started!");

    if config.loop_forever {
        loop {
            loop_once(&config, &con);
        }
    } else {
        loop_once(&config, &con);
    }
}

fn loop_once(config: &Config, con: &WorkerConnection) {
    match con.dequeue(config) {
        Ok((job, args)) => {
            println!("Performing {}", job.name().0);
            job.perform(&con, &args).expect("Job failed");
        }
        Err(err) => {
            println!("Failed to dequeue job with error\n{:?}", err);
            panic!()
        }
    }
}

pub struct Config {
    pub timeout: usize,
    pub loop_forever: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            timeout: 30,
            loop_forever: true,
        }
    }
}

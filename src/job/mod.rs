extern crate serde_json;

use serde::{Serialize, Deserialize};
use connection::*;

pub trait Job {
    fn perform(&self, arg: &str);
}

pub trait PerformJob {
    fn perform_now<A: Serialize>(&self, arg: A);
    fn perform_later<A: Serialize>(&self, _con: &WorkerConnection, arg: A);
}

impl<T> PerformJob for T
where
    T: Job,
{
    fn perform_now<A: Serialize>(&self, arg: A) {
        self.perform(&serialize_arg(arg));
    }

    fn perform_later<A: Serialize>(&self, _con: &WorkerConnection, arg: A) {
        self.perform(&serialize_arg(arg));
    }
}

fn serialize_arg<T: Serialize>(value: T) -> String {
    serde_json::to_string(&value).expect("Serialization failed?!")
}

pub fn deserialize_arg<'a, T: Deserialize<'a>>(arg: &'a str) -> T {
    serde_json::from_str(arg).unwrap()
}

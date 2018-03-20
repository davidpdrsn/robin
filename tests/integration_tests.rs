extern crate robin;
#[macro_use]
extern crate serde_derive;

mod test_helper;

use test_helper::*;

#[test]
fn enqueuing_and_performing_jobs() {
    let mut t = TestHelper::new();

    let args = VerifyableJobArgs { file: "enqueuing_and_performing_jobs" };

    t.setup(&args);

    let client = t.spawn(move |con| VerifyableJob.perform_later(&con, &args).unwrap());
    let worker = t.spawn(move |con| boot_with_config(Config::test_config(), con));

    client.join().unwrap();
    worker.join().unwrap();

    VerifyableJob::assert_performed_with(&args);

    t.teardown(&args);
}

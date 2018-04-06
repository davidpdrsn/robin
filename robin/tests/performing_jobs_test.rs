extern crate robin;
#[macro_use]
extern crate serde_derive;
extern crate uuid;

mod test_helper;
use test_helper::*;

#[test]
fn enqueuing_and_performing_jobs() {
    let mut t = TestHelper::new();

    let args = VerifyableJobArgs {
        file: "enqueuing_and_performing_jobs",
    };

    t.setup(&args);

    let client = t.spawn_client(move |con| Jobs::VerifyableJob.perform_later(&con, &args).unwrap());
    let worker = t.spawn_worker();

    client.join().unwrap();
    worker.join().unwrap();

    Jobs::assert_verifiable_job_performed_with(&args);

    t.teardown(&args);
}

#[test]
fn perform_now_test() {
    let mut t = TestHelper::new();

    let args = VerifyableJobArgs {
        file: "perform_now_test",
    };

    t.setup(&args);

    let client = t.spawn_client(move |con| Jobs::VerifyableJob.perform_now(&con, &args).unwrap());

    client.join().unwrap();

    Jobs::assert_verifiable_job_performed_with(&args);

    t.teardown(&args);
}

#[test]
fn running_multiple_jobs() {
    let mut t = TestHelper::new();

    let args_one = VerifyableJobArgs {
        file: "running_two_jobs_one",
    };
    let args_two = VerifyableJobArgs {
        file: "running_two_jobs_two",
    };
    let args_three = VerifyableJobArgs {
        file: "running_two_jobs_three",
    };

    t.setup(&args_one);
    t.setup(&args_two);
    t.setup(&args_three);

    let client = t.spawn_client(move |con| {
        Jobs::VerifyableJob.perform_later(&con, &args_one).unwrap();
        Jobs::VerifyableJob.perform_later(&con, &args_two).unwrap();
        Jobs::VerifyableJob
            .perform_later(&con, &args_three)
            .unwrap();
    });

    let worker = t.spawn_worker();

    client.join().unwrap();
    worker.join().unwrap();

    Jobs::assert_verifiable_job_performed_with(&args_one);
    Jobs::assert_verifiable_job_performed_with(&args_two);
    Jobs::assert_verifiable_job_performed_with(&args_three);

    t.teardown(&args_one);
    t.teardown(&args_two);
    t.teardown(&args_three);
}

#[test]
fn job_fails_then_gets_retried_and_passes() {
    let mut t = TestHelper::new();

    let file = "job_fails_then_gets_retried_and_passes";
    let args = PassSecondTimeArgs { file: file };

    t.setup(&args);

    let client = t.spawn_client(move |con| {
        Jobs::PassSecondTime
            .perform_later(&con, &args)
            .expect("Failed to enqueue job");
    });

    let worker = t.spawn_worker();

    client.join().expect("failed to end client");
    worker.join().expect("failed to end worker");

    let contents = read_tmp_test_file(file).expect("failed to read file at the end of the test");
    assert_eq!(contents, "OK");

    t.teardown(&args);
}

#[test]
fn job_doesnt_get_retried_forever() {
    let mut t = TestHelper::new();

    let args = FailForeverArgs {};

    t.setup(&args);

    let client = t.spawn_client(move |con| {
        Jobs::FailForever
            .perform_later(&con, &args)
            .expect("Failed to enqueue job");
    });

    let worker = t.spawn_worker();

    client.join().expect("failed to end client");
    worker.join().expect("failed to end worker");

    t.teardown(&args);
}

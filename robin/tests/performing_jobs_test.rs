#[macro_use]
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

    let client = t.spawn_client(move |con| VerifyableJob::perform_later(&args, &con).unwrap());
    let worker = t.spawn_worker();

    client.join().unwrap();
    worker.join().unwrap();

    assert_verifiable_job_performed_with(&args);

    t.teardown(&args);
}

#[test]
fn perform_now_test() {
    let mut t = TestHelper::new();

    let args = VerifyableJobArgs {
        file: "perform_now_test",
    };

    t.setup(&args);

    let client = t.spawn_client(move |con| VerifyableJob::perform_now(&args, &con).unwrap());

    client.join().unwrap();

    assert_verifiable_job_performed_with(&args);

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
        VerifyableJob::perform_later(&args_one, &con).unwrap();
        VerifyableJob::perform_later(&args_two, &con).unwrap();
        VerifyableJob::perform_later(&args_three, &con).unwrap();
    });

    let worker = t.spawn_worker();

    client.join().unwrap();
    worker.join().unwrap();

    assert_verifiable_job_performed_with(&args_one);
    assert_verifiable_job_performed_with(&args_two);
    assert_verifiable_job_performed_with(&args_three);

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
        PassSecondTime::perform_later(&args, &con).expect("Failed to enqueue job");
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
        FailForever::perform_later(&args, &con).expect("Failed to enqueue job");
    });

    let worker = t.spawn_worker();

    client.join().expect("failed to end client");
    worker.join().expect("failed to end worker");

    t.teardown(&args);
}

#[test]
fn jobs_with_unit_as_args() {
    use robin::error::Error;

    jobs! {
        JobWithoutArgs(()),
    }

    impl JobWithoutArgs {
        fn perform(_args: (), _con: &WorkerConnection) -> JobResult {
            TestError::with_msg("it worked")
        }
    }

    let con = establish(Config::default(), __robin_lookup_job).expect("Failed to connect");

    let result = JobWithoutArgs::perform_now(&(), &con);

    match result {
        Err(Error::JobFailed(msg)) => assert_eq!(msg.description(), "it worked".to_string()),
        _ => panic!("no match"),
    }
}

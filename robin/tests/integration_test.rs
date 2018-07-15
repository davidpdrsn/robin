#[macro_use]
extern crate robin;
extern crate uuid;

mod test_helpers;
use test_helpers::*;

use robin::memory_queue::*;
use robin::prelude::*;
use robin::redis_queue::*;

robin_test!(performing_jobs, || {
    jobs! { TestJob(String) }

    impl TestJob {
        fn perform<Q>(unique_string: String, _con: &Connection<Q>) -> JobResult {
            write_tmp_test_file(unique_string.clone(), unique_string);
            Ok(())
        }
    }

    let filename = uuid();

    let config = test_config();
    let queue_config = test_redis_init();
    let con = robin_establish_connection!(RedisQueue, config, queue_config).unwrap();

    TestJob::perform_later(&filename, &con).unwrap();

    robin::worker::spawn_workers::<RedisQueue, _, _>(
        &config.clone(),
        queue_config.clone(),
        __robin_lookup_job,
    ).perform_all_jobs_and_die();

    assert_eq!(read_tmp_test_file(filename.clone()).unwrap(), filename);
});

robin_test!(main_queue_size_test, || {
    jobs! { TestJob(()) }

    impl TestJob {
        fn perform<Q>(_args: (), _con: &Connection<Q>) -> JobResult {
            Ok(())
        }
    }

    let config = test_config();
    let queue_config = test_redis_init();
    let con = robin_establish_connection!(RedisQueue, config, queue_config).unwrap();

    assert_eq!(con.main_queue_size().unwrap(), 0);

    let job_count = config.worker_count + 1;
    for _ in 0..job_count {
        TestJob::perform_later(&(), &con).unwrap();
    }
    assert_eq!(con.main_queue_size().unwrap(), job_count);

    robin::worker::spawn_workers::<RedisQueue, _, _>(
        &config.clone(),
        queue_config.clone(),
        __robin_lookup_job,
    ).perform_all_jobs_and_die();

    assert_eq!(con.main_queue_size().unwrap(), 0);
});

robin_test!(retrying_jobs, || {
    jobs! { TestJob(()) }

    impl TestJob {
        fn perform<Q>(_args: (), _con: &Connection<Q>) -> JobResult {
            TestError("fail").into_job_result()
        }
    }

    let config = test_config();
    let queue_config = test_redis_init();
    let con = robin_establish_connection!(RedisQueue, config, queue_config).unwrap();

    assert_eq!(con.main_queue_size().unwrap(), 0);

    TestJob::perform_later(&(), &con).unwrap();

    robin::worker::spawn_workers::<RedisQueue, _, _>(
        &config.clone(),
        queue_config.clone(),
        __robin_lookup_job,
    ).perform_all_jobs_and_die();

    assert_eq!(con.main_queue_size().unwrap(), 0);
    assert_eq!(con.retry_queue_size().unwrap(), 0);
});

robin_test!(performing_with_in_memory_queue, || {
    use std::time::Duration;

    jobs! { TestJob(String) }

    impl TestJob {
        fn perform<Q>(unique_string: String, _con: &Connection<Q>) -> JobResult {
            write_tmp_test_file(unique_string.clone(), unique_string);
            Ok(())
        }
    }

    let filename = uuid();

    let config = test_config();

    let queue_config = MemoryQueueConfig::default();

    let con: Connection<MemoryQueue> =
        robin::connection::establish(config, queue_config.clone(), __robin_lookup_job).unwrap();

    TestJob::perform_later(&filename, &con).unwrap();

    robin::worker::spawn_workers::<MemoryQueue, _, _>(
        &config.clone(),
        queue_config.clone(),
        __robin_lookup_job,
    ).perform_all_jobs_and_die();

    assert_eq!(read_tmp_test_file(filename.clone()).unwrap(), filename);
});

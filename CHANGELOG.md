# Change log

All user visible changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/), as described
for Rust libraries in [RFC #1105](https://github.com/rust-lang/rfcs/blob/master/text/1105-api-evolution.md)

## Unreleased

### Added

- Added `worker::spawn_workers` which is useful during testing. See the docs for more info.
- Add trait `JobQueueErrorInformation`. The errors returned from job queues will implement this trait, and provide a bit more information about exactly what happened.
- `Connection` now has `delete`, `size`, and `empty` methods for both the main and retry queues.

### Changed

- Change `jobs!` to also require the argument type that your job expects. This mean you'll no longer be able to enqueue your jobs with the wrong type of arguments.
- Remove the export of `serde::Serialize` from `prelude` since it is no longer necessary due to [#50](https://github.com/davidpdrsn/robin/pull/50).
- We now use the [log crate](https://crates.io/crates/log) for all logging.
- We now spawn a dedicated worker that only operates on the retry queue. That means if `config.worker_count` is 10 you'll actually get 11 workers.
- `WorkerConnection` has been renamed to `Connection`.
- `Connection` is now generic over the type of jobs backend. See the docs for the minor change if you need to make to continue using Redis. In the future we will provide other job backends than Redis.
- The value contained inside an `Error::UnknownJob` has been changed from a `String` to a `JobName`.
- `Error::SerdeJsonError` has been renamed to `Error::SerdeError`.

### Removed

- Remove `#[derive(Job)]`. Turns out `job!` was able to generate all the cod we needed.
- The variants `Error::UnknownRedisError` and `Error::RedisError` has been removed. They are replaced with `JobQueueError` and `JobQueueErrorInformation`.
- Make the `connections::queue_adapters` module private. There is no reason for users to depend on this.
- `Connection::size` has been made private. Instead call `Connection::main_queue_size` or `Connection::retry_queue_size` depending on what you want.
- `QueueIdentifier::redis_queue_name`. The redis queue type now handles this internally.

### Fixed

N/A

## [0.3.0] - 2018-04-08

### Added

- `jobs!` macro which removes all boilerplate around defining jobs.
- `robin_establish_connection!` macro which makes it simpler to open a new connection without needing to know about `LookupJob`.
- `robin_boot_worker!` macro which makes it simpler to boot the worker without needing to know about `LookupJob`.
- While the worker is running it will now print jobs/second

### Changed

- `Config.worker_count` will now default to number of CPUs.

### Removed

N/A

### Fixed

N/A

## [0.2.0] - 2018-04-07

### Added

- `Config` has gotten a `redis_url` field. That URL will be used when connecting to Redis in workers and clients. Defaults to `redis://127.0.0.1/`.
- `perform_now` has been added back. It can be called on anything that also has `perform_later`, but will block and perform the job right now.
- Make the `perform_with` attribute on `#[derive(Job)]` optional. Will default to a function called `perform_my_job` if the enum variant is `MyJob`.

### Changed

- Functions that previously took the connection in the first place, and job arguments in the second place now take the connection in the last place. That was mainly `perform_now` and `perform_later`.
- The error type in `JobResult` has been changed from `String` to `Box<std::error::Error>`. Allows clients to keep using their own error types without having to map errors to strings all the time.

### Removed

N/A

### Fixed

N/A

## 0.1.0 - 2018-04-04

Initial release.

[0.3.0]: https://github.com/davidpdrsn/robin/compare/0.2.0...v0.3.0
[0.2.0]: https://github.com/davidpdrsn/robin/compare/0.1.0...0.2.0

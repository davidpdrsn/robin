# Change log

All user visible changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/), as described
for Rust libraries in [RFC #1105](https://github.com/rust-lang/rfcs/blob/master/text/1105-api-evolution.md)

## Unreleased

### Added

- `Config` has gotten a `redis_url` field. That URL will be used when connecting to Redis in workers and clients. Defaults to `redis://127.0.0.1/`.
- `perform_now` has been added back. It can be called on anything that also has `perform_later`, but will block and perform the job right now.
- Make the `perform_with` attribute on `#[derive(Job)]` optional. Will default to a function called `perform_my_job` if the enum variant is `MyJob`.

### Changed

- Functions that previously took the connection in the first place, and job arguments in the second place now take the connection in the last place. That was mainly `perform_now` and `perform_later`.

### Deprecated

N/A

### Fixed

N/A

## 0.1.0 - 2018-04-04

Initial release.

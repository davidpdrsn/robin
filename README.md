# Robin

- Master: [![Build Status](https://travis-ci.org/davidpdrsn/robin.svg?branch=master)](https://travis-ci.org/davidpdrsn/robin)

Background jobs for Rust inspired by ActiveJob and Sidekiq :heart:

## Not production ready

Probably a bad idea to use this for anything serious. Still early days.

## Example

[The docs have a complete example](https://docs.rs/robin).

## Installation

Add this to your `Cargo.toml` and you're good to go

```toml
[dependencies]
robin = "0.1.0"
```

Robin uses Redis for storing jobs, so make sure you have that installed.

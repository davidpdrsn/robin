# Robin

[![Build Status](https://travis-ci.org/davidpdrsn/robin.svg?branch=master)](https://travis-ci.org/davidpdrsn/robin)
[![Crates.io](https://img.shields.io/crates/v/robin.svg)](https://crates.io/crates/robin)
[![Documentation](https://docs.rs/robin/badge.svg)](https://docs.rs/robin/)

Background jobs for Rust inspired by ActiveJob and Sidekiq :heart:

## Not production ready

Probably a bad idea to use this for anything serious. Still early days.

## Example

[The docs have a complete example](https://docs.rs/robin).

## Installation

Add this to your `Cargo.toml` and you're good to go

```toml
[dependencies]
robin = "0.3.0"
```

Robin uses Redis for storing jobs, so make sure you have that installed.

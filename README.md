# HJBlog Rs


## Description

A blog website written for practing Rust and [Actix-web](https://actix.rs/).
The application is not complete and is not meant for running into a production environment.


## Dependencies

### Requirements

For Ubuntu 24.04:

- [Rust](https://www.rust-lang.org/learn/get-started)
- OpenSSL:

```bash
sudo apt install libssl-dev
```

### Optional Dependencies

- [Cargo Audit](https://crates.io/crates/cargo-audit): Tool for auditing dependencies for crates with security vulnerabilities reported to the [RustSec Advisory Database](https://github.com/RustSec/advisory-db/).

```bash
cargo install cargo-audit --locked
```

- [Bunyan Formatter](https://crates.io/crates/bunyan): a Rust port of (a subset of) the original [NodeJS bunyan CLI](https://github.com/trentm/node-bunyan).

Install:

```bash
cargo install bunyan
```

Usage:

```bash
cargo run | bunyan
```


## Install and Run

Project setup:

```bash
git clone 'https://github.com/hjrgrn/hjblog-rs'
cd hjblog-rs
```

Test:

```bash
cargo test
```

Run:

```bash
cargo run
```

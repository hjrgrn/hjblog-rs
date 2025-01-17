# HJBlog Rs


## Description

A blog website written for practing Rust and [Actix-web](https://actix.rs/).
The application is not complete and is not meant for running into a production environment.


## Dependencies

### Requirements

For Ubuntu 24.04:

- [Rust](https://www.rust-lang.org/learn/get-started)
- OpenSSL dev files:

```bash
sudo apt install libssl-dev
```

### Optional Dependencies

- [Docker Engine](https://docs.docker.com/engine/install/): To run the app using Docker.

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

Run using Docker:

```bash
chmod +x ./scripts/*
./scripts/build_image.sh
./scripts/run_container.sh
```

*NOTE*: The scripts are meant to be run inside the root directory of the project.
*NOTE*: The default configuration using Docker is [Local.toml](/configuration/Local.toml), which binds the server
using the address `0.0.0.0`, if you manage your firewall with Ufw or Firewalld the `5000` of your machine will be exposed
to your local network becouse Docker bypasses your firewall rules;
read [this](https://docs.docker.com/engine/network/packet-filtering-firewalls/#docker-and-ufw) for more informations.

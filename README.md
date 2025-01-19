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

- [Docker Engine](https://docs.docker.com/engine/install/): To run the Postgres instance and to deploy the app using Docker.

- [Sqlx Cli](https://crates.io/crates/sqlx-cli): SQLx's associated command-line utility for managing databases.

```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

- PostgresSQL Client, the latest version avaible in your distribution repo:

```bash
sudo apt install postgresql-client-16
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
chmod +x ./scripts/*
./scripts/init_db.sh
```

*NOTE*: if you have already initialized the database skip `./scripts/init_db.sh` and instead run:

```bash
docker container start <name_of_the_container>
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
./scripts/build_image.sh
./scripts/run_container.sh
```

*NOTE*: The scripts are meant to be run inside the root directory of the project.

*NOTE*: The default configuration using Docker is [Local.toml](/configuration/Local.toml), which binds the server
using the address `0.0.0.0`, if you manage your firewall with Ufw or Firewalld the `5000` port of your machine will be exposed
to your local network becouse Docker bypasses your firewall rules;
read [this](https://docs.docker.com/engine/network/packet-filtering-firewalls/#docker-and-ufw) for more informations.

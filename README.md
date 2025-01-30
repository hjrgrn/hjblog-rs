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

- PostgresSQL Client, the latest version available in your distribution repo:

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


## Setup And Run

```bash
git clone 'https://github.com/hjrgrn/hjblog-rs'
cd hjblog-rs
chmod +x ./scripts/*
```

### Without Docker

```bash
./scripts/init_db.sh
cargo run
```

### Using Docker

```bash
./scripts/global_init.sh
```

### Config

You can define your own configuration using the exemplar files in [configuration](/configuration/), the values inside the `configuration` files can also be changed using environment variables that have this pattern: `APP_<CONFIG__VALUE>`; for example, to change the port that the application use: `APP_APPLICATION__PORT=8080`. Other values outside `configuration` can be defined, take a look at [init_db script](/scripts/init_db.sh), which is the script for setting up a development database using Docker.

### Cli Tools

Server side CLI tools are available, to list them:

```bash
cargo run --bin ls
```

### Tests

```bash
cargo test
```


*NOTE*: The scripts are meant to be run inside the root directory of the project.

*NOTE*: The default configuration while running the application using Docker is [Local.toml](/configuration/Local.toml), this configuration assumes that application and Postgres instance are running on the same machine, the scripts generates a custom network bridge named `hjblog_bridge` that will allow dns functionality between Postgres and application; also, in this configuration the server is using the address `0.0.0.0`, if you manage your firewall with Ufw or Firewalld the `5000` port of your machine will be exposed to your local network becouse Docker bypasses your firewall rules; read [this](https://docs.docker.com/engine/network/packet-filtering-firewalls/#docker-and-ufw) for more informations.

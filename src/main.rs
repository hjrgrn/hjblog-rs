use hj_blog_rs::{
    settings::get_config,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};
use std::{io, net::TcpListener};

#[tokio::main]
async fn main() -> io::Result<()> {
    let config = get_config().expect("Failed to parse configuration files.");

    let subscriber = get_subscriber("hjblog".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let listener = TcpListener::bind(&config.application.get_full_address())?;
    run(listener)?.await
}

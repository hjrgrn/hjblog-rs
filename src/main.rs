use hj_blog_rs::{startup::run, telemetry::{get_subscriber, init_subscriber}};
use std::{io, net::TcpListener};

#[tokio::main]
async fn main() -> io::Result<()> {
    let subscriber = get_subscriber("hjblog".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let listener = TcpListener::bind("127.0.0.1:5000")?;
    run(listener)?.await
}

use hj_blog_rs::{
    settings::get_config,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};
use sqlx::postgres::PgPoolOptions;
use std::{io, net::TcpListener};

#[tokio::main]
async fn main() -> io::Result<()> {
    let config = get_config().expect("Failed to parse configuration files.");

    let subscriber = get_subscriber("hjblog".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(config.database.with_db());

    let listener = TcpListener::bind(config.server.get_full_address())?;
    run(
        listener,
        connection_pool,
        config.server.hmac_secret,
        config.server.cookie_secure,
    )?
    .await
}

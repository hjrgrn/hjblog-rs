use hj_blog_rs::{
    settings::{get_config, DatabaseSettings, Settings},
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};
use once_cell::sync::Lazy;
use rand::{distributions::Alphanumeric, Rng};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::{char, net::TcpListener};
use tokio::{select, task::JoinHandle};
use tokio_util::sync::CancellationToken;

pub struct TestApp {
    pub address: String,
    pub port: u16,
    #[allow(dead_code)]
    pub db_pool: PgPool,
    pub api_client: reqwest::Client,
    /// `cancellation_token` is needed for cleanup.
    /// `TestApp.token.cancel()` needs to be called at
    /// the end of the test function.
    /// TODO: this logic goes into drop
    pub token: CancellationToken,
    /// `handle` is needed for cleanup, `TestApp.handle.await`
    /// needs to be called after having called `TestApp.cancellation_token.cancel`.
    /// If a test fails the temporary database created for that specific test
    /// won't be cancelled and will be avaible for inspection.
    pub handle: JoinHandle<()>,
}

impl TestApp {
    pub fn get_full_url(&self) -> String {
        format!("http://{}:{}", self.address, self.port)
    }
}

/// Ensures that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    // NOTE: We cannot assign the output of `get_subscriber` to a variable based on the
    // value TEST_LOG because the sink is part of the type returned by `get_subscriber`,
    // therefore they are not the same type.
    // We could work around it, but this is the most straight forward way of making it work.
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

/// # `spawn_app`
///
/// Spawn a test instance of the application and returns information used for interacting
/// with the it in the tests
pub async fn spawn_app() -> TestApp {
    // The first time `initialize` is invoked the code `TRACING` is executed.
    // All other invocations will instead skip execution.
    Lazy::force(&TRACING);

    let config = {
        let mut c = get_config().expect("Failed to read configuration.");
        // Use a different database for each test case
        c.database.database_name = rand::thread_rng()
            .sample_iter(Alphanumeric)
            .map(char::from)
            .filter(|c| c.is_ascii_alphabetic())
            .take(48)
            .collect();
        // Find an avaible non priviledged port
        c.application.port = 0;
        c
    };

    // Create and migrate the test database
    let db_pool = configure_database(&config.database).await;

    let listener = TcpListener::bind(&config.application.get_full_address())
        .expect("Failed to bind TcpListener");
    let port = listener
        .local_addr()
        .expect("Failed to obtain local address from TcpListener.")
        .port();
    let address = config.application.host.clone();
    let token = CancellationToken::new();
    let handle = tokio::spawn(switch(listener, token.clone(), config));
    let api_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();
    TestApp {
        address,
        port,
        api_client,
        db_pool,
        token,
        handle,
    }
}

/// # Configure Database helper function
///
/// It creates a new database inside our Postgres instance that will be
/// used for testing purpose, this based on the information found in
/// `config`.
async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Connecting to Postgres without specifying a database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");
    // Using the connection generated before we create a database with the name generate
    // automatically in the caller function
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // We connect to the database that we have created
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");

    // Migrate database
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database.");

    connection_pool
}

/// # `switch`
///
/// This function allows for cleanup, removing the test database after it has been used.
async fn switch(listener: TcpListener, token: CancellationToken, config: Settings) {
    let connection_to_db = PgPool::connect_with(config.database.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    select! {
        _ = token.cancelled() => {
            connection_to_db.close().await;
            let mut connection_to_postgres = PgConnection::connect_with(&config.database.without_db())
                .await
                .expect(
                    &format!(
                        "Failed to close and delete test database {}",
                        &config.database.database_name
                    )
                );
            connection_to_postgres
                .execute(format!(r#"DROP DATABASE "{}" WITH (FORCE)"#, config.database.database_name).as_str())
                .await
                .expect(
                    &format!(
                        "Failed to close and delete test database {}",
                        &config.database.database_name
                    )
                );
            connection_to_postgres.close().await.expect("Failed to close connection to Postgres instance.");
        }
        _ = run(listener, connection_to_db.clone()).expect("Failed to spawn test instance.") => {}
    }
}

use hj_blog_rs::{
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};
use once_cell::sync::Lazy;
use std::net::TcpListener;

pub struct TestApp {
    pub address: String,
    pub port: u16,
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

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind TcpListener");
    let port = listener
        .local_addr()
        .expect("Failed to obtain local address from TcpListener.")
        .port();
    tokio::spawn(run(listener).expect("Failed to spawn test instance."));
    TestApp {
        address: "127.0.0.1".to_owned(),
        port,
    }
}

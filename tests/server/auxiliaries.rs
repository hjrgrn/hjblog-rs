use hj_blog_rs::startup::run;
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

pub async fn spawn_app() -> TestApp {
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

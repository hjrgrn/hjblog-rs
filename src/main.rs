use hj_blog_rs::startup::run;
use std::{io, net::TcpListener};

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:5000")?;
    run(listener)?.await
}

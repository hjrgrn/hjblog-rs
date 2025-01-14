use actix_web::{dev::Server, web, App, HttpServer};
use std::{io, net::TcpListener};

use crate::routes::health_check;

pub fn run(listener: TcpListener) -> Result<Server, io::Error> {
    Ok(
        HttpServer::new(move || App::new().route("/health_check", web::get().to(health_check)))
            .listen(listener)?
            .run(),
    )
}

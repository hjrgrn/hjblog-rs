use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::{Pool, Postgres};
use std::{io, net::TcpListener};

use crate::routes::health_check;

pub fn run(listener: TcpListener, connection_pool: Pool<Postgres>) -> Result<Server, io::Error> {
    let pool = web::Data::new(connection_pool);
    Ok(HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .app_data(pool.clone())
    })
    .listen(listener)?
    .run())
}

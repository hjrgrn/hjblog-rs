use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::{Pool, Postgres};
use std::{io, net::TcpListener};

use crate::routes::{error_404, health_check, index};

pub fn run(listener: TcpListener, connection_pool: Pool<Postgres>) -> Result<Server, io::Error> {
    let pool = web::Data::new(connection_pool);
    Ok(HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/", web::get().to(index))
            .route("/home", web::get().to(index))
            .route("/index", web::get().to(index))
            .service(actix_files::Files::new("/static/css", "./static/css"))
            .service(actix_files::Files::new(
                "/static/scripts",
                "./static/scripts",
            ))
            .default_service(web::route().to(error_404))
            .app_data(pool.clone())
    })
    .listen(listener)?
    .run())
}

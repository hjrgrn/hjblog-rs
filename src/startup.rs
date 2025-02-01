use actix_session::{
    config::CookieContentSecurity, storage::CookieSessionStore, SessionMiddleware,
};
use actix_web::{cookie::Key, dev::Server, web, App, HttpServer};
use secrecy::{ExposeSecret, SecretString};
use sqlx::{Pool, Postgres};
use std::{io, net::TcpListener};

use crate::routes::{auth_scope, error_404, health_check, index};

pub fn run(
    listener: TcpListener,
    connection_pool: Pool<Postgres>,
    hmac_secret: SecretString,
) -> Result<Server, io::Error> {
    let pool = web::Data::new(connection_pool);
    let secret_key = Key::from(hmac_secret.expose_secret().as_bytes());

    Ok(HttpServer::new(move || {
        App::new()
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_content_security(CookieContentSecurity::Private)
                    .build(),
            )
            .route("/health_check", web::get().to(health_check))
            .route("/", web::get().to(index))
            .route("/home", web::get().to(index))
            .route("/index", web::get().to(index))
            .service(auth_scope())
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

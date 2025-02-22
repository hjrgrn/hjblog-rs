use actix_session::{
    config::CookieContentSecurity, storage::CookieSessionStore, SessionMiddleware,
};
use actix_web::{cookie::Key, dev::Server, web, App, HttpServer};
use actix_web_flash_messages::{storage::SessionMessageStore, FlashMessagesFramework};
use secrecy::{ExposeSecret, SecretString};
use sqlx::{Pool, Postgres};
use std::{io, net::TcpListener};

use crate::routes::{
    auth_scope, error_404, health_check,
    home::{blog::blog_get, index::index_get},
    user_actions::scope::user_actions_scope,
    user_management_scope,
};

pub fn run(
    listener: TcpListener,
    connection_pool: Pool<Postgres>,
    hmac_secret: SecretString,
    cookie_secure: bool,
) -> Result<Server, io::Error> {
    let pool = web::Data::new(connection_pool);
    let secret_key = Key::from(hmac_secret.expose_secret().as_bytes());
    let flash_farmework = FlashMessagesFramework::builder(SessionMessageStore::default()).build();

    Ok(HttpServer::new(move || {
        App::new()
            .wrap(flash_farmework.clone())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_content_security(CookieContentSecurity::Private)
                    .cookie_secure(cookie_secure)
                    .cookie_http_only(true)
                    .build(),
            )
            .route("/health_check", web::get().to(health_check))
            .route("/", web::get().to(index_get))
            .route("/home", web::get().to(index_get))
            .route("/index", web::get().to(index_get))
            .route("/blog", web::get().to(blog_get))
            .service(auth_scope())
            .service(user_management_scope())
            .service(user_actions_scope())
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

use actix_web::{web, Scope};

use super::{
    login::{get::login_get, post::login_post},
    logout::logout,
    register::{get::register_get, post::register_post},
};

pub fn auth_scope() -> Scope {
    web::scope("/auth")
        .route("/login", web::get().to(login_get))
        .route("/login", web::post().to(login_post))
        .route("/logout", web::get().to(logout))
        .route("/register", web::get().to(register_get))
        .route("/register", web::post().to(register_post))
}

use actix_web::{web, Scope};

use super::{
    get_login::login_get, get_register::register_get, logout::logout, post_login::login_post,
    post_register::register_post,
};

pub fn auth_scope() -> Scope {
    web::scope("/auth")
        .route("/login", web::get().to(login_get))
        .route("/login", web::post().to(login_post))
        .route("/logout", web::get().to(logout))
        .route("/register", web::get().to(register_get))
        .route("/register", web::post().to(register_post))
}

use actix_web::{web, Scope};

use super::{
    get_login::login, get_register::register, logout::logout, post_login::login_form,
    post_register::register_form,
};

pub fn auth_scope() -> Scope {
    web::scope("/auth")
        .route("/login", web::get().to(login))
        .route("/login", web::post().to(login_form))
        .route("/logout", web::get().to(logout))
        .route("/register", web::get().to(register))
        .route("/register", web::post().to(register_form))
}

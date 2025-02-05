use actix_web::{web, Scope};

use super::{get_login::login, logout::logout, post_login::login_form};

pub fn auth_scope() -> Scope {
    web::scope("/auth")
        .route("/login", web::get().to(login))
        .route("/login", web::post().to(login_form))
        .route("/logout", web::get().to(logout))
}

use actix_web::{web, Scope};

use super::{
    change_email::{get::change_email_get, post::change_email_post},
    change_password::{get::change_password_get, post::change_password_post},
    change_username::{get::change_username_get, post::change_username_post},
    manage_profile::manage_profile,
};

pub fn user_management_scope() -> Scope {
    web::scope("/profile")
        .route("/manage_profile", web::get().to(manage_profile))
        .route("/change_username", web::get().to(change_username_get))
        .route("/change_username", web::post().to(change_username_post))
        .route("/change_password", web::get().to(change_password_get))
        .route("/change_password", web::post().to(change_password_post))
        .route("/change_email", web::get().to(change_email_get))
        .route("/change_email", web::post().to(change_email_post))
}

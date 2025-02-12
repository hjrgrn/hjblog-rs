use actix_web::{web, Scope};

use super::{change_username::{get::change_username_get, post::change_username_post}, manage_profile::manage_profile};

pub fn user_management_scope() -> Scope {
    web::scope("/profile")
        .route("/manage_profile", web::get().to(manage_profile))
        .route("/change_username", web::get().to(change_username_get))
        .route("/change_username", web::post().to(change_username_post))
}

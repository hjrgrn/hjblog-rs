use actix_web::{web, Scope};

use super::new_post::{get::new_post_get, post::new_post_post};

pub fn user_actions_scope() -> Scope {
    web::scope("/user_actions")
        .route("/new_post", web::get().to(new_post_get))
        .route("/new_post", web::post().to(new_post_post))
}

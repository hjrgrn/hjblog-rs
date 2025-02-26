use actix_web::{web, Scope};

use super::{
    comment::{get::comment_post_get, post::comment_post_post}, new_post::{get::new_post_get, post::new_post_post}, visit_post::visit_post_get
};

pub fn user_actions_scope() -> Scope {
    web::scope("/user_actions")
        .route("/new_post", web::get().to(new_post_get))
        .route("/new_post", web::post().to(new_post_post))
        .route("/visit_post/{post_id}", web::get().to(visit_post_get))
        .route("/comment/{post_id}", web::get().to(comment_post_get))
        .route("/comment/{post_id}", web::post().to(comment_post_post))
}

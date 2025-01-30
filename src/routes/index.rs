use actix_web::{error::InternalError, web};
use askama_actix::Template;
use chrono::{DateTime, Local};
use serde::Deserialize;
use sqlx::{query_as, PgPool};

use super::errors::error_500;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub title: Option<String>,
    pub posts: Option<Vec<Post>>,
}

#[derive(Deserialize, sqlx::FromRow)]
#[allow(dead_code)]
pub struct Post {
    username: String,
    title: String,
    content: String,
    posted: DateTime<Local>,
}

/// TODO: comment
pub async fn index(pool: web::Data<PgPool>) -> Result<IndexTemplate, InternalError<sqlx::Error>> {
    let posts = match get_posts(&pool).await {
        Ok(p) => {
            if p.len() == 0 {
                None
            } else {
                Some(p)
            }
        }
        Err(e) => {
            return Err(InternalError::from_response(e, error_500().await));
        }
    };
    Ok(IndexTemplate {
        title: Some(String::from("Home")),
        posts,
    })
}

// TODO: comment
#[tracing::instrument(
    name = "Querying the database for posts."
    skip(pool)
)]
async fn get_posts(pool: &PgPool) -> Result<Vec<Post>, sqlx::Error> {
    query_as::<_, Post>("SELECT users.username, posts.title, posts.content, posts.posted FROM posts JOIN users ON (users.id = posts.author_id) ORDER BY posts.posted DESC, posts.id DESC LIMIT 7").fetch_all(pool).await
}

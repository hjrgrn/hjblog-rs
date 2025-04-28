use actix_web::{error::InternalError, web};
use actix_web_flash_messages::IncomingFlashMessages;
use askama_actix::Template;
use sqlx::{query_as, PgPool};

use crate::{
    routes::{
        auxiliaries::{get_flash_messages, FormattedFlashMessage},
        errors::e500,
        CurrentUser,
    },
    session_state::TypedSession,
};

use super::auxiliaries::Post;

#[derive(Template)]
#[template(path = "home/index.html")]
pub struct IndexTemplate {
    pub title: Option<String>,
    pub current_user: Option<CurrentUser>,
    pub posts: Option<Vec<Post>>,
    pub flash_messages: Option<Vec<FormattedFlashMessage>>,
}

/// TODO: comment
pub async fn index_get(
    pool: web::Data<PgPool>,
    session: TypedSession,
    messages: IncomingFlashMessages,
) -> Result<IndexTemplate, InternalError<anyhow::Error>> {
    let flash_messages = get_flash_messages(&messages);
    let current_user = match session.get_current_user(&pool).await {
        Ok(cu) => cu,
        Err(e) => {
            return Err(e500(e.into()).await);
        }
    };

    let posts = match get_posts(&pool).await {
        Ok(p) => {
            if p.is_empty() {
                None
            } else {
                Some(p)
            }
        }
        Err(e) => {
            return Err(e500(e.into()).await);
        }
    };
    Ok(IndexTemplate {
        title: Some(String::from("Home")),
        current_user,
        posts,
        flash_messages,
    })
}

// TODO: comment
#[tracing::instrument(
    name = "Querying the database for posts."
    skip(pool)
)]
async fn get_posts(pool: &PgPool) -> Result<Vec<Post>, sqlx::Error> {
    query_as::<_, Post>(
        "SELECT \
            posts.id, \
            users.username, \
            posts.title, \
            posts.content, \
            posts.posted, \
            posts.author_id \
        FROM posts JOIN users ON (users.id = posts.author_id) \
        ORDER BY posts.posted DESC, posts.id DESC LIMIT 7"
    ).fetch_all(pool).await
}

use actix_web::{
    error::InternalError,
    web::{self, Path},
};
use actix_web_flash_messages::IncomingFlashMessages;
use askama_actix::Template;
use sqlx::{query_as, PgPool};
use uuid::Uuid;

use crate::{
    routes::{
        auxiliaries::{get_flash_messages, FormattedFlashMessage},
        errors::{e403, e500},
        home::auxiliaries::{Comment, Post},
        CurrentUser,
    },
    session_state::TypedSession,
};

#[derive(Template)]
#[template(path = "user_actions/visit_post.html")]
pub struct VisitPostTemplate {
    pub title: Option<String>,
    pub current_user: Option<CurrentUser>,
    pub post: Post,
    pub flash_messages: Option<Vec<FormattedFlashMessage>>,
    pub identified: bool,
    pub comments: Vec<Comment>,
}

// TODO: comment, telemetry, refactoring
pub async fn visit_post_get(
    pool: web::Data<PgPool>,
    session: TypedSession,
    messages: IncomingFlashMessages,
    post_id: Path<Uuid>,
) -> Result<VisitPostTemplate, InternalError<anyhow::Error>> {
    let flash_messages = get_flash_messages(&messages);
    let current_user = match session.get_current_user(&pool).await {
        Ok(cu) => cu,
        Err(e) => {
            return Err(e500(e.into()).await);
        }
    };

    let post = match query_as::<_, Post>(
        "SELECT \
            posts.id, \
            users.username, \
            posts.title, \
            posts.content, \
            posts.posted, \
            posts.author_id \
        FROM posts JOIN users ON (users.id = posts.author_id) \
        WHERE posts.id = $1",
    )
    .bind(post_id.as_ref())
    .fetch_optional(pool.as_ref())
    .await
    {
        Ok(opt) => {
            match opt {
                Some(p) => p,
                None => {
                    // user provided a non existing post id
                    return Err(e403(anyhow::anyhow!("User provided a false post id")).await);
                }
            }
        }
        Err(e) => {
            return Err(e500(e.into()).await);
        }
    };

    let comments = match query_as::<_, Comment>(
        "SELECT \
            comments.id, \
            comments.post_id, \
            comments.content, \
            comments.author_id, \
            comments.written, \
            users.username \
        FROM comments JOIN users ON (users.id = comments.author_id) \
        WHERE comments.post_id = $1
        ORDER BY comments.written DESC LIMIT 7",
    )
    .bind(post_id.as_ref())
    .fetch_all(pool.as_ref())
    .await
    {
        Ok(c) => c,
        Err(e) => {
            return Err(e500(e.into()).await);
        }
    };

    let identified = match &current_user {
        Some(cu) => {
            if post.author_id == cu.id {
                true
            } else {
                false
            }
        }
        None => false,
    };

    Ok(VisitPostTemplate {
        title: Some(String::from("Post")),
        current_user,
        post,
        flash_messages,
        identified,
        comments,
    })
}

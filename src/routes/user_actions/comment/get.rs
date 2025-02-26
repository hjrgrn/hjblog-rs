use actix_web::{
    error::InternalError,
    http::header::LOCATION,
    web::{self, Path},
    HttpResponse, Responder,
};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use askama_actix::Template;
use sqlx::{query_as, PgPool};
use uuid::Uuid;

use crate::{
    routes::{
        auxiliaries::{get_flash_messages, FormattedFlashMessage},
        errors::{e403, e500},
        home::auxiliaries::Post,
        CurrentUser,
    },
    session_state::TypedSession,
};

#[derive(Template)]
#[template(path = "user_actions/comment_post.html")]
pub struct CommentTemplate {
    pub title: Option<String>,
    pub current_user: Option<CurrentUser>,
    pub flash_messages: Option<Vec<FormattedFlashMessage>>,
    pub post: Post,
}

pub async fn comment_post_get(
    session: TypedSession,
    pool: web::Data<PgPool>,
    messages: IncomingFlashMessages,
    post_id: Path<Uuid>,
) -> Result<impl Responder, InternalError<anyhow::Error>> {
    let flash_messages = get_flash_messages(&messages);
    let current_user = match session.get_current_user(&pool).await {
        Ok(opt) => {
            match opt {
                Some(cu) => cu,
                None => {
                    FlashMessage::warning("You are already not logged in, you need to be logged in to view this page.")
                        .send();
                    return Ok(HttpResponse::SeeOther()
                        .insert_header((LOCATION, "/auth/login"))
                        .finish());
                }
            }
        }
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

    let ctx = CommentTemplate {
        title: Some(String::from("Comment Post")),
        current_user: Some(current_user),
        flash_messages,
        post,
    };

    let body = match ctx.render() {
        Ok(b) => b,
        Err(e) => {
            return Err(e500(e.into()).await);
        }
    };

    Ok(HttpResponse::Ok().body(body))
}

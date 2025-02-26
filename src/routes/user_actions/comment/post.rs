use actix_web::{
    error::InternalError,
    http::header::LOCATION,
    web::{self, Path},
    HttpResponse,
};
use actix_web_flash_messages::FlashMessage;
use serde::Deserialize;
use sqlx::{query, PgPool};
use uuid::Uuid;

use crate::{
    routes::errors::{e403, e500},
    session_state::TypedSession,
};

#[derive(Deserialize)]
pub struct CommentPostData {
    content: String,
}

// TODO: comment, input sanitazing(a domain maybe)
#[tracing::instrument(
    name = "Comment on a post",
    skip(form, pool, session)
    fields(
        username=tracing::field::Empty,
        user_id=tracing::field::Empty,
        post_id
    )
)]
pub async fn comment_post_post(
    form: web::Form<CommentPostData>,
    session: TypedSession,
    pool: web::Data<PgPool>,
    post_id: Path<Uuid>,
) -> Result<HttpResponse, InternalError<anyhow::Error>> {
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

    tracing::Span::current().record("username", &tracing::field::display(current_user.username));
    tracing::Span::current().record("user_id", &tracing::field::display(current_user.id));

    // IDEA: istead of querying the database just try INSERT
    // and match on the error type returned
    match query("SELECT id FROM posts WHERE (id = $1)")
        .bind(post_id.as_ref())
        .fetch_optional(pool.as_ref())
        .await
    {
        Ok(r) => match r {
            Some(_) => {}
            None => {
                // user provided a non existing post id
                return Err(e403(anyhow::anyhow!("User provided a false post id")).await);
            }
        },
        Err(e) => return Err(e500(e.into()).await),
    };

    match query("INSERT INTO comments (id, post_id, content, author_id) VALUES ($1, $2, $3, $4)")
        .bind(Uuid::new_v4())
        .bind(post_id.as_ref())
        .bind(&form.content)
        .bind(current_user.id)
        .execute(pool.as_ref())
        .await
    {
        Ok(_) => {
            FlashMessage::info("Your comment has been posted").send();
        }
        Err(e) => {
            // TODO: match on e
            return Err(e500(e.into()).await);
        }
    }

    Ok(HttpResponse::SeeOther()
        .insert_header((
            LOCATION,
            format!("/user_actions/visit_post/{}", post_id.as_ref()),
        ))
        .finish())
}

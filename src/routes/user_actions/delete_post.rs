use actix_web::{
    error::InternalError,
    http::header::LOCATION,
    web::{self, Path},
    HttpResponse, Responder,
};
use actix_web_flash_messages::FlashMessage;
use sqlx::{Executor, PgPool};
use uuid::Uuid;

use crate::{
    routes::errors::{e403, e500},
    session_state::TypedSession,
};

pub async fn delete_post_get(
    pool: web::Data<PgPool>,
    session: TypedSession,
    post_id: Path<Uuid>,
) -> Result<impl Responder, InternalError<anyhow::Error>> {
    let post_id = post_id.into_inner();
    match session.get_current_user(&pool).await {
        Ok(opt) => {
            match opt {
                Some(cu) => {
                    if !cu.is_admin {
                        return Err(e403(anyhow::anyhow!(format!(
                            "User {} tried to access protected route.",
                            cu.id
                        )))
                        .await);
                    }
                }
                None => {
                    FlashMessage::warning("You are already not logged in, you need to be logged in to view this page.")
                        .send();
                    return Ok(HttpResponse::SeeOther()
                        .insert_header((LOCATION, "/auth/login"))
                        .finish());
                }
            }
        }
        Err(e) => return Err(e500(e.into()).await),
    };

    delete_post_and_comments(&pool, &post_id).await?;

    FlashMessage::info("The post has been deleted correctly").send();
    Ok(HttpResponse::SeeOther()
        .insert_header((LOCATION, "/"))
        .finish())
}

#[tracing::instrument(name = "Delete post and relative comments", skip(pool, post_id), fields(post_id = %post_id))]
async fn delete_post_and_comments(pool: &PgPool, post_id: &Uuid) -> Result<(), InternalError<anyhow::Error>> {
    let mut transaction = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return Err(e500(e.into()).await),
    };

    let query = sqlx::query("SELECT FROM posts WHERE (id = $1) FOR UPDATE").bind(post_id);
    if let Err(e) = transaction.execute(query).await {
        return Err(e500(e.into()).await);
    };

    let query = sqlx::query("DELETE FROM posts WHERE (id = $1)").bind(post_id);
    if let Err(e) = transaction.execute(query).await {
        return Err(e500(e.into()).await);
    };

    let query = sqlx::query("DELETE FROM comments WHERE (post_id = $1)").bind(post_id);
    if let Err(e) = transaction.execute(query).await {
        return Err(e500(e.into()).await);
    };

    if let Err(e) = transaction.commit().await {
        return Err(e500(e.into()).await);
    };

    Ok(())
}

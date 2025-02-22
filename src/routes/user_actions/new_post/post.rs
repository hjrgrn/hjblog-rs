use actix_web::{error::InternalError, http::header::LOCATION, web, HttpResponse, Responder};
use actix_web_flash_messages::FlashMessage;
use serde::Deserialize;
use sqlx::{query, PgPool};
use uuid::Uuid;

use crate::{
    routes::errors::{e403, e500},
    session_state::TypedSession,
};

#[derive(Deserialize)]
pub struct NewPostData {
    title: String,
    content: String,
}

// TODO: comment, input sanitazing(a domain maybe)
#[tracing::instrument(
    name = "Change Password",
    skip(form, pool, session)
    fields(
        username=tracing::field::Empty,
        user_id=tracing::field::Empty
    )
)]
pub async fn new_post_post(
    form: web::Form<NewPostData>,
    session: TypedSession,
    pool: web::Data<PgPool>,
) -> Result<impl Responder, InternalError<anyhow::Error>> {
    let current_user = match session.get_current_user(&pool).await {
        Ok(opt) => {
            match opt {
                Some(cu) => {
                    if cu.is_admin {
                        cu
                    } else {
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
        Err(e) => {
            return Err(e500(e.into()).await);
        }
    };
    tracing::Span::current().record("username", &tracing::field::display(&current_user.username));
    tracing::Span::current().record("user_id", &tracing::field::display(&current_user.id));

    if form.title.len() > 60 {
        FlashMessage::warning("Title is too long, retry").send();
        return Ok(HttpResponse::SeeOther()
            .insert_header((LOCATION, "/user_actions/new_post"))
            .finish());
    } else if form.title.len() < 3 {
        FlashMessage::warning("Title is too short, retry").send();
        return Ok(HttpResponse::SeeOther()
            .insert_header((LOCATION, "/user_actions/new_post"))
            .finish());
    }

    if form.content.len() > 2000 {
        FlashMessage::warning("Content is too long, retry").send();
        return Ok(HttpResponse::SeeOther()
            .insert_header((LOCATION, "/user_actions/new_post"))
            .finish());
    } else if form.content.len() < 3 {
        FlashMessage::warning("Content is too short, retry").send();
        return Ok(HttpResponse::SeeOther()
            .insert_header((LOCATION, "/user_actions/new_post"))
            .finish());
    }

    match query("INSERT INTO posts (id, title, content, author_id) VALUES ($1, $2, $3, $4)")
        .bind(Uuid::new_v4())
        .bind(&form.title)
        .bind(&form.content)
        .bind(&current_user.id)
        .execute(pool.as_ref())
        .await
    {
        Ok(_) => {
            FlashMessage::info("Your post has been published.").send();
            Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/"))
                .finish())
        }
        Err(e) => {
            return Err(e500(e.into()).await);
        }
    }
}

use actix_web::{error::InternalError, http::header::LOCATION, web, HttpResponse, Responder};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use askama_actix::Template;
use sqlx::PgPool;

use crate::{
    routes::{
        auxiliaries::{get_flash_messages, FormattedFlashMessage},
        errors::{e403, e500},
        CurrentUser,
    },
    session_state::TypedSession,
};

#[derive(Template)]
#[template(path = "user_actions/new_post.html")]
pub struct NewPostTemplate {
    pub title: Option<String>,
    pub current_user: Option<CurrentUser>,
    pub flash_messages: Option<Vec<FormattedFlashMessage>>,
}

pub async fn new_post_get(
    session: TypedSession,
    pool: web::Data<PgPool>,
    messages: IncomingFlashMessages,
) -> Result<impl Responder, InternalError<anyhow::Error>> {
    let flash_messages = get_flash_messages(&messages);
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

    let ctx = NewPostTemplate {
        title: Some(String::from("New Post")),
        current_user: Some(current_user),
        flash_messages,
    };

    let body = match ctx.render() {
        Ok(b) => b,
        Err(e) => {
            return Err(e500(e.into()).await);
        }
    };

    Ok(HttpResponse::Ok().body(body))
}

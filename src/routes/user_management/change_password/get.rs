use crate::{
    routes::{
        auxiliaries::{get_flash_messages, FormattedFlashMessage},
        errors::e500,
        CurrentUser,
    },
    session_state::TypedSession,
};
use actix_web::{http::header::LOCATION, web::Data, HttpResponse, Responder};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use askama_actix::Template;
use sqlx::PgPool;

#[derive(Template)]
#[template(path = "change_password.html")]
pub struct ChangePasswordTemplate {
    pub title: Option<String>,
    pub flash_messages: Option<Vec<FormattedFlashMessage>>,
    pub current_user: CurrentUser,
}

pub async fn change_password_get(
    messages: IncomingFlashMessages,
    session: TypedSession,
    pool: Data<PgPool>,
) -> Result<impl Responder, actix_web::error::InternalError<anyhow::Error>> {
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

    let ctx = ChangePasswordTemplate {
        title: Some(String::from("Change Password")),
        flash_messages,
        current_user,
    };
    let body = match ctx.render() {
        Ok(c) => c,
        Err(e) => {
            return Err(e500(e.into()).await);
        }
    };
    Ok(HttpResponse::Ok().body(body))
}

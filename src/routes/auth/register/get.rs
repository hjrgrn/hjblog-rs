use actix_web::{http::header::LOCATION, HttpResponse, Responder};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use askama_actix::Template;

use crate::{
    routes::{
        auxiliaries::{get_flash_messages, FormattedFlashMessage},
        errors::e500,
    },
    session_state::TypedSession,
};

#[derive(Template)]
#[template(path = "auth/register.html")]
pub struct RegisterTemplate {
    pub title: Option<String>,
    pub flash_messages: Option<Vec<FormattedFlashMessage>>,
}

/// # `register_get`
///
/// Response to get "/auth/register"
pub async fn register_get(
    session: TypedSession,
    messages: IncomingFlashMessages,
) -> Result<impl Responder, actix_web::error::InternalError<anyhow::Error>> {
    let flash_messages = get_flash_messages(&messages);

    let user_id = match session.get_user_id() {
        Ok(id) => id,
        Err(e) => {
            return Err(e500(e.into()).await);
        }
    };
    if user_id.is_some() {
        FlashMessage::warning("You are already registered, before register again logout.").send();
        return Ok(HttpResponse::SeeOther()
            .insert_header((LOCATION, "/"))
            .finish());
    }

    let ctx = RegisterTemplate {
        title: Some("Register".into()),
        flash_messages,
    };
    let body = match ctx.render() {
        Ok(c) => c,
        Err(e) => {
            return Err(e500(e.into()).await);
        }
    };

    Ok(HttpResponse::Ok().body(body))
}

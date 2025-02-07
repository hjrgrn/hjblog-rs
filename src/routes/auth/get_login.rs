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
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub title: Option<String>,
    pub flash_messages: Option<Vec<FormattedFlashMessage>>,
}

/// # `login_get`
///
/// Response to get "/auth/login"
pub async fn login_get(
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
    match user_id {
        Some(_) => {
            FlashMessage::warning("You are already logged in, before logging in again logout.")
                .send();
            return Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/"))
                .finish());
        }
        None => {}
    }

    let ctx = LoginTemplate {
        title: Some("Sign In".into()),
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

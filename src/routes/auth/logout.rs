use actix_web::{http::header::LOCATION, HttpResponse};
use actix_web_flash_messages::FlashMessage;

use crate::{routes::errors::e500, session_state::TypedSession};

/// # `logout`
///
/// Response to get "/auth/logout"
#[tracing::instrument(
    name = "Logging out user"
    skip(session),
    fields(
        user_id=tracing::field::Empty,
    )
)]
pub async fn logout(
    session: TypedSession,
) -> Result<HttpResponse, actix_web::error::InternalError<anyhow::Error>> {
    let user_id = match session.get_user_id() {
        Ok(id) => id,
        Err(e) => {
            return Err(e500(e.into()).await);
        }
    };
    let user_id = match user_id {
        Some(id) => id,
        None => {
            FlashMessage::warning("You are not logged in.").send();
            return Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/auth/login"))
                .finish());
        }
    };
    tracing::Span::current().record("user_id", tracing::field::display(&user_id));
    FlashMessage::info("See you space cowboy...").send();
    session.logout();
    return Ok(HttpResponse::SeeOther()
        .insert_header((LOCATION, "/"))
        .finish());
}

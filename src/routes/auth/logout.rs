use actix_web::{http::header::LOCATION, HttpResponse};
use actix_web_flash_messages::FlashMessage;

use crate::{routes::errors::e500, session_state::TypedSession};

/// TODO: comment, refactoring
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
    tracing::Span::current().record("user_id", &tracing::field::display(&user_id));

    match session.delete_user_id() {
        Some(_) => {
            FlashMessage::info("See you space cowboy").send();
            return Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/"))
                .finish());
        }
        None => {
            // NOTE: this should not happen
            return Err(e500(anyhow::anyhow!("Failed to remove \"user_id\" entry from TypedSession when user is logged in, this shouldn't have happende.")).await);
        }
    }
}

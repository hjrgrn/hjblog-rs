//! TODO: option dataform
use actix_web::{
    error::InternalError,
    http::header::LOCATION,
    web::{Data, Form},
    HttpResponse,
};
use actix_web_flash_messages::FlashMessage;
use secrecy::SecretString;
use serde::Deserialize;
use sqlx::PgPool;

use crate::{routes::errors::e500, session_state::TypedSession};

use super::auxiliaries::{validate_login_credentials, AuthError, LoginCredentials};

#[derive(Deserialize)]
pub struct LoginFormData {
    username: String,
    password: SecretString,
}

// TODO: comment, refactor, tracing
#[tracing::instrument(
    skip(form, pool, session),
    fields(
        username=tracing::field::Empty,
        user_id=tracing::field::Empty
    )
)]
pub async fn login_form(
    session: TypedSession,
    form: Form<LoginFormData>,
    pool: Data<PgPool>,
) -> Result<HttpResponse, InternalError<anyhow::Error>> {
    // FIX: code duplication in super::get_login::login
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

    let credentials = LoginCredentials {
        username: form.0.username,
        password: form.0.password,
    };

    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    match validate_login_credentials(credentials, &pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
            session.renew();
            match session.insert_user_id(user_id) {
                Ok(_) => {}
                Err(e) => return Err(e500(e.into()).await),
            }
            let name = match session.get_current_user(&pool).await {
                Ok(opt) => {
                    match opt {
                        Some(cu) => cu.username,
                        None => {
                            // This should not happen
                            return Err(
                                e500(anyhow::anyhow!("This shouldn't have happened.")).await
                            );
                        }
                    }
                }
                Err(e) => {
                    // This should not happen
                    return Err(e500(e.into()).await);
                }
            };
            FlashMessage::info(format!("Welcome back {}!", name)).send();
            Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/"))
                .finish())
        }
        Err(e) => {
            match e {
                AuthError::InvalidCredentials(e) => {
                    FlashMessage::warning("Invalid credentials, try again.").send();
                    return Err(InternalError::from_response(
                        e,
                        HttpResponse::SeeOther()
                            .insert_header((LOCATION, "/auth/login"))
                            .finish(),
                    ))
                }
                AuthError::UnexpectedError(e) => return Err(e500(e.into()).await),
            };
        }
    }
}

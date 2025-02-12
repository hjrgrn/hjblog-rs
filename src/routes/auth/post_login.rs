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

use crate::{domain::{ValidPassword, ValidUserName}, routes::errors::e500, session_state::TypedSession};

use super::auxiliaries::{validate_basic_credentials, AuthError, BasicCredentials};

#[derive(Deserialize)]
pub struct LoginFormData {
    username: String,
    password: SecretString,
}

/// # `login_post`
///
/// Response to post "/auth/login"
#[tracing::instrument(
    skip(form, pool, session),
    fields(
        username=tracing::field::Empty,
        user_id=tracing::field::Empty
    )
)]
pub async fn login_post(
    session: TypedSession,
    form: Form<LoginFormData>,
    pool: Data<PgPool>,
) -> Result<HttpResponse, InternalError<anyhow::Error>> {
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

    let username = match ValidUserName::parse(&form.0.username) {
        Ok(n) => n,
        Err(e) => {
            FlashMessage::warning(&format!("{}", e)).send();
            return Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/profile/change_username"))
                .finish());
        }
    };
    let password = match ValidPassword::parse(&form.0.password) {
        Ok(p) => p,
        Err(e) => {
            FlashMessage::warning(&format!("{}", e)).send();
            return Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/profile/change_username"))
                .finish());
        }
    };

    let credentials = BasicCredentials {
        username,
        password,
    };

    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    match validate_basic_credentials(credentials, &pool).await {
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
                    ));
                }
                AuthError::UnexpectedError(e) => return Err(e500(e.into()).await),
            };
        }
    }
}

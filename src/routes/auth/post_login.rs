use actix_web::{
    error::InternalError,
    http::header::LOCATION,
    web::{Data, Form},
    HttpResponse,
};
use secrecy::SecretString;
use serde::Deserialize;
use sqlx::PgPool;

use crate::{routes::errors::e500, session_state::TypedSession};

use super::auxiliaries::{validate_credentials, AuthError, Credentials};

#[derive(Deserialize)]
pub struct LoginFormData {
    username: String,
    password: SecretString,
}

// TODO: comment, refactor, flash, tracing
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
            return Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/"))
                .finish());
        }
        None => {}
    }

    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password,
    };

    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    match validate_credentials(credentials, &pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
            session.renew();
            match session.insert_user_id(user_id) {
                Ok(_) => {}
                Err(e) => return Err(e500(e.into()).await),
            }
            Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/"))
                .finish())
        }
        Err(e) => {
            match e {
                AuthError::InvalidCredentials(e) => {
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

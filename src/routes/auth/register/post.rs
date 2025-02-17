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

use crate::{
    routes::{
        auth::auxiliaries::{register_user, RegisterCredentials, RegisterError},
        errors::e500,
    },
    session_state::TypedSession,
};

#[derive(Deserialize)]
pub struct LoginFormData {
    username: String,
    email: String,
    password: SecretString,
    confirm_password: SecretString,
}

/// # `register_post`
///
/// Response to post "/auth/register"
#[tracing::instrument(
    skip(form, pool, session),
    fields(
        username=tracing::field::Empty,
        email=tracing::field::Empty,
        user_id=tracing::field::Empty
    )
)]
pub async fn register_post(
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
            FlashMessage::warning("You are already registered, before register again logout.")
                .send();
            return Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/"))
                .finish());
        }
        None => {}
    }

    let credentials = match RegisterCredentials::parse(
        &form.username,
        &form.email,
        &form.password,
        &form.confirm_password,
    ) {
        Ok(c) => c,
        Err(e) => {
            FlashMessage::warning(format!("{}", e)).send();
            return Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/auth/register"))
                .finish());
        }
    };

    tracing::Span::current().record(
        "username",
        &tracing::field::display(&credentials.get_username()),
    );
    tracing::Span::current().record("email", &tracing::field::display(&credentials.get_email()));

    match register_user(&credentials, &pool, false).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
            session.renew();
            match session.insert_user_id(user_id) {
                Ok(_) => {}
                Err(e) => return Err(e500(e.into()).await),
            }

            FlashMessage::info(format!(
                "Welcome {}, you have been registered correctly!",
                credentials.get_username()
            ))
            .send();
            Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/"))
                .finish())
        }
        Err(e) => match e {
            RegisterError::InvalidCredentials(e) => {
                FlashMessage::warning(format!("{}", e)).send();
                return Err(InternalError::from_response(
                    e,
                    HttpResponse::SeeOther()
                        .insert_header((LOCATION, "/auth/register"))
                        .finish(),
                ));
            }
            RegisterError::Sqlx(e) => return Err(e500(e.into()).await),
            RegisterError::Argon2(e) => return Err(e500(e.into()).await),
        },
    }
}

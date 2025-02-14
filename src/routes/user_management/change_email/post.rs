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
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ChangeEmailData {
    email: String,
    password: SecretString,
}

use crate::{
    domain::{ValidEmail, ValidPassword, ValidUserName},
    routes::{
        auth::auxiliaries::{validate_basic_credentials, AuthError, BasicCredentials},
        errors::e500,
        user_management::auxiliaries::UpdateProfileError,
    },
    session_state::TypedSession,
};

/// # `change_email_post`
///
/// Response to post "/profile/change_email"
/// TODO: refactoring
#[tracing::instrument(
    name = "Change Email",
    skip(form, pool, session),
    fields(
        old_email=tracing::field::Empty,
        new_email=tracing::field::Empty,
    )
)]
pub async fn change_email_post(
    session: TypedSession,
    pool: Data<PgPool>,
    form: Form<ChangeEmailData>,
) -> Result<HttpResponse, InternalError<anyhow::Error>> {
    // FIX: code duplication
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
    let new_email = match ValidEmail::parse(&form.0.email) {
        Ok(n) => n,
        Err(e) => {
            FlashMessage::warning(&format!("{}", e)).send();
            return Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/profile/change_email"))
                .finish());
        }
    };
    let password = match ValidPassword::parse(&form.0.password) {
        Ok(p) => p,
        Err(e) => {
            FlashMessage::warning(&format!("{}", e)).send();
            return Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/profile/change_email"))
                .finish());
        }
    };

    let old_email = match ValidEmail::parse(&current_user.email) {
        Ok(p) => p,
        Err(e) => {
            FlashMessage::warning(&format!("{}", e)).send();
            return Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/profile/change_email"))
                .finish());
        }
    };

    // IDEA: return domains from `.get_current_user` istead of raw Strings and such
    let username = match ValidUserName::parse(&current_user.username) {
        Ok(u) => u,
        Err(e) => {
            FlashMessage::warning(&format!("{}", e)).send();
            return Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/profile/change_email"))
                .finish());
        }
    };

    let credentials = BasicCredentials { username, password };

    tracing::Span::current().record("old_email", &tracing::field::display(old_email.as_ref()));
    tracing::Span::current().record("new_email", &tracing::field::display(new_email.as_ref()));

    match validate_basic_credentials(credentials, &pool).await {
        Ok(_) => {}
        Err(e) => {
            match e {
                AuthError::InvalidCredentials(e) => {
                    session.logout();
                    FlashMessage::warning("Invalid credentials, you have been logged out.").send();
                    return Err(InternalError::from_response(
                        e,
                        HttpResponse::SeeOther()
                            .insert_header((LOCATION, "/"))
                            .finish(),
                    ));
                }
                AuthError::UnexpectedError(e) => return Err(e500(e.into()).await),
            };
        }
    }
    match update_email(&pool, new_email.as_ref(), &current_user.id).await {
        Ok(()) => {
            FlashMessage::info(format!(
                "Your email has been updated to {}",
                new_email.as_ref()
            ))
            .send();
            Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/"))
                .finish())
        }
        Err(e) => match e {
            UpdateProfileError::InvalidValue(err) => {
                FlashMessage::warning(&format!("{}", err)).send();
                return Err(InternalError::from_response(
                    err,
                    HttpResponse::SeeOther()
                        .insert_header((LOCATION, "/profile/change_email"))
                        .finish(),
                ));
            }
            UpdateProfileError::UnexpectedError(e) => {
                return Err(e500(e.into()).await);
            }
        },
    }
}

// TODO: comment, refactoring, code duplication
async fn update_email(
    pool: &PgPool,
    email: &str,
    user_id: &Uuid,
) -> Result<(), UpdateProfileError> {
    let res = sqlx::query("SELECT id FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await;
    match res {
        Ok(opt) => match opt {
            Some(_) => {
                return Err(anyhow::anyhow!(
                    "The new email you provided is already taken, please try again."
                )
                .into());
            }
            None => {}
        },
        Err(e) => {
            return Err(e.into());
        }
    }
    sqlx::query("UPDATE users SET email = $1 WHERE (id = $2)")
        .bind(email)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

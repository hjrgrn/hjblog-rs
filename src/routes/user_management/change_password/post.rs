use actix_web::{
    error::InternalError,
    http::header::LOCATION,
    web::{Data, Form},
    HttpResponse,
};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use rand::rngs::OsRng;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ChangePasswordData {
    old_password: SecretString,
    new_password: SecretString,
}

use crate::{
    domain::{ValidPassword, ValidUserName},
    routes::{
        auth::auxiliaries::{validate_basic_credentials, AuthError, BasicCredentials},
        errors::e500,
        user_management::auxiliaries::UpdateProfileError,
    },
    session_state::TypedSession,
};

/// # `change_password_post`
///
/// Response to post "/profile/change_password"
/// TODO: refactoring
#[tracing::instrument(name = "Change Password", skip(form, pool, session))]
pub async fn change_password_post(
    session: TypedSession,
    pool: Data<PgPool>,
    form: Form<ChangePasswordData>,
) -> Result<HttpResponse, InternalError<anyhow::Error>> {
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
    let old_password = match ValidPassword::parse(&form.0.old_password) {
        Ok(p) => p,
        Err(e) => {
            FlashMessage::warning(&format!("{}", e)).send();
            return Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/profile/change_username"))
                .finish());
        }
    };

    let new_password = match ValidPassword::parse(&form.0.new_password) {
        Ok(p) => p,
        Err(e) => {
            FlashMessage::warning(&format!("{}", e)).send();
            return Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/profile/change_username"))
                .finish());
        }
    };

    let username = match ValidUserName::parse(&current_user.username) {
        Ok(p) => p,
        Err(e) => {
            // NOTE: this should not happen, if this happens we have
            // corrupted data in the database
            let err = format!("This shouldn't have happened, we probabily have corrupted data in the database:\n{e}");
            return Err(e500(anyhow::anyhow!(err).into()).await);
        }
    };

    let credentials = BasicCredentials {
        username,
        password: old_password,
    };

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
    match update_password(&pool, new_password.as_ref(), &current_user.id).await {
        Ok(()) => {
            FlashMessage::info("Your password has been updated.").send();
            Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/"))
                .finish())
        }
        Err(e) => match e {
            UpdateProfileError::InvalidValue(err) => {
                // This should not happen
                FlashMessage::warning(&format!("{}", err)).send();
                return Err(InternalError::from_response(
                    err,
                    HttpResponse::SeeOther()
                        .insert_header((LOCATION, "/profile/change_username"))
                        .finish(),
                ));
            }
            UpdateProfileError::UnexpectedError(e) => {
                return Err(e500(e.into()).await);
            }
        },
    }
}

// TODO: comment, refactoring
async fn update_password(
    pool: &PgPool,
    password: &SecretString,
    user_id: &Uuid,
) -> Result<(), UpdateProfileError> {
    // FIX: we are hashing twice for no reason
    let salt = SaltString::generate(OsRng);
    let hash_pass = Argon2::default()
        .hash_password(password.expose_secret().as_bytes(), &salt)
        .context("Problems with hashing passwords with Argon2")?
        .to_string();

    sqlx::query("UPDATE users SET hash_pass = $1 WHERE (id = $2)")
        .bind(hash_pass)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

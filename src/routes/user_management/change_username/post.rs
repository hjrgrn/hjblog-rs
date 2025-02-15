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
pub struct ChangeUsernameData {
    username: String,
    password: SecretString,
}

use crate::{
    domain::{ValidPassword, ValidUserName},
    routes::{
        auth::auxiliaries::{validate_basic_credentials, AuthError, BasicCredentials},
        errors::e500,
        user_management::auxiliaries::UpdateProfileError,
        CurrentUser,
    },
    session_state::TypedSession,
};

/// # `change_username_post`
///
/// Response to post "/profile/change_username"
/// TODO: refactoring
#[tracing::instrument(
    name = "Change Username",
    skip(form, pool, session),
    fields(
        old_username=tracing::field::Empty,
        new_username=tracing::field::Empty,
    )
)]
pub async fn change_username_post(
    session: TypedSession,
    pool: Data<PgPool>,
    form: Form<ChangeUsernameData>,
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
    let (new_username, password, old_username) =
        match get_infos_for_username(&form.0, &current_user) {
            Ok(t) => t,
            Err(e) => {
                FlashMessage::warning(&format!("{}", e)).send();
                return Ok(HttpResponse::SeeOther()
                    .insert_header((LOCATION, "/profile/change_username"))
                    .finish());
            }
        };

    tracing::Span::current().record(
        "old_username",
        &tracing::field::display(old_username.as_ref()),
    );
    tracing::Span::current().record(
        "new_username",
        &tracing::field::display(new_username.as_ref()),
    );

    let credentials = BasicCredentials {
        username: old_username,
        password,
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
    match update_name(&pool, new_username.as_ref(), &current_user.id).await {
        Ok(()) => {
            FlashMessage::info(format!(
                "Your username has been updated to {}",
                new_username.as_ref()
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

/// `update_name`
///
/// `change_username`'s helper, updates the database with the new username.
async fn update_name(
    pool: &PgPool,
    username: &str,
    user_id: &Uuid,
) -> Result<(), UpdateProfileError> {
    let res = sqlx::query("SELECT id FROM users WHERE username = $1")
        .bind(username)
        .fetch_optional(pool)
        .await;
    match res {
        Ok(opt) => match opt {
            Some(_) => {
                return Err(anyhow::anyhow!(
                    "The new name you provided is already taken, please try again."
                )
                .into());
            }
            None => {}
        },
        Err(e) => {
            return Err(e.into());
        }
    }
    sqlx::query("UPDATE users SET username = $1 WHERE (id = $2)")
        .bind(username)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// `get_infos_for_username`
///
/// `change_email_post`'s helper function that simplifies the code
/// in exchange for a small amount of resources.
/// Returns new_username, password and old_username in this order, or an error.
fn get_infos_for_username(
    form: &ChangeUsernameData,
    current_user: &CurrentUser,
) -> Result<(ValidUserName, ValidPassword, ValidUserName), anyhow::Error> {
    let new_username = ValidUserName::parse(&form.username)?;
    let password = ValidPassword::parse(&form.password)?;
    let old_username = ValidUserName::parse(&current_user.username)?;

    Ok((new_username, password, old_username))
}

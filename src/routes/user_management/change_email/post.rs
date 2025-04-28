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
        CurrentUser,
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
    let (new_email, password, old_email, username) =
        match get_infos_for_email(&form.0, &current_user) {
            Ok(t) => t,
            Err(e) => {
                FlashMessage::warning(format!("{}", e)).send();
                return Ok(HttpResponse::SeeOther()
                    .insert_header((LOCATION, "/profile/change_email"))
                    .finish());
            }
        };

    let credentials = BasicCredentials { username, password };

    tracing::Span::current().record("old_email", tracing::field::display(old_email.as_ref()));
    tracing::Span::current().record("new_email", tracing::field::display(new_email.as_ref()));

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
                AuthError::UnexpectedError(e) => return Err(e500(e).await),
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
                FlashMessage::warning(format!("{}", err)).send();
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

/// `update_email`
///
/// `change_email_post`'s helper function, updates the database with the new email
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
        Ok(opt) => {
            if opt.is_some() {
                return Err(anyhow::anyhow!(
                    "The new email you provided is already taken, please try again."
                )
                .into());
            }
        }
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

/// `get_infos_for_email`
///
/// `change_email_post`'s helper function that simplifies the code
/// in exchange for a small amount of resources.
/// Returns new_email, password, old_email, username in this order, or an error.
fn get_infos_for_email(
    form: &ChangeEmailData,
    current_user: &CurrentUser,
) -> Result<(ValidEmail, ValidPassword, ValidEmail, ValidUserName), anyhow::Error> {
    let new_email = ValidEmail::parse(&form.email)?;
    let password = ValidPassword::parse(&form.password)?;
    let old_email = ValidEmail::parse(&current_user.email)?;
    let username = ValidUserName::parse(&current_user.username)?;
    Ok((new_email, password, old_email, username))
}

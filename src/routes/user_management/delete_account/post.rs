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
pub struct DeleteAccountData {
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
#[tracing::instrument(
    name = "Delete Account",
    skip(form, pool, session),
    fields(
        user_id=tracing::field::Empty,
        username=tracing::field::Empty,
    )
)]
pub async fn delete_account_post(
    session: TypedSession,
    pool: Data<PgPool>,
    form: Form<DeleteAccountData>,
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

    tracing::Span::current().record("username", &tracing::field::display(&current_user.username));
    tracing::Span::current().record("user_id", &tracing::field::display(&current_user.id));

    let (password, username) = match get_infos_for_delete_account(&form, &current_user) {
        Ok(t) => t,
        Err(e) => {
            FlashMessage::warning(&format!("{}", e)).send();
            return Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/profile/change_username"))
                .finish());
        }
    };

    let credentials = BasicCredentials { username, password };

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

    match delete_account(&pool, &current_user.id).await {
        Ok(()) => {
            session.logout();
            FlashMessage::info("Your account has been deleted correctly.\nSee you space cowboy...")
                .send();
            Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/"))
                .finish())
        }
        Err(e) => match e {
            UpdateProfileError::InvalidValue(err) => {
                // NOTE: This should not happen
                FlashMessage::warning(&format!("{}", err)).send();
                return Err(InternalError::from_response(
                    err,
                    HttpResponse::SeeOther()
                        .insert_header((LOCATION, "/profile/delete_account"))
                        .finish(),
                ));
            }
            UpdateProfileError::UnexpectedError(e) => {
                return Err(e500(e.into()).await);
            }
        },
    }
}

/// `delete_account`
///
/// `delete_account_post`'s helper, updates the database with the new username.
async fn delete_account(pool: &PgPool, user_id: &Uuid) -> Result<(), UpdateProfileError> {
    sqlx::query("DELETE FROM users WHERE (id = $1)")
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| UpdateProfileError::UnexpectedError(e))?;
    Ok(())
}

/// `get_infos_for_delete_account`
///
/// `delete_account_post`'s helper function that simplifies the code
/// in exchange for a small amount of resources.
/// Returns password and username in this order, or an error.
fn get_infos_for_delete_account(
    form: &DeleteAccountData,
    current_user: &CurrentUser,
) -> Result<(ValidPassword, ValidUserName), anyhow::Error> {
    let password = ValidPassword::parse(&form.password)?;
    let username = ValidUserName::parse(&current_user.username)?;

    Ok((password, username))
}

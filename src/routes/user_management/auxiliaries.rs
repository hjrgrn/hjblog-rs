use std::fmt::Debug;

use actix_web::{http::header::LOCATION, HttpResponse, Responder};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use askama::Template;
use sqlx::PgPool;

use crate::{
    auxiliaries::error_chain_fmt,
    routes::{auxiliaries::get_flash_messages, errors::e500},
    session_state::TypedSession,
};

use super::{
    change_email::get::ChangeEmailTemplate, change_password::get::ChangePasswordTemplate,
    change_username::get::ChangeUsernameTemplate, delete_account::get::DeleteAccountTemplate,
    manage_profile::ManageProfileTemplate,
};

#[derive(thiserror::Error)]
pub enum UpdateProfileError {
    #[error(transparent)]
    InvalidValue(#[from] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] sqlx::Error),
}

impl Debug for UpdateProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

pub enum Mode {
    ChangeUsername,
    ChangeEmail,
    ChangePassword,
    DeleteAccount,
    ManageProfile,
}

pub async fn user_management_get_requests(
    messages: &IncomingFlashMessages,
    session: &TypedSession,
    pool: &PgPool,
    mode: Mode,
) -> Result<impl Responder, actix_web::error::InternalError<anyhow::Error>> {
    let flash_messages = get_flash_messages(&messages);
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

    let body = match mode {
        Mode::ChangeUsername => {
            let ctx = ChangeUsernameTemplate {
                title: Some(String::from("Change Username")),
                flash_messages,
                current_user,
            };
            match ctx.render() {
                Ok(c) => c,
                Err(e) => {
                    return Err(e500(e.into()).await);
                }
            }
        }
        Mode::ChangeEmail => {
            let ctx = ChangeEmailTemplate {
                title: Some(String::from("Change Email")),
                flash_messages,
                current_user,
            };
            match ctx.render() {
                Ok(c) => c,
                Err(e) => {
                    return Err(e500(e.into()).await);
                }
            }
        }
        Mode::ChangePassword => {
            let ctx = ChangePasswordTemplate {
                title: Some(String::from("Change Password")),
                flash_messages,
                current_user,
            };
            match ctx.render() {
                Ok(c) => c,
                Err(e) => {
                    return Err(e500(e.into()).await);
                }
            }
        }
        Mode::DeleteAccount => {
            let ctx = DeleteAccountTemplate {
                title: Some(String::from("Delete Account")),
                flash_messages,
                current_user,
            };
            match ctx.render() {
                Ok(c) => c,
                Err(e) => {
                    return Err(e500(e.into()).await);
                }
            }
        }
        Mode::ManageProfile => {
            let ctx = ManageProfileTemplate {
                title: Some(format!("Manage your profile {}", current_user.username).into()),
                flash_messages,
                current_user,
            };
            match ctx.render() {
                Ok(c) => c,
                Err(e) => {
                    return Err(e500(e.into()).await);
                }
            }
        }
    };

    Ok(HttpResponse::Ok().body(body))
}

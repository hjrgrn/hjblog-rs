use crate::{
    routes::{
        auxiliaries::FormattedFlashMessage,
        user_management::auxiliaries::{user_management_get_requests, Mode},
        CurrentUser,
    },
    session_state::TypedSession,
};
use actix_web::{web::Data, Responder};
use actix_web_flash_messages::IncomingFlashMessages;
use askama_actix::Template;
use sqlx::PgPool;

#[derive(Template)]
#[template(path = "profile/delete_account.html")]
pub struct DeleteAccountTemplate {
    pub title: Option<String>,
    pub flash_messages: Option<Vec<FormattedFlashMessage>>,
    pub current_user: CurrentUser,
}

pub async fn delete_account_get(
    messages: IncomingFlashMessages,
    session: TypedSession,
    pool: Data<PgPool>,
) -> Result<impl Responder, actix_web::error::InternalError<anyhow::Error>> {
    user_management_get_requests(&messages, &session, &pool, Mode::DeleteAccount).await
}

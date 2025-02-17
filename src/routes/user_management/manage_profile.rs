use crate::{
    routes::{auxiliaries::FormattedFlashMessage, CurrentUser},
    session_state::TypedSession,
};
use actix_web::{web::Data, Responder};
use actix_web_flash_messages::IncomingFlashMessages;
use askama_actix::Template;
use sqlx::PgPool;

use super::auxiliaries::{user_management_get_requests, Mode};

#[derive(Template)]
#[template(path = "profile/manage_profile.html")]
pub struct ManageProfileTemplate {
    pub title: Option<String>,
    pub flash_messages: Option<Vec<FormattedFlashMessage>>,
    pub current_user: CurrentUser,
}

pub async fn manage_profile(
    messages: IncomingFlashMessages,
    session: TypedSession,
    pool: Data<PgPool>,
) -> Result<impl Responder, actix_web::error::InternalError<anyhow::Error>> {
    user_management_get_requests(&messages, &session, &pool, Mode::ManageProfile).await
}

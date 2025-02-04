use actix_web_flash_messages::FlashMessage;
use actix_web_flash_messages::IncomingFlashMessages;
use actix_web_flash_messages::Level;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize, sqlx::FromRow)]
pub struct CurrentUser {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub city_id: Option<String>,
    pub is_admin: bool,
    pub profile_pic: Option<String>,
}

pub struct FormattedFlashMessage {
    pub content: String,
    pub level: String,
}

impl From<&FlashMessage> for FormattedFlashMessage {
    fn from(value: &FlashMessage) -> Self {
        let level = match value.level() {
            Level::Info => String::from("alert-success"),
            _ => String::from("alert-danger"),
        };
        FormattedFlashMessage {
            content: value.content().into(),
            level,
        }
    }
}

/// TODO: comment
pub fn get_flash_messages(messages: &IncomingFlashMessages) -> Option<Vec<FormattedFlashMessage>> {
    let mut v: Vec<FormattedFlashMessage> = Vec::new();
    let mut some = false;

    for message in messages.iter() {
        v.push(message.into());
        some = true;
    }

    if some {
        return Some(v);
    }

    None
}

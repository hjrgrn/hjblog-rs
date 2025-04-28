use actix_web::error::InternalError;
use actix_web_flash_messages::FlashMessage;
use actix_web_flash_messages::IncomingFlashMessages;
use actix_web_flash_messages::Level;
use serde::Deserialize;
use uuid::Uuid;

use super::errors::e400;
use super::errors::e500;

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

/// Contains all the information needed to
/// create a correct pagination in some of the routes
pub struct Pagination {
    pub o: u64,
    pub offset: u64,
    pub current_page: u64,
    pub prev_page: u64,
    pub max_page: u64,
    pub next_page: u64,
    pub more: bool,
    pub path: String,
}

#[derive(Deserialize)]
pub struct PaginationData {
    pub o: Option<u64>,
    pub index: Option<u64>,
}

impl Pagination {
    /// Get the link for the page indicated by `index`
    pub fn get_link(&self, index: &u64) -> String {
        format!("{}?index={}&o={}", self.path, index, self.o)
    }
    /// Get the link for the previous page
    pub fn get_previous_page_link(&self) -> String {
        let o = self.o.checked_sub(1).unwrap_or_default();
        format!("{}?index=0&o={}", self.path, o)
    }
    /// Get the link for the next page
    pub fn get_next_page_link(&self) -> String {
        let o = match self.o.checked_add(1) {
            Some(o) => o,
            None => u64::MAX,
        };
        format!("{}?index=0&o={}", self.path, o)
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

/// TODO: comment
/// Helper function, makes `blog_get` more managable at the expense of memory usage
/// returns some of the indexes needed for pagination: `current_page`, `prev_page`, `next_page`, and `max_page`
pub async fn get_indexes(
    count: u64,
    max_per_page: u64,
    page_span: u64,
    index: Option<u64>,
) -> Result<(u64, u64, u64, u64), InternalError<anyhow::Error>> {
    let max_page = match count.checked_div(max_per_page) {
        Some(mp) => mp,
        None => {
            return Err(e500(anyhow::anyhow!(
                "This should not have happened, worng `max_per_page` parameter"
            ))
            .await);
        }
    };

    let mut current_page = index.unwrap_or_default();
    if current_page > max_page {
        current_page = max_page;
    }
    let prev_page = current_page.checked_sub(page_span).unwrap_or_default();
    let mut next_page = match current_page.checked_add(page_span) {
        Some(n) => n,
        None => {
            return Err(e400(anyhow::anyhow!(
                "User has tryied to tamper with `index` query parameter"
            ))
            .await);
        }
    };
    if next_page > max_page {
        next_page = max_page;
    }

    Ok((current_page, prev_page, next_page, max_page))
}

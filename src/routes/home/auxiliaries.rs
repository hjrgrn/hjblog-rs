use chrono::{DateTime, Local};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize, sqlx::FromRow)]
pub struct Post {
    pub id: Uuid,
    pub username: String,
    pub title: String,
    pub content: String,
    pub posted: DateTime<Local>,
}

impl Post {
    pub fn get_path(&self) -> String {
        format!("/user/visit_post/{}", self.id)
    }
}

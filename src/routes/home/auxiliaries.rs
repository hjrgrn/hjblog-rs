use chrono::{DateTime, Local};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize, sqlx::FromRow, Debug)]
pub struct Post {
    pub id: Uuid,
    pub username: String,
    pub title: String,
    pub content: String,
    pub posted: DateTime<Local>,
    pub author_id: Uuid,
}

impl Post {
    pub fn get_path(&self) -> String {
        format!("/user_actions/visit_post/{}", self.id)
    }
}

#[derive(Deserialize, sqlx::FromRow)]
pub struct Comment {
    pub id: Uuid,
    pub post_id: Uuid,
    pub content: String,
    pub author_id: Uuid,
    pub written: DateTime<Local>,
    pub username: String,
}

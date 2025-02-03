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

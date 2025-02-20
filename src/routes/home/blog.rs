use std::ops::{Add, Sub};

use actix_web::{error::InternalError, web};
use actix_web_flash_messages::IncomingFlashMessages;
use askama_actix::Template;
use serde::Deserialize;
use sqlx::{query, query_as, PgPool, Row};

use crate::{
    routes::{
        auxiliaries::{get_flash_messages, FormattedFlashMessage},
        errors::{e400, e500},
        CurrentUser,
    },
    session_state::TypedSession,
};

use super::auxiliaries::Post;

#[derive(Template)]
#[template(path = "home/blog.html")]
pub struct BlogTemplate {
    pub title: Option<String>,
    pub current_user: Option<CurrentUser>,
    pub posts: Option<Vec<Post>>,
    pub flash_messages: Option<Vec<FormattedFlashMessage>>,
    pub pagination: Pagination,
}

#[derive(Deserialize)]
pub struct BlogGetData {
    o: Option<u64>,
    index: Option<u64>,
}

pub struct Pagination {
    pub o: u64,
    pub offset: u64,
    pub current_page: u64,
    pub prev_page: u64,
    pub max_page: u64,
    pub next_page: u64,
    pub more_posts: bool,
}

/// TODO: comment, operations
impl Pagination {
    pub fn get_link(&self, index: &u64) -> String {
        format!("/blog?index={}&o={}", index, self.o)
    }
    pub fn get_previous_posts_link(&self) -> String {
        format!("/blog?index=0&o={}", self.o.sub(1))
    }

    pub fn get_next_posts_link(&self) -> String {
        format!("/blog?index=0&o={}", self.o.add(1))
    }
}

/// TODO: comment, refactoring, bug fixing, limitations
pub async fn blog_get(
    pool: web::Data<PgPool>,
    session: TypedSession,
    messages: IncomingFlashMessages,
    query_data: web::Query<BlogGetData>,
) -> Result<BlogTemplate, InternalError<anyhow::Error>> {
    let flash_messages = get_flash_messages(&messages);
    let current_user = match session.get_current_user(&pool).await {
        Ok(cu) => cu,
        Err(e) => {
            return Err(e500(e.into()).await);
        }
    };

    let (o, offset) = match query_data.o {
        // TODO: take care of this
        Some(o) => (o, o.checked_mul(100).unwrap_or(u64::MAX)),
        None => (0, 0),
    };

    // max page index for pagination
    let row = match query("SELECT COUNT(id) FROM posts")
        .fetch_one(pool.as_ref())
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return Err(e500(e.into()).await);
        }
    };
    let count: i64 = match row.try_get("count") {
        Ok(c) => c,
        Err(e) => {
            return Err(e500(e.into()).await);
        }
    };
    let count = if count < 0 { 0 } else { count as u64 };

    // TODO: get this from configuration
    let max_per_page = 5;
    let page_span = 3;
    let max_page;

    let mut count = match count.checked_sub(offset.into()) {
        Some(c) => c,
        None => {
            return Err(e400(anyhow::anyhow!(
                "User has tryied to tamper with `o` query parameter in `/blog` route"
            ))
            .await)
        }
    };
    let mut more_posts = false;
    if count > 99 {
        count = 99;
        more_posts = true;
    }
    if count == 0 {
        if offset > 0 {
            return Err(e400(anyhow::anyhow!(
                "User has tryied to tamper with `o` query parameter in `/blog` route"
            ))
            .await);
        }
        let pagination = Pagination {
            o,
            offset,
            more_posts,
            current_page: 0,
            next_page: 0,
            prev_page: 0,
            max_page: 0,
        };

        return Ok(BlogTemplate {
            title: Some("Blog".to_string()),
            current_user,
            posts: None,
            flash_messages,
            pagination,
        });
    }
    max_page = match count.checked_div(max_per_page) {
        Some(mp) => mp,
        None => {
            return Err(e500(anyhow::anyhow!(
                "This should not have happened, worng `max_per_page` parameter"
            ))
            .await);
        }
    };

    let mut current_page = match query_data.index {
        Some(i) => i,
        None => 0,
    };
    if current_page > max_page {
        current_page = max_page;
    }
    let prev_page = match current_page.checked_sub(page_span) {
        Some(p) => p,
        None => 0,
    };
    let mut next_page = match current_page.checked_add(page_span) {
        Some(n) => n,
        None => {
            return Err(e400(anyhow::anyhow!(
                "User has tryied to tamper with `index` query parameter in `/blog` route"
            ))
            .await);
        }
    };
    if next_page > max_page {
        next_page = max_page
    }

    let addend = match current_page.checked_mul(max_per_page) {
        Some(a) => a,
        None => {
            return Err(e400(anyhow::anyhow!(
                "User has tryied to tamper with `index` query parameter in `/blog` route"
            ))
            .await);
        }
    };
    let start = match offset.checked_add(addend) {
        Some(s) => s,
        None => {
            return Err(e400(anyhow::anyhow!(
                "User has tryied to tamper with `index` query parameter in `/blog` route"
            ))
            .await);
        }
    };

    let posts = match query_as::<_, Post>(
        "SELECT posts.id, users.username, posts.title, posts.content, posts.posted \
        FROM posts JOIN users ON (users.id = posts.author_id) \
        ORDER BY posts.posted DESC, posts.id DESC LIMIT ($1) OFFSET ($2)",
    )
    .bind(max_per_page as i64)
    .bind(start as i64)
    .fetch_all(pool.as_ref())
    .await
    {
        Ok(p) => p,
        Err(e) => {
            return Err(e500(e.into()).await);
        }
    };

    let pagination = Pagination {
        o,
        offset,
        more_posts,
        current_page,
        prev_page,
        next_page,
        max_page,
    };

    Ok(BlogTemplate {
        title: Some("Blog".to_string()),
        current_user,
        posts: Some(posts),
        flash_messages,
        pagination,
    })
}

use actix_web::{error::InternalError, web};
use actix_web_flash_messages::IncomingFlashMessages;
use askama_actix::Template;
use sqlx::{query, query_as, PgPool, Row};

use crate::{
    routes::{
        auxiliaries::{
            get_flash_messages, get_indexes, FormattedFlashMessage, Pagination, PaginationData,
        },
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

/// `blog_get`
///
/// Handler that hanldes a get request for `/blog` route.
pub async fn blog_get(
    pool: web::Data<PgPool>,
    session: TypedSession,
    messages: IncomingFlashMessages,
    query_data: web::Query<PaginationData>,
) -> Result<BlogTemplate, InternalError<anyhow::Error>> {
    let flash_messages = get_flash_messages(&messages);
    let current_user = match session.get_current_user(&pool).await {
        Ok(cu) => cu,
        Err(e) => {
            return Err(e500(e.into()).await);
        }
    };

    let (o, offset) = match query_data.o {
        Some(o) => (o, o.checked_mul(100).unwrap_or(u64::MAX)),
        None => (0, 0),
    };

    let (count, more_posts) = get_count(&pool, offset).await?;

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
            more: more_posts,
            current_page: 0,
            next_page: 0,
            prev_page: 0,
            max_page: 0,
            path: String::from("/blog"),
        };

        return Ok(BlogTemplate {
            title: Some("Blog".to_string()),
            current_user,
            posts: None,
            flash_messages,
            pagination,
        });
    }

    // TODO: get this from configuration
    let max_per_page = 5;
    let page_span = 3;

    let (current_page, prev_page, next_page, max_page) =
        get_indexes(count, max_per_page, page_span, query_data.index).await?;

    let posts = get_posts(current_page, max_per_page, offset, &pool).await?;

    let pagination = Pagination {
        o,
        offset,
        more: more_posts,
        current_page,
        prev_page,
        next_page,
        max_page,
        path: String::from("/blog"),
    };

    Ok(BlogTemplate {
        title: Some("Blog".to_string()),
        current_user,
        posts: Some(posts),
        flash_messages,
        pagination,
    })
}

/// Helper function, makes `blog_get` more managable
/// returns `count` and `more_posts`
#[tracing::instrument(
    name = "Calculating count and more_posts using database",
    skip(pool, offset)
)]
pub async fn get_count(
    pool: &PgPool,
    offset: u64,
) -> Result<(u64, bool), InternalError<anyhow::Error>> {
    // max page index for pagination
    let row = match query("SELECT COUNT(id) FROM posts").fetch_one(pool).await {
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

    let mut count = match count.checked_sub(offset) {
        Some(c) => c,
        None => {
            return Err(e400(anyhow::anyhow!(
                "User has tryied to tamper with `o` query parameter in `/blog` route"
            ))
            .await)
        }
    };
    let mut more_posts = false;
    if count > 100 {
        count = 99;
        more_posts = true;
    }
    Ok((count, more_posts))
}

/// Helper function, makes `blog_get` more managable at the expense of memory usage
/// returns posts
#[tracing::instrument(
    name = "Fetching posts from database",
    skip(current_page, max_per_page, offset, pool)
)]
pub async fn get_posts(
    current_page: u64,
    max_per_page: u64,
    offset: u64,
    pool: &PgPool,
) -> Result<Vec<Post>, InternalError<anyhow::Error>> {
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

    Ok(
        match query_as::<_, Post>(
            "SELECT \
                posts.id, \
                users.username, \
                posts.title, \
                posts.content,
                posts.posted, \
                posts.author_id \
            FROM posts JOIN users ON (users.id = posts.author_id) \
            ORDER BY posts.posted DESC, posts.id DESC LIMIT ($1) OFFSET ($2)",
        )
        .bind(max_per_page as i64)
        .bind(start as i64)
        .fetch_all(pool)
        .await
        {
            Ok(p) => p,
            Err(e) => {
                return Err(e500(e.into()).await);
            }
        },
    )
}

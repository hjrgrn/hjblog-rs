use crate::{
    routes::{
        auxiliaries::{
            get_flash_messages, get_indexes, FormattedFlashMessage, Pagination, PaginationData,
        },
        errors::{e400, e403, e500},
        home::auxiliaries::{Comment, Post},
        CurrentUser,
    },
    session_state::TypedSession,
};
use actix_web::{
    error::InternalError,
    web::{self, Path},
};
use actix_web_flash_messages::IncomingFlashMessages;
use askama_actix::Template;
use sqlx::{query, query_as, PgPool, Row};
use uuid::Uuid;

#[derive(Template)]
#[template(path = "user_actions/all_comments.html")]
pub struct AllCommentsTemplate {
    pub title: Option<String>,
    pub current_user: Option<CurrentUser>,
    pub post: Post,
    pub flash_messages: Option<Vec<FormattedFlashMessage>>,
    pub identified: bool,
    pub comments: Vec<Comment>,
    pub pagination: Pagination,
}

// TODO: comment, telemetry, refactoring
pub async fn all_comments_get(
    pool: web::Data<PgPool>,
    session: TypedSession,
    messages: IncomingFlashMessages,
    post_id: Path<Uuid>,
    query_data: web::Query<PaginationData>,
) -> Result<AllCommentsTemplate, InternalError<anyhow::Error>> {
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

    let (count, more) = get_count(&pool, offset, post_id.as_ref()).await?;

    let post = match query_as::<_, Post>(
        "SELECT \
            posts.id, \
            users.username, \
            posts.title, \
            posts.content, \
            posts.posted, \
            posts.author_id \
        FROM posts JOIN users ON (users.id = posts.author_id) \
        WHERE posts.id = $1",
    )
    .bind(post_id.as_ref())
    .fetch_optional(pool.as_ref())
    .await
    {
        Ok(opt) => {
            match opt {
                Some(p) => p,
                None => {
                    // user provided a non existing post id
                    return Err(e403(anyhow::anyhow!("User provided a false post id")).await);
                }
            }
        }
        Err(e) => {
            return Err(e500(e.into()).await);
        }
    };

    let identified = match &current_user {
        Some(cu) => {
            if cu.is_admin {
                true
            } else {
                cu.id == post.author_id
            }
        }
        None => false,
    };

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
            more,
            current_page: 0,
            next_page: 0,
            prev_page: 0,
            max_page: 0,
            path: format!("/user_actions/all_comments/{}", post.id),
        };

        return Ok(AllCommentsTemplate {
            title: Some("All Comments".to_string()),
            current_user,
            post,
            flash_messages,
            pagination,
            identified,
            comments: Vec::new(),
        });
    }

    // TODO: get this from configuration
    let max_per_page = 5;
    let page_span = 3;

    let (current_page, prev_page, next_page, max_page) =
        get_indexes(count, max_per_page, page_span, query_data.index).await?;

    let comments =
        get_comments(current_page, max_per_page, offset, &pool, post_id.as_ref()).await?;

    let pagination = Pagination {
        o,
        offset,
        more,
        current_page,
        next_page,
        prev_page,
        max_page,
        path: format!("/user_actions/all_comments/{}", post.id),
    };

    Ok(AllCommentsTemplate {
        title: Some("Blog".to_string()),
        current_user,
        post,
        flash_messages,
        pagination,
        identified,
        comments,
    })
}

/// Helper function, makes `all_comments_get` more manageable
/// returns `count` and `more`
/// FIX: duplication in blog route
#[tracing::instrument(
    name = "Calculating count and more_comments using database",
    skip(pool, offset)
)]
pub async fn get_count(
    pool: &PgPool,
    offset: u64,
    post_id: &Uuid,
) -> Result<(u64, bool), InternalError<anyhow::Error>> {
    // max page index for pagination
    let row = match query("SELECT COUNT(id) FROM comments WHERE post_id = $1")
        .bind(post_id)
        .fetch_one(pool)
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

    let mut count = match count.checked_sub(offset) {
        Some(c) => c,
        None => {
            return Err(e400(anyhow::anyhow!(
                "User has tryied to tamper with `o` query parameter in `/user_actions/all_comments` route"
            ))
            .await)
        }
    };
    let mut more_comments = false;
    if count > 100 {
        count = 99;
        more_comments = true;
    }
    Ok((count, more_comments))
}

/// Helper function, makes `all_comments_get` more managable at the expense of memory usage
/// returns posts
/// FIX: code duplication in `blog`
#[tracing::instrument(
    name = "Fetching comments from database",
    skip(current_page, max_per_page, offset, pool)
)]
pub async fn get_comments(
    current_page: u64,
    max_per_page: u64,
    offset: u64,
    pool: &PgPool,
    post_id: &Uuid,
) -> Result<Vec<Comment>, InternalError<anyhow::Error>> {
    let addend = match current_page.checked_mul(max_per_page) {
        Some(a) => a,
        None => {
            return Err(e400(anyhow::anyhow!(
                "User has tryied to tamper with `index` query parameter in `/user_actions/all_comments_get` route"
            ))
            .await);
        }
    };
    let start = match offset.checked_add(addend) {
        Some(s) => s,
        None => {
            return Err(e400(anyhow::anyhow!(
                "User has tryied to tamper with `index` query parameter in `/all_comments_get` route"
            ))
            .await);
        }
    };

    Ok(
        match query_as::<_, Comment>(
            "SELECT \
                comments.id, \
                comments.post_id, \
                comments.content, \
                comments.author_id, \
                comments.written, \
                users.username \
            FROM comments JOIN users ON (users.id = comments.author_id) \
            WHERE \
                comments.post_id = $1 \
            ORDER BY comments.written DESC, comments.id DESC LIMIT ($2) OFFSET ($3)",
        )
        .bind(post_id)
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

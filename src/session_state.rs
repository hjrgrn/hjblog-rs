use std::future::{ready, Ready};

use actix_session::{Session, SessionExt, SessionGetError, SessionInsertError};
use actix_web::FromRequest;
use sqlx::{query_as, PgPool};
use uuid::Uuid;

use crate::routes::CurrentUser;

pub struct TypedSession(Session);

impl TypedSession {
    const USER_ID_KEY: &'static str = "user_id";

    pub fn renew(&self) {
        self.0.renew();
    }

    pub fn insert_user_id(&self, user_id: Uuid) -> Result<(), SessionInsertError> {
        self.0.insert(Self::USER_ID_KEY, user_id)
    }

    pub fn get_user_id(&self) -> Result<Option<Uuid>, SessionGetError> {
        self.0.get(Self::USER_ID_KEY)
    }

    pub async fn get_current_user(
        &self,
        pool: &PgPool,
    ) -> Result<Option<CurrentUser>, CurrentUserError> {
        let id = match self.get_user_id()? {
            Some(id) => id,
            None => {
                return Ok(None);
            }
        };
        match get_current_user(id, pool).await? {
            Some(cu) => Ok(Some(cu)),
            // NOTE: a key that is not in our database has been provided,
            // given our cookies are private we either have a bug or our
            // key has been compromised
            None => Err(CurrentUserError::UnexpectedError(anyhow::anyhow!(
                "Unexpected error occurred, the secret key may be compromised."
            ))),
        }
    }
}

impl FromRequest for TypedSession {
    type Error = <Session as FromRequest>::Error;
    // NOTE: `Ready` becouse it completes the first time it is polled
    type Future = Ready<Result<TypedSession, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        ready(Ok(TypedSession(req.get_session())))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CurrentUserError {
    #[error(transparent)]
    SessionGetError(#[from] SessionGetError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("Unexpected error")]
    UnexpectedError(#[from] anyhow::Error),
}

/// TODO: comment, telemetry, error, incorporate it into TypedSession
#[tracing::instrument(name = "Extracting current user from database", skip(pool))]
async fn get_current_user(
    user_id: Uuid,
    pool: &PgPool,
) -> Result<Option<CurrentUser>, sqlx::Error> {
    query_as::<_, CurrentUser>(
        "SELECT id, username, email, city_id, is_admin, profile_pic FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

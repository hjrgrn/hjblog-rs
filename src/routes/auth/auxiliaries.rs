use anyhow::Context;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use secrecy::{ExposeSecret, SecretString};
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

use crate::telemetry::spawn_blocking_with_tracing;

#[derive(Debug)]
pub struct Credentials {
    pub username: String,
    pub password: SecretString,
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials.")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

/// TODO: comment, refactor
/// NOTE: this function prevent timing attack by performing the same amount of work even if it didn't receive
/// a valid username, [OWASP](https://owasp.org/www-project-web-security-testing-guide/latest/4-Web_Application_Security_Testing/03-Identity_Management_Testing/04-Testing_for_Account_Enumeration_and_Guessable_User_Account).
#[tracing::instrument(name = "Validate credentials", skip(credentials, pool))]
pub async fn validate_credentials(
    credentials: Credentials,
    pool: &PgPool,
) -> Result<uuid::Uuid, AuthError> {
    let mut user_id = None;
    let mut expected_password_hash = SecretString::new("$argon2id$v=19$m=19456,t=2,p=1$whUl96AIbUiqrY6CINRAKg$+7nxehFtPiM0dXnxD9Ln0BMEi2SwZFOf8YDlXzJd8TU".into());
    if let Some((stored_user_id, stored_password_hash)) =
        get_stored_credentials(&credentials.username, &pool).await?
    {
        user_id = Some(stored_user_id);
        expected_password_hash = stored_password_hash;
    };

    spawn_blocking_with_tracing(move || {
        verify_password_hash(expected_password_hash, credentials.password)
    })
    .await
    .context("Failed to spawn blocking task")??;

    // `Some` only if found credentials in the store
    user_id
        .ok_or_else(|| anyhow::anyhow!("Unknown username."))
        .map_err(AuthError::InvalidCredentials)
}

/// TODO: comment, refactor
pub async fn get_stored_credentials(
    username: &str,
    pool: &PgPool,
) -> Result<Option<(uuid::Uuid, SecretString)>, anyhow::Error> {
    let row = sqlx::query("SELECT id, hash_pass FROM users WHERE username = $1")
        .bind(username)
        .fetch_optional(pool)
        .await
        .context("Failed to perform a query to retrive stored credentials.")?;

    let row = match row {
        Some(r) => {
            let user_id: Uuid = match r.try_get("id") {
                Ok(id) => id,
                Err(e) => {
                    return Err(anyhow::anyhow!(e));
                }
            };
            let hash_pass: String = match r.try_get("hash_pass") {
                Ok(hp) => hp,
                Err(e) => {
                    return Err(anyhow::anyhow!(e));
                }
            };
            Some((user_id, SecretString::new(hash_pass.into())))
        }
        None => None,
    };

    Ok(row)
}

#[tracing::instrument(
    name = "Verify password hash",
    skip(expected_password_hash, password_candidate)
)]
pub fn verify_password_hash(
    expected_password_hash: SecretString,
    password_candidate: SecretString,
) -> Result<(), AuthError> {
    let expected_password_hash = PasswordHash::new(expected_password_hash.expose_secret())
        .context("Failed to parse hash in PHC string format.")?;

    Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .context("Invalid password")
        .map_err(AuthError::InvalidCredentials)
}

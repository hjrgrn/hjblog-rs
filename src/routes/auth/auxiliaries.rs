use std::fmt::Debug;

use anyhow::Context;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand::rngs::OsRng;
use secrecy::{ExposeSecret, SecretString};
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

use crate::auxiliaries::error_chain_fmt;
use crate::domain::ValidUserName;
use crate::domain::{ValidEmail, ValidPassword};
use crate::telemetry::spawn_blocking_with_tracing;

#[derive(Debug)]
pub struct BasicCredentials {
    pub username: ValidUserName,
    pub password: ValidPassword,
}

#[derive(Debug)]
pub struct RegisterCredentials {
    username: ValidUserName,
    email: ValidEmail,
    password: ValidPassword,
}

impl RegisterCredentials {
    pub fn parse(
        username: &str,
        email: &str,
        password: &SecretString,
        confirm_password: &SecretString,
    ) -> Result<Self, anyhow::Error> {
        if password.expose_secret() != confirm_password.expose_secret() {
            return Err(anyhow::anyhow!(
                "You typed two different passwords. Try again."
            ));
        }

        let username = ValidUserName::parse(username)?;
        let email = ValidEmail::parse(email)?;
        let password = ValidPassword::parse(&password)?;

        Ok(Self {
            username,
            email,
            password,
        })
    }

    pub fn get_username(&self) -> &str {
        self.username.as_ref()
    }

    pub fn get_email(&self) -> &str {
        self.email.as_ref()
    }
}

#[derive(thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials.")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl Debug for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[derive(thiserror::Error)]
pub enum RegisterError {
    #[error(transparent)]
    InvalidCredentials(#[from] anyhow::Error),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Argon2(#[from] argon2::password_hash::Error),
}

impl Debug for RegisterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

/// TODO: comment
/// NOTE: this function prevent timing attack by performing the same amount of work even if it didn't receive
/// a valid username, [OWASP](https://owasp.org/www-project-web-security-testing-guide/latest/4-Web_Application_Security_Testing/03-Identity_Management_Testing/04-Testing_for_Account_Enumeration_and_Guessable_User_Account).
#[tracing::instrument(name = "Validate credentials", skip(credentials, pool))]
pub async fn validate_basic_credentials(
    credentials: BasicCredentials,
    pool: &PgPool,
) -> Result<uuid::Uuid, AuthError> {
    let mut user_id = None;
    let mut expected_password_hash = SecretString::new("$argon2id$v=19$m=19456,t=2,p=1$whUl96AIbUiqrY6CINRAKg$+7nxehFtPiM0dXnxD9Ln0BMEi2SwZFOf8YDlXzJd8TU".into());
    if let Some((stored_user_id, stored_password_hash)) =
        get_stored_credentials(&credentials.username.as_ref(), &pool).await?
    {
        user_id = Some(stored_user_id);
        expected_password_hash = stored_password_hash;
    };

    spawn_blocking_with_tracing(move || {
        verify_password_hash(
            &expected_password_hash,
            credentials.password.as_ref(),
        )
    })
    .await
    .context("Failed to spawn blocking task")??;

    // `Some` only if found credentials in the store
    user_id
        .ok_or_else(|| anyhow::anyhow!("Unknown username."))
        .map_err(AuthError::InvalidCredentials)
}

/// TODO: comment
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

/// TODO: comment
#[tracing::instrument(
    name = "Verify password hash",
    skip(expected_password_hash, password_candidate)
)]
pub fn verify_password_hash(
    expected_password_hash: &SecretString,
    password_candidate: &SecretString,
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

/// TODO: comment, refactor
#[tracing::instrument(
    name = "Registering a new user",
    skip(pool, credentials),
    fields(
        username=tracing::field::Empty,
        email=tracing::field::Empty,
        user_id=tracing::field::Empty
    )
)]
pub async fn register_user(
    credentials: &RegisterCredentials,
    pool: &PgPool,
    admin: bool,
) -> Result<Uuid, RegisterError> {
    let user_id = Uuid::new_v4();
    let row = sqlx::query("SELECT id FROM users WHERE username = $1")
        .bind(credentials.username.as_ref())
        .fetch_optional(pool)
        .await?;
    match row {
        Some(_) => return Err(anyhow::anyhow!("Your credentials are already taken.").into()),
        None => {}
    }

    let row = sqlx::query("SELECT id FROM users WHERE email = $1")
        .bind(credentials.email.as_ref())
        .fetch_optional(pool)
        .await?;
    match row {
        Some(_) => return Err(anyhow::anyhow!("Your credentials are already taken.").into()),
        None => {}
    }

    let salt = SaltString::generate(OsRng);
    let hash_pass = Argon2::default()
        .hash_password(credentials.password.expose_secret().as_bytes(), &salt)?
        .to_string();

    sqlx::query(
        r#"INSERT INTO users
(
    id,
    username,
    email,
    hash_pass,
    is_admin,
    is_two_factor_authentication_enabled
) VALUES ($1, $2, $3, $4, $5, $6)
"#,
    )
    .bind(user_id)
    .bind(credentials.username.as_ref())
    .bind(credentials.email.as_ref())
    .bind(hash_pass)
    .bind(admin)
    .bind(false)
    .execute(pool)
    .await?;

    let row = sqlx::query("SELECT id FROM users WHERE (username = $1)")
        .bind(credentials.username.as_ref())
        .fetch_optional(pool)
        .await?;
    match row {
        Some(r) => match r.try_get("id") {
            Ok(id) => Ok(id),
            Err(e) => {
                return Err(anyhow::anyhow!("This error shouldn't have happened:\n{}", e).into());
            }
        },
        None => {
            return Err(anyhow::anyhow!(
                "Failed to find the row I just inserted, this shouldn't have happened."
            )
            .into())
        }
    }
}

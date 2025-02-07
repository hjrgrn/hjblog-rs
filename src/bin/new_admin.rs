//! # `new_admin`
//!
//! CLI tool that creates a new admin account.
//!
//! ## Usage
//!
//! After having initialized the database and provided a valid configuration,
//! in th root directory call:
//! ```bash
//! cargo run --bin new_admin
//! ```
use std::io::{self, BufRead, StdinLock, Write};

use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use crossterm::{
    event::{read, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use hj_blog_rs::{
    domain::{ValidEmail, ValidPassword, ValidUserName},
    settings::get_config,
    telemetry::{get_subscriber, init_subscriber},
};
use rand::rngs::OsRng;
use secrecy::SecretString;
use sqlx::{query, Connection, PgConnection};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let config = get_config().expect("Failed to obtain the config files.");
    let subscriber = get_subscriber("hjblog: new_admin".into(), "info".into(), io::stdout);
    init_subscriber(subscriber);
    let mut connection = PgConnection::connect_with(&config.database.with_db()).await?;

    let mut stdin = io::stdin().lock();

    println!("Creating a new admin account.");

    let username = get_username(&mut stdin, &mut connection).await?;

    let email = get_email(&mut stdin, &mut connection).await?;

    let hash_pass = get_hash_pass()?;

    query(
        "INSERT INTO users (id, username, email, hash_pass, is_admin) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(username.as_ref())
    .bind(email.as_ref())
    .bind(&hash_pass)
    .bind(true)
    .execute(&mut connection)
    .await
    .expect("Failed to create a new user.");

    println!("New admin {} registered correctly", username);

    Ok(())
}

async fn get_username(
    stdin: &mut StdinLock<'static>,
    connection: &mut PgConnection,
) -> Result<ValidUserName, anyhow::Error> {
    print!("Type your name:\n> ");
    std::io::stdout().flush()?;
    let mut username = String::new();
    stdin.read_line(&mut username)?;
    let username = ValidUserName::parse(&username)?;

    match query("SELECT id FROM users WHERE (username = $1)")
        .bind(username.as_ref())
        .fetch_optional(connection)
        .await
    {
        Ok(r) => match r {
            Some(_) => {
                return Err(anyhow::anyhow!(
                    "The name you typed is already taken, try again."
                ));
            }
            None => {}
        },
        Err(e) => return Err(e.into()),
    };

    Ok(username)
}

async fn get_email(
    stdin: &mut StdinLock<'static>,
    connection: &mut PgConnection,
) -> Result<ValidEmail, anyhow::Error> {
    print!("Type your email:\n> ");
    std::io::stdout().flush()?;
    let mut email = String::new();
    stdin.read_line(&mut email)?;
    let email = ValidEmail::parse(&email)?;

    match query("SELECT id FROM users WHERE (email = $1)")
        .bind(email.as_ref())
        .fetch_optional(connection)
        .await
    {
        Ok(r) => match r {
            Some(_) => {
                return Err(anyhow::anyhow!(
                    "The email you typed is already registered, try again."
                ));
            }
            None => {}
        },
        Err(e) => {
            return Err(e.into());
        }
    };
    Ok(email)
}

fn get_hash_pass() -> Result<String, anyhow::Error> {
    print!("Type your password:\n> ");
    std::io::stdout().flush()?;
    let mut password = String::new();

    enable_raw_mode()?;
    while let Ok(event) = read() {
        match event {
            Event::Key(KeyEvent { kind, code, .. }) => {
                if kind == KeyEventKind::Press {
                    match code {
                        KeyCode::Char(c) => {
                            password.push(c);
                        }
                        KeyCode::Enter => {
                            break;
                        }
                        _ => {
                            break;
                        }
                    }
                }
            }
            _ => {
                break;
            }
        }
    }
    disable_raw_mode()?;
    println!("");

    let password = ValidPassword::parse(&SecretString::new(format!("{}", password).into()))?;

    let salt = SaltString::generate(OsRng);
    Ok(Argon2::default()
        .hash_password(password.expose_secret().as_bytes(), &salt)?
        .to_string())
}

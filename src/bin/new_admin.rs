//! TODO: comment
use std::io::{self, BufRead, StdinLock, Write};

use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use crossterm::{
    event::{read, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use hj_blog_rs::{
    settings::get_config,
    telemetry::{get_subscriber, init_subscriber},
};
use rand::rngs::OsRng;
use sqlx::{query, Connection, PgConnection};
use uuid::Uuid;
use validator::ValidateEmail;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let config = get_config().expect("Failed to obtain the config files.");
    let subscriber = get_subscriber("hjblog: new_admin".into(), "info".into(), io::stdout);
    init_subscriber(subscriber);
    let mut connection = PgConnection::connect_with(&config.database.with_db())
        .await
        .expect("Failed to establish a connection to postgres.");

    let mut stdin = io::stdin().lock();

    println!("Creating a new admin account.");

    let username = get_username(&mut stdin, &mut connection).await?;

    let email = get_email(&mut stdin, &mut connection).await?;

    let hash_pass = get_hash_pass()?;

    query(
        "INSERT INTO users (id, username, email, hash_pass, is_admin) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(&username)
    .bind(&email)
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
) -> Result<String, anyhow::Error> {
    print!("Type your name:\n> ");
    std::io::stdout().flush()?;
    let mut username = String::new();
    stdin.read_line(&mut username)?;
    username = username.trim().to_string();
    if username.len() > 60 {
        return Err(anyhow::anyhow!(
            "The name you typed is too long.\nProcedure aborted, try again."
        ));
    }
    match query("SELECT id FROM users WHERE (username = $1)")
        .bind(&username)
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
) -> Result<String, anyhow::Error> {
    print!("Type your email:\n> ");
    std::io::stdout().flush()?;
    let mut email = String::new();
    stdin.read_line(&mut email)?;
    email = email.trim().to_string();
    if email.len() > 200 {
        return Err(anyhow::anyhow!(
            "The email you typed is too long.\nProcedure aborted, try again."
        ));
    }
    if !email.validate_email() {
        return Err(anyhow::anyhow!("The email you typed is invalid"));
    }
    match query("SELECT id FROM users WHERE (email = $1)")
        .bind(&email)
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

    if password.len() > 200 {
        return Err(anyhow::anyhow!(
            "The password you typed is too long.\nProcedure aborted, try again."
        ));
    }
    if password.len() < 3 {
        return Err(anyhow::anyhow!(
            "The password you typed is too long.\nProcedure aborted, try again."
        ));
    }

    let salt = SaltString::generate(OsRng);
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

//! TODO: comment
use std::io::{self, BufRead, Write};

use hj_blog_rs::{
    settings::get_config,
    telemetry::{get_subscriber, init_subscriber},
};
use sqlx::{query, Connection, PgConnection, Row};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let config = get_config().expect("Failed to obtain the config files.");
    let subscriber = get_subscriber("hjblog: new_admin".into(), "info".into(), io::stdout);
    init_subscriber(subscriber);
    let mut connection = PgConnection::connect_with(&config.database.with_db()).await?;

    let author_id: Uuid = match query("SELECT id FROM users WHERE (is_admin = TRUE)")
        .fetch_optional(&mut connection)
        .await
    {
        Ok(r) => match r {
            Some(row) => row.try_get("id").expect("This should not happen"),
            None => {
                return Err(anyhow::anyhow!("Currently there are no admin registered, so I cannot create comments, please run `new_admin` then retry."));
            }
        },
        Err(e) => {
            return Err(e.into());
        }
    };

    let mut stdin = io::stdin().lock();

    let mut posts_amount_str = String::new();
    print!("How many post do you want me to generate(0 to abort)?\n> ");
    std::io::stdout().flush()?;
    stdin.read_line(&mut posts_amount_str)?;
    let mut post_amount: u8 = posts_amount_str.trim().parse()?;
    if post_amount == 0 {
        println!("Procedure aborted as required.");
        return Ok(());
    } else if post_amount > 100 {
        post_amount = 100;
    }

    let mut comments_amount_str = String::new();
    print!("How many comments for every post do you want me to generate?\n> ");
    std::io::stdout().flush()?;
    stdin.read_line(&mut comments_amount_str)?;
    let mut comments_amount: u8 = comments_amount_str.trim().parse()?;
    if comments_amount == 0 {
        println!("Procedure aborted as required.");
        return Ok(());
    } else if comments_amount > 200 {
        comments_amount = 200;
    }

    for i in 0..post_amount {
        let title = format!("test-title-{}", i);
        let content = format!("This is a test,\nContent: {}", i);
        let post_id = Uuid::new_v4();
        match query("INSERT INTO posts (id, title, content, author_id) VALUES ($1, $2, $3, $4)")
            .bind(post_id)
            .bind(&title)
            .bind(&content)
            .bind(author_id)
            .execute(&mut connection)
            .await
        {
            Ok(_) => {}
            Err(e) => {
                if i == 0 {
                    eprintln!("No posts have been created");
                } else {
                    eprintln!("Only {} posts have been created", i);
                }
                return Err(e.into());
            }
        }

        for j in 0..comments_amount {
            let content = format!("Test comment number {}", j);
            let comment_id = Uuid::new_v4();
            match query(
                "INSERT INTO comments (id, post_id, content, author_id) VALUES ($1, $2, $3, $4)",
            )
            .bind(comment_id)
            .bind(post_id)
            .bind(&content)
            .bind(author_id)
            .execute(&mut connection)
            .await
            {
                Ok(_) => {}
                Err(e) => {
                    if i == 0 {
                        eprintln!("No comments for post {} has been created", post_id);
                    } else {
                        eprintln!("Only {} comments for post {} have been created", j, post_id);
                    }
                    return Err(e.into());
                }
            }
        }
    }

    println!("{} posts have been created successfully.", post_amount);

    Ok(())
}

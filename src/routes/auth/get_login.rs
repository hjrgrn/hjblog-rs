use actix_session::Session;
use actix_web::{http::header::LOCATION, HttpResponse, Responder};
use askama_actix::Template;
use uuid::Uuid;

use crate::routes::errors::e500;

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub title: Option<String>,
}

/// TODO: comment, flash messages, refactor, telemetry
pub async fn login(
    session: Session,
) -> Result<impl Responder, actix_web::error::InternalError<anyhow::Error>> {
    let user_id = match session.get::<Uuid>("user_id") {
        Ok(id) => id,
        Err(e) => {
            return Err(e500(e.into()).await);
        }
    };
    match user_id {
        Some(_) => {
            return Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/"))
                .finish());
        }
        None => {}
    }

    let ctx = LoginTemplate {
        title: Some("Sign In".into()),
    };
    let body = match ctx.render() {
        Ok(c) => c,
        Err(e) => {
            return Err(e500(e.into()).await);
        }
    };

    Ok(HttpResponse::Ok().body(body))
}

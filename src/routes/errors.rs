use actix_web::HttpResponse;
use askama_actix::Template;

use super::auxiliaries::FormattedFlashMessage;

#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate {
    pub title: Option<String>,
    pub input: ErrorInput,
    pub flash_messages: Option<Vec<FormattedFlashMessage>>,
}

pub struct ErrorInput {
    pub h1: String,
    pub p: String,
}

/// # `error_404`
///
/// Handler that returns a 404 code with a custom HTML doc.
/// Used as a default 404 response in the application.
pub async fn error_404() -> HttpResponse {
    let body = match generate_error_template(
        "404: Resource not found.",
        "I couldn't find the resource you asked for.",
        404,
    ) {
        Ok(b) => b,
        Err(_) => {
            return HttpResponse::NotFound().finish();
        }
    };
    HttpResponse::NotFound().body(body)
}

/// # `error_500`
///
/// Handler that returns a 500 code with a custom HTML doc.
pub async fn error_500() -> HttpResponse {
    let body = match generate_error_template(
        "500: Internal Server Error.",
        "We are having some technical difficulties, please try again later.",
        500,
    ) {
        Ok(b) => b,
        Err(_) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    HttpResponse::InternalServerError().body(body)
}

/// # `error_400`
///
/// Handler that returns a 400 code with a custom HTML doc.
pub async fn error_400() -> HttpResponse {
    let body = match generate_error_template(
        "400: Bad Request",
        "The server cannot or will not process the request due to something that is perceived to be a client error (for example, malformed request syntax, invalid request message framing, or deceptive request routing).",
        400,
    ) {
        Ok(b) => b,
        Err(_) => {
            return HttpResponse::BadRequest().finish();
        }
    };
    HttpResponse::BadRequest().body(body)
}

/// # `error_403`
///
/// Handler that returns a 403 code with a custom HTML doc.
pub async fn error_403() -> HttpResponse {
    let body = match generate_error_template(
        "403: Forbidden",
        "You don't have the permission to access the requested resource. It is either read-protected or not readable by the server.",
        403,
    ) {
        Ok(b) => b,
        Err(_) => {
            return HttpResponse::Forbidden().finish();
        }
    };
    HttpResponse::Forbidden().body(body)
}

/// TODO: comment
#[tracing::instrument(name = "Generate error template", skip(h1, p))]
fn generate_error_template(
    h1: &str,
    p: &str,
    status_code: u16,
) -> Result<String, askama_actix::Error> {
    let input = ErrorInput {
        h1: h1.to_string(),
        p: p.to_string(),
    };
    let ctx = ErrorTemplate {
        title: Some(h1.to_string()),
        input,
        flash_messages: None,
    };
    ctx.render()
}

/// TODO: comment
pub async fn e500<T>(e: T) -> actix_web::error::InternalError<T>
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    actix_web::error::InternalError::from_response(e, error_500().await)
}

/// TODO: comment
pub async fn e400<T>(e: T) -> actix_web::error::InternalError<T>
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    actix_web::error::InternalError::from_response(e, error_400().await)
}

/// TODO: comment
pub async fn e403<T>(e: T) -> actix_web::error::InternalError<T>
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    actix_web::error::InternalError::from_response(e, error_403().await)
}

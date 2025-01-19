use actix_web::HttpResponse;
use askama_actix::Template;

#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate {
    pub title: Option<String>,
    pub input: ErrorInput,
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
    let title = "404: Resource not found.";
    let input = ErrorInput {
        h1: String::from(title),
        p: String::from("I couldn't find the resource you asked for."),
    };
    let ctx = ErrorTemplate {
        title: Some(title.to_string()),
        input,
    };
    let body = match ctx.render() {
        Ok(b) => b,
        Err(e) => {
            tracing::error!("Failed to render the template.\nError: {}", e);
            return HttpResponse::NotFound().finish();
        }
    };
    HttpResponse::NotFound().body(body)
}

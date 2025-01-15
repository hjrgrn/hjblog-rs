use actix_web::HttpResponse;

#[tracing::instrument(name = "Checking health of the server.")]
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

use crate::auxiliaries::{assert_redirects_to, spawn_app};

#[tokio::test]
async fn redirect_to_index_if_login_success() {
    let app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": &app.test_admin.username,
        "password": &app.test_admin.password
    });
    let response = app.post_login(&login_body).await;
    assert_redirects_to(&response, "/");
}

#[tokio::test]
async fn redirect_to_login_if_login_unsuccessfull() {
    let app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": "random-username",
        "password": "random-password"
    });
    let response = app.post_login(&login_body).await;
    assert_redirects_to(&response, "/auth/login");
    let response = app
        .get_request("/")
        .await
        .expect("Failed to request \"/\".");
    let body = response.text().await.unwrap();

    assert!(body.contains("Invalid credentials, try again."));
    assert!(body.contains(r#"<div class="alert-danger alert-generic">"#));
}

#[tokio::test]
async fn redirect_to_index_if_already_logged_in() {
    let app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": &app.test_admin.username,
        "password": &app.test_admin.password
    });
    let _ = app.post_login(&login_body).await;
    let response = app
        .get_request("/auth/login")
        .await
        .expect("Failed to query the server.");

    assert_redirects_to(&response, "/");

    let response = app
        .get_request("/")
        .await
        .expect("Failed to request \"/\".");
    let body = response.text().await.unwrap();

    assert!(body.contains("You are already logged in, before logging in again logout."));
    assert!(body.contains(r#"<div class="alert-danger alert-generic">"#));
}

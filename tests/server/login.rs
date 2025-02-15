use crate::auxiliaries::{assert_redirects_to, spawn_app};

#[tokio::test]
async fn redirect_to_index_if_login_success() {
    let test_app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": &test_app.test_admin.username,
        "password": &test_app.test_admin.password
    });
    let response = test_app.post_request(&login_body, "/auth/login").await;
    assert_redirects_to(&response, "/");

    let response = test_app
        .get_request("/")
        .await
        .expect("Failed to request route \"/\".");
    let body = response.text().await.unwrap();
    assert!(body.contains(&format!(
        r#"<a href="/profile/manage_profile" class="link">{}</a>"#,
        &test_app.test_admin.username
    )))
}

#[tokio::test]
async fn redirect_to_login_if_login_unsuccessfull() {
    let test_app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": "random-username",
        "password": "random-password"
    });
    let response = test_app.post_request(&login_body, "/auth/login").await;
    assert_redirects_to(&response, "/auth/login");
    let response = test_app
        .get_request("/")
        .await
        .expect("Failed to request \"/\".");
    let body = response.text().await.unwrap();

    assert!(body.contains("Invalid credentials, try again."));
    assert!(body.contains(r#"<div class="alert-danger alert-generic">"#));
}

#[tokio::test]
async fn redirect_to_index_if_already_logged_in() {
    let test_app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": &test_app.test_admin.username,
        "password": &test_app.test_admin.password
    });
    let _ = test_app.post_request(&login_body, "/auth/login").await;
    let response = test_app
        .get_request("/auth/login")
        .await
        .expect("Failed to query the server.");

    assert_redirects_to(&response, "/");

    let response = test_app
        .get_request("/")
        .await
        .expect("Failed to request \"/\".");
    let body = response.text().await.unwrap();

    assert!(body.contains("You are already logged in, before logging in again logout."));
    assert!(body.contains(r#"<div class="alert-danger alert-generic">"#));
}

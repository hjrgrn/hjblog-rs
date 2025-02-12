//! TODO: waiting for the other fields
use crate::auxiliaries::{assert_redirects_to, spawn_app};

#[tokio::test]
async fn redirects_you_to_login_if_not_logged_in() {
    let test_app = spawn_app().await;

    let response = test_app
        .get_request("/profile/manage_profile")
        .await
        .expect("Failed to request route \"/auth/logout\".");
    assert_redirects_to(&response, "/auth/login");

    let response = test_app
        .get_request("/auth/login")
        .await
        .expect("Failed to request route \"/auth/login\".");
    let body = response.text().await.unwrap();

    assert!(
        body.contains("You are already not logged in, you need to be logged in to view this page.")
    );
    assert!(body.contains(r#"<div class="alert-danger alert-generic">"#));
}

#[tokio::test]
async fn test_manage_profile_template() {
    let test_app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": &test_app.test_admin.username,
        "password": &test_app.test_admin.password
    });
    let response = test_app.post_login(&login_body).await;
    assert_redirects_to(&response, "/");

    let response = test_app
        .get_request("/profile/manage_profile")
        .await
        .expect("Failed to request route \"/auth/login\".");

    let body = response.text().await.unwrap();
    assert!(body.contains(&format!("<h1>{}", &test_app.test_admin.username)));
    assert!(body.contains(&format!("<p>Username: {}</p>", &test_app.test_admin.username)));
}

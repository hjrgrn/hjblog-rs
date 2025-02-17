use crate::auxiliaries::{assert_redirects_to, spawn_app};

#[tokio::test]
async fn redirects_you_to_login_if_not_logged_in() {
    let test_app = spawn_app().await;

    let response = test_app
        .get_request("/auth/logout")
        .await
        .expect("Failed to request route \"/auth/logout\".");
    assert_redirects_to(&response, "/auth/login");

    let response = test_app
        .get_request("/auth/login")
        .await
        .expect("Failed to request route \"/auth/login\".");
    let body = response.text().await.unwrap();

    assert!(body.contains("You are not logged in."));
    assert!(body.contains(r#"<div class="alert-danger alert-generic">"#));
}

#[tokio::test]
async fn redirects_you_to_index_if_logout_is_successfull() {
    let test_app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": &test_app.test_admin.username,
        "password": &test_app.test_admin.password
    });
    let _ = test_app.post_request(&login_body, "/auth/login").await;
    // Check if we are logged in
    let response = test_app
        .get_request("/")
        .await
        .expect("Failed to request route \"/\".");
    let body = response.text().await.unwrap();
    assert!(body.contains(&format!(
        "class=\"link\">{}</a>",
        &test_app.test_admin.username
    )));

    let response = test_app
        .get_request("/auth/logout")
        .await
        .expect("Failed to request route \"/auth/logout\".");
    assert_redirects_to(&response, "/");

    let response = test_app
        .get_request("/index")
        .await
        .expect("Failed to request route \"/index\".");
    let body = response.text().await.unwrap();

    assert!(body.contains("See you space cowboy..."));
    assert!(body.contains(r#"<div class="alert-success alert-generic">"#));
    assert!(body.contains(r#"class="link">Register</a>"#));
    assert!(body.contains(r#"<a href="/auth/login" class="link">Log In</a>"#));
}

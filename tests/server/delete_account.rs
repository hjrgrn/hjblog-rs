use crate::auxiliaries::{assert_redirects_to, spawn_app};

#[tokio::test]
async fn delete_account_redirects_you_to_login_if_not_logged_in() {
    let test_app = spawn_app().await;

    let response = test_app
        .get_request("/profile/delete_account")
        .await
        .expect("Failed to request route \"/profile/delete_account\".");
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
async fn delete_account_redirect_to_index_if_successfull() {
    let test_app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": &test_app.test_admin.username,
        "password": &test_app.test_admin.password,
    });
    let response = test_app.post_request(&login_body, "/auth/login").await;
    assert_redirects_to(&response, "/");

    let delete_account_body = serde_json::json!({
        "password": &test_app.test_admin.password,
    });

    let response = test_app
        .post_request(&delete_account_body, "/profile/delete_account")
        .await;
    assert_redirects_to(&response, "/");

    let login_body = serde_json::json!({
        "username": &test_app.test_admin.username,
        "password": &test_app.test_admin.password,
    });

    let response = test_app.post_request(&login_body, "/auth/login").await;
    assert_redirects_to(&response, "/auth/login");

    let response = test_app
        .get_request("/")
        .await
        .expect("Failed to request route \"/auth/login\".");

    let body = response.text().await.unwrap();

    assert!(body.contains("Invalid credentials, try again."));
    assert!(body.contains(r#"<div class="alert-danger alert-generic">"#));
}

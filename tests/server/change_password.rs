use uuid::Uuid;

use crate::auxiliaries::{assert_redirects_to, spawn_app};

#[tokio::test]
async fn accept_request_if_logged_in() {
    let test_app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": &test_app.test_admin.username,
        "password": &test_app.test_admin.password,
    });
    let response = test_app.post_login(&login_body).await;
    assert_redirects_to(&response, "/");

    let response = test_app
        .get_request("/profile/change_password")
        .await
        .expect("Failed to request route \"/\".");

    assert!(response.status().as_u16() == 200);
}

#[tokio::test]
async fn change_password_redirects_you_to_login_if_not_logged_in() {
    let test_app = spawn_app().await;

    let response = test_app
        .get_request("/profile/change_password")
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
async fn change_password_logs_you_out_if_wrong_old_password() {
    let test_app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": &test_app.test_admin.username,
        "password": &test_app.test_admin.password,
    });
    let response = test_app.post_login(&login_body).await;
    assert_redirects_to(&response, "/");

    let new_password = Uuid::new_v4().to_string();

    let change_username_body = serde_json::json!({
        "old_password": "random-password",
        "new_password": &new_password
    });

    let response = test_app.post_change_password(&change_username_body).await;
    assert_redirects_to(&response, "/");

    let response = test_app
        .get_request("/")
        .await
        .expect("Failed to request route \"/\".");
    let body = response.text().await.unwrap();
    assert!(!body.contains(&format!(
        r#"<a href="/profile/manage_profile" class="link">{}</a>"#,
        &test_app.test_admin.username
    )))
}

#[tokio::test]
async fn changes_password_redirect_to_index_if_successfull_and_changes_password() {
    let test_app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": &test_app.test_admin.username,
        "password": &test_app.test_admin.password,
    });
    let response = test_app.post_login(&login_body).await;
    assert_redirects_to(&response, "/");

    let new_password = Uuid::new_v4().to_string();

    let change_username_body = serde_json::json!({
        "old_password": &test_app.test_admin.password,
        "new_password": &new_password
    });

    let response = test_app.post_change_password(&change_username_body).await;
    assert_redirects_to(&response, "/");

    let response = test_app
        .get_request("/")
        .await
        .expect("Failed to request route \"/\".");
    let body = response.text().await.unwrap();

    assert!(body.contains("Your password has been updated."));
    assert!(body.contains(r#"<div class="alert-success alert-generic">"#));
    assert!(body.contains(&format!(
        r#"<a href="/profile/manage_profile" class="link">{}</a>"#,
        &test_app.test_admin.username
    )));

    let response = test_app
        .get_request("/auth/logout")
        .await
        .expect("Failed to request route \"/auth/logout\".");
    assert_redirects_to(&response, "/");

    let login_body = serde_json::json!({
        "username": &test_app.test_admin.username,
        "password": &new_password
    });

    let response = test_app.post_login(&login_body).await;
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

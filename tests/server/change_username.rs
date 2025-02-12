use uuid::Uuid;

use crate::auxiliaries::{assert_redirects_to, spawn_app};

#[tokio::test]
async fn test_template() {
    let test_app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": &test_app.test_admin.username,
        "password": &test_app.test_admin.password,
    });
    let response = test_app.post_login(&login_body).await;
    assert_redirects_to(&response, "/");

    let response = test_app
        .get_request("/profile/change_username")
        .await
        .expect("Failed to request route \"/\".");

    let body = response.text().await.unwrap();

    assert!(body.contains(&format!(
        r#"<p>Old username: {}</p>"#,
        &test_app.test_admin.username
    )));
}

#[tokio::test]
async fn redirects_you_to_login_if_not_logged_in() {
    let test_app = spawn_app().await;

    let response = test_app
        .get_request("/profile/change_username")
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
async fn changes_username_redirect_to_index_if_successfull() {
    let test_app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": &test_app.test_admin.username,
        "password": &test_app.test_admin.password,
    });
    let response = test_app.post_login(&login_body).await;
    assert_redirects_to(&response, "/");

    let new_username = Uuid::new_v4().to_string();

    let change_username_body = serde_json::json!({
        "username": new_username,
        "password": &test_app.test_admin.password,
    });

    let response = test_app.post_change_username(&change_username_body).await;
    assert_redirects_to(&response, "/");

    let response = test_app
        .get_request("/")
        .await
        .expect("Failed to request route \"/auth/login\".");
    let body = response.text().await.unwrap();

    assert!(body.contains(&format!(
        "Your username has been updated to {}",
        new_username
    )));
    assert!(body.contains(r#"<div class="alert-success alert-generic">"#));
    assert!(body.contains(&format!(
        r#"<a href="/profile/manage_profile" class="link">{}</a>"#,
        new_username
    )));
}

#[tokio::test]
async fn redirect_to_index_and_logout_if_wrong_password() {
    let test_app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": &test_app.test_admin.username,
        "password": &test_app.test_admin.password,
    });
    let response = test_app.post_login(&login_body).await;
    assert_redirects_to(&response, "/");

    let new_username = Uuid::new_v4().to_string();

    let change_username_body = serde_json::json!({
        "username": new_username,
        "password": "random-password",
    });

    let response = test_app.post_change_username(&change_username_body).await;
    assert_redirects_to(&response, "/");

    let response = test_app
        .get_request("/")
        .await
        .expect("Failed to request route \"/\".");
    let body = response.text().await.unwrap();

    assert!(body.contains(r#"<a href="/auth/login" class="link">Log In</a>"#));
}

#[tokio::test]
async fn doesn_t_allow_you_to_change_if_already_existing_name() {
    let test_app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": &test_app.test_admin.username,
        "password": &test_app.test_admin.password,
    });
    let response = test_app.post_login(&login_body).await;
    assert_redirects_to(&response, "/");

    let change_username_body = serde_json::json!({
        "username": &test_app.test_user.username,
        "password": &test_app.test_admin.password,
    });

    let response = test_app.post_change_username(&change_username_body).await;
    assert_redirects_to(&response, "/profile/change_username");

    let response = test_app
        .get_request("/")
        .await
        .expect("Failed to request route \"/\".");
    let body = response.text().await.unwrap();

    assert!(body.contains("The new name you provided is already taken, please try again."));
    assert!(body.contains(r#"<div class="alert-danger alert-generic">"#));
}

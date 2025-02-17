use fake::{faker::internet::en::SafeEmail, Fake};

use crate::auxiliaries::{assert_redirects_to, spawn_app};

#[tokio::test]
async fn test_change_email_template() {
    let test_app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": &test_app.test_admin.username,
        "password": &test_app.test_admin.password,
    });
    let response = test_app.post_request(&login_body, "/auth/login").await;
    assert_redirects_to(&response, "/");

    let response = test_app
        .get_request("/profile/change_email")
        .await
        .expect("Failed to request route \"/profile/change_email\".");

    let body = response.text().await.unwrap();

    assert!(body.contains(&format!(
        r#"<p>Old email: {}</p>"#,
        &test_app.test_admin.email
    )));
}

#[tokio::test]
async fn change_email_redirects_you_to_login_if_not_logged_in() {
    let test_app = spawn_app().await;

    let response = test_app
        .get_request("/profile/change_email")
        .await
        .expect("Failed to request route \"/profile/change_email\".");
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
async fn changes_email_redirect_to_index_if_successfull() {
    let test_app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": &test_app.test_admin.username,
        "password": &test_app.test_admin.password,
    });
    let response = test_app.post_request(&login_body, "/auth/login").await;
    assert_redirects_to(&response, "/");

    let new_email: String = SafeEmail().fake();

    let change_email_body = serde_json::json!({
        "email": new_email,
        "password": &test_app.test_admin.password,
    });

    let response = test_app
        .post_request(&change_email_body, "/profile/change_email")
        .await;
    assert_redirects_to(&response, "/");

    let response = test_app
        .get_request("/")
        .await
        .expect("Failed to request route \"/\".");
    let body = response.text().await.unwrap();

    assert!(body.contains(&format!("Your email has been updated to {}", new_email)));
    assert!(body.contains(r#"<div class="alert-success alert-generic">"#));
}

#[tokio::test]
async fn redirect_to_index_and_logout_if_wrong_password() {
    let test_app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": &test_app.test_admin.username,
        "password": &test_app.test_admin.password,
    });
    let response = test_app.post_request(&login_body, "/auth/login").await;
    assert_redirects_to(&response, "/");

    let new_email: String = SafeEmail().fake();

    let change_email_body = serde_json::json!({
        "email": new_email,
        "password": "random-password"
    });

    let response = test_app
        .post_request(&change_email_body, "/profile/change_email")
        .await;
    assert_redirects_to(&response, "/");

    let response = test_app
        .get_request("/")
        .await
        .expect("Failed to request route \"/\".");
    let body = response.text().await.unwrap();

    assert!(body.contains("Invalid credentials, you have been logged out."));
    assert!(body.contains(r#"<div class="alert-danger alert-generic">"#));
    assert!(body.contains(r#"<a href="/auth/login" class="link">Log In</a>"#));
}

#[tokio::test]
async fn change_email_doesn_t_allow_you_to_change_if_already_existing_email() {
    let test_app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": &test_app.test_admin.username,
        "password": &test_app.test_admin.password,
    });
    let response = test_app.post_request(&login_body, "/auth/login").await;
    assert_redirects_to(&response, "/");

    let change_email_body = serde_json::json!({
        "email": &test_app.test_user.email,
        "password": &test_app.test_admin.password,
    });

    let response = test_app
        .post_request(&change_email_body, "/profile/change_email")
        .await;
    assert_redirects_to(&response, "/profile/change_email");

    let response = test_app
        .get_request("/")
        .await
        .expect("Failed to request route \"/\".");
    let body = response.text().await.unwrap();

    assert!(body.contains("The new email you provided is already taken, please try again."));
    assert!(body.contains(r#"<div class="alert-danger alert-generic">"#));
}

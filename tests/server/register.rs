//! TODO: city
use fake::{faker::internet::en::SafeEmail, Fake};
use rand::{distributions::Alphanumeric, Rng};
use uuid::Uuid;

use crate::auxiliaries::{assert_redirects_to, spawn_app};

#[tokio::test]
async fn redirect_to_index_if_register_success() {
    let test_app = spawn_app().await;
    let username = Uuid::new_v4();
    let email: String = SafeEmail().fake();
    let password = Uuid::new_v4();

    let register_body = serde_json::json!({
        "username": username,
        "email": email,
        "password": password,
        "confirm_password": password
    });
    let response = test_app
        .post_request(&register_body, "/auth/register")
        .await;
    assert_redirects_to(&response, "/");

    let response = test_app
        .get_request("/")
        .await
        .expect("Failed to request \"/\".");

    let body = response.text().await.unwrap();

    assert!(body.contains(&format!(
        "Welcome {}, you have been registered correctly!",
        username
    )));
    assert!(body.contains(r#"<div class="alert-success alert-generic">"#));
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
        .get_request("/auth/register")
        .await
        .expect("Failed to query the server.");

    assert_redirects_to(&response, "/");

    let response = test_app
        .get_request("/")
        .await
        .expect("Failed to request \"/\".");
    let body = response.text().await.unwrap();

    assert!(body.contains("You are already registered, before register again logout."));
    assert!(body.contains(r#"<div class="alert-danger alert-generic">"#));

    let username = Uuid::new_v4();
    let email: String = SafeEmail().fake();
    let password = Uuid::new_v4();

    let register_body = serde_json::json!({
        "username": username,
        "email": email,
        "password": password,
        "confirm_password": password
    });
    let response = test_app
        .post_request(&register_body, "/auth/register")
        .await;
    assert_redirects_to(&response, "/");

    let response = test_app
        .get_request("/")
        .await
        .expect("Failed to request \"/\".");
    let body = response.text().await.unwrap();

    assert!(body.contains("You are already registered, before register again logout."));
    assert!(body.contains(r#"<div class="alert-danger alert-generic">"#));
}

#[tokio::test]
async fn refuse_invalid_username() {
    let test_app = spawn_app().await;
    let too_long_name: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(70)
        .map(char::from)
        .collect();
    let invalid_chars_name = "<mario";
    let password = Uuid::new_v4().to_string();
    let email: String = SafeEmail().fake();

    let cases: Vec<(&str, &str, &str, &str)> = vec![
        (
            invalid_chars_name,
            &password,
            &email,
            "Invalid username: username contains forbidden characters",
        ),
        (
            &too_long_name,
            &password,
            &email,
            "Invalid username: username is too long.",
        ),
        (
            "",
            &password,
            &email,
            "Invalid username: username contains whitespaces.",
        ),
    ];

    for case in cases.iter() {
        let register_body = serde_json::json!({
            "username": case.0,
            "email": case.2,
            "password": case.1,
            "confirm_password": case.1
        });
        let response = test_app
            .post_request(&register_body, "/auth/register")
            .await;
        assert_redirects_to(&response, "/auth/register");
        let response = test_app
            .get_request("/")
            .await
            .expect("Failed to request \"/\"");

        let body = response.text().await.unwrap();

        assert!(body.contains(case.3));
        assert!(body.contains(r#"<div class="alert-danger alert-generic">"#));
        assert!(!body.contains("/auth/logout"));
    }
}

#[tokio::test]
async fn refuse_invalid_email() {
    let test_app = spawn_app().await;
    let username = Uuid::new_v4().to_string();
    let mut too_long_email: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(200)
        .map(char::from)
        .collect();
    let ext: String = SafeEmail().fake();
    too_long_email.push_str(&ext);

    let invalid_email = "invalidemail";
    let invalid_email_error_msg = format!(
        "Invalid email: &quot;{}&quot; is not a valid email",
        invalid_email
    );
    let password = Uuid::new_v4().to_string();

    let cases: Vec<(&str, &str, &str, &str)> = vec![
        (
            &username,
            &password,
            &too_long_email,
            "Invalid email: email provided is too long.",
        ),
        (
            &username,
            &password,
            &invalid_email,
            &invalid_email_error_msg,
        ),
    ];

    for case in cases.iter() {
        let register_body = serde_json::json!({
            "username": case.0,
            "email": case.2,
            "password": case.1,
            "confirm_password": case.1
        });
        let response = test_app
            .post_request(&register_body, "/auth/register")
            .await;
        assert_redirects_to(&response, "/auth/register");
        let response = test_app
            .get_request("/")
            .await
            .expect("Failed to request \"/\"");

        let body = response.text().await.unwrap();

        assert!(body.contains(case.3));
        assert!(body.contains(r#"<div class="alert-danger alert-generic">"#));
        assert!(!body.contains("/auth/logout"));
    }
}

#[tokio::test]
async fn fail_to_register_when_user_types_two_different_passwords() {
    let test_app = spawn_app().await;
    let username = Uuid::new_v4();
    let email: String = SafeEmail().fake();
    let password = Uuid::new_v4();
    let another_password = Uuid::new_v4();

    let register_body = serde_json::json!({
        "username": username,
        "email": email,
        "password": password,
        "confirm_password": another_password
    });
    let response = test_app
        .post_request(&register_body, "/auth/register")
        .await;
    assert_redirects_to(&response, "/auth/register");

    let response = test_app
        .get_request("/")
        .await
        .expect("Failed to request \"/\".");

    let body = response.text().await.unwrap();
    assert!(body.contains("You typed two different passwords. Try again."));
    assert!(body.contains(r#"<div class="alert-danger alert-generic">"#));
    assert!(!body.contains("/auth/logout"));
}

#[tokio::test]
async fn fail_to_register_when_user_email_already_taken() {
    let test_app = spawn_app().await;
    let username = Uuid::new_v4();
    let email = &test_app.test_admin.email;
    let password = Uuid::new_v4();

    let register_body = serde_json::json!({
        "username": username,
        "email": email,
        "password": password,
        "confirm_password": password
    });
    let response = test_app
        .post_request(&register_body, "/auth/register")
        .await;
    assert_redirects_to(&response, "/auth/register");

    let response = test_app
        .get_request("/")
        .await
        .expect("Failed to request \"/\".");

    let body = response.text().await.unwrap();
    assert!(body.contains("Your credentials are already taken."));
    assert!(body.contains(r#"<div class="alert-danger alert-generic">"#));
    assert!(!body.contains("/auth/logout"));
}

#[tokio::test]
async fn fail_to_register_when_user_name_already_taken() {
    let test_app = spawn_app().await;
    let username = &test_app.test_admin.username;
    let email: String = SafeEmail().fake();
    let password = Uuid::new_v4();

    let register_body = serde_json::json!({
        "username": username,
        "email": email,
        "password": password,
        "confirm_password": password
    });
    let response = test_app
        .post_request(&register_body, "/auth/register")
        .await;
    assert_redirects_to(&response, "/auth/register");

    let response = test_app
        .get_request("/")
        .await
        .expect("Failed to request \"/\".");

    let body = response.text().await.unwrap();
    assert!(body.contains("Your credentials are already taken."));
    assert!(body.contains(r#"<div class="alert-danger alert-generic">"#));
    assert!(!body.contains("/auth/logout"));
}

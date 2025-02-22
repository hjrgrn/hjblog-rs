use rand::{distributions::Alphanumeric, Rng};

use crate::auxiliaries::{assert_redirects_to, spawn_app};

#[tokio::test]
async fn new_post_accept_request_if_logged_in_as_admin() {
    let test_app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": &test_app.test_admin.username,
        "password": &test_app.test_admin.password,
    });
    let response = test_app.post_request(&login_body, "/auth/login").await;
    assert_redirects_to(&response, "/");

    let response = test_app
        .get_request("/user_actions/new_post")
        .await
        .expect("Failed to request route \"/\".");

    assert!(response.status().as_u16() == 200);
}

#[tokio::test]
async fn new_post_responds_with_forbidden_if_logged_in_as_non_admin() {
    let test_app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": &test_app.test_user.username,
        "password": &test_app.test_user.password,
    });
    let response = test_app.post_request(&login_body, "/auth/login").await;
    assert_redirects_to(&response, "/");

    let response = test_app
        .get_request("/user_actions/new_post")
        .await
        .expect("Failed to request route \"/\".");

    assert!(response.status().as_u16() == 403);
}

#[tokio::test]
async fn new_post_redirects_to_login_if_not_logged_in() {
    let test_app = spawn_app().await;

    let response = test_app
        .get_request("/user_actions/new_post")
        .await
        .expect("Failed to request route \"/user_actions/new_post\".");

    assert_redirects_to(&response, "/auth/login");
}

#[tokio::test]
async fn new_post_refuses_title_and_content_that_are_too_long() {
    let test_app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": &test_app.test_admin.username,
        "password": &test_app.test_admin.password,
    });
    let response = test_app.post_request(&login_body, "/auth/login").await;
    assert_redirects_to(&response, "/");

    let too_long_string = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(2001)
        .map(char::from)
        .collect::<String>();

    let new_post_body = serde_json::json!({
        "title": "hello-world",
        "content": too_long_string,
    });

    let response = test_app
        .post_request(&new_post_body, "/user_actions/new_post")
        .await;
    assert_redirects_to(&response, "/user_actions/new_post");

    let response = test_app
        .get_request("/user_actions/new_post")
        .await
        .expect("Failed to request route \"/user_actions/new_post\".");

    let body = response.text().await.unwrap();

    assert!(body.contains("Content is too long, retry"));

    let new_post_body = serde_json::json!({
        "title": too_long_string,
        "content": "hello-world",
    });

    let response = test_app
        .post_request(&new_post_body, "/user_actions/new_post")
        .await;
    assert_redirects_to(&response, "/user_actions/new_post");

    let response = test_app
        .get_request("/user_actions/new_post")
        .await
        .expect("Failed to request route \"/user_actions/new_post\".");

    let body = response.text().await.unwrap();

    assert!(body.contains("Title is too long, retry"));
}

#[tokio::test]
async fn new_post_posts_a_post() {
    let test_app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": &test_app.test_admin.username,
        "password": &test_app.test_admin.password,
    });
    let response = test_app.post_request(&login_body, "/auth/login").await;
    assert_redirects_to(&response, "/");

    let title = "post-title";
    let content = "post-content";

    let response = test_app
        .get_request("/")
        .await
        .expect("Failed to request route \"/\".");

    let body = response.text().await.unwrap();

    assert!(!body.contains(title));
    assert!(!body.contains(content));

    let new_post_body = serde_json::json!({
        "title": title,
        "content": content,
    });

    let response = test_app
        .post_request(&new_post_body, "/user_actions/new_post")
        .await;
    assert_redirects_to(&response, "/");

    let response = test_app
        .get_request("/")
        .await
        .expect("Failed to request route \"/\".");

    let body = response.text().await.unwrap();

    assert!(body.contains(title));
    assert!(body.contains("Your post has been published."));
}

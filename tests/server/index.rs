//! TODO: waiting for post, picture and register
use crate::auxiliaries::spawn_app;

#[tokio::test]
async fn testing_index_template_when_no_posts() {
    let test_app = spawn_app().await;

    let response = test_app
        .api_client
        .get(&format!("{}/", &test_app.get_full_url()))
        .send()
        .await
        .expect("Failed to execute request.");

    let status = response.status().as_u16();
    let body = response
        .text()
        .await
        .expect("Failed to extract the body of the response.");

    assert_eq!(status, 200);
    assert!(body.contains(r#"<h1 class="presentation_h1">HJ's Blog</h1>"#));
    assert!(body.contains(r#"<h2 class="posts_compact_h2">Latest Posts</h2>"#));
    assert!(body.contains(r#"<p class="presentation_par">Welcome to HJ's Blog.</p>"#));
    // No posts
    assert!(!body.contains(r#"<span class="posts_compact_date">"#));
    test_app.token.cancel();
    let _ = test_app.handle.await;
}

#[tokio::test]
async fn testing_navbar_when_logged_as_admin() {
    let test_app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": &test_app.test_admin.username,
        "password": &test_app.test_admin.password
    });
    let _ = test_app.post_login(&login_body).await;

    let response = test_app
        .api_client
        .get(&format!("{}/", &test_app.get_full_url()))
        .send()
        .await
        .expect("Failed to execute request.");

    let status = response.status().as_u16();
    let body = response
        .text()
        .await
        .expect("Failed to extract the body of the response.");

    assert_eq!(status, 200);
    assert!(body.contains(&format!(
        "class=\"link\">{}</a>",
        &test_app.test_admin.username
    )));
    assert!(body.contains(r#"class="link">Log Out</a>"#));
    assert!(body.contains(r#"class="link">Post</a>"#));
    assert!(body.contains(r#"<div class="alert-success alert-generic">"#));
    assert!(body.contains(&format!("Welcome back {}!", &test_app.test_admin.username)));
}

#[tokio::test]
async fn testing_navbar_when_not_logged() {
    let test_app = spawn_app().await;

    let response = test_app
        .api_client
        .get(&format!("{}/", &test_app.get_full_url()))
        .send()
        .await
        .expect("Failed to execute request.");

    let status = response.status().as_u16();
    let body = response
        .text()
        .await
        .expect("Failed to extract the body of the response.");

    assert_eq!(status, 200);
    assert!(body.contains(r#"class="link">Register</a>"#));
    assert!(body.contains(r#"<a href="/auth/login" class="link">Log In</a>"#));
}

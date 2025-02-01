//! TODO: complete this test suit after the development of auth
use crate::auxiliaries::spawn_app;

#[tokio::test]
async fn testing_index_template() {
    let test_app = spawn_app().await;

    // Act
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

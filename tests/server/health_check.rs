use crate::auxiliaries::spawn_app;

#[tokio::test]
/// # health_check_works
///
/// `health_check` route should return 200 OK to a GET request
/// and return a response with no body
async fn health_check_works() {
    let test_app = spawn_app().await;

    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", &test_app.get_full_url()))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
    test_app.token.cancel();
    let _ = test_app.handle.await;
}

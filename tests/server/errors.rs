use crate::auxiliaries::spawn_app;

#[tokio::test]
async fn responds_with_404_when_requiring_an_unexisting_page() {
    let test_app = spawn_app().await;

    // Act
    let response = test_app
        .api_client
        .get(format!("{}/non-existing-page", &test_app.get_full_url()))
        .send()
        .await
        .expect("Failed to execute request.");

    let status = response.status().as_u16();
    let body = response
        .text()
        .await
        .expect("Failed to extract the body of the response.");

    assert_eq!(status, 404);
    assert!(body.contains("<h1>404: Resource not found.</h1>"));
    assert!(body.contains("<p>I couldn&#x27;t find the resource you asked for.</p>"));
    test_app.token.cancel();
    let _ = test_app.handle.await;
}

use crate::auxiliaries::{assert_redirects_to, spawn_app};
use sqlx::{query, Row};
use uuid::Uuid;

#[tokio::test]
async fn delete_post_refuses_request_if_not_logged() {
    let test_app = spawn_app().await;

    test_app.generate_posts(1).await;
    let post_id: Uuid = query("SELECT id FROM posts")
        .fetch_one(&test_app.db_pool)
        .await
        .unwrap()
        .try_get("id")
        .unwrap();

    let response = test_app
        .get_request(&format!("/user_actions/delete_post/{}", post_id))
        .await
        .expect("Failed to request route \"/user_actions/delete_post\".");

    assert_redirects_to(&response, "/auth/login");
}

#[tokio::test]
async fn delete_post_accept_request_if_logged_as_admin() {
    let test_app = spawn_app().await;

    test_app.generate_posts(1).await;
    let post_id: Uuid = query("SELECT id FROM posts")
        .fetch_one(&test_app.db_pool)
        .await
        .unwrap()
        .try_get("id")
        .unwrap();

    let login_body = serde_json::json!({
        "username": &test_app.test_admin.username,
        "password": &test_app.test_admin.password,
    });
    let response = test_app.post_request(&login_body, "/auth/login").await;
    assert_redirects_to(&response, "/");

    let response = test_app
        .get_request(&format!("/user_actions/all_comments/{}", post_id))
        .await
        .expect("Failed to request route \"/user_actions/all_comments\".");
    let body = response.text().await.unwrap();
    assert!(body.contains("Delete this post"));

    let response = test_app
        .get_request(&format!("/user_actions/delete_post/{}", post_id))
        .await
        .expect("Failed to request route \"/user_actions/delete_post\".");

    assert_redirects_to(&response, "/");

    let response = test_app
        .get_request("/")
        .await
        .expect("Failed to request route \"/\".");

    let body = response.text().await.unwrap();

    assert!(body.contains("The post has been deleted correctly"));
}

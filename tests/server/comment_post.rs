use crate::auxiliaries::{assert_redirects_to, spawn_app};
use sqlx::{query, Row};
use uuid::Uuid;

#[tokio::test]
async fn comment_post_accept_request_if_logged() {
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
        .get_request(&format!("/user_actions/comment/{}", post_id))
        .await
        .expect("Failed to request route \"/user_actions/comment_post\".");

    assert!(response.status().as_u16() == 200);

    let body = response.text().await.unwrap();

    assert!(body.contains(r#"<p>Write your comment here:</p>"#));
}

#[tokio::test]
async fn comment_post_allows_you_to_comment_if_you_are_logged_in() {
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

    let comment_content = "random content for random post";
    let comment_body = serde_json::json!({ "content": comment_content });
    let response = test_app
        .post_request(&comment_body, &format!("/user_actions/comment/{}", post_id))
        .await;

    assert_redirects_to(&response, &format!("/user_actions/visit_post/{}", post_id));

    let response = test_app
        .get_request(&format!("/user_actions/visit_post/{}", post_id))
        .await
        .expect("Failed to request route \"/user_actions/visit_post\".");

    let body = response.text().await.unwrap();

    assert!(body.contains(comment_content));
    assert!(body.contains(r#"<div class="alert-success alert-generic">"#));
    assert!(body.contains(r#"<span class="visit_post_delete_comment" onclick="modalShow('modal-delete-comment')">Delete comment</span>"#));
}

#[tokio::test]
async fn comment_post_allows_you_to_delete_comment_if_you_are_the_author() {
    let test_app = spawn_app().await;

    test_app.generate_posts(1).await;
    let post_id: Uuid = query("SELECT id FROM posts")
        .fetch_one(&test_app.db_pool)
        .await
        .unwrap()
        .try_get("id")
        .unwrap();

    let login_body = serde_json::json!({
        "username": &test_app.test_user.username,
        "password": &test_app.test_user.password,
    });
    let response = test_app.post_request(&login_body, "/auth/login").await;
    assert_redirects_to(&response, "/");

    let comment_content = "random content for random post";
    let comment_body = serde_json::json!({ "content": comment_content });
    let response = test_app
        .post_request(&comment_body, &format!("/user_actions/comment/{}", post_id))
        .await;

    assert_redirects_to(&response, &format!("/user_actions/visit_post/{}", post_id));

    let response = test_app
        .get_request(&format!("/user_actions/visit_post/{}", post_id))
        .await
        .expect("Failed to request route \"/user_actions/visit_post\".");

    let body = response.text().await.unwrap();

    assert!(body.contains(comment_content));
    assert!(body.contains(r#"<div class="alert-success alert-generic">"#));
    assert!(body.contains(r#"<span class="visit_post_delete_comment" onclick="modalShow('modal-delete-comment')">Delete comment</span>"#));
}

#[tokio::test]
async fn comment_post_forbids_access_if_you_are_not_logged_in() {
    let test_app = spawn_app().await;

    test_app.generate_posts(1).await;
    let post_id: Uuid = query("SELECT id FROM posts")
        .fetch_one(&test_app.db_pool)
        .await
        .unwrap()
        .try_get("id")
        .unwrap();


    let comment_content = "random content for random post";
    let comment_body = serde_json::json!({ "content": comment_content });
    let response = test_app
        .post_request(&comment_body, &format!("/user_actions/comment/{}", post_id))
        .await;

    assert_redirects_to(&response, "/auth/login");

    let response = test_app
        .get_request(&format!("/user_actions/comment/{}", post_id))
        .await
        .expect("Failed to request route \"/user_actions/comment\".");

    assert_redirects_to(&response, "/auth/login");
}

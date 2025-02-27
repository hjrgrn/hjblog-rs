use hj_blog_rs::routes::home::auxiliaries::Post;
use sqlx::query_as;

use crate::auxiliaries::spawn_app;

#[tokio::test]
async fn all_comments_as_anonymous() {
    let test_app = spawn_app().await;

    test_app.generate_posts(1).await;

    let post = query_as::<_, Post>(
        "SELECT \
            posts.id, \
            users.username, \
            posts.title, \
            posts.content, \
            posts.posted, \
            posts.author_id \
        FROM posts JOIN users ON (users.id = posts.author_id)",
    )
    .fetch_one(&test_app.db_pool)
    .await
    .unwrap();

    test_app
        .generate_commens_to_post(post.id, 1, test_app.test_user.user_id)
        .await;

    let response = test_app
        .get_request(&format!("/user_actions/all_comments/{}", post.id))
        .await
        .expect("Failed to request route \"/user_actions/all_comments\".");

    let body = response.text().await.unwrap();

    assert!(body.contains(r#"<p>This is a test, content: 0</p>"#));
    assert!(!body.contains(r#"<span class="visit_post_delete_comment" onclick="modalShow('modal-delete-comment')">Delete comment</span>"#));
}

// TODO:

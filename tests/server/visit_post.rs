use hj_blog_rs::routes::home::auxiliaries::Post;
use sqlx::query_as;

use crate::auxiliaries::{assert_redirects_to, spawn_app};

#[tokio::test]
async fn visit_a_post_with_no_comments_as_anonymous() {
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

    let response = test_app
        .get_request(&format!("/user_actions/visit_post/{}", post.id))
        .await
        .expect("Failed to request route \"/visit_post\".");

    let body = response.text().await.unwrap();
    assert!(body.contains(&format!(
        r#"class="visit_post_author">{}</a></p>"#,
        test_app.test_admin.username
    )));
    assert!(
        body.contains(r#"<p>Nobody has commented this post yet, be the first one to do so.</p>"#,)
    );
    assert!(
        body.contains(r#"<p>Nobody has commented this post yet, be the first one to do so.</p>"#,)
    );
    assert!(body.contains(r#"<a class="visit_post_delete_post" href="/auth/login">Login</a>"#,));
}

#[tokio::test]
async fn visit_a_post_with_comments_as_anonymous() {
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
        .generate_commens_to_post(post.id, 10, test_app.test_user.user_id)
        .await;

    let response = test_app
        .get_request(&format!("/user_actions/visit_post/{}", post.id))
        .await
        .expect("Failed to request route \"/visit_post\".");

    let body = response.text().await.unwrap();
    assert!(body.contains(r#"<p>This is a test, content:"#));
    assert!(body.contains(r#"All comments →"#));
    assert!(body.contains(&format!(
        r#"class="visit_post_comment_link">{}"#,
        test_app.test_user.username
    )));

    assert!(body.contains(r#"<a class="visit_post_delete_post" href="/auth/login">Login</a>"#,));
}

#[tokio::test]
async fn visit_a_post_with_comments_as_author_of_the_comment() {
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
        .generate_commens_to_post(post.id, 10, test_app.test_user.user_id)
        .await;

    let login_body = serde_json::json!({
        "username": &test_app.test_user.username,
        "password": &test_app.test_user.password
    });
    let response = test_app.post_request(&login_body, "/auth/login").await;
    assert_redirects_to(&response, "/");

    let response = test_app
        .get_request(&format!("/user_actions/visit_post/{}", post.id))
        .await
        .expect("Failed to request route \"/visit_post\".");

    let body = response.text().await.unwrap();
    assert!(body.contains(r#"<span class="visit_post_delete_comment" onclick="modalShow('modal-delete-comment')">Delete comment</span>"#));
}

#[tokio::test]
async fn visit_a_post_with_comments_as_non_author_of_the_comment() {
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
        .generate_commens_to_post(post.id, 10, test_app.test_admin.user_id)
        .await;

    let login_body = serde_json::json!({
        "username": &test_app.test_user.username,
        "password": &test_app.test_user.password
    });
    let response = test_app.post_request(&login_body, "/auth/login").await;
    assert_redirects_to(&response, "/");

    let response = test_app
        .get_request(&format!("/user_actions/visit_post/{}", post.id))
        .await
        .expect("Failed to request route \"/visit_post\".");

    let body = response.text().await.unwrap();
    assert!(!body.contains(r#"<span class="visit_post_delete_comment" onclick="modalShow('modal-delete-comment')">Delete comment</span>"#));
}

#[tokio::test]
async fn visit_a_post_without_comments_as_loggedin_user() {
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

    let login_body = serde_json::json!({
        "username": &test_app.test_user.username,
        "password": &test_app.test_user.password
    });
    let response = test_app.post_request(&login_body, "/auth/login").await;
    assert_redirects_to(&response, "/");

    let response = test_app
        .get_request(&format!("/user_actions/visit_post/{}", post.id))
        .await
        .expect("Failed to request route \"/visit_post\".");

    let body = response.text().await.unwrap();
    assert!(body.contains(r#"<p>Nobody has commented this post yet, be the first one to do so.</p>"#));
}

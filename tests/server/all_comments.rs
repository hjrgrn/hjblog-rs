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

#[tokio::test]
async fn all_comments_displays_comments_if_there_are_comments() {
    // TODO: get parameters from config
    let comment_num = 200;
    let max_per_page = 5;
    let max_buffer = 99;
    let max_page = max_buffer / max_per_page;
    let page_span = 3;
    let mut page_amount = comment_num / max_per_page;
    if comment_num % max_per_page != 0 {
        page_amount += 1;
    }

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
        .generate_commens_to_post(post.id, comment_num, test_app.test_user.user_id)
        .await;

    for i in 0..page_amount {
        let o = if i <= max_page { 0 } else { 1 };
        let index = if i <= max_page { i } else { i - max_page - 1 };
        let response = test_app
            .get_request(&format!(
                "/user_actions/all_comments/{}?index={}&o={}",
                post.id, index, o
            ))
            .await
            .expect("Failed to request route \"/all_comments\".");

        let body = response.text().await.unwrap();

        for j in 1..page_span + 1 {
            let idx = index.saturating_sub(j);
            if idx > 0 {
                assert!(body.contains(&format!(
                    r#"<a class="page_num" href="/user_actions/all_comments/{}?index={}&amp;o={}">{}</a>"#,
                    post.id, idx, o, idx
                )));
            }
        }

        assert!(body.contains(&format!(
            r#"<span class="page_num page_num_main">{}</span>"#,
            index
        )));
        for h in 1..page_span + 1 {
            let idx = index + h;
            if idx < max_page {
                assert!(body.contains(&format!(
                    r#"<a class="page_num" href="/user_actions/all_comments/{}?index={}&amp;o={}">{}</a>"#,
                    post.id, idx, o, idx
                )));
            }
        }

        assert!(body.contains(&format!(r#"">{}</a>"#, &test_app.test_admin.username)));

        assert!(!body.contains(&format!(r#"<p>Test comment number {}</p>"#, comment_num - 1)));
        if i == max_page {
            assert!(body.contains(
                &format!(r#"<a class= "page_num page_offset" href="/user_actions/all_comments/{}?index=0&amp;o=1">More comments</a>"#, post.id)
            ));
        } else if i == max_page + 1 {
            assert!(body.contains(
                &format!(r#"<a class="page_num page_offset" href="/user_actions/all_comments/{}?index=0&amp;o=0">Previous comments</a>"#, post.id)
            ));
        } else {
            assert!(!body.contains(
                &format!(r#"<a class= "page_num page_offset" href="/user_actions/all_comments/{}?index=0&amp;o=1">More comments</a>"#, post.id)
            ));
        }
    }
}

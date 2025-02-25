use crate::auxiliaries::spawn_app;

#[tokio::test]
async fn blog_displays_one_post() {
    let test_app = spawn_app().await;

    test_app.generate_posts(1).await;

    let response = test_app
        .get_request("/blog")
        .await
        .expect("Failed to request route \"/blog\".");

    let body = response.text().await.unwrap();

    assert!(body.contains(r#"<span class="page_num page_num_main">0</span>"#));
}

#[tokio::test]
async fn blog_displays_posts_if_there_are_posts() {
    // TODO: get parameters from config
    let mut post_num = 200;
    let max_per_page = 5;
    let max_buffer = 99;
    let max_page = max_buffer / max_per_page;
    let page_span = 3;
    let mut page_amount = post_num / max_per_page;
    if post_num % max_per_page != 0 {
        page_amount = page_amount + 1;
    }

    let test_app = spawn_app().await;

    test_app.generate_posts(post_num as u16).await;

    for i in 0..page_amount {
        let o = if i <= max_page { 0 } else { 1 };
        let index = if i <= max_page { i } else { i - max_page - 1 };
        let response = test_app
            .get_request(&format!("/blog?index={}&o={}", index, o))
            .await
            .expect("Failed to request route \"/blog\".");

        let body = response.text().await.unwrap();

        for j in 1..page_span + 1 {
            let idx = index - j;
            if idx > 0 {
                assert!(body.contains(&format!(
                    r#"<a class="page_num" href="/blog?index={}&amp;o={}">{}</a>"#,
                    idx, o, idx
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
                    r#"<a class="page_num" href="/blog?index={}&amp;o={}">{}</a>"#,
                    idx, o, idx
                )));
            }
        }

        for _ in 0..max_per_page {
            assert!(body.contains(&format!(
                r#"class="post_card_author">{}</a>"#,
                &test_app.test_admin.username
            )));
            post_num = post_num - 1;
            assert!(body.contains(&format!(r#"Content: {}</p>"#, post_num)));
        }
        assert!(!body.contains(&format!(r#"Content: {}</p>"#, post_num - 1)));
        if i == max_page {
            assert!(body.contains(
                r#"<a class= "page_num page_offset" href="/blog?index=0&amp;o=1">More posts</a>"#
            ));
        } else if i == max_page + 1 {
            assert!(body.contains(
                r#"<a class="page_num page_offset" href="/blog?index=0&amp;o=0">Previous posts</a>"#
            ));
        } else {
            assert!(!body.contains(
                r#"<a class= "page_num page_offset" href="/blog?index=0&amp;o=1">More posts</a>"#
            ));
        }
    }
}

#[tokio::test]
async fn blog_respond_with_400_if_user_tampers_with_query_parameters() {
    let test_app = spawn_app().await;

    let response = test_app
        .get_request("/blog?index=0&o=0")
        .await
        .expect("Failed to request route \"/blog\".");

    let body = response.text().await.unwrap();

    assert!(body.contains(r#"<h3 class="post_card_h3_error">Currently there are no posts to be displayed, please try again later.</h3>"#));

    let response = test_app
        .get_request("/blog?index=0&o=1")
        .await
        .expect("Failed to request route \"/blog\".");

    assert!(response.status().as_u16() == 400);
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::Method::GET;
    use httpmock::{Mock, MockServer};

    #[tokio::test]
    async fn test_scrape() {
        // モックサーバーを立ち上げる
        let server = MockServer::start().await;

        let mock = Mock::new()
            .expect_method(GET)
            .expect_path("/mock_page")
            .return_status(200)
            .return_body("<a href='test_link.html'>Test Link</a>")
            .create_on(&server);

        let url = format!("{}/mock_page", server.url());

        // スクレイピング関数をテスト
        let links = scrape_links(&url).await.unwrap();

        // モックサーバーが期待通りのリクエストを受け取ったことを確認
        mock.assert();

        // スクレイピング関数が期待通りの結果を返したことを確認
        assert_eq!(links, vec!["test_link.html"]);
    }
}

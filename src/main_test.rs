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
    #[tokio::test]
    async fn test_fetch_page() {
        // Start a mock server.
        let server = MockServer::start().await;

        // Define a mock.
        let mock = Mock::new()
            .expect_method(GET)
            .expect_path("/PublicInfo")
            .return_status(200)
            .return_body("This is the mock response body")
            .create_on(&server);

        // The URL to be fetched is the mock server URL.
        let url = server.url("/PublicInfo");

        // Call the function to be tested.
        let response = fetch_page(&url).await.unwrap();

        // Check the response.
        assert_eq!(response, "This is the mock response body");

        // Ensure the mock was called.
        mock.assert_hits(1);
    }
}

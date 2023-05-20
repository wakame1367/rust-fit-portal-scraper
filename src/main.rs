use reqwest::Error;
use scraper::{Html, Selector};
use url::Url;

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::Method::GET;
    use httpmock::MockServer;

    #[tokio::test]
    async fn test_fetch_page() {
        // Start a mock server.
        let server = MockServer::start();

        // Define a mock.
        let mock = server.mock(|when, then| {
            when.method(GET).path("/PublicInfo");
            then.status(200).body("This is the mock response body");
        });

        // The URL to be fetched is the mock server URL.
        let url = server.url("/PublicInfo");

        // Call the function to be tested.
        let response = fetch_page(&url).await.unwrap();

        // Check the response.
        assert_eq!(response, "This is the mock response body");

        // Ensure the mock was called.
        mock.assert();
    }
}

async fn fetch_page(url: &str) -> Result<String, Error> {
    let text = reqwest::get(url).await?.text().await?;
    Ok(text)
}

async fn extract_download_link(page: &str, base: &Url) -> Result<Url, url::ParseError> {
    let document = Html::parse_document(page);
    let selector = Selector::parse("a[href]").unwrap();
    let url = document
        .select(&selector)
        .filter_map(|el| el.value().attr("href"))
        .find(|&href| href.contains("servlet.FileDownload"))
        .ok_or_else(|| url::ParseError::SetHostOnCannotBeABaseUrl)?;
    let url = base.join(url)?;
    Ok(url)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = Url::parse("https://www.fit-portal.go.jp/PublicInfo")?;
    let page = fetch_page(base_url.as_str()).await?;
    let download_link = extract_download_link(&page, &base_url).await?;
    println!("{}", download_link);
    Ok(())
}

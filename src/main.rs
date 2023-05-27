use reqwest::get;
use reqwest::Error;
use scraper::{Html, Selector};
use tokio::fs::File as AsyncFile;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use url::Url;

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use tokio::io::AsyncReadExt;
    use url::Url;

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
    #[tokio::test]
    async fn test_extract_download_link() {
        let base = Url::parse("https://www.fit-portal.go.jp/PublicInfo").unwrap();
        let page = r#"
        <html>
            <body>
                <a href="servlet.FileDownload?file=00P0K00002BkA63UAF">Download</a>
            </body>
        </html>
        "#;

        let expected = Url::parse(
            "https://www.fit-portal.go.jp/PublicInfo/servlet.FileDownload?file=00P0K00002BkA63UAF",
        )
        .unwrap();
        let actual = extract_download_link(page, &base).await.unwrap();

        assert_eq!(actual, expected);
    }
    #[tokio::test]
    async fn test_download_file() {
        // Create a named temporary file
        let tmpfile = tempfile::NamedTempFile::new().unwrap();
        // Get the path of the temporary file
        let tmp_path = tmpfile.path().to_str().unwrap();
        // Set up a mock server to act as the file server
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(GET).path("/testfile.txt");
            then.status(200)
                .header(
                    "Content-Disposition",
                    "attachment; filename=\"testfile.txt\"",
                )
                .body("This is a test file");
        });

        // Use the mock server's URL in the download_file function
        let url = Url::parse(&format!("{}/testfile.txt", server.url("/"))).unwrap();

        // Call the download_file function
        download_file(url, tmp_path).await.unwrap();

        // Create an asynchronous file handle
        let mut file = AsyncFile::create(tmp_path).await.unwrap();

        // Write to the file
        file.write_all(b"This is a test file").await.unwrap();
        // Check that the file was downloaded correctly
        let mut file = File::open(tmp_path).await.unwrap();
        let mut contents = vec![];
        file.read_to_end(&mut contents).await.unwrap();

        assert_eq!(contents, b"This is a test file");

        // Check that the mock was called exactly once
        mock.assert_hits(1);
    }
    #[tokio::test]
    async fn test_extract_download_links() {
        let base = Url::parse("https://www.fit-portal.go.jp/PublicInfo").unwrap();
        let page = r#"
    <html>
        <body>
            <a href="servlet.FileDownload?file=00P0K00002BkA63UAF">Download</a>
            <a href="servlet.FileDownload?file=00P0K00002BkA63UAG">Download</a>
        </body>
    </html>
    "#;

        let expected = vec![
        Url::parse(
            "https://www.fit-portal.go.jp/PublicInfo/servlet.FileDownload?file=00P0K00002BkA63UAF",
        )
        .unwrap(),
        Url::parse(
            "https://www.fit-portal.go.jp/PublicInfo/servlet.FileDownload?file=00P0K00002BkA63UAG",
        )
        .unwrap(),
    ];
        let actual = extract_download_links(page, &base).await.unwrap();

        assert_eq!(actual, expected);
    }
}

async fn fetch_page(url: &str) -> Result<String, Error> {
    let text = reqwest::get(url).await?.text().await?;
    Ok(text)
}

async fn extract_download_links(page: &str, base: &Url) -> Result<Vec<Url>, url::ParseError> {
    let document = Html::parse_document(page);
    let selector = Selector::parse("a[href]").unwrap();
    let urls = document
        .select(&selector)
        .filter_map(|el| el.value().attr("href"))
        .filter(|href| href.contains("servlet.FileDownload"))
        .map(|href| base.join(href))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(urls)
}

async fn download_file(
    url: Url,
    destination: &str,
    index: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    // HTTP GETリクエストを実行し、レスポンスを取得します。
    let response = get(url.clone()).await?;

    let filename = format!("file_{}", index);

    let destination = format!("{}{}", destination, filename);

    // ファイルを開きます（存在しない場合は新規作成します）。
    let mut dest = AsyncFile::create(destination).await?;

    // レスポンスボディをファイルにコピーします。
    let content = response.bytes().await?;
    dest.write_all(&content).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = Url::parse("https://www.fit-portal.go.jp/PublicInfo")?;
    let page = fetch_page(base_url.as_str()).await?;
    let download_links = extract_download_links(&page, &base_url).await?;
    println!("{:?}", download_links);
    let destination = "downloaded_file";
    for (index, download_link) in download_links.iter().enumerate() {
        download_file(download_link.clone(), destination, index).await?;
    }
    // ファイルのダウンロードが完了したことを示すメッセージを出力します。
    println!("Downloaded files to {}", destination);
    Ok(())
}

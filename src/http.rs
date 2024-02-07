use reqwest::get as getReqwest;
use url::Url;

// Queries the given URL and returns the response body.
pub async fn get_body(url: Url) -> reqwest::Result<String> {
    getReqwest(url).await?.text().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_body_pass() {
        let url = Url::parse("https://httpbin.org/get").unwrap();
        let res = get_body(url).await;
        assert_eq!(res.is_ok(), true, "the request should be successful");
    }

    #[tokio::test]
    async fn test_get_body_fail() {
        let url = Url::parse("https://invalidurl.org/get").unwrap();
        let res = get_body(url).await;
        assert_eq!(res.is_err(), true);
    }
}

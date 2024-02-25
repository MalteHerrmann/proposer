use crate::{errors::CommonwealthError, helper::UpgradeHelper, http::get_body};
use url::Url;

/// Check if the page body can be retrieved (i.e. the link is valid) and do some basic
/// validation on the expected contents.
pub async fn check_commonwealth_link(
    commonwealth_link: &str,
    upgrade_helper: &UpgradeHelper,
) -> Result<(), CommonwealthError> {
    let body = get_body(Url::parse(commonwealth_link)?).await?;

    if !body.contains(upgrade_helper.target_version.as_str())
        || !body.contains(upgrade_helper.network.to_string().as_str())
    // FIXME: sending a GET request to Commonwealth does not return the full contents of the proposal? Only a couple of lines with ... at the end?
    // || !body.contains(upgrade_helper.previous_version.to_string().as_str())
    // || !body.contains(upgrade_helper.upgrade_height.to_string().as_str())
    {
        return Err(CommonwealthError::InvalidCommonwealthLink);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::Network;
    use chrono::Utc;
    use std::path::PathBuf;
    use std::str::FromStr;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Sets up a mock server to return the given response template
    /// mocking the Commonwealth page contents.
    async fn setup_mock_api() -> MockServer {
        let template = ResponseTemplate::new(200)
            .set_body_string(include_str!("testdata/commonwealth_response.html"));

        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/evmos/discussion/14754-evmos-mainnet-v1600-upgrade"))
            .respond_with(template)
            .mount(&mock_server)
            .await;

        mock_server
    }

    #[tokio::test]
    async fn test_check_commonwealth_link_pass() {
        let helper = UpgradeHelper::new(
            PathBuf::from("./.evmosd"),
            Network::Mainnet,
            "v15.0.0",
            "v16.0.0",
            Utc::now(),
            60,
            "",
        );

        let mock_server = setup_mock_api().await;
        let mock_path = Url::from_str(mock_server.uri().as_str())
            .expect("failed to parse mock server uri")
            .join("/evmos/discussion/14754-evmos-mainnet-v1600-upgrade")
            .expect("failed to join url");

        assert!(check_commonwealth_link(mock_path.as_str(), &helper)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_check_commonwealth_link_fail() {
        let helper = UpgradeHelper::new(
            PathBuf::from("./.evmosd"),
            Network::Mainnet,
            "v11.1.2",
            "v14.0.0",
            Utc::now(),
            60,
            "",
        );

        let mock_server = setup_mock_api().await;
        let mock_path = Url::from_str(mock_server.uri().as_str())
            .expect("failed to parse mock server uri")
            .join("/evmos/discussion/14754-evmos-mainnet-v1600-upgrade")
            .expect("failed to join url");

        assert!(check_commonwealth_link(mock_path.as_str(), &helper)
            .await
            .is_err());
    }
}

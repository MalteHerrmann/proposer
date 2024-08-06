use crate::{errors::CommonwealthError, helper::UpgradeHelper, http::get_body};
use url::Url;

/// Check if the page body can be retrieved (i.e. the link is valid) and do some basic
/// validation on the expected contents.
pub async fn check_commonwealth_link(
    commonwealth_link: &str,
    _: &UpgradeHelper, // TODO: use when accessing the Commonwealth API
) -> Result<(), CommonwealthError> {
    // NOTE: for now we just check that the contents at the given URL can be retrieved
    let _ = get_body(Url::parse(commonwealth_link)?).await?;
    Ok(())

    // TODO: This is not working as expected because the returned contents in the body are not reliably containing the
    // desired information. This could probably be better with access to the Commonwealth API.
    // if !body.contains(upgrade_helper.target_version.as_str())
    //     || !body.contains(upgrade_helper.network.to_string().as_str())
    //     || !body.contains(upgrade_helper.previous_version.to_string().as_str())
    //     || !body.contains(upgrade_helper.upgrade_height.to_string().as_str())
    // {
    //     return Err(CommonwealthError::InvalidCommonwealthLink);
    // }
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
    async fn setup_mock_api(called_endpoint: &str) -> MockServer {
        let template = ResponseTemplate::new(200)
            .set_body_string(include_str!("testdata/commonwealth_response.html"));

        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path(called_endpoint))
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

        let endpoint = "/evmos/discussion/14754-evmos-mainnet-v1600-upgrade";
        let mock_server = setup_mock_api(endpoint).await;
        let mock_path = Url::from_str(mock_server.uri().as_str())
            .expect("failed to parse mock server uri")
            .join(endpoint)
            .expect("failed to join url");

        assert!(check_commonwealth_link(mock_path.as_str(), &helper)
            .await
            .is_ok());
    }
}

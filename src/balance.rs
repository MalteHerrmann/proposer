use crate::{config::NetworkConfig, errors::KeysError, http::get_body};
use serde::{Deserialize, Serialize};

const BALANCES_ENDPOINT: &str = "cosmos/bank/v1beta1/balances/";

/// Represents the response from a successful balance query for a specific denomination.
#[derive(Serialize, Deserialize)]
struct BalanceResponse {
    balance: Balance,
}

/// Represents the balance of an address for a specific denomination.
#[derive(Serialize, Deserialize)]
struct Balance {
    denom: String,
    amount: String,
}

/// Checks if a given address has a non-zero balance on the given network.
pub async fn has_balance(address: &str, network_config: &NetworkConfig) -> Result<bool, KeysError> {
    let balances_endpoint = network_config
        .rest
        .join(BALANCES_ENDPOINT)?
        .join(format!("{}/by_denom?denom={}", address, network_config.fee_denom).as_str())?;

    let balance: BalanceResponse =
        serde_json::from_str(get_body(balances_endpoint).await?.as_str())?;

    Ok(balance.balance.amount != "0")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use url::Url;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const TEST_ADDRESS: &str = "evmos1hafptm4zxy7y4fj6j7m6fj5n89v2zjy5l7ltae";

    /// Sets up a mock server to return the given response template
    /// when receiving a GET request on the release URL.
    /// Returns the mock server.
    ///
    /// This is used to mock requests to the Evmos blockchain.
    async fn setup_mock_api() -> MockServer {
        let balance_response: Value =
            serde_json::from_str(include_str!("testdata/balance_non_zero.json"))
                .expect("failed to parse balance JSON");

        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(
                BALANCES_ENDPOINT.to_owned() + format!("{}/by_denom", TEST_ADDRESS).as_str(),
            ))
            .and(query_param("denom", "aevmos"))
            .respond_with(ResponseTemplate::new(200).set_body_json(balance_response))
            .mount(&mock_server)
            .await;

        mock_server
    }

    #[tokio::test]
    async fn test_has_balance() {
        let mock_server = setup_mock_api().await;

        let network_config = NetworkConfig {
            rest: Url::parse(mock_server.uri().as_str()).unwrap(),
            ..NetworkConfig::default()
        };

        assert!(
            has_balance(TEST_ADDRESS, &network_config).await.unwrap(),
            "expected a non-zero balance"
        );
    }
}

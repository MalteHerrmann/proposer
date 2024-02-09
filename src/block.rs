extern crate reqwest;

use crate::errors::BlockError;
use crate::{http::get_body, network::Network};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use url::Url;

/// The number of blocks to use for the block time estimation.
pub const N_BLOCKS: u64 = 50_000;

/// The REST endpoint for querying blocks.
const BLOCKS_ENDPOINT: &str = "/cosmos/base/tendermint/v1beta1/blocks/";

/// The REST endpoint for querying the latest block.
const LATEST_BLOCK_ENDPOINT: &str = "/cosmos/base/tendermint/v1beta1/blocks/latest";

/// Represents a block from the Evmos network.
#[derive(Debug)]
pub struct Block {
    height: u64,
    time: DateTime<Utc>,
}

/// Represents the relevant information from the block query response
/// that is used for further processing.
#[derive(Serialize, Deserialize)]
struct BlockResponse {
    block: BlockInfo,
}

/// Represents the block information from the block query response.
#[derive(Serialize, Deserialize)]
struct BlockInfo {
    header: Header,
}

/// Represents the header information from the block query response.
#[derive(Serialize, Deserialize)]
struct Header {
    height: String,
    time: String,
}

/// Gets the estimated block height for the given upgrade time.
pub async fn get_estimated_height(
    base_url: &Url,
    upgrade_time: DateTime<Utc>,
) -> Result<u64, BlockError> {
    println!("Get latest block");
    let block = get_latest_block(&base_url).await?;
    println!("Get block: {}", block.height - N_BLOCKS);
    let block_minus_n = get_block(&base_url, block.height - N_BLOCKS).await?;
    let seconds_per_block: f32 =
        (block.time - block_minus_n.time).num_seconds() as f32 / N_BLOCKS as f32;

    let seconds_to_upgrade = (upgrade_time - block.time).num_seconds() as f32;
    let blocks_to_upgrade = (seconds_to_upgrade / seconds_per_block) as u64;

    Ok(blocks_to_upgrade + block.height)
}

/// Gets the latest block from the Evmos network.
async fn get_latest_block(base_url: &Url) -> Result<Block, BlockError> {
    let url = base_url.join(LATEST_BLOCK_ENDPOINT)?;
    let body = get_body(url).await?;

    process_block_body(body)
}

/// Gets the block at the given height from the Evmos network.
///
/// TODO: add mocking
async fn get_block(base_url: &Url, height: u64) -> Result<Block, BlockError> {
    // Combine the REST endpoint with the block height
    let url = base_url
        .join(BLOCKS_ENDPOINT)?
        .join(height.to_string().as_str())?;

    let body = get_body(url).await?;

    process_block_body(body)
}

/// Returns the appropriate REST provider for the given network.
pub fn get_rest_provider(network: Network) -> Url {
    let base_url = match network {
        Network::LocalNode => "http://localhost:1317",
        Network::Mainnet => "https://rest.evmos.lava.build",
        Network::Testnet => "https://rest.evmos-testnet.lava.build",
    };

    Url::parse(base_url).unwrap()
}

/// Processes the block body.
fn process_block_body(body: String) -> Result<Block, BlockError> {
    let body: BlockResponse = serde_json::from_str(&body)?;

    let height = body.block.header.height.parse::<u64>()?;

    let captured_time = Regex::new(r"[T0-9\-:]+")?
        .find(&body.block.header.time)
        .ok_or(BlockError::ParseTime)?
        .as_str();

    let time_format = "%Y-%m-%dT%H:%M:%S";
    let naive_date_time = NaiveDateTime::parse_from_str(captured_time, time_format)?;
    let time = Utc.from_utc_datetime(&naive_date_time);

    Ok(Block { height, time })
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::{Days, TimeZone};
    use serde_json::Value;
    use std::str::FromStr;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Sets up a mock server to return the given response template
    /// when receiving a GET request on the release URL.
    /// Returns the mock server.
    ///
    /// This is used to mock requests to the Evmos blockchain.
    async fn setup_mock_api() -> MockServer {
        let latest_block: Value = serde_json::from_str(include_str!("testdata/block_mainnet_18798834.json"))
                .expect("failed to parse block JSON");

        let block_minus_n: Value = serde_json::from_str(include_str!("testdata/block_mainnet_18748834.json"))
                .expect("failed to parse block JSON");

        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(LATEST_BLOCK_ENDPOINT))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(latest_block))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path(BLOCKS_ENDPOINT.to_owned() + "18748834"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(block_minus_n))
            .mount(&mock_server)
            .await;

        mock_server
    }

    #[tokio::test]
    async fn test_get_estimated_height() {
        let mock_server = setup_mock_api().await;
        let mock_path = Url::from_str(mock_server.uri().as_str())
                .expect("failed to parse mock server uri");

        let now = Utc::now();
        let upgrade_time = now.checked_add_days(Days::new(5)).unwrap();
        let res = get_estimated_height(&mock_path, upgrade_time).await;
        assert!(res.is_ok(), "expected no error; got: {}", res.unwrap_err());

        let height = res.unwrap();
        assert!(height > 18798834, "expected a different block height");
    }

    #[tokio::test]
    async fn test_get_latest_block_pass() {
        let mock_server = setup_mock_api().await;
        let mock_path =
            Url::from_str(mock_server.uri().as_str()).expect("failed to parse mock server uri");

        let res = get_latest_block(&mock_path).await;
        assert!(res.is_ok(), "expected no error; got: {}", res.unwrap_err());

        let block = res.unwrap();
        assert_eq!(block.height, 18798834, "expected different height");
    }

    #[tokio::test]
    async fn test_get_block_pass() {
        let mock_server = setup_mock_api().await;
        let mock_path =
            Url::from_str(mock_server.uri().as_str()).expect("failed to parse mock server uri");

        let res = get_block(&mock_path, 18748834).await;
        assert!(res.is_ok(), "expected no error; got: {}", res.unwrap_err());

        let block = res.unwrap();
        assert_eq!(block.height, 18748834, "expected a different block height");
        assert_eq!(
            block.time,
            Utc.with_ymd_and_hms(2024, 01, 05, 04, 39, 20).unwrap(),
            "expected a different block time",
        );
    }

    #[test]
    fn test_process_block_body_pass() {
        let block_response = include_str!("testdata/block_testnet.json");

        let res = process_block_body(block_response.to_string());
        assert!(res.is_ok(), "expected no error; got: {}", res.unwrap_err());

        let block = res.unwrap();
        assert_eq!(block.height, 18500000, "expected a different block height");
        assert_eq!(
            block.time,
            Utc.with_ymd_and_hms(2023, 11, 07, 02, 41, 36).unwrap(),
            "expected a different block time",
        );
    }
}

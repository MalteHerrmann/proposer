extern crate reqwest;
use crate::{http::get_body, network::Network};
use chrono::{DateTime, TimeZone, Utc};
use regex::Captures;
use url::Url;

/// The number of blocks to use for the block time estimation.
pub const N_BLOCKS: u64 = 50_000;

/// The REST endpoint for querying blocks.
const BLOCKS_ENDPOINT: &str = "cosmos/base/tendermint/v1beta1/blocks/";

/// The REST endpoint for querying the latest block.
const LATEST_BLOCK_ENDPOINT: &str = "cosmos/base/tendermint/v1beta1/blocks/latest";

/// Represents a block from the Evmos network.
#[derive(Debug)]
pub struct Block {
    height: u64,
    time: DateTime<Utc>,
}

/// Gets the estimated block height for the given upgrade time.
///
/// TODO: add error handling
pub async fn get_estimated_height(network: Network, upgrade_time: DateTime<Utc>) -> u64 {
    let base_url = get_rest_provider(network);
    let block = get_latest_block(base_url).await;
    let base_url = get_rest_provider(network);
    let block_minus_n = get_block(base_url, block.height - N_BLOCKS).await;
    let seconds_per_block: f32 =
        (block.time - block_minus_n.time).num_seconds() as f32 / N_BLOCKS as f32;

    let seconds_to_upgrade = (upgrade_time - block.time).num_seconds() as f32;
    let blocks_to_upgrade = (seconds_to_upgrade / seconds_per_block) as u64;

    blocks_to_upgrade + block.height
}

/// Gets the latest block from the Evmos network.
///
/// TODO: add error handling
async fn get_latest_block(base_url: Url) -> Block {
    let url = base_url
        .join(LATEST_BLOCK_ENDPOINT)
        .expect("the latest block endpoint should be valid");

    let body = get_body(url)
        .await
        .expect("the latest block should be successfully queried");

    process_block_body(body)
}

/// Gets the block at the given height from the Evmos network.
///
/// TODO: add error handling
/// TODO: add mocking
async fn get_block(base_url: Url, height: u64) -> Block {
    // Combine the REST endpoint with the block height
    let url = base_url
        .join(BLOCKS_ENDPOINT)
        .expect("the blocks endpoint should be valid")
        .join(height.to_string().as_str())
        .expect("the blocks endpoint should be valid");

    let body = get_body(url)
        .await
        .expect("the block should be successfully queried");

    process_block_body(body)
}

/// Returns the appropriate REST provider for the given network.
///
/// TODO: add error handling
fn get_rest_provider(network: Network) -> Url {
    let base_url = match network {
        Network::LocalNode => "http://localhost:1317",
        Network::Mainnet => "https://rest.evmos.lava.build",
        Network::Testnet => "https://rest.evmos-testnet.lava.build",
    };

    Url::parse(base_url).unwrap()
}

/// Processes the block body.
///
/// TODO: add error handling
fn process_block_body(body: String) -> Block {
    // build regex to find the block height
    let re = regex::Regex::new(r#"height":"(\d+)","time":"([T0-9\-:]+)"#).unwrap();

    let captures: Captures;
    let captures_res = re.captures(&body);
    match captures_res {
        None => panic!("failed to parse block response body"),
        Some(c) => captures = c,
    }

    // Extract the block height
    let captured_height = captures.get(1).map_or("", |m| m.as_str());
    let parsed_height = captured_height.parse::<u64>();
    let height: u64;
    match parsed_height {
        Ok(h) => height = h,
        Err(_) => panic!("Could not parse block height"),
    }

    // Parse the block time
    let captured_time = captures.get(2).map_or("", |m| m.as_str());
    let time_format = "%Y-%m-%dT%H:%M:%S";
    let time_res = chrono::NaiveDateTime::parse_from_str(captured_time, time_format);
    let time: DateTime<Utc>;
    match time_res {
        Ok(t) => time = Utc.from_utc_datetime(&t),
        Err(e) => panic!("Could not parse block time: {}", e),
    }

    Block { height, time }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::Network;
    use chrono::{Days, TimeZone};

    #[tokio::test]
    async fn test_get_estimated_height() {
        let now = Utc::now();
        let upgrade_time = now.checked_add_days(Days::new(5)).unwrap();
        let height = get_estimated_height(Network::Mainnet, upgrade_time).await;
        assert!(height > 16705125, "expected a different block height");
    }

    #[tokio::test]
    async fn test_get_latest_block_mainnet() {
        let block = get_latest_block(get_rest_provider(Network::Mainnet)).await;
        assert!(block.height > 0);
    }

    // TODO: add mocking
    #[tokio::test]
    async fn test_get_latest_block_testnet() {
        let base_url = get_rest_provider(Network::Testnet);
        let block = get_latest_block(base_url).await;
        assert!(block.height > 0);
    }

    // TODO: add mocking
    #[tokio::test]
    async fn test_get_block_mainnet() {
        let base_url = get_rest_provider(Network::Mainnet);
        let block = get_block(base_url, 16705125).await;
        assert_eq!(block.height, 16705125, "expected a different block height");
        assert_eq!(
            block.time,
            Utc.with_ymd_and_hms(2023, 10, 25, 17, 21, 50).unwrap(),
            "expected a different block time",
        );
    }

    // TODO: add mocking
    #[tokio::test]
    async fn test_get_block_testnet() {
        let base_url = get_rest_provider(Network::Testnet);
        let block = get_block(base_url, 18500000).await;
        assert_eq!(block.height, 18500000, "expected a different block height");
        assert_eq!(
            block.time,
            Utc.with_ymd_and_hms(2023, 10, 25, 17, 22, 23).unwrap(),
            "expected a different block time",
        );
    }

    #[test]
    fn test_process_block_body_pass() {
        let body = r#"{"block_id":{"hash":"CDHpDYu4tRibegIDTHust45sWB6ebnNE0Wq4sMpbSP8=","part_set_header":{"total":1,"hash":"bLAKlbU5Y0rqC1p07Xuhxm355sa+wPxwD9roDtnIzqA="}},"block":{"header":{"version":{"block":"11","app":"0"},"chain_id":"evmos_9001-2","height":"16699401","time":"2023-10-25T10:09:34.440526177Z","last_block_id""#;
        let block = process_block_body(body.to_string());

        assert_eq!(block.height, 16699401, "expected a different block height");
        assert_eq!(
            block.time,
            Utc.with_ymd_and_hms(2023, 10, 25, 10, 09, 34).unwrap(),
            "expected a different block time",
        );
    }
}

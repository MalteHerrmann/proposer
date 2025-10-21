use chrono::{DateTime, Duration, Utc};
use std::{path::PathBuf, str::FromStr};
use url::Url;

/// Contains the network information for a given network.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NetworkConfig {
    pub allow_rc: bool,
    pub binary: PathBuf,
    pub chain_id: String,
    pub cosmos_rpc: Url,
    pub fee_denom: String,
    pub name: String,
    pub path: PathBuf,
    pub rest: Url,
    /// The desired time of the upgrade in UTC.
    pub target_time_utc: String,
    // TODO: potentially remove going forward
    pub voting_period: Option<i64>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            allow_rc: false,
            binary: PathBuf::new(),
            chain_id: "".into(),
            cosmos_rpc: Url::parse("http://localhost:26657").expect("cosmos rpc url"),
            fee_denom: "".into(),
            name: "".into(),
            path: PathBuf::new(),
            rest: Url::parse("http://localhost:1317").expect("rest url"),
            target_time_utc: "".into(),
            voting_period: None,
        }
    }
}

/// Returns the available network configurations.
///
/// TODO: This should eventually read from YAML.
pub fn get_available_configs() -> Vec<NetworkConfig> {
    vec![
        NetworkConfig {
            allow_rc: true,
            binary: PathBuf::from_str("evmosd").expect("binary"),
            chain_id: "evmos_9002-1".into(),
            cosmos_rpc: Url::parse("http://localhost:26657").expect("cosmos rpc url"),
            fee_denom: "aevmos".into(),
            name: "Local Node".into(),
            path: PathBuf::from_str("/Users/malteherrmann/.tmp-evmosd").expect("node home path"),
            rest: Url::parse("http://localhost:1317").expect("rest url"),
            target_time_utc: "16:00".into(),
            voting_period: Some(Duration::hours(1).num_hours()),
        },
        NetworkConfig {
            allow_rc: true,
            binary: PathBuf::from_str("evmosd").expect("binary"),
            chain_id: "evmos_9001-1".into(),
            cosmos_rpc: Url::parse("https://rpc.evmos-testnet.lava.build:443")
                .expect("cosmos rpc url"),
            fee_denom: "atevmos".into(),
            name: "Testnet".into(),
            path: PathBuf::from_str("/Users/malteherrmann/.evmosd").expect("node home path"),
            rest: Url::parse("https://rest.evmos-testnet.lava.build").expect("rest url"),
            target_time_utc: "16:00".into(),
            voting_period: Some(Duration::hours(12).num_hours()),
        },
        NetworkConfig {
            allow_rc: false,
            binary: PathBuf::from_str("evmosd").expect("binary"),
            chain_id: "evmos_9000-4".into(),
            cosmos_rpc: Url::parse("https://rpc.evmos.lava.build:443").expect("cosmos rpc url"),
            fee_denom: "aevmos".into(),
            name: "Mainnet".into(),
            path: PathBuf::from_str("/Users/malteherrmann/.evmosd").expect("node home path"),
            rest: Url::parse("https://rest.evmos.lava.build").expect("rest url"),
            target_time_utc: "16:00".into(),
            voting_period: Some(Duration::hours(120).num_hours()),
        },
        NetworkConfig {
            allow_rc: false,
            binary: PathBuf::from_str("nobled").expect("binary"),
            chain_id: "noble-1".into(),
            cosmos_rpc: Url::parse("https://rpc.noble.xyz:443").expect("cosmos rpc url"),
            fee_denom: "uusdc".into(),
            name: "Noble Mainnet".into(),
            // TODO: use $HOME instead and expand, but doesn't work like that out of the box but
            // requires manual parsing.
            path: PathBuf::from_str("/Users/malteherrmann/.noble").expect("node home path"),
            rest: Url::parse("https://api.noble.xyz").expect("rest url"),
            target_time_utc: "16:00".into(),
            voting_period: None,
        },
        NetworkConfig {
            allow_rc: false,
            binary: PathBuf::from_str("nobled").expect("binary"),
            chain_id: "grand-1".into(),
            cosmos_rpc: Url::parse("https://rpc.testnet.noble.xyz:443").expect("cosmos rpc url"),
            fee_denom: "uusdc".into(),
            name: "Noble Testnet".into(),
            path: PathBuf::from_str("/Users/malteherrmann/.noble").expect("node home path"),
            rest: Url::parse("https://api.testnet.noble.xyz").expect("rest url"),
            target_time_utc: "16:00".into(),
            voting_period: None,
        },
    ]
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct UpgradeConfig {
    /// The previous version to upgrade from.
    pub previous_version: String,
    /// The summary of the changes in the release.
    pub summary: String,
    /// The target version to upgrade to.
    pub target_version: String,
    /// The name of the upgrade.
    pub upgrade_name: String,
    /// The projected time of the upgrade.
    pub upgrade_time: DateTime<Utc>,
    /// The block height where the upgrade is applied.
    pub upgrade_height: u64,
}

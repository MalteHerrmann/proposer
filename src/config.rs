use chrono::Duration;
use std::{path::PathBuf, str::FromStr};

/// Contains the configuration of the proposer tool for a given
/// profile.
pub struct Config {
    pub networks: Vec<NetworkConfig>,
}

/// Contains the network information for a given network.
#[derive(Clone, Default)]
pub struct NetworkConfig {
    pub name: String,
    pub path: PathBuf,
    pub allow_rc: bool,
    pub rest: String,
    // TODO: potentially remove going forward
    pub voting_period: Option<Duration>,
    pub target_time_utc: String,
}

pub fn get_evmos_config() -> Config {
    Config {
        networks: vec![
            NetworkConfig {
                name: "Local Node".to_string(),
                path: PathBuf::from_str("/Users/malte/.tmp-evmosd").expect("node home path"),
                allow_rc: true,
                rest: "http://localhost:1317".into(),
                voting_period: Some(Duration::hours(1)),
                target_time_utc: "16:00".into(),
            },
            NetworkConfig {
                name: "Testnet".to_string(),
                path: PathBuf::from_str("/Users/malte/.evmosd").expect("node home path"),
                allow_rc: true,
                rest: "https://rest.evmos-testnet.lava.build".into(),
                voting_period: Some(Duration::hours(12)),
                target_time_utc: "16:00".into(),
            },
            NetworkConfig {
                name: "Mainnet".to_string(),
                path: PathBuf::from_str("/Users/malte/.evmosd").expect("node home path"),
                allow_rc: false,
                rest: "https://rest.evmos.lava.build".into(),
                voting_period: Some(Duration::hours(120)),
                target_time_utc: "16:00".into(),
            },
        ],
    }
}

#[derive(Default)]
pub struct UpgradeConfig {
    pub upgrade_name: String,
}

pub fn get_example_upgrade() -> UpgradeConfig {
    UpgradeConfig {
        upgrade_name: "Evmos v10.0.0 Testnet Upgrade".into(),
    }
}

use crate::block::get_estimated_height;
use crate::errors::{HelperError, InputError, ValidationError};
use crate::llm::create_summary;
use crate::release::{get_instance, get_release};
use crate::{inputs, network::Network, version};
use chrono::{DateTime, Duration, Utc};
use std::path::{Path, PathBuf};
use std::{fs, io};

/// Contains all relevant information for the scheduled upgrade.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct UpgradeHelper {
    /// The chain ID of the node.
    pub chain_id: String,
    /// The name of the config file.
    pub config_file_name: String,
    /// The home directory of the node.
    pub home: PathBuf,
    /// The network to create the commands and proposal description for.
    pub network: Network,
    /// The previous version to upgrade from.
    pub previous_version: String,
    /// The name of the proposal.
    pub proposal_name: String,
    /// The name of the proposal file.
    pub proposal_file_name: String,
    /// The summary of the changes in the release.
    pub summary: String,
    /// The target version to upgrade to.
    pub target_version: String,
    /// The scheduled height of the upgrade.
    pub upgrade_height: u64,
    /// The scheduled time of the upgrade.
    pub upgrade_time: DateTime<Utc>,
    /// The number of hours for the voting period.
    pub voting_period: i64,
}

impl UpgradeHelper {
    /// Creates a new instance of the upgrade helper.
    pub fn new(
        network: Network,
        previous_version: &str,
        target_version: &str,
        upgrade_time: DateTime<Utc>,
        upgrade_height: u64,
        summary: &str,
    ) -> UpgradeHelper {
        let chain_id = get_chain_id(network);
        // TODO: Get from input eventually
        let home = get_home(network);

        let proposal_name = format!("Evmos {} {} Upgrade", network, target_version);
        let voting_period = get_voting_period(network);
        let proposal_file_name = format!("proposal-{}-{}.md", network, target_version);
        let config_file_name = format!("proposal-{}-{}.json", network, target_version);

        UpgradeHelper {
            chain_id,
            config_file_name,
            home,
            network,
            previous_version: previous_version.to_string(),
            proposal_name,
            proposal_file_name,
            summary: summary.to_string(),
            target_version: target_version.to_string(),
            upgrade_height,
            upgrade_time,
            voting_period: voting_period.num_hours(),
        }
    }

    /// Validates the upgrade helper.
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Check if the target version is valid
        if !version::is_valid_version_for_network(self.network, self.target_version.as_str()) {
            return Err(ValidationError::TargetVersion(
                self.network,
                self.target_version.clone(),
            ));
        }

        // Check if the previous version is valid
        if !version::is_valid_version(self.previous_version.as_str()) {
            return Err(ValidationError::PreviousVersion(
                self.previous_version.clone(),
            ));
        }

        // Check if the upgrade time is valid
        let valid_time = inputs::is_valid_upgrade_time(self.upgrade_time);
        if !valid_time {
            return Err(ValidationError::UpgradeTime(self.upgrade_time));
        }

        // Check if home folder exists
        if !path_exists(&self.home) {
            return Err(ValidationError::HomeDir(self.home.clone()));
        }

        Ok(())
    }

    /// Exports the upgrade helper to a JSON file.
    pub fn write_to_json(&self) -> io::Result<()> {
        let json = serde_json::to_string_pretty(&self).expect("Failed to convert to JSON");
        let path = Path::new(&self.config_file_name);

        fs::write(&path, json)
    }
}

/// Returns the upgrade helper from a JSON file.
pub fn from_json(path: &Path) -> Result<UpgradeHelper, HelperError> {
    let json = fs::read_to_string(path)?;

    Ok(serde_json::from_str(&json)?)
}

/// Returns the upgrade helper from the command line arguments and
/// runs some basic validation on the configuration.
pub fn get_helper_from_json(path: &Path) -> Result<UpgradeHelper, HelperError> {
    let upgrade_helper = from_json(path)?;
    upgrade_helper.validate()?;

    Ok(upgrade_helper)
}

/// Creates a new instance of the upgrade helper based on querying the user for the necessary input.
pub async fn get_helper_from_inputs() -> Result<UpgradeHelper, InputError> {
    // Query and check the network to use
    let used_network = inputs::get_used_network()?;

    // Query and check the version to upgrade from
    let previous_version = inputs::get_text("Previous version to upgrade from:")?;
    let valid_version = version::is_valid_version(previous_version.as_str());
    if !valid_version {
        return Err(InputError::from(ValidationError::PreviousVersion(
            previous_version,
        )));
    }

    // Query and check the target version to upgrade to
    let target_version = inputs::get_text("Target version to upgrade to:")?;
    if !version::is_valid_version_for_network(used_network, target_version.as_str()) {
        return Err(InputError::from(ValidationError::TargetVersion(
            used_network,
            target_version,
        )));
    }

    // Query and check the upgrade time and height
    let voting_period = get_voting_period(used_network);
    let upgrade_time = inputs::get_upgrade_time(voting_period, Utc::now())?;
    let upgrade_height = get_estimated_height(used_network, upgrade_time).await?;

    // Query and check the summary of the changes in the release
    let release = get_release(get_instance().as_ref(), target_version.as_str()).await?;
    let summary = create_summary(&release).await?;

    // Create an instance of the helper
    Ok(UpgradeHelper::new(
        used_network,
        previous_version.as_str(),
        target_version.as_str(),
        upgrade_time,
        upgrade_height,
        summary.as_str(),
    ))
}

#[cfg(test)]
mod helper_tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_new_upgrade_helper() {
        let network = Network::Testnet;
        let previous_version = "v14.0.0";
        let target_version = "v14.0.0-rc1";
        let upgrade_time = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();
        let helper = UpgradeHelper::new(
            network,
            previous_version,
            target_version,
            upgrade_time,
            60,
            "",
        );
        assert_eq!(helper.chain_id, "evmos_9000-4");
        assert_eq!(helper.config_file_name, "proposal-Testnet-v14.0.0-rc1.json");
        assert!(
            helper.home.to_str().unwrap().contains(".evmosd"),
            "expected different home directory"
        );
        assert_eq!(helper.network, Network::Testnet);
        assert_eq!(helper.previous_version, "v14.0.0");
        assert_eq!(helper.proposal_name, "Evmos Testnet v14.0.0-rc1 Upgrade");
        assert_eq!(helper.proposal_file_name, "proposal-Testnet-v14.0.0-rc1.md");
        assert_eq!(helper.target_version, "v14.0.0-rc1");
    }

    #[test]
    fn test_write_to_json_and_read_from_json() {
        let upgrade_height = 60;
        let helper = UpgradeHelper::new(
            Network::Testnet,
            "v14.0.0",
            "v14.0.0-rc1",
            Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap(),
            upgrade_height,
            "",
        );

        assert!(
            helper.write_to_json().is_ok(),
            "expected success writing helper information to JSON file"
        );

        // assert that the config file exists
        let path = Path::new(&helper.config_file_name);
        assert!(path_exists(path), "expected config file to exist");

        let read_input_helper = from_json(path).expect("failed to read helper from JSON file");
        assert_eq!(helper.chain_id, read_input_helper.chain_id);
        assert_eq!(helper.config_file_name, read_input_helper.config_file_name);
        assert_eq!(helper.upgrade_height, read_input_helper.upgrade_height);

        // remove the config file
        match fs::remove_file(&path) {
            Ok(_) => {}
            Err(e) => {
                println!("Failed to remove file '{}': {}", path.to_str().unwrap(), e);
                assert!(false, "expected success removing config file");
            }
        }
    }
}

/// Checks whether a given path exists.
fn path_exists(path: &Path) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        metadata.is_dir() || metadata.is_file()
    } else {
        false
    }
}

#[cfg(test)]
mod path_tests {
    use super::*;

    #[test]
    fn test_path_exists() {
        let path = Path::new("/tmp");
        assert_eq!(path_exists(path), true);
    }

    #[test]
    fn test_path_does_not_exist() {
        let path = Path::new("/tmp/does-not-exist");
        assert_eq!(path_exists(path), false);
    }
}

/// Returns the voting period duration based on the network.
pub fn get_voting_period(network: Network) -> Duration {
    match network {
        Network::LocalNode => Duration::hours(1),
        Network::Testnet => Duration::hours(12),
        Network::Mainnet => Duration::hours(120),
    }
}

/// Returns the home directory based on the network.
fn get_home(network: Network) -> PathBuf {
    // home dir of user
    let mut user_home = dirs::home_dir().expect("Failed to get home directory");
    match network {
        Network::LocalNode => user_home.push(".tmp-evmosd"),
        Network::Testnet => user_home.push(".evmosd"),
        Network::Mainnet => user_home.push(".evmosd"),
    }
    user_home
}

/// Returns the chain ID based on the network.
fn get_chain_id(network: Network) -> String {
    match network {
        Network::LocalNode => "evmos_9000-4".to_string(),
        Network::Testnet => "evmos_9000-4".to_string(),
        Network::Mainnet => "evmos_9001-2".to_string(),
    }
}

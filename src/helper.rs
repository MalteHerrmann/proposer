use crate::{
    block::{get_estimated_height, round_to_nearest_500},
    config,
    errors::{HelperError, InputError, ValidationError},
    evmosd, inputs,
    llm::{create_summary, OpenAIModel},
    release::{get_instance, get_release},
    version,
};
use chrono::Utc;
use std::path::Path;
use std::{fs, io};

/// Contains all relevant information for the scheduled upgrade.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct UpgradeHelper {
    /// The link to the Commonwealth proposal (optional).
    pub commonwealth_link: Option<String>,
    /// The name of the config file.
    pub config_file_name: String,
    /// The configuration of the used node binary.
    pub network_config: config::NetworkConfig,
    /// The name of the proposal file.
    pub proposal_file_name: String,
    /// The configuration of the generated proposal contents.
    pub upgrade_config: config::UpgradeConfig,
}

impl UpgradeHelper {
    /// Creates a new instance of the upgrade helper.
    pub fn new(
        network_config: &config::NetworkConfig,
        upgrade_config: &config::UpgradeConfig,
    ) -> UpgradeHelper {
        let proposal_file_name = format!(
            "proposal-{}-{}.md",
            network_config.name, upgrade_config.target_version
        );
        let config_file_name = format!(
            "proposal-{}-{}.json",
            network_config.name, upgrade_config.target_version
        );

        UpgradeHelper {
            commonwealth_link: None,
            config_file_name,
            network_config: network_config.clone(),
            proposal_file_name,
            upgrade_config: upgrade_config.clone(),
        }
    }

    /// Validates the upgrade helper.
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Check if the target version is valid
        if !version::is_valid_version_for_network(
            &self.network_config,
            self.upgrade_config.target_version.as_str(),
        ) {
            return Err(ValidationError::TargetVersion(
                self.network_config.name.clone(),
                self.upgrade_config.target_version.clone(),
            ));
        }

        // Check if the previous version is valid
        if !version::is_valid_version(self.upgrade_config.previous_version.as_str()) {
            return Err(ValidationError::PreviousVersion(
                self.upgrade_config.previous_version.clone(),
            ));
        }

        // Check if the upgrade time is valid
        if !inputs::is_valid_upgrade_time(self.upgrade_config.upgrade_time) {
            return Err(ValidationError::UpgradeTime(
                self.upgrade_config.upgrade_time,
            ));
        }

        // Check if home folder exists
        if !path_exists(&self.network_config.path) {
            return Err(ValidationError::HomeDir(self.network_config.path.clone()));
        }

        // Check if the home folder contains the client configuration
        evmosd::get_client_config(&self.network_config.path.join("config/client.toml"))?;

        Ok(())
    }

    /// Exports the upgrade helper to a JSON file.
    pub fn write_to_json(&self) -> io::Result<()> {
        let json = serde_json::to_string_pretty(&self).expect("Failed to convert to JSON");
        let path = Path::new(&self.config_file_name);

        fs::write(path, json)
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
pub async fn get_helper_from_inputs(model: OpenAIModel) -> Result<UpgradeHelper, InputError> {
    // Query and check the network to use
    let mut network_config = inputs::get_network_config(&config::get_evmos_config())?;

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
    if !version::is_valid_version_for_network(&network_config, target_version.as_str()) {
        return Err(InputError::from(ValidationError::TargetVersion(
            network_config.name,
            target_version,
        )));
    }

    // Query and check the upgrade time and height
    let upgrade_time = inputs::get_upgrade_time(&network_config, Utc::now())?;
    let upgrade_height =
        round_to_nearest_500(get_estimated_height(&network_config.rest, upgrade_time).await?);

    // Query and check the summary of the changes in the release
    let release = get_release(get_instance().as_ref(), target_version.as_str()).await?;
    let summary = create_summary(&release, model).await?;

    // Get the used home directory for the Evmos binary.
    let evmosd_home = inputs::get_node_home(&network_config)?;
    network_config.path = evmosd_home;

    // Create an instance of the helper
    Ok(UpgradeHelper::new(
        &network_config,
        &config::UpgradeConfig {
            // TODO: get upgrade name from user
            upgrade_name: "".to_string(),
            previous_version,
            target_version,
            upgrade_time,
            upgrade_height,
            summary,
        },
    ))
}

#[cfg(test)]
mod helper_tests {
    use super::*;
    use crate::config::{NetworkConfig, UpgradeConfig};
    use chrono::TimeZone;

    #[test]
    fn test_new_upgrade_helper() {
        let upgrade_time = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();
        let helper = UpgradeHelper::new(
            &NetworkConfig::default(),
            &UpgradeConfig {
                upgrade_name: "".to_string(),
                previous_version: "v14.0.0".to_string(),
                target_version: "v14.0.0-rc1".to_string(),
                upgrade_time,
                upgrade_height: 60,
                summary: "".to_string(),
            },
        );

        assert_eq!(helper.upgrade_config.previous_version, "".to_string());
        assert_eq!(
            helper.upgrade_config.target_version,
            "v14.0.0-rc1".to_string()
        );
        assert_eq!(helper.upgrade_config.upgrade_time, upgrade_time);
        assert_eq!(helper.upgrade_config.upgrade_height, 60);
        assert_eq!(helper.upgrade_config.summary, "".to_string());
    }

    #[test]
    fn test_write_to_json_and_read_from_json() {
        let upgrade_height = 60;
        let helper = UpgradeHelper::new(
            &NetworkConfig::default(),
            &UpgradeConfig {
                upgrade_name: "".to_string(),
                previous_version: "v14.0.0".to_string(),
                target_version: "v14.0.0-rc1".to_string(),
                upgrade_time: Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap(),
                upgrade_height,
                summary: "".to_string(),
            },
        );

        assert!(
            helper.write_to_json().is_ok(),
            "expected success writing helper information to JSON file"
        );

        // assert that the config file exists
        let path = Path::new(&helper.config_file_name);
        assert!(path_exists(path), "expected config file to exist");

        let read_input_helper = from_json(path).expect("failed to read helper from JSON file");
        assert_eq!(
            helper.network_config.chain_id,
            read_input_helper.network_config.chain_id
        );
        assert_eq!(helper.config_file_name, read_input_helper.config_file_name);
        assert_eq!(
            helper.upgrade_config.previous_version,
            read_input_helper.upgrade_config.previous_version
        );
        assert_eq!(
            helper.upgrade_config.target_version,
            read_input_helper.upgrade_config.target_version
        );
        assert_eq!(
            helper.upgrade_config.upgrade_time,
            read_input_helper.upgrade_config.upgrade_time
        );
        assert_eq!(
            helper.upgrade_config.upgrade_height,
            read_input_helper.upgrade_config.upgrade_height
        );
        assert_eq!(
            helper.upgrade_config.summary,
            read_input_helper.upgrade_config.summary
        );

        // remove the config file
        assert!(
            fs::remove_file(path).is_ok(),
            "expected success removing config file"
        );
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
        assert!(path_exists(path));
    }

    #[test]
    fn test_path_does_not_exist() {
        let path = Path::new("/tmp/does-not-exist");
        assert!(!path_exists(path));
    }
}

use crate::errors::PrepareError;
use crate::evmosd::ClientConfig;
use crate::helper::UpgradeHelper;
use crate::release::{get_asset_string, get_instance, get_release};
use handlebars::{no_escape, Handlebars};
use serde_json::json;
use std::io;

/// Prepares the command to submit the proposal using the Evmos CLI.
pub async fn prepare_command(
    helper: &UpgradeHelper,
    client_config: &ClientConfig,
    key: &str,
) -> Result<String, PrepareError> {
    let mut description = get_description_from_md(&helper.proposal_file_name)?;
    let release = get_release(
        &get_instance(),
        helper.upgrade_config.target_version.as_str(),
    )
    .await?;
    let assets = get_asset_string(&release).await?;

    // TODO: get fees from network conditions?
    let fees = format!("10000000000{}", helper.network_config.fee_denom);

    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);
    handlebars.register_escape_fn(no_escape);

    handlebars
        .register_template_file("command", "src/templates/command.hbs")
        .expect("Failed to register command template file");

    handlebars
        .register_template_file("commonwealth_template", "src/templates/commonwealth.hbs")
        .expect("Failed to register commonwealth template file");

    if helper.commonwealth_link.is_some() {
        description = format!(
            "{}{}",
            description.as_str(),
            handlebars.render(
                "commonwealth_template",
                &json!({"commonwealth": helper.commonwealth_link})
            )?
        );
    }

    let data = json!({
        "assets": assets,
        "chain_id": helper.network_config.chain_id,
        "commonwealth": helper.commonwealth_link,
        "description": description.replace('\n', "\\n"),  // NOTE: this is necessary to not print the actual new lines when rendering the template.
        "fees": fees,
        "height": helper.upgrade_config.upgrade_height,
        "home": helper.network_config.path,
        "key": key,
        "keyring": client_config.keyring_backend,
        "title": helper.upgrade_config.upgrade_name,
        "tm_rpc": helper.network_config.cosmos_rpc,
        "version": helper.upgrade_config.target_version,
    });

    let command = handlebars.render("command", &data)?;

    Ok(command)
}

/// Returns the description string from the given Markdown file.
fn get_description_from_md(filename: &str) -> io::Result<String> {
    std::fs::read_to_string(filename)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{NetworkConfig, UpgradeConfig};
    use chrono::Utc;

    #[tokio::test]
    async fn test_prepare_command() {
        let nc = NetworkConfig::default();
        let uc = UpgradeConfig{
            previous_version: "v13.0.0".to_string(),
            target_version: "v14.0.0".to_string(),
            upgrade_time: Utc::now(),
            upgrade_height: 60,
            ..UpgradeConfig::default()
        };

        let helper = UpgradeHelper::new(&nc, &uc);

        let client_config = ClientConfig {
            chain_id: "evmos_9000-4".to_string(),
            keyring_backend: "test".to_string(),
            output: "text".to_string(),
            node: "https://tm.evmos-testnet.lava.build:443".to_string(),
            broadcast_mode: "sync".to_string(),
        };

        // Write description to file
        let description = "This is a test proposal.";
        std::fs::write(&helper.proposal_file_name, description)
            .expect("Unable to write proposal to file");

        // Parse the description and prepare exported command
        let command = prepare_command(&helper, &client_config, "dev0")
            .await
            .expect("failed to prepare command");

        // Remove description file
        std::fs::remove_file(&helper.proposal_file_name)
            .expect("failed to remove description file after test");

        assert_eq!(
            command,
            include_str!("testdata/example_command.sh"),
            "expected different proposal command"
        );
    }

    #[test]
    fn test_get_description_from_md() {
        let description = get_description_from_md("src/templates/command.hbs");
        assert!(description.is_ok(), "description should be ok, but is not");
    }

    #[test]
    fn test_get_description_from_md_invalid_file() {
        let description = get_description_from_md("src/templates/command.hbs.invalid");
        assert!(
            description.is_err(),
            "description should be err, but is not"
        );
    }
}

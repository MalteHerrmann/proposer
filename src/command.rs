use crate::errors::PrepareError;
use crate::evmosd::ClientConfig;
use crate::helper::UpgradeHelper;
use crate::network::{get_denom, Network};
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
    let release = get_release(&get_instance(), helper.target_version.as_str()).await?;
    let assets = get_asset_string(&release).await?;
    let denom = get_denom(helper.network);

    // TODO: get fees from network conditions?
    let fees = format!("10000000000{}", denom);
    let tm_rpc = get_rpc_url(helper.network);

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
        "chain_id": helper.chain_id,
        "commonwealth": helper.commonwealth_link,
        "description": description.replace('\n', "\\n"),  // NOTE: this is necessary to not print the actual new lines when rendering the template.
        "fees": fees,
        "height": helper.upgrade_height,
        "home": helper.evmosd_home,
        "key": key,
        "keyring": client_config.keyring_backend,
        "title": helper.proposal_name,
        "tm_rpc": tm_rpc,
        "version": helper.target_version,
    });

    let command = handlebars.render("command", &data)?;

    Ok(command)
}

/// Returns the description string from the given Markdown file.
fn get_description_from_md(filename: &str) -> io::Result<String> {
    std::fs::read_to_string(filename)
}

/// Returns the RPC URL based on the network.
fn get_rpc_url(network: Network) -> String {
    match network {
        Network::Mainnet => "https://tm.evmos.lava.build:443".to_string(),
        Network::Testnet => "https://tm.evmos-testnet.lava.build:443".to_string(),
        Network::LocalNode => "http://localhost:26657".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::Network;
    use chrono::Utc;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_prepare_command() {
        let helper = UpgradeHelper::new(
            PathBuf::from("./.evmosd"),
            Network::Testnet,
            "v13.0.0",
            "v14.0.0",
            Utc::now(),
            60,
            "",
        );

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

    #[test]
    fn test_get_rpc_url() {
        let rpc = get_rpc_url(Network::Mainnet);
        assert_eq!(rpc, "https://tm.evmos.lava.build:443", "rpc does not match");

        let rpc = get_rpc_url(Network::Testnet);
        assert_eq!(
            rpc, "https://tm.evmos-testnet.lava.build:443",
            "rpc does not match"
        );

        let rpc = get_rpc_url(Network::LocalNode);
        assert_eq!(rpc, "http://localhost:26657", "rpc does not match");
    }
}

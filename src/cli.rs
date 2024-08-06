use crate::evmosd::get_client_config;
use crate::{
    command,
    commonwealth::check_commonwealth_link,
    errors::{CommandError, ProposalError},
    helper::{get_helper_from_inputs, get_helper_from_json},
    inputs, keys,
    llm::OpenAIModel,
    network::Network,
    proposal, utils,
};
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

/// Utility to help with preparing software upgrades for the Evmos Core Team.
#[derive(Debug, Parser)]
pub struct Cli {
    /// The sub-command to execute.
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

/// This enum defines the sub-commands that can be executed.
///
/// Each sub-command has its own set of arguments.
#[derive(Debug, Subcommand)]
pub enum SubCommand {
    /// The `generate-proposal` sub-command.
    GenerateProposal(GenerateProposalArgs),
    /// The `generate-command` sub-command, which has an optional file path argument.
    GenerateCommand(GenerateCommandArgs),
}

/// This struct defines the pattern of the arguments for the `generate-proposal` sub-command.
#[derive(Debug, Clone, Args)]
pub struct GenerateProposalArgs {
    /// The LLM model to use for summarizing the release notes.
    #[clap(short, long, default_value_t = OpenAIModel::Gpt4o)]
    model: OpenAIModel,
}

/// This struct defines the pattern of the arguments for the `generate-command` sub-command.
/// There is one optional argument for this sub-command, which is the path to the configuration file.
/// If no path is provided, the default configuration file name is used.
#[derive(Debug, Clone, Args)]
pub struct GenerateCommandArgs {
    /// The path to the configuration file.
    #[clap(short, long)]
    config: Option<PathBuf>,
}

/// Runs the logic for the `generate-command` sub-command.
pub async fn generate_command(args: GenerateCommandArgs) -> Result<(), CommandError> {
    let helper_config_path = match args.config {
        Some(config_file_name) => config_file_name,
        None => inputs::choose_config()?, // NOTE: if no config file is provided, prompt the user to choose one
    };

    let mut upgrade_helper = get_helper_from_json(&helper_config_path)?;
    let client_config = get_client_config(
        upgrade_helper
            .evmosd_home
            .join("config/client.toml")
            .as_path(),
    )?;

    if upgrade_helper.network == Network::Mainnet {
        let commonwealth_link = inputs::choose_commonwealth_link().await?;
        check_commonwealth_link(&commonwealth_link, &upgrade_helper).await?;
        upgrade_helper.commonwealth_link = Some(commonwealth_link.clone());
    }

    let keys_with_balances = keys::get_keys_with_balances(keys::FilterKeysConfig {
        config: client_config.clone(),
        home: upgrade_helper.evmosd_home.clone(),
        network: upgrade_helper.network,
    })
    .await?;
    let key = inputs::get_key(keys_with_balances)?;

    // Prepare command to submit proposal
    let command = command::prepare_command(&upgrade_helper, &client_config, &key).await?;

    // Write command to file
    Ok(utils::write_content_to_file(
        &command,
        &upgrade_helper.proposal_file_name.replace(".md", ".sh"),
    )?)
}

/// Runs the logic for the `generate-proposal` sub-command.
///
/// This sub-command queries the user for the necessary information to prepare the proposal description
/// for a standard Evmos software upgrade.
pub async fn generate_proposal(args: GenerateProposalArgs) -> Result<(), ProposalError> {
    // Create an instance of the helper
    let upgrade_helper = get_helper_from_inputs(args.model).await?;

    // Validate the helper configuration
    upgrade_helper.validate()?;

    // Export the configuration
    upgrade_helper.write_to_json()?;

    // Render the proposal description
    let description = proposal::render_proposal(&upgrade_helper)?;

    // Write the proposal description to file
    Ok(utils::write_content_to_file(
        &description,
        &upgrade_helper.proposal_file_name,
    )?)
}

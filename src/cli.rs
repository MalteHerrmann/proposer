use crate::{
    command,
    helper::{get_helper_from_inputs, get_helper_from_json},
    inputs, proposal, utils,
};
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use std::process;

/// Utility to help with preparing software upgrades for the Evmos Core Team.
#[derive(Debug, Parser)]
pub struct CLI {
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
    GenerateProposal,
    /// The `generate-command` sub-command, which has an optional file path argument.
    GenerateCommand(GenerateCommandArgs),
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
pub async fn generate_command(args: GenerateCommandArgs) -> Result<(), Box<dyn std::error::Error>> {
    let config = match args.config {
        Some(config_file_name) => config_file_name,
        None => inputs::choose_config()?, // NOTE: if no config file is provided, prompt the user to choose one
    };

    let upgrade_helper = get_helper_from_json(&config)?;

    // Run the main functionality of the helper.
    command::run_command_preparation(&upgrade_helper).await
}

/// Runs the logic for the `generate-proposal` sub-command.
///
/// This sub-command queries the user for the necessary information to prepare the proposal description
/// for a standard Evmos software upgrade.
pub async fn generate_proposal() {
    // Create an instance of the helper
    let upgrade_helper = match get_helper_from_inputs().await {
        Ok(helper) => helper,
        Err(e) => {
            println!("Error creating helper: {}", e);
            process::exit(1);
        }
    };

    // Validate the helper configuration
    match upgrade_helper.validate() {
        Ok(_) => {}
        Err(e) => {
            println!("Invalid configuration: {}", e);
            process::exit(1);
        }
    };

    // Export the configuration
    match upgrade_helper.write_to_json() {
        Ok(_) => {
            println!(
                "successfully wrote config to json: {}",
                &upgrade_helper.config_file_name
            )
        }
        Err(e) => {
            println!(
                "failed to write config to {}: {}",
                &upgrade_helper.config_file_name, e
            );
        }
    }

    // Render the proposal description
    let description =
        proposal::render_proposal(&upgrade_helper).expect("failed to prepare proposal");

    // Write the proposal description to file
    utils::write_content_to_file(&description, &upgrade_helper.proposal_file_name)
        .expect("failed to write proposal to file");
}

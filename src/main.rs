mod block;
mod cli;
mod command;
mod constants;
mod errors;
mod helper;
mod http;
mod inputs;
mod mock_error;
mod network;
mod proposal;
mod release;
mod utils;
mod version;

use chrono::Utc;
use helper::UpgradeHelper;
// External imports
use clap::Parser; // NOTE: needs to be imported for Cli::parse() to work

// Crate imports
use crate::{
  block::get_estimated_height,
  cli::{SubCommand, CLI},
};

#[tokio::main]
async fn main() {
    match CLI::parse().subcmd {
        SubCommand::GenerateProposal => {
            if let Err(e) = cli::generate_proposal().await {
                println!("Error generating proposal: {}", e);
                process::exit(1);
            };
        }
        SubCommand::GenerateCommand(args) => {
            if let Err(e) = cli::generate_command(args).await {
                println!("Error generating command: {}", e);
                process::exit(1);
            }
        }
    }
}

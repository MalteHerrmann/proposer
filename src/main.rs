mod block;
mod cli;
mod command;
mod errors;
mod helper;
mod http;
mod inputs;
mod network;
mod proposal;
mod release;
mod utils;
mod version;

// External imports
use clap::Parser; // NOTE: needs to be imported for Cli::parse() to work

// Crate imports
use crate::cli::{SubCommand, CLI};

#[tokio::main]
async fn main() {
    match CLI::parse().subcmd {
        SubCommand::GenerateProposal => {
            cli::generate_proposal().await;
        }
        SubCommand::GenerateCommand(args) => {
            cli::generate_command(args)
                .await
                .expect("Error generating command");
        }
    }
}

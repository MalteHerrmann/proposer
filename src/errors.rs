use std::path::PathBuf;
use chrono::{DateTime, Utc};
use inquire::InquireError;
use thiserror::Error;
use crate::network::Network;

#[derive(Error, Debug)]
pub enum ProposerError {
    #[error("Failed to render proposal: {0}")]
    Render(#[from] handlebars::RenderError),
    #[error("Failed to write to file: {0}")]
    Write(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Home directory does not exist: {0}")]
    HomeDir(PathBuf),
    #[error("Invalid previous version: {0}")]
    PreviousVersion(String),
    #[error("Invalid target version for {0}: {1}")]
    TargetVersion(Network, String),
    #[error("Invalid upgrade time: {0}")]
    UpgradeTime(DateTime<Utc>),
}

#[derive(Error, Debug)]
pub enum HelperError {
    #[error("Failed to read from file: {0}")]
    Read(#[from] std::io::Error),
    #[error("Failed to parse JSON: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("Invalid helper configuration: {0}")]
    Validate(#[from] ValidationError),
}

#[derive(Error, Debug)]
pub enum InputError {
    #[error("Invalid network: {0}")]
    InvalidNetwork(String),
    #[error("Got IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("No configuration files found in current directory: {0}")]
    NoConfigFiles(PathBuf),
    #[error("Error during user input: {0}")]
    UserInput(#[from] InquireError),
    #[error("Failed to validate input: {0}")]
    Validate(#[from] ValidationError),
}
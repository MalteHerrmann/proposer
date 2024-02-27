use crate::network::Network;
use async_openai::error::OpenAIError;
use chrono::{DateTime, Utc};
use inquire::InquireError;
use std::path::PathBuf;
use std::string::FromUtf8Error;
use thiserror::Error;

/// High level error type than can occur when handling the block information
#[derive(Error, Debug)]
pub enum BlockError {
    #[error("Failed to get block information: {0}")]
    GetBlock(#[from] reqwest::Error),
    #[error("Failed to get parse date: {0}")]
    ParseDate(#[from] chrono::ParseError),
    #[error("Failed to parse int: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("Failed to parse time")]
    ParseTime,
    #[error("Failed to parse url: {0}")]
    ParseUrl(#[from] url::ParseError),
    #[error("Failed to build regex: {0}")]
    Regex(#[from] regex::Error),
    #[error("Failed to unpack block response: {0}")]
    UnpackResponse(#[from] serde_json::Error),
}

/// High level error type that can occur when generating the submission command
#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Failed to get client configuration: {0}")]
    ClientConfig(#[from] ConfigError),
    #[error("Failed to validate Commonwealth link: {0}")]
    Commonwealth(#[from] CommonwealthError),
    #[error("Failed to get helper: {0}")]
    GetHelper(#[from] HelperError),
    #[error("Failed to get user input: {0}")]
    Input(#[from] InputError),
    #[error("Failed to get key: {0}")]
    Key(#[from] KeysError),
    #[error("Failed to prepare command: {0}")]
    Prepare(#[from] PrepareError),
    #[error("Failed to render command: {0}")]
    Render(#[from] handlebars::RenderError),
    #[error("Failed to write to file: {0}")]
    Write(#[from] std::io::Error),
}

/// Error type for failures related to Commonwealth
#[derive(Error, Debug)]
pub enum CommonwealthError {
    #[error("Invalid commonwealth link")]
    InvalidCommonwealthLink,
    #[error("Failed to parse url: {0}")]
    ParseUrl(#[from] url::ParseError),
    #[error("Failed to get body: {0}")]
    GetBody(#[from] reqwest::Error),
}

/// Error type for failed parsing of the client configuration
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read from file: {0}")]
    Read(#[from] std::io::Error),
    #[error("Failed to parse toml: {0}")]
    Parse(#[from] toml::de::Error),
}

/// Error type for failed helper operations
#[derive(Error, Debug)]
pub enum HelperError {
    #[error("Failed to read from file: {0}")]
    Read(#[from] std::io::Error),
    #[error("Failed to parse JSON: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("Invalid helper configuration: {0}")]
    Validate(#[from] ValidationError),
}

/// Error type for failed user input
#[derive(Error, Debug)]
pub enum InputError {
    #[error("Error getting block information: {0}")]
    Block(#[from] BlockError),
    #[error("Failed to get GitHub data: {0}")]
    GitHub(#[from] octocrab::Error),
    #[error("Invalid network: {0}")]
    InvalidNetwork(String),
    #[error("Got IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("No configuration files found in current directory: {0}")]
    NoConfigFiles(PathBuf),
    #[error("Failed to create summary: {0}")]
    Summary(#[from] SummaryError),
    #[error("Error during user input: {0}")]
    UserInput(#[from] InquireError),
    #[error("Failed to validate input: {0}")]
    Validate(#[from] ValidationError),
}

/// Error type for failed operations regarding keys
#[derive(Error, Debug)]
pub enum KeysError {
    #[error("Failed to execute CLI command: {0}")]
    CLICommand(#[from] std::io::Error),
    #[error("Failed to get balance: {0}")]
    GetBalance(#[from] reqwest::Error),
    #[error("Failed to get keys from keyring: {0}")]
    GetKeys(#[from] InputError),
    #[error("No keys with balance found")]
    NoKeysWithBalance,
    #[error("Failed to convert bytes to string: {0}")]
    OutputConversion(#[from] FromUtf8Error),
    #[error("Failed to parse url: {0}")]
    ParseUrl(#[from] url::ParseError),
    #[error("Failed to unpack balance response: {0}")]
    UnpackResponse(#[from] serde_json::Error),
}

/// Error type for failed interactions with the LLM to generate the release notes summary
#[derive(Error, Debug)]
pub enum SummaryError {
    #[error("Failed to communicate with LLM: {0}")]
    LLM(#[from] OpenAIError),
    #[error("No summary generated")]
    NoSummary,
    #[error("Failed to get release notes: {0}")]
    ReleaseNotes(#[from] ReleaseError),
}

/// Error type for failed preparation of the proposal command
#[derive(Error, Debug)]
pub enum PrepareError {
    #[error("Failed to download checksums: {0}")]
    DownloadChecksums(#[from] reqwest::Error),
    #[error("checksum.txt not found in assets")]
    GetChecksumAsset,
    #[error("Failed to get helper: {0}")]
    GetHelper(#[from] HelperError),
    #[error("Failed to get release from GitHub: {0}")]
    GetRelease(#[from] octocrab::Error),
    #[error("Failed user input: {0}")]
    Input(#[from] InputError),
    #[error("Failed to get summary: {0}")]
    Summary(#[from] SummaryError),
    #[error("Failed to read proposal file: {0}")]
    ReadProposal(#[from] std::io::Error),
    #[error("Failed to render command: {0}")]
    RenderCommand(#[from] handlebars::RenderError),
    #[error("Failed to validate helper: {0}")]
    ValidateHelper(#[from] ValidationError),
}

/// High level error type that can occur while preparing the proposal contents
#[derive(Error, Debug)]
pub enum ProposalError {
    #[error("Failed to get user input: {0}")]
    Input(#[from] InputError),
    #[error("Failed to render proposal: {0}")]
    Render(#[from] handlebars::RenderError),
    #[error("Failed to validate helper: {0}")]
    Validate(#[from] ValidationError),
    #[error("Failed to write to file: {0}")]
    Write(#[from] std::io::Error),
}

/// High level error type that can occur when handling the release information
#[derive(Error, Debug)]
pub enum ReleaseError {
    #[error("No release notes found")]
    NoReleaseNotes,
}

/// Error type for failed validations
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Failed to validate client configuration: {0}")]
    ClientConfig(#[from] ConfigError),
    #[error("Home directory does not exist: {0}")]
    HomeDir(PathBuf),
    #[error("Invalid previous version: {0}")]
    PreviousVersion(String),
    #[error("Invalid target version for {0}: {1}")]
    TargetVersion(Network, String),
    #[error("Invalid upgrade time: {0}")]
    UpgradeTime(DateTime<Utc>),
}

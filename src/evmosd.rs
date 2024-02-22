use crate::errors::ConfigError;
use serde::Deserialize;
use std::path::Path;

/// The client configuration for the `evmosd` node.
#[derive(Clone, Deserialize)]
pub struct ClientConfig {
    #[serde(rename = "chain-id")]
    pub chain_id: String,
    #[serde(rename = "keyring-backend")]
    pub keyring_backend: String,
    pub output: String,
    pub node: String,
    #[serde(rename = "broadcast-mode")]
    pub broadcast_mode: String,
}

/// This method returns the client configuration for the `evmosd` node.
pub fn get_client_config(path: &Path) -> Result<ClientConfig, ConfigError> {
    Ok(toml::from_str::<ClientConfig>(
        std::fs::read_to_string(path)?.as_str(),
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_dir::{DirBuilder, FileType::EmptyFile, TestDir};

    #[test]
    fn test_get_client_config() {
        let test_dir = TestDir::temp().create("config/client.toml", EmptyFile);

        let config_contents = r#"
            chain-id = "evmos_9000-1"
            keyring-backend = "os"
            output = "text"
            node = "tcp://localhost:26657"
            broadcast-mode = "sync"
        "#;

        let client_config_path = test_dir.path("config/client.toml");
        std::fs::write(&client_config_path, config_contents)
            .expect("failed to write config to file in test setup");

        let config = get_client_config(&client_config_path).unwrap();

        assert_eq!(config.chain_id, "evmos_9000-1");
        assert_eq!(config.keyring_backend, "os");
        assert_eq!(config.output, "text");
        assert_eq!(config.node, "tcp://localhost:26657");
        assert_eq!(config.broadcast_mode, "sync");
    }
}

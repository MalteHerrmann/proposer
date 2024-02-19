use crate::{balance, block::get_rest_provider, errors::KeysError, network::Network};
use std::{process};
use serde::{Deserialize, Serialize};

/// Represents a key with its name and address.
#[derive(Debug, Deserialize, Serialize)]
pub struct Key {
    pub name: String,
    pub address: String,
}

/// Returns a list of keys that have a non-zero balance on the configured network.
pub async fn get_keys_with_balances(network: &Network) -> Result<Vec<String>, KeysError> {
    let keys = get_keys_from_keyring()?;
    filter_keys_with_balance(keys, network).await
}

/// Returns a list of keys from the configured keyring.
/// The keyring is configured in the user's home directory.
fn get_keys_from_keyring() -> Result<Vec<Key>, KeysError> {
    let output = process::Command::new("evmosd")
        .args(&["keys", "list", "--output", "json"])
        .output()?;

    let keys_out = String::from_utf8(output.stdout)?;
    Ok(parse_keys_output(&keys_out)?)
}

/// Parses the output from the `keys list` command.
/// Returns a list of keys.
fn parse_keys_output(output: &str) -> Result<Vec<Key>, serde_json::Error> {
    let keys: Vec<Key> = serde_json::from_str(output)?;
    Ok(keys)
}

/// Filters the keys for ones that have a non-zero balance on the configured network.
async fn filter_keys_with_balance(
    keys: Vec<Key>,
    network: &Network,
) -> Result<Vec<String>, KeysError> {
    let mut keys_with_balance = Vec::new();
    let base_url = get_rest_provider(*network);

    for key in keys {
        if balance::has_balance(&key.address, network, &base_url).await? {
            keys_with_balance.push(key.name);
        }
    }

    if keys_with_balance.is_empty() {
        return Err(KeysError::NoKeysWithBalance);
    }

    Ok(keys_with_balance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_keys_with_balances() {
        let network = Network::Testnet;
        let res = get_keys_with_balances(&network).await;
        assert!(res.is_ok());

        let keys = res.unwrap();
        assert!(keys.len() >= 1);
    }

    #[test]
    fn test_parse_keys_output() {
        let output = r#"[{"name":"newkey","type":"local","address":"evmos12ly0g0dj6amk5uch77mz7d022h3sd10enf4ln9","pubkey":"{\"@type\":\"/ethermint.crypto.v1.ethsecp256k1.PubKey\",\"key\":\"ArSwAFlw2JBRr4xGii2TjTU15gOWkAO0YEfhNZvWhWqQ\"}"},{"name":"testnet-address","type":"local","address":"evmos1k0sx0f62383ufue5gn6xth029wut0twut294fw","pubkey":"{\"@type\":\"/ethermint.crypto.v1.ethsecp256k1.PubKey\",\"key\":\"Aw2SsAa2V1dgLhdTZuztA++8kVaCxJX1g+WP9F+QzEW5\"}"}]"#;

        let res = parse_keys_output(output);
        assert!(res.is_ok());

        let keys = res.unwrap();
        assert_eq!(keys.len(), 2, "expected two keys");
        assert_eq!(keys[0].address, "evmos12ly0g0dj6amk5uch77mz7d022h3sd10enf4ln9");
        assert_eq!(keys[1].address, "evmos1k0sx0f62383ufue5gn6xth029wut0twut294fw");
    }
}


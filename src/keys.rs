use crate::{balance, block::get_rest_provider, errors::KeysError, network::Network};
use std::{path::Path, process};

/// Returns a list of keys that have a non-zero balance on the configured network.
pub async fn get_keys_with_balances(network: &Network) -> Result<Vec<String>, KeysError> {
    let keys = get_keys_from_keyring()?;
    let home_dir = dirs::home_dir().ok_or(KeysError::HomeDir)?;

    filter_keys_with_balance(keys, network, home_dir.as_path()).await
}

/// Returns a list of keys from the configured keyring.
/// The keyring is configured in the user's home directory.
fn get_keys_from_keyring() -> Result<Vec<String>, KeysError> {
    let output = process::Command::new("evmosd")
        .args(&["keys", "list"])
        .output()?;

    let keys_out = String::from_utf8(output.stdout)?;
    Ok(parse_keys_output(&keys_out))
}

/// Parses the output from the `keys list` command.
/// Returns a list of keys.
fn parse_keys_output(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| get_key_from_line(line))
        .collect::<Vec<String>>()
}

/// Returns the key from the given line.
///
/// The line is expected to be in the format:
/// - address: evmos1...
fn get_key_from_line(line: &str) -> Option<String> {
    if !line.contains("address:") {
        return None;
    }

    let split_line = line.split_whitespace().collect::<Vec<&str>>();
    if split_line.len() < 3 {
        return None;
    }

    Some(split_line[2].to_string())
}

/// Filters the keys for ones that have a non-zero balance on the configured network.
async fn filter_keys_with_balance(
    keys: Vec<String>,
    network: &Network,
    home_dir: &Path,
) -> Result<Vec<String>, KeysError> {
    let mut keys_with_balance = Vec::new();
    let base_url = get_rest_provider(*network);

    for key in keys {
        let address = get_address_from_key(&key, home_dir)?;
        if balance::has_balance(&address, network, &base_url).await? {
            keys_with_balance.push(key);
        }
    }

    if keys_with_balance.is_empty() {
        return Err(KeysError::NoKeysWithBalance);
    }

    Ok(keys_with_balance)
}

/// Returns the address from the given key.
fn get_address_from_key(key: &str, home: &Path) -> Result<String, KeysError> {
    let home_dir = home.to_str().unwrap();

    let output = process::Command::new("evmosd")
        .args(&["keys", "show", key, "-a", "--home", home_dir])
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    } else {
        Err(KeysError::AddressFromKey(key.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_keys_with_balances() {
        let network = Network::Testnet;
        let keys = get_keys_with_balances(&network).await.unwrap();
        assert!(keys.len() >= 1);
    }

    #[test]
    fn test_parse_keys_output() {
        let lines = r#"
            - address: evmos12ly0g0dj6amk5uch77mz7d022h3sc00enf5ln8
              name: newkey
              pubkey: '{"@type":"/ethermint.crypto.v1.ethsecp256k1.PubKey","key":"ArSwAFlw2JBRr4xGii2TjTU15gOWkAO0ZEfhMZwWhWqQ"}'
              type: local
            - address: evmos1k0sx0f62383ufue5gn6xth029wvz9twut294fw
              name: testnet-address
              pubkey: '{"@type":"/ethermint.crypto.v1.ethsecp256k1.PubKey","key":"Aw2SsAa1VOdgLhdFZuztA++8kVaCxJX1g+WP9F+QzEW5"}'
              type: local
            "#;

        let keys = parse_keys_output(lines);
        assert_eq!(keys.len(), 2, "expected two keys");
        assert_eq!(keys[0], "evmos12ly0g0dj6amk5uch77mz7d022h3sc00enf5ln8");
        assert_eq!(keys[1], "evmos1k0sx0f62383ufue5gn6xth029wvz9twut294fw");
    }

    #[test]
    fn test_get_address_from_key_error() {
        let key = "invalid-key";
        let home_dir = dirs::home_dir().expect("failed to get home directory");

        let res = get_address_from_key(key, &home_dir);
        assert!(res.is_err());
    }
}


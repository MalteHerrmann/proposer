use crate::block::get_rest_provider;
use crate::errors::KeysError;
use crate::network::Network;
use std::{path::Path, process};
use crate::balance;

/// Returns a list of keys that have a non-zero balance on the configured network.
pub async fn get_keys_with_balances(network: &Network) -> Result<Vec<String>, KeysError> {
    let keys = get_keys_from_keyring()?;
    let home_dir = dirs::home_dir()
        .ok_or(KeysError::HomeDir)?;

    filter_keys_with_balance(keys, network, home_dir.as_path()).await
}

/// Returns a list of keys from the configured keyring.
/// The keyring is configured in the user's home directory.
fn get_keys_from_keyring() -> Result<Vec<String>, KeysError> {
    let output = process::Command::new("evmosd")
        .args(&["keys", "list"])
        .output()?;

    let keys = String::from_utf8(output.stdout)?;

    Ok(keys
        .lines()
        .map(|line| line.split_whitespace().next().unwrap().to_string())
        .collect())
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

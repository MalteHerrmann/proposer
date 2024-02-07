use crate::errors::PrepareError;
use crate::http::get;
use octocrab::{
    models::repos::{Asset, Release},
    Result,
};
use serde_json::Value;
use std::collections::HashMap;

/// Sends a HTTP request to the GitHub release page and returns the response.
pub async fn get_release(version: &str) -> Result<Release> {
    let octocrab = octocrab::instance();

    octocrab
        .repos("evmos", "evmos")
        .releases()
        .get_by_tag(version)
        .await
}

/// Checks if the release for the target version already exists by
/// sending a HTTP request to the GitHub release page.
pub async fn check_release_exists(version: &str) -> Result<Release> {
    get_release(version).await
}

/// Returns the asset string for the release assets.
/// The asset string is used in the Evmos CLI command.
pub async fn get_asset_string(release: &Release) -> Result<String, PrepareError> {
    let checksums = get_checksum_map(release.assets.clone()).await?;

    Ok(build_assets_json(release, checksums).to_string())
}

/// Builds the assets JSON object.
fn build_assets_json(release: &Release, checksums: HashMap<String, String>) -> Value {
    let mut assets = serde_json::json!({
        "binaries": {}
    });

    for asset in release.assets.clone() {
        let os_key = match get_os_key_from_asset_name(&asset.name) {
            Some(key) => key,
            None => {
                continue;
            }
        };

        let checksum = match checksums.get(&asset.name) {
            Some(checksum) => checksum,
            None => {
                continue;
            }
        };

        let url = format!("{}?checksum={}", asset.browser_download_url, checksum);

        insert_into_assets(&mut assets, os_key, url);
    }

    assets
}

/// Inserts a new key value pair into the assets binaries.
/// The key is the OS key and the value is the download URL.
fn insert_into_assets(assets: &mut Value, key: String, url: String) {
    let binaries = assets["binaries"].as_object_mut().unwrap();
    binaries.insert(key, serde_json::json!(url));
}

/// Returns the checksum from the release assets.
fn get_checksum_from_assets(assets: &[Asset]) -> Option<&Asset> {
    assets.iter().find(|asset| asset.name == "checksums.txt")
}

/// Returns the OS key from the asset name.
fn get_os_key_from_asset_name(name: &str) -> Option<String> {
    // Check for regex (Linux|Darwin)_(amd64|arm64).tar.gz and store os and arch in variables
    return match regex::Regex::new(r"(Linux|Darwin)_(amd64|arm64)") {
        Ok(re) => {
            let captures = re.captures(name)?;
            let os = captures.get(1)?.as_str().to_ascii_lowercase();
            let arch = captures.get(2)?.as_str();

            Some(format!("{os}/{arch}"))
        }
        Err(_) => {
            println!("no key found for asset: {}", name);
            None
        }
    };
}

/// Downloads the checksum file from the release assets and returns the built checksum string.
async fn get_checksum_map(assets: Vec<Asset>) -> Result<HashMap<String, String>, PrepareError> {
    let checksum = match get_checksum_from_assets(assets.as_slice()) {
        Some(checksum) => checksum,
        None => return Err(PrepareError::GetChecksumAsset),
    };
    let response = get(checksum.browser_download_url.clone()).await?;
    let body = response.text().await?;

    let mut checksums = HashMap::new();

    for line in body.lines() {
        let line = line.trim();
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() != 2 {
            println!("Invalid checksum line: {}", line);
            continue;
        }

        // NOTE: Windows links are not supported in the submit-legacy-proposal command
        if parts[1].contains("Windows") {
            continue;
        }

        checksums.insert(parts[1].to_string(), parts[0].to_string());
    }

    Ok(checksums)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_get_release_pass() {
        let release = get_release("v14.0.0").await.unwrap();
        assert_eq!(release.tag_name, "v14.0.0");
    }

    #[tokio::test]
    async fn test_get_release_fail() {
        let res = get_release("invalidj.xjaf/ie").await;
        assert_eq!(res.is_err(), true);
    }

    #[tokio::test]
    async fn test_check_release_exists_pass() {
        assert!(check_release_exists("v14.0.0").await.is_ok());
    }

    #[tokio::test]
    async fn test_check_release_exists_fail() {
        assert!(check_release_exists("v14.0.8").await.is_err());
    }

    #[tokio::test]
    async fn test_get_checksum_map_pass() {
        let release = get_release("v14.0.0").await.unwrap();
        let checksums = get_checksum_map(release.assets.clone()).await.unwrap();

        assert!(checksums.contains_key("evmos_14.0.0_Linux_amd64.tar.gz"));
        assert!(checksums.contains_key("evmos_14.0.0_Linux_arm64.tar.gz"));
        assert!(checksums.contains_key("evmos_14.0.0_Darwin_amd64.tar.gz"));
        assert!(checksums.contains_key("evmos_14.0.0_Darwin_arm64.tar.gz"));
    }

    #[tokio::test]
    async fn test_get_asset_string_pass() {
        let release = get_release("v15.0.0").await.expect("Failed to get release");

        let assets = get_asset_string(&release)
            .await
            .expect("Failed to get asset string");

        let expected_assets = json!({
            "binaries": {
                "darwin/arm64" :"https://github.com/evmos/evmos/releases/download/v15.0.0/evmos_15.0.0_Darwin_arm64.tar.gz?checksum=3855eaec2fc69eafe8cff188b8ca832c2eb7d20ca3cb0f55558143a68cdc600f",
                "darwin/amd64":"https://github.com/evmos/evmos/releases/download/v15.0.0/evmos_15.0.0_Darwin_amd64.tar.gz?checksum=ba454bb8acf5c2cf09a431b0cd3ef77dfc303dc57c14518b38fb3b7b8447797a",
                "linux/arm64":"https://github.com/evmos/evmos/releases/download/v15.0.0/evmos_15.0.0_Linux_arm64.tar.gz?checksum=aae9513f9cc5ff96d799450aaa39a84bea665b7369e7170dd62bb56130dd4a21",
                "linux/amd64":"https://github.com/evmos/evmos/releases/download/v15.0.0/evmos_15.0.0_Linux_amd64.tar.gz?checksum=9f7af7f923ff4c60c11232ba060bef4dfff807282d0470a070c87c6de937a611",
            }
        });

        let expected_assets_string = expected_assets.to_string();
        assert_eq!(assets, expected_assets_string, "expected different assets");
    }

    #[test]
    fn test_get_os_key_from_asset_name_pass() {
        let name = "evmos_14.0.0_Linux_amd64.tar.gz";
        let key = get_os_key_from_asset_name(name).unwrap();
        assert_eq!(key, "linux/amd64");
    }

    #[test]
    fn test_get_os_key_from_asset_name_fail() {
        let name = "evmos_14.0.amd64.tar";
        assert!(get_os_key_from_asset_name(name).is_none());
    }
}

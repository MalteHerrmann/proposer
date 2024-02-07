use crate::errors::PrepareError;
use crate::http::get_body;
use octocrab::{
    models::repos::{Asset, Release},
    Octocrab, Result,
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Sends a HTTP request to the GitHub release page and returns the response.
pub async fn get_release(instance: &Octocrab, version: &str) -> Result<Release> {
    instance
        .repos("evmos", "evmos")
        .releases()
        .get_by_tag(version)
        .await
}

#[cfg(test)]
mod release_tests {
    use super::*;
    use crate::mock_error::setup_error_handler;
    use serde::{Deserialize, Serialize};
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    #[derive(Serialize, Deserialize)]
    struct FakeRelease(Release);

    /// Sets up a mock server to return the given response template
    /// when receiving a GET request on the release URL.
    /// Returns the mock server.
    ///
    /// This is used to mock the GitHub API without having to actually run queries to it.
    async fn setup_api(template: ResponseTemplate) -> MockServer {
        const RELEASE_URL: &str = "/repos/evmos/evmos/releases/tags/v14.0.0";

        // Create a mock server
        let mock_server = MockServer::start().await;

        // Set up the mock server to return the fake response when receiving
        // a GET request on the release URL
        Mock::given(method("GET"))
            .and(path(RELEASE_URL))
            .respond_with(template)
            .mount(&mock_server)
            .await;

        // Set up the error handling for failed get requests
        setup_error_handler(
            &mock_server,
            &format!("GET on {} not received", RELEASE_URL),
        )
        .await;

        // Return the mock server
        mock_server
    }

    /// Sets up an Octocrab instance with the mock server URI.
    fn setup_octocrab(uri: &str) -> Octocrab {
        Octocrab::builder().base_uri(uri).unwrap().build().unwrap()
    }

    #[tokio::test]
    async fn test_get_release_pass() {
        let release_response: Release =
            serde_json::from_str(include_str!("testdata/release.json")).unwrap();

        let page_response = FakeRelease(release_response);

        let template = ResponseTemplate::new(200).set_body_json(&page_response);
        let mock_server = setup_api(template).await;

        let client = setup_octocrab(&mock_server.uri());
        let release = get_release(&client, "v14.0.0").await.unwrap();
        assert_eq!(release.tag_name, "v14.0.0");
    }

    #[tokio::test]
    async fn test_get_release_fail() {
        let template = ResponseTemplate::new(404);
        let mock_server = setup_api(template).await;
        let client = setup_octocrab(&mock_server.uri());

        let res = get_release(&client, "invalidj.xjaf/ie").await;
        assert_eq!(res.is_err(), true);
    }
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
    let body = get_body(checksum.browser_download_url.clone()).await?;

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

/// Returns an Octocrab instance.
pub fn get_instance() -> Arc<Octocrab> {
    octocrab::instance()
}

#[cfg(test)]
mod assets_tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_get_checksum_map_pass() {
        let release: Release = serde_json::from_str(include_str!("testdata/release.json")).unwrap();

        let checksums = get_checksum_map(release.assets.clone()).await.unwrap();

        assert!(checksums.contains_key("evmos_14.0.0_Linux_amd64.tar.gz"));
        assert!(checksums.contains_key("evmos_14.0.0_Linux_arm64.tar.gz"));
        assert!(checksums.contains_key("evmos_14.0.0_Darwin_amd64.tar.gz"));
        assert!(checksums.contains_key("evmos_14.0.0_Darwin_arm64.tar.gz"));
    }

    #[tokio::test]
    async fn test_get_asset_string_pass() {
        let release: Release = serde_json::from_str(include_str!("testdata/release.json")).unwrap();

        let assets = get_asset_string(&release)
            .await
            .expect("Failed to get asset string");

        let expected_assets = json!({
            "binaries":{
                "darwin/amd64":"https://github.com/evmos/evmos/releases/download/v14.0.0/evmos_14.0.0_Darwin_amd64.tar.gz?checksum=35202b28c856d289778010a90fdd6c49c49a451a8d7f60a13b0612d0cd70e178",
                "darwin/arm64":"https://github.com/evmos/evmos/releases/download/v14.0.0/evmos_14.0.0_Darwin_arm64.tar.gz?checksum=541d4bac1513c84278c8d6b39c86aca109cc1ecc17652df56e57488ffbafd2d5",
                "linux/amd64":"https://github.com/evmos/evmos/releases/download/v14.0.0/evmos_14.0.0_Linux_amd64.tar.gz?checksum=427c2c4a37f3e8cf6833388240fcda152a5372d4c5132ca2e3861a7085d35cd0",
                "linux/arm64":"https://github.com/evmos/evmos/releases/download/v14.0.0/evmos_14.0.0_Linux_arm64.tar.gz?checksum=a84279d66b6b0ecd87b85243529d88598995eeb124bc16bb8190a7bf022825fb",
            }
        });

        assert_eq!(
            assets,
            expected_assets.to_string(),
            "expected different assets"
        );
    }

    #[tokio::test]
    async fn test_get_asset_string_fail() {
        let release: Release =
            serde_json::from_str(include_str!("testdata/release_no_assets.json")).unwrap();

        assert!(get_asset_string(&release).await.is_err());
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

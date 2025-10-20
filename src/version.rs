use crate::config::NetworkConfig;
use regex::Regex;

/// Returns a boolean value if the defined version fulfills the semantic
/// versioning requirements.
pub fn is_valid_version(version: &str) -> bool {
    Regex::new(r"^v\d+\.\d+\.\d+(-rc\d+)*$")
        .unwrap()
        .is_match(version)
}

/// Returns a boolean value if the defined target version fits
/// the requirements for the selected network type.
/// The target version must be in the format `vX.Y.Z`.
/// Testnet upgrades must use a release candidate with the suffix `-rcX`.
pub fn is_valid_version_for_network(cfg: &NetworkConfig, target_version: &str) -> bool {
    let mut pattern = r"^v\d+\.\d{1}\.\d+".to_string();
    if cfg.allow_rc {
        pattern.push_str(r"(-rc\d+)*");
    }
    pattern.push('$');

    Regex::new(&pattern)
        .expect("invalid regex")
        .is_match(target_version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_version_pass() {
        assert!(is_valid_version("v14.0.0"));
        assert!(is_valid_version("v14.0.0-rc1"));
    }

    #[test]
    fn test_is_valid_version_fail() {
        assert!(!is_valid_version("v14.0."));
        assert!(!is_valid_version("v.0.1"));
    }

    #[test]
    fn test_is_valid_target_version_local_node_pass() {
        let cfg = NetworkConfig::default();
        assert!(is_valid_version_for_network(&cfg, "v14.0.0",));
    }

    #[test]
    fn test_is_valid_target_version_local_node_fail() {
        assert!(
            !is_valid_version_for_network(&NetworkConfig::default(), "v14.0",)
        );
    }

    #[test]
    fn test_is_valid_target_version_testnet_pass() {
        let cfg = NetworkConfig{
            allow_rc: true,
            ..NetworkConfig::default()
        };
        assert!(is_valid_version_for_network(&cfg, "v14.0.0-rc1",));
    }

    #[test]
    fn test_is_valid_target_version_testnet_fail() {
        assert!(
            !is_valid_version_for_network(&NetworkConfig::default(), "v14.00",)
        );
    }

    #[test]
    fn test_is_valid_target_version_mainnet_pass() {
        assert!(
            is_valid_version_for_network(&NetworkConfig::default(), "v14.0.0",)
        );
    }

    #[test]
    fn test_is_valid_target_version_mainnet_fail() {
        assert!(
            !is_valid_version_for_network(&NetworkConfig::default(), "v14.0.0-rc1",)
        );
    }
}

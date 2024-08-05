use crate::network::Network;
use regex::Regex;

/// Returns a boolean value if the defined version fulfills the semantic
/// versioning requirements.
pub fn is_valid_version(version: &str) -> bool {
    let valid = r"^v\d+\.\d+\.\d+(-rc\d+)*$";

    Regex::new(valid).unwrap().is_match(version)
}

/// Returns a boolean value if the defined target version fits
/// the requirements for the selected network type.
/// The target version must be in the format `vX.Y.Z`.
/// Testnet upgrades must use a release candidate with the suffix `-rcX`.
pub fn is_valid_version_for_network(network: Network, target_version: &str) -> bool {
    let re = match network {
        Network::LocalNode => Regex::new(r"^v\d+\.\d{1}\.\d+(-rc\d+)*$").unwrap(),
        Network::Testnet => Regex::new(r"^v\d+\.\d{1}\.\d+-rc\d+$").unwrap(),
        Network::Mainnet => Regex::new(r"^v\d+\.\d{1}\.\d+$").unwrap(),
    };

    re.is_match(target_version)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network;

    #[test]
    fn test_is_valid_version_pass() {
        assert_eq!(is_valid_version("v14.0.0"), true);
        assert_eq!(is_valid_version("v14.0.0-rc1"), true);
    }

    #[test]
    fn test_is_valid_version_fail() {
        assert_eq!(is_valid_version("v14.0."), false);
        assert_eq!(is_valid_version("v.0.1"), false);
    }

    #[test]
    fn test_is_valid_target_version_local_node_pass() {
        assert_eq!(
            is_valid_version_for_network(network::Network::LocalNode, "v14.0.0",),
            true
        );
    }

    #[test]
    fn test_is_valid_target_version_local_node_fail() {
        assert_eq!(
            is_valid_version_for_network(network::Network::LocalNode, "v14.0",),
            false
        );
    }

    #[test]
    fn test_is_valid_target_version_testnet_pass() {
        assert_eq!(
            is_valid_version_for_network(network::Network::Testnet, "v14.0.0-rc1",),
            true
        );
    }

    #[test]
    fn test_is_valid_target_version_testnet_fail() {
        assert_eq!(
            is_valid_version_for_network(network::Network::Testnet, "v14.0.0",),
            false
        );
    }

    #[test]
    fn test_is_valid_target_version_mainnet_pass() {
        assert_eq!(
            is_valid_version_for_network(network::Network::Mainnet, "v14.0.0",),
            true
        );
    }

    #[test]
    fn test_is_valid_target_version_mainnet_fail() {
        assert_eq!(
            is_valid_version_for_network(network::Network::Mainnet, "v14.0.0-rc1",),
            false
        );
    }
}

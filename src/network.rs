use std::fmt;

// Enum to represent different network options
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Network {
    LocalNode,
    Testnet,
    Mainnet,
}

impl fmt::Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Network::LocalNode => write!(f, "Local Node"),
            Network::Testnet => write!(f, "Testnet"),
            Network::Mainnet => write!(f, "Mainnet"),
        }
    }
}

/// Returns the native denomination for the given network.
pub fn get_denom(network: Network) -> String {
    match network {
        Network::LocalNode => "aevmos".to_string(),
        Network::Testnet => "atevmos".to_string(),
        Network::Mainnet => "aevmos".to_string(),
    }
}

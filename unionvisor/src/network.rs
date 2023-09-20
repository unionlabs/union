use std::{fmt::Display, str::FromStr};

use thiserror::Error;

#[derive(Debug, Clone, Default, Copy)]
pub enum Network {
    // Testnet1,
    // Testnet2,
    #[default]
    Testnet3,
    // DevnetMinimal,
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Network::Union1 => write!(f, "union-1"),
            Network::Testnet1 => write!(f, "union-testnet-1"),
            Network::Testnet2 => write!(f, "union-testnet-2"),
            Network::DevnetMinimal => write!(f, "union-minimal-1"),
        }
    }
}

#[derive(Debug, Error)]
#[error("unknown network {0}")]
struct UnknownNetworkError(String);

impl FromStr for Network {
    type Err = UnknownNetworkError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // "union-testnet-1" => Ok(Network::Testnet1),
            // "union-testnet-2" => Ok(Network::Testnet2),
            "union-testnet-3" => Ok(Network::Testnet3),
            // "union-minimal-1" => Ok(Network::DevnetMinimal),
            s => Err(UnknownNetworkError(s.to_owned())),
        }
    }
}

impl Network {
    // `self` will be used later on to determine which URL to use based on the network
    #[allow(clippy::unused_self)]
    pub fn seeds(&self) -> &str {
        ""
    }

    // `self` will be used later on to determine which URL to use based on the network
    #[allow(clippy::unused_self)]
    pub fn genesis_url(&self) -> &str {
        // We use a secret gist for the genesis.json until testnet is public
        // "https://raw.githubusercontent.com/unionlabs/genesis/main/union-testnet-1/genesis.json"
        match self {
            // Network::Testnet1 => todo!(),
            // Network::Testnet2 => todo!(),
            Network::Testnet3 => "https://gist.githubusercontent.com/cor/5cab203f0f41549a2e04e48fc10bf889/raw/0bb5a9d3c02f38b6a34f85314dff04100f3605bc/union-testnet-3-genesis.json",
            // _ => 
            // Network::DevnetMinimal => todo!(),
        }
    }
}

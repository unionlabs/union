use std::{fmt::Display, str::FromStr};

use color_eyre::eyre::bail;

#[derive(Debug, Clone, Default, Copy)]
pub enum Network {
    Union1,
    Testnet1,
    #[default]
    Testnet2,
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Network::Union1 => write!(f, "union-1"),
            Network::Testnet1 => write!(f, "union-testnet-1"),
            Network::Testnet2 => write!(f, "union-testnet-2"),
        }
    }
}

impl FromStr for Network {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "union-1" => Ok(Network::Union1),
            "union-testnet-1" => Ok(Network::Testnet1),
            "union-testnet-2" => Ok(Network::Testnet2),
            _ => bail!("unknown network"),
        }
    }
}

impl Network {
    // `self` will be used later on to determine which URL to use based on the network
    #[allow(clippy::unused_self)]
    pub fn seeds(&self) -> &str {
        "6a2d3a6f846792b99c4cfa3ccb40f80073bac30d@union-testnet.cor.systems:26656,7139b86ff37752437cf01a4970bf2b26c45c53e0@uniontestnet.poisonphang.com:26656"
    }

    // `self` will be used later on to determine which URL to use based on the network
    #[allow(clippy::unused_self)]
    pub fn genesis_url(&self) -> &str {
        // We use a secret gist for the genesis.json until testnet is public
        // "https://raw.githubusercontent.com/unionfi/genesis/main/union-testnet-1/genesis.json"
        "https://gist.githubusercontent.com/cor/ffb8ec1a35a28fc1b3aad89f3fb466d4/raw/434b9f88a0ee6099cbbabafcd144f958a03ce655/union-testnet-2-gentx.json"
    }
}

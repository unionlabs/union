use std::{fmt::Display, str::FromStr};

use color_eyre::eyre::bail;

#[derive(Debug, Clone, Default, Copy)]
pub enum Network {
    Union1,
    #[default]
    Testnet1,
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Network::Union1 => write!(f, "union-1"),
            Network::Testnet1 => write!(f, "union-testnet-1"),
        }
    }
}

impl FromStr for Network {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "union-1" => Ok(Network::Union1),
            "union-testnet-1" => Ok(Network::Testnet1),
            _ => bail!("unknown network"),
        }
    }
}

impl Network {
    pub fn seeds(&self) -> &str {
        "c649931f0ef98bc3e086bbfbcf3b04896a9ec7de@uniontestnet.poisonphang.com:26656"
    }

    pub fn genesis_url(&self) -> &str {
        // We use a secret gist for the genesis.json until testnet is public
        // "https://raw.githubusercontent.com/unionfi/genesis/main/union-testnet-1/genesis.json"
        "https://gist.githubusercontent.com/cor/53e083cc949ee4c63eb50c123a39bee8/raw/ec36b47ae60658307bfec383a477300213c933d3/genesis.json"
    }
}

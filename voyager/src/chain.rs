use chain_utils::{
    cosmos::{Cosmos, CosmosInitError},
    ethereum::{Ethereum, EthereumInitError},
    scroll::{Scroll, ScrollInitError},
    union::{Union, UnionInitError},
};
use unionlabs::ethereum::config::{Mainnet, Minimal, PresetBaseKind};

use crate::config::ChainConfigType;

pub enum AnyChain {
    Union(Union),
    Cosmos(Cosmos),
    EthereumMainnet(Ethereum<Mainnet>),
    EthereumMinimal(Ethereum<Minimal>),
    Scroll(Scroll),
}

#[derive(Debug, thiserror::Error)]
pub enum AnyChainTryFromConfigError {
    #[error("error initializing a union chain")]
    Union(#[from] UnionInitError),
    #[error("error initializing a cosmos chain")]
    Cosmos(#[from] CosmosInitError),
    #[error("error initializing an ethereum chain")]
    Ethereum(#[from] EthereumInitError),
    #[error("error initializing a scroll chain")]
    Scroll(#[from] ScrollInitError),
}

impl AnyChain {
    pub async fn try_from_config(
        config: ChainConfigType,
    ) -> Result<Self, AnyChainTryFromConfigError> {
        Ok(match config {
            ChainConfigType::Union(union) => Self::Union(Union::new(union).await?),
            ChainConfigType::Cosmos(cosmos) => Self::Cosmos(Cosmos::new(cosmos).await?),
            ChainConfigType::Ethereum(ethereum) => {
                let config = chain_utils::ethereum::Config {
                    ibc_handler_address: ethereum.ibc_handler_address,
                    signers: ethereum.signers,
                    eth_rpc_api: ethereum.eth_rpc_api,
                    eth_beacon_rpc_api: ethereum.eth_beacon_rpc_api,
                };
                match ethereum.preset_base {
                    PresetBaseKind::Minimal => {
                        Self::EthereumMinimal(Ethereum::<Minimal>::new(config).await?)
                    }
                    PresetBaseKind::Mainnet => {
                        Self::EthereumMainnet(Ethereum::<Mainnet>::new(config).await?)
                    }
                }
            }
            ChainConfigType::Scroll(scroll) => Self::Scroll(Scroll::new(scroll).await?),
        })
    }
}

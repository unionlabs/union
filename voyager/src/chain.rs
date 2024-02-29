use chain_utils::{
    cosmos::{Cosmos, CosmosInitError},
    evm::{Evm, EvmInitError},
    scroll::{Scroll, ScrollInitError},
    union::{Union, UnionInitError},
};
use unionlabs::ethereum::config::{Mainnet, Minimal, PresetBaseKind};

use crate::config::ChainConfigType;

pub enum AnyChain {
    Union(Union),
    Cosmos(Cosmos),
    EvmMainnet(Evm<Mainnet>),
    EvmMinimal(Evm<Minimal>),
    Scroll(Scroll),
}

#[derive(Debug, thiserror::Error)]
pub enum AnyChainTryFromConfigError {
    #[error("error initializing a union chain")]
    Union(#[from] UnionInitError),
    #[error("error initializing a cosmos chain")]
    Cosmos(#[from] CosmosInitError),
    #[error("error initializing an ethereum chain")]
    Evm(#[from] EvmInitError),
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
            ChainConfigType::Evm(evm) => {
                let config = chain_utils::evm::Config {
                    ibc_handler_address: evm.ibc_handler_address,
                    signers: evm.signers,
                    eth_rpc_api: evm.eth_rpc_api,
                    eth_beacon_rpc_api: evm.eth_beacon_rpc_api,
                };
                match evm.preset_base {
                    PresetBaseKind::Minimal => Self::EvmMinimal(Evm::<Minimal>::new(config).await?),
                    PresetBaseKind::Mainnet => Self::EvmMainnet(Evm::<Mainnet>::new(config).await?),
                }
            }
            ChainConfigType::Scroll(scroll) => Self::Scroll(Scroll::new(scroll).await?),
        })
    }
}

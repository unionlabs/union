#![feature(trait_alias)]
// #![warn(clippy::pedantic)]

use std::collections::HashMap;

use enumorph::Enumorph;
use serde::{Deserialize, Serialize};
use tracing::warn;
use unionlabs::{
    ethereum::config::{ChainSpec, Mainnet, Minimal, PresetBaseKind},
    hash::H160,
    traits::{Chain, ChainIdOf, FromStrExact},
    uint::U256,
    ClientType, WasmClientType,
};

use crate::{
    arbitrum::{Arbitrum, ArbitrumInitError},
    berachain::{Berachain, BerachainInitError},
    cosmos::{Cosmos, CosmosInitError},
    ethereum::Ethereum,
    keyring::KeyringConfig,
    scroll::{Scroll, ScrollInitError},
    union::{Union, UnionInitError},
    wasm::Wasm,
};

pub mod arbitrum;
pub mod berachain;
pub mod cosmos;
pub mod ethereum;
pub mod scroll;
pub mod union;

pub mod cosmos_sdk;

pub mod wasm;

pub mod private_key;

pub mod keyring;

pub trait GetChain<C: Chain> {
    fn get_chain(&self, chain_id: &ChainIdOf<C>) -> Option<C>;

    fn with_chain<'a, T, F>(
        &'a self,
        chain_id: &'a ChainIdOf<C>,
        f: F,
    ) -> Result<T, ChainNotFoundError<C>>
    where
        T: 'static,
        F: FnOnce(C) -> T + 'a,
    {
        match self.get_chain(chain_id) {
            Some(chain) => Ok(f(chain)),
            None => {
                warn!(%chain_id, chain_type = %<C as Chain>::ChainType::EXPECTING, "chain not found");
                Err(ChainNotFoundError {
                    chain_id: chain_id.clone(),
                })
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("chain {} not found (type: {})", chain_id, <C as Chain>::ChainType::EXPECTING)]
pub struct ChainNotFoundError<C: Chain> {
    chain_id: ChainIdOf<C>,
}

#[derive(Debug, Clone, Default)]
pub struct Chains {
    pub chains: HashMap<String, AnyChain>,
}

impl GetChain<Union> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Union>) -> Option<Union> {
        self.chains
            .get(chain_id)
            .cloned()
            .map(|chain| chain.try_into().expect("chain is correct type"))
    }
}

impl GetChain<Cosmos> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Cosmos>) -> Option<Cosmos> {
        self.chains
            .get(chain_id)
            .cloned()
            .map(|chain| chain.try_into().expect("chain is correct type"))
    }
}

impl GetChain<Scroll> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Scroll>) -> Option<Scroll> {
        self.chains
            .get(&chain_id.to_string())
            .cloned()
            .map(|chain| chain.try_into().expect("chain is correct type"))
    }
}

impl GetChain<Arbitrum> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Arbitrum>) -> Option<Arbitrum> {
        self.chains
            .get(&chain_id.to_string())
            .cloned()
            .map(|chain| chain.try_into().expect("chain is correct type"))
    }
}

impl GetChain<Berachain> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Berachain>) -> Option<Berachain> {
        self.chains
            .get(&chain_id.to_string())
            .cloned()
            .map(|chain| chain.try_into().expect("chain is correct type"))
    }
}

impl GetChain<Wasm<Union>> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Wasm<Union>>) -> Option<Wasm<Union>> {
        self.chains
            .get(chain_id)
            .cloned()
            .map(|chain| chain.try_into().expect("chain is correct type"))
            .map(Wasm)
    }
}

impl GetChain<Wasm<Cosmos>> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Wasm<Cosmos>>) -> Option<Wasm<Cosmos>> {
        self.chains
            .get(chain_id)
            .cloned()
            .map(|chain| chain.try_into().expect("chain is correct type"))
            .map(Wasm)
    }
}

impl GetChain<Ethereum<Minimal>> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Ethereum<Minimal>>) -> Option<Ethereum<Minimal>> {
        self.chains
            .get(&chain_id.to_string())
            .cloned()
            .map(|chain| chain.try_into().expect("chain is correct type"))
    }
}

impl GetChain<Ethereum<Mainnet>> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Ethereum<Mainnet>>) -> Option<Ethereum<Mainnet>> {
        self.chains
            .get(&chain_id.to_string())
            .cloned()
            .map(|chain| chain.try_into().expect("chain is correct type"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "chain_type")]
pub enum ChainConfigType {
    Union(union::Config),
    Cosmos(cosmos::Config),
    Ethereum(EthereumChainConfig),
    Scroll(scroll::Config),
    Arbitrum(arbitrum::Config),
    Berachain(berachain::Config),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthereumChainConfig {
    pub preset_base: PresetBaseKind,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,
    pub multicall_address: H160,

    /// The signers that will be used to submit transactions by voyager.
    pub keyring: KeyringConfig,

    // TODO(benluelo): Use `Url` or something similar
    /// The RPC endpoint for the execution chain.
    pub eth_rpc_api: String,
    /// The RPC endpoint for the beacon chain.
    pub eth_beacon_rpc_api: String,

    #[serde(default)]
    pub max_gas_price: Option<U256>,
}

#[derive(Debug, Clone, Enumorph)]
pub enum AnyChain {
    Union(Union),
    Cosmos(Cosmos),
    EthereumMainnet(Ethereum<Mainnet>),
    EthereumMinimal(Ethereum<Minimal>),
    Scroll(Scroll),
    Arbitrum(Arbitrum),
    Berachain(Berachain),
}

impl AnyChain {
    pub fn downcast<T: TryFrom<Self, Error = Self> + Chain>(
        self,
    ) -> Result<T, IncorrectChainTypeError> {
        self.try_into().map_err(|c| {
            any_chain!(|c| IncorrectChainTypeError {
                found_chain_name: c.chain_id().to_string(),
                found_chain_type: <Hc as Chain>::ChainType::EXPECTING,
                expected_chain_type: T::ChainType::EXPECTING
            })
        })
    }

    pub fn chain_id(&self) -> String {
        let this = self;

        any_chain!(|this| this.chain_id().to_string())
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("chain `{found_chain_name}` is of type `{found_chain_type}` but `{expected_chain_type}` was expected")]
pub struct IncorrectChainTypeError {
    pub found_chain_name: String,
    pub found_chain_type: &'static str,
    pub expected_chain_type: &'static str,
}

#[macro_export]
macro_rules! any_chain {
    (|$c:ident| $expr:expr) => {
        match $c {
            AnyChain::Union($c) => {
                #[allow(dead_code)]
                type Hc = $crate::union::Union;
                $expr
            }
            AnyChain::Cosmos($c) => {
                #[allow(dead_code)]
                type Hc = $crate::cosmos::Cosmos;
                $expr
            }
            AnyChain::EthereumMainnet($c) => {
                #[allow(dead_code)]
                type Hc = $crate::ethereum::Ethereum<::unionlabs::ethereum::config::Mainnet>;
                $expr
            }
            AnyChain::EthereumMinimal($c) => {
                #[allow(dead_code)]
                type Hc = $crate::ethereum::Ethereum<::unionlabs::ethereum::config::Minimal>;
                $expr
            }
            AnyChain::Scroll($c) => {
                #[allow(dead_code)]
                type Hc = $crate::scroll::Scroll;
                $expr
            }
            AnyChain::Arbitrum($c) => {
                #[allow(dead_code)]
                type Hc = $crate::arbitrum::Arbitrum;
                $expr
            }
            AnyChain::Berachain($c) => {
                #[allow(dead_code)]
                type Hc = $crate::berachain::Berachain;
                $expr
            }
        }
    };
}

#[derive(Debug, thiserror::Error)]
pub enum AnyChainTryFromConfigError {
    #[error("error initializing a union chain")]
    Union(#[from] UnionInitError),
    #[error("error initializing a cosmos chain")]
    Cosmos(#[from] CosmosInitError),
    // #[error("error initializing an ethereum chain")]
    // Ethereum(#[from] EthereumInitError),
    #[error("error initializing a scroll chain")]
    Scroll(#[from] ScrollInitError),
    #[error("error initializing an arbitrum chain")]
    Arbitrum(#[from] ArbitrumInitError),
    #[error("error initializing a berachain chain")]
    Berachain(#[from] BerachainInitError),
}

impl AnyChain {
    pub async fn try_from_config(
        config: ChainConfigType,
    ) -> Result<Self, AnyChainTryFromConfigError> {
        Ok(match config {
            ChainConfigType::Union(union) => Self::Union(Union::new(union).await?),
            ChainConfigType::Cosmos(cosmos) => Self::Cosmos(Cosmos::new(cosmos).await?),
            ChainConfigType::Ethereum(_ethereum) => {
                // let config = crate::ethereum::Config {
                //     ibc_handler_address: ethereum.ibc_handler_address,
                //     multicall_address: ethereum.multicall_address,
                //     keyring: ethereum.keyring,
                //     eth_rpc_api: ethereum.eth_rpc_api,
                //     eth_beacon_rpc_api: ethereum.eth_beacon_rpc_api,
                //     max_gas_price: ethereum.max_gas_price,
                // };
                todo!()
                // match ethereum.preset_base {
                //     PresetBaseKind::Minimal => {
                //         Self::EthereumMinimal(Ethereum::<Minimal>::new(config).await?)
                //     }
                //     PresetBaseKind::Mainnet => {
                //         Self::EthereumMainnet(Ethereum::<Mainnet>::new(config).await?)
                //     }
                // }
            }
            ChainConfigType::Scroll(scroll) => Self::Scroll(Scroll::new(scroll).await?),
            ChainConfigType::Arbitrum(arbitrum) => Self::Arbitrum(Arbitrum::new(arbitrum).await?),
            ChainConfigType::Berachain(berachain) => {
                Self::Berachain(Berachain::new(berachain).await?)
            }
        })
    }
}

pub trait LightClientType<Tr: Chain>: Chain {
    /// How [`Self`] tracks [`Tr`].
    const TYPE: ClientType;
}

impl LightClientType<Union> for Wasm<Cosmos> {
    const TYPE: ClientType = ClientType::Wasm(WasmClientType::Cometbls);
}

impl LightClientType<Wasm<Cosmos>> for Union {
    const TYPE: ClientType = ClientType::Tendermint;
}

impl LightClientType<Cosmos> for Union {
    const TYPE: ClientType = ClientType::Tendermint;
}

impl LightClientType<Union> for Cosmos {
    const TYPE: ClientType = ClientType::_11Cometbls;
}

impl LightClientType<Cosmos> for Cosmos {
    const TYPE: ClientType = ClientType::Tendermint;
}

impl<C: ChainSpec> LightClientType<Wasm<Union>> for Ethereum<C> {
    const TYPE: ClientType = ClientType::Cometbls;
}

impl LightClientType<Wasm<Union>> for Scroll {
    const TYPE: ClientType = ClientType::Cometbls;
}

impl LightClientType<Wasm<Union>> for Berachain {
    const TYPE: ClientType = ClientType::Cometbls;
}

impl LightClientType<Scroll> for Wasm<Union> {
    const TYPE: ClientType = ClientType::Wasm(WasmClientType::Scroll);
}

impl LightClientType<Berachain> for Wasm<Union> {
    const TYPE: ClientType = ClientType::Wasm(WasmClientType::Berachain);
}

impl LightClientType<Wasm<Union>> for Arbitrum {
    const TYPE: ClientType = ClientType::Cometbls;
}

impl LightClientType<Arbitrum> for Wasm<Union> {
    const TYPE: ClientType = ClientType::Wasm(WasmClientType::Arbitrum);
}

impl LightClientType<Ethereum<Mainnet>> for Wasm<Union> {
    const TYPE: ClientType = ClientType::Wasm(WasmClientType::EthereumMainnet);
}

impl LightClientType<Ethereum<Minimal>> for Wasm<Union> {
    const TYPE: ClientType = ClientType::Wasm(WasmClientType::EthereumMinimal);
}

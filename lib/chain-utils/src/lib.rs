// #![warn(clippy::pedantic)]
#![feature(trait_alias)]

use std::{collections::HashMap, sync::Arc};

use bip32::secp256k1::ecdsa;
use crossbeam_queue::ArrayQueue;
use enumorph::Enumorph;
use futures::Future;
use serde::{Deserialize, Serialize};
use unionlabs::{
    ethereum::config::{ChainSpec, Mainnet, Minimal, PresetBaseKind},
    hash::H160,
    traits::{Chain, ChainIdOf, FromStrExact},
    ClientType, WasmClientType,
};

use crate::{
    arbitrum::{Arbitrum, ArbitrumInitError},
    cosmos::{Cosmos, CosmosInitError},
    ethereum::{Ethereum, EthereumInitError},
    private_key::PrivateKey,
    scroll::{Scroll, ScrollInitError},
    union::{Union, UnionInitError},
    wasm::Wasm,
};

pub mod arbitrum;
pub mod cosmos;
pub mod ethereum;
pub mod scroll;
pub mod union;

pub mod cosmos_sdk;

pub mod wasm;

pub mod private_key;

#[derive(Debug, Clone)]
pub struct Pool<T> {
    pool: Arc<ArrayQueue<T>>,
}

impl<T: Clone> Pool<T> {
    pub fn new(ts: impl ExactSizeIterator<Item = T>) -> Self {
        let data = ArrayQueue::new(ts.len());

        for t in ts {
            data.push(t)
                .map_err(|_| ())
                .expect("queue is initialized with the correct length; qed;");
        }

        Self {
            pool: Arc::new(data),
        }
    }

    pub async fn with<R, F: FnOnce(T) -> Fut, Fut: Future<Output = R>>(&self, f: F) -> R {
        let t = loop {
            match self.pool.pop() {
                Some(t) => break t,
                None => {
                    const RETRY_SECONDS: u64 = 3;

                    tracing::warn!(
                        "high traffic in pool of {}, ran out of items! trying again in {RETRY_SECONDS} seconds",
                        std::any::type_name::<T>()
                    );

                    tokio::time::sleep(std::time::Duration::from_secs(RETRY_SECONDS)).await;

                    continue;
                }
            }
        };

        // TODO: Figure out a way to pass this as ref
        let r = f(t.clone()).await;

        self.pool
            .push(t)
            .map_err(|_| ())
            .expect("no additional items are added; qed;");

        r
    }
}

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
                tracing::warn!(%chain_id, chain_type = %<C as Chain>::ChainType::EXPECTING, "chain not found");
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

type ChainMap<C> = HashMap<ChainIdOf<C>, C>;

#[derive(Debug, Clone, Default)]
pub struct Chains {
    // TODO: Use some sort of typemap here instead of individual fields
    pub ethereum_minimal: ChainMap<Ethereum<Minimal>>,
    pub ethereum_mainnet: ChainMap<Ethereum<Mainnet>>,
    pub union: ChainMap<Union>,
    pub cosmos: ChainMap<Cosmos>,
    pub scroll: ChainMap<Scroll>,
    pub arbitrum: ChainMap<Arbitrum>,
}

impl GetChain<Union> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Union>) -> Option<Union> {
        self.union.get(chain_id).cloned()
    }
}

impl GetChain<Cosmos> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Cosmos>) -> Option<Cosmos> {
        self.cosmos.get(chain_id).cloned()
    }
}

impl GetChain<Scroll> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Scroll>) -> Option<Scroll> {
        self.scroll.get(chain_id).cloned()
    }
}

impl GetChain<Arbitrum> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Arbitrum>) -> Option<Arbitrum> {
        self.arbitrum.get(chain_id).cloned()
    }
}

impl GetChain<Wasm<Union>> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Wasm<Union>>) -> Option<Wasm<Union>> {
        self.union.get(chain_id).cloned().map(Wasm)
    }
}

impl GetChain<Wasm<Cosmos>> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Wasm<Cosmos>>) -> Option<Wasm<Cosmos>> {
        self.cosmos.get(chain_id).cloned().map(Wasm)
    }
}

impl GetChain<Ethereum<Minimal>> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Ethereum<Minimal>>) -> Option<Ethereum<Minimal>> {
        self.ethereum_minimal.get(chain_id).cloned()
    }
}

impl GetChain<Ethereum<Mainnet>> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Ethereum<Mainnet>>) -> Option<Ethereum<Mainnet>> {
        self.ethereum_mainnet.get(chain_id).cloned()
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthereumChainConfig {
    pub preset_base: PresetBaseKind,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    /// The signer that will be used to submit transactions by voyager.
    pub signers: Vec<PrivateKey<ecdsa::SigningKey>>,

    // TODO(benluelo): Use `Url` or something similar
    /// The RPC endpoint for the execution chain.
    pub eth_rpc_api: String,
    /// The RPC endpoint for the beacon chain.
    pub eth_beacon_rpc_api: String,
}

#[derive(Debug, Enumorph)]
pub enum AnyChain {
    Union(Union),
    Cosmos(Cosmos),
    EthereumMainnet(Ethereum<Mainnet>),
    EthereumMinimal(Ethereum<Minimal>),
    Scroll(Scroll),
    Arbitrum(Arbitrum),
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
    #[error("error initializing an arbitrum chain")]
    Arbitrum(#[from] ArbitrumInitError),
}

impl AnyChain {
    pub async fn try_from_config(
        config: ChainConfigType,
    ) -> Result<Self, AnyChainTryFromConfigError> {
        Ok(match config {
            ChainConfigType::Union(union) => Self::Union(Union::new(union).await?),
            ChainConfigType::Cosmos(cosmos) => Self::Cosmos(Cosmos::new(cosmos).await?),
            ChainConfigType::Ethereum(ethereum) => {
                let config = crate::ethereum::Config {
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
            ChainConfigType::Arbitrum(arbitrum) => Self::Arbitrum(Arbitrum::new(arbitrum).await?),
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

impl LightClientType<Cosmos> for Cosmos {
    const TYPE: ClientType = ClientType::Tendermint;
}

impl<C: ChainSpec> LightClientType<Wasm<Union>> for Ethereum<C> {
    const TYPE: ClientType = ClientType::Cometbls;
}

impl LightClientType<Wasm<Union>> for Scroll {
    const TYPE: ClientType = ClientType::Cometbls;
}

impl LightClientType<Scroll> for Wasm<Union> {
    const TYPE: ClientType = ClientType::Wasm(WasmClientType::Scroll);
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

// #![warn(clippy::pedantic)]
#![feature(trait_alias)]

use std::{collections::HashMap, sync::Arc};

use crossbeam_queue::ArrayQueue;
use futures::Future;
use unionlabs::{
    ethereum::config::{Mainnet, Minimal},
    traits::{Chain, ChainIdOf, FromStrExact},
};

use crate::{cosmos::Cosmos, ethereum::Ethereum, scroll::Scroll, union::Union, wasm::Wasm};

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

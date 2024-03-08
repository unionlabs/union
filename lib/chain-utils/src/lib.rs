// #![warn(clippy::pedantic)]
#![feature(trait_alias)]

use std::{collections::HashMap, fmt::Debug, sync::Arc};

use crossbeam_queue::ArrayQueue;
use futures::Future;
use unionlabs::{
    ethereum::config::{Mainnet, Minimal},
    traits::{Chain, ChainIdOf},
};

use crate::{
    cosmos::Cosmos,
    evm::{Ethereum, ReadWrite},
    scroll::Scroll,
    union::Union,
    wasm::Wasm,
};

pub mod cosmos;
pub mod evm;
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
    fn get_chain(&self, chain_id: &ChainIdOf<C>) -> C;
}

#[derive(Debug, Clone, Default)]
pub struct Chains {
    // TODO: Use some sort of typemap here instead of individual fields
    pub evm_minimal: HashMap<ChainIdOf<Ethereum<Minimal>>, Ethereum<Minimal>>,
    pub evm_mainnet: HashMap<ChainIdOf<Ethereum<Mainnet>>, Ethereum<Mainnet>>,
    pub union: HashMap<ChainIdOf<Union>, Union>,
    pub cosmos: HashMap<ChainIdOf<Cosmos>, Cosmos>,
    pub scroll: HashMap<ChainIdOf<Scroll>, Scroll>,
}

impl GetChain<Union> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Union>) -> Union {
        self.union.get(chain_id).unwrap().clone()
    }
}

impl GetChain<Cosmos> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Cosmos>) -> Cosmos {
        self.cosmos.get(chain_id).unwrap().clone()
    }
}

impl GetChain<Scroll> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Scroll>) -> Scroll {
        self.scroll.get(chain_id).unwrap().clone()
    }
}

impl GetChain<Wasm<Union>> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Wasm<Union>>) -> Wasm<Union> {
        Wasm(self.union.get(chain_id).unwrap().clone())
    }
}

impl GetChain<Wasm<Cosmos>> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Wasm<Cosmos>>) -> Wasm<Cosmos> {
        Wasm(self.cosmos.get(chain_id).unwrap().clone())
    }
}

impl GetChain<Ethereum<Minimal>> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Ethereum<Minimal>>) -> Ethereum<Minimal> {
        self.evm_minimal.get(chain_id).unwrap().clone()
    }
}

impl GetChain<Ethereum<Mainnet>> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Ethereum<Mainnet>>) -> Ethereum<Mainnet> {
        self.evm_mainnet.get(chain_id).unwrap().clone()
    }
}

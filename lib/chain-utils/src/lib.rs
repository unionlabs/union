// #![warn(clippy::pedantic)]
#![allow(
     // required due to return_position_impl_trait_in_trait false positives
    clippy::manual_async_fn,
)]
#![feature(trait_alias)]

use std::{collections::HashMap, fmt::Debug, sync::Arc};

use crossbeam_queue::ArrayQueue;
use futures::Future;
use unionlabs::{
    ethereum::config::{Mainnet, Minimal},
    traits::{Chain, ChainIdOf},
};

use crate::{cosmos::Cosmos, evm::Evm, union::Union, wasm::Wasm};

pub mod cosmos;
pub mod evm;
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
    pub evm_minimal: HashMap<ChainIdOf<Evm<Minimal>>, Evm<Minimal>>,
    pub evm_mainnet: HashMap<ChainIdOf<Evm<Mainnet>>, Evm<Mainnet>>,
    pub union: HashMap<ChainIdOf<Union>, Union>,
    pub cosmos: HashMap<ChainIdOf<Cosmos>, Cosmos>,
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

impl GetChain<Evm<Minimal>> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Evm<Minimal>>) -> Evm<Minimal> {
        self.evm_minimal.get(chain_id).unwrap().clone()
    }
}

impl GetChain<Evm<Mainnet>> for Chains {
    fn get_chain(&self, chain_id: &ChainIdOf<Evm<Mainnet>>) -> Evm<Mainnet> {
        self.evm_mainnet.get(chain_id).unwrap().clone()
    }
}

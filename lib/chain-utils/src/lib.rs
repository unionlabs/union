// #![warn(clippy::pedantic)]
#![allow(
     // required due to return_position_impl_trait_in_trait false positives
    clippy::manual_async_fn,
)]
#![feature(trait_alias)]

use std::{fmt::Debug, sync::Arc};

use crossbeam_queue::ArrayQueue;
use futures::{Future, Stream};
use unionlabs::{
    events::IbcEvent,
    hash::H256,
    ibc::core::client::height::Height,
    traits::{Chain, ClientState},
};

pub mod cosmos;
pub mod evm;
pub mod union;

pub mod cosmos_sdk;

pub mod private_key;

pub trait EventSource {
    type Event;
    type Error: Debug;
    /// The initial state of this event source, if any.
    type Seed;

    fn events(self, seed: Self::Seed) -> impl Stream<Item = Result<Self::Event, Self::Error>>;
}

// Serialize, Deserialize
#[derive(Debug, Clone, PartialEq)]
pub struct ChainEvent<C: Chain> {
    /// The chain this event originated from.
    pub chain_id: <C::SelfClientState as ClientState>::ChainId,
    pub block_hash: H256,
    pub height: Height,
    pub event: IbcEvent<C::ClientId, C::ClientType, String>,
}

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

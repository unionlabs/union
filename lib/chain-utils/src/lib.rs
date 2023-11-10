// #![warn(clippy::pedantic)]
#![allow(
     // required due to return_position_impl_trait_in_trait false positives
    clippy::manual_async_fn,
)]

use std::{
    error::Error,
    fmt::{Debug, Display},
    sync::Arc,
};

use crossbeam_queue::ArrayQueue;
use futures::{Future, Stream};
use unionlabs::{
    ethereum::H256,
    events::IbcEvent,
    ibc::core::client::height::Height,
    traits::{Chain, ChainIdOf},
};

pub mod evm;
pub mod union;

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
    pub chain_id: ChainIdOf<C>,
    pub block_hash: H256,
    pub height: Height,
    pub event: IbcEvent<C::ClientId, C::ClientType, String>,
}

// TODO: Make this a more generic error and put it in unionlabs::errors
#[derive(Debug, Clone, PartialEq)]
pub struct ChainClientIdParseError {
    expected: &'static [&'static str],
    found: String,
}

impl Display for ChainClientIdParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "expected one of `{}`, found `{}`",
            self.expected
                .iter()
                .map(|exp| format!("`{exp}`"))
                .collect::<Vec<_>>()
                .join(","),
            self.found,
        ))
    }
}

impl Error for ChainClientIdParseError {}

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
                        "high traffic in queue of {}, ran out of items! trying again in {RETRY_SECONDS} seconds",
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

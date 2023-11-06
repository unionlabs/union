use std::{fmt::Display, marker::PhantomData};

use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use serde::{Deserialize, Serialize};
use unionlabs::{
    ibc::core::client::height::IsHeight,
    traits::{Chain, ChainIdOf, ChainOf, ClientState, HeightOf, LightClientBase},
    QueryHeight,
};

use crate::{
    any_enum, defer, fetch,
    fetch::{AnyFetch, Fetch, FetchTrustedClientState},
    identified, now, seq, wait, AnyLightClientIdentified, LightClient, RelayerMsg,
};

any_enum! {
    /// Defines messages that are sent *to* the lightclient `L`.
    #[any = AnyWait]
    pub enum Wait<L: LightClient> {
        Block(WaitForBlock<L>),
        Timestamp(WaitForTimestamp<L>),
        TrustedHeight(WaitForTrustedHeight<L>),
    }
}

impl<L: LightClient> Wait<L> {
    pub async fn handle(self, l: L) -> Vec<RelayerMsg>
    where
        AnyLightClientIdentified<AnyWait>: From<identified!(Wait<L>)>,
        AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<L::Counterparty>)>,
    {
        match self {
            Wait::Block(WaitForBlock(height)) => {
                let chain_height = l.chain().query_latest_height().await;

                assert_eq!(
                    chain_height.revision_number(),
                    height.revision_number(),
                    "chain_height: {chain_height}, height: {height}",
                );

                if chain_height.revision_height() >= height.revision_height() {
                    [].into()
                } else {
                    [seq([
                        // REVIEW: Defer until `now + chain.block_time()`? Would require a new method on chain
                        defer(now() + 1),
                        wait::<L>(l.chain().chain_id(), WaitForBlock(height)),
                    ])]
                    .into()
                }
            }
            Wait::Timestamp(WaitForTimestamp {
                timestamp,
                __marker,
            }) => {
                let chain_ts = l.chain().query_latest_timestamp().await;

                if chain_ts >= timestamp {
                    [].into()
                } else {
                    [seq([
                        // REVIEW: Defer until `now + chain.block_time()`? Would require a new method on chain
                        defer(now() + 1),
                        wait::<L>(
                            l.chain().chain_id(),
                            WaitForTimestamp {
                                timestamp,
                                __marker,
                            },
                        ),
                    ])]
                    .into()
                }
            }
            Wait::TrustedHeight(WaitForTrustedHeight {
                client_id,
                height,
                counterparty_client_id,
                counterparty_chain_id,
            }) => {
                let latest_height = l.chain().query_latest_height_as_destination().await;
                let trusted_client_state = l
                    .query_client_state(client_id.clone().into(), latest_height)
                    .await;

                if trusted_client_state.height().revision_height() >= height.revision_height() {
                    tracing::debug!(
                        "client height reached ({} >= {})",
                        trusted_client_state.height(),
                        height
                    );

                    [fetch::<L::Counterparty>(
                        counterparty_chain_id,
                        FetchTrustedClientState {
                            at: QueryHeight::Specific(trusted_client_state.height()),
                            client_id: counterparty_client_id.clone(),
                        },
                    )]
                    .into()
                } else {
                    [seq([
                        // REVIEW: Defer until `now + counterparty_chain.block_time()`? Would require a new method on chain
                        defer(now() + 1),
                        wait::<L>(
                            l.chain().chain_id(),
                            Wait::TrustedHeight(WaitForTrustedHeight {
                                client_id,
                                height,
                                counterparty_client_id,
                                counterparty_chain_id,
                            }),
                        ),
                    ])]
                    .into()
                }
            }
        }
    }
}

impl<L: LightClient> Display for Wait<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Wait::Block(block) => write!(f, "Block({})", block.0),
            Wait::Timestamp(ts) => write!(f, "Timestamp({})", ts.timestamp),
            Wait::TrustedHeight(th) => write!(f, "TrustedHeight({})", th.height),
        }
    }
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct WaitForBlock<L: LightClient>(pub HeightOf<ChainOf<L>>);

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct WaitForTimestamp<L: LightClient> {
    pub timestamp: i64,
    #[serde(skip)]
    pub __marker: PhantomData<L>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct WaitForTrustedHeight<L: LightClient> {
    pub client_id: L::ClientId,
    pub counterparty_client_id: <L::Counterparty as LightClientBase>::ClientId,
    pub counterparty_chain_id: ChainIdOf<L::Counterparty>,
    pub height: HeightOf<ChainOf<L::Counterparty>>,
}

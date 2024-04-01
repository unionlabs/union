use std::fmt::Display;

use chain_utils::{ChainNotFoundError, GetChain};
use macros::apply;
use queue_msg::{
    defer_absolute, fetch, msg_struct, now, seq, wait, HandleWait, QueueError, QueueMsg,
    QueueMsgTypes,
};
use unionlabs::{
    ibc::core::client::height::IsHeight,
    proof::ClientStatePath,
    traits::{ChainIdOf, ClientState, HeightOf},
};

use crate::{
    any_enum, any_lc,
    fetch::{AnyFetch, Fetch, FetchState},
    id, identified, AnyLightClientIdentified, ChainExt, DoFetchState, RelayMessageTypes,
};

#[apply(any_enum)]
#[any = AnyWait]
pub enum Wait<Hc: ChainExt, Tr: ChainExt> {
    Block(WaitForBlock<Hc, Tr>),
    Timestamp(WaitForTimestamp<Hc, Tr>),
    TrustedHeight(WaitForTrustedHeight<Hc, Tr>),
}

impl HandleWait<RelayMessageTypes> for AnyLightClientIdentified<AnyWait> {
    async fn handle(
        self,
        store: &<RelayMessageTypes as QueueMsgTypes>::Store,
    ) -> Result<QueueMsg<RelayMessageTypes>, QueueError> {
        let wait = self;

        any_lc! {
            |wait| {
                Ok(store
                    .with_chain(&wait.chain_id, move |c| async move { wait.t.handle(&c).await })
                    .map_err(|e: ChainNotFoundError<Hc>| QueueError::Fatal(Box::new(e)))?
                    .await)
            }
        }
    }
}

impl<Hc, Tr> Wait<Hc, Tr>
where
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Hc, Tr>)>,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Tr, Hc>)>,
    Hc: ChainExt + DoFetchState<Hc, Tr>,
    Tr: ChainExt,
{
    pub async fn handle(self, c: &Hc) -> QueueMsg<RelayMessageTypes> {
        match self {
            Wait::Block(WaitForBlock { height, __marker }) => {
                let chain_height = c.query_latest_height().await.unwrap();

                assert_eq!(
                    chain_height.revision_number(),
                    height.revision_number(),
                    "chain_height: {chain_height}, height: {height}",
                );

                if chain_height.revision_height() >= height.revision_height() {
                    QueueMsg::Noop
                } else {
                    seq([
                        // REVIEW: Defer until `now + chain.block_time()`? Would require a new method on chain
                        defer_absolute(now() + 1),
                        wait(id::<Hc, Tr, _>(
                            c.chain_id(),
                            WaitForBlock { height, __marker }.into(),
                        )),
                    ])
                }
            }
            Wait::Timestamp(WaitForTimestamp {
                timestamp,
                __marker,
            }) => {
                let chain_ts = c.query_latest_timestamp().await.unwrap();

                if chain_ts >= timestamp {
                    QueueMsg::Noop
                } else {
                    seq([
                        // REVIEW: Defer until `now + chain.block_time()`? Would require a new method on chain
                        defer_absolute(now() + 1),
                        wait(id::<Hc, Tr, _>(
                            c.chain_id(),
                            WaitForTimestamp {
                                timestamp,
                                __marker,
                            }
                            .into(),
                        )),
                    ])
                }
            }
            Wait::TrustedHeight(WaitForTrustedHeight {
                client_id,
                counterparty_client_id,
                counterparty_chain_id,
                height,
            }) => {
                let latest_height = c.query_latest_height_as_destination().await.unwrap();

                let trusted_client_state =
                    Hc::query_client_state(c, client_id.clone(), latest_height).await;

                if trusted_client_state.height().revision_height() >= height.revision_height() {
                    tracing::debug!(
                        "client height reached ({} >= {})",
                        trusted_client_state.height(),
                        height
                    );

                    fetch(id::<Tr, Hc, _>(
                        counterparty_chain_id,
                        FetchState {
                            at: trusted_client_state.height(),
                            path: ClientStatePath {
                                client_id: counterparty_client_id.clone(),
                            }
                            .into(),
                        },
                    ))
                } else {
                    seq([
                        // REVIEW: Defer until `now + counterparty_chain.block_time()`? Would require a new method on chain
                        defer_absolute(now() + 1),
                        wait(id(
                            c.chain_id(),
                            WaitForTrustedHeight {
                                client_id,
                                height,
                                counterparty_client_id,
                                counterparty_chain_id,
                            },
                        )),
                    ])
                }
            }
        }
    }
}

impl<Hc: ChainExt, Tr: ChainExt> Display for Wait<Hc, Tr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Wait::Block(block) => write!(f, "Block({})", block.height),
            Wait::Timestamp(ts) => write!(f, "Timestamp({})", ts.timestamp),
            Wait::TrustedHeight(th) => write!(f, "TrustedHeight({})", th.height),
        }
    }
}

#[apply(msg_struct)]
#[cover(Tr)]
pub struct WaitForBlock<Hc: ChainExt, Tr: ChainExt> {
    pub height: HeightOf<Hc>,
}

#[apply(msg_struct)]
#[cover(Hc, Tr)]
pub struct WaitForTimestamp<Hc: ChainExt, Tr: ChainExt> {
    pub timestamp: i64,
}

#[apply(msg_struct)]
pub struct WaitForTrustedHeight<Hc: ChainExt, Tr: ChainExt> {
    pub client_id: Hc::ClientId,
    pub counterparty_client_id: Tr::ClientId,
    pub counterparty_chain_id: ChainIdOf<Tr>,
    pub height: Tr::Height,
}

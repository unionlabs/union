use std::{fmt::Debug, marker::PhantomData};

use chain_utils::GetChain;
use futures::Future;
use macros::apply;
use queue_msg::{data, fetch, queue_msg, HandleFetch, Op, QueueError, QueueMessage};
use tracing::instrument;
use unionlabs::{
    ics24,
    never::Never,
    traits::{ChainIdOf, ClientIdOf, HeightOf},
    QueryHeight,
};

use crate::{
    any_enum, any_lc,
    data::{AnyData, Data, LatestHeight, SelfClientState, SelfConsensusState},
    id, identified, AnyLightClientIdentified, ChainExt, DoFetchProof, DoFetchState,
    DoFetchUpdateHeaders, RelayMessage,
};

#[apply(any_enum)]
/// Fetch some data that will likely be used in a [`QueueMsg::Aggregate`].
#[any = AnyFetch]
#[specific = LightClientSpecificFetch]
pub enum Fetch<Hc: ChainExt, Tr: ChainExt> {
    State(FetchState<Hc, Tr>),
    Proof(FetchProof<Hc, Tr>),

    LatestHeight(FetchLatestHeight<Hc, Tr>),

    UnfinalizedTrustedClientState(FetchUnfinalizedTrustedClientState<Hc, Tr>),

    SelfClientState(FetchSelfClientState<Hc, Tr>),
    SelfConsensusState(FetchSelfConsensusState<Hc, Tr>),

    UpdateHeaders(FetchUpdateHeaders<Hc, Tr>),

    #[serde(untagged)]
    LightClientSpecific(LightClientSpecificFetch<Hc, Tr>),
}

impl HandleFetch<RelayMessage> for AnyLightClientIdentified<AnyFetch> {
    #[instrument(skip_all, fields(chain_id = %self.chain_id()))]
    async fn handle(
        self,
        store: &<RelayMessage as QueueMessage>::Store,
    ) -> Result<Op<RelayMessage>, QueueError> {
        let fetch = self;

        any_lc! {
            |fetch| {
                Ok(store
                    .with_chain(&fetch.chain_id, move |c| async move { fetch.t.handle(c).await })
                    .map_err(|e| QueueError::Fatal(Box::new(e)))?
                    .await)
            }
        }
    }
}

pub trait DoFetch<Hc: ChainExt>: Sized + Debug + Clone + PartialEq {
    fn do_fetch(c: &Hc, _: Self) -> impl Future<Output = Op<RelayMessage>>;
}

impl<Hc: ChainExt> DoFetch<Hc> for Never {
    async fn do_fetch(_: &Hc, this: Self) -> Op<RelayMessage> {
        match this {}
    }
}

#[queue_msg]
pub struct FetchSelfClientState<Hc: ChainExt, #[cover] Tr: ChainExt> {
    pub at: QueryHeight<HeightOf<Hc>>,
}

#[queue_msg]
pub struct FetchSelfConsensusState<Hc: ChainExt, #[cover] Tr: ChainExt> {
    pub at: QueryHeight<HeightOf<Hc>>,
}

#[queue_msg]
pub struct FetchProof<Hc: ChainExt, Tr: ChainExt> {
    pub at: HeightOf<Hc>,
    pub path: ics24::Path<Hc::ClientId, Tr::Height>,
}

#[queue_msg]
pub struct FetchState<Hc: ChainExt, Tr: ChainExt> {
    pub at: QueryHeight<HeightOf<Hc>>,
    pub path: ics24::Path<Hc::ClientId, Tr::Height>,
}

#[queue_msg]
pub struct FetchUpdateHeaders<Hc: ChainExt, Tr: ChainExt> {
    pub counterparty_chain_id: ChainIdOf<Tr>,
    // id of the counterparty client that will be updated with the fetched headers
    pub counterparty_client_id: ClientIdOf<Tr>,
    pub update_from: HeightOf<Hc>,
    pub update_to: HeightOf<Hc>,
}

#[queue_msg]
pub struct FetchLatestHeight<#[cover] Hc: ChainExt, #[cover] Tr: ChainExt> {}

#[queue_msg]
pub struct FetchUnfinalizedTrustedClientState<Hc: ChainExt, #[cover] Tr: ChainExt> {
    client_id: Hc::ClientId,
}

#[queue_msg]
pub struct LightClientSpecificFetch<Hc: ChainExt, Tr: ChainExt>(pub Hc::Fetch<Tr>);

impl<Hc, Tr> Fetch<Hc, Tr>
where
    Hc: ChainExt<Fetch<Tr>: DoFetch<Hc>>
        + DoFetchState<Hc, Tr>
        + DoFetchProof<Hc, Tr>
        + DoFetchUpdateHeaders<Hc, Tr>,

    Tr: ChainExt,
    AnyLightClientIdentified<AnyData>: From<identified!(Data<Hc, Tr>)>,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
{
    pub async fn handle(self, c: Hc) -> Op<RelayMessage> {
        match self {
            Fetch::Proof(msg) => Hc::proof(&c, msg.at, msg.path),
            Fetch::State(msg) => match msg.at {
                QueryHeight::Latest => fetch(id(
                    c.chain_id(),
                    FetchState {
                        at: QueryHeight::Specific(c.query_latest_height().await.unwrap()),
                        path: msg.path,
                    },
                )),
                QueryHeight::Specific(at) => Hc::state(&c, at, msg.path),
            },
            Fetch::LatestHeight(FetchLatestHeight { __marker: _ }) => data(id(
                c.chain_id(),
                LatestHeight {
                    height: c.query_latest_height().await.unwrap(),
                    __marker: PhantomData,
                },
            )),
            Fetch::UnfinalizedTrustedClientState(FetchUnfinalizedTrustedClientState {
                client_id,
                __marker: _,
            }) => {
                let _client_state = Hc::query_unfinalized_trusted_client_state(&c, client_id).await;

                // data(id(
                //     c.chain_id(),
                //     UnfinalizedTrustedClientState {
                //         height,
                //         client_state,
                //     },
                // ))
                todo!()
            }
            Fetch::SelfClientState(FetchSelfClientState {
                at: height,
                __marker: _,
            }) => {
                // TODO: Split this into a separate query and aggregate
                let height = match height {
                    QueryHeight::Latest => c.query_latest_height().await.unwrap(),
                    QueryHeight::Specific(h) => h,
                };

                data(id::<Hc, Tr, _>(
                    c.chain_id(),
                    SelfClientState {
                        self_client_state: c.self_client_state(height).await,
                        __marker: PhantomData,
                    },
                ))
            }
            Fetch::SelfConsensusState(FetchSelfConsensusState {
                at: height,
                __marker: _,
            }) => {
                // TODO: Split this into a separate query and aggregate
                let height = match height {
                    QueryHeight::Latest => c.query_latest_height().await.unwrap(),
                    QueryHeight::Specific(h) => h,
                };

                data(id::<Hc, Tr, _>(
                    c.chain_id(),
                    SelfConsensusState {
                        self_consensus_state: c.self_consensus_state(height).await,
                        __marker: PhantomData,
                    },
                ))
            }
            Fetch::UpdateHeaders(fetch_update_headers) => {
                Hc::fetch_update_headers(&c, fetch_update_headers)
            }
            Fetch::LightClientSpecific(LightClientSpecificFetch(fetch)) => c.do_fetch(fetch).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use chain_utils::{cosmos::Cosmos, union::Union, wasm::Wasm};

    use super::*;
    use crate::chain::union::UnionFetch;

    #[test]
    fn sanity_check() {
        static_assertions::assert_impl_all!(Union: DoFetchState<Union, Wasm<Cosmos>>);
        static_assertions::assert_impl_all!(UnionFetch<Union, Wasm<Cosmos>>: DoFetch<Union>);
    }
}

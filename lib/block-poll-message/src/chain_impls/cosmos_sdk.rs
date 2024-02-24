use std::{collections::VecDeque, marker::PhantomData, num::NonZeroU64};

use chain_utils::cosmos_sdk::{CosmosSdkChain, CosmosSdkChainExt};
use enumorph::Enumorph;
use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use frunk::{hlist_pat, HList};
use futures::FutureExt;
use protos::cosmos::tx::v1beta1::GetTxsEventRequest;
use queue_msg::{
    aggregate,
    aggregation::{do_aggregate, UseAggregate},
    conc, data, fetch, seq, QueueMsg,
};
use serde::{Deserialize, Serialize};
use unionlabs::{
    events::{
        AcknowledgePacket, ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit, ChannelOpenTry,
        ClientMisbehaviour, ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit,
        ConnectionOpenTry, CreateClient, IbcEvent, RecvPacket, SendPacket, SubmitEvidence,
        TimeoutPacket, UpdateClient, WriteAcknowledgement,
    },
    hash::H256,
    ibc::core::client::height::IsHeight,
    id::ConnectionId,
    option_unwrap, promote,
    tendermint::abci::{event::Event, event_attribute::EventAttribute},
    traits::{ClientIdOf, HeightOf},
};

use crate::{
    aggregate::{AnyAggregate, ChainSpecificAggregate},
    data::{AnyData, ChainEvent, ChainSpecificData, Data},
    fetch::{AnyFetch, ChainSpecificFetch, DoFetch, DoFetchBlockRange, FetchBlockRange},
    AnyChainIdentified, BlockPollingTypes, ChainExt, DoAggregate, Identified, IsAggregateData,
};

pub(crate) trait CosmosSdkChainSealed: CosmosSdkChain {}

impl<C: CosmosSdkChainSealed> ChainExt for C {
    type Data = CosmosSdkData<C>;
    type Fetch = CosmosSdkFetch<C>;
    type Aggregate = CosmosSdkAggregate<C>;
}

const _: fn() = || {
    unionlabs::impl_maybe_arbitrary::<CosmosSdkAggregate<chain_utils::cosmos::Cosmos>>();
};

impl<C: ChainExt<Fetch = CosmosSdkFetch<C>> + CosmosSdkChain> DoFetchBlockRange<C> for C
where
    AnyChainIdentified<AnyFetch>: From<Identified<C, ChainSpecificFetch<C>>>,
    AnyChainIdentified<AnyFetch>: From<Identified<C, FetchBlockRange<C>>>,
{
    fn fetch_block_range(
        c: &C,
        FetchBlockRange {
            from_height,
            to_height,
        }: FetchBlockRange<C>,
    ) -> QueueMsg<BlockPollingTypes> {
        fetch(Identified::new(
            c.chain_id(),
            ChainSpecificFetch::<C>(CosmosSdkFetch::from(FetchBlocks {
                from_height,
                to_height,
            })),
        ))
    }
}

const PER_PAGE_LIMIT: u64 = 10;

impl<C> DoFetch<C> for CosmosSdkFetch<C>
where
    C: CosmosSdkChain
        + ChainExt<
            Fetch = CosmosSdkFetch<C>,
            Aggregate = CosmosSdkAggregate<C>,
            Data = CosmosSdkData<C>,
        >,

    AnyChainIdentified<AnyFetch>: From<Identified<C, ChainSpecificFetch<C>>>,
    AnyChainIdentified<AnyData>: From<Identified<C, ChainSpecificData<C>>>,
    AnyChainIdentified<AnyAggregate>: From<Identified<C, ChainSpecificAggregate<C>>>,
{
    async fn do_fetch(c: &C, this: Self) -> QueueMsg<BlockPollingTypes> {
        match this {
            CosmosSdkFetch::FetchTransactions(FetchTransactions { height, page }) => {
                tracing::info!(%height, %page, "fetching block");

                let response = protos::cosmos::tx::v1beta1::service_client::ServiceClient::connect(
                    c.grpc_url(),
                )
                .await
                .unwrap()
                .get_txs_event(GetTxsEventRequest {
                    query: format!("tx.height = {}", height.revision_height()),
                    page: page.get(),
                    limit: PER_PAGE_LIMIT,
                    ..Default::default()
                })
                .await
                .unwrap()
                .into_inner();

                conc(
                    response
                        .tx_responses
                        .into_iter()
                        // .inspect(|x| {
                        //     dbg!(x);
                        // })
                        .flat_map(|txr| {
                            txr.events.into_iter().filter_map(move |event| {
                                IbcEvent::<ClientIdOf<C>, _, _>::try_from_tendermint_event(Event {
                                    ty: event.r#type,
                                    attributes: event
                                        .attributes
                                        .into_iter()
                                        .map(|attr| EventAttribute {
                                            key: attr.key,
                                            value: attr.value,
                                            index: attr.index,
                                        })
                                        .collect(),
                                })
                                .transpose()
                                .unwrap()
                                .map(|x: IbcEvent<_, _, _>| (x, txr.txhash.clone()))
                            })
                        })
                        .map(|(ibc_event, tx_hash)| {
                            match ibc_event {
                                IbcEvent::SubmitEvidence(SubmitEvidence { .. }) => {
                                    // TODO: Not sure how to handle this one, since it only contains the hash
                                    // union
                                    //     .code_id_of_client_id(client_id)
                                    //     .then(|checksum| union.client_type_of_code_id(checksum))
                                    //     .await
                                    panic!()
                                }
                                IbcEvent::CreateClient(CreateClient { ref client_id, .. })
                                | IbcEvent::UpdateClient(UpdateClient { ref client_id, .. })
                                | IbcEvent::ClientMisbehaviour(ClientMisbehaviour {
                                    ref client_id,
                                    ..
                                })
                                | IbcEvent::ConnectionOpenInit(ConnectionOpenInit {
                                    ref client_id,
                                    ..
                                })
                                | IbcEvent::ConnectionOpenTry(ConnectionOpenTry {
                                    ref client_id,
                                    ..
                                })
                                | IbcEvent::ConnectionOpenAck(ConnectionOpenAck {
                                    ref client_id,
                                    ..
                                })
                                | IbcEvent::ConnectionOpenConfirm(ConnectionOpenConfirm {
                                    ref client_id,
                                    ..
                                }) => aggregate(
                                    [fetch(Identified::new(
                                        c.chain_id(),
                                        ChainSpecificFetch::<C>(CosmosSdkFetch::from(
                                            ClientTypeFromClientId {
                                                client_id: client_id.clone(),
                                            },
                                        )),
                                    ))],
                                    [],
                                    Identified::new(
                                        c.chain_id(),
                                        ChainSpecificAggregate::<C>(CosmosSdkAggregate::from(
                                            AggregateEventWithClientType::<C> {
                                                tx_hash: hex::decode(&tx_hash)
                                                    .unwrap()
                                                    .try_into()
                                                    .unwrap(),
                                                height,
                                                event: ibc_event,
                                            },
                                        )),
                                    ),
                                ),

                                IbcEvent::ChannelOpenInit(ChannelOpenInit {
                                    ref connection_id,
                                    ..
                                })
                                | IbcEvent::ChannelOpenTry(ChannelOpenTry {
                                    ref connection_id,
                                    ..
                                })
                                | IbcEvent::ChannelOpenAck(ChannelOpenAck {
                                    ref connection_id,
                                    ..
                                })
                                | IbcEvent::ChannelOpenConfirm(ChannelOpenConfirm {
                                    ref connection_id,
                                    ..
                                })
                                | IbcEvent::WriteAcknowledgement(WriteAcknowledgement {
                                    ref connection_id,
                                    ..
                                })
                                | IbcEvent::RecvPacket(RecvPacket {
                                    ref connection_id, ..
                                })
                                | IbcEvent::SendPacket(SendPacket {
                                    ref connection_id, ..
                                })
                                | IbcEvent::AcknowledgePacket(AcknowledgePacket {
                                    ref connection_id,
                                    ..
                                })
                                | IbcEvent::TimeoutPacket(TimeoutPacket {
                                    ref connection_id,
                                    ..
                                }) => aggregate(
                                    [fetch(Identified::new(
                                        c.chain_id(),
                                        ChainSpecificFetch(
                                            CosmosSdkFetch::FetchClientTypeFromConnectionId(
                                                ClientTypeFromConnectionId {
                                                    connection_id: connection_id.clone(),
                                                },
                                            ),
                                        ),
                                    ))],
                                    [],
                                    Identified::new(
                                        c.chain_id(),
                                        ChainSpecificAggregate::<C>(CosmosSdkAggregate::from(
                                            AggregateEventWithClientType {
                                                tx_hash: hex::decode(&tx_hash)
                                                    .unwrap()
                                                    .try_into()
                                                    .unwrap(),
                                                height,
                                                event: ibc_event,
                                            },
                                        )),
                                    ),
                                ),
                            }
                        })
                        .chain(((page.get() * PER_PAGE_LIMIT) < response.total).then(|| {
                            queue_msg::fetch(Identified::new(
                                c.chain_id(),
                                ChainSpecificFetch(CosmosSdkFetch::from(FetchTransactions {
                                    height,
                                    page: page.checked_add(1).unwrap(),
                                })),
                            ))
                        })),
                )
            }
            CosmosSdkFetch::FetchClientTypeFromConnectionId(ClientTypeFromConnectionId {
                connection_id,
            }) => fetch(Identified::new(
                c.chain_id(),
                ChainSpecificFetch(CosmosSdkFetch::FetchClientTypeFromClientId(
                    ClientTypeFromClientId {
                        client_id: c.client_id_of_connection(connection_id.clone()).await,
                    },
                )),
            )),
            CosmosSdkFetch::FetchClientTypeFromClientId(ClientTypeFromClientId { client_id }) => {
                data(Identified::new(
                    c.chain_id(),
                    ChainSpecificData::<C>(CosmosSdkData::ClientType(ClientType {
                        client_type: match client_id.to_string().rsplit_once('-').unwrap().0 {
                            "07-tendermint" => unionlabs::ClientType::Tendermint,
                            "08-wasm" => unionlabs::ClientType::Wasm(
                                c.checksum_of_client_id(client_id)
                                    .then(|checksum| c.client_type_of_checksum(checksum))
                                    .await,
                            ),
                            ty => panic!("unsupported client type {ty}"),
                        },
                        __marker: PhantomData,
                    })),
                ))
            }
            CosmosSdkFetch::FetchBlocks(FetchBlocks {
                from_height,
                to_height,
            }) => {
                assert!(from_height.revision_height() < to_height.revision_height());

                let new_from_height = from_height.increment();

                if to_height.revision_height() - from_height.revision_height() == 1 {
                    fetch(Identified::new(
                        c.chain_id(),
                        ChainSpecificFetch::<C>(
                            FetchTransactions {
                                height: from_height,
                                // who needs const blocks
                                page: promote!(NonZeroU64: option_unwrap!(NonZeroU64::new(1_u64))),
                            }
                            .into(),
                        ),
                    ))
                } else {
                    // is exclusive on `to`, so fetch the `from` block and "discard" the `to` block
                    // the assumption is that another message with `to..N` will be queued, which then following
                    // this logic will fetch `to`.
                    seq([fetch(Identified::new(
                        c.chain_id(),
                        ChainSpecificFetch::<C>(
                            FetchTransactions {
                                height: from_height,
                                // who needs const blocks
                                page: promote!(NonZeroU64: option_unwrap!(NonZeroU64::new(1_u64))),
                            }
                            .into(),
                        ),
                    ))]
                    .into_iter()
                    .chain((new_from_height != to_height).then(|| {
                        fetch(Identified::new(
                            c.chain_id(),
                            ChainSpecificFetch::<C>(CosmosSdkFetch::from(FetchBlocks {
                                from_height: new_from_height,
                                to_height,
                            })),
                        ))
                    })))
                }
            }
        }
    }
}

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: CosmosSdkChain + ChainExt")
)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub enum CosmosSdkData<C: CosmosSdkChain + ChainExt> {
    #[display(fmt = "ClientType")]
    ClientType(ClientType<C>),
}

const _: () = {
    try_from_block_poll_msg! {
        chain = C,
        generics = (C: CosmosSdkChain + ChainExt),
        msgs = CosmosSdkData(
            ClientType(ClientType<C>),
        ),
        where = (C: ChainExt<Data = CosmosSdkData<C>>)
    }
};

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: CosmosSdkChain + ChainExt")
)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub struct ClientType<C: CosmosSdkChain + ChainExt> {
    pub client_type: unionlabs::ClientType,
    #[serde(skip)]
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    pub __marker: PhantomData<fn() -> C>,
}

// FETCH

#[derive(
    DebugNoBound,
    CloneNoBound,
    PartialEqNoBound,
    Serialize,
    Deserialize,
    derive_more::Display,
    Enumorph,
)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: CosmosSdkChain + ChainExt")
)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub enum CosmosSdkFetch<C: CosmosSdkChain + ChainExt> {
    #[display(fmt = "FetchBlocks({}..{})", "_0.from_height", "_0.to_height")]
    FetchBlocks(FetchBlocks<C>),
    #[display(fmt = "FetchTransactions({}, {})", "_0.height", "_0.page")]
    FetchTransactions(FetchTransactions<C>),
    #[display(fmt = "ClientTypeFromConnectionId({})", "_0.connection_id")]
    FetchClientTypeFromConnectionId(ClientTypeFromConnectionId),
    #[display(fmt = "ClientTypeFromClientId({})", "_0.client_id")]
    FetchClientTypeFromClientId(ClientTypeFromClientId<C>),
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: CosmosSdkChain + ChainExt")
)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub struct FetchBlocks<C: CosmosSdkChain + ChainExt> {
    from_height: HeightOf<C>,
    to_height: HeightOf<C>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: CosmosSdkChain + ChainExt")
)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub struct FetchTransactions<C: CosmosSdkChain + ChainExt> {
    height: HeightOf<C>,
    page: NonZeroU64,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub struct ClientTypeFromConnectionId {
    connection_id: ConnectionId,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: CosmosSdkChain + ChainExt")
)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub struct ClientTypeFromClientId<C: CosmosSdkChain + ChainExt> {
    client_id: C::ClientId,
}

#[derive(
    DebugNoBound,
    CloneNoBound,
    PartialEqNoBound,
    Serialize,
    Deserialize,
    derive_more::Display,
    Enumorph,
)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: CosmosSdkChain")
)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub enum CosmosSdkAggregate<C: CosmosSdkChain> {
    #[display(fmt = "AggregateEventWithClientType")]
    AggregateEventWithClientType(AggregateEventWithClientType<C>),
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: CosmosSdkChain")
)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub struct AggregateEventWithClientType<C: CosmosSdkChain> {
    pub tx_hash: H256,
    pub height: C::Height,
    pub event: IbcEvent<C::ClientId, C::ClientType, String>,
}

impl<C> UseAggregate<BlockPollingTypes> for Identified<C, AggregateEventWithClientType<C>>
where
    C: CosmosSdkChain + ChainExt,
    Identified<C, ClientType<C>>: IsAggregateData,
    AnyChainIdentified<AnyData>: From<Identified<C, Data<C>>>,
{
    type AggregatedData = HList![
        Identified<C, ClientType<C>>,
    ];

    fn aggregate(
        Identified {
            chain_id,
            t:
                AggregateEventWithClientType {
                    tx_hash,
                    height,
                    event,
                },
        }: Self,
        hlist_pat![Identified {
            chain_id: client_type_chain_id,
            t: ClientType {
                client_type,
                __marker: _
            },
        }]: Self::AggregatedData,
    ) -> QueueMsg<BlockPollingTypes> {
        assert_eq!(chain_id, client_type_chain_id);

        data(Identified::<C, _>::new(
            chain_id,
            ChainEvent {
                client_type,
                tx_hash,
                height,
                event,
            },
        ))
    }
}

impl<C> DoAggregate for Identified<C, CosmosSdkAggregate<C>>
where
    C: ChainExt + CosmosSdkChain,

    Identified<C, ClientType<C>>: IsAggregateData,

    Identified<C, AggregateEventWithClientType<C>>: UseAggregate<BlockPollingTypes>,
    AnyChainIdentified<AnyData>: From<Identified<C, Data<C>>>,
{
    fn do_aggregate(
        Identified { chain_id, t: data }: Self,
        aggregate_data: VecDeque<AnyChainIdentified<AnyData>>,
    ) -> QueueMsg<BlockPollingTypes> {
        match data {
            CosmosSdkAggregate::AggregateEventWithClientType(data) => {
                do_aggregate(Identified::<C, _>::new(chain_id, data), aggregate_data)
            }
        }
    }
}

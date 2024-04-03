use std::{collections::VecDeque, marker::PhantomData, num::NonZeroU32};

use chain_utils::cosmos_sdk::{CosmosSdkChain, CosmosSdkChainExt};
use frunk::{hlist_pat, HList};
use futures::FutureExt;
use queue_msg::{
    aggregate,
    aggregation::{do_aggregate, UseAggregate},
    conc, data, fetch, queue_msg, QueueMsg,
};
use tendermint_rpc::Client;
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
    aggregate::{Aggregate, AnyAggregate},
    data::{AnyData, ChainEvent, Data},
    fetch::{AnyFetch, DoFetch, DoFetchBlockRange, Fetch, FetchBlockRange},
    id, AnyChainIdentified, BlockMessageTypes, ChainExt, DoAggregate, Identified, IsAggregateData,
};

pub trait CosmosSdkChainSealed: CosmosSdkChain + ChainExt {}

impl<C: CosmosSdkChainSealed> ChainExt for C {
    type Data = CosmosSdkData<C>;
    type Fetch = CosmosSdkFetch<C>;
    type Aggregate = CosmosSdkAggregate<C>;
}

const _: fn() = || {
    unionlabs::impl_maybe_arbitrary::<CosmosSdkAggregate<chain_utils::cosmos::Cosmos>>();
};

impl<C: CosmosSdkChainSealed<Fetch = CosmosSdkFetch<C>>> DoFetchBlockRange<C> for C
where
    AnyChainIdentified<AnyFetch>: From<Identified<C, Fetch<C>>>,
{
    fn fetch_block_range(
        c: &C,
        FetchBlockRange {
            from_height,
            to_height,
        }: FetchBlockRange<C>,
    ) -> QueueMsg<BlockMessageTypes> {
        fetch(id(
            c.chain_id(),
            Fetch::<C>::specific(FetchBlocks {
                from_height,
                to_height,
            }),
        ))
    }
}

const PER_PAGE_LIMIT: u8 = 10;

impl<C> DoFetch<C> for CosmosSdkFetch<C>
where
    C: CosmosSdkChainSealed<
        Fetch = CosmosSdkFetch<C>,
        Aggregate = CosmosSdkAggregate<C>,
        Data = CosmosSdkData<C>,
    >,

    AnyChainIdentified<AnyFetch>: From<Identified<C, Fetch<C>>>,
    AnyChainIdentified<AnyData>: From<Identified<C, Data<C>>>,
    AnyChainIdentified<AnyAggregate>: From<Identified<C, Aggregate<C>>>,
{
    async fn do_fetch(c: &C, this: Self) -> QueueMsg<BlockMessageTypes> {
        match this {
            CosmosSdkFetch::FetchTransactions(FetchTransactions { height, page }) => {
                tracing::info!(%height, %page, "fetching block");

                let response = c
                    .tm_client()
                    .tx_search(
                        tendermint_rpc::query::Query::eq("tx.height", height.revision_height()),
                        false,
                        page.get(),
                        PER_PAGE_LIMIT,
                        tendermint_rpc::Order::Descending,
                    )
                    .await
                    .unwrap();

                conc(
                    response
                        .txs
                        .into_iter()
                        // .inspect(|x| {
                        //     dbg!(x);
                        // })
                        .flat_map(|txr| {
                            txr.tx_result.events.into_iter().filter_map(move |event| {
                                IbcEvent::<ClientIdOf<C>, _, _>::try_from_tendermint_event(Event {
                                    ty: event.kind,
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
                                .map(|x: IbcEvent<_, _, _>| (x, txr.hash))
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
                                    [fetch(id(
                                        c.chain_id(),
                                        Fetch::<C>::specific(ClientTypeFromClientId {
                                            client_id: client_id.clone(),
                                        }),
                                    ))],
                                    [],
                                    id(
                                        c.chain_id(),
                                        Aggregate::<C>::specific(
                                            AggregateEventWithClientType::<C> {
                                                tx_hash: tendermint_hash_to_h256(tx_hash),
                                                height,
                                                event: ibc_event,
                                            },
                                        ),
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
                                    [fetch(id(
                                        c.chain_id(),
                                        Fetch::specific(ClientTypeFromConnectionId {
                                            connection_id: connection_id.clone(),
                                        }),
                                    ))],
                                    [],
                                    id(
                                        c.chain_id(),
                                        Aggregate::<C>::specific(AggregateEventWithClientType {
                                            tx_hash: tendermint_hash_to_h256(tx_hash),
                                            height,
                                            event: ibc_event,
                                        }),
                                    ),
                                ),
                            }
                        })
                        .chain(
                            ((page.get() * PER_PAGE_LIMIT as u32) < response.total_count).then(
                                || {
                                    queue_msg::fetch(id(
                                        c.chain_id(),
                                        Fetch::specific(FetchTransactions {
                                            height,
                                            page: page.checked_add(1).unwrap(),
                                        }),
                                    ))
                                },
                            ),
                        ),
                )
            }
            CosmosSdkFetch::FetchClientTypeFromConnectionId(ClientTypeFromConnectionId {
                connection_id,
            }) => fetch(id(
                c.chain_id(),
                Fetch::specific(ClientTypeFromClientId {
                    client_id: c.client_id_of_connection(connection_id.clone()).await,
                }),
            )),
            CosmosSdkFetch::FetchClientTypeFromClientId(ClientTypeFromClientId { client_id }) => {
                data(id(
                    c.chain_id(),
                    Data::<C>::specific(ClientType {
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
                    }),
                ))
            }
            CosmosSdkFetch::FetchBlocks(FetchBlocks {
                from_height,
                to_height,
            }) => {
                assert!(from_height.revision_height() < to_height.revision_height());

                let new_from_height = from_height.increment();

                if to_height.revision_height() - from_height.revision_height() == 1 {
                    fetch(id(
                        c.chain_id(),
                        Fetch::<C>::specific(FetchTransactions {
                            height: from_height,
                            // who needs const blocks
                            page: promote!(NonZeroU32: option_unwrap!(NonZeroU32::new(1_u32))),
                        }),
                    ))
                } else {
                    // is exclusive on `to`, so fetch the `from` block and "discard" the `to` block
                    // the assumption is that another message with `to..N` will be queued, which then following
                    // this logic will fetch `to`.
                    conc(
                        [fetch(id(
                            c.chain_id(),
                            Fetch::<C>::specific(FetchTransactions {
                                height: from_height,
                                // who needs const blocks
                                page: promote!(NonZeroU32: option_unwrap!(NonZeroU32::new(1_u32))),
                            }),
                        ))]
                        .into_iter()
                        .chain((new_from_height != to_height).then(|| {
                            fetch(id(
                                c.chain_id(),
                                Fetch::<C>::specific(FetchBlocks {
                                    from_height: new_from_height,
                                    to_height,
                                }),
                            ))
                        })),
                    )
                }
            }
        }
    }
}

#[queue_msg]
#[derive(enumorph::Enumorph)]
pub enum CosmosSdkData<C: CosmosSdkChainSealed> {
    ClientType(ClientType<C>),
}

const _: () = {
    try_from_block_poll_msg! {
        chain = C,
        generics = (C: CosmosSdkChainSealed),
        msgs = CosmosSdkData(
            ClientType(ClientType<C>),
        ),
        where = (C: ChainExt<Data = CosmosSdkData<C>>)
    }
};

#[queue_msg]
pub struct ClientType<#[cover] C: CosmosSdkChainSealed> {
    pub client_type: unionlabs::ClientType,
}

// FETCH

#[queue_msg]
#[derive(enumorph::Enumorph)]
pub enum CosmosSdkFetch<C: CosmosSdkChainSealed> {
    FetchBlocks(FetchBlocks<C>),
    FetchTransactions(FetchTransactions<C>),
    FetchClientTypeFromConnectionId(ClientTypeFromConnectionId),
    FetchClientTypeFromClientId(ClientTypeFromClientId<C>),
}

#[queue_msg]
pub struct FetchBlocks<C: CosmosSdkChainSealed> {
    pub from_height: HeightOf<C>,
    pub to_height: HeightOf<C>,
}

#[queue_msg]
pub struct FetchTransactions<C: CosmosSdkChain> {
    pub height: HeightOf<C>,
    pub page: NonZeroU32,
}

#[queue_msg]
pub struct ClientTypeFromConnectionId {
    pub connection_id: ConnectionId,
}

#[queue_msg]
pub struct ClientTypeFromClientId<C: CosmosSdkChain> {
    pub client_id: C::ClientId,
}

#[queue_msg]
#[derive(enumorph::Enumorph)]
pub enum CosmosSdkAggregate<C: CosmosSdkChain> {
    AggregateEventWithClientType(AggregateEventWithClientType<C>),
}

#[queue_msg]
pub struct AggregateEventWithClientType<C: CosmosSdkChain> {
    pub tx_hash: H256,
    pub height: C::Height,
    pub event: IbcEvent<C::ClientId, C::ClientType, String>,
}

impl<C> UseAggregate<BlockMessageTypes> for Identified<C, AggregateEventWithClientType<C>>
where
    C: CosmosSdkChainSealed,
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
    ) -> QueueMsg<BlockMessageTypes> {
        assert_eq!(chain_id, client_type_chain_id);

        data(id(
            chain_id,
            ChainEvent {
                client_type,
                tx_hash,
                // don't ask
                height: height.increment(),
                event,
            },
        ))
    }
}

impl<C> DoAggregate for Identified<C, CosmosSdkAggregate<C>>
where
    C: CosmosSdkChainSealed,

    Identified<C, ClientType<C>>: IsAggregateData,

    Identified<C, AggregateEventWithClientType<C>>: UseAggregate<BlockMessageTypes>,
    AnyChainIdentified<AnyData>: From<Identified<C, Data<C>>>,
{
    fn do_aggregate(
        Identified { chain_id, t: data }: Self,
        aggregate_data: VecDeque<AnyChainIdentified<AnyData>>,
    ) -> QueueMsg<BlockMessageTypes> {
        match data {
            CosmosSdkAggregate::AggregateEventWithClientType(data) => {
                do_aggregate(id(chain_id, data), aggregate_data)
            }
        }
    }
}

fn tendermint_hash_to_h256(hash: tendermint::Hash) -> H256 {
    match hash {
        tendermint::Hash::Sha256(hash) => hash.into(),
        tendermint::Hash::None => panic!("empty hash???"),
    }
}

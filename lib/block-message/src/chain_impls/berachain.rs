use std::collections::VecDeque;

use chain_utils::{
    berachain::Berachain,
    ethereum::{EthereumConsensusChain, IBCHandlerEvents},
};
use enumorph::Enumorph;
use ethers::{contract::EthLogDecode, providers::Middleware, types::Filter};
use futures::StreamExt;
use queue_msg::{aggregation::do_aggregate, conc, fetch, noop, queue_msg, QueueMsg};
use tracing::{debug, info, warn};
use unionlabs::{
    ibc::core::client::height::IsHeight,
    traits::{Chain, HeightOf},
};

use crate::{
    aggregate::{Aggregate, AnyAggregate},
    chain_impls::ethereum::{
        fetch_channel, mk_aggregate_event, AggregateWithChannel, ChannelData, ConnectionData,
        FetchChannel,
    },
    data::{AnyData, ChainEvent, Data},
    fetch::{AnyFetch, DoFetch, DoFetchBlockRange, Fetch, FetchBlockRange},
    id, AnyChainIdentified, BlockMessageTypes, ChainExt, DoAggregate, Identified, IsAggregateData,
};

impl ChainExt for Berachain {
    type Data = BerachainData;
    type Fetch = BerachainFetch;
    type Aggregate = BerachainAggregate;
}

impl DoFetchBlockRange<Berachain> for Berachain
where
    AnyChainIdentified<AnyFetch>: From<Identified<Berachain, Fetch<Berachain>>>,
{
    fn fetch_block_range(
        c: &Berachain,
        range: FetchBlockRange<Berachain>,
    ) -> QueueMsg<BlockMessageTypes> {
        fetch(id(
            c.chain_id(),
            Fetch::<Berachain>::specific(FetchBlocks {
                from_height: range.from_height,
                to_height: range.to_height,
            }),
        ))
    }
}

impl DoFetch<Berachain> for BerachainFetch
where
    AnyChainIdentified<AnyData>: From<Identified<Berachain, Data<Berachain>>>,
    AnyChainIdentified<AnyAggregate>: From<Identified<Berachain, Aggregate<Berachain>>>,
    AnyChainIdentified<AnyFetch>: From<Identified<Berachain, Fetch<Berachain>>>,
{
    async fn do_fetch(c: &Berachain, msg: Self) -> QueueMsg<BlockMessageTypes> {
        match msg {
            Self::FetchBlockEvents(FetchBlockEvents { height }) => {
                info!(%height, "fetching block events");

                let block_number = c
                    .execution_height_of_beacon_slot(height.revision_height)
                    .await;

                debug!("cometbft height {height} is evm block number {block_number}");

                if height.revision_height > 1 {
                    let previous_block_number = c
                        .execution_height_of_beacon_slot(height.revision_height - 1)
                        .await;

                    if block_number == previous_block_number {
                        debug!(slot = %height.revision_height, "slot was missed");
                        return noop();
                    }
                }

                conc(
                    futures::stream::iter(
                        c.provider
                            .get_logs(
                                &Filter::new()
                                    .address(ethers::types::H160::from(c.ibc_handler_address))
                                    .from_block(block_number)
                                    .to_block(block_number),
                            )
                            .await
                            .unwrap(),
                    )
                    .filter_map(|log| async {
                        let tx_hash = log
                            .transaction_hash
                            .expect("log should have transaction_hash")
                            .into();

                        debug!(?log, "raw log");

                        match IBCHandlerEvents::decode_log(&log.into()) {
                            Ok(event) => {
                                debug!(?event, "found IBCHandler event");
                                Some(
                                    mk_aggregate_event(c, event, height, tx_hash, |event_height| {
                                        c.execution_height_of_beacon_slot(
                                            event_height.revision_height,
                                        )
                                    })
                                    .await,
                                )
                            }
                            Err(e) => {
                                warn!("could not decode evm event {}", e);
                                None
                            }
                        }
                    })
                    .collect::<Vec<_>>()
                    .await,
                )
            }
            Self::FetchBlocks(FetchBlocks {
                from_height,
                to_height,
            }) => {
                assert!(from_height.revision_height() < to_height.revision_height());

                if to_height.revision_height() - from_height.revision_height() == 1 {
                    fetch(id(
                        c.chain_id(),
                        Fetch::<Berachain>::specific(FetchBlockEvents {
                            height: from_height,
                        }),
                    ))
                } else {
                    // this is exclusive on `to`, so fetch the `from` block and "discard" the `to` block
                    // the assumption is that another message with `to..N` will be queued, which then following
                    // this logic will fetch `to`.

                    let new_from_height = from_height.increment();

                    conc(
                        [fetch(id(
                            c.chain_id(),
                            Fetch::<Berachain>::specific(FetchBlockEvents {
                                height: from_height,
                            }),
                        ))]
                        .into_iter()
                        .chain((new_from_height != to_height).then(|| {
                            fetch(id(
                                c.chain_id(),
                                Fetch::<Berachain>::specific(FetchBlocks {
                                    from_height: new_from_height,
                                    to_height,
                                }),
                            ))
                        })),
                    )
                }
            }
            Self::FetchChannel(FetchChannel { height, path }) => {
                fetch_channel(
                    c,
                    path,
                    c.execution_height_of_beacon_slot(height.revision_height())
                        .await,
                )
                .await
            }
        }
    }
}

#[queue_msg]
pub struct FetchBlockEvents {
    pub height: HeightOf<Berachain>,
}

#[queue_msg]
pub struct FetchBlocks {
    pub from_height: HeightOf<Berachain>,
    pub to_height: HeightOf<Berachain>,
}

#[queue_msg]
#[derive(Enumorph)]
pub enum BerachainFetch {
    FetchBlocks(FetchBlocks),
    FetchBlockEvents(FetchBlockEvents),

    FetchChannel(FetchChannel<Berachain>),
}

#[queue_msg]
#[derive(Enumorph)]
pub enum BerachainAggregate {
    AggregateWithChannel(AggregateWithChannel<Berachain>),
}

impl DoAggregate for Identified<Berachain, BerachainAggregate>
where
    AnyChainIdentified<AnyData>: From<Identified<Berachain, ChainEvent<Berachain>>>,

    Identified<Berachain, ChannelData<Berachain>>: IsAggregateData,
    Identified<Berachain, ConnectionData<Berachain>>: IsAggregateData,
{
    fn do_aggregate(
        Identified { chain_id, t }: Self,
        data: VecDeque<AnyChainIdentified<AnyData>>,
    ) -> QueueMsg<BlockMessageTypes> {
        match t {
            BerachainAggregate::AggregateWithChannel(msg) => do_aggregate(id(chain_id, msg), data),
        }
    }
}

#[queue_msg]
#[derive(Enumorph)]
pub enum BerachainData {
    Channel(ChannelData<Berachain>),
    Connection(ConnectionData<Berachain>),
}

const _: () = {
    try_from_block_poll_msg! {
        chain = Berachain,
        generics = (),
        msgs = BerachainData(
            Channel(ChannelData<Berachain>),
            Connection(ConnectionData<Berachain>),
        ),
    }
};

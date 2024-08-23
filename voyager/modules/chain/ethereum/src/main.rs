use std::{collections::VecDeque, sync::Arc};

use beacon_api::client::BeaconApiClient;
use chain_utils::ethereum::{IBCHandlerEvents, IbcHandlerExt};
use contracts::{
    ibc_channel_handshake::IBCChannelHandshakeEvents, ibc_client::IBCClientEvents,
    ibc_connection::IBCConnectionEvents, ibc_handler::IBCHandler, ibc_packet::IBCPacketEvents,
};
use ethers::{
    contract::EthLogDecode,
    providers::{Middleware, Provider, ProviderError, Ws, WsClientError},
    types::Filter,
};
use futures::{stream::FuturesUnordered, TryStreamExt};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
};
use queue_msg::{promise, aggregation::do_aggregate, conc, call, noop, BoxDynError, Op};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, error, info, instrument, warn};
use unionlabs::{
    ethereum::{ibc_commitment_key, IBC_HANDLER_COMMITMENTS_SLOT},
    hash::{H160, H256},
    ibc::{core::client::height::Height, lightclients::ethereum::storage_proof::StorageProof},
    ics24::{ChannelEndPath, ClientStatePath, ConnectionPath, Path},
    id::ClientId,
    uint::U256,
    ErrorReporter, QueryHeight,
};
use voyager_message::{
    callback::{
        Callback, AggregateDecodeClientStateMetaFromConnection, AggregateFetchClientFromChannel,
        AggregateFetchClientFromConnection, AggregateFetchCounterpartyChannelAndConnection,
        AggregateFetchCounterpartyChannelAndConnectionFromSourceChannel, InfoOrMeta,
    },
    data::{ClientInfo, Data},
    call::{
        compound::{fetch_client_state_meta, fetch_connection_from_channel_info},
        Call, FetchClientInfo, FetchState,
    },
    plugin::{ChainModuleServer, PluginInfo, PluginKind, PluginModuleServer, RawClientState},
    run_module_server, ClientType, IbcInterface, VoyagerMessage,
};

use crate::{
    aggregate::{EventInfo, ModuleAggregate},
    data::ModuleData,
    fetch::{FetchBeaconBlockRange, FetchEvents, FetchGetLogs, ModuleFetch},
};

pub mod aggregate;
pub mod data;
pub mod fetch;

const ETHEREUM_REVISION_NUMBER: u64 = 0;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    run_module_server(Module::new, ChainModuleServer::into_rpc).await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: U256,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    pub provider: Provider<Ws>,
    pub beacon_api_client: BeaconApiClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    /// The RPC endpoint for the execution chain.
    pub eth_rpc_api: String,
    /// The RPC endpoint for the beacon chain.
    pub eth_beacon_rpc_api: String,
}

impl Module {
    fn plugin_name(&self) -> String {
        pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

        format!("{PLUGIN_NAME}/{}", self.chain_id)
    }

    pub async fn new(config: Config) -> Result<Self, InitError> {
        let provider = Provider::new(Ws::connect(config.eth_rpc_api).await?);

        let chain_id = provider.get_chainid().await?;

        Ok(Self {
            chain_id: U256(chain_id),
            ibc_handler_address: config.ibc_handler_address,
            provider,
            beacon_api_client: BeaconApiClient::new(config.eth_beacon_rpc_api).await?,
        })
    }

    #[must_use]
    pub fn make_height(&self, height: u64) -> Height {
        Height {
            revision_number: ETHEREUM_REVISION_NUMBER,
            revision_height: height,
        }
    }

    pub fn mk_aggregate_event(
        &self,
        event: IBCHandlerEvents,
        event_height: Height,
        tx_hash: H256,
    ) -> Op<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>> {
        let channel_fetch = |port_id: &str, channel_id: &str, height| {
            call(FetchState {
                path: ChannelEndPath {
                    port_id: port_id.parse().unwrap(),
                    channel_id: channel_id.parse().unwrap(),
                }
                .into(),
                chain_id: self.chain_id.to_string(),
                at: QueryHeight::Specific(height),
            })
        };

        let connection_fetch = |connection_id: &str, height| {
            call(FetchState {
                path: ConnectionPath {
                    connection_id: connection_id.parse().unwrap(),
                }
                .into(),
                chain_id: self.chain_id.to_string(),
                at: QueryHeight::Specific(height),
            })
        };

        match event {
            IBCHandlerEvents::ChannelEvent(
                IBCChannelHandshakeEvents::ChannelCloseConfirmFilter(_),
            )
            | IBCHandlerEvents::ChannelEvent(IBCChannelHandshakeEvents::ChannelCloseInitFilter(
                _,
            )) => {
                todo!()
            }

            IBCHandlerEvents::ChannelEvent(IBCChannelHandshakeEvents::ChannelOpenInitFilter(
                raw_event,
            )) => promise(
                [
                    connection_fetch(&raw_event.connection_id, event_height),
                    promise(
                        [connection_fetch(&raw_event.connection_id, event_height)],
                        [],
                        AggregateFetchClientFromConnection {
                            fetch_type: InfoOrMeta::Both,
                        },
                    ),
                ],
                [],
                Callback::plugin(
                    self.plugin_name(),
                    EventInfo {
                        chain_id: self.chain_id.to_string(),
                        height: event_height,
                        tx_hash,
                        raw_event,
                    },
                ),
            ),
            IBCHandlerEvents::ChannelEvent(IBCChannelHandshakeEvents::ChannelOpenTryFilter(
                raw_event,
            )) => promise(
                [
                    connection_fetch(&raw_event.connection_id, event_height),
                    promise(
                        [connection_fetch(&raw_event.connection_id, event_height)],
                        [],
                        AggregateFetchClientFromConnection {
                            fetch_type: InfoOrMeta::Both,
                        },
                    ),
                ],
                [],
                Callback::plugin(
                    self.plugin_name(),
                    EventInfo {
                        chain_id: self.chain_id.to_string(),
                        height: event_height,
                        tx_hash,
                        raw_event,
                    },
                ),
            ),
            IBCHandlerEvents::ChannelEvent(IBCChannelHandshakeEvents::ChannelOpenAckFilter(
                raw_event,
            )) => promise(
                [
                    channel_fetch(&raw_event.port_id, &raw_event.channel_id, event_height),
                    connection_fetch(&raw_event.connection_id, event_height),
                    promise(
                        [connection_fetch(&raw_event.connection_id, event_height)],
                        [],
                        AggregateFetchClientFromConnection {
                            fetch_type: InfoOrMeta::Both,
                        },
                    ),
                ],
                [],
                Callback::plugin(
                    self.plugin_name(),
                    EventInfo {
                        chain_id: self.chain_id.to_string(),
                        height: event_height,
                        tx_hash,
                        raw_event,
                    },
                ),
            ),
            IBCHandlerEvents::ChannelEvent(
                IBCChannelHandshakeEvents::ChannelOpenConfirmFilter(raw_event),
            ) => promise(
                [
                    channel_fetch(&raw_event.port_id, &raw_event.channel_id, event_height),
                    connection_fetch(&raw_event.connection_id, event_height),
                    promise(
                        [connection_fetch(&raw_event.connection_id, event_height)],
                        [],
                        AggregateFetchClientFromConnection {
                            fetch_type: InfoOrMeta::Both,
                        },
                    ),
                ],
                [],
                Callback::plugin(
                    self.plugin_name(),
                    EventInfo {
                        chain_id: self.chain_id.to_string(),
                        height: event_height,
                        tx_hash,
                        raw_event,
                    },
                ),
            ),

            IBCHandlerEvents::ConnectionEvent(IBCConnectionEvents::ConnectionOpenInitFilter(
                raw_event,
            )) => promise(
                [promise(
                    [connection_fetch(&raw_event.connection_id, event_height)],
                    [],
                    AggregateFetchClientFromConnection {
                        fetch_type: InfoOrMeta::Both,
                    },
                )],
                [],
                Callback::plugin(
                    self.plugin_name(),
                    EventInfo {
                        chain_id: self.chain_id.to_string(),
                        height: event_height,
                        tx_hash,
                        raw_event,
                    },
                ),
            ),
            IBCHandlerEvents::ConnectionEvent(IBCConnectionEvents::ConnectionOpenTryFilter(
                raw_event,
            )) => promise(
                [promise(
                    [connection_fetch(&raw_event.connection_id, event_height)],
                    [],
                    AggregateFetchClientFromConnection {
                        fetch_type: InfoOrMeta::Both,
                    },
                )],
                [],
                Callback::plugin(
                    self.plugin_name(),
                    EventInfo {
                        chain_id: self.chain_id.to_string(),
                        height: event_height,
                        tx_hash,
                        raw_event,
                    },
                ),
            ),
            IBCHandlerEvents::ConnectionEvent(IBCConnectionEvents::ConnectionOpenAckFilter(
                raw_event,
            )) => promise(
                [promise(
                    [connection_fetch(&raw_event.connection_id, event_height)],
                    [],
                    AggregateFetchClientFromConnection {
                        fetch_type: InfoOrMeta::Both,
                    },
                )],
                [],
                Callback::plugin(
                    self.plugin_name(),
                    EventInfo {
                        chain_id: self.chain_id.to_string(),
                        height: event_height,
                        tx_hash,
                        raw_event,
                    },
                ),
            ),
            IBCHandlerEvents::ConnectionEvent(
                IBCConnectionEvents::ConnectionOpenConfirmFilter(raw_event),
            ) => promise(
                [promise(
                    [connection_fetch(&raw_event.connection_id, event_height)],
                    [],
                    AggregateFetchClientFromConnection {
                        fetch_type: InfoOrMeta::Both,
                    },
                )],
                [],
                Callback::plugin(
                    self.plugin_name(),
                    EventInfo {
                        chain_id: self.chain_id.to_string(),
                        height: event_height,
                        tx_hash,
                        raw_event,
                    },
                ),
            ),

            IBCHandlerEvents::ClientEvent(IBCClientEvents::ClientCreatedFilter(raw_event)) => {
                promise(
                    [
                        fetch_client_state_meta(
                            self.chain_id.to_string(),
                            raw_event.client_id.parse().unwrap(),
                            QueryHeight::Specific(event_height),
                        ),
                        call(FetchClientInfo {
                            chain_id: self.chain_id.to_string(),
                            client_id: raw_event.client_id.parse().unwrap(),
                        }),
                    ],
                    [],
                    Callback::plugin(
                        self.plugin_name(),
                        EventInfo {
                            chain_id: self.chain_id.to_string(),
                            height: event_height,
                            tx_hash,
                            raw_event,
                        },
                    ),
                )
            }
            IBCHandlerEvents::ClientEvent(IBCClientEvents::ClientRegisteredFilter(raw_event)) => {
                info!(?raw_event, "observed ClientRegistered event");

                noop()
            }
            IBCHandlerEvents::ClientEvent(IBCClientEvents::ClientUpdatedFilter(raw_event)) => {
                promise(
                    [
                        fetch_client_state_meta(
                            self.chain_id.to_string(),
                            raw_event.client_id.parse().unwrap(),
                            QueryHeight::Specific(event_height),
                        ),
                        call(FetchClientInfo {
                            chain_id: self.chain_id.to_string(),
                            client_id: raw_event.client_id.parse().unwrap(),
                        }),
                    ],
                    [],
                    Callback::plugin(
                        self.plugin_name(),
                        EventInfo {
                            chain_id: self.chain_id.to_string(),
                            height: event_height,
                            tx_hash,
                            raw_event,
                        },
                    ),
                )
            }
            IBCHandlerEvents::PacketEvent(IBCPacketEvents::RecvPacketFilter(raw_event)) => {
                promise(
                    [
                        // client info of the client underlying this connection on this chain
                        promise(
                            [channel_fetch(
                                &raw_event.packet.destination_port,
                                &raw_event.packet.destination_channel,
                                event_height,
                            )],
                            [],
                            AggregateFetchClientFromChannel {
                                fetch_type: InfoOrMeta::Both,
                            },
                        ),
                        // channel on this chain
                        channel_fetch(
                            &raw_event.packet.destination_port,
                            &raw_event.packet.destination_channel,
                            event_height,
                        ),
                        // connection on this chain
                        fetch_connection_from_channel_info(
                            self.chain_id.to_string(),
                            QueryHeight::Specific(event_height),
                            raw_event.packet.destination_port.parse().unwrap(),
                            raw_event.packet.destination_channel.parse().unwrap(),
                        ),
                        // channel and connection on counterparty chain
                        promise(
                            [promise(
                                [fetch_connection_from_channel_info(
                                    self.chain_id.to_string(),
                                    QueryHeight::Specific(event_height),
                                    raw_event.packet.destination_port.parse().unwrap(),
                                    raw_event.packet.destination_channel.parse().unwrap(),
                                )],
                                [],
                                AggregateDecodeClientStateMetaFromConnection {},
                            )],
                            [],
                            AggregateFetchCounterpartyChannelAndConnection {
                                counterparty_port_id: raw_event.packet.source_port.parse().unwrap(),
                                counterparty_channel_id: raw_event
                                    .packet
                                    .source_channel
                                    .parse()
                                    .unwrap(),
                            },
                        ),
                    ],
                    [],
                    Callback::plugin(
                        self.plugin_name(),
                        EventInfo {
                            chain_id: self.chain_id.to_string(),
                            height: event_height,
                            tx_hash,
                            raw_event,
                        },
                    ),
                )
            }
            IBCHandlerEvents::PacketEvent(IBCPacketEvents::SendPacketFilter(raw_event)) => {
                promise(
                    [
                        // client underlying the channel on this chain
                        promise(
                            [channel_fetch(
                                &raw_event.source_port,
                                &raw_event.source_channel,
                                event_height,
                            )],
                            [],
                            AggregateFetchClientFromChannel {
                                fetch_type: InfoOrMeta::Both,
                            },
                        ),
                        // channel on this chain
                        channel_fetch(
                            &raw_event.source_port,
                            &raw_event.source_channel,
                            event_height,
                        ),
                        // connection on this chain
                        fetch_connection_from_channel_info(
                            self.chain_id.to_string(),
                            QueryHeight::Specific(event_height),
                            raw_event.source_port.parse().unwrap(),
                            raw_event.source_channel.parse().unwrap(),
                        ),
                        // channel and connection on the counterparty chain
                        promise(
                            [
                                promise(
                                    [fetch_connection_from_channel_info(
                                        self.chain_id.to_string(),
                                        QueryHeight::Specific(event_height),
                                        raw_event.source_port.parse().unwrap(),
                                        raw_event.source_channel.parse().unwrap(),
                                    )],
                                    [],
                                    AggregateDecodeClientStateMetaFromConnection {},
                                ),
                                channel_fetch(
                                    &raw_event.source_port,
                                    &raw_event.source_channel,
                                    event_height,
                                ),
                            ],
                            [],
                            AggregateFetchCounterpartyChannelAndConnectionFromSourceChannel {},
                        ),
                    ],
                    [],
                    Callback::plugin(
                        self.plugin_name(),
                        EventInfo {
                            chain_id: self.chain_id.to_string(),
                            height: event_height,
                            tx_hash,
                            raw_event,
                        },
                    ),
                )
            }
            IBCHandlerEvents::PacketEvent(IBCPacketEvents::WriteAcknowledgementFilter(
                raw_event,
            )) => promise(
                [
                    // client underlying the channel on this chain
                    promise(
                        [channel_fetch(
                            &raw_event.packet.destination_port,
                            &raw_event.packet.destination_channel,
                            event_height,
                        )],
                        [],
                        AggregateFetchClientFromChannel {
                            fetch_type: InfoOrMeta::Both,
                        },
                    ),
                    // channel on this chain
                    channel_fetch(
                        &raw_event.packet.destination_port,
                        &raw_event.packet.destination_channel,
                        event_height,
                    ),
                    // connection on this chain
                    fetch_connection_from_channel_info(
                        self.chain_id.to_string(),
                        QueryHeight::Specific(event_height),
                        raw_event.packet.destination_port.parse().unwrap(),
                        raw_event.packet.destination_channel.parse().unwrap(),
                    ),
                    // channel and connection on the counterparty chain
                    promise(
                        [promise(
                            [fetch_connection_from_channel_info(
                                self.chain_id.to_string(),
                                QueryHeight::Specific(event_height),
                                raw_event.packet.destination_port.parse().unwrap(),
                                raw_event.packet.destination_channel.parse().unwrap(),
                            )],
                            [],
                            AggregateDecodeClientStateMetaFromConnection {},
                        )],
                        [],
                        AggregateFetchCounterpartyChannelAndConnection {
                            counterparty_port_id: raw_event.packet.source_port.parse().unwrap(),
                            counterparty_channel_id: raw_event
                                .packet
                                .source_channel
                                .parse()
                                .unwrap(),
                        },
                    ),
                ],
                [],
                Callback::plugin(
                    self.plugin_name(),
                    EventInfo {
                        chain_id: self.chain_id.to_string(),
                        height: event_height,
                        tx_hash,
                        raw_event,
                    },
                ),
            ),
            IBCHandlerEvents::PacketEvent(IBCPacketEvents::AcknowledgePacketFilter(raw_event)) => {
                promise(
                    [
                        // client underlying the channel on this chain
                        promise(
                            [channel_fetch(
                                &raw_event.packet.source_port,
                                &raw_event.packet.source_channel,
                                event_height,
                            )],
                            [],
                            AggregateFetchClientFromChannel {
                                fetch_type: InfoOrMeta::Both,
                            },
                        ),
                        // channel on this chain
                        channel_fetch(
                            &raw_event.packet.source_port,
                            &raw_event.packet.source_channel,
                            event_height,
                        ),
                        // connection on this chain
                        fetch_connection_from_channel_info(
                            self.chain_id.to_string(),
                            QueryHeight::Specific(event_height),
                            raw_event.packet.source_port.parse().unwrap(),
                            raw_event.packet.source_channel.parse().unwrap(),
                        ),
                        // channel and connection on the counterparty chain
                        promise(
                            [promise(
                                [fetch_connection_from_channel_info(
                                    self.chain_id.to_string(),
                                    QueryHeight::Specific(event_height),
                                    raw_event.packet.source_port.parse().unwrap(),
                                    raw_event.packet.source_channel.parse().unwrap(),
                                )],
                                [],
                                AggregateDecodeClientStateMetaFromConnection {},
                            )],
                            [],
                            AggregateFetchCounterpartyChannelAndConnection {
                                counterparty_port_id: raw_event
                                    .packet
                                    .destination_port
                                    .parse()
                                    .unwrap(),
                                counterparty_channel_id: raw_event
                                    .packet
                                    .destination_channel
                                    .parse()
                                    .unwrap(),
                            },
                        ),
                    ],
                    [],
                    Callback::plugin(
                        self.plugin_name(),
                        EventInfo {
                            chain_id: self.chain_id.to_string(),
                            height: event_height,
                            tx_hash,
                            raw_event,
                        },
                    ),
                )
            }
            // TODO: Handle this
            IBCHandlerEvents::PacketEvent(IBCPacketEvents::TimeoutPacketFilter(raw_event)) => {
                panic!("{raw_event:?}")
            }
            IBCHandlerEvents::OwnableEvent(_) => noop(),
        }
    }

    fn ibc_handler(&self) -> IBCHandler<Provider<Ws>> {
        IBCHandler::new(self.ibc_handler_address, Arc::new(self.provider.clone()))
    }

    #[instrument(skip(self))]
    pub async fn execution_height_of_beacon_slot(&self, slot: u64) -> u64 {
        let execution_height = self
            .beacon_api_client
            .execution_height(beacon_api::client::BlockId::Slot(slot))
            .await
            .unwrap();

        debug!("beacon slot {slot} is execution height {execution_height}");

        execution_height
    }

    #[instrument(skip_all, fields(%path, %height))]
    pub async fn fetch_ibc_state(&self, path: Path, height: Height) -> Result<Value, BoxDynError> {
        let execution_height = self
            .execution_height_of_beacon_slot(height.revision_height)
            .await;

        Ok(match path {
            Path::ClientState(path) => serde_json::to_value(
                self.ibc_handler()
                    .ibc_state_read(execution_height, path.clone())
                    .await
                    .unwrap(),
            )
            .unwrap(),
            Path::ClientConsensusState(path) => serde_json::to_value(
                self.ibc_handler()
                    .ibc_state_read(execution_height, path.clone())
                    .await
                    .unwrap(),
            )
            .unwrap(),
            Path::Connection(path) => serde_json::to_value(
                self.ibc_handler()
                    .ibc_state_read(execution_height, path.clone())
                    .await
                    .unwrap(),
            )
            .unwrap(),
            Path::ChannelEnd(path) => serde_json::to_value(
                self.ibc_handler()
                    .ibc_state_read(execution_height, path.clone())
                    .await
                    .unwrap(),
            )
            .unwrap(),
            Path::Commitment(path) => serde_json::to_value(
                self.ibc_handler()
                    .ibc_state_read(execution_height, path.clone())
                    .await
                    .unwrap(),
            )
            .unwrap(),
            Path::Acknowledgement(path) => serde_json::to_value(
                self.ibc_handler()
                    .ibc_state_read(execution_height, path.clone())
                    .await
                    .unwrap(),
            )
            .unwrap(),
            Path::Receipt(path) => serde_json::to_value(
                self.ibc_handler()
                    .ibc_state_read(execution_height, path.clone())
                    .await
                    .unwrap(),
            )
            .unwrap(),
            Path::NextSequenceSend(path) => serde_json::to_value(
                self.ibc_handler()
                    .ibc_state_read(execution_height, path.clone())
                    .await
                    .unwrap(),
            )
            .unwrap(),
            Path::NextSequenceRecv(path) => serde_json::to_value(
                self.ibc_handler()
                    .ibc_state_read(execution_height, path.clone())
                    .await
                    .unwrap(),
            )
            .unwrap(),
            Path::NextSequenceAck(path) => serde_json::to_value(
                self.ibc_handler()
                    .ibc_state_read(execution_height, path.clone())
                    .await
                    .unwrap(),
            )
            .unwrap(),
            Path::NextConnectionSequence(path) => serde_json::to_value(
                self.ibc_handler()
                    .ibc_state_read(execution_height, path.clone())
                    .await
                    .unwrap(),
            )
            .unwrap(),
            Path::NextClientSequence(path) => serde_json::to_value(
                self.ibc_handler()
                    .ibc_state_read(execution_height, path.clone())
                    .await
                    .unwrap(),
            )
            .unwrap(),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InitError {
    #[error("unable to connect to websocket")]
    Ws(#[from] WsClientError),
    #[error("provider error")]
    Provider(#[from] ProviderError),
    #[error("beacon error")]
    Beacon(#[from] beacon_api::client::NewError),
}

#[async_trait]
impl PluginModuleServer<ModuleData, ModuleFetch, ModuleAggregate> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn info(&self) -> RpcResult<PluginInfo> {
        Ok(PluginInfo {
            name: self.plugin_name(),
            kind: Some(PluginKind::Chain),
            interest_filter: None,
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    fn handle_aggregate(
        &self,
        aggregate: ModuleAggregate,
        data: VecDeque<Data<ModuleData>>,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>>> {
        Ok(match aggregate {
            ModuleAggregate::CreateClient(aggregate) => do_aggregate(aggregate, data),
            ModuleAggregate::UpdateClient(aggregate) => do_aggregate(aggregate, data),
            ModuleAggregate::ConnectionOpenInit(aggregate) => do_aggregate(aggregate, data),
            ModuleAggregate::ConnectionOpenTry(aggregate) => do_aggregate(aggregate, data),
            ModuleAggregate::ConnectionOpenAck(aggregate) => do_aggregate(aggregate, data),
            ModuleAggregate::ConnectionOpenConfirm(aggregate) => do_aggregate(aggregate, data),
            ModuleAggregate::ChannelOpenInit(aggregate) => do_aggregate(aggregate, data),
            ModuleAggregate::ChannelOpenTry(aggregate) => do_aggregate(aggregate, data),
            ModuleAggregate::ChannelOpenAck(aggregate) => do_aggregate(aggregate, data),
            ModuleAggregate::ChannelOpenConfirm(aggregate) => do_aggregate(aggregate, data),
            ModuleAggregate::SendPacket(aggregate) => do_aggregate(aggregate, data),
            ModuleAggregate::RecvPacket(aggregate) => do_aggregate(aggregate, data),
            ModuleAggregate::WriteAcknowledgement(aggregate) => do_aggregate(aggregate, data),
            ModuleAggregate::PacketAcknowledgement(aggregate) => do_aggregate(aggregate, data),
            ModuleAggregate::PacketTimeout(aggregate) => do_aggregate(aggregate, data),
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn handle_fetch(
        &self,
        msg: ModuleFetch,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>>> {
        match msg {
            ModuleFetch::FetchEvents(FetchEvents {
                from_height,
                to_height,
            }) => Ok(call(Call::plugin(
                self.plugin_name(),
                FetchBeaconBlockRange {
                    from_slot: from_height.revision_height,
                    to_slot: to_height.revision_height,
                },
            ))),
            ModuleFetch::FetchBeaconBlockRange(FetchBeaconBlockRange { from_slot, to_slot }) => {
                debug!(%from_slot, %to_slot, "fetching beacon block range");

                assert!(from_slot < to_slot);

                if to_slot - from_slot == 1 {
                    Ok(call(Call::plugin(
                        self.plugin_name(),
                        FetchGetLogs { from_slot, to_slot },
                    )))
                } else {
                    // attempt to shrink from..to
                    // note that this is *exclusive* on `to`
                    for slot in (from_slot + 1)..to_slot {
                        info!(%slot, "querying slot");

                        match self
                            .beacon_api_client
                            .block(beacon_api::client::BlockId::Slot(slot))
                            .await
                        {
                            Err(beacon_api::errors::Error::NotFound(
                                beacon_api::errors::NotFoundError {
                                    message,
                                    error,
                                    status_code,
                                },
                            )) => {
                                info!(%slot, %message, %error, %status_code, "beacon block not found for slot");
                                continue;
                            }
                            Err(err) => {
                                panic!("error fetching beacon block for slot {slot}: {err}")
                            }
                            Ok(_) => {
                                return Ok(conc([
                                    call(Call::plugin(
                                        self.plugin_name(),
                                        FetchGetLogs {
                                            from_slot,
                                            to_slot: slot,
                                        },
                                    )),
                                    call(Call::plugin(
                                        self.plugin_name(),
                                        FetchBeaconBlockRange {
                                            from_slot: slot,
                                            to_slot,
                                        },
                                    )),
                                ]));
                            }
                        }
                    }

                    // if the range is not shrinkable (i.e. all blocks between `from` and `to` are missing, but `from` and `to` both exist), fetch logs between `from` and `to`
                    Ok(call(Call::plugin(
                        self.plugin_name(),
                        FetchGetLogs { from_slot, to_slot },
                    )))
                }
            }
            ModuleFetch::FetchGetLogs(FetchGetLogs { from_slot, to_slot }) => {
                debug!(%from_slot, %to_slot, "fetching logs in beacon block range");

                let event_height = Height {
                    revision_number: ETHEREUM_REVISION_NUMBER,
                    revision_height: to_slot,
                };

                let from_block = self.execution_height_of_beacon_slot(from_slot).await;
                let to_block = self.execution_height_of_beacon_slot(to_slot).await;

                if from_block == to_block {
                    info!(%from_block, %to_block, %from_slot, %to_slot, "beacon block range is empty");
                    Ok(noop())
                } else {
                    const ETH_GET_LOGS_BATCH_SIZE: u64 = 100;

                    info!(%from_block, %to_block, "fetching block range");

                    let mut from_block = from_block;

                    let mut msgs = vec![];
                    let join_set = FuturesUnordered::new();

                    for (from_block, to_block) in std::iter::from_fn(move || {
                        if from_block < to_block {
                            // NOTE: These -1s are very important, else events will be double fetched
                            if to_block - from_block < ETH_GET_LOGS_BATCH_SIZE {
                                Some((from_block, {
                                    from_block = to_block;
                                    from_block - 1
                                }))
                            } else {
                                Some((from_block, {
                                    from_block += ETH_GET_LOGS_BATCH_SIZE;
                                    from_block - 1
                                }))
                            }
                        } else {
                            None
                        }
                    }) {
                        info!(%from_block, %to_block, %from_slot, %to_slot, "fetching logs in range");

                        join_set.push(async move {
                            self.provider
                                .get_logs(
                                    &Filter::new()
                                        .address(ethers::types::H160::from(
                                            self.ibc_handler_address,
                                        ))
                                        .from_block(from_block)
                                        .to_block(to_block),
                                )
                                .await
                        });
                    }

                    for logs in join_set
                        .try_collect::<Vec<_>>()
                        .await
                        .expect("unable to fetch logs")
                    {
                        for log in logs {
                            let tx_hash = log
                                .transaction_hash
                                .expect("log should have transaction_hash")
                                .into();

                            debug!(?log, "raw log");

                            match IBCHandlerEvents::decode_log(&log.into()) {
                                Ok(event) => {
                                    debug!(?event, "found IBCHandler event");
                                    msgs.push(self.mk_aggregate_event(event, event_height, tx_hash))
                                }
                                Err(e) => {
                                    warn!("could not decode evm event {}", e);
                                }
                            }
                        }
                    }

                    info!(
                        %from_block,
                        %to_block,

                        %from_slot,
                        %to_slot,

                        slot_range_len = %(to_slot - from_slot),
                        block_range_len = %(to_block - from_block),

                        total = %msgs.len(),

                        "fetched logs in block range"
                    );

                    Ok(conc(msgs))
                }
            }
        }
    }
}

#[async_trait]
impl ChainModuleServer<ModuleData, ModuleFetch, ModuleAggregate> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    fn chain_id(&self) -> RpcResult<String> {
        Ok(self.chain_id.to_string())
    }

    /// Query the latest finalized height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_latest_height(&self) -> RpcResult<Height> {
        self.beacon_api_client
            .finality_update()
            .await
            .map(|response| self.make_height(response.data.attested_header.beacon.slot))
            .map_err(|err| ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>))
    }

    /// Query the latest (non-finalized) height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_latest_height_as_destination(&self) -> RpcResult<Height> {
        let height = self
            .beacon_api_client
            .block(beacon_api::client::BlockId::Head)
            .await
            .map_err(|err| ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>))?
            .data
            .message
            .slot;

        // // HACK: we introduced this because we were using alchemy for the
        // // execution endpoint and our custom beacon endpoint that rely on
        // // its own execution chain. Alchemy was a bit delayed and the
        // // execution height for the beacon head wasn't existing for few
        // // secs. We wait for an extra beacon head to let alchemy catch up.
        // // We should be able to remove that once we rely on an execution
        // // endpoint that is itself used by the beacon endpoint (no different
        // // POV).
        // loop {
        //     let next_height = self
        //         .beacon_api_client
        //         .block(beacon_api::client::BlockId::Head)
        //         .await?
        //         .data
        //         .message
        //         .slot;
        //     if next_height > height {
        //         break;
        //     }

        //     tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        // }

        Ok(self.make_height(height))
    }

    /// Query the latest finalized timestamp of this chain.
    // TODO: Use a better timestamp type here
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_latest_timestamp(&self) -> RpcResult<i64> {
        Ok(self
            .beacon_api_client
            .finality_update()
            .await
            .map_err(|err| ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>))?
            .data
            .attested_header
            .execution
            .timestamp
            .try_into()
            .unwrap())
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn fetch_block_range(
        &self,
        from_height: Height,
        to_height: Height,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>>> {
        Ok(call(Call::plugin(
            self.plugin_name(),
            FetchEvents {
                from_height,
                to_height,
            },
        )))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn client_info(&self, client_id: ClientId) -> RpcResult<ClientInfo> {
        Ok(ClientInfo {
            client_type: ClientType::new(
                self.ibc_handler()
                    .client_types(client_id.to_string())
                    .await
                    .unwrap(),
            ),
            ibc_interface: IbcInterface::new(IbcInterface::IBC_SOLIDITY),
            metadata: Default::default(),
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_ibc_state(&self, at: Height, path: Path) -> RpcResult<Value> {
        self.fetch_ibc_state(path, at).await.map_err(|err| {
            ErrorObject::owned(
                -1,
                format!("error fetching ibc state: {}", ErrorReporter(&*err)),
                None::<()>,
            )
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_ibc_proof(&self, at: Height, path: Path) -> RpcResult<Value> {
        let location = ibc_commitment_key(&path.to_string(), IBC_HANDLER_COMMITMENTS_SLOT);

        let execution_height = self
            .execution_height_of_beacon_slot(at.revision_height)
            .await;

        let proof = self
            .provider
            .get_proof(
                ethers::types::H160::from(self.ibc_handler_address),
                vec![location.to_be_bytes().into()],
                Some(execution_height.into()),
            )
            .await
            .unwrap();

        let proof = match <[_; 1]>::try_from(proof.storage_proof) {
            Ok([proof]) => proof,
            Err(invalid) => {
                panic!("received invalid response from eth_getProof, expected length of 1 but got `{invalid:#?}`");
            }
        };

        let proof = StorageProof {
            key: U256::from_be_bytes(proof.key.to_fixed_bytes()),
            value: proof.value.into(),
            proof: proof
                .proof
                .into_iter()
                .map(|bytes| bytes.to_vec())
                .collect(),
        };

        Ok(serde_json::to_value(proof).expect("serialization is infallible; qed;"))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_raw_unfinalized_trusted_client_state(
        &self,
        client_id: ClientId,
    ) -> RpcResult<RawClientState<'static>> {
        let latest_execution_height = self.provider.get_block_number().await.unwrap().as_u64();

        let ClientInfo {
            client_type,
            ibc_interface,
            metadata: _,
        } = self.client_info(client_id.clone()).await?;

        Ok(RawClientState {
            client_type,
            ibc_interface,
            bytes: self
                .ibc_handler()
                .ibc_state_read(latest_execution_height, ClientStatePath { client_id })
                .await
                .unwrap()
                .0
                .into(),
        })
    }
}

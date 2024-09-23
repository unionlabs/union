// #![warn(clippy::unwrap_used)] // oh boy this will be a lot of work

use std::{collections::VecDeque, sync::Arc};

use beacon_api::client::BeaconApiClient;
use chain_utils::ethereum::{IBCHandlerEvents, IbcHandlerExt};
use contracts::{
    ibc_channel_handshake::IBCChannelHandshakeEvents, ibc_client::IBCClientEvents,
    ibc_connection::IBCConnectionEvents, ibc_handler::IBCHandler, ibc_packet::IBCPacketEvents,
};
use ethers::{
    contract::EthLogDecode,
    providers::{Middleware, Provider, Ws},
    types::Filter,
};
use futures::{stream::FuturesUnordered, TryStreamExt};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
};
use queue_msg::{call, conc, data, noop, BoxDynError, Op};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, info, instrument, warn};
use unionlabs::{
    ethereum::{ibc_commitment_key, IBC_HANDLER_COMMITMENTS_SLOT},
    hash::H160,
    ibc::{
        core::{channel, client::height::Height},
        lightclients::ethereum::storage_proof::StorageProof,
    },
    ics24::{ChannelEndPath, ClientStatePath, ConnectionPath, Path},
    id::{ChannelId, ClientId, PortId},
    uint::U256,
    ErrorReporter, QueryHeight,
};
use voyager_message::{
    call::Call,
    core::{ChainId, ClientInfo, ClientType, IbcInterface},
    data::{
        ChainEvent, ChannelMetadata, ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit,
        ChannelOpenTry, ConnectionMetadata, ConnectionOpenAck, ConnectionOpenConfirm,
        ConnectionOpenInit, ConnectionOpenTry, CreateClient, Data, PacketMetadata, UpdateClient,
    },
    module::{
        ChainModuleInfo, ChainModuleServer, ModuleInfo, QueueInteractionsServer, RawClientState,
    },
    reconnecting_jsonrpc_ws_client,
    rpc::{json_rpc_error_to_rpc_error, missing_state, VoyagerRpcClient, VoyagerRpcClientExt},
    run_module_server, DefaultCmd, ModuleContext, ModuleServer, VoyagerMessage,
};

use crate::{
    call::{FetchBeaconBlockRange, FetchEvents, FetchGetLogs, MakeFullEvent, ModuleCall},
    callback::ModuleCallback,
    data::ModuleData,
};

pub mod call;
pub mod callback;
pub mod data;

const ETHEREUM_REVISION_NUMBER: u64 = 0;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_module_server::<Module, _, _, _>().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId<'static>,

    pub ibc_handler_address: H160,

    pub provider: Provider<Ws>,
    pub beacon_api_client: BeaconApiClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The expected chain id of this ethereum-like chain.
    pub chain_id: ChainId<'static>,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    /// The RPC endpoint for the execution chain.
    pub eth_rpc_api: String,
    /// The RPC endpoint for the beacon chain.
    pub eth_beacon_rpc_api: String,
}

impl ModuleContext for Module {
    type Config = Config;
    type Cmd = DefaultCmd;
    type Info = ChainModuleInfo;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        Module::new(config).await
    }

    fn info(config: Self::Config) -> ModuleInfo<Self::Info> {
        ModuleInfo {
            name: plugin_name(&config.chain_id),
            kind: ChainModuleInfo {
                chain_id: config.chain_id,
            },
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

fn plugin_name(chain_id: &ChainId<'_>) -> String {
    pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

    format!("{PLUGIN_NAME}/{}", chain_id)
}

impl Module {
    pub fn plugin_name(&self) -> String {
        plugin_name(&self.chain_id)
    }

    pub async fn new(config: Config) -> Result<Self, BoxDynError> {
        let provider = Provider::new(Ws::connect(config.eth_rpc_api).await?);

        let chain_id = provider.get_chainid().await?;

        Ok(Self {
            chain_id: ChainId::new(U256(chain_id).to_string()),
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

    async fn make_packet_metadata(
        &self,
        event_height: Height,
        self_port_id: PortId,
        self_channel_id: ChannelId,
        voyager_rpc_client: &reconnecting_jsonrpc_ws_client::Client,
    ) -> RpcResult<(
        ChainId<'static>,
        ClientInfo,
        ChannelMetadata,
        ChannelMetadata,
        channel::order::Order,
    )> {
        let self_channel = voyager_rpc_client
            .query_ibc_state_typed(
                self.chain_id.clone(),
                event_height.into(),
                ChannelEndPath {
                    port_id: self_port_id.clone(),
                    channel_id: self_channel_id.clone(),
                },
            )
            .await
            .map_err(json_rpc_error_to_rpc_error)?
            .state
            .ok_or_else(missing_state("connection must exist", None))?;

        let self_connection = voyager_rpc_client
            .query_ibc_state_typed(
                self.chain_id.clone(),
                event_height.into(),
                ConnectionPath {
                    connection_id: self_channel.connection_hops[0].clone(),
                },
            )
            .await
            .map_err(json_rpc_error_to_rpc_error)?;

        let self_connection_state = self_connection
            .state
            .ok_or_else(missing_state("connection must exist", None))?;

        let client_info = voyager_rpc_client
            .client_info(
                self.chain_id.clone(),
                self_connection_state.client_id.clone(),
            )
            .await
            .map_err(json_rpc_error_to_rpc_error)?;

        let client_meta = voyager_rpc_client
            .client_meta(
                self.chain_id.clone(),
                event_height.into(),
                self_connection_state.client_id.clone(),
            )
            .await
            .map_err(json_rpc_error_to_rpc_error)?;

        let other_channel = voyager_rpc_client
            .query_ibc_state_typed(
                client_meta.chain_id.clone(),
                QueryHeight::Latest,
                ChannelEndPath {
                    port_id: self_channel.counterparty.port_id.clone(),
                    channel_id: self_channel.counterparty.channel_id.parse().unwrap(),
                },
            )
            .await
            .map_err(json_rpc_error_to_rpc_error)?;

        let other_channel_state = other_channel
            .state
            .ok_or_else(missing_state("channel must exist", None))?;

        let source_channel = ChannelMetadata {
            port_id: self_port_id.clone(),
            channel_id: self_channel_id.clone(),
            version: self_channel.version,
            connection: ConnectionMetadata {
                client_id: self_connection_state.client_id,
                connection_id: self_connection.path.connection_id.clone(),
            },
        };
        let destination_channel = ChannelMetadata {
            port_id: other_channel.path.port_id.clone(),
            channel_id: other_channel.path.channel_id.clone(),
            version: other_channel_state.version,
            connection: ConnectionMetadata {
                client_id: self_connection_state.counterparty.client_id,
                connection_id: self_connection_state.counterparty.connection_id.unwrap(),
            },
        };

        Ok((
            client_meta.chain_id,
            client_info,
            source_channel,
            destination_channel,
            self_channel.ordering,
        ))
    }
}

#[async_trait]
impl QueueInteractionsServer<ModuleData, ModuleCall, ModuleCallback> for ModuleServer<Module> {
    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn callback(
        &self,
        cb: ModuleCallback,
        _data: VecDeque<Data<ModuleData>>,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match cb {}
    }

    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn call(
        &self,
        msg: ModuleCall,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match msg {
            ModuleCall::MakeFullEvent(MakeFullEvent {
                slot,
                tx_hash,
                event,
            }) => {
                let height = self.ctx.make_height(slot);

                match event {
                    IBCHandlerEvents::ChannelEvent(
                        IBCChannelHandshakeEvents::ChannelCloseConfirmFilter(_),
                    )
                    | IBCHandlerEvents::ChannelEvent(
                        IBCChannelHandshakeEvents::ChannelCloseInitFilter(_),
                    ) => {
                        todo!()
                    }

                    IBCHandlerEvents::ChannelEvent(
                        IBCChannelHandshakeEvents::ChannelOpenInitFilter(raw_event),
                    ) => {
                        let connection = self
                            .voyager_rpc_client
                            .query_ibc_state_typed(
                                self.ctx.chain_id.clone(),
                                height.into(),
                                ConnectionPath {
                                    connection_id: raw_event.connection_id.parse().unwrap(),
                                },
                            )
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?
                            .state
                            .ok_or_else(missing_state("connection must exist", None))?;

                        let client_info = self
                            .voyager_rpc_client
                            .client_info(self.ctx.chain_id.clone(), connection.client_id.clone())
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?;

                        let client_meta = self
                            .voyager_rpc_client
                            .client_meta(
                                self.ctx.chain_id.clone(),
                                height.into(),
                                connection.client_id.clone(),
                            )
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?;

                        let channel = self
                            .voyager_rpc_client
                            .query_ibc_state_typed(
                                self.ctx.chain_id.clone(),
                                height.into(),
                                ChannelEndPath {
                                    port_id: raw_event.port_id.parse().unwrap(),
                                    channel_id: raw_event.channel_id.parse().unwrap(),
                                },
                            )
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?
                            .state
                            .ok_or_else(missing_state("connection must exist", None))?;

                        Ok(data(ChainEvent {
                            chain_id: self.ctx.chain_id.clone(),
                            client_info,
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height: height,
                            event: ChannelOpenInit {
                                port_id: raw_event.port_id.parse().unwrap(),
                                channel_id: raw_event.channel_id.parse().unwrap(),
                                counterparty_port_id: raw_event
                                    .counterparty_port_id
                                    .parse()
                                    .unwrap(),
                                connection,
                                version: channel.version,
                            }
                            .into(),
                        }))
                    }
                    IBCHandlerEvents::ChannelEvent(
                        IBCChannelHandshakeEvents::ChannelOpenTryFilter(raw_event),
                    ) => {
                        let connection = self
                            .voyager_rpc_client
                            .query_ibc_state_typed(
                                self.ctx.chain_id.clone(),
                                height.into(),
                                ConnectionPath {
                                    connection_id: raw_event.connection_id.parse().unwrap(),
                                },
                            )
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?
                            .state
                            .ok_or_else(missing_state("connection must exist", None))?;

                        let client_info = self
                            .voyager_rpc_client
                            .client_info(self.ctx.chain_id.clone(), connection.client_id.clone())
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?;

                        let client_meta = self
                            .voyager_rpc_client
                            .client_meta(
                                self.ctx.chain_id.clone(),
                                height.into(),
                                connection.client_id.clone(),
                            )
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?;

                        let channel = self
                            .voyager_rpc_client
                            .query_ibc_state_typed(
                                self.ctx.chain_id.clone(),
                                height.into(),
                                ChannelEndPath {
                                    port_id: raw_event.port_id.parse().unwrap(),
                                    channel_id: raw_event.channel_id.parse().unwrap(),
                                },
                            )
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?
                            .state
                            .ok_or_else(missing_state("channel must exist", None))?;

                        Ok(data(ChainEvent {
                            chain_id: self.ctx.chain_id.clone(),
                            client_info,
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height: height,
                            event: ChannelOpenTry {
                                port_id: raw_event.port_id.parse().unwrap(),
                                channel_id: raw_event.channel_id.parse().unwrap(),
                                counterparty_port_id: raw_event
                                    .counterparty_port_id
                                    .parse()
                                    .unwrap(),
                                counterparty_channel_id: raw_event
                                    .counterparty_channel_id
                                    .parse()
                                    .unwrap(),
                                connection,
                                version: channel.version,
                            }
                            .into(),
                        }))
                    }
                    IBCHandlerEvents::ChannelEvent(
                        IBCChannelHandshakeEvents::ChannelOpenAckFilter(raw_event),
                    ) => {
                        let connection = self
                            .voyager_rpc_client
                            .query_ibc_state_typed(
                                self.ctx.chain_id.clone(),
                                height.into(),
                                ConnectionPath {
                                    connection_id: raw_event.connection_id.parse().unwrap(),
                                },
                            )
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?
                            .state
                            .ok_or_else(missing_state("connection must exist", None))?;

                        let client_info = self
                            .voyager_rpc_client
                            .client_info(self.ctx.chain_id.clone(), connection.client_id.clone())
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?;

                        let client_meta = self
                            .voyager_rpc_client
                            .client_meta(
                                self.ctx.chain_id.clone(),
                                height.into(),
                                connection.client_id.clone(),
                            )
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?;

                        let channel = self
                            .voyager_rpc_client
                            .query_ibc_state_typed(
                                self.ctx.chain_id.clone(),
                                height.into(),
                                ChannelEndPath {
                                    port_id: raw_event.port_id.parse().unwrap(),
                                    channel_id: raw_event.channel_id.parse().unwrap(),
                                },
                            )
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?
                            .state
                            .ok_or_else(missing_state("channel must exist", None))?;

                        Ok(data(ChainEvent {
                            chain_id: self.ctx.chain_id.clone(),
                            client_info,
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height: height,
                            event: ChannelOpenAck {
                                port_id: raw_event.port_id.parse().unwrap(),
                                channel_id: raw_event.channel_id.parse().unwrap(),
                                counterparty_port_id: raw_event
                                    .counterparty_port_id
                                    .parse()
                                    .unwrap(),
                                counterparty_channel_id: raw_event
                                    .counterparty_channel_id
                                    .parse()
                                    .unwrap(),
                                connection,
                                version: channel.version,
                            }
                            .into(),
                        }))
                    }
                    IBCHandlerEvents::ChannelEvent(
                        IBCChannelHandshakeEvents::ChannelOpenConfirmFilter(raw_event),
                    ) => {
                        let connection = self
                            .voyager_rpc_client
                            .query_ibc_state_typed(
                                self.ctx.chain_id.clone(),
                                height.into(),
                                ConnectionPath {
                                    connection_id: raw_event.connection_id.parse().unwrap(),
                                },
                            )
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?
                            .state
                            .ok_or_else(missing_state("connection must exist", None))?;

                        let client_info = self
                            .voyager_rpc_client
                            .client_info(self.ctx.chain_id.clone(), connection.client_id.clone())
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?;

                        let client_meta = self
                            .voyager_rpc_client
                            .client_meta(
                                self.ctx.chain_id.clone(),
                                height.into(),
                                connection.client_id.clone(),
                            )
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?;

                        let channel = self
                            .voyager_rpc_client
                            .query_ibc_state_typed(
                                self.ctx.chain_id.clone(),
                                height.into(),
                                ChannelEndPath {
                                    port_id: raw_event.port_id.parse().unwrap(),
                                    channel_id: raw_event.channel_id.parse().unwrap(),
                                },
                            )
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?
                            .state
                            .ok_or_else(missing_state("channel must exist", None))?;

                        Ok(data(ChainEvent {
                            chain_id: self.ctx.chain_id.clone(),
                            client_info,
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height: height,
                            event: ChannelOpenConfirm {
                                port_id: raw_event.port_id.parse().unwrap(),
                                channel_id: raw_event.channel_id.parse().unwrap(),
                                counterparty_port_id: raw_event
                                    .counterparty_port_id
                                    .parse()
                                    .unwrap(),
                                counterparty_channel_id: raw_event
                                    .counterparty_channel_id
                                    .parse()
                                    .unwrap(),
                                connection,
                                version: channel.version,
                            }
                            .into(),
                        }))
                    }

                    IBCHandlerEvents::ConnectionEvent(
                        IBCConnectionEvents::ConnectionOpenInitFilter(raw_event),
                    ) => {
                        let client_info = self
                            .voyager_rpc_client
                            .client_info(
                                self.ctx.chain_id.clone(),
                                raw_event.client_id.parse().unwrap(),
                            )
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?;

                        let client_meta = self
                            .voyager_rpc_client
                            .client_meta(
                                self.ctx.chain_id.clone(),
                                height.into(),
                                raw_event.client_id.parse().unwrap(),
                            )
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?;

                        Ok(data(ChainEvent {
                            chain_id: self.ctx.chain_id.clone(),
                            client_info,
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height: height,
                            event: ConnectionOpenInit {
                                client_id: raw_event.client_id.parse().unwrap(),
                                connection_id: raw_event.connection_id.parse().unwrap(),
                                counterparty_client_id: raw_event
                                    .counterparty_client_id
                                    .parse()
                                    .unwrap(),
                            }
                            .into(),
                        }))
                    }
                    IBCHandlerEvents::ConnectionEvent(
                        IBCConnectionEvents::ConnectionOpenTryFilter(raw_event),
                    ) => {
                        let client_info = self
                            .voyager_rpc_client
                            .client_info(
                                self.ctx.chain_id.clone(),
                                raw_event.client_id.parse().unwrap(),
                            )
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?;

                        let client_meta = self
                            .voyager_rpc_client
                            .client_meta(
                                self.ctx.chain_id.clone(),
                                height.into(),
                                raw_event.client_id.parse().unwrap(),
                            )
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?;

                        Ok(data(ChainEvent {
                            chain_id: self.ctx.chain_id.clone(),
                            client_info,
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height: height,
                            event: ConnectionOpenTry {
                                client_id: raw_event.client_id.parse().unwrap(),
                                connection_id: raw_event.connection_id.parse().unwrap(),
                                counterparty_client_id: raw_event
                                    .counterparty_client_id
                                    .parse()
                                    .unwrap(),
                                counterparty_connection_id: raw_event
                                    .counterparty_connection_id
                                    .parse()
                                    .unwrap(),
                            }
                            .into(),
                        }))
                    }
                    IBCHandlerEvents::ConnectionEvent(
                        IBCConnectionEvents::ConnectionOpenAckFilter(raw_event),
                    ) => {
                        let client_info = self
                            .voyager_rpc_client
                            .client_info(
                                self.ctx.chain_id.clone(),
                                raw_event.client_id.parse().unwrap(),
                            )
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?;

                        let client_meta = self
                            .voyager_rpc_client
                            .client_meta(
                                self.ctx.chain_id.clone(),
                                height.into(),
                                raw_event.client_id.parse().unwrap(),
                            )
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?;

                        Ok(data(ChainEvent {
                            chain_id: self.ctx.chain_id.clone(),
                            client_info,
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height: height,
                            event: ConnectionOpenAck {
                                client_id: raw_event.client_id.parse().unwrap(),
                                connection_id: raw_event.connection_id.parse().unwrap(),
                                counterparty_client_id: raw_event
                                    .counterparty_client_id
                                    .parse()
                                    .unwrap(),
                                counterparty_connection_id: raw_event
                                    .counterparty_connection_id
                                    .parse()
                                    .unwrap(),
                            }
                            .into(),
                        }))
                    }
                    IBCHandlerEvents::ConnectionEvent(
                        IBCConnectionEvents::ConnectionOpenConfirmFilter(raw_event),
                    ) => {
                        let client_info = self
                            .voyager_rpc_client
                            .client_info(
                                self.ctx.chain_id.clone(),
                                raw_event.client_id.parse().unwrap(),
                            )
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?;

                        let client_meta = self
                            .voyager_rpc_client
                            .client_meta(
                                self.ctx.chain_id.clone(),
                                height.into(),
                                raw_event.client_id.parse().unwrap(),
                            )
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?;

                        Ok(data(ChainEvent {
                            chain_id: self.ctx.chain_id.clone(),
                            client_info,
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height: height,
                            event: ConnectionOpenConfirm {
                                client_id: raw_event.client_id.parse().unwrap(),
                                connection_id: raw_event.connection_id.parse().unwrap(),
                                counterparty_client_id: raw_event
                                    .counterparty_client_id
                                    .parse()
                                    .unwrap(),
                                counterparty_connection_id: raw_event
                                    .counterparty_connection_id
                                    .parse()
                                    .unwrap(),
                            }
                            .into(),
                        }))
                    }

                    IBCHandlerEvents::ClientEvent(IBCClientEvents::ClientCreatedFilter(
                        raw_event,
                    )) => {
                        let client_id = raw_event.client_id.parse::<ClientId>().unwrap();

                        let client_info = self
                            .voyager_rpc_client
                            .client_info(self.ctx.chain_id.clone(), client_id.clone())
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?;

                        let client_meta = self
                            .voyager_rpc_client
                            .client_meta(
                                self.ctx.chain_id.clone(),
                                height.into(),
                                raw_event.client_id.parse().unwrap(),
                            )
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?;

                        Ok(data(ChainEvent {
                            chain_id: self.ctx.chain_id.clone(),
                            client_info: client_info.clone(),
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height: height,
                            event: CreateClient {
                                client_id,
                                client_type: client_info.client_type,
                                consensus_height: client_meta.height,
                            }
                            .into(),
                        }))
                    }
                    IBCHandlerEvents::ClientEvent(IBCClientEvents::ClientRegisteredFilter(
                        raw_event,
                    )) => {
                        info!(?raw_event, "observed ClientRegistered event");

                        Ok(noop())
                    }
                    IBCHandlerEvents::ClientEvent(IBCClientEvents::ClientUpdatedFilter(
                        raw_event,
                    )) => {
                        let client_id = raw_event.client_id.parse::<ClientId>().unwrap();

                        let client_info = self
                            .voyager_rpc_client
                            .client_info(self.ctx.chain_id.clone(), client_id.clone())
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?;

                        let client_meta = self
                            .voyager_rpc_client
                            .client_meta(
                                self.ctx.chain_id.clone(),
                                height.into(),
                                raw_event.client_id.parse().unwrap(),
                            )
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?;

                        Ok(data(ChainEvent {
                            chain_id: self.ctx.chain_id.clone(),
                            client_info: client_info.clone(),
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height: height,
                            event: UpdateClient {
                                client_id,
                                client_type: client_info.client_type,
                                consensus_heights: vec![raw_event.height.into()],
                            }
                            .into(),
                        }))
                    }
                    IBCHandlerEvents::PacketEvent(event) => {
                        match event {
                            // packet origin is this chain
                            IBCPacketEvents::SendPacketFilter(event) => {
                                let (
                                    counterparty_chain_id,
                                    client_info,
                                    source_channel,
                                    destination_channel,
                                    channel_ordering,
                                ) = self
                                    .ctx
                                    .make_packet_metadata(
                                        height,
                                        event.source_port.parse().unwrap(),
                                        event.source_channel.parse().unwrap(),
                                        &self.voyager_rpc_client,
                                    )
                                    .await?;

                                Ok(data(ChainEvent {
                                    chain_id: self.ctx.chain_id.clone(),
                                    client_info,
                                    counterparty_chain_id,
                                    tx_hash,
                                    provable_height: height,
                                    event: voyager_message::data::SendPacket {
                                        packet_data: event.data.to_vec(),
                                        packet: PacketMetadata {
                                            sequence: event.sequence.try_into().unwrap(),
                                            source_channel,
                                            destination_channel,
                                            channel_ordering,
                                            timeout_height: event.timeout_height.into(),
                                            timeout_timestamp: event.timeout_timestamp,
                                        },
                                    }
                                    .into(),
                                }))
                            }
                            IBCPacketEvents::TimeoutPacketFilter(event) => {
                                let (
                                    counterparty_chain_id,
                                    client_info,
                                    source_channel,
                                    destination_channel,
                                    channel_ordering,
                                ) = self
                                    .ctx
                                    .make_packet_metadata(
                                        height,
                                        event.packet.source_port.parse().unwrap(),
                                        event.packet.source_channel.parse().unwrap(),
                                        &self.voyager_rpc_client,
                                    )
                                    .await?;

                                Ok(data(ChainEvent {
                                    chain_id: self.ctx.chain_id.clone(),
                                    client_info,
                                    counterparty_chain_id,
                                    tx_hash,
                                    provable_height: height,
                                    event: voyager_message::data::TimeoutPacket {
                                        packet: PacketMetadata {
                                            sequence: event.packet.sequence.try_into().unwrap(),
                                            source_channel,
                                            destination_channel,
                                            channel_ordering,
                                            timeout_height: event.packet.timeout_height.into(),
                                            timeout_timestamp: event.packet.timeout_timestamp,
                                        },
                                    }
                                    .into(),
                                }))
                            }
                            IBCPacketEvents::AcknowledgePacketFilter(event) => {
                                let (
                                    counterparty_chain_id,
                                    client_info,
                                    source_channel,
                                    destination_channel,
                                    channel_ordering,
                                ) = self
                                    .ctx
                                    .make_packet_metadata(
                                        height,
                                        event.packet.source_port.parse().unwrap(),
                                        event.packet.source_channel.parse().unwrap(),
                                        &self.voyager_rpc_client,
                                    )
                                    .await?;

                                Ok(data(ChainEvent {
                                    chain_id: self.ctx.chain_id.clone(),
                                    client_info,
                                    counterparty_chain_id,
                                    tx_hash,
                                    provable_height: height,
                                    event: voyager_message::data::AcknowledgePacket {
                                        packet: PacketMetadata {
                                            sequence: event.packet.sequence.try_into().unwrap(),
                                            source_channel,
                                            destination_channel,
                                            channel_ordering,
                                            timeout_height: event.packet.timeout_height.into(),
                                            timeout_timestamp: event.packet.timeout_timestamp,
                                        },
                                    }
                                    .into(),
                                }))
                            }
                            // packet origin is the counterparty chain
                            IBCPacketEvents::WriteAcknowledgementFilter(event) => {
                                let (
                                    counterparty_chain_id,
                                    client_info,
                                    destination_channel,
                                    source_channel,
                                    channel_ordering,
                                ) = self
                                    .ctx
                                    .make_packet_metadata(
                                        height,
                                        event.packet.destination_port.parse().unwrap(),
                                        event.packet.destination_channel.parse().unwrap(),
                                        &self.voyager_rpc_client,
                                    )
                                    .await?;

                                Ok(data(ChainEvent {
                                    chain_id: self.ctx.chain_id.clone(),
                                    client_info,
                                    counterparty_chain_id,
                                    tx_hash,
                                    provable_height: height,
                                    event: voyager_message::data::WriteAcknowledgement {
                                        packet_data: event.packet.data.to_vec(),
                                        packet_ack: event.acknowledgement.to_vec(),
                                        packet: PacketMetadata {
                                            sequence: event.packet.sequence.try_into().unwrap(),
                                            source_channel,
                                            destination_channel,
                                            channel_ordering,
                                            timeout_height: event.packet.timeout_height.into(),
                                            timeout_timestamp: event.packet.timeout_timestamp,
                                        },
                                    }
                                    .into(),
                                }))
                            }
                            IBCPacketEvents::RecvPacketFilter(event) => {
                                let (
                                    counterparty_chain_id,
                                    client_info,
                                    destination_channel,
                                    source_channel,
                                    channel_ordering,
                                ) = self
                                    .ctx
                                    .make_packet_metadata(
                                        height,
                                        event.packet.destination_port.parse().unwrap(),
                                        event.packet.destination_channel.parse().unwrap(),
                                        &self.voyager_rpc_client,
                                    )
                                    .await?;

                                Ok(data(ChainEvent {
                                    chain_id: self.ctx.chain_id.clone(),
                                    client_info,
                                    counterparty_chain_id,
                                    tx_hash,
                                    provable_height: height,
                                    event: voyager_message::data::RecvPacket {
                                        packet_data: event.packet.data.to_vec(),
                                        packet: PacketMetadata {
                                            sequence: event.packet.sequence.try_into().unwrap(),
                                            source_channel,
                                            destination_channel,
                                            channel_ordering,
                                            timeout_height: event.packet.timeout_height.into(),
                                            timeout_timestamp: event.packet.timeout_timestamp,
                                        },
                                    }
                                    .into(),
                                }))
                            }
                        }
                    }
                    IBCHandlerEvents::OwnableEvent(_) => Ok(noop()),
                }
            }
            ModuleCall::FetchEvents(FetchEvents {
                from_height,
                to_height,
            }) => Ok(call(Call::plugin(
                self.ctx.plugin_name(),
                FetchBeaconBlockRange {
                    from_slot: from_height.revision_height,
                    to_slot: to_height.revision_height,
                },
            ))),
            ModuleCall::FetchBeaconBlockRange(FetchBeaconBlockRange { from_slot, to_slot }) => {
                debug!(%from_slot, %to_slot, "fetching beacon block range");

                assert!(from_slot < to_slot);

                if to_slot - from_slot == 1 {
                    Ok(call(Call::plugin(
                        self.ctx.plugin_name(),
                        FetchGetLogs { from_slot, to_slot },
                    )))
                } else {
                    // attempt to shrink from..to
                    // note that this is *exclusive* on `to`
                    for slot in (from_slot + 1)..to_slot {
                        info!(%slot, "querying slot");

                        match self
                            .ctx
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
                                        self.ctx.plugin_name(),
                                        FetchGetLogs {
                                            from_slot,
                                            to_slot: slot,
                                        },
                                    )),
                                    call(Call::plugin(
                                        self.ctx.plugin_name(),
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
                        self.ctx.plugin_name(),
                        FetchGetLogs { from_slot, to_slot },
                    )))
                }
            }
            ModuleCall::FetchGetLogs(FetchGetLogs { from_slot, to_slot }) => {
                debug!(%from_slot, %to_slot, "fetching logs in beacon block range");

                let from_block = self.ctx.execution_height_of_beacon_slot(from_slot).await;
                let to_block = self.ctx.execution_height_of_beacon_slot(to_slot).await;

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
                            self.ctx
                                .provider
                                .get_logs(
                                    &Filter::new()
                                        .address(ethers::types::H160::from(
                                            self.ctx.ibc_handler_address,
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
                                    msgs.push(call(Call::plugin(
                                        self.ctx.plugin_name(),
                                        MakeFullEvent {
                                            slot: to_slot,
                                            tx_hash,
                                            event,
                                        },
                                    )))
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
impl ChainModuleServer<ModuleData, ModuleCall, ModuleCallback> for ModuleServer<Module> {
    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    fn chain_id(&self) -> RpcResult<ChainId<'static>> {
        Ok(self.ctx.chain_id.clone())
    }

    /// Query the latest finalized height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn query_latest_height(&self) -> RpcResult<Height> {
        self.ctx
            .beacon_api_client
            .finality_update()
            .await
            .map(|response| {
                self.ctx
                    .make_height(response.data.attested_header.beacon.slot)
            })
            .map_err(|err| ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>))
    }

    /// Query the latest (non-finalized) height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn query_latest_height_as_destination(&self) -> RpcResult<Height> {
        let height = self
            .ctx
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

        Ok(self.ctx.make_height(height))
    }

    /// Query the latest finalized timestamp of this chain.
    // TODO: Use a better timestamp type here
    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn query_latest_timestamp(&self) -> RpcResult<i64> {
        Ok(self
            .ctx
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

    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn fetch_block_range(
        &self,
        from_height: Height,
        to_height: Height,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        Ok(call(Call::plugin(
            self.ctx.plugin_name(),
            FetchEvents {
                from_height,
                to_height,
            },
        )))
    }

    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn client_info(&self, client_id: ClientId) -> RpcResult<ClientInfo> {
        Ok(ClientInfo {
            client_type: ClientType::new(
                self.ctx
                    .ibc_handler()
                    .client_types(client_id.to_string())
                    .await
                    .unwrap(),
            ),
            ibc_interface: IbcInterface::new(IbcInterface::IBC_SOLIDITY),
            metadata: Default::default(),
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn query_ibc_state(&self, at: Height, path: Path) -> RpcResult<Value> {
        self.ctx.fetch_ibc_state(path, at).await.map_err(|err| {
            ErrorObject::owned(
                -1,
                format!("error fetching ibc state: {}", ErrorReporter(&*err)),
                None::<()>,
            )
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn query_ibc_proof(&self, at: Height, path: Path) -> RpcResult<Value> {
        let location = ibc_commitment_key(&path.to_string(), IBC_HANDLER_COMMITMENTS_SLOT);

        let execution_height = self
            .ctx
            .execution_height_of_beacon_slot(at.revision_height)
            .await;

        let proof = self
            .ctx
            .provider
            .get_proof(
                ethers::types::H160::from(self.ctx.ibc_handler_address),
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

    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn query_raw_unfinalized_trusted_client_state(
        &self,
        client_id: ClientId,
    ) -> RpcResult<RawClientState<'static>> {
        let latest_execution_height = self.ctx.provider.get_block_number().await.unwrap().as_u64();

        let ClientInfo {
            client_type,
            ibc_interface,
            metadata: _,
        } = self.client_info(client_id.clone()).await?;

        Ok(RawClientState {
            client_type,
            ibc_interface,
            bytes: self
                .ctx
                .ibc_handler()
                .ibc_state_read(latest_execution_height, ClientStatePath { client_id })
                .await
                .unwrap()
                .0
                .into(),
        })
    }
}

// type Pls = <(<Module as ModuleContext>::Info, Module) as voyager_message::module::IntoRpc<
//     ModuleData,
//     ModuleCall,
//     ModuleCallback,
//     // RpcModule = ModuleServer<ModuleContext>,
// >>::RpcModule;

// static_assertions::assert_type_eq_all!(Pls, ModuleServer<Module>);

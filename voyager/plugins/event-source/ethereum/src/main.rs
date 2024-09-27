// #![warn(clippy::unwrap_used)] // oh boy this will be a lot of work

use std::collections::VecDeque;

use beacon_api::client::BeaconApiClient;
use chain_utils::ethereum::IBCHandlerEvents;
use contracts::{
    ibc_channel_handshake::IBCChannelHandshakeEvents, ibc_client::IBCClientEvents,
    ibc_connection::IBCConnectionEvents, ibc_packet::IBCPacketEvents,
};
use ethers::{
    contract::EthLogDecode,
    providers::{Middleware, Provider, Ws},
    types::Filter,
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use queue_msg::{call, conc, data, noop, optimize::OptimizationResult, seq, BoxDynError, Op};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument, trace, warn};
use unionlabs::{
    hash::{H160, H256},
    ibc::core::{channel, client::height::Height},
    ics24::{ChannelEndPath, ConnectionPath},
    id::{ChannelId, ClientId, PortId},
    uint::U256,
    ErrorReporter, QueryHeight,
};
use voyager_message::{
    call::{Call, WaitForHeight},
    core::{ChainId, ClientInfo},
    data::{
        ChainEvent, ChannelMetadata, ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit,
        ChannelOpenTry, ConnectionMetadata, ConnectionOpenAck, ConnectionOpenConfirm,
        ConnectionOpenInit, ConnectionOpenTry, CreateClient, Data, PacketMetadata, UpdateClient,
    },
    module::{ModuleInfo, PluginInfo, PluginServer, PluginTypes},
    rpc::{json_rpc_error_to_error_object, missing_state, VoyagerRpcClient, VoyagerRpcClientExt},
    run_module_server, DefaultCmd, ExtensionsExt, ModuleContext, VoyagerClient, VoyagerMessage,
};

use crate::{
    call::{FetchBlock, FetchGetLogs, MakeFullEvent, ModuleCall},
    callback::ModuleCallback,
    data::ModuleData,
};

pub mod call;
pub mod callback;
pub mod data;

const ETHEREUM_REVISION_NUMBER: u64 = 0;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_module_server::<Module>().await
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
    type Info = PluginInfo;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        Module::new(config).await
    }

    fn info(config: Self::Config) -> ModuleInfo<Self::Info> {
        ModuleInfo {
            kind: PluginInfo {
                name: plugin_name(&config.chain_id),
                interest_filter: format!(
                    r#"[.. | ."@type"? == "fetch_blocks" and ."@value".chain_id == "{}"] | any"#,
                    config.chain_id
                ),
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

    async fn make_packet_metadata(
        &self,
        event_height: Height,
        self_port_id: PortId,
        self_channel_id: ChannelId,
        voyager_rpc_client: &VoyagerClient,
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
            .map_err(json_rpc_error_to_error_object)?
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
            .map_err(json_rpc_error_to_error_object)?;

        let self_connection_state = self_connection
            .state
            .ok_or_else(missing_state("connection must exist", None))?;

        let client_info = voyager_rpc_client
            .client_info(
                self.chain_id.clone(),
                self_connection_state.client_id.clone(),
            )
            .await
            .map_err(json_rpc_error_to_error_object)?;

        let client_meta = voyager_rpc_client
            .client_meta(
                self.chain_id.clone(),
                event_height.into(),
                self_connection_state.client_id.clone(),
            )
            .await
            .map_err(json_rpc_error_to_error_object)?;

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
            .map_err(json_rpc_error_to_error_object)?;

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

impl PluginTypes for Module {
    type D = ModuleData;
    type C = ModuleCall;
    type Cb = ModuleCallback;
}

#[async_trait]
impl PluginServer<ModuleData, ModuleCall, ModuleCallback> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn run_pass(
        &self,
        _: &Extensions,
        msgs: Vec<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>>,
    ) -> RpcResult<OptimizationResult<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        Ok(OptimizationResult {
            optimize_further: vec![],
            ready: msgs
                .into_iter()
                .map(|op| match op {
                    Op::Call(Call::FetchBlocks(fetch)) if fetch.chain_id == self.chain_id => {
                        call(Call::plugin(
                            self.plugin_name(),
                            FetchBlock {
                                slot: fetch.start_height.revision_height,
                            },
                        ))
                    }
                    op => op,
                })
                .enumerate()
                .map(|(i, op)| (vec![i], op))
                .collect(),
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn callback(
        &self,
        _: &Extensions,
        cb: ModuleCallback,
        _data: VecDeque<Data<ModuleData>>,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match cb {}
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn call(
        &self,
        e: &Extensions,
        msg: ModuleCall,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match msg {
            ModuleCall::MakeFullEvent(MakeFullEvent {
                block_number,
                tx_hash,
                event,
            }) => {
                // TODO: Remove after https://github.com/unionlabs/union/issues/3039
                let slot = self
                    .beacon_api_client
                    .block(
                        self.provider
                            .get_block(block_number)
                            .await
                            .unwrap()
                            .unwrap()
                            .other
                            .get_with("parentBeaconBlockRoot", serde_json::from_value::<H256>)
                            .unwrap()
                            .unwrap()
                            .into(),
                    )
                    .await
                    .unwrap()
                    .data
                    .message
                    .slot;

                info!("execution block {block_number} is beacon slot {slot}");

                let height = self.make_height(slot);
                let voyager_client = e.try_get::<VoyagerClient>()?;

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
                        let connection = voyager_client
                            .query_ibc_state_typed(
                                self.chain_id.clone(),
                                height.into(),
                                ConnectionPath {
                                    connection_id: raw_event.connection_id.parse().unwrap(),
                                },
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?
                            .state
                            .ok_or_else(missing_state("connection must exist", None))?;

                        let client_info = voyager_client
                            .client_info(self.chain_id.clone(), connection.client_id.clone())
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        let client_meta = voyager_client
                            .client_meta(
                                self.chain_id.clone(),
                                height.into(),
                                connection.client_id.clone(),
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        let channel = voyager_client
                            .query_ibc_state_typed(
                                self.chain_id.clone(),
                                height.into(),
                                ChannelEndPath {
                                    port_id: raw_event.port_id.parse().unwrap(),
                                    channel_id: raw_event.channel_id.parse().unwrap(),
                                },
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?
                            .state
                            .ok_or_else(missing_state("connection must exist", None))?;

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
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
                        let connection = voyager_client
                            .query_ibc_state_typed(
                                self.chain_id.clone(),
                                height.into(),
                                ConnectionPath {
                                    connection_id: raw_event.connection_id.parse().unwrap(),
                                },
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?
                            .state
                            .ok_or_else(missing_state("connection must exist", None))?;

                        let client_info = voyager_client
                            .client_info(self.chain_id.clone(), connection.client_id.clone())
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        let client_meta = voyager_client
                            .client_meta(
                                self.chain_id.clone(),
                                height.into(),
                                connection.client_id.clone(),
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        let channel = voyager_client
                            .query_ibc_state_typed(
                                self.chain_id.clone(),
                                height.into(),
                                ChannelEndPath {
                                    port_id: raw_event.port_id.parse().unwrap(),
                                    channel_id: raw_event.channel_id.parse().unwrap(),
                                },
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?
                            .state
                            .ok_or_else(missing_state("channel must exist", None))?;

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
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
                        let connection = voyager_client
                            .query_ibc_state_typed(
                                self.chain_id.clone(),
                                height.into(),
                                ConnectionPath {
                                    connection_id: raw_event.connection_id.parse().unwrap(),
                                },
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?
                            .state
                            .ok_or_else(missing_state("connection must exist", None))?;

                        let client_info = voyager_client
                            .client_info(self.chain_id.clone(), connection.client_id.clone())
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        let client_meta = voyager_client
                            .client_meta(
                                self.chain_id.clone(),
                                height.into(),
                                connection.client_id.clone(),
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        let channel = voyager_client
                            .query_ibc_state_typed(
                                self.chain_id.clone(),
                                height.into(),
                                ChannelEndPath {
                                    port_id: raw_event.port_id.parse().unwrap(),
                                    channel_id: raw_event.channel_id.parse().unwrap(),
                                },
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?
                            .state
                            .ok_or_else(missing_state("channel must exist", None))?;

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
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
                        let connection = voyager_client
                            .query_ibc_state_typed(
                                self.chain_id.clone(),
                                height.into(),
                                ConnectionPath {
                                    connection_id: raw_event.connection_id.parse().unwrap(),
                                },
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?
                            .state
                            .ok_or_else(missing_state("connection must exist", None))?;

                        let client_info = voyager_client
                            .client_info(self.chain_id.clone(), connection.client_id.clone())
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        let client_meta = voyager_client
                            .client_meta(
                                self.chain_id.clone(),
                                height.into(),
                                connection.client_id.clone(),
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        let channel = voyager_client
                            .query_ibc_state_typed(
                                self.chain_id.clone(),
                                height.into(),
                                ChannelEndPath {
                                    port_id: raw_event.port_id.parse().unwrap(),
                                    channel_id: raw_event.channel_id.parse().unwrap(),
                                },
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?
                            .state
                            .ok_or_else(missing_state("channel must exist", None))?;

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
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
                        let client_info = voyager_client
                            .client_info(
                                self.chain_id.clone(),
                                raw_event.client_id.parse().unwrap(),
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        let client_meta = voyager_client
                            .client_meta(
                                self.chain_id.clone(),
                                height.into(),
                                raw_event.client_id.parse().unwrap(),
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
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
                        let client_info = voyager_client
                            .client_info(
                                self.chain_id.clone(),
                                raw_event.client_id.parse().unwrap(),
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        let client_meta = voyager_client
                            .client_meta(
                                self.chain_id.clone(),
                                height.into(),
                                raw_event.client_id.parse().unwrap(),
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
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
                        let client_info = voyager_client
                            .client_info(
                                self.chain_id.clone(),
                                raw_event.client_id.parse().unwrap(),
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        let client_meta = voyager_client
                            .client_meta(
                                self.chain_id.clone(),
                                height.into(),
                                raw_event.client_id.parse().unwrap(),
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
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
                        let client_info = voyager_client
                            .client_info(
                                self.chain_id.clone(),
                                raw_event.client_id.parse().unwrap(),
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        let client_meta = voyager_client
                            .client_meta(
                                self.chain_id.clone(),
                                height.into(),
                                raw_event.client_id.parse().unwrap(),
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
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

                        let client_info = voyager_client
                            .client_info(self.chain_id.clone(), client_id.clone())
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        let client_meta = voyager_client
                            .client_meta(
                                self.chain_id.clone(),
                                height.into(),
                                raw_event.client_id.parse().unwrap(),
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
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

                        let client_info = voyager_client
                            .client_info(self.chain_id.clone(), client_id.clone())
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        let client_meta = voyager_client
                            .client_meta(
                                self.chain_id.clone(),
                                height.into(),
                                raw_event.client_id.parse().unwrap(),
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
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
                                    .make_packet_metadata(
                                        height,
                                        event.source_port.parse().unwrap(),
                                        event.source_channel.parse().unwrap(),
                                        e.try_get()?,
                                    )
                                    .await?;

                                Ok(data(ChainEvent {
                                    chain_id: self.chain_id.clone(),
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
                                    .make_packet_metadata(
                                        height,
                                        event.packet.source_port.parse().unwrap(),
                                        event.packet.source_channel.parse().unwrap(),
                                        e.try_get()?,
                                    )
                                    .await?;

                                Ok(data(ChainEvent {
                                    chain_id: self.chain_id.clone(),
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
                                    .make_packet_metadata(
                                        height,
                                        event.packet.source_port.parse().unwrap(),
                                        event.packet.source_channel.parse().unwrap(),
                                        e.try_get()?,
                                    )
                                    .await?;

                                Ok(data(ChainEvent {
                                    chain_id: self.chain_id.clone(),
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
                                    .make_packet_metadata(
                                        height,
                                        event.packet.destination_port.parse().unwrap(),
                                        event.packet.destination_channel.parse().unwrap(),
                                        e.try_get()?,
                                    )
                                    .await?;

                                Ok(data(ChainEvent {
                                    chain_id: self.chain_id.clone(),
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
                                    .make_packet_metadata(
                                        height,
                                        event.packet.destination_port.parse().unwrap(),
                                        event.packet.destination_channel.parse().unwrap(),
                                        e.try_get()?,
                                    )
                                    .await?;

                                Ok(data(ChainEvent {
                                    chain_id: self.chain_id.clone(),
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
            ModuleCall::FetchBlock(FetchBlock { slot }) => {
                debug!(%slot, "fetching beacon slot");

                info!(%slot, "querying slot");

                match self
                    .beacon_api_client
                    .block(beacon_api::client::BlockId::Slot(slot))
                    .await
                {
                    // if the slot was missed, just try to get the next block
                    Err(beacon_api::errors::Error::NotFound(
                        beacon_api::errors::NotFoundError {
                            message,
                            error,
                            status_code,
                        },
                    )) => {
                        info!(%slot, %message, %error, %status_code, "beacon block not found for slot");

                        Ok(call(Call::plugin(
                            self.plugin_name(),
                            FetchBlock { slot: slot + 1 },
                        )))
                    }
                    Err(err) => Err(ErrorObject::owned(
                        -1,
                        format!(
                            "error fetching beacon block for slot {slot}: {}",
                            ErrorReporter(err)
                        ),
                        None::<()>,
                    )),
                    // TODO: Check block.extra.finalized once we do https://github.com/unionlabs/union/issues/3041
                    Ok(_) => {
                        // if the block is finalized, fetch events from this block and then try the next block
                        if self
                            .beacon_api_client
                            .finality_update()
                            .await
                            .unwrap()
                            .data
                            .attested_header
                            .beacon
                            .slot
                            >= slot
                        {
                            info!(%slot, "block is finalized");

                            Ok(conc([
                                call(Call::plugin(
                                    self.plugin_name(),
                                    FetchGetLogs {
                                        block_number: self
                                            .beacon_api_client
                                            .execution_height(slot.into())
                                            .await
                                            .map_err(|e| {
                                                ErrorObject::owned(
                                                    -1,
                                                    format!(
                                                        "error fetching execution block \
                                                        number of beacon slot {slot}: {}",
                                                        ErrorReporter(e)
                                                    ),
                                                    None::<()>,
                                                )
                                            })?,
                                    },
                                )),
                                call(Call::plugin(
                                    self.plugin_name(),
                                    FetchBlock { slot: slot + 1 },
                                )),
                            ]))
                        } else {
                            // if the block is NOT finalized, wait for it to be finalized
                            // note that this is the only time we queue a wait - this is to reduce the amount of times a message needs to go through the queue. if we're catching up on fetching blocks, this will significantly reduce the amount of times messages need to be processed before producing a `ChainEvent`.

                            info!(%slot, "block is not finalized");

                            Ok(seq([
                                call(WaitForHeight {
                                    chain_id: self.chain_id.clone(),
                                    height: self.make_height(slot),
                                }),
                                call(Call::plugin(self.plugin_name(), FetchBlock { slot })),
                            ]))
                        }
                    }
                }
            }
            ModuleCall::FetchGetLogs(FetchGetLogs { block_number }) => {
                debug!(%block_number, "fetching logs in execution block");

                let logs = self
                    .provider
                    .get_logs(
                        &Filter::new()
                            .address(ethers::types::H160::from(self.ibc_handler_address))
                            .from_block(block_number)
                            .to_block(block_number),
                    )
                    .await
                    .map_err(|e| {
                        ErrorObject::owned(
                            -1,
                            format!(
                                "error fetching logs in block {block_number}: {}",
                                ErrorReporter(e)
                            ),
                            None::<()>,
                        )
                    })?;

                info!(%block_number, "found {} logs", logs.len());

                Ok(conc(logs.into_iter().flat_map(|log| {
                    let tx_hash = log
                        .transaction_hash
                        .expect("log should have transaction_hash")
                        .into();

                    match IBCHandlerEvents::decode_log(&log.clone().into()) {
                        Ok(event) => {
                            trace!(?event, "found IBCHandler event");

                            Some(call(Call::plugin(
                                self.plugin_name(),
                                MakeFullEvent {
                                    block_number,
                                    tx_hash,
                                    event,
                                },
                            )))
                        }
                        Err(e) => {
                            warn!(
                                ?log,
                                "could not decode IBCHandler event: {}",
                                ErrorReporter(e)
                            );
                            None
                        }
                    }
                })))
            }
        }
    }
}

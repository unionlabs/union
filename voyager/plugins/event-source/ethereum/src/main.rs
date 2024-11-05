// #![warn(clippy::unwrap_used)] // oh boy this will be a lot of work

use std::collections::VecDeque;

use alloy::{
    providers::{Provider, ProviderBuilder, RootProvider},
    rpc::types::Filter,
    sol_types::SolEventInterface,
    transports::BoxTransport,
};
use beacon_api::client::BeaconApiClient;
use ibc_solidity::ibc::{
    self,
    Ibc::{self, IbcInstance},
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument, trace, warn};
use unionlabs::{
    hash::H160,
    ibc::core::{channel, client::height::Height},
    id::{ChannelId, ClientId, ConnectionId, PortId},
    ErrorReporter,
};
use voyager_message::{
    call::Call,
    core::{ChainId, ClientInfo, QueryHeight},
    data::{
        ChainEvent, ChannelMetadata, ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit,
        ChannelOpenTry, ConnectionMetadata, ConnectionOpenAck, ConnectionOpenConfirm,
        ConnectionOpenInit, ConnectionOpenTry, CreateClient, Data, PacketMetadata, UpdateClient,
    },
    module::{PluginInfo, PluginServer},
    rpc::{json_rpc_error_to_error_object, missing_state, VoyagerRpcClient},
    run_plugin_server, DefaultCmd, ExtensionsExt, Plugin, PluginMessage, VoyagerClient,
    VoyagerMessage, FATAL_JSONRPC_ERROR_CODE,
};
use voyager_vm::{call, conc, data, defer, noop, now, pass::PassResult, seq, BoxDynError, Op};

use crate::{
    call::{FetchGetLogs, IbcEvents, MakeFullEvent, ModuleCall},
    callback::ModuleCallback,
};

pub mod call;
pub mod callback;
pub mod data;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_plugin_server::<Module>().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId<'static>,

    pub ibc_handler_address: H160,

    pub provider: RootProvider<BoxTransport>,
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

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        Module::new(config).await
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.chain_id),
            interest_filter: format!(
                r#"[.. | ."@type"? == "fetch_blocks" and ."@value".chain_id == "{}"] | any"#,
                config.chain_id
            ),
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

    fn ibc_handler(&self) -> IbcInstance<BoxTransport, RootProvider<BoxTransport>> {
        Ibc::new(self.ibc_handler_address.get().into(), self.provider.clone())
    }

    pub async fn new(config: Config) -> Result<Self, BoxDynError> {
        let provider = ProviderBuilder::new()
            .on_builtin(&config.eth_rpc_api)
            .await?;

        let chain_id = provider.get_chain_id().await?;

        Ok(Self {
            chain_id: ChainId::new(chain_id.to_string()),
            ibc_handler_address: config.ibc_handler_address,
            provider,
            beacon_api_client: BeaconApiClient::new(config.eth_beacon_rpc_api).await?,
        })
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
            .query_channel(
                self.chain_id.clone(),
                event_height.into(),
                self_port_id.clone(),
                self_channel_id.clone(),
            )
            .await
            .map_err(json_rpc_error_to_error_object)?
            .state
            .ok_or_else(missing_state("connection must exist", None))?;

        let self_connection_id = self_channel.connection_hops[0].clone();
        let self_connection = voyager_rpc_client
            .query_connection(
                self.chain_id.clone(),
                event_height.into(),
                self_connection_id.clone(),
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

        let other_port_id = self_channel.counterparty.port_id.clone();
        let other_channel_id = self_channel.counterparty.channel_id.unwrap();

        let other_channel = voyager_rpc_client
            .query_channel(
                client_meta.chain_id.clone(),
                QueryHeight::Latest,
                other_port_id.clone(),
                other_channel_id.clone(),
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
                connection_id: self_connection_id,
            },
        };
        let destination_channel = ChannelMetadata {
            port_id: other_port_id.clone(),
            channel_id: other_channel_id.clone(),
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
impl PluginServer<ModuleCall, ModuleCallback> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn run_pass(
        &self,
        _: &Extensions,
        msgs: Vec<Op<VoyagerMessage>>,
    ) -> RpcResult<PassResult<VoyagerMessage>> {
        Ok(PassResult {
            optimize_further: vec![],
            ready: msgs
                .into_iter()
                .map(|op| match op {
                    Op::Call(Call::FetchBlocks(fetch)) if fetch.chain_id == self.chain_id => {
                        call(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(FetchGetLogs {
                                block_number: fetch.start_height.height(),
                                up_to: None,
                            }),
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
        _data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match cb {}
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn call(&self, e: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::MakeFullEvent(MakeFullEvent {
                block_number,
                tx_hash,
                event,
            }) => {
                let provable_height = Height::new(block_number);
                let voyager_client = e.try_get::<VoyagerClient>()?;

                match event {
                    IbcEvents::ClientCreated(raw_event) => {
                        let client_id = ClientId::new(raw_event.clientType, raw_event.clientId);

                        let client_info = voyager_client
                            .client_info(self.chain_id.clone(), client_id.clone())
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        let client_meta = voyager_client
                            .client_meta(
                                self.chain_id.clone(),
                                provable_height.into(),
                                client_id.clone(),
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
                            client_info: client_info.clone(),
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height,
                            event: CreateClient {
                                client_id,
                                client_type: client_info.client_type,
                                consensus_height: client_meta.height,
                            }
                            .into(),
                        }))
                    }
                    IbcEvents::ClientRegistered(raw_event) => {
                        info!(?raw_event, "observed ClientRegistered event");

                        Ok(noop())
                    }
                    IbcEvents::ClientUpdated(raw_event) => {
                        let client_type = self
                            .ibc_handler()
                            .clientTypes(raw_event.clientId)
                            .call()
                            .await
                            .unwrap()
                            ._0;

                        let client_id = ClientId::new(client_type, raw_event.clientId);

                        let client_info = voyager_client
                            .client_info(self.chain_id.clone(), client_id.clone())
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        let client_meta = voyager_client
                            .client_meta(
                                self.chain_id.clone(),
                                provable_height.into(),
                                client_id.clone(),
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
                            client_info: client_info.clone(),
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height,
                            event: UpdateClient {
                                client_id,
                                client_type: client_info.client_type,
                                consensus_heights: vec![Height::new(raw_event.height)],
                            }
                            .into(),
                        }))
                    }

                    IbcEvents::ConnectionOpenInit(raw_event) => {
                        let ibc_handler = self.ibc_handler();

                        let client_type = ibc_handler
                            .clientTypes(raw_event.clientId)
                            .call()
                            .await
                            .unwrap()
                            ._0;

                        let client_id = ClientId::new(client_type, raw_event.clientId);

                        let client_info = voyager_client
                            .client_info(self.chain_id.clone(), client_id.clone())
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        let client_meta = voyager_client
                            .client_meta(
                                self.chain_id.clone(),
                                provable_height.into(),
                                client_id.clone(),
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        let counterparty_client_prefix = e
                            .try_get::<VoyagerClient>()?
                            .query_client_prefix(
                                client_meta.chain_id.clone(),
                                raw_event.counterpartyClientId,
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
                            client_info,
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height,
                            event: ConnectionOpenInit {
                                client_id,
                                connection_id: ConnectionId::new(raw_event.connectionId),
                                counterparty_client_id: ClientId::new(
                                    counterparty_client_prefix,
                                    raw_event.counterpartyClientId,
                                ),
                            }
                            .into(),
                        }))
                    }
                    IbcEvents::ConnectionOpenTry(raw_event) => {
                        let client_type = self
                            .ibc_handler()
                            .clientTypes(raw_event.clientId)
                            .call()
                            .await
                            .unwrap()
                            ._0;

                        let client_id = ClientId::new(client_type, raw_event.clientId);

                        let client_info = voyager_client
                            .client_info(self.chain_id.clone(), client_id.clone())
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        let client_meta = voyager_client
                            .client_meta(
                                self.chain_id.clone(),
                                provable_height.into(),
                                client_id.clone(),
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        let counterparty_client_prefix = e
                            .try_get::<VoyagerClient>()?
                            .query_client_prefix(
                                client_meta.chain_id.clone(),
                                raw_event.counterpartyClientId,
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
                            client_info,
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height,
                            event: ConnectionOpenTry {
                                client_id,
                                connection_id: ConnectionId::new(raw_event.connectionId),
                                counterparty_client_id: ClientId::new(
                                    counterparty_client_prefix,
                                    raw_event.counterpartyClientId,
                                ),
                                counterparty_connection_id: ConnectionId::new(
                                    raw_event.counterpartyConnectionId,
                                ),
                            }
                            .into(),
                        }))
                    }
                    IbcEvents::ConnectionOpenAck(raw_event) => {
                        let client_type = self
                            .ibc_handler()
                            .clientTypes(raw_event.clientId)
                            .call()
                            .await
                            .unwrap()
                            ._0;

                        let client_id = ClientId::new(client_type, raw_event.clientId);

                        let client_info = voyager_client
                            .client_info(self.chain_id.clone(), client_id.clone())
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        let client_meta = voyager_client
                            .client_meta(
                                self.chain_id.clone(),
                                provable_height.into(),
                                client_id.clone(),
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        let counterparty_client_prefix = e
                            .try_get::<VoyagerClient>()?
                            .query_client_prefix(
                                client_meta.chain_id.clone(),
                                raw_event.counterpartyClientId,
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
                            client_info,
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height,
                            event: ConnectionOpenAck {
                                client_id,
                                connection_id: ConnectionId::new(raw_event.connectionId),
                                counterparty_client_id: ClientId::new(
                                    counterparty_client_prefix,
                                    raw_event.counterpartyClientId,
                                ),
                                counterparty_connection_id: ConnectionId::new(
                                    raw_event.counterpartyConnectionId,
                                ),
                            }
                            .into(),
                        }))
                    }
                    IbcEvents::ConnectionOpenConfirm(raw_event) => {
                        let client_type = self
                            .ibc_handler()
                            .clientTypes(raw_event.clientId)
                            .call()
                            .await
                            .unwrap()
                            ._0;

                        let client_id = ClientId::new(client_type, raw_event.clientId);
                        let client_info = voyager_client
                            .client_info(self.chain_id.clone(), client_id.clone())
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        let client_meta = voyager_client
                            .client_meta(
                                self.chain_id.clone(),
                                provable_height.into(),
                                client_id.clone(),
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        let counterparty_client_prefix = e
                            .try_get::<VoyagerClient>()?
                            .query_client_prefix(
                                client_meta.chain_id.clone(),
                                raw_event.counterpartyClientId,
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
                            client_info,
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height,
                            event: ConnectionOpenConfirm {
                                client_id,
                                connection_id: ConnectionId::new(raw_event.connectionId),
                                counterparty_client_id: ClientId::new(
                                    counterparty_client_prefix,
                                    raw_event.counterpartyClientId,
                                ),
                                counterparty_connection_id: ConnectionId::new(
                                    raw_event.counterpartyConnectionId,
                                ),
                            }
                            .into(),
                        }))
                    }
                    IbcEvents::ChannelOpenInit(raw_event) => {
                        let connection = voyager_client
                            .query_connection(
                                self.chain_id.clone(),
                                provable_height.into(),
                                ConnectionId::new(raw_event.connectionId),
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
                                provable_height.into(),
                                connection.client_id.clone(),
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        let port_id = PortId::new(raw_event.portId).unwrap();
                        let channel_id = ChannelId::new(raw_event.channelId);

                        let channel = voyager_client
                            .query_channel(
                                self.chain_id.clone(),
                                provable_height.into(),
                                port_id.clone(),
                                channel_id.clone(),
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
                            provable_height,
                            event: ChannelOpenInit {
                                port_id,
                                channel_id,
                                counterparty_port_id: PortId::new(raw_event.counterpartyPortId)
                                    .unwrap(),
                                connection,
                                version: channel.version,
                            }
                            .into(),
                        }))
                    }
                    IbcEvents::ChannelOpenTry(raw_event) => {
                        let connection = voyager_client
                            .query_connection(
                                self.chain_id.clone(),
                                provable_height.into(),
                                ConnectionId::new(raw_event.connectionId),
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
                                provable_height.into(),
                                connection.client_id.clone(),
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        let port_id = PortId::new(raw_event.portId).unwrap();
                        let channel_id = ChannelId::new(raw_event.channelId);

                        let channel = voyager_client
                            .query_channel(
                                self.chain_id.clone(),
                                provable_height.into(),
                                port_id.clone(),
                                channel_id.clone(),
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
                            provable_height,
                            event: ChannelOpenTry {
                                port_id,
                                channel_id,
                                counterparty_port_id: PortId::new(raw_event.counterpartyPortId)
                                    .unwrap(),
                                counterparty_channel_id: ChannelId::new(
                                    raw_event.counterpartyChannelId,
                                ),
                                connection,
                                version: channel.version,
                            }
                            .into(),
                        }))
                    }
                    IbcEvents::ChannelOpenAck(raw_event) => {
                        let connection = voyager_client
                            .query_connection(
                                self.chain_id.clone(),
                                provable_height.into(),
                                ConnectionId::new(raw_event.connectionId),
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
                                provable_height.into(),
                                connection.client_id.clone(),
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        let port_id = PortId::new(raw_event.portId).unwrap();
                        let channel_id = ChannelId::new(raw_event.channelId);

                        let channel = voyager_client
                            .query_channel(
                                self.chain_id.clone(),
                                provable_height.into(),
                                port_id.clone(),
                                channel_id.clone(),
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
                            provable_height,
                            event: ChannelOpenAck {
                                port_id,
                                channel_id,
                                counterparty_port_id: PortId::new(raw_event.counterpartyPortId)
                                    .unwrap(),
                                counterparty_channel_id: ChannelId::new(
                                    raw_event.counterpartyChannelId,
                                ),
                                connection,
                                version: channel.version,
                            }
                            .into(),
                        }))
                    }
                    IbcEvents::ChannelOpenConfirm(raw_event) => {
                        let connection = voyager_client
                            .query_connection(
                                self.chain_id.clone(),
                                provable_height.into(),
                                ConnectionId::new(raw_event.connectionId),
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
                                provable_height.into(),
                                connection.client_id.clone(),
                            )
                            .await
                            .map_err(json_rpc_error_to_error_object)?;

                        let port_id = PortId::new(raw_event.portId).unwrap();
                        let channel_id = ChannelId::new(raw_event.channelId);

                        let channel = voyager_client
                            .query_channel(
                                self.chain_id.clone(),
                                provable_height.into(),
                                port_id.clone(),
                                channel_id.clone(),
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
                            provable_height,
                            event: ChannelOpenConfirm {
                                port_id,
                                channel_id,
                                counterparty_port_id: channel.counterparty.port_id,
                                counterparty_channel_id: channel.counterparty.channel_id.unwrap(),
                                connection,
                                version: channel.version,
                            }
                            .into(),
                        }))
                    }

                    IbcEvents::ChannelCloseInit(_) | IbcEvents::ChannelCloseConfirm(_) => {
                        warn!("observed channel close message, these are not handled currently");

                        Ok(noop())
                    }

                    // packet origin is this chain
                    IbcEvents::SendPacket(event) => {
                        let self_port_id = PortId::new(
                            self.ibc_handler()
                                .channelOwner(event.packet.sourceChannel)
                                .call()
                                .await
                                .unwrap()
                                ._0
                                .to_string(),
                        )
                        .unwrap();

                        let (
                            counterparty_chain_id,
                            client_info,
                            source_channel,
                            destination_channel,
                            channel_ordering,
                        ) = self
                            .make_packet_metadata(
                                provable_height,
                                self_port_id,
                                ChannelId::new(event.packet.sourceChannel),
                                e.try_get()?,
                            )
                            .await?;

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
                            client_info,
                            counterparty_chain_id,
                            tx_hash,
                            provable_height,
                            event: voyager_message::data::SendPacket {
                                packet_data: event.packet.data.to_vec().into(),
                                packet: PacketMetadata {
                                    sequence: event.packet.sequence.try_into().unwrap(),
                                    source_channel,
                                    destination_channel,
                                    channel_ordering,
                                    timeout_height: Height::new(event.packet.timeoutHeight),
                                    timeout_timestamp: event.packet.timeoutTimestamp,
                                },
                            }
                            .into(),
                        }))
                    }
                    IbcEvents::TimeoutPacket(event) => {
                        let self_port_id = PortId::new(
                            self.ibc_handler()
                                .channelOwner(event.packet.sourceChannel)
                                .call()
                                .await
                                .unwrap()
                                ._0
                                .to_string(),
                        )
                        .unwrap();

                        let (
                            counterparty_chain_id,
                            client_info,
                            source_channel,
                            destination_channel,
                            channel_ordering,
                        ) = self
                            .make_packet_metadata(
                                provable_height,
                                self_port_id,
                                ChannelId::new(event.packet.sourceChannel),
                                e.try_get()?,
                            )
                            .await?;

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
                            client_info,
                            counterparty_chain_id,
                            tx_hash,
                            provable_height,
                            event: voyager_message::data::TimeoutPacket {
                                packet: PacketMetadata {
                                    sequence: event.packet.sequence.try_into().unwrap(),
                                    source_channel,
                                    destination_channel,
                                    channel_ordering,
                                    timeout_height: Height::new(event.packet.timeoutHeight),
                                    timeout_timestamp: event.packet.timeoutTimestamp,
                                },
                            }
                            .into(),
                        }))
                    }
                    IbcEvents::AcknowledgePacket(event) => {
                        let self_port_id = PortId::new(
                            self.ibc_handler()
                                .channelOwner(event.packet.sourceChannel)
                                .call()
                                .await
                                .unwrap()
                                ._0
                                .to_string(),
                        )
                        .unwrap();

                        let (
                            counterparty_chain_id,
                            client_info,
                            source_channel,
                            destination_channel,
                            channel_ordering,
                        ) = self
                            .make_packet_metadata(
                                provable_height,
                                self_port_id,
                                ChannelId::new(event.packet.sourceChannel),
                                e.try_get()?,
                            )
                            .await?;

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
                            client_info,
                            counterparty_chain_id,
                            tx_hash,
                            provable_height,
                            event: voyager_message::data::AcknowledgePacket {
                                packet: PacketMetadata {
                                    sequence: event.packet.sequence.try_into().unwrap(),
                                    source_channel,
                                    destination_channel,
                                    channel_ordering,
                                    timeout_height: Height::new(event.packet.timeoutHeight),
                                    timeout_timestamp: event.packet.timeoutTimestamp,
                                },
                            }
                            .into(),
                        }))
                    }
                    // packet origin is the counterparty chain
                    IbcEvents::WriteAcknowledgement(event) => {
                        let self_port_id = PortId::new(
                            self.ibc_handler()
                                .channelOwner(event.packet.destinationChannel)
                                .call()
                                .await
                                .unwrap()
                                ._0
                                .to_string(),
                        )
                        .unwrap();

                        let (
                            counterparty_chain_id,
                            client_info,
                            destination_channel,
                            source_channel,
                            channel_ordering,
                        ) = self
                            .make_packet_metadata(
                                provable_height,
                                self_port_id,
                                ChannelId::new(event.packet.destinationChannel),
                                e.try_get()?,
                            )
                            .await?;

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
                            client_info,
                            counterparty_chain_id,
                            tx_hash,
                            provable_height,
                            event: voyager_message::data::WriteAcknowledgement {
                                packet_data: event.packet.data.to_vec().into(),
                                packet_ack: event.acknowledgement.to_vec().into(),
                                packet: PacketMetadata {
                                    sequence: event.packet.sequence.try_into().unwrap(),
                                    source_channel,
                                    destination_channel,
                                    channel_ordering,
                                    timeout_height: Height::new(event.packet.timeoutHeight),
                                    timeout_timestamp: event.packet.timeoutTimestamp,
                                },
                            }
                            .into(),
                        }))
                    }
                    IbcEvents::RecvPacket(event) => {
                        let self_port_id = PortId::new(
                            self.ibc_handler()
                                .channelOwner(event.packet.destinationChannel)
                                .call()
                                .await
                                .unwrap()
                                ._0
                                .to_string(),
                        )
                        .unwrap();

                        let (
                            counterparty_chain_id,
                            client_info,
                            destination_channel,
                            source_channel,
                            channel_ordering,
                        ) = self
                            .make_packet_metadata(
                                provable_height,
                                self_port_id,
                                ChannelId::new(event.packet.destinationChannel),
                                e.try_get()?,
                            )
                            .await?;

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
                            client_info,
                            counterparty_chain_id,
                            tx_hash,
                            provable_height,
                            event: voyager_message::data::RecvPacket {
                                packet_data: event.packet.data.to_vec().into(),
                                packet: PacketMetadata {
                                    sequence: event.packet.sequence.try_into().unwrap(),
                                    source_channel,
                                    destination_channel,
                                    channel_ordering,
                                    timeout_height: Height::new(event.packet.timeoutHeight),
                                    timeout_timestamp: event.packet.timeoutTimestamp,
                                },
                            }
                            .into(),
                        }))
                    }
                    IbcEvents::RecvIntentPacket(_event) => {
                        todo!()
                    }
                }
            }
            ModuleCall::FetchGetLogs(FetchGetLogs {
                block_number,
                up_to,
            }) => {
                if up_to.is_some_and(|up_to| up_to < block_number) {
                    return Err(ErrorObject::owned(
                        FATAL_JSONRPC_ERROR_CODE,
                        "`up_to` must be either > `block_number` or null",
                        None::<()>,
                    ));
                }

                let latest_height = e
                    .try_get::<VoyagerClient>()?
                    .query_latest_height(self.chain_id.clone(), true)
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                if latest_height.height() < block_number {
                    debug!(block_number, "block is not yet finalized");

                    return Ok(seq([
                        defer(now() + 1),
                        call(Call::Plugin(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(FetchGetLogs {
                                block_number,
                                up_to,
                            }),
                        ))),
                    ]));
                }

                debug!(%block_number, "fetching logs in execution block");

                let logs = self
                    .provider
                    .get_logs(
                        &Filter::new()
                            .address(alloy::primitives::Address::from(
                                self.ibc_handler_address.get(),
                            ))
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

                let events = logs.into_iter().flat_map(|log| {
                    let tx_hash = log
                        .transaction_hash
                        .expect("log should have transaction_hash")
                        .into();

                    match ibc::Ibc::IbcEvents::decode_log(&log.inner, true) {
                        Ok(event) => {
                            trace!(?event, "found IbcHandler event");

                            Some(call(PluginMessage::new(
                                self.plugin_name(),
                                ModuleCall::from(MakeFullEvent {
                                    block_number,
                                    tx_hash,
                                    event: match event.data {
                                        Ibc::IbcEvents::ClientRegistered(client_registered) => {
                                            IbcEvents::ClientRegistered(client_registered)
                                        }
                                        Ibc::IbcEvents::ClientCreated(client_created) => {
                                            IbcEvents::ClientCreated(client_created)
                                        }
                                        Ibc::IbcEvents::ClientUpdated(client_updated) => {
                                            IbcEvents::ClientUpdated(client_updated)
                                        }
                                        Ibc::IbcEvents::ConnectionOpenInit(
                                            connection_open_init,
                                        ) => IbcEvents::ConnectionOpenInit(connection_open_init),
                                        Ibc::IbcEvents::ConnectionOpenTry(connection_open_try) => {
                                            IbcEvents::ConnectionOpenTry(connection_open_try)
                                        }
                                        Ibc::IbcEvents::ConnectionOpenAck(connection_open_ack) => {
                                            IbcEvents::ConnectionOpenAck(connection_open_ack)
                                        }
                                        Ibc::IbcEvents::ConnectionOpenConfirm(
                                            connection_open_confirm,
                                        ) => IbcEvents::ConnectionOpenConfirm(
                                            connection_open_confirm,
                                        ),
                                        Ibc::IbcEvents::ChannelOpenInit(channel_open_init) => {
                                            IbcEvents::ChannelOpenInit(channel_open_init)
                                        }
                                        Ibc::IbcEvents::ChannelOpenTry(channel_open_try) => {
                                            IbcEvents::ChannelOpenTry(channel_open_try)
                                        }
                                        Ibc::IbcEvents::ChannelOpenAck(channel_open_ack) => {
                                            IbcEvents::ChannelOpenAck(channel_open_ack)
                                        }
                                        Ibc::IbcEvents::ChannelOpenConfirm(
                                            channel_open_confirm,
                                        ) => IbcEvents::ChannelOpenConfirm(channel_open_confirm),
                                        Ibc::IbcEvents::ChannelCloseInit(channel_close_init) => {
                                            IbcEvents::ChannelCloseInit(channel_close_init)
                                        }
                                        Ibc::IbcEvents::ChannelCloseConfirm(
                                            channel_close_confirm,
                                        ) => IbcEvents::ChannelCloseConfirm(channel_close_confirm),
                                        Ibc::IbcEvents::SendPacket(send_packet) => {
                                            IbcEvents::SendPacket(send_packet)
                                        }
                                        Ibc::IbcEvents::RecvPacket(recv_packet) => {
                                            IbcEvents::RecvPacket(recv_packet)
                                        }
                                        Ibc::IbcEvents::RecvIntentPacket(recv_intent_packet) => {
                                            IbcEvents::RecvIntentPacket(recv_intent_packet)
                                        }
                                        Ibc::IbcEvents::WriteAcknowledgement(
                                            write_acknowledgement,
                                        ) => IbcEvents::WriteAcknowledgement(write_acknowledgement),
                                        Ibc::IbcEvents::AcknowledgePacket(acknowledge_packet) => {
                                            IbcEvents::AcknowledgePacket(acknowledge_packet)
                                        }
                                        Ibc::IbcEvents::TimeoutPacket(timeout_packet) => {
                                            IbcEvents::TimeoutPacket(timeout_packet)
                                        }
                                    },
                                }),
                            )))
                        }
                        Err(e) => {
                            warn!(
                                ?log,
                                "could not decode IbcHandler event: {}",
                                ErrorReporter(e)
                            );
                            None
                        }
                    }
                });

                let next_fetch = match up_to {
                    Some(up_to) => {
                        if up_to > block_number {
                            Some(call(Call::Plugin(PluginMessage::new(
                                self.plugin_name(),
                                ModuleCall::from(FetchGetLogs {
                                    block_number: block_number + 1,
                                    up_to: Some(up_to),
                                }),
                            ))))
                        } else {
                            None
                        }
                    }
                    None => Some(call(Call::Plugin(PluginMessage::new(
                        self.plugin_name(),
                        ModuleCall::from(FetchGetLogs {
                            block_number: block_number + 1,
                            up_to: None,
                        }),
                    )))),
                };

                Ok(conc(next_fetch.into_iter().chain(events)))
            }
        }
    }
}

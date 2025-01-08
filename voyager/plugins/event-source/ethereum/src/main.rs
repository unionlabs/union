#![warn(clippy::unwrap_used)]

use std::collections::VecDeque;

use alloy::{
    providers::{Provider, ProviderBuilder, RootProvider},
    rpc::types::Filter,
    sol_types::SolEventInterface,
    transports::BoxTransport,
};
use beacon_api::client::BeaconApiClient;
use ibc_solidity::Ibc;
use ibc_union_spec::{
    ChannelMetadata, ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit, ChannelOpenTry,
    ChannelPath, ConnectionMetadata, ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit,
    ConnectionOpenTry, ConnectionPath, CreateClient, FullEvent, IbcUnion, PacketAck,
    PacketMetadata, PacketRecv, PacketSend, PacketTimeout, UpdateClient, WriteAck,
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument, trace, warn};
use unionlabs::{ibc::core::client::height::Height, primitives::H160, ErrorReporter};
use voyager_message::{
    call::Call,
    core::{ChainId, ClientInfo, IbcSpec, QueryHeight},
    data::{ChainEvent, Data},
    into_value,
    module::{PluginInfo, PluginServer},
    rpc::missing_state,
    DefaultCmd, ExtensionsExt, Plugin, PluginMessage, VoyagerClient, VoyagerMessage,
    FATAL_JSONRPC_ERROR_CODE,
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
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub ibc_handler_address: H160,

    pub provider: RootProvider<BoxTransport>,
    pub beacon_api_client: BeaconApiClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The expected chain id of this ethereum-like chain.
    pub chain_id: ChainId,

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

fn plugin_name(chain_id: &ChainId) -> String {
    pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

    format!("{PLUGIN_NAME}/{}", chain_id)
}

impl Module {
    pub fn plugin_name(&self) -> String {
        plugin_name(&self.chain_id)
    }

    pub async fn new(config: Config) -> Result<Self, BoxDynError> {
        let provider = ProviderBuilder::new()
            .on_builtin(&config.eth_rpc_api)
            .await?;

        // TODO: Assert chain id is correct
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
        self_channel_id: u32,
        voyager_rpc_client: &VoyagerClient,
    ) -> RpcResult<(ChainId, ClientInfo, ChannelMetadata, ChannelMetadata)> {
        let self_channel = voyager_rpc_client
            .query_ibc_state(
                self.chain_id.clone(),
                event_height.into(),
                ChannelPath {
                    channel_id: self_channel_id,
                },
            )
            .await?
            .state
            .ok_or_else(missing_state("connection must exist", None))?;

        let self_connection_id = self_channel.connection_id;
        let self_connection = voyager_rpc_client
            .query_ibc_state(
                self.chain_id.clone(),
                event_height.into(),
                ConnectionPath {
                    connection_id: self_connection_id,
                },
            )
            .await?;

        let self_connection_state = self_connection
            .state
            .ok_or_else(missing_state("connection must exist", None))?;

        let client_info = voyager_rpc_client
            .client_info::<IbcUnion>(self.chain_id.clone(), self_connection_state.client_id)
            .await?;

        let client_meta = voyager_rpc_client
            .client_meta::<IbcUnion>(
                self.chain_id.clone(),
                event_height.into(),
                self_connection_state.client_id,
            )
            .await?;

        let other_channel_id = self_channel.counterparty_channel_id;

        let other_channel = voyager_rpc_client
            .query_ibc_state(
                client_meta.chain_id.clone(),
                QueryHeight::Latest,
                ChannelPath {
                    channel_id: other_channel_id,
                },
            )
            .await?;

        let other_channel_state = other_channel
            .state
            .ok_or_else(missing_state("channel must exist", None))?;

        let source_channel = ChannelMetadata {
            channel_id: self_channel_id,
            version: self_channel.version,
            connection: ConnectionMetadata {
                client_id: self_connection_state.client_id,
                connection_id: self_connection_id,
            },
        };
        let destination_channel = ChannelMetadata {
            channel_id: other_channel_id,
            version: other_channel_state.version,
            connection: ConnectionMetadata {
                client_id: self_connection_state.counterparty_client_id,
                connection_id: self_connection_state.counterparty_connection_id,
            },
        };

        Ok((
            client_meta.chain_id,
            client_info,
            source_channel,
            destination_channel,
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
                    IbcEvents::CreateClient(raw_event) => {
                        let client_info = voyager_client
                            .client_info::<IbcUnion>(self.chain_id.clone(), raw_event.client_id)
                            .await?;

                        let client_meta = voyager_client
                            .client_meta::<IbcUnion>(
                                self.chain_id.clone(),
                                provable_height.into(),
                                raw_event.client_id,
                            )
                            .await?;

                        let event = CreateClient {
                            client_id: raw_event.client_id,
                            client_type: client_info.client_type.clone(),
                        }
                        .into();

                        ibc_union_spec::log_event(&event, &self.chain_id);

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
                            client_info: client_info.clone(),
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height,
                            ibc_spec_id: IbcUnion::ID,
                            event: into_value::<FullEvent>(event),
                        }))
                    }
                    IbcEvents::RegisterClient(raw_event) => {
                        info!(?raw_event, "observed RegisterClient event");

                        Ok(noop())
                    }
                    IbcEvents::UpdateClient(raw_event) => {
                        let client_info = voyager_client
                            .client_info::<IbcUnion>(self.chain_id.clone(), raw_event.client_id)
                            .await?;

                        let client_meta = voyager_client
                            .client_meta::<IbcUnion>(
                                self.chain_id.clone(),
                                provable_height.into(),
                                raw_event.client_id,
                            )
                            .await?;

                        let event = UpdateClient {
                            client_type: client_info.client_type.clone(),
                            client_id: raw_event.client_id,
                            height: raw_event.height,
                        }
                        .into();

                        ibc_union_spec::log_event(&event, &self.chain_id);

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
                            client_info: client_info.clone(),
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height,
                            ibc_spec_id: IbcUnion::ID,
                            event: into_value::<FullEvent>(event),
                        }))
                    }

                    IbcEvents::ConnectionOpenInit(raw_event) => {
                        let client_info = voyager_client
                            .client_info::<IbcUnion>(self.chain_id.clone(), raw_event.client_id)
                            .await?;

                        let client_meta = voyager_client
                            .client_meta::<IbcUnion>(
                                self.chain_id.clone(),
                                provable_height.into(),
                                raw_event.client_id,
                            )
                            .await?;

                        let event = ConnectionOpenInit {
                            client_id: raw_event.client_id,
                            connection_id: raw_event.connection_id,
                            counterparty_client_id: raw_event.counterparty_client_id,
                        }
                        .into();

                        ibc_union_spec::log_event(&event, &self.chain_id);

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
                            client_info,
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            ibc_spec_id: IbcUnion::ID,
                            provable_height,
                            event: into_value::<FullEvent>(event),
                        }))
                    }
                    IbcEvents::ConnectionOpenTry(raw_event) => {
                        let client_info = voyager_client
                            .client_info::<IbcUnion>(self.chain_id.clone(), raw_event.client_id)
                            .await?;

                        let client_meta = voyager_client
                            .client_meta::<IbcUnion>(
                                self.chain_id.clone(),
                                provable_height.into(),
                                raw_event.client_id,
                            )
                            .await?;

                        let event = ConnectionOpenTry {
                            client_id: raw_event.client_id,
                            connection_id: raw_event.connection_id,
                            counterparty_client_id: raw_event.counterparty_client_id,
                            counterparty_connection_id: raw_event.counterparty_connection_id,
                        }
                        .into();

                        ibc_union_spec::log_event(&event, &self.chain_id);

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
                            client_info,
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height,
                            ibc_spec_id: IbcUnion::ID,
                            event: into_value::<FullEvent>(event),
                        }))
                    }
                    IbcEvents::ConnectionOpenAck(raw_event) => {
                        let client_info = voyager_client
                            .client_info::<IbcUnion>(self.chain_id.clone(), raw_event.client_id)
                            .await?;

                        let client_meta = voyager_client
                            .client_meta::<IbcUnion>(
                                self.chain_id.clone(),
                                provable_height.into(),
                                raw_event.client_id,
                            )
                            .await?;

                        let event = ConnectionOpenAck {
                            client_id: raw_event.client_id,
                            connection_id: raw_event.connection_id,
                            counterparty_client_id: raw_event.counterparty_client_id,
                            counterparty_connection_id: raw_event.counterparty_connection_id,
                        }
                        .into();

                        ibc_union_spec::log_event(&event, &self.chain_id);

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
                            client_info,
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height,
                            ibc_spec_id: IbcUnion::ID,
                            event: into_value::<FullEvent>(event),
                        }))
                    }
                    IbcEvents::ConnectionOpenConfirm(raw_event) => {
                        let client_info = voyager_client
                            .client_info::<IbcUnion>(self.chain_id.clone(), raw_event.client_id)
                            .await?;

                        let client_meta = voyager_client
                            .client_meta::<IbcUnion>(
                                self.chain_id.clone(),
                                provable_height.into(),
                                raw_event.client_id,
                            )
                            .await?;

                        let event = ConnectionOpenConfirm {
                            client_id: raw_event.client_id,
                            connection_id: raw_event.connection_id,
                            counterparty_client_id: raw_event.counterparty_client_id,
                            counterparty_connection_id: raw_event.counterparty_connection_id,
                        }
                        .into();

                        ibc_union_spec::log_event(&event, &self.chain_id);

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
                            client_info,
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height,
                            ibc_spec_id: IbcUnion::ID,
                            event: into_value::<FullEvent>(event),
                        }))
                    }
                    IbcEvents::ChannelOpenInit(raw_event) => {
                        let connection = voyager_client
                            .query_ibc_state(
                                self.chain_id.clone(),
                                provable_height.into(),
                                ConnectionPath {
                                    connection_id: raw_event.connection_id,
                                },
                            )
                            .await?
                            .state
                            .ok_or_else(missing_state("connection must exist", None))?;

                        let client_info = voyager_client
                            .client_info::<IbcUnion>(self.chain_id.clone(), connection.client_id)
                            .await?;

                        let client_meta = voyager_client
                            .client_meta::<IbcUnion>(
                                self.chain_id.clone(),
                                provable_height.into(),
                                connection.client_id,
                            )
                            .await?;

                        let channel_id = raw_event.channel_id;

                        let channel = voyager_client
                            .query_ibc_state(
                                self.chain_id.clone(),
                                provable_height.into(),
                                ChannelPath { channel_id },
                            )
                            .await?
                            .state
                            .ok_or_else(missing_state("connection must exist", None))?;

                        let event = ChannelOpenInit {
                            port_id: raw_event.port_id.into(),
                            channel_id,
                            counterparty_port_id: raw_event.counterparty_port_id.into(),
                            connection,
                            version: channel.version,
                        }
                        .into();

                        ibc_union_spec::log_event(&event, &self.chain_id);

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
                            client_info,
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height,
                            ibc_spec_id: IbcUnion::ID,
                            event: into_value::<FullEvent>(event),
                        }))
                    }
                    IbcEvents::ChannelOpenTry(raw_event) => {
                        let connection = voyager_client
                            .query_ibc_state(
                                self.chain_id.clone(),
                                provable_height.into(),
                                ConnectionPath {
                                    connection_id: raw_event.connection_id,
                                },
                            )
                            .await?
                            .state
                            .ok_or_else(missing_state("connection must exist", None))?;

                        let client_info = voyager_client
                            .client_info::<IbcUnion>(self.chain_id.clone(), connection.client_id)
                            .await?;

                        let client_meta = voyager_client
                            .client_meta::<IbcUnion>(
                                self.chain_id.clone(),
                                provable_height.into(),
                                connection.client_id,
                            )
                            .await?;

                        let channel_id = raw_event.channel_id;

                        let channel = voyager_client
                            .query_ibc_state(
                                self.chain_id.clone(),
                                provable_height.into(),
                                ChannelPath { channel_id },
                            )
                            .await?
                            .state
                            .ok_or_else(missing_state("channel must exist", None))?;

                        let event = ChannelOpenTry {
                            port_id: raw_event.port_id.into(),
                            channel_id,
                            counterparty_port_id: raw_event.counterparty_port_id.into(),
                            counterparty_channel_id: raw_event.counterparty_channel_id,
                            connection,
                            version: channel.version,
                        }
                        .into();

                        ibc_union_spec::log_event(&event, &self.chain_id);

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
                            client_info,
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height,
                            ibc_spec_id: IbcUnion::ID,
                            event: into_value::<FullEvent>(event),
                        }))
                    }
                    IbcEvents::ChannelOpenAck(raw_event) => {
                        let connection = voyager_client
                            .query_ibc_state(
                                self.chain_id.clone(),
                                provable_height.into(),
                                ConnectionPath {
                                    connection_id: raw_event.connection_id,
                                },
                            )
                            .await?
                            .state
                            .ok_or_else(missing_state("connection must exist", None))?;

                        let client_info = voyager_client
                            .client_info::<IbcUnion>(self.chain_id.clone(), connection.client_id)
                            .await?;

                        let client_meta = voyager_client
                            .client_meta::<IbcUnion>(
                                self.chain_id.clone(),
                                provable_height.into(),
                                connection.client_id,
                            )
                            .await?;

                        let channel_id = raw_event.channel_id;

                        let channel = voyager_client
                            .query_ibc_state(
                                self.chain_id.clone(),
                                provable_height.into(),
                                ChannelPath { channel_id },
                            )
                            .await?
                            .state
                            .ok_or_else(missing_state("channel must exist", None))?;

                        let event = ChannelOpenAck {
                            port_id: raw_event.port_id.into(),
                            channel_id,
                            counterparty_port_id: raw_event.counterparty_port_id.into(),
                            counterparty_channel_id: raw_event.counterparty_channel_id,
                            connection,
                            version: channel.version,
                        }
                        .into();

                        ibc_union_spec::log_event(&event, &self.chain_id);

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
                            client_info,
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height,
                            ibc_spec_id: IbcUnion::ID,
                            event: into_value::<FullEvent>(event),
                        }))
                    }
                    IbcEvents::ChannelOpenConfirm(raw_event) => {
                        let connection = voyager_client
                            .query_ibc_state(
                                self.chain_id.clone(),
                                provable_height.into(),
                                ConnectionPath {
                                    connection_id: raw_event.connection_id,
                                },
                            )
                            .await?
                            .state
                            .ok_or_else(missing_state("connection must exist", None))?;

                        let client_info = voyager_client
                            .client_info::<IbcUnion>(self.chain_id.clone(), connection.client_id)
                            .await?;

                        let client_meta = voyager_client
                            .client_meta::<IbcUnion>(
                                self.chain_id.clone(),
                                provable_height.into(),
                                connection.client_id,
                            )
                            .await?;

                        let channel_id = raw_event.channel_id;

                        let channel = voyager_client
                            .query_ibc_state(
                                self.chain_id.clone(),
                                provable_height.into(),
                                ChannelPath { channel_id },
                            )
                            .await?
                            .state
                            .ok_or_else(missing_state("channel must exist", None))?;

                        let event = ChannelOpenConfirm {
                            port_id: raw_event.port_id.into(),
                            channel_id,
                            counterparty_port_id: channel.counterparty_port_id.into(),
                            counterparty_channel_id: channel.counterparty_channel_id,
                            connection,
                            version: channel.version,
                        }
                        .into();

                        ibc_union_spec::log_event(&event, &self.chain_id);

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
                            client_info,
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height,
                            ibc_spec_id: IbcUnion::ID,
                            event: into_value::<FullEvent>(event),
                        }))
                    }

                    IbcEvents::ChannelCloseInit(_) | IbcEvents::ChannelCloseConfirm(_) => {
                        warn!("observed channel close message, these are not handled currently");

                        Ok(noop())
                    }

                    // packet origin is this chain
                    IbcEvents::PacketSend(event) => {
                        let (
                            counterparty_chain_id,
                            client_info,
                            source_channel,
                            destination_channel,
                        ) = self
                            .make_packet_metadata(
                                provable_height,
                                event.packet.source_channel,
                                e.try_get()?,
                            )
                            .await?;

                        let event = PacketSend {
                            packet_data: event.packet.data.to_vec().into(),
                            packet: PacketMetadata {
                                source_channel,
                                destination_channel,
                                timeout_height: event.packet.timeout_height,
                                timeout_timestamp: event.packet.timeout_timestamp,
                            },
                        }
                        .into();

                        ibc_union_spec::log_event(&event, &self.chain_id);

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
                            client_info,
                            counterparty_chain_id,
                            tx_hash,
                            provable_height,
                            ibc_spec_id: IbcUnion::ID,
                            event: into_value::<FullEvent>(event),
                        }))
                    }
                    IbcEvents::PacketTimeout(event) => {
                        let (
                            counterparty_chain_id,
                            client_info,
                            source_channel,
                            destination_channel,
                        ) = self
                            .make_packet_metadata(
                                provable_height,
                                event.packet.source_channel,
                                e.try_get()?,
                            )
                            .await?;

                        let event = PacketTimeout {
                            packet: PacketMetadata {
                                source_channel,
                                destination_channel,
                                timeout_height: event.packet.timeout_height,
                                timeout_timestamp: event.packet.timeout_timestamp,
                            },
                            packet_data: event.packet.data.into(),
                        }
                        .into();

                        ibc_union_spec::log_event(&event, &self.chain_id);

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
                            client_info,
                            counterparty_chain_id,
                            tx_hash,
                            provable_height,
                            ibc_spec_id: IbcUnion::ID,
                            event: into_value::<FullEvent>(event),
                        }))
                    }
                    IbcEvents::PacketAck(event) => {
                        let (
                            counterparty_chain_id,
                            client_info,
                            source_channel,
                            destination_channel,
                        ) = self
                            .make_packet_metadata(
                                provable_height,
                                event.packet.source_channel,
                                e.try_get()?,
                            )
                            .await?;

                        let event = PacketAck {
                            packet: PacketMetadata {
                                source_channel,
                                destination_channel,
                                timeout_height: event.packet.timeout_height,
                                timeout_timestamp: event.packet.timeout_timestamp,
                            },
                            packet_data: event.packet.data.into(),
                            acknowledgement: event.acknowledgement.into(),
                        }
                        .into();

                        ibc_union_spec::log_event(&event, &self.chain_id);

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
                            client_info,
                            counterparty_chain_id,
                            tx_hash,
                            provable_height,
                            ibc_spec_id: IbcUnion::ID,
                            event: into_value::<FullEvent>(event),
                        }))
                    }
                    // packet origin is the counterparty chain
                    IbcEvents::WriteAck(event) => {
                        let (
                            counterparty_chain_id,
                            client_info,
                            destination_channel,
                            source_channel,
                        ) = self
                            .make_packet_metadata(
                                provable_height,
                                event.packet.destination_channel,
                                e.try_get()?,
                            )
                            .await?;

                        let event = WriteAck {
                            packet_data: event.packet.data.to_vec().into(),
                            acknowledgement: event.acknowledgement.to_vec().into(),
                            packet: PacketMetadata {
                                source_channel,
                                destination_channel,
                                timeout_height: event.packet.timeout_height,
                                timeout_timestamp: event.packet.timeout_timestamp,
                            },
                        }
                        .into();

                        ibc_union_spec::log_event(&event, &self.chain_id);

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
                            client_info,
                            counterparty_chain_id,
                            tx_hash,
                            provable_height,
                            ibc_spec_id: IbcUnion::ID,
                            event: into_value::<FullEvent>(event),
                        }))
                    }
                    IbcEvents::PacketRecv(event) => {
                        let (
                            counterparty_chain_id,
                            client_info,
                            destination_channel,
                            source_channel,
                        ) = self
                            .make_packet_metadata(
                                provable_height,
                                event.packet.destination_channel,
                                e.try_get()?,
                            )
                            .await?;

                        let event = PacketRecv {
                            packet_data: event.packet.data.to_vec().into(),
                            packet: PacketMetadata {
                                source_channel,
                                destination_channel,
                                timeout_height: event.packet.timeout_height,
                                timeout_timestamp: event.packet.timeout_timestamp,
                            },
                            relayer_msg: event.relayer_msg.into(),
                        }
                        .into();

                        ibc_union_spec::log_event(&event, &self.chain_id);

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
                            client_info,
                            counterparty_chain_id,
                            tx_hash,
                            provable_height,
                            ibc_spec_id: IbcUnion::ID,
                            event: into_value::<FullEvent>(event),
                        }))
                    }
                    IbcEvents::IntentPacketRecv(_event) => {
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
                    .await?;

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

                    match Ibc::IbcEvents::decode_log(&log.inner, true) {
                        Ok(event) => {
                            trace!(?event, "found IbcHandler event");

                            Some(call(PluginMessage::new(
                                self.plugin_name(),
                                ModuleCall::from(MakeFullEvent {
                                    block_number,
                                    tx_hash,
                                    event: match event.data {
                                        Ibc::IbcEvents::RegisterClient(client_registered) => {
                                            IbcEvents::RegisterClient(client_registered)
                                        }
                                        Ibc::IbcEvents::CreateClient(client_created) => {
                                            IbcEvents::CreateClient(client_created)
                                        }
                                        Ibc::IbcEvents::UpdateClient(client_updated) => {
                                            IbcEvents::UpdateClient(client_updated)
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
                                        Ibc::IbcEvents::PacketSend(packet_send) => {
                                            IbcEvents::PacketSend(packet_send)
                                        }
                                        Ibc::IbcEvents::PacketRecv(packet_recv) => {
                                            IbcEvents::PacketRecv(packet_recv)
                                        }
                                        Ibc::IbcEvents::IntentPacketRecv(intent_packet_recv) => {
                                            IbcEvents::IntentPacketRecv(intent_packet_recv)
                                        }
                                        Ibc::IbcEvents::WriteAck(write_acknowledgement) => {
                                            IbcEvents::WriteAck(write_acknowledgement)
                                        }
                                        Ibc::IbcEvents::PacketAck(acknowledge_packet) => {
                                            IbcEvents::PacketAck(acknowledge_packet)
                                        }
                                        Ibc::IbcEvents::PacketTimeout(timeout_packet) => {
                                            IbcEvents::PacketTimeout(timeout_packet)
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

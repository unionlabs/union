// #![warn(clippy::unwrap_used)] // allow for now

use std::{cmp::Ordering, collections::VecDeque};

use alloy::{
    providers::{layers::CacheLayer, DynProvider, Provider, ProviderBuilder},
    rpc::types::Filter,
    sol_types::SolEventInterface,
};
use ibc_solidity::Ibc;
use ibc_union_spec::{
    event::{
        ChannelMetadata, ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit, ChannelOpenTry,
        ConnectionMetadata, ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit,
        ConnectionOpenTry, CreateClient, FullEvent, PacketAck, PacketMetadata, PacketRecv,
        PacketSend, PacketTimeout, UpdateClient, WriteAck,
    },
    path::{ChannelPath, ConnectionPath},
    query::PacketByHash,
    ChannelId, IbcUnion, Packet, Timestamp,
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, info_span, instrument, trace, warn};
use unionlabs::{
    ibc::core::client::height::Height,
    never::Never,
    primitives::{H160, H256},
    ErrorReporter,
};
use voyager_message::{
    call::{Call, WaitForHeight},
    data::{ChainEvent, Data},
    filter::simple_take_filter,
    into_value,
    module::{PluginInfo, PluginServer},
    primitives::{ChainId, ClientInfo, IbcSpec},
    DefaultCmd, ExtensionsExt, Plugin, PluginMessage, VoyagerClient, VoyagerMessage,
};
use voyager_vm::{call, conc, data, noop, pass::PassResult, seq, BoxDynError, Op};

use crate::call::{FetchBlocks, FetchGetLogs, IbcEvents, MakeFullEvent, ModuleCall};

pub mod call;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub ibc_handler_address: H160,

    pub chunk_block_fetch_size: u64,

    pub provider: DynProvider,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The expected chain id of this ethereum-like chain.
    pub chain_id: ChainId,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    #[serde(default = "default_chunk_block_fetch_size")]
    pub chunk_block_fetch_size: u64,

    /// The RPC endpoint for the execution chain.
    pub rpc_url: String,

    #[serde(default)]
    pub max_cache_size: u32,
}

fn default_chunk_block_fetch_size() -> u64 {
    10
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = Never;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        Module::new(config).await
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.chain_id),
            interest_filter: simple_take_filter(format!(
                r#"[.. | ."@type"? == "fetch_blocks" and ."@value".chain_id == "{}"] | any"#,
                config.chain_id
            )),
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
        let provider = DynProvider::new(
            ProviderBuilder::new()
                .layer(CacheLayer::new(config.max_cache_size))
                .connect(&config.rpc_url)
                .await?,
        );

        // TODO: Assert chain id is correct
        let chain_id = provider.get_chain_id().await?;

        Ok(Self {
            chain_id: ChainId::new(chain_id.to_string()),
            ibc_handler_address: config.ibc_handler_address,
            chunk_block_fetch_size: config.chunk_block_fetch_size,
            provider,
        })
    }

    async fn make_packet_metadata(
        &self,
        event_height: Height,
        self_channel_id: ChannelId,
        voyager_client: &VoyagerClient,
    ) -> RpcResult<(ChainId, ClientInfo, ChannelMetadata, ChannelMetadata)> {
        let self_channel = voyager_client
            .query_ibc_state(
                self.chain_id.clone(),
                event_height,
                ChannelPath {
                    channel_id: self_channel_id,
                },
            )
            .await?;

        let self_connection_id = self_channel.connection_id;
        let self_connection = voyager_client
            .query_ibc_state(
                self.chain_id.clone(),
                event_height,
                ConnectionPath {
                    connection_id: self_connection_id,
                },
            )
            .await?;

        let client_info = voyager_client
            .client_info::<IbcUnion>(self.chain_id.clone(), self_connection.client_id)
            .await?;

        let client_state_meta = voyager_client
            .client_state_meta::<IbcUnion>(
                self.chain_id.clone(),
                event_height.into(),
                self_connection.client_id,
            )
            .await?;

        let counterparty_latest_height = voyager_client
            .query_latest_height(client_state_meta.counterparty_chain_id.clone(), false)
            .await?;

        let other_channel_id = self_channel.counterparty_channel_id.unwrap();

        let other_channel = voyager_client
            .query_ibc_state(
                client_state_meta.counterparty_chain_id.clone(),
                counterparty_latest_height,
                ChannelPath {
                    channel_id: other_channel_id,
                },
            )
            .await?;

        let self_channel = ChannelMetadata {
            channel_id: self_channel_id,
            version: self_channel.version,
            connection: ConnectionMetadata {
                client_id: self_connection.client_id,
                connection_id: self_connection_id,
            },
        };
        let other_channel = ChannelMetadata {
            channel_id: other_channel_id,
            version: other_channel.version,
            connection: ConnectionMetadata {
                client_id: self_connection.counterparty_client_id,
                connection_id: self_connection.counterparty_connection_id.unwrap(),
            },
        };

        Ok((
            client_state_meta.counterparty_chain_id,
            client_info,
            self_channel,
            other_channel,
        ))
    }
}

#[async_trait]
impl PluginServer<ModuleCall, Never> for Module {
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
                            ModuleCall::from(FetchBlocks {
                                block_number: fetch.start_height.height(),
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
        cb: Never,
        _data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match cb {}
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn call(&self, e: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::FetchBlocks(FetchBlocks { block_number }) => {
                self.fetch_blocks(e.try_get::<VoyagerClient>()?, block_number)
                    .await
            }
            ModuleCall::FetchGetLogs(FetchGetLogs { block_number }) => {
                self.fetch_get_logs(block_number).await
            }
            ModuleCall::MakeFullEvent(MakeFullEvent {
                block_number,
                tx_hash,
                event,
            }) => {
                self.make_full_event(e.try_get::<VoyagerClient>()?, block_number, tx_hash, event)
                    .await
            }
        }
    }
}

impl Module {
    #[instrument(skip_all, fields(%block_number))]
    async fn fetch_blocks(
        &self,
        voyager_client: &VoyagerClient,
        block_number: u64,
    ) -> RpcResult<Op<VoyagerMessage>> {
        let latest_height = voyager_client
            .query_latest_height(self.chain_id.clone(), true)
            .await?
            .height();

        info!(%latest_height, %block_number, "fetching blocks");

        let continuation = |next_height: u64| {
            seq([
                // TODO: Make this a config param
                call(WaitForHeight {
                    chain_id: self.chain_id.clone(),
                    height: Height::new(next_height),
                    finalized: true,
                }),
                call(PluginMessage::new(
                    self.plugin_name(),
                    ModuleCall::from(FetchBlocks {
                        block_number: next_height,
                    }),
                )),
            ])
        };

        match block_number.cmp(&latest_height) {
            // height < latest_height
            // fetch transactions on all blocks height..next_height (*exclusive* on the upper bound!)
            // and then queue the continuation starting at next_height
            Ordering::Equal | Ordering::Less => {
                let next_height = (latest_height - block_number)
                    .clamp(1, self.chunk_block_fetch_size)
                    + block_number;

                info!(
                    from_height = block_number,
                    to_height = next_height,
                    "batch fetching blocks in range {block_number}..{next_height}"
                );

                Ok(conc(
                    (block_number..next_height)
                        .map(|block_number| {
                            call(PluginMessage::new(
                                self.plugin_name(),
                                ModuleCall::from(FetchGetLogs { block_number }),
                            ))
                        })
                        .chain([continuation(next_height)]),
                ))
            }
            // height > latest_height
            Ordering::Greater => {
                warn!(
                    "the latest finalized height ({latest_height}) \
                    is less than the requested height ({block_number})"
                );

                Ok(continuation(block_number))
            }
        }
    }

    #[instrument(skip_all, fields(%block_number))]
    async fn fetch_get_logs(&self, block_number: u64) -> RpcResult<Op<VoyagerMessage>> {
        debug!("fetching logs in execution block");

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

        info!(logs_count = logs.len(), "found logs");

        let events = logs.into_iter().flat_map(|log| {
            let tx_hash = log
                .transaction_hash
                .expect("log should have transaction_hash")
                .into();

            info_span!("tx_hash", %tx_hash).in_scope(|| {
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
                                    Ibc::IbcEvents::ConnectionOpenInit(connection_open_init) => {
                                        IbcEvents::ConnectionOpenInit(connection_open_init)
                                    }
                                    Ibc::IbcEvents::ConnectionOpenTry(connection_open_try) => {
                                        IbcEvents::ConnectionOpenTry(connection_open_try)
                                    }
                                    Ibc::IbcEvents::ConnectionOpenAck(connection_open_ack) => {
                                        IbcEvents::ConnectionOpenAck(connection_open_ack)
                                    }
                                    Ibc::IbcEvents::ConnectionOpenConfirm(
                                        connection_open_confirm,
                                    ) => IbcEvents::ConnectionOpenConfirm(connection_open_confirm),
                                    Ibc::IbcEvents::ChannelOpenInit(channel_open_init) => {
                                        IbcEvents::ChannelOpenInit(channel_open_init)
                                    }
                                    Ibc::IbcEvents::ChannelOpenTry(channel_open_try) => {
                                        IbcEvents::ChannelOpenTry(channel_open_try)
                                    }
                                    Ibc::IbcEvents::ChannelOpenAck(channel_open_ack) => {
                                        IbcEvents::ChannelOpenAck(channel_open_ack)
                                    }
                                    Ibc::IbcEvents::ChannelOpenConfirm(channel_open_confirm) => {
                                        IbcEvents::ChannelOpenConfirm(channel_open_confirm)
                                    }
                                    Ibc::IbcEvents::ChannelCloseInit(channel_close_init) => {
                                        IbcEvents::ChannelCloseInit(channel_close_init)
                                    }
                                    Ibc::IbcEvents::ChannelCloseConfirm(channel_close_confirm) => {
                                        IbcEvents::ChannelCloseConfirm(channel_close_confirm)
                                    }
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
            })
        });

        Ok(conc(events))
    }

    #[instrument(skip_all, fields(%block_number, %tx_hash))]
    async fn make_full_event(
        &self,
        voyager_client: &VoyagerClient,
        block_number: u64,
        tx_hash: H256,
        event: IbcEvents,
    ) -> RpcResult<Op<VoyagerMessage>> {
        trace!(?event, "raw event");

        let provable_height = Height::new(block_number);

        match event {
            IbcEvents::CreateClient(raw_event) => {
                let client_id = raw_event.client_id.try_into().unwrap();

                let client_info = voyager_client
                    .client_info::<IbcUnion>(self.chain_id.clone(), client_id)
                    .await?;

                let event = CreateClient {
                    client_id,
                    client_type: client_info.client_type.clone(),
                }
                .into();

                ibc_union_spec::log_event(&event, &self.chain_id);

                Ok(data(ChainEvent {
                    chain_id: self.chain_id.clone(),
                    client_info: client_info.clone(),
                    counterparty_chain_id: ChainId::new(raw_event.counterparty_chain_id),
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
                let client_id = raw_event.client_id.try_into().unwrap();

                let client_info = voyager_client
                    .client_info::<IbcUnion>(self.chain_id.clone(), client_id)
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        self.chain_id.clone(),
                        provable_height.into(),
                        client_id,
                    )
                    .await?;

                let event = UpdateClient {
                    client_type: client_info.client_type.clone(),
                    client_id,
                    height: raw_event.height,
                }
                .into();

                ibc_union_spec::log_event(&event, &self.chain_id);

                Ok(data(ChainEvent {
                    chain_id: self.chain_id.clone(),
                    client_info: client_info.clone(),
                    counterparty_chain_id: client_state_meta.counterparty_chain_id,
                    tx_hash,
                    provable_height,
                    ibc_spec_id: IbcUnion::ID,
                    event: into_value::<FullEvent>(event),
                }))
            }

            IbcEvents::ConnectionOpenInit(raw_event) => {
                let client_id = raw_event.client_id.try_into().unwrap();
                let connection_id = raw_event.connection_id.try_into().unwrap();
                let counterparty_client_id = raw_event.counterparty_client_id.try_into().unwrap();

                let client_info = voyager_client
                    .client_info::<IbcUnion>(self.chain_id.clone(), client_id)
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        self.chain_id.clone(),
                        provable_height.into(),
                        client_id,
                    )
                    .await?;

                let event = ConnectionOpenInit {
                    client_id,
                    connection_id,
                    counterparty_client_id,
                }
                .into();

                ibc_union_spec::log_event(&event, &self.chain_id);

                Ok(data(ChainEvent {
                    chain_id: self.chain_id.clone(),
                    client_info,
                    counterparty_chain_id: client_state_meta.counterparty_chain_id,
                    tx_hash,
                    ibc_spec_id: IbcUnion::ID,
                    provable_height,
                    event: into_value::<FullEvent>(event),
                }))
            }
            IbcEvents::ConnectionOpenTry(raw_event) => {
                let client_id = raw_event.client_id.try_into().unwrap();
                let connection_id = raw_event.connection_id.try_into().unwrap();
                let counterparty_client_id = raw_event.counterparty_client_id.try_into().unwrap();
                let counterparty_connection_id =
                    raw_event.counterparty_connection_id.try_into().unwrap();

                let client_info = voyager_client
                    .client_info::<IbcUnion>(self.chain_id.clone(), client_id)
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        self.chain_id.clone(),
                        provable_height.into(),
                        client_id,
                    )
                    .await?;

                let event = ConnectionOpenTry {
                    client_id,
                    connection_id,
                    counterparty_client_id,
                    counterparty_connection_id,
                }
                .into();

                ibc_union_spec::log_event(&event, &self.chain_id);

                Ok(data(ChainEvent {
                    chain_id: self.chain_id.clone(),
                    client_info,
                    counterparty_chain_id: client_state_meta.counterparty_chain_id,
                    tx_hash,
                    provable_height,
                    ibc_spec_id: IbcUnion::ID,
                    event: into_value::<FullEvent>(event),
                }))
            }
            IbcEvents::ConnectionOpenAck(raw_event) => {
                let client_id = raw_event.client_id.try_into().unwrap();
                let connection_id = raw_event.connection_id.try_into().unwrap();
                let counterparty_client_id = raw_event.counterparty_client_id.try_into().unwrap();
                let counterparty_connection_id =
                    raw_event.counterparty_connection_id.try_into().unwrap();

                let client_info = voyager_client
                    .client_info::<IbcUnion>(self.chain_id.clone(), client_id)
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        self.chain_id.clone(),
                        provable_height.into(),
                        client_id,
                    )
                    .await?;

                let event = ConnectionOpenAck {
                    client_id,
                    connection_id,
                    counterparty_client_id,
                    counterparty_connection_id,
                }
                .into();

                ibc_union_spec::log_event(&event, &self.chain_id);

                Ok(data(ChainEvent {
                    chain_id: self.chain_id.clone(),
                    client_info,
                    counterparty_chain_id: client_state_meta.counterparty_chain_id,
                    tx_hash,
                    provable_height,
                    ibc_spec_id: IbcUnion::ID,
                    event: into_value::<FullEvent>(event),
                }))
            }
            IbcEvents::ConnectionOpenConfirm(raw_event) => {
                let client_id = raw_event.client_id.try_into().unwrap();
                let connection_id = raw_event.connection_id.try_into().unwrap();
                let counterparty_client_id = raw_event.counterparty_client_id.try_into().unwrap();
                let counterparty_connection_id =
                    raw_event.counterparty_connection_id.try_into().unwrap();

                let client_info = voyager_client
                    .client_info::<IbcUnion>(self.chain_id.clone(), client_id)
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        self.chain_id.clone(),
                        provable_height.into(),
                        client_id,
                    )
                    .await?;

                let event = ConnectionOpenConfirm {
                    client_id,
                    connection_id,
                    counterparty_client_id,
                    counterparty_connection_id,
                }
                .into();

                ibc_union_spec::log_event(&event, &self.chain_id);

                Ok(data(ChainEvent {
                    chain_id: self.chain_id.clone(),
                    client_info,
                    counterparty_chain_id: client_state_meta.counterparty_chain_id,
                    tx_hash,
                    provable_height,
                    ibc_spec_id: IbcUnion::ID,
                    event: into_value::<FullEvent>(event),
                }))
            }
            IbcEvents::ChannelOpenInit(raw_event) => {
                let channel_id = raw_event.channel_id.try_into().unwrap();
                let connection_id = raw_event.connection_id.try_into().unwrap();

                let connection = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        provable_height,
                        ConnectionPath { connection_id },
                    )
                    .await?;

                let client_info = voyager_client
                    .client_info::<IbcUnion>(self.chain_id.clone(), connection.client_id)
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        self.chain_id.clone(),
                        provable_height.into(),
                        connection.client_id,
                    )
                    .await?;

                let channel = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        provable_height,
                        ChannelPath { channel_id },
                    )
                    .await?;

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
                    counterparty_chain_id: client_state_meta.counterparty_chain_id,
                    tx_hash,
                    provable_height,
                    ibc_spec_id: IbcUnion::ID,
                    event: into_value::<FullEvent>(event),
                }))
            }
            IbcEvents::ChannelOpenTry(raw_event) => {
                let channel_id = raw_event.channel_id.try_into().unwrap();
                let connection_id = raw_event.connection_id.try_into().unwrap();
                let counterparty_channel_id = raw_event.counterparty_channel_id.try_into().unwrap();

                let connection = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        provable_height,
                        ConnectionPath { connection_id },
                    )
                    .await?;

                let client_info = voyager_client
                    .client_info::<IbcUnion>(self.chain_id.clone(), connection.client_id)
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        self.chain_id.clone(),
                        provable_height.into(),
                        connection.client_id,
                    )
                    .await?;

                let channel = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        provable_height,
                        ChannelPath { channel_id },
                    )
                    .await?;

                let event = ChannelOpenTry {
                    port_id: raw_event.port_id.into(),
                    channel_id,
                    counterparty_port_id: raw_event.counterparty_port_id.into(),
                    counterparty_channel_id,
                    connection,
                    version: channel.version,
                }
                .into();

                ibc_union_spec::log_event(&event, &self.chain_id);

                Ok(data(ChainEvent {
                    chain_id: self.chain_id.clone(),
                    client_info,
                    counterparty_chain_id: client_state_meta.counterparty_chain_id,
                    tx_hash,
                    provable_height,
                    ibc_spec_id: IbcUnion::ID,
                    event: into_value::<FullEvent>(event),
                }))
            }
            IbcEvents::ChannelOpenAck(raw_event) => {
                let channel_id = raw_event.channel_id.try_into().unwrap();
                let connection_id = raw_event.connection_id.try_into().unwrap();
                let counterparty_channel_id = raw_event.counterparty_channel_id.try_into().unwrap();

                let connection = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        provable_height,
                        ConnectionPath { connection_id },
                    )
                    .await?;

                let client_info = voyager_client
                    .client_info::<IbcUnion>(self.chain_id.clone(), connection.client_id)
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        self.chain_id.clone(),
                        provable_height.into(),
                        connection.client_id,
                    )
                    .await?;

                let channel = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        provable_height,
                        ChannelPath { channel_id },
                    )
                    .await?;

                let event = ChannelOpenAck {
                    port_id: raw_event.port_id.into(),
                    channel_id,
                    counterparty_port_id: raw_event.counterparty_port_id.into(),
                    counterparty_channel_id,
                    connection,
                    version: channel.version,
                }
                .into();

                ibc_union_spec::log_event(&event, &self.chain_id);

                Ok(data(ChainEvent {
                    chain_id: self.chain_id.clone(),
                    client_info,
                    counterparty_chain_id: client_state_meta.counterparty_chain_id,
                    tx_hash,
                    provable_height,
                    ibc_spec_id: IbcUnion::ID,
                    event: into_value::<FullEvent>(event),
                }))
            }
            IbcEvents::ChannelOpenConfirm(raw_event) => {
                let channel_id = raw_event.channel_id.try_into().unwrap();
                let connection_id = raw_event.connection_id.try_into().unwrap();
                let counterparty_channel_id = raw_event.counterparty_channel_id.try_into().unwrap();

                let connection = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        provable_height,
                        ConnectionPath { connection_id },
                    )
                    .await?;

                let client_info = voyager_client
                    .client_info::<IbcUnion>(self.chain_id.clone(), connection.client_id)
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        self.chain_id.clone(),
                        provable_height.into(),
                        connection.client_id,
                    )
                    .await?;

                let channel = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        provable_height,
                        ChannelPath { channel_id },
                    )
                    .await?;

                let event = ChannelOpenConfirm {
                    port_id: raw_event.port_id.into(),
                    channel_id,
                    counterparty_port_id: channel.counterparty_port_id,
                    counterparty_channel_id,
                    connection,
                    version: channel.version,
                }
                .into();

                ibc_union_spec::log_event(&event, &self.chain_id);

                Ok(data(ChainEvent {
                    chain_id: self.chain_id.clone(),
                    client_info,
                    counterparty_chain_id: client_state_meta.counterparty_chain_id,
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
            IbcEvents::PacketSend(raw_event) => {
                let packet = convert_packet(raw_event.packet)?;

                let (counterparty_chain_id, client_info, source_channel, destination_channel) =
                    self.make_packet_metadata(
                        provable_height,
                        packet.source_channel_id,
                        voyager_client,
                    )
                    .await?;

                let event = PacketSend {
                    packet_data: packet.data.to_vec().into(),
                    packet: PacketMetadata {
                        source_channel,
                        destination_channel,
                        timeout_height: packet.timeout_height,
                        timeout_timestamp: packet.timeout_timestamp,
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
            IbcEvents::PacketTimeout(raw_event) => {
                let (counterparty_chain_id, client_info, source_channel, destination_channel) =
                    self.make_packet_metadata(
                        provable_height,
                        raw_event.channel_id.try_into().unwrap(),
                        voyager_client,
                    )
                    .await?;

                let packet = voyager_client
                    .query(
                        self.chain_id.clone(),
                        PacketByHash {
                            channel_id: raw_event.channel_id.try_into().unwrap(),
                            packet_hash: raw_event.packet_hash.0.into(),
                        },
                    )
                    .await?;

                let event = PacketTimeout {
                    packet: PacketMetadata {
                        source_channel,
                        destination_channel,
                        timeout_height: packet.timeout_height,
                        timeout_timestamp: packet.timeout_timestamp,
                    },
                    packet_data: packet.data,
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
            IbcEvents::PacketAck(raw_event) => {
                let (counterparty_chain_id, client_info, source_channel, destination_channel) =
                    self.make_packet_metadata(
                        provable_height,
                        raw_event.channel_id.try_into().unwrap(),
                        voyager_client,
                    )
                    .await?;

                let packet = voyager_client
                    .query(
                        self.chain_id.clone(),
                        PacketByHash {
                            channel_id: raw_event.channel_id.try_into().unwrap(),
                            packet_hash: raw_event.packet_hash.0.into(),
                        },
                    )
                    .await?;

                let event = PacketAck {
                    packet: PacketMetadata {
                        source_channel,
                        destination_channel,
                        timeout_height: packet.timeout_height,
                        timeout_timestamp: packet.timeout_timestamp,
                    },
                    packet_data: packet.data,
                    acknowledgement: raw_event.acknowledgement.into(),
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
            IbcEvents::WriteAck(raw_event) => {
                let (counterparty_chain_id, client_info, destination_channel, source_channel) =
                    self.make_packet_metadata(
                        provable_height,
                        raw_event.channel_id.try_into().unwrap(),
                        voyager_client,
                    )
                    .await?;

                let packet = voyager_client
                    .query(
                        counterparty_chain_id.clone(),
                        PacketByHash {
                            channel_id: source_channel.channel_id,
                            packet_hash: raw_event.packet_hash.0.into(),
                        },
                    )
                    .await?;

                let event = WriteAck {
                    packet_data: packet.data.to_vec().into(),
                    acknowledgement: raw_event.acknowledgement.to_vec().into(),
                    packet: PacketMetadata {
                        source_channel,
                        destination_channel,
                        timeout_height: packet.timeout_height,
                        timeout_timestamp: packet.timeout_timestamp,
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
            IbcEvents::PacketRecv(raw_event) => {
                let (counterparty_chain_id, client_info, destination_channel, source_channel) =
                    self.make_packet_metadata(
                        provable_height,
                        raw_event.channel_id.try_into().unwrap(),
                        voyager_client,
                    )
                    .await?;

                let packet = voyager_client
                    .query(
                        counterparty_chain_id.clone(),
                        PacketByHash {
                            channel_id: source_channel.channel_id,
                            packet_hash: raw_event.packet_hash.0.into(),
                        },
                    )
                    .await?;

                let event = PacketRecv {
                    packet_data: packet.data.to_vec().into(),
                    packet: PacketMetadata {
                        source_channel,
                        destination_channel,
                        timeout_height: packet.timeout_height,
                        timeout_timestamp: packet.timeout_timestamp,
                    },
                    maker_msg: raw_event.maker_msg.into(),
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
}

fn convert_packet(value: ibc_solidity::Packet) -> RpcResult<Packet> {
    Ok(Packet {
        source_channel_id: ChannelId::from_raw(value.source_channel_id)
            .ok_or_else(|| ErrorObject::owned(-1, "invalid source channel id", None::<()>))?,
        destination_channel_id: ChannelId::from_raw(value.destination_channel_id)
            .ok_or_else(|| ErrorObject::owned(-1, "invalid destination channel id", None::<()>))?,
        data: value.data.into(),
        timeout_height: value.timeout_height,
        timeout_timestamp: Timestamp::from_nanos(value.timeout_timestamp),
    })
}

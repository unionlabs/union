use std::{cmp::Ordering, collections::VecDeque};

use aptos_move_ibc::ibc::{self, ClientExt as _};
use aptos_rest_client::{
    aptos_api_types::{Address, MoveType},
    error::RestError,
    Transaction,
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::{ErrorObject, ErrorObjectOwned},
    Extensions,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument};
use unionlabs::{
    hash::H256,
    ibc::core::{
        channel::{self, channel::Channel, order::Order},
        client::height::Height,
        commitment::merkle_prefix::MerklePrefix,
        connection::{self, connection_end::ConnectionEnd},
    },
    ics24::{ChannelEndPath, ConnectionPath},
    id::{ChannelId, ClientId, PortId},
    ErrorReporter, QueryHeight,
};
use voyager_message::{
    call::{Call, WaitForHeight},
    core::{ChainId, ClientInfo, ClientType},
    data::{
        AcknowledgePacket, ChainEvent, ChannelMetadata, ChannelOpenAck, ChannelOpenConfirm,
        ChannelOpenInit, ChannelOpenTry, ConnectionMetadata, ConnectionOpenAck,
        ConnectionOpenConfirm, ConnectionOpenInit, ConnectionOpenTry, CreateClient, Data,
        FullIbcEvent, PacketMetadata, RecvPacket, SendPacket, UpdateClient, WriteAcknowledgement,
    },
    module::{PluginInfo, PluginServer},
    reconnecting_jsonrpc_ws_client,
    rpc::{
        json_rpc_error_to_error_object, missing_state, VoyagerRpcClient, VoyagerRpcClientExt as _,
    },
    run_plugin_server, DefaultCmd, ExtensionsExt, Plugin, PluginMessage, VoyagerClient,
    VoyagerMessage,
};
use voyager_vm::{call, conc, data, defer, now, pass::PassResult, seq, BoxDynError, Op};

use crate::{
    call::{FetchBlocks, FetchTransactions, MakeEvent, ModuleCall},
    callback::ModuleCallback,
};

pub mod call;
pub mod callback;
pub mod data;

pub mod events;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_plugin_server::<Module>().await
}

#[derive(clap::Subcommand)]
pub enum Cmd {
    ChainId,
    VaultAddress,
    SubmitTx,
    FetchAbi,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId<'static>,

    pub aptos_client: aptos_rest_client::Client,
    pub movement_rpc_url: String,

    pub ibc_handler_address: Address,
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        let aptos_client = aptos_rest_client::Client::new(config.rpc_url.parse()?);

        let chain_id = aptos_client.get_index().await?.inner().chain_id;

        Ok(Self {
            chain_id: ChainId::new(chain_id.to_string()),
            aptos_client,
            movement_rpc_url: config.movement_rpc_url,
            ibc_handler_address: config.ibc_handler_address,
        })
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId<'static>,
    pub rpc_url: String,
    pub movement_rpc_url: String,
    pub ibc_handler_address: Address,
}

impl aptos_move_ibc::ibc::ClientExt for Module {
    fn client(&self) -> &aptos_rest_client::Client {
        &self.aptos_client
    }
}

fn plugin_name(chain_id: &ChainId<'_>) -> String {
    pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

    format!("{PLUGIN_NAME}/{}", chain_id)
}

impl Module {
    fn plugin_name(&self) -> String {
        plugin_name(&self.chain_id)
    }

    #[must_use]
    pub fn make_height(&self, height: u64) -> Height {
        Height {
            // TODO: Make this a constant
            revision_number: 0,
            revision_height: height,
        }
    }

    pub async fn ledger_version_of_height(&self, height: u64) -> u64 {
        let ledger_version = self
            .aptos_client
            .get_block_by_height(height, false)
            .await
            // .map_err(rest_error_to_rpc_error)?
            .unwrap()
            .into_inner()
            .last_version
            .0;

        debug!("height {height} is ledger version {ledger_version}");

        ledger_version
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
                            ModuleCall::FetchBlocks(FetchBlocks {
                                height: fetch.start_height.revision_height,
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
            ModuleCall::FetchTransactions(FetchTransactions { height }) => {
                info!("fetching block height {height}");
                let events = self
                    .aptos_client
                    .get_block_by_height(height, true)
                    .await
                    .map_err(|e| {
                        ErrorObject::owned(
                            -1,
                            format!("error fetching height: {}", ErrorReporter(e)),
                            None::<()>,
                        )
                    })?
                    .into_inner()
                    .transactions
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|tx| match tx {
                        Transaction::UserTransaction(tx) => Some(tx),
                        _ => None,
                    })
                    .flat_map(|tx| {
                        tx.events
                            .into_iter()
                            .map(move |events| (events, tx.info.hash))
                    })
                    .filter_map(|(e, hash)| match e.typ {
                        MoveType::Struct(s) => {
                            (s.address == self.ibc_handler_address).then_some((s, e.data, hash))
                        }
                        _ => None,
                    })
                    .map(|(typ, data, hash)| {
                        let event = match dbg!(typ).name.0.as_str() {
                            "ClientCreatedEvent" => {
                                serde_json::from_value::<ibc::ClientCreatedEvent>(data)
                                    .unwrap()
                                    .into()
                            }
                            "ClientUpdated" => serde_json::from_value::<ibc::ClientUpdated>(data)
                                .unwrap()
                                .into(),
                            "ConnectionOpenInit" => {
                                serde_json::from_value::<ibc::ConnectionOpenInit>(data)
                                    .unwrap()
                                    .into()
                            }
                            "ConnectionOpenTry" => {
                                serde_json::from_value::<ibc::ConnectionOpenTry>(data)
                                    .unwrap()
                                    .into()
                            }
                            "ConnectionOpenAck" => {
                                serde_json::from_value::<ibc::ConnectionOpenAck>(data)
                                    .unwrap()
                                    .into()
                            }
                            "ConnectionOpenConfirm" => {
                                serde_json::from_value::<ibc::ConnectionOpenConfirm>(data)
                                    .unwrap()
                                    .into()
                            }
                            "ChannelOpenInit" => {
                                serde_json::from_value::<ibc::ChannelOpenInit>(data)
                                    .unwrap()
                                    .into()
                            }
                            "ChannelOpenTry" => serde_json::from_value::<ibc::ChannelOpenTry>(data)
                                .unwrap()
                                .into(),
                            "ChannelOpenAck" => serde_json::from_value::<ibc::ChannelOpenAck>(data)
                                .unwrap()
                                .into(),
                            "ChannelOpenConfirm" => {
                                serde_json::from_value::<ibc::ChannelOpenConfirm>(data)
                                    .unwrap()
                                    .into()
                            }
                            "WriteAcknowledgement" => {
                                serde_json::from_value::<ibc::WriteAcknowledgement>(data)
                                    .unwrap()
                                    .into()
                            }
                            "RecvPacket" => serde_json::from_value::<ibc::RecvPacket>(data)
                                .unwrap()
                                .into(),
                            "SendPacket" => serde_json::from_value::<ibc::SendPacket>(data)
                                .unwrap()
                                .into(),
                            "AcknowledgePacket" => {
                                serde_json::from_value::<ibc::AcknowledgePacket>(data)
                                    .unwrap()
                                    .into()
                            }
                            "TimeoutPacket" => serde_json::from_value::<ibc::TimeoutPacket>(data)
                                .unwrap()
                                .into(),
                            unknown => panic!("unknown event `{unknown}`"),
                        };
                        // TODO: Check the type before deserializing
                        call(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(MakeEvent {
                                event,
                                tx_hash: H256::new(*hash.0),
                                height,
                            }),
                        ))
                    });

                Ok(conc(events))
            }
            ModuleCall::FetchBlocks(FetchBlocks { height }) => Ok(conc([
                call(PluginMessage::new(
                    self.plugin_name(),
                    ModuleCall::from(FetchTransactions { height }),
                )),
                {
                    let latest_height = self
                        .aptos_client
                        .get_index()
                        .await
                        .unwrap()
                        .into_inner()
                        .block_height
                        .0;
                    match height.cmp(&latest_height) {
                        Ordering::Less => {
                            let next_height = (latest_height - height).clamp(1, 10) + height;
                            conc(
                                ((height + 1)..next_height)
                                    .into_iter()
                                    .map(|height| {
                                        call(PluginMessage::new(
                                            self.plugin_name(),
                                            ModuleCall::from(FetchTransactions { height }),
                                        ))
                                    })
                                    .chain([call(PluginMessage::new(
                                        self.plugin_name(),
                                        ModuleCall::from(FetchBlocks {
                                            height: next_height,
                                        }),
                                    ))]),
                            )
                        }
                        Ordering::Equal | Ordering::Greater => seq([
                            defer(now() + 1),
                            call(PluginMessage::new(
                                self.plugin_name(),
                                ModuleCall::from(FetchBlocks { height: height + 1 }),
                            )),
                        ]),
                    }
                },
            ])),
            ModuleCall::MakeEvent(MakeEvent {
                event,
                tx_hash,
                height,
            }) => {
                fn ibc_height(h: aptos_move_ibc::height::Height) -> Height {
                    Height {
                        revision_number: h.revision_number.0,
                        revision_height: h.revision_height.0,
                    }
                }

                let (full_event, client_id): (FullIbcEvent, ClientId) = match event {
                    events::IbcEvent::CreateClient(event) => (
                        CreateClient {
                            client_id: event.client_id.parse().unwrap(),
                            client_type: ClientType::new(event.client_type),
                            consensus_height: ibc_height(event.consensus_height),
                        }
                        .into(),
                        event.client_id.parse().unwrap(),
                    ),
                    events::IbcEvent::UpdateClient(event) => (
                        UpdateClient {
                            client_id: event.client_id.parse().unwrap(),
                            client_type: ClientType::new(event.client_type),
                            consensus_heights: vec![ibc_height(event.height)],
                        }
                        .into(),
                        event.client_id.parse().unwrap(),
                    ),
                    events::IbcEvent::ConnectionOpenInit(event) => (
                        ConnectionOpenInit {
                            client_id: event.client_id.parse().unwrap(),
                            connection_id: event.connection_id.parse().unwrap(),
                            counterparty_client_id: event.counterparty_client_id.parse().unwrap(),
                        }
                        .into(),
                        event.client_id.parse().unwrap(),
                    ),
                    events::IbcEvent::ConnectionOpenTry(event) => (
                        ConnectionOpenTry {
                            client_id: event.client_id.parse().unwrap(),
                            connection_id: event.connection_id.parse().unwrap(),
                            counterparty_client_id: event.counterparty_client_id.parse().unwrap(),
                            counterparty_connection_id: event
                                .counterparty_connection_id
                                .parse()
                                .unwrap(),
                        }
                        .into(),
                        event.client_id.parse().unwrap(),
                    ),
                    events::IbcEvent::ConnectionOpenAck(event) => (
                        ConnectionOpenAck {
                            client_id: event.client_id.parse().unwrap(),
                            connection_id: event.connection_id.parse().unwrap(),
                            counterparty_client_id: event.counterparty_client_id.parse().unwrap(),
                            counterparty_connection_id: event
                                .counterparty_connection_id
                                .parse()
                                .unwrap(),
                        }
                        .into(),
                        event.client_id.parse().unwrap(),
                    ),
                    events::IbcEvent::ConnectionOpenConfirm(e) => (
                        ConnectionOpenConfirm {
                            client_id: e.client_id.parse().unwrap(),
                            connection_id: e.connection_id.parse().unwrap(),
                            counterparty_client_id: e.counterparty_client_id.parse().unwrap(),
                            counterparty_connection_id: e
                                .counterparty_connection_id
                                .parse()
                                .unwrap(),
                        }
                        .into(),
                        e.client_id.parse().unwrap(),
                    ),
                    events::IbcEvent::ChannelOpenInit(event) => {
                        let ledger_version = self.ledger_version_of_height(height).await;

                        let connection = self
                            .get_connection(
                                self.ibc_handler_address.into(),
                                (event.connection_id,),
                                Some(ledger_version),
                            )
                            .await
                            .unwrap()
                            .into_option()
                            .unwrap();

                        let connection = convert_connection(connection);

                        let client_id = connection.client_id.clone();

                        (
                            ChannelOpenInit {
                                port_id: event.port_id.parse().unwrap(),
                                channel_id: event.channel_id.parse().unwrap(),
                                counterparty_port_id: event.counterparty_port_id.parse().unwrap(),
                                connection,
                                version: event.version,
                            }
                            .into(),
                            client_id,
                        )
                    }
                    events::IbcEvent::ChannelOpenTry(event) => {
                        let ledger_version = self.ledger_version_of_height(height).await;

                        let connection = self
                            .get_connection(
                                self.ibc_handler_address.into(),
                                (event.connection_id,),
                                Some(ledger_version),
                            )
                            .await
                            .unwrap()
                            .into_option()
                            .unwrap();

                        let connection = convert_connection(connection);

                        let client_id = connection.client_id.clone();

                        (
                            ChannelOpenTry {
                                port_id: event.port_id.parse().unwrap(),
                                channel_id: event.channel_id.parse().unwrap(),
                                counterparty_port_id: event.counterparty_port_id.parse().unwrap(),
                                counterparty_channel_id: event
                                    .counterparty_port_id
                                    .parse()
                                    .unwrap(),
                                connection,
                                version: event.version,
                            }
                            .into(),
                            client_id,
                        )
                    }
                    events::IbcEvent::ChannelOpenAck(event) => {
                        let ledger_version = self.ledger_version_of_height(height).await;

                        let connection = self
                            .get_connection(
                                self.ibc_handler_address.into(),
                                (event.connection_id,),
                                Some(ledger_version),
                            )
                            .await
                            .unwrap()
                            .into_option()
                            .unwrap();

                        let channel = self
                            .get_channel(
                                self.ibc_handler_address.into(),
                                (event.port_id.clone(), event.channel_id.clone()),
                                Some(ledger_version),
                            )
                            .await
                            .unwrap()
                            .into_option()
                            .unwrap();

                        let connection = convert_connection(connection);

                        let channel = convert_channel(channel);

                        let client_id = connection.client_id.clone();

                        (
                            ChannelOpenAck {
                                port_id: event.port_id.parse().unwrap(),
                                channel_id: event.channel_id.parse().unwrap(),
                                counterparty_port_id: event.counterparty_port_id.parse().unwrap(),
                                counterparty_channel_id: event
                                    .counterparty_channel_id
                                    .parse()
                                    .unwrap(),
                                connection,
                                version: channel.version,
                            }
                            .into(),
                            client_id,
                        )
                    }
                    events::IbcEvent::ChannelOpenConfirm(event) => {
                        let ledger_version = self.ledger_version_of_height(height).await;

                        let connection = self
                            .get_connection(
                                self.ibc_handler_address.into(),
                                (event.connection_id,),
                                Some(ledger_version),
                            )
                            .await
                            .unwrap()
                            .into_option()
                            .unwrap();

                        let channel = self
                            .get_channel(
                                self.ibc_handler_address.into(),
                                (event.port_id.clone(), event.channel_id.clone()),
                                Some(ledger_version),
                            )
                            .await
                            .unwrap()
                            .into_option()
                            .unwrap();

                        let connection = convert_connection(connection);

                        let channel = convert_channel(channel);

                        let client_id = connection.client_id.clone();

                        (
                            ChannelOpenConfirm {
                                port_id: event.port_id.parse().unwrap(),
                                channel_id: event.channel_id.parse().unwrap(),
                                counterparty_port_id: event.counterparty_port_id.parse().unwrap(),
                                counterparty_channel_id: event
                                    .counterparty_channel_id
                                    .parse()
                                    .unwrap(),
                                connection,
                                version: channel.version,
                            }
                            .into(),
                            client_id,
                        )
                    }
                    events::IbcEvent::WriteAcknowledgement(event) => {
                        let (
                            _counterparty_chain_id,
                            _client_info,
                            destination_channel,
                            source_channel,
                            channel_ordering,
                        ) = self
                            .make_packet_metadata(
                                self.make_height(height),
                                event.packet.destination_port.parse().unwrap(),
                                event.packet.destination_channel.parse().unwrap(),
                                e.try_get()?,
                            )
                            .await?;

                        let client_id = destination_channel.connection.client_id.clone();

                        (
                            WriteAcknowledgement {
                                packet_data: event.packet.data.into(),
                                packet_ack: event.acknowledgement.into(),
                                packet: PacketMetadata {
                                    sequence: (*event.packet.sequence.inner()).try_into().unwrap(),
                                    source_channel,
                                    destination_channel,
                                    channel_ordering,
                                    timeout_height: ibc_height(event.packet.timeout_height),
                                    timeout_timestamp: *event.packet.timeout_timestamp.inner(),
                                },
                            }
                            .into(),
                            client_id,
                        )
                    }
                    events::IbcEvent::RecvPacket(event) => {
                        let (
                            _counterparty_chain_id,
                            _client_info,
                            destination_channel,
                            source_channel,
                            channel_ordering,
                        ) = self
                            .make_packet_metadata(
                                self.make_height(height),
                                event.packet.destination_port.parse().unwrap(),
                                event.packet.destination_channel.parse().unwrap(),
                                e.try_get()?,
                            )
                            .await?;

                        let client_id = destination_channel.connection.client_id.clone();

                        (
                            RecvPacket {
                                packet_data: event.packet.data.into(),
                                packet: PacketMetadata {
                                    sequence: (*event.packet.sequence.inner()).try_into().unwrap(),
                                    source_channel,
                                    destination_channel,
                                    channel_ordering,
                                    timeout_height: ibc_height(event.packet.timeout_height),
                                    timeout_timestamp: *event.packet.timeout_timestamp.inner(),
                                },
                            }
                            .into(),
                            client_id,
                        )
                    }
                    events::IbcEvent::SendPacket(event) => {
                        let (
                            _counterparty_chain_id,
                            _client_info,
                            source_channel,
                            destination_channel,
                            channel_ordering,
                        ) = self
                            .make_packet_metadata(
                                self.make_height(height),
                                event.source_port.parse().unwrap(),
                                event.source_channel.parse().unwrap(),
                                e.try_get()?,
                            )
                            .await?;

                        let client_id = source_channel.connection.client_id.clone();

                        (
                            SendPacket {
                                packet_data: event.data.into(),
                                packet: PacketMetadata {
                                    sequence: (*event.sequence.inner()).try_into().unwrap(),
                                    source_channel,
                                    destination_channel,
                                    channel_ordering,
                                    timeout_height: ibc_height(event.timeout_height),
                                    timeout_timestamp: *event.timeout_timestamp.inner(),
                                },
                            }
                            .into(),
                            client_id,
                        )
                    }
                    events::IbcEvent::AcknowledgePacket(event) => {
                        let (
                            _counterparty_chain_id,
                            _client_info,
                            source_channel,
                            destination_channel,
                            channel_ordering,
                        ) = self
                            .make_packet_metadata(
                                self.make_height(height),
                                event.packet.source_port.parse().unwrap(),
                                event.packet.source_channel.parse().unwrap(),
                                e.try_get()?,
                            )
                            .await?;

                        let client_id = source_channel.connection.client_id.clone();

                        (
                            AcknowledgePacket {
                                packet: PacketMetadata {
                                    sequence: (*event.packet.sequence.inner()).try_into().unwrap(),
                                    source_channel,
                                    destination_channel,
                                    channel_ordering,
                                    timeout_height: ibc_height(event.packet.timeout_height),
                                    timeout_timestamp: *event.packet.timeout_timestamp.inner(),
                                },
                            }
                            .into(),
                            client_id,
                        )
                    }
                    events::IbcEvent::TimeoutPacket(_) => todo!(),
                };

                let voyager_client = e.try_get::<VoyagerClient>()?;

                let client_info = voyager_client
                    .client_info(self.chain_id.clone(), client_id.clone())
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                let client_meta = voyager_client
                    .client_meta(
                        self.chain_id.clone(),
                        self.make_height(height).into(),
                        client_id.clone(),
                    )
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                Ok(data(ChainEvent {
                    chain_id: self.chain_id.clone(),
                    client_info,
                    counterparty_chain_id: client_meta.chain_id,
                    tx_hash,
                    // TODO: Review this, does it need to be +1?
                    provable_height: self.make_height(height),
                    event: full_event,
                }))
            }
        }
    }
}

pub fn rest_error_to_rpc_error(e: RestError) -> ErrorObjectOwned {
    ErrorObject::owned(-1, format!("rest error: {}", ErrorReporter(e)), None::<()>)
}

pub fn convert_connection(
    connection: aptos_move_ibc::connection_end::ConnectionEnd,
) -> ConnectionEnd {
    ConnectionEnd {
        client_id: connection.client_id.parse().unwrap(),
        versions: connection
            .versions
            .into_iter()
            .map(|version| connection::version::Version {
                identifier: version.identifier,
                features: version
                    .features
                    .into_iter()
                    .map(|feature| Order::from_proto_str(&feature).expect("unknown feature"))
                    .collect(),
            })
            .collect(),
        state: connection::state::State::try_from(u8::try_from(connection.state.0).unwrap())
            .unwrap(),
        counterparty: connection::counterparty::Counterparty {
            client_id: connection.counterparty.client_id.parse().unwrap(),
            connection_id: if connection.counterparty.connection_id.is_empty() {
                None
            } else {
                Some(connection.counterparty.connection_id.parse().unwrap())
            },
            prefix: MerklePrefix {
                key_prefix: connection.counterparty.prefix.key_prefix.into(),
            },
        },
        delay_period: connection.delay_period.0,
    }
}

pub fn convert_channel(channel: aptos_move_ibc::channel::Channel) -> Channel {
    Channel {
        state: channel.state.try_into().unwrap(),
        ordering: channel.ordering.try_into().unwrap(),
        counterparty: channel::counterparty::Counterparty {
            port_id: channel.counterparty.port_id.parse().unwrap(),
            channel_id: channel.counterparty.channel_id.parse().unwrap(),
        },
        connection_hops: channel
            .connection_hops
            .into_iter()
            .map(|hop| hop.parse().unwrap())
            .collect(),
        version: channel.version,
    }
}

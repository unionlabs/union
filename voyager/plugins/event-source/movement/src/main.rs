use std::{cmp::Ordering, collections::VecDeque};

use aptos_move_ibc::{
    connection_end::ConnectionEnd,
    ibc::{self, ClientExt as _},
};
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
    ibc::core::client::height::Height,
    ics24::ethabi::{ChannelPath, ConnectionPath},
    ErrorReporter,
};
use voyager_message::{
    call::Call,
    core::{ChainId, ClientInfo, ClientType, QueryHeight},
    data::{ChainEvent, Data},
    ibc_union::{
        AcknowledgePacket, ChannelMetadata, ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit,
        ChannelOpenTry, ClientCreated, ClientUpdated, ConnectionMetadata, ConnectionOpenAck,
        ConnectionOpenConfirm, ConnectionOpenInit, ConnectionOpenTry, FullIbcEvent, IbcUnion,
        PacketMetadata, RecvPacket, SendPacket, WriteAcknowledgement,
    },
    into_value,
    module::{PluginInfo, PluginServer},
    rpc::missing_state,
    DefaultCmd, ExtensionsExt, IbcSpec, Plugin, PluginMessage, VoyagerClient, VoyagerMessage,
};
use voyager_vm::{call, conc, data, defer, now, pass::PassResult, seq, BoxDynError, Op};

use crate::{
    call::{FetchBlocks, FetchTransactions, MakeFullEvent, ModuleCall},
    callback::ModuleCallback,
};

pub mod call;
pub mod callback;
pub mod data;

pub mod events;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
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
    pub chain_id: ChainId,

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
    pub chain_id: ChainId,
    pub rpc_url: String,
    pub movement_rpc_url: String,
    pub ibc_handler_address: Address,
}

impl aptos_move_ibc::ibc::ClientExt for Module {
    fn client(&self) -> &aptos_rest_client::Client {
        &self.aptos_client
    }
}

fn plugin_name(chain_id: &ChainId) -> String {
    pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

    format!("{PLUGIN_NAME}/{}", chain_id)
}

impl Module {
    fn plugin_name(&self) -> String {
        plugin_name(&self.chain_id)
    }

    #[must_use]
    pub fn make_height(&self, height: u64) -> Height {
        Height::new(height)
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
            .client_info::<IbcUnion>(
                self.chain_id.clone(),
                self_connection_state.client_id.clone(),
            )
            .await?;

        let client_meta = voyager_rpc_client
            .client_meta::<IbcUnion>(
                self.chain_id.clone(),
                event_height.into(),
                self_connection_state.client_id.clone(),
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
            channel_id: self_channel_id.clone(),
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
                            ModuleCall::FetchBlocks(FetchBlocks {
                                height: fetch.start_height.height(),
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
                            ModuleCall::from(MakeFullEvent {
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
            ModuleCall::MakeFullEvent(MakeFullEvent {
                event,
                tx_hash,
                height,
            }) => {
                let (full_event, client_id): (FullIbcEvent, u32) = match event {
                    events::IbcEvent::CreateClient(event) => (
                        ClientCreated {
                            client_id: event.client_id,
                            client_type: ClientType::new(event.client_type),
                        }
                        .into(),
                        event.client_id,
                    ),
                    events::IbcEvent::UpdateClient(event) => (
                        ClientUpdated {
                            client_id: event.client_id,
                            client_type: ClientType::new(event.client_type),
                            height: event.height,
                        }
                        .into(),
                        event.client_id,
                    ),
                    events::IbcEvent::ConnectionOpenInit(event) => (
                        ConnectionOpenInit {
                            client_id: event.client_id,
                            connection_id: event.connection_id,
                            counterparty_client_id: event.counterparty_client_id,
                        }
                        .into(),
                        event.client_id,
                    ),
                    events::IbcEvent::ConnectionOpenTry(event) => (
                        ConnectionOpenTry {
                            client_id: event.client_id,
                            connection_id: event.connection_id,
                            counterparty_client_id: event.counterparty_client_id,
                            counterparty_connection_id: event.counterparty_connection_id,
                        }
                        .into(),
                        event.client_id,
                    ),
                    events::IbcEvent::ConnectionOpenAck(event) => (
                        ConnectionOpenAck {
                            client_id: event.client_id,
                            connection_id: event.connection_id,
                            counterparty_client_id: event.counterparty_client_id,
                            counterparty_connection_id: event.counterparty_connection_id,
                        }
                        .into(),
                        event.client_id,
                    ),
                    events::IbcEvent::ConnectionOpenConfirm(event) => (
                        ConnectionOpenConfirm {
                            client_id: event.client_id,
                            connection_id: event.connection_id,
                            counterparty_client_id: event.counterparty_client_id,
                            counterparty_connection_id: event.counterparty_connection_id,
                        }
                        .into(),
                        event.client_id,
                    ),
                    events::IbcEvent::ChannelOpenInit(event) => {
                        let ledger_version = self.ledger_version_of_height(height).await;

                        let connection = self
                            .get_connection(
                                self.ibc_handler_address.into(),
                                Some(ledger_version),
                                (event.connection_id,),
                            )
                            .await
                            .unwrap()
                            .unwrap();

                        let connection = convert_connection(connection);
                        let client_id = connection.client_id;

                        (
                            ChannelOpenInit {
                                port_id: event.port_id.parse().unwrap(),
                                channel_id: event.channel_id,
                                counterparty_port_id: event.counterparty_port_id.into(),
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
                                Some(ledger_version),
                                (event.connection_id,),
                            )
                            .await
                            .unwrap()
                            .unwrap();

                        let connection = convert_connection(connection);

                        let client_id = connection.client_id;

                        (
                            ChannelOpenTry {
                                port_id: event.port_id.parse().unwrap(),
                                channel_id: event.channel_id,
                                counterparty_port_id: event.counterparty_port_id.into(),
                                counterparty_channel_id: event.counterparty_channel_id,
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
                                Some(ledger_version),
                                (event.connection_id,),
                            )
                            .await
                            .unwrap()
                            .unwrap();

                        let channel = self
                            .get_channel(
                                self.ibc_handler_address.into(),
                                Some(ledger_version),
                                (event.channel_id,),
                            )
                            .await
                            .unwrap()
                            .unwrap();

                        let connection = convert_connection(connection);

                        let client_id = connection.client_id.clone();

                        (
                            ChannelOpenAck {
                                port_id: event.port_id.parse().unwrap(),
                                channel_id: event.channel_id,
                                counterparty_port_id: event.counterparty_port_id.into(),
                                counterparty_channel_id: event.counterparty_channel_id,
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
                                Some(ledger_version),
                                (event.connection_id,),
                            )
                            .await
                            .unwrap()
                            .unwrap();

                        let channel = self
                            .get_channel(
                                self.ibc_handler_address.into(),
                                Some(ledger_version),
                                (event.channel_id,),
                            )
                            .await
                            .unwrap()
                            .unwrap();

                        let connection = convert_connection(connection);

                        let client_id = connection.client_id;

                        (
                            ChannelOpenConfirm {
                                port_id: event.port_id.parse().unwrap(),
                                channel_id: event.channel_id,
                                counterparty_port_id: event.counterparty_port_id.into(),
                                counterparty_channel_id: event.counterparty_channel_id,
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
                        ) = self
                            .make_packet_metadata(
                                self.make_height(height),
                                event.packet.destination_channel,
                                e.try_get()?,
                            )
                            .await?;

                        let client_id = destination_channel.connection.client_id.clone();

                        (
                            WriteAcknowledgement {
                                packet_data: event.packet.data.into(),
                                acknowledgement: event.acknowledgement.into(),
                                packet: PacketMetadata {
                                    source_channel,
                                    destination_channel,
                                    timeout_height: event.packet.timeout_height,
                                    timeout_timestamp: event.packet.timeout_timestamp,
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
                        ) = self
                            .make_packet_metadata(
                                self.make_height(height),
                                event.packet.destination_channel,
                                e.try_get()?,
                            )
                            .await?;

                        let client_id = destination_channel.connection.client_id.clone();

                        (
                            RecvPacket {
                                packet_data: event.packet.data.into(),
                                packet: PacketMetadata {
                                    source_channel,
                                    destination_channel,
                                    timeout_height: event.packet.timeout_height,
                                    timeout_timestamp: event.packet.timeout_timestamp,
                                },
                                // TODO(aeryz): why is this H160?
                                relayer: Default::default(),
                                relayer_msg: Default::default(),
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
                        ) = self
                            .make_packet_metadata(
                                self.make_height(height),
                                event.source_channel,
                                e.try_get()?,
                            )
                            .await?;

                        let client_id = source_channel.connection.client_id.clone();

                        (
                            SendPacket {
                                packet_data: event.data.into(),
                                packet: PacketMetadata {
                                    source_channel,
                                    destination_channel,
                                    timeout_height: event.timeout_height,
                                    timeout_timestamp: event.timeout_timestamp,
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
                        ) = self
                            .make_packet_metadata(
                                self.make_height(height),
                                event.packet.source_channel,
                                e.try_get()?,
                            )
                            .await?;

                        let client_id = source_channel.connection.client_id.clone();

                        (
                            AcknowledgePacket {
                                packet_data: event.packet.data.into(),
                                packet: PacketMetadata {
                                    source_channel,
                                    destination_channel,
                                    timeout_height: event.packet.timeout_height,
                                    timeout_timestamp: event.packet.timeout_timestamp,
                                },
                                relayer: Default::default(),
                                acknowledgement: event.acknowledgement.into(),
                            }
                            .into(),
                            client_id,
                        )
                    }
                    events::IbcEvent::TimeoutPacket(_) => todo!(),
                };

                let voyager_client = e.try_get::<VoyagerClient>()?;

                let client_info = voyager_client
                    .client_info::<IbcUnion>(self.chain_id.clone(), client_id.clone())
                    .await?;

                let client_meta = voyager_client
                    .client_meta::<IbcUnion>(
                        self.chain_id.clone(),
                        self.make_height(height).into(),
                        client_id.clone(),
                    )
                    .await?;

                Ok(data(ChainEvent {
                    chain_id: self.chain_id.clone(),
                    client_info,
                    counterparty_chain_id: client_meta.chain_id,
                    tx_hash,
                    // TODO: Review this, does it need to be +1?
                    provable_height: self.make_height(height),
                    event: into_value::<FullIbcEvent>(full_event),
                    ibc_version_id: IbcUnion::ID,
                }))
            }
        }
    }
}

pub fn rest_error_to_rpc_error(e: RestError) -> ErrorObjectOwned {
    ErrorObject::owned(-1, format!("rest error: {}", ErrorReporter(e)), None::<()>)
}

fn convert_connection(connection: ConnectionEnd) -> ibc_solidity::ibc::Connection {
    ibc_solidity::ibc::Connection {
        state: match connection.state {
            0 => ibc_solidity::ibc::ConnectionState::Unspecified,
            1 => ibc_solidity::ibc::ConnectionState::Init,
            2 => ibc_solidity::ibc::ConnectionState::TryOpen,
            3 => ibc_solidity::ibc::ConnectionState::Open,
            _ => panic!("connection state cannot be more than 3"),
        },
        client_id: connection.client_id,
        counterparty_client_id: connection.counterparty_client_id,
        counterparty_connection_id: connection.counterparty_connection_id,
    }
}

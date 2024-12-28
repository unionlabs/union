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
use ibc_union_spec::{
    ChannelMetadata, ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit, ChannelOpenTry,
    ChannelPath, ConnectionMetadata, ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit,
    ConnectionOpenTry, ConnectionPath, CreateClient, FullEvent, IbcUnion, PacketAck,
    PacketMetadata, PacketRecv, PacketSend, UpdateClient, WriteAck,
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::{ErrorObject, ErrorObjectOwned},
    Extensions,
};
use move_bindgen::MoveOutputType;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, info, instrument};
use unionlabs::{hash::H256, ibc::core::client::height::Height, ErrorReporter};
use voyager_message::{
    call::{Call, WaitForHeight},
    core::{ChainId, ClientInfo, ClientType, IbcSpec, QueryHeight},
    data::{ChainEvent, Data},
    into_value,
    module::{PluginInfo, PluginServer},
    rpc::missing_state,
    DefaultCmd, ExtensionsExt, Plugin, PluginMessage, VoyagerClient, VoyagerMessage,
};
use voyager_vm::{call, conc, data, pass::PassResult, seq, BoxDynError, Op};

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

impl aptos_move_ibc::recv_packet::ClientExt for Module {
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
                            "ClientCreatedEvent" => from_raw_event::<ibc::ClientCreatedEvent>(data),
                            "ClientUpdated" => from_raw_event::<ibc::ClientUpdated>(data),
                            "ConnectionOpenInit" => from_raw_event::<ibc::ConnectionOpenInit>(data),
                            "ConnectionOpenTry" => from_raw_event::<ibc::ConnectionOpenTry>(data),
                            "ConnectionOpenAck" => from_raw_event::<ibc::ConnectionOpenAck>(data),
                            "ConnectionOpenConfirm" => {
                                from_raw_event::<ibc::ConnectionOpenConfirm>(data)
                            }
                            "ChannelOpenInit" => from_raw_event::<ibc::ChannelOpenInit>(data),
                            "ChannelOpenTry" => from_raw_event::<ibc::ChannelOpenTry>(data),
                            "ChannelOpenAck" => from_raw_event::<ibc::ChannelOpenAck>(data),
                            "ChannelOpenConfirm" => from_raw_event::<ibc::ChannelOpenConfirm>(data),
                            "WriteAcknowledgement" => {
                                from_raw_event::<ibc::WriteAcknowledgement>(data)
                            }
                            "RecvPacket" => from_raw_event::<ibc::RecvPacket>(data),
                            "SendPacket" => from_raw_event::<ibc::SendPacket>(data),
                            "AcknowledgePacket" => from_raw_event::<ibc::AcknowledgePacket>(data),
                            "TimeoutPacket" => from_raw_event::<ibc::TimeoutPacket>(data),
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
                            call(WaitForHeight {
                                chain_id: self.chain_id.clone(),
                                height: Height::new(height + 1),
                                finalized: true,
                            }),
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
                let (full_event, client_id): (FullEvent, u32) = match event {
                    events::IbcEvent::CreateClient(event) => (
                        CreateClient {
                            client_id: event.client_id,
                            client_type: ClientType::new(event.client_type),
                        }
                        .into(),
                        event.client_id,
                    ),
                    events::IbcEvent::UpdateClient(event) => (
                        UpdateClient {
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

                        let client_id = connection.client_id;

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

                        let client_id = destination_channel.connection.client_id;

                        (
                            WriteAck {
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

                        let client_id = destination_channel.connection.client_id;

                        (
                            PacketRecv {
                                packet_data: event.packet.data.into(),
                                packet: PacketMetadata {
                                    source_channel,
                                    destination_channel,
                                    timeout_height: event.packet.timeout_height,
                                    timeout_timestamp: event.packet.timeout_timestamp,
                                },
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

                        let client_id = source_channel.connection.client_id;

                        (
                            PacketSend {
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

                        let client_id = source_channel.connection.client_id;

                        (
                            PacketAck {
                                packet_data: event.packet.data.into(),
                                packet: PacketMetadata {
                                    source_channel,
                                    destination_channel,
                                    timeout_height: event.packet.timeout_height,
                                    timeout_timestamp: event.packet.timeout_timestamp,
                                },
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
                    .client_info::<IbcUnion>(self.chain_id.clone(), client_id)
                    .await?;

                let client_meta = voyager_client
                    .client_meta::<IbcUnion>(
                        self.chain_id.clone(),
                        self.make_height(height).into(),
                        client_id,
                    )
                    .await?;

                Ok(data(ChainEvent {
                    chain_id: self.chain_id.clone(),
                    client_info,
                    counterparty_chain_id: client_meta.chain_id,
                    tx_hash,
                    // TODO: Review this, does it need to be +1?
                    provable_height: self.make_height(height),
                    event: into_value::<FullEvent>(full_event),
                    ibc_spec_id: IbcUnion::ID,
                }))
            }
        }
    }
}

pub fn rest_error_to_rpc_error(e: RestError) -> ErrorObjectOwned {
    ErrorObject::owned(-1, format!("rest error: {}", ErrorReporter(e)), None::<()>)
}

fn convert_connection(connection: ConnectionEnd) -> ibc_solidity::Connection {
    ibc_solidity::Connection {
        state: match connection.state {
            0 => ibc_solidity::ConnectionState::Unspecified,
            1 => ibc_solidity::ConnectionState::Init,
            2 => ibc_solidity::ConnectionState::TryOpen,
            3 => ibc_solidity::ConnectionState::Open,
            _ => panic!("connection state cannot be more than 3"),
        },
        client_id: connection.client_id,
        counterparty_client_id: connection.counterparty_client_id,
        counterparty_connection_id: connection.counterparty_connection_id,
    }
}

fn from_raw_event<T: MoveOutputType + Into<events::IbcEvent>>(data: Value) -> events::IbcEvent {
    let raw_event = serde_json::from_value::<T::Raw>(data).unwrap();
    T::from_raw(raw_event).into()
}

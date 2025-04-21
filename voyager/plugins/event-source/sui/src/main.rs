use std::{cmp::Ordering, collections::VecDeque};

use ibc_union_spec::{
    event::{
        ChannelMetadata, ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit, ChannelOpenTry,
        ConnectionMetadata, ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit,
        ConnectionOpenTry, CreateClient, FullEvent, PacketAck, PacketMetadata, PacketRecv,
        PacketSend, UpdateClient, WriteAck,
    },
    path::{ChannelPath, ConnectionPath},
    ChannelId, ClientId, Connection, ConnectionState, IbcUnion, Timestamp,
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::{ErrorObject, ErrorObjectOwned},
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sui_sdk::{rpc_types::SuiTransactionBlockResponseOptions, SuiClientBuilder};
use tracing::{debug, info, instrument};
use unionlabs::{ibc::core::client::height::Height, primitives::H256, ErrorReporter};
use voyager_message::{
    call::{Call, WaitForHeight},
    data::{ChainEvent, Data},
    filter::simple_take_filter,
    into_value,
    module::{PluginInfo, PluginServer},
    primitives::{ChainId, ClientInfo, ClientType, IbcSpec},
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

#[derive(Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub sui_client: sui_sdk::SuiClient,

    pub ibc_handler_address: String,
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        let sui_client = SuiClientBuilder::default().build(&config.rpc_url).await?;

        let chain_id = sui_client.read_api().get_chain_identifier().await?;

        Ok(Self {
            chain_id: ChainId::new(chain_id.to_string()),
            sui_client,
            ibc_handler_address: config.ibc_handler_address,
        })
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId,
    pub rpc_url: String,
    pub ibc_handler_address: String,
}

fn plugin_name(chain_id: &ChainId) -> String {
    pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

    format!("{PLUGIN_NAME}/{}", chain_id)
}

impl Module {
    fn plugin_name(&self) -> String {
        plugin_name(&self.chain_id)
    }

    async fn fetch_blocks(&self, height: u64) -> RpcResult<Op<VoyagerMessage>> {
        Ok(conc([
            call(PluginMessage::new(
                self.plugin_name(),
                ModuleCall::from(FetchTransactions { height }),
            )),
            {
                let latest_height = self
                    .sui_client
                    .read_api()
                    .get_latest_checkpoint_sequence_number()
                    .await
                    .unwrap();

                match latest_height.cmp(&latest_height) {
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
        ]))
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
            ModuleCall::FetchBlocks(FetchBlocks { height }) => self.fetch_blocks(height).await,
            ModuleCall::FetchTransactions(FetchTransactions { height }) => {
                info!("fetching block height {height}");

                let tx_digests = self
                    .sui_client
                    .read_api()
                    .get_checkpoint(sui_sdk::rpc_types::CheckpointId::SequenceNumber(height))
                    .await
                    .unwrap()
                    .transactions;

                let events = self
                    .sui_client
                    .read_api()
                    .multi_get_transactions_with_options(
                        tx_digests,
                        SuiTransactionBlockResponseOptions::new().with_events(),
                    )
                    .await
                    .unwrap()
                    .into_iter()
                    .flat_map(|tx| {
                        tx.events
                            .unwrap()
                            .data
                            .into_iter()
                            .map(move |events| (events, tx.digest))
                    })
                    .filter_map(|(e, hash)| {
                        (e.package_id.to_string() == self.ibc_handler_address).then_some((e, hash))
                    })
                    .map(|(e, hash)| {
                        let event = match e.type_.name.as_str() {
                            "ClientCreatedEvent" => {
                                let create_client: events::CreateClient =
                                    serde_json::from_value(e.parsed_json).unwrap();
                                events::IbcEvent::CreateClient(create_client)
                            }
                            e => panic!("unknown: {e}"),
                        };
                        call(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(MakeFullEvent {
                                event,
                                tx_hash: H256::new(hash.into_inner()),
                                height,
                            }),
                        ))
                    });

                Ok(conc(events))
            }
            ModuleCall::MakeFullEvent(MakeFullEvent {
                event,
                tx_hash,
                height,
            }) => {
                let (full_event, client_id): (FullEvent, ClientId) = match event {
                    events::IbcEvent::CreateClient(event) => (
                        CreateClient {
                            client_type: ClientType::new(event.client_type),
                            client_id: event.client_id.try_into().unwrap(),
                        }
                        .into(),
                        event.client_id.try_into().unwrap(),
                    ),
                };
                ibc_union_spec::log_event(&full_event, &self.chain_id);

                let voyager_client = e.try_get::<VoyagerClient>()?;

                let client_info = voyager_client
                    .client_info::<IbcUnion>(self.chain_id.clone(), client_id)
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        self.chain_id.clone(),
                        Height::new(height).into(),
                        client_id,
                    )
                    .await?;

                Ok(data(ChainEvent {
                    chain_id: self.chain_id.clone(),
                    client_info,
                    counterparty_chain_id: client_state_meta.counterparty_chain_id,
                    tx_hash,
                    // TODO: Review this, does it need to be +1?
                    provable_height: Height::new(height),
                    event: into_value::<FullEvent>(full_event),
                    ibc_spec_id: IbcUnion::ID,
                }))
            }
        }
        //     match msg {
        //         ModuleCall::FetchTransactions(FetchTransactions { height }) => {
        //             info!("fetching block height {height}");
        //             let events = self
        //                 .aptos_client
        //                 .get_block_by_height(height, true)
        //                 .await
        //                 .map_err(|e| {
        //                     ErrorObject::owned(
        //                         -1,
        //                         format!("error fetching height: {}", ErrorReporter(e)),
        //                         None::<()>,
        //                     )
        //                 })?
        //                 .into_inner()
        //                 .transactions
        //                 .unwrap_or_default()
        //                 .into_iter()
        //                 .filter_map(|tx| match tx {
        //                     Transaction::UserTransaction(tx) => Some(tx),
        //                     _ => None,
        //                 })
        //                 .flat_map(|tx| {
        //                     tx.events
        //                         .into_iter()
        //                         .map(move |events| (events, tx.info.hash))
        //                 })
        //                 .filter_map(|(e, hash)| match e.typ {
        //                     MoveType::Struct(s) => {
        //                         (s.address == self.ibc_handler_address).then_some((s, e.data, hash))
        //                     }
        //                     _ => None,
        //                 })
        //                 .filter(|(typ, _, _)| typ.name.0.as_str() != "CreateLensClient")
        //                 .map(|(typ, data, hash)| {
        //                     let event = match dbg!(typ).name.0.as_str() {
        //                         "CreateClient" => from_raw_event::<ibc::CreateClient>(data),
        //                         "UpdateClient" => from_raw_event::<ibc::UpdateClient>(data),
        //                         "ConnectionOpenInit" => from_raw_event::<ibc::ConnectionOpenInit>(data),
        //                         "ConnectionOpenTry" => from_raw_event::<ibc::ConnectionOpenTry>(data),
        //                         "ConnectionOpenAck" => from_raw_event::<ibc::ConnectionOpenAck>(data),
        //                         "ConnectionOpenConfirm" => {
        //                             from_raw_event::<ibc::ConnectionOpenConfirm>(data)
        //                         }
        //                         "ChannelOpenInit" => from_raw_event::<ibc::ChannelOpenInit>(data),
        //                         "ChannelOpenTry" => from_raw_event::<ibc::ChannelOpenTry>(data),
        //                         "ChannelOpenAck" => from_raw_event::<ibc::ChannelOpenAck>(data),
        //                         "ChannelOpenConfirm" => from_raw_event::<ibc::ChannelOpenConfirm>(data),
        //                         "WriteAck" => from_raw_event::<ibc::WriteAck>(data),
        //                         "PacketRecv" => from_raw_event::<ibc::PacketRecv>(data),
        //                         "PacketSend" => from_raw_event::<ibc::PacketSend>(data),
        //                         "PacketAck" => from_raw_event::<ibc::PacketAck>(data),
        //                         "TimeoutPacket" => from_raw_event::<ibc::TimeoutPacket>(data),
        //                         unknown => panic!("unknown event `{unknown}`"),
        //                     };
        //                     // TODO: Check the type before deserializing
        //                     call(PluginMessage::new(
        //                         self.plugin_name(),
        //                         ModuleCall::from(MakeFullEvent {
        //                             event,
        //                             tx_hash: H256::new(*hash.0),
        //                             height,
        //                         }),
        //                     ))
        //                 });

        //             Ok(conc(events))
        //         }
        //         ModuleCall::FetchBlocks(FetchBlocks { height }) => Ok(conc([
        //             call(PluginMessage::new(
        //                 self.plugin_name(),
        //                 ModuleCall::from(FetchTransactions { height }),
        //             )),
        //             {
        //                 let latest_height = self
        //                     .aptos_client
        //                     .get_index()
        //                     .await
        //                     .unwrap()
        //                     .into_inner()
        //                     .block_height
        //                     .0;
        //                 match height.cmp(&latest_height) {
        //                     Ordering::Less => {
        //                         let next_height = (latest_height - height).clamp(1, 10) + height;
        //                         conc(
        //                             ((height + 1)..next_height)
        //                                 .map(|height| {
        //                                     call(PluginMessage::new(
        //                                         self.plugin_name(),
        //                                         ModuleCall::from(FetchTransactions { height }),
        //                                     ))
        //                                 })
        //                                 .chain([call(PluginMessage::new(
        //                                     self.plugin_name(),
        //                                     ModuleCall::from(FetchBlocks {
        //                                         height: next_height,
        //                                     }),
        //                                 ))]),
        //                         )
        //                     }
        //                     Ordering::Equal | Ordering::Greater => seq([
        //                         call(WaitForHeight {
        //                             chain_id: self.chain_id.clone(),
        //                             height: Height::new(height + 1),
        //                             finalized: true,
        //                         }),
        //                         call(PluginMessage::new(
        //                             self.plugin_name(),
        //                             ModuleCall::from(FetchBlocks { height: height + 1 }),
        //                         )),
        //                     ]),
        //                 }
        //             },
        //         ])),
        //         ModuleCall::MakeFullEvent(MakeFullEvent {
        //             event,
        //             tx_hash,
        //             height,
        //         }) => {
        //             let (full_event, client_id): (FullEvent, ClientId) = match event {
        //                 events::IbcEvent::CreateClient(event) => (
        //                     CreateClient {
        //                         client_id: event.client_id.try_into().unwrap(),
        //                         client_type: ClientType::new(event.client_type),
        //                     }
        //                     .into(),
        //                     event.client_id.try_into().unwrap(),
        //                 ),
        //                 events::IbcEvent::UpdateClient(event) => (
        //                     UpdateClient {
        //                         client_id: event.client_id.try_into().unwrap(),
        //                         client_type: ClientType::new(event.client_type),
        //                         height: event.counterparty_height,
        //                     }
        //                     .into(),
        //                     event.client_id.try_into().unwrap(),
        //                 ),
        //                 events::IbcEvent::ConnectionOpenInit(event) => (
        //                     ConnectionOpenInit {
        //                         client_id: event.client_id.try_into().unwrap(),
        //                         connection_id: event.connection_id.try_into().unwrap(),
        //                         counterparty_client_id: event
        //                             .counterparty_client_id
        //                             .try_into()
        //                             .unwrap(),
        //                     }
        //                     .into(),
        //                     event.client_id.try_into().unwrap(),
        //                 ),
        //                 events::IbcEvent::ConnectionOpenTry(event) => (
        //                     ConnectionOpenTry {
        //                         client_id: event.client_id.try_into().unwrap(),
        //                         connection_id: event.connection_id.try_into().unwrap(),
        //                         counterparty_client_id: event
        //                             .counterparty_client_id
        //                             .try_into()
        //                             .unwrap(),
        //                         counterparty_connection_id: event
        //                             .counterparty_connection_id
        //                             .try_into()
        //                             .unwrap(),
        //                     }
        //                     .into(),
        //                     event.client_id.try_into().unwrap(),
        //                 ),
        //                 events::IbcEvent::ConnectionOpenAck(event) => (
        //                     ConnectionOpenAck {
        //                         client_id: event.client_id.try_into().unwrap(),
        //                         connection_id: event.connection_id.try_into().unwrap(),
        //                         counterparty_client_id: event
        //                             .counterparty_client_id
        //                             .try_into()
        //                             .unwrap(),
        //                         counterparty_connection_id: event
        //                             .counterparty_connection_id
        //                             .try_into()
        //                             .unwrap(),
        //                     }
        //                     .into(),
        //                     event.client_id.try_into().unwrap(),
        //                 ),
        //                 events::IbcEvent::ConnectionOpenConfirm(event) => (
        //                     ConnectionOpenConfirm {
        //                         client_id: event.client_id.try_into().unwrap(),
        //                         connection_id: event.connection_id.try_into().unwrap(),
        //                         counterparty_client_id: event
        //                             .counterparty_client_id
        //                             .try_into()
        //                             .unwrap(),
        //                         counterparty_connection_id: event
        //                             .counterparty_connection_id
        //                             .try_into()
        //                             .unwrap(),
        //                     }
        //                     .into(),
        //                     event.client_id.try_into().unwrap(),
        //                 ),
        //                 events::IbcEvent::ChannelOpenInit(event) => {
        //                     let ledger_version = self.ledger_version_of_height(height).await;

        //                     let connection = self
        //                         .get_connection(
        //                             self.ibc_handler_address.into(),
        //                             Some(ledger_version),
        //                             (event.connection_id,),
        //                         )
        //                         .await
        //                         .unwrap()
        //                         .unwrap();

        //                     let connection = convert_connection(connection);
        //                     let client_id = connection.client_id;

        //                     (
        //                         ChannelOpenInit {
        //                             port_id: event.port_id.parse().unwrap(),
        //                             channel_id: event.channel_id.try_into().unwrap(),
        //                             counterparty_port_id: event.counterparty_port_id.into(),
        //                             connection,
        //                             version: event.version,
        //                         }
        //                         .into(),
        //                         client_id,
        //                     )
        //                 }
        //                 events::IbcEvent::ChannelOpenTry(event) => {
        //                     let ledger_version = self.ledger_version_of_height(height).await;

        //                     let connection = self
        //                         .get_connection(
        //                             self.ibc_handler_address.into(),
        //                             Some(ledger_version),
        //                             (event.connection_id,),
        //                         )
        //                         .await
        //                         .unwrap()
        //                         .unwrap();

        //                     let connection = convert_connection(connection);

        //                     let client_id = connection.client_id;

        //                     (
        //                         ChannelOpenTry {
        //                             port_id: event.port_id.parse().unwrap(),
        //                             channel_id: event.channel_id.try_into().unwrap(),
        //                             counterparty_port_id: event.counterparty_port_id.into(),
        //                             counterparty_channel_id: event
        //                                 .counterparty_channel_id
        //                                 .try_into()
        //                                 .unwrap(),
        //                             connection,
        //                             version: event.version,
        //                         }
        //                         .into(),
        //                         client_id,
        //                     )
        //                 }
        //                 events::IbcEvent::ChannelOpenAck(event) => {
        //                     let ledger_version = self.ledger_version_of_height(height).await;

        //                     let connection = self
        //                         .get_connection(
        //                             self.ibc_handler_address.into(),
        //                             Some(ledger_version),
        //                             (event.connection_id,),
        //                         )
        //                         .await
        //                         .unwrap()
        //                         .unwrap();

        //                     let channel = self
        //                         .get_channel(
        //                             self.ibc_handler_address.into(),
        //                             Some(ledger_version),
        //                             (event.channel_id,),
        //                         )
        //                         .await
        //                         .unwrap()
        //                         .unwrap();

        //                     let connection = convert_connection(connection);

        //                     let client_id = connection.client_id;

        //                     (
        //                         ChannelOpenAck {
        //                             port_id: event.port_id.parse().unwrap(),
        //                             channel_id: event.channel_id.try_into().unwrap(),
        //                             counterparty_port_id: event.counterparty_port_id.into(),
        //                             counterparty_channel_id: event
        //                                 .counterparty_channel_id
        //                                 .try_into()
        //                                 .unwrap(),
        //                             connection,
        //                             version: channel.version,
        //                         }
        //                         .into(),
        //                         client_id,
        //                     )
        //                 }
        //                 events::IbcEvent::ChannelOpenConfirm(event) => {
        //                     let ledger_version = self.ledger_version_of_height(height).await;

        //                     let connection = self
        //                         .get_connection(
        //                             self.ibc_handler_address.into(),
        //                             Some(ledger_version),
        //                             (event.connection_id,),
        //                         )
        //                         .await
        //                         .unwrap()
        //                         .unwrap();

        //                     let channel = self
        //                         .get_channel(
        //                             self.ibc_handler_address.into(),
        //                             Some(ledger_version),
        //                             (event.channel_id,),
        //                         )
        //                         .await
        //                         .unwrap()
        //                         .unwrap();

        //                     let connection = convert_connection(connection);

        //                     let client_id = connection.client_id;

        //                     (
        //                         ChannelOpenConfirm {
        //                             port_id: event.port_id.parse().unwrap(),
        //                             channel_id: event.channel_id.try_into().unwrap(),
        //                             counterparty_port_id: event.counterparty_port_id.into(),
        //                             counterparty_channel_id: event
        //                                 .counterparty_channel_id
        //                                 .try_into()
        //                                 .unwrap(),
        //                             connection,
        //                             version: channel.version,
        //                         }
        //                         .into(),
        //                         client_id,
        //                     )
        //                 }
        //                 events::IbcEvent::WriteAcknowledgement(event) => {
        //                     let (
        //                         _counterparty_chain_id,
        //                         _client_info,
        //                         destination_channel,
        //                         source_channel,
        //                     ) = self
        //                         .make_packet_metadata(
        //                             self.make_height(height),
        //                             event.packet.destination_channel_id.try_into().unwrap(),
        //                             e.try_get()?,
        //                         )
        //                         .await?;

        //                     let client_id = destination_channel.connection.client_id;

        //                     (
        //                         WriteAck {
        //                             packet_data: event.packet.data.into(),
        //                             acknowledgement: event.acknowledgement.into(),
        //                             packet: PacketMetadata {
        //                                 source_channel,
        //                                 destination_channel,
        //                                 timeout_height: event.packet.timeout_height,
        //                                 timeout_timestamp: Timestamp::from_nanos(
        //                                     event.packet.timeout_timestamp,
        //                                 ),
        //                             },
        //                         }
        //                         .into(),
        //                         client_id,
        //                     )
        //                 }
        //                 events::IbcEvent::RecvPacket(event) => {
        //                     let (
        //                         _counterparty_chain_id,
        //                         _client_info,
        //                         destination_channel,
        //                         source_channel,
        //                     ) = self
        //                         .make_packet_metadata(
        //                             self.make_height(height),
        //                             event.packet.destination_channel_id.try_into().unwrap(),
        //                             e.try_get()?,
        //                         )
        //                         .await?;

        //                     let client_id = destination_channel.connection.client_id;

        //                     (
        //                         PacketRecv {
        //                             packet_data: event.packet.data.into(),
        //                             packet: PacketMetadata {
        //                                 source_channel,
        //                                 destination_channel,
        //                                 timeout_height: event.packet.timeout_height,
        //                                 timeout_timestamp: Timestamp::from_nanos(
        //                                     event.packet.timeout_timestamp,
        //                                 ),
        //                             },
        //                             maker_msg: Default::default(),
        //                         }
        //                         .into(),
        //                         client_id,
        //                     )
        //                 }
        //                 events::IbcEvent::SendPacket(event) => {
        //                     let (
        //                         _counterparty_chain_id,
        //                         _client_info,
        //                         source_channel,
        //                         destination_channel,
        //                     ) = self
        //                         .make_packet_metadata(
        //                             self.make_height(height),
        //                             event.source_channel_id.try_into().unwrap(),
        //                             e.try_get()?,
        //                         )
        //                         .await?;

        //                     let client_id = source_channel.connection.client_id;

        //                     (
        //                         PacketSend {
        //                             packet_data: event.data.into(),
        //                             packet: PacketMetadata {
        //                                 source_channel,
        //                                 destination_channel,
        //                                 timeout_height: event.timeout_height,
        //                                 timeout_timestamp: Timestamp::from_nanos(
        //                                     event.timeout_timestamp,
        //                                 ),
        //                             },
        //                         }
        //                         .into(),
        //                         client_id,
        //                     )
        //                 }
        //                 events::IbcEvent::AcknowledgePacket(event) => {
        //                     let (
        //                         _counterparty_chain_id,
        //                         _client_info,
        //                         source_channel,
        //                         destination_channel,
        //                     ) = self
        //                         .make_packet_metadata(
        //                             self.make_height(height),
        //                             event.packet.source_channel_id.try_into().unwrap(),
        //                             e.try_get()?,
        //                         )
        //                         .await?;

        //                     let client_id = source_channel.connection.client_id;

        //                     (
        //                         PacketAck {
        //                             packet_data: event.packet.data.into(),
        //                             packet: PacketMetadata {
        //                                 source_channel,
        //                                 destination_channel,
        //                                 timeout_height: event.packet.timeout_height,
        //                                 timeout_timestamp: Timestamp::from_nanos(
        //                                     event.packet.timeout_timestamp,
        //                                 ),
        //                             },
        //                             acknowledgement: event.acknowledgement.into(),
        //                         }
        //                         .into(),
        //                         client_id,
        //                     )
        //                 }
        //                 events::IbcEvent::TimeoutPacket(_) => todo!(),
        //             };

        //             ibc_union_spec::log_event(&full_event, &self.chain_id);

        //             let voyager_client = e.try_get::<VoyagerClient>()?;

        //             let client_info = voyager_client
        //                 .client_info::<IbcUnion>(self.chain_id.clone(), client_id)
        //                 .await?;

        //             let client_state_meta = voyager_client
        //                 .client_state_meta::<IbcUnion>(
        //                     self.chain_id.clone(),
        //                     self.make_height(height).into(),
        //                     client_id,
        //                 )
        //                 .await?;

        //             Ok(data(ChainEvent {
        //                 chain_id: self.chain_id.clone(),
        //                 client_info,
        //                 counterparty_chain_id: client_state_meta.counterparty_chain_id,
        //                 tx_hash,
        //                 // TODO: Review this, does it need to be +1?
        //                 provable_height: self.make_height(height),
        //                 event: into_value::<FullEvent>(full_event),
        //                 ibc_spec_id: IbcUnion::ID,
        //             }))
        //         }
        //     }
    }
}

// pub fn rest_error_to_rpc_error(e: RestError) -> ErrorObjectOwned {
//     ErrorObject::owned(-1, format!("rest error: {}", ErrorReporter(e)), None::<()>)
// }

// fn convert_connection(connection: ConnectionEnd) -> Connection {
//     Connection {
//         state: match connection.state {
//             1 => ConnectionState::Init,
//             2 => ConnectionState::TryOpen,
//             3 => ConnectionState::Open,
//             _ => panic!("connection state must be 1..=3"),
//         },
//         client_id: connection.client_id.try_into().unwrap(),
//         counterparty_client_id: connection.counterparty_client_id.try_into().unwrap(),
//         counterparty_connection_id: connection.counterparty_connection_id.try_into().ok(),
//     }
// }

// fn from_raw_event<T: MoveOutputType + Into<events::IbcEvent>>(data: Value) -> events::IbcEvent {
//     let raw_event = serde_json::from_value::<T::Raw>(data).unwrap();
//     T::from_raw(raw_event).into()
// }

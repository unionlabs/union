use std::{cmp::Ordering, collections::VecDeque};

use call::FetchEvents;
use ibc_union_spec::{
    event::{
        ChannelMetadata, ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit, ChannelOpenTry,
        ConnectionMetadata, ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit,
        ConnectionOpenTry, CreateClient, FullEvent, PacketMetadata, PacketRecv, PacketSend,
        UpdateClient,
    },
    path::{ChannelPath, ConnectionPath},
    query::PacketByHash,
    Channel, ChannelId, ChannelState, ClientId, Connection, ConnectionId, IbcUnion, Timestamp,
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use sui_sdk::{
    rpc_types::{EventFilter, SuiEvent, SuiTransactionBlockResponseOptions},
    types::base_types::SuiAddress,
    SuiClientBuilder,
};
use tracing::{info, instrument};
use unionlabs::{ibc::core::client::height::Height, primitives::H256, ErrorReporter};
use voyager_sdk::{
    hook::simple_take_filter,
    into_value,
    message::{
        call::{Call, WaitForHeight, WaitForHeightRelative},
        data::{ChainEvent, Data, EventProvableHeight},
        PluginMessage, VoyagerMessage,
    },
    plugin::Plugin,
    primitives::{ChainId, ClientInfo, ClientType, IbcSpec, QueryHeight},
    rpc::{types::PluginInfo, PluginServer},
    vm::{call, conc, data, noop, pass::PassResult, seq, Op},
    DefaultCmd, ExtensionsExt, VoyagerClient,
};

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

    pub ibc_handler_address: SuiAddress,
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> anyhow::Result<Self> {
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
                r#"[.. | (."@type"? == "index" or ."@type"? == "index_range") and ."@value".chain_id == "{}"] | any"#,
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
    pub ibc_handler_address: SuiAddress,
}

fn plugin_name(chain_id: &ChainId) -> String {
    pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

    format!("{PLUGIN_NAME}/{}", chain_id)
}

impl Module {
    fn plugin_name(&self) -> String {
        plugin_name(&self.chain_id)
    }

    fn make_full_event(&self, e: SuiEvent) -> Option<events::IbcEvent> {
        match e.type_.name.as_str() {
            "CreateClient" => {
                let create_client: events::CreateClient =
                    serde_json::from_value(e.parsed_json).unwrap();
                Some(events::IbcEvent::CreateClient(create_client))
            }
            "UpdateClient" => {
                let update_client: events::UpdateClient =
                    serde_json::from_value(e.parsed_json).unwrap();
                Some(events::IbcEvent::UpdateClient(update_client))
            }
            "ConnectionOpenInit" => {
                let connection_open: events::ConnectionOpenInit =
                    serde_json::from_value(e.parsed_json).unwrap();
                Some(events::IbcEvent::ConnectionOpenInit(connection_open))
            }
            "ConnectionOpenTry" => {
                let connection_open: events::ConnectionOpenTry =
                    serde_json::from_value(e.parsed_json).unwrap();
                Some(events::IbcEvent::ConnectionOpenTry(connection_open))
            }
            "ConnectionOpenAck" => {
                let connection_open: events::ConnectionOpenAck =
                    serde_json::from_value(e.parsed_json).unwrap();
                Some(events::IbcEvent::ConnectionOpenAck(connection_open))
            }
            "ConnectionOpenConfirm" => {
                let connection_open: events::ConnectionOpenConfirm =
                    serde_json::from_value(e.parsed_json).unwrap();
                Some(events::IbcEvent::ConnectionOpenConfirm(connection_open))
            }
            "ChannelOpenInit" => {
                let channel_open: events::ChannelOpenInit =
                    serde_json::from_value(e.parsed_json).unwrap();
                Some(events::IbcEvent::ChannelOpenInit(channel_open))
            }
            "ChannelOpenTry" => {
                let channel_open: events::ChannelOpenTry =
                    serde_json::from_value(e.parsed_json).unwrap();
                Some(events::IbcEvent::ChannelOpenTry(channel_open))
            }
            "ChannelOpenAck" => {
                let channel_open: events::ChannelOpenAck =
                    serde_json::from_value(e.parsed_json).unwrap();
                Some(events::IbcEvent::ChannelOpenAck(channel_open))
            }
            "ChannelOpenConfirm" => {
                let channel_open: events::ChannelOpenConfirm =
                    serde_json::from_value(e.parsed_json).unwrap();
                Some(events::IbcEvent::ChannelOpenConfirm(channel_open))
            }
            "PacketSend" => {
                let channel_open: events::PacketSend =
                    serde_json::from_value(e.parsed_json).unwrap();
                Some(events::IbcEvent::PacketSend(channel_open))
            }
            "PacketRecv" => {
                let packet_recv: events::PacketRecv =
                    serde_json::from_value(e.parsed_json).unwrap();
                Some(events::IbcEvent::PacketRecv(packet_recv))
            }
            "Initiated" => None,
            e => panic!("unknown: {e}"),
        }
    }

    async fn fetch_blocks(
        &self,
        voyager_client: &VoyagerClient,
        height: u64,
    ) -> RpcResult<Op<VoyagerMessage>> {
        Ok(conc([
            call(PluginMessage::new(
                self.plugin_name(),
                ModuleCall::from(FetchTransactions { height }),
            )),
            {
                let latest_height = voyager_client
                    .query_latest_height(self.chain_id.clone(), true)
                    .await?
                    .height();

                match latest_height.cmp(&latest_height) {
                    Ordering::Less => {
                        let next_height = (latest_height - height).clamp(1, 20) + height;
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

    async fn make_packet_metadata(
        &self,
        event_height: Height,
        self_channel_id: ChannelId,
        voyager_client: &VoyagerClient,
    ) -> RpcResult<(ChainId, ClientInfo, ChannelMetadata, ChannelMetadata)> {
        let self_channel = voyager_client
            .query_ibc_state(
                self.chain_id.clone(),
                QueryHeight::Specific(event_height),
                ChannelPath {
                    channel_id: self_channel_id,
                },
            )
            .await?;

        let self_connection_id = self_channel.connection_id;
        let self_connection = voyager_client
            .query_ibc_state(
                self.chain_id.clone(),
                QueryHeight::Specific(event_height),
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
                QueryHeight::Specific(counterparty_latest_height),
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

    async fn connection_event_to_chain_event(
        &self,
        voyager_client: &VoyagerClient,
        client_id: ClientId,
        tx_hash: H256,
        event: FullEvent,
        provable_height: u64,
    ) -> RpcResult<Op<VoyagerMessage>> {
        let client_info = voyager_client
            .client_info::<IbcUnion>(self.chain_id.clone(), client_id)
            .await?;

        let client_state_meta = voyager_client
            .client_state_meta::<IbcUnion>(
                self.chain_id.clone(),
                Height::new(provable_height).into(),
                client_id,
            )
            .await?;

        ibc_union_spec::log_event(&event, &self.chain_id);

        Ok(data(ChainEvent {
            chain_id: self.chain_id.clone(),
            client_info,
            counterparty_chain_id: client_state_meta.counterparty_chain_id,
            tx_hash,
            provable_height: EventProvableHeight::Exactly(Height::new(provable_height)),
            ibc_spec_id: IbcUnion::ID,
            event: into_value::<FullEvent>(event),
        }))
    }

    async fn channel_event_to_chain_event<EventFn: FnOnce(Channel, Connection) -> FullEvent>(
        &self,
        voyager_client: &VoyagerClient,
        channel_id: ChannelId,
        connection_id: ConnectionId,
        expected_state: ChannelState,
        tx_hash: H256,
        event_fn: EventFn,
        provable_height: u64,
    ) -> RpcResult<Op<VoyagerMessage>> {
        let provable_height = Height::new(provable_height);
        let channel = voyager_client
            .query_ibc_state(
                self.chain_id.clone(),
                QueryHeight::Specific(provable_height),
                ChannelPath { channel_id },
            )
            .await?;

        if channel.state != expected_state {
            info!(state = %channel.state, "channel state is not init");
            return Ok(noop());
        }

        let connection = voyager_client
            .query_ibc_state(
                self.chain_id.clone(),
                QueryHeight::Specific(provable_height),
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

        let event = event_fn(channel, connection);

        ibc_union_spec::log_event(&event, &self.chain_id);

        Ok(data(ChainEvent {
            chain_id: self.chain_id.clone(),
            client_info,
            counterparty_chain_id: client_state_meta.counterparty_chain_id,
            tx_hash,
            provable_height: EventProvableHeight::Exactly(provable_height),
            ibc_spec_id: IbcUnion::ID,
            event: into_value::<FullEvent>(event),
        }))
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
                    Op::Call(Call::Index(fetch)) if fetch.chain_id == self.chain_id => {
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
            ModuleCall::FetchEvents(FetchEvents { cursor }) => {
                info!("fetching events: cursor: {:?}", cursor);

                let mut events = self
                    .sui_client
                    .event_api()
                    .query_events(
                        EventFilter::MoveEventModule {
                            package: self.ibc_handler_address.into(),
                            module: "ibc".parse().unwrap(),
                        },
                        cursor,
                        Some(1),
                        false,
                    )
                    .await
                    .unwrap();

                if events.data.is_empty() {
                    Ok(seq([
                        call(WaitForHeightRelative {
                            chain_id: self.chain_id.clone(),
                            height_diff: 10,
                            finalized: true,
                        }),
                        call(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(FetchEvents { cursor }),
                        )),
                    ]))
                } else {
                    assert!(events.data.len() == 1);

                    let tx_hash = events.data[0].id.tx_digest;

                    let event = self.make_full_event(events.data.pop().unwrap());

                    let height = self
                        .sui_client
                        .read_api()
                        .get_transaction_with_options(
                            tx_hash,
                            SuiTransactionBlockResponseOptions::new(),
                        )
                        .await
                        .unwrap()
                        .checkpoint
                        .unwrap();

                    // do not do anything if the event is `initiated`
                    if let Some(event) = event {
                        Ok(conc([
                            call(PluginMessage::new(
                                self.plugin_name(),
                                ModuleCall::from(MakeFullEvent {
                                    event,
                                    tx_hash: H256::new(tx_hash.into_inner()),
                                    height,
                                }),
                            )),
                            call(PluginMessage::new(
                                self.plugin_name(),
                                ModuleCall::from(FetchEvents {
                                    cursor: events.next_cursor,
                                }),
                            )),
                        ]))
                    } else {
                        Ok(call(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(FetchEvents {
                                cursor: events.next_cursor,
                            }),
                        )))
                    }
                }
            }
            ModuleCall::FetchBlocks(FetchBlocks { height }) => {
                self.fetch_blocks(e.voyager_client()?, height).await
            }
            ModuleCall::FetchTransactions(FetchTransactions { height }) => {
                info!("fetching block height {height}");

                let tx_digests = self
                    .sui_client
                    .read_api()
                    .get_checkpoint(sui_sdk::rpc_types::CheckpointId::SequenceNumber(height))
                    .await
                    .map_err(|e| {
                        ErrorObject::owned(
                            -1,
                            ErrorReporter(e).with_message("error fetching a checkpoint"),
                            None::<()>,
                        )
                    })?
                    .transactions;

                let events = self
                    .sui_client
                    .read_api()
                    .multi_get_transactions_with_options(
                        tx_digests,
                        SuiTransactionBlockResponseOptions::new().with_events(),
                    )
                    .await
                    .map_err(|e| {
                        ErrorObject::owned(
                            -1,
                            ErrorReporter(e).with_message("error fetching txs"),
                            None::<()>,
                        )
                    })?
                    .into_iter()
                    .flat_map(|tx| {
                        tx.events
                            .expect("events exist")
                            .data
                            .into_iter()
                            .map(move |events| (events, tx.digest))
                    })
                    .filter_map(|(e, hash)| {
                        (e.type_.address == self.ibc_handler_address.into()).then_some((e, hash))
                    })
                    .map(|(e, hash)| {
                        println!("event: {e:?}");
                        let event = self.make_full_event(e).unwrap();

                        info!("found event: {event:?}");
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
                let chain_event = |counterparty_chain_id: ChainId,
                                   event,
                                   client_info|
                 -> RpcResult<Op<VoyagerMessage>> {
                    ibc_union_spec::log_event(&event, &self.chain_id);

                    Ok(data(ChainEvent {
                        chain_id: self.chain_id.clone(),
                        client_info,
                        counterparty_chain_id,
                        tx_hash,
                        provable_height: EventProvableHeight::Exactly(Height::new(height)),
                        ibc_spec_id: IbcUnion::ID,
                        event: into_value::<FullEvent>(event),
                    }))
                };

                let voyager_client = e.voyager_client()?;

                match event {
                    events::IbcEvent::CreateClient(raw_event) => {
                        let client_info = voyager_client
                            .client_info::<IbcUnion>(
                                self.chain_id.clone(),
                                raw_event.client_id.try_into().unwrap(),
                            )
                            .await?;

                        let event = CreateClient {
                            client_type: ClientType::new(raw_event.client_type),
                            client_id: raw_event.client_id.try_into().unwrap(),
                        }
                        .into();

                        chain_event(
                            ChainId::new(raw_event.counterparty_chain_id),
                            event,
                            client_info,
                        )
                    }
                    events::IbcEvent::UpdateClient(raw_event) => {
                        let client_id = raw_event.client_id.try_into().unwrap();

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

                        let event = UpdateClient {
                            client_type: ClientType::new(raw_event.client_type),
                            client_id: raw_event.client_id.try_into().unwrap(),
                            height: raw_event.height.0,
                        }
                        .into();

                        chain_event(client_state_meta.counterparty_chain_id, event, client_info)
                    }
                    events::IbcEvent::ConnectionOpenInit(raw_event) => {
                        let client_id = raw_event.client_id.try_into().unwrap();
                        self.connection_event_to_chain_event(
                            voyager_client,
                            client_id,
                            tx_hash,
                            ConnectionOpenInit {
                                client_id,
                                connection_id: raw_event.connection_id.try_into().unwrap(),
                                counterparty_client_id: raw_event
                                    .counterparty_client_id
                                    .try_into()
                                    .unwrap(),
                            }
                            .into(),
                            height,
                        )
                        .await
                    }
                    events::IbcEvent::ConnectionOpenTry(raw_event) => {
                        let client_id = raw_event.client_id.try_into().unwrap();
                        self.connection_event_to_chain_event(
                            voyager_client,
                            client_id,
                            tx_hash,
                            ConnectionOpenTry {
                                client_id,
                                connection_id: raw_event.connection_id.try_into().unwrap(),
                                counterparty_client_id: raw_event
                                    .counterparty_client_id
                                    .try_into()
                                    .unwrap(),
                                counterparty_connection_id: raw_event
                                    .counterparty_connection_id
                                    .try_into()
                                    .unwrap(),
                            }
                            .into(),
                            height,
                        )
                        .await
                    }
                    events::IbcEvent::ConnectionOpenAck(raw_event) => {
                        let client_id = raw_event.client_id.try_into().unwrap();
                        self.connection_event_to_chain_event(
                            voyager_client,
                            client_id,
                            tx_hash,
                            ConnectionOpenAck {
                                client_id: raw_event.client_id.try_into().unwrap(),
                                connection_id: raw_event.connection_id.try_into().unwrap(),
                                counterparty_client_id: raw_event
                                    .counterparty_client_id
                                    .try_into()
                                    .unwrap(),
                                counterparty_connection_id: raw_event
                                    .counterparty_connection_id
                                    .try_into()
                                    .unwrap(),
                            }
                            .into(),
                            height,
                        )
                        .await
                    }
                    events::IbcEvent::ConnectionOpenConfirm(raw_event) => {
                        let client_id = raw_event.client_id.try_into().unwrap();
                        self.connection_event_to_chain_event(
                            voyager_client,
                            client_id,
                            tx_hash,
                            ConnectionOpenConfirm {
                                client_id: raw_event.client_id.try_into().unwrap(),
                                connection_id: raw_event.connection_id.try_into().unwrap(),
                                counterparty_client_id: raw_event
                                    .counterparty_client_id
                                    .try_into()
                                    .unwrap(),
                                counterparty_connection_id: raw_event
                                    .counterparty_connection_id
                                    .try_into()
                                    .unwrap(),
                            }
                            .into(),
                            height,
                        )
                        .await
                    }
                    events::IbcEvent::ChannelOpenInit(raw_event) => {
                        let channel_id = raw_event.channel_id.try_into().unwrap();
                        let connection_id = raw_event.connection_id.try_into().unwrap();
                        self.channel_event_to_chain_event(
                            voyager_client,
                            channel_id,
                            connection_id,
                            ChannelState::Init,
                            tx_hash,
                            |channel, connection| {
                                ChannelOpenInit {
                                    port_id: raw_event.port_id.into_bytes().into(),
                                    channel_id,
                                    counterparty_port_id: raw_event.counterparty_port_id.into(),
                                    connection,
                                    version: channel.version,
                                }
                                .into()
                            },
                            height,
                        )
                        .await
                    }
                    events::IbcEvent::ChannelOpenTry(raw_event) => {
                        let channel_id = raw_event.channel_id.try_into().unwrap();
                        let connection_id = raw_event.connection_id.try_into().unwrap();
                        self.channel_event_to_chain_event(
                            voyager_client,
                            channel_id,
                            connection_id,
                            ChannelState::TryOpen,
                            tx_hash,
                            |channel, connection| {
                                ChannelOpenTry {
                                    port_id: raw_event.port_id.into_bytes().into(),
                                    channel_id,
                                    counterparty_port_id: raw_event.counterparty_port_id.into(),
                                    counterparty_channel_id: raw_event
                                        .counterparty_channel_id
                                        .try_into()
                                        .unwrap(),
                                    connection,
                                    version: channel.version,
                                }
                                .into()
                            },
                            height,
                        )
                        .await
                    }
                    events::IbcEvent::ChannelOpenAck(raw_event) => {
                        let channel_id = raw_event.channel_id.try_into().unwrap();
                        let connection_id = raw_event.connection_id.try_into().unwrap();
                        self.channel_event_to_chain_event(
                            voyager_client,
                            channel_id,
                            connection_id,
                            ChannelState::Open,
                            tx_hash,
                            |channel, connection| {
                                ChannelOpenAck {
                                    port_id: raw_event.port_id.into_bytes().into(),
                                    channel_id,
                                    counterparty_port_id: raw_event.counterparty_port_id.into(),
                                    counterparty_channel_id: raw_event
                                        .counterparty_channel_id
                                        .try_into()
                                        .unwrap(),
                                    connection,
                                    version: channel.version,
                                }
                                .into()
                            },
                            height,
                        )
                        .await
                    }
                    events::IbcEvent::ChannelOpenConfirm(raw_event) => {
                        let channel_id = raw_event.channel_id.try_into().unwrap();
                        let connection_id = raw_event.connection_id.try_into().unwrap();
                        self.channel_event_to_chain_event(
                            voyager_client,
                            channel_id,
                            connection_id,
                            ChannelState::Open,
                            tx_hash,
                            |channel, connection| {
                                ChannelOpenConfirm {
                                    port_id: raw_event.port_id.into_bytes().into(),
                                    channel_id,
                                    counterparty_port_id: channel.counterparty_port_id,
                                    counterparty_channel_id: raw_event
                                        .counterparty_channel_id
                                        .try_into()
                                        .unwrap(),
                                    connection,
                                    version: channel.version,
                                }
                                .into()
                            },
                            height,
                        )
                        .await
                    }
                    events::IbcEvent::PacketSend(event) => {
                        // TODO(aeryz): add ack check
                        let packet: events::Packet = event.packet;

                        let (
                            counterparty_chain_id,
                            client_info,
                            source_channel,
                            destination_channel,
                        ) = self
                            .make_packet_metadata(
                                Height::new(height),
                                packet.source_channel_id.try_into().unwrap(),
                                voyager_client,
                            )
                            .await?;

                        chain_event(
                            counterparty_chain_id,
                            PacketSend {
                                packet_data: packet.data.into(),
                                packet: PacketMetadata {
                                    source_channel,
                                    destination_channel,
                                    timeout_timestamp: Timestamp::from_nanos(
                                        packet.timeout_timestamp.0,
                                    ),
                                },
                            }
                            .into(),
                            client_info,
                        )
                    }
                    events::IbcEvent::PacketRecv(event) => {
                        let voyager_client = e.voyager_client()?;
                        let (
                            counterparty_chain_id,
                            client_info,
                            destination_channel,
                            source_channel,
                        ) = self
                            .make_packet_metadata(
                                Height::new(height),
                                event.channel_id.try_into().unwrap(),
                                voyager_client,
                            )
                            .await?;

                        let packet = voyager_client
                            .query(
                                counterparty_chain_id.clone(),
                                PacketByHash {
                                    channel_id: source_channel.channel_id,
                                    packet_hash: event.packet_hash.try_into().unwrap(),
                                },
                            )
                            .await?;

                        chain_event(
                            counterparty_chain_id,
                            PacketRecv {
                                packet_data: packet.data.into(),
                                packet: PacketMetadata {
                                    source_channel,
                                    destination_channel,
                                    timeout_height: packet.timeout_height,
                                    timeout_timestamp: packet.timeout_timestamp,
                                },
                                maker_msg: event.maker_msg.into(),
                            }
                            .into(),
                            client_info,
                        )
                    }
                }
            }
        }
    }
}

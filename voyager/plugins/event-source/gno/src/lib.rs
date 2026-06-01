#![warn(clippy::unwrap_used)]

use std::{
    cmp::Ordering,
    collections::{BTreeSet, VecDeque},
};

use ibc_union_spec::{
    Connection, ConnectionState, IbcUnion, MustBeZero, Packet,
    event::{
        ChannelMetadata, ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit, ChannelOpenTry,
        ConnectionMetadata, ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit,
        ConnectionOpenTry, CounterpartyChannelMetadata, CreateClient, PacketMetadata, PacketRecv,
        PacketSend, UpdateClient, WriteAck,
    },
    path::BatchPacketsPath,
    query::PacketByHash,
};
use jsonrpsee::{Extensions, core::async_trait};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::{debug, error, info, instrument, trace, warn};
use unionlabs::{ibc::core::client::height::Height, never::Never, primitives::H256};
use voyager_sdk::{
    ExtensionsExt, VoyagerClient,
    anyhow::{self, bail},
    hook::simple_take_filter,
    into_value,
    message::{
        PluginMessage, VoyagerMessage,
        call::{Call, WaitForHeight},
        data::{ChainEvent, Data, EventProvableHeight},
    },
    plugin::Plugin,
    primitives::{ChainId, ClientType, QueryHeight},
    rpc::{PluginServer, RpcError, RpcErrorExt, RpcResult, types::PluginInfo},
    vm::{Op, call, conc, data, noop, pass::PassResult, seq},
};

use crate::{
    call::{FetchBlock, FetchBlocks, MakeChainEvent, ModuleCall},
    ibc_events::{ChannelEvent, IbcEvent},
};

pub mod ibc_events;

pub mod call;

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub gno_client: gno_rpc::Client,

    pub chunk_block_fetch_size: u64,
    pub refetch_delay: u64,

    pub index_trivial_events: bool,

    pub ibc_core_realm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId,

    pub rpc_url: String,

    #[serde(default = "default_chunk_block_fetch_size")]
    pub chunk_block_fetch_size: u64,

    #[serde(default = "default_refetch_delay")]
    pub refetch_delay: u64,

    /// Whether or not to fully index events that do not produce a counterparty action (packet_recv, packet_acknowledgement, packet_timeout, update_client).
    #[serde(default)]
    pub index_trivial_events: bool,

    #[serde(default)]
    pub ibc_core_realm: String,
}

fn default_chunk_block_fetch_size() -> u64 {
    10
}

fn default_refetch_delay() -> u64 {
    120
}

#[derive(clap::Subcommand)]
pub enum Cmd {
    /// Return an op to fetch the events from a single block from the chain.
    FetchSingleBlock { height: Height },
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = Never;

    type Config = Config;
    type Cmd = Cmd;

    async fn new(config: Self::Config) -> anyhow::Result<Self> {
        let gno_client = gno_rpc::Client::new(config.rpc_url).await?;

        let chain_id = gno_client.status(None).await?.node_info.network;

        if chain_id != config.chain_id.as_str() {
            bail!(
                "incorrect chain id: expected `{}`, but found `{}`",
                config.chain_id,
                chain_id
            );
        }

        Ok(Self {
            gno_client,
            chain_id: ChainId::new(chain_id),
            chunk_block_fetch_size: config.chunk_block_fetch_size,
            refetch_delay: config.refetch_delay,
            index_trivial_events: config.index_trivial_events,
            ibc_core_realm: config.ibc_core_realm,
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

    async fn cmd(config: Self::Config, cmd: Self::Cmd) {
        match cmd {
            Cmd::FetchSingleBlock { height } => {
                print!(
                    "{}",
                    into_value(call::<VoyagerMessage>(PluginMessage::new(
                        plugin_name(&config.chain_id),
                        ModuleCall::from(FetchBlock {
                            already_seen_events: Default::default(),
                            height,
                        })
                    )))
                )
            }
        }
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
                    Op::Call(Call::Index(fetch)) if fetch.chain_id == self.chain_id => {
                        call(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(FetchBlocks {
                                height: fetch.start_height,
                                until: None,
                            }),
                        ))
                    }
                    Op::Call(Call::IndexRange(fetch)) if fetch.chain_id == self.chain_id => {
                        call(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(FetchBlocks {
                                height: fetch.range.from_height(),
                                until: Some(fetch.range.to_height()),
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
            ModuleCall::FetchBlocks(FetchBlocks { height, until }) => {
                self.fetch_blocks(e.voyager_client()?, height, until).await
            }
            ModuleCall::FetchBlock(FetchBlock {
                already_seen_events,
                height,
            }) => self.fetch_block(height, already_seen_events).await,
            ModuleCall::MakeChainEvent(MakeChainEvent {
                height,
                tx_hash,
                event,
            }) => {
                self.make_chain_event(e.voyager_client()?, height, tx_hash, event)
                    .await
            }
        }
    }
}

impl Module {
    #[instrument(skip_all, fields(%height))]
    async fn fetch_blocks(
        &self,
        voyager_client: &VoyagerClient,
        height: Height,
        until: Option<Height>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        if let Some(until) = until {
            if height.height() > until.height() {
                return Err(RpcError::fatal_from_message(format!(
                    "height {height} cannot be greater than the until height {until}"
                )));
            } else if height.height() == until.height() {
                // if this is a ranged fetch, we need to fetch the upper bound of the range individually since FetchBlocks is exclusive on the upper bound
                return Ok(call(PluginMessage::new(
                    self.plugin_name(),
                    ModuleCall::from(FetchBlock {
                        already_seen_events: None,
                        height,
                    }),
                )));
            }
        }

        let latest_height = voyager_client
            .query_latest_height(self.chain_id.clone(), true)
            .await?;

        info!(%latest_height, %height, ?until, "fetching blocks");

        let continuation = |next_height: Height| {
            seq([
                // TODO: Make this a config param
                call(WaitForHeight {
                    chain_id: self.chain_id.clone(),
                    height: next_height,
                    finalized: true,
                }),
                call(PluginMessage::new(
                    self.plugin_name(),
                    ModuleCall::from(FetchBlocks {
                        height: next_height,
                        until,
                    }),
                )),
            ])
        };

        match height.cmp(&latest_height) {
            // height < latest_height
            // fetch transactions on all blocks height..next_height (*exclusive* on the upper bound!)
            // and then queue the continuation starting at next_height
            Ordering::Equal | Ordering::Less => {
                let next_height = (latest_height.height() - height.height())
                    .clamp(1, self.chunk_block_fetch_size)
                    + height.height();

                let next_height =
                    next_height.min(until.map_or(next_height, |until| until.height()));

                info!(
                    from_height = height.height(),
                    to_height = next_height,
                    ?until,
                    "batch fetching blocks in range {height}..{next_height}"
                );

                Ok(conc(
                    (height.height()..next_height)
                        .map(|h| {
                            call(PluginMessage::new(
                                self.plugin_name(),
                                ModuleCall::from(FetchBlock {
                                    already_seen_events: None,
                                    height: Height::new_with_revision(height.revision(), h),
                                }),
                            ))
                        })
                        .chain([continuation(Height::new_with_revision(
                            height.revision(),
                            next_height,
                        ))]),
                ))
            }
            Ordering::Greater => {
                warn!(
                    "the latest finalized height ({latest_height}) \
                    is less than the requested height ({height})"
                );

                Ok(continuation(height))
            }
        }
    }

    #[instrument(
        skip_all,
        fields(
            %height,
            already_seen_events_count = already_seen_events.as_ref().map(|a| a.len()),
            refetching = already_seen_events.is_some(),
        )
    )]
    async fn fetch_block(
        &self,
        height: Height,
        already_seen_events: Option<BTreeSet<H256>>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        info!(%height, "fetching events in block");

        // list of MakeChainEvent ops that will be queued in a conc
        let mut make_chain_event_ops: Vec<Op<VoyagerMessage>> = vec![];

        let mut handle_event = |event: gno_rpc::types::Event, tx_hash| -> RpcResult<()> {
            trace!(?event, "observed event");

            let gno_rpc::types::Event::TmEvent(event) = event else {
                return Ok(());
            };

            if event.pkg_path != self.ibc_core_realm {
                return Ok(());
            };

            let Some(event) =
                IbcEvent::from_gno_event(event.clone()).with_message(format!("{event:?}"))?
            else {
                return Ok(());
            };

            let make_chain_event = || {
                // if event.is_trivial() && !self.index_trivial_events {
                //     debug!("not indexing trivial event");
                //     None
                // } else {
                // let event = match event.event {
                //     IbcEvent::BatchSend {
                //     } => {
                //         debug!(%packet_hash, %batch_hash, %channel_id, "found batch send event");
                //         if seen_batches.insert((channel_id, batch_hash)) {
                //             info!(%batch_hash, %channel_id, "found batch send event");
                //             event.clone()
                //         } else {
                //             return None;
                //         }
                //     }
                //     _ => event.clone(),
                // };
                Some(call(PluginMessage::new(
                    self.plugin_name(),
                    ModuleCall::from(MakeChainEvent {
                        height,
                        tx_hash: Some(tx_hash),
                        event,
                    }),
                )))
                // }
            };

            if let Some(e) = make_chain_event() {
                make_chain_event_ops.push(e);
            }

            Ok(())
        };

        let block_response = self
            .gno_client
            .block(
                (height.height() as i64)
                    .try_into()
                    .map_err(RpcError::fatal("invalid gno height"))?,
            )
            .await
            .map_err(RpcError::retryable(format_args!(
                "error fetching block at height {height}"
            )))?;

        for tx in block_response.block.data.txs.into_iter().flatten() {
            let tx_hash: H256 = Sha256::digest(tx).into();

            info!("fetching events in tx {tx_hash}");

            let tx_response =
                self.gno_client
                    .tx(tx_hash.into_encoding())
                    .await
                    .map_err(RpcError::retryable(format_args!(
                        "error fetching tx {tx_hash}"
                    )))?;

            for event in tx_response
                .tx_result
                .response_base
                .events
                .into_iter()
                .flatten()
            {
                handle_event(event, tx_response.hash.into_encoding())?;
            }
        }

        Ok(conc(make_chain_event_ops))
    }

    #[instrument(level = "info", skip_all, fields(%height, tx_hash = tx_hash.map(|h| h.to_string())))]
    async fn make_chain_event(
        &self,
        voyager_client: &VoyagerClient,
        height: Height,
        tx_hash: Option<H256>,
        event: IbcEvent,
    ) -> RpcResult<Op<VoyagerMessage>> {
        // events at height N are provable at height N+k where k>0
        let provable_height = EventProvableHeight::Min(height.increment());

        debug!(?event, "raw event");

        match event {
            IbcEvent::CreateClient {
                client_id,
                client_type,
            } => {
                let client_info = voyager_client
                    .client_info::<IbcUnion>(self.chain_id.clone(), client_id)
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(self.chain_id.clone(), height.into(), client_id)
                    .await?;

                let event = CreateClient {
                    client_id,
                    client_type: ClientType::new(client_type),
                }
                .into();

                ibc_union_spec::log_event(&event, &self.chain_id);

                Ok(data(ChainEvent::new::<IbcUnion>(
                    self.chain_id.clone(),
                    client_info,
                    client_state_meta.counterparty_chain_id,
                    tx_hash,
                    provable_height,
                    event,
                )))
            }
            IbcEvent::UpdateClient {
                client_id,
                counterparty_height,
            } => {
                let client_info = voyager_client
                    .client_info::<IbcUnion>(self.chain_id.clone(), client_id)
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(self.chain_id.clone(), height.into(), client_id)
                    .await?;

                let event = UpdateClient {
                    client_id,
                    client_type: client_info.client_type.clone(),
                    height: counterparty_height,
                }
                .into();

                ibc_union_spec::log_event(&event, &self.chain_id);

                Ok(data(ChainEvent::new::<IbcUnion>(
                    self.chain_id.clone(),
                    client_info.clone(),
                    client_state_meta.counterparty_chain_id,
                    tx_hash,
                    provable_height,
                    event,
                )))
            }
            IbcEvent::ConnectionOpenInit {
                connection_id,
                client_id,
                counterparty_client_id,
            } => {
                let client_info = voyager_client
                    .client_info::<IbcUnion>(self.chain_id.clone(), client_id)
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(self.chain_id.clone(), height.into(), client_id)
                    .await?;

                let event = ConnectionOpenInit {
                    client_id,
                    connection_id,
                    counterparty_client_id,
                }
                .into();

                ibc_union_spec::log_event(&event, &self.chain_id);

                Ok(data(ChainEvent::new::<IbcUnion>(
                    self.chain_id.clone(),
                    client_info,
                    client_state_meta.counterparty_chain_id,
                    tx_hash,
                    provable_height,
                    event,
                )))
            }
            IbcEvent::ConnectionOpenTry {
                connection_id,
                client_id,
                counterparty_client_id,
                counterparty_connection_id,
            } => {
                let client_info = voyager_client
                    .client_info::<IbcUnion>(self.chain_id.clone(), client_id)
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(self.chain_id.clone(), height.into(), client_id)
                    .await?;

                let event = ConnectionOpenTry {
                    connection_id,
                    counterparty_connection_id,
                    client_id,
                    counterparty_client_id,
                }
                .into();

                ibc_union_spec::log_event(&event, &self.chain_id);

                Ok(data(ChainEvent::new::<IbcUnion>(
                    self.chain_id.clone(),
                    client_info,
                    client_state_meta.counterparty_chain_id,
                    tx_hash,
                    provable_height,
                    event,
                )))
            }
            IbcEvent::ConnectionOpenAck {
                connection_id,
                client_id,
                counterparty_client_id,
                counterparty_connection_id,
            } => {
                let client_info = voyager_client
                    .client_info::<IbcUnion>(self.chain_id.clone(), client_id)
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(self.chain_id.clone(), height.into(), client_id)
                    .await?;

                let event = ConnectionOpenAck {
                    connection_id,
                    counterparty_connection_id,
                    client_id,
                    counterparty_client_id,
                }
                .into();

                ibc_union_spec::log_event(&event, &self.chain_id);

                Ok(data(ChainEvent::new::<IbcUnion>(
                    self.chain_id.clone(),
                    client_info,
                    client_state_meta.counterparty_chain_id,
                    tx_hash,
                    provable_height,
                    event,
                )))
            }
            IbcEvent::ConnectionOpenConfirm {
                connection_id,
                client_id,
                counterparty_client_id,
                counterparty_connection_id,
            } => {
                let client_info = voyager_client
                    .client_info::<IbcUnion>(self.chain_id.clone(), client_id)
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(self.chain_id.clone(), height.into(), client_id)
                    .await?;

                let event = ConnectionOpenConfirm {
                    connection_id,
                    counterparty_connection_id,
                    client_id,
                    counterparty_client_id,
                }
                .into();

                ibc_union_spec::log_event(&event, &self.chain_id);

                Ok(data(ChainEvent::new::<IbcUnion>(
                    self.chain_id.clone(),
                    client_info,
                    client_state_meta.counterparty_chain_id,
                    tx_hash,
                    provable_height,
                    event,
                )))
            }
            IbcEvent::ChannelOpenInit(ChannelEvent {
                port_id,
                channel_id,
                counterparty_port_id,
                counterparty_channel_id: _, // THIS WILL BE ZERO
                connection_id: _,
                connection_client_id,
                connection_counterparty_client_id,
                connection_counterparty_connection_id,
                version,
            }) => {
                let client_info = voyager_client
                    .client_info::<IbcUnion>(self.chain_id.clone(), connection_client_id)
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        self.chain_id.clone(),
                        height.into(),
                        connection_client_id,
                    )
                    .await?;

                let event = ChannelOpenInit {
                    port_id: port_id.to_string().into_bytes().into(),
                    channel_id,
                    counterparty_port_id: counterparty_port_id.into_encoding(),
                    connection: Connection {
                        state: ConnectionState::Open, // we assume open since the channel requires the connection to be open before beginning the handshake
                        client_id: connection_client_id,
                        counterparty_client_id: connection_counterparty_client_id,
                        counterparty_connection_id: Some(connection_counterparty_connection_id),
                    },
                    version,
                }
                .into();

                ibc_union_spec::log_event(&event, &self.chain_id);

                Ok(data(ChainEvent::new::<IbcUnion>(
                    self.chain_id.clone(),
                    client_info,
                    client_state_meta.counterparty_chain_id,
                    tx_hash,
                    provable_height,
                    event,
                )))
            }
            IbcEvent::ChannelOpenTry(ChannelEvent {
                port_id,
                channel_id,
                counterparty_port_id,
                counterparty_channel_id,
                connection_id: _,
                connection_client_id,
                connection_counterparty_client_id,
                connection_counterparty_connection_id,
                version,
            }) => {
                let client_info = voyager_client
                    .client_info::<IbcUnion>(self.chain_id.clone(), connection_client_id)
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        self.chain_id.clone(),
                        height.into(),
                        connection_client_id,
                    )
                    .await?;

                let event = ChannelOpenTry {
                    port_id: port_id.to_string().into_bytes().into(),
                    channel_id,
                    counterparty_port_id: counterparty_port_id.into_encoding(),
                    counterparty_channel_id: counterparty_channel_id.expect("must be set"),
                    connection: Connection {
                        state: ConnectionState::TryOpen,
                        client_id: connection_client_id,
                        counterparty_client_id: connection_counterparty_client_id,
                        counterparty_connection_id: Some(connection_counterparty_connection_id),
                    },
                    version,
                }
                .into();

                ibc_union_spec::log_event(&event, &self.chain_id);

                Ok(data(ChainEvent::new::<IbcUnion>(
                    self.chain_id.clone(),
                    client_info,
                    client_state_meta.counterparty_chain_id,
                    tx_hash,
                    provable_height,
                    event,
                )))
            }
            IbcEvent::ChannelOpenAck(ChannelEvent {
                port_id,
                channel_id,
                counterparty_port_id,
                counterparty_channel_id,
                connection_id: _,
                connection_client_id,
                connection_counterparty_client_id,
                connection_counterparty_connection_id,
                version,
            }) => {
                let client_info = voyager_client
                    .client_info::<IbcUnion>(self.chain_id.clone(), connection_client_id)
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        self.chain_id.clone(),
                        height.into(),
                        connection_client_id,
                    )
                    .await?;

                let event = ChannelOpenAck {
                    port_id: port_id.to_string().into_bytes().into(),
                    channel_id,
                    counterparty_port_id: counterparty_port_id.into_encoding(),
                    counterparty_channel_id: counterparty_channel_id.expect("must be set"),
                    connection: Connection {
                        state: ConnectionState::Open, // we assume open since the channel requires the connection to be open before beginning the handshake
                        client_id: connection_client_id,
                        counterparty_client_id: connection_counterparty_client_id,
                        counterparty_connection_id: Some(connection_counterparty_connection_id),
                    },
                    version,
                }
                .into();

                ibc_union_spec::log_event(&event, &self.chain_id);

                Ok(data(ChainEvent::new::<IbcUnion>(
                    self.chain_id.clone(),
                    client_info,
                    client_state_meta.counterparty_chain_id,
                    tx_hash,
                    provable_height,
                    event,
                )))
            }

            IbcEvent::ChannelOpenConfirm(ChannelEvent {
                port_id,
                channel_id,
                counterparty_port_id,
                counterparty_channel_id,
                connection_id: _,
                connection_client_id,
                connection_counterparty_client_id,
                connection_counterparty_connection_id,
                version,
            }) => {
                let client_info = voyager_client
                    .client_info::<IbcUnion>(self.chain_id.clone(), connection_client_id)
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        self.chain_id.clone(),
                        height.into(),
                        connection_client_id,
                    )
                    .await?;

                let event = ChannelOpenConfirm {
                    port_id: port_id.to_string().into_bytes().into(),
                    channel_id,
                    counterparty_port_id: counterparty_port_id.into_encoding(),
                    counterparty_channel_id: counterparty_channel_id.expect("must be set"),
                    connection: Connection {
                        state: ConnectionState::Open, // we assume open since the channel requires the connection to be open before beginning the handshake
                        client_id: connection_client_id,
                        counterparty_client_id: connection_counterparty_client_id,
                        counterparty_connection_id: Some(connection_counterparty_connection_id),
                    },
                    version,
                }
                .into();

                ibc_union_spec::log_event(&event, &self.chain_id);

                Ok(data(ChainEvent::new::<IbcUnion>(
                    self.chain_id.clone(),
                    client_info,
                    client_state_meta.counterparty_chain_id,
                    tx_hash,
                    provable_height,
                    event,
                )))
            }
            IbcEvent::PacketSend {
                packet_hash: _,
                packet_data,
                source_channel_id,
                source_channel_version: _,
                source_connection_id: _,
                source_connection_client_id: _,
                destination_channel_id,
                destination_connection_id: _,
                destination_connection_client_id: _,
                timeout_timestamp,
            } => {
                let packet = Packet {
                    source_channel_id,
                    destination_channel_id,
                    data: packet_data,
                    timeout_height: MustBeZero,
                    timeout_timestamp,
                };

                let state = voyager_client
                    .maybe_query_ibc_state(
                        self.chain_id.clone(),
                        QueryHeight::Latest,
                        // BatchPacketsPath::from_packet(&packet),
                        BatchPacketsPath::from_packet(&packet),
                    )
                    .await?;

                if state.state.is_none() {
                    info!("packet already acknowledged");
                    return Ok(noop());
                }

                let source_channel = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        QueryHeight::Specific(height),
                        ibc_union_spec::path::ChannelPath {
                            channel_id: packet.source_channel_id,
                        },
                    )
                    .await?;

                let source_connection = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        QueryHeight::Specific(height),
                        ibc_union_spec::path::ConnectionPath {
                            connection_id: source_channel.connection_id,
                        },
                    )
                    .await?;

                let client_info = voyager_client
                    .client_info::<IbcUnion>(self.chain_id.clone(), source_connection.client_id)
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        self.chain_id.clone(),
                        height.into(),
                        source_connection.client_id,
                    )
                    .await?;

                let event = PacketSend {
                    packet_data: packet.data,
                    packet: PacketMetadata {
                        source_channel: ChannelMetadata {
                            channel_id: packet.source_channel_id,
                            version: source_channel.version.clone(),
                            connection: ConnectionMetadata {
                                client_id: source_connection.client_id,
                                connection_id: source_channel.connection_id,
                            },
                        },
                        destination_channel: CounterpartyChannelMetadata {
                            channel_id: packet.destination_channel_id,
                            connection: ConnectionMetadata {
                                client_id: source_connection.counterparty_client_id,
                                connection_id: source_connection
                                    .counterparty_connection_id
                                    .expect("must be set"),
                            },
                        },
                        timeout_timestamp: packet.timeout_timestamp,
                    },
                }
                .into();

                ibc_union_spec::log_event(&event, &self.chain_id);

                Ok(data(ChainEvent::new::<IbcUnion>(
                    self.chain_id.clone(),
                    client_info,
                    client_state_meta.counterparty_chain_id,
                    tx_hash,
                    provable_height,
                    event,
                )))
            }
            // IbcEvent::BatchSend {
            //     channel_id,
            //     packet_hash: _,
            //     batch_hash,
            // } => {
            //     let source_channel = voyager_client
            //         .query_ibc_state(
            //             self.chain_id.clone(),
            //             QueryHeight::Specific(height),
            //             ibc_union_spec::path::ChannelPath { channel_id },
            //         )
            //         .await?;

            //     let source_connection = voyager_client
            //         .query_ibc_state(
            //             self.chain_id.clone(),
            //             QueryHeight::Specific(height),
            //             ibc_union_spec::path::ConnectionPath {
            //                 connection_id: source_channel.connection_id,
            //             },
            //         )
            //         .await?;

            //     let client_info = voyager_client
            //         .client_info::<IbcUnion>(self.chain_id.clone(), source_connection.client_id)
            //         .await?;

            //     let client_state_meta = voyager_client
            //         .client_state_meta::<IbcUnion>(
            //             self.chain_id.clone(),
            //             height.into(),
            //             source_connection.client_id,
            //         )
            //         .await?;

            //     let event = BatchSend {
            //         batch_hash,
            //         source_channel: ChannelMetadata {
            //             channel_id,
            //             version: source_channel.version.clone(),
            //             connection: ConnectionMetadata {
            //                 client_id: source_connection.client_id,
            //                 connection_id: source_channel.connection_id,
            //             },
            //         },
            //         destination_channel: CounterpartyChannelMetadata {
            //             channel_id: source_channel
            //                 .counterparty_channel_id
            //                 .expect("channel is open"),
            //             connection: ConnectionMetadata {
            //                 client_id: source_connection.counterparty_client_id,
            //                 connection_id: source_connection.counterparty_connection_id.unwrap(),
            //             },
            //         },
            //     }
            //     .into();

            //     ibc_union_spec::log_event(&event, &self.chain_id);

            //     Ok(data(ChainEvent::new::<IbcUnion>(
            //         self.chain_id.clone(),
            //         client_info,
            //         client_state_meta.counterparty_chain_id,
            //         tx_hash,
            //         provable_height,
            //         event,
            //     )))
            // }
            // IbcEvent::PacketAck {
            //     acknowledgement,
            //     channel_id,
            //     packet_hash,
            // } => {
            //     let packet = voyager_client
            //         .query(
            //             self.chain_id.clone(),
            //             PacketByHash {
            //                 channel_id,
            //                 packet_hash,
            //             },
            //         )
            //         .await?
            //         .packet;

            //     let source_channel = voyager_client
            //         .query_ibc_state(
            //             self.chain_id.clone(),
            //             QueryHeight::Specific(height),
            //             ibc_union_spec::path::ChannelPath {
            //                 channel_id: packet.source_channel_id,
            //             },
            //         )
            //         .await?;

            //     let source_connection = voyager_client
            //         .query_ibc_state(
            //             self.chain_id.clone(),
            //             QueryHeight::Specific(height),
            //             ibc_union_spec::path::ConnectionPath {
            //                 connection_id: source_channel.connection_id,
            //             },
            //         )
            //         .await?;

            //     let client_info = voyager_client
            //         .client_info::<IbcUnion>(self.chain_id.clone(), source_connection.client_id)
            //         .await?;

            //     let client_state_meta = voyager_client
            //         .client_state_meta::<IbcUnion>(
            //             self.chain_id.clone(),
            //             height.into(),
            //             source_connection.client_id,
            //         )
            //         .await?;

            //     let event = PacketAck {
            //         packet_data: packet.data,
            //         packet: PacketMetadata {
            //             source_channel: ChannelMetadata {
            //                 channel_id: packet.source_channel_id,
            //                 version: source_channel.version.clone(),
            //                 connection: ConnectionMetadata {
            //                     client_id: source_connection.client_id,
            //                     connection_id: source_channel.connection_id,
            //                 },
            //             },
            //             destination_channel: CounterpartyChannelMetadata {
            //                 channel_id: packet.destination_channel_id,
            //                 connection: ConnectionMetadata {
            //                     client_id: source_connection.counterparty_client_id,
            //                     connection_id: source_connection
            //                         .counterparty_connection_id
            //                         .unwrap(),
            //                 },
            //             },
            //             timeout_timestamp: packet.timeout_timestamp,
            //         },
            //         acknowledgement: acknowledgement.into_encoding(),
            //     }
            //     .into();

            //     ibc_union_spec::log_event(&event, &self.chain_id);

            //     Ok(data(ChainEvent::new::<IbcUnion>(
            //         self.chain_id.clone(),
            //         client_info,
            //         client_state_meta.counterparty_chain_id,
            //         tx_hash,
            //         provable_height,
            //         event,
            //     )))
            // }
            IbcEvent::PacketRecv {
                packet_hash,
                packet_data: _,
                source_channel_id: _,
                source_connection_id: _,
                source_connection_client_id: _,
                destination_channel_id,
                destination_channel_version: _,
                destination_connection_id: _,
                destination_connection_client_id: _,
                timeout_timestamp: _,
                maker_msg,
            } => {
                let destination_channel = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        QueryHeight::Specific(height),
                        ibc_union_spec::path::ChannelPath {
                            channel_id: destination_channel_id,
                        },
                    )
                    .await?;

                let destination_connection = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        QueryHeight::Specific(height),
                        ibc_union_spec::path::ConnectionPath {
                            connection_id: destination_channel.connection_id,
                        },
                    )
                    .await?;

                let client_info = voyager_client
                    .client_info::<IbcUnion>(
                        self.chain_id.clone(),
                        destination_connection.client_id,
                    )
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        self.chain_id.clone(),
                        height.into(),
                        destination_connection.client_id,
                    )
                    .await?;

                let packet = voyager_client
                    .query(
                        client_state_meta.counterparty_chain_id.clone(),
                        PacketByHash {
                            channel_id: destination_channel
                                .counterparty_channel_id
                                .expect("must be set"),
                            packet_hash,
                        },
                    )
                    .await?
                    .packet;

                let event = PacketRecv {
                    packet_data: packet.data,
                    packet: PacketMetadata {
                        source_channel: CounterpartyChannelMetadata {
                            channel_id: packet.source_channel_id,
                            connection: ConnectionMetadata {
                                client_id: destination_connection.counterparty_client_id,
                                connection_id: destination_connection
                                    .counterparty_connection_id
                                    .expect("must be set"),
                            },
                        },
                        destination_channel: ChannelMetadata {
                            channel_id: packet.destination_channel_id,
                            version: destination_channel.version.clone(),
                            connection: ConnectionMetadata {
                                client_id: destination_connection.client_id,
                                connection_id: destination_channel.connection_id,
                            },
                        },
                        timeout_timestamp: packet.timeout_timestamp,
                    },
                    maker_msg: maker_msg.into_encoding(),
                }
                .into();

                ibc_union_spec::log_event(&event, &self.chain_id);

                Ok(data(ChainEvent::new::<IbcUnion>(
                    self.chain_id.clone(),
                    client_info,
                    client_state_meta.counterparty_chain_id,
                    tx_hash,
                    provable_height,
                    event,
                )))
            }
            IbcEvent::WriteAck {
                packet_hash,
                packet_data: _,
                source_channel_id: _,
                source_connection_id: _,
                source_connection_client_id: _,
                destination_channel_id,
                destination_channel_version: _,
                destination_connection_id: _,
                destination_connection_client_id: _,
                timeout_timestamp: _,
                acknowledgement,
            } => {
                let destination_channel = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        QueryHeight::Specific(height),
                        ibc_union_spec::path::ChannelPath {
                            channel_id: destination_channel_id,
                        },
                    )
                    .await?;

                let destination_connection = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        QueryHeight::Specific(height),
                        ibc_union_spec::path::ConnectionPath {
                            connection_id: destination_channel.connection_id,
                        },
                    )
                    .await?;

                let client_info = voyager_client
                    .client_info::<IbcUnion>(
                        self.chain_id.clone(),
                        destination_connection.client_id,
                    )
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        self.chain_id.clone(),
                        height.into(),
                        destination_connection.client_id,
                    )
                    .await?;

                let packet = voyager_client
                    .query(
                        client_state_meta.counterparty_chain_id.clone(),
                        PacketByHash {
                            channel_id: destination_channel
                                .counterparty_channel_id
                                .expect("must be set"),
                            packet_hash,
                        },
                    )
                    .await?
                    .packet;

                let event = WriteAck {
                    packet_data: packet.data,
                    packet: PacketMetadata {
                        source_channel: CounterpartyChannelMetadata {
                            channel_id: packet.source_channel_id,
                            connection: ConnectionMetadata {
                                client_id: destination_connection.counterparty_client_id,
                                connection_id: destination_connection
                                    .counterparty_connection_id
                                    .expect("must be set"),
                            },
                        },
                        destination_channel: ChannelMetadata {
                            channel_id: packet.destination_channel_id,
                            version: destination_channel.version.clone(),
                            connection: ConnectionMetadata {
                                client_id: destination_connection.client_id,
                                connection_id: destination_channel.connection_id,
                            },
                        },
                        timeout_timestamp: packet.timeout_timestamp,
                    },
                    acknowledgement: acknowledgement.into_encoding(),
                }
                .into();

                ibc_union_spec::log_event(&event, &self.chain_id);

                Ok(data(ChainEvent::new::<IbcUnion>(
                    self.chain_id.clone(),
                    client_info,
                    client_state_meta.counterparty_chain_id,
                    tx_hash,
                    provable_height,
                    event,
                )))
            }
            _ => {
                error!("unimplemented: {event:?}");
                Ok(noop())
            }
        }
    }
}

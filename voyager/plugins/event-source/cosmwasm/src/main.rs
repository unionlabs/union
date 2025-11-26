// #![warn(clippy::unwrap_used)]

use core::slice;
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, VecDeque, btree_map::Entry},
    num::{NonZeroU8, NonZeroU32, ParseIntError},
};

use cosmos_sdk_event::CosmosSdkEvent;
use ibc_union_spec::{
    IbcUnion, MustBeZero, Packet,
    event::{
        BatchSend, ChannelMetadata, ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit,
        ChannelOpenTry, ConnectionMetadata, ConnectionOpenAck, ConnectionOpenConfirm,
        ConnectionOpenInit, ConnectionOpenTry, CounterpartyChannelMetadata, CreateClient,
        PacketAck, PacketMetadata, PacketRecv, PacketSend, UpdateClient, WriteAck,
    },
    path::ChannelPath,
    query::PacketByHash,
};
use jsonrpsee::{
    Extensions,
    core::{RpcResult, async_trait},
    types::ErrorObject,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, error, info, info_span, instrument, trace, warn};
use unionlabs::{
    ErrorReporter,
    ibc::core::client::height::Height,
    never::Never,
    primitives::{Bech32, H256},
};
use voyager_sdk::{
    ExtensionsExt, VoyagerClient, anyhow,
    hook::simple_take_filter,
    into_value,
    message::{
        PluginMessage, VoyagerMessage,
        call::{Call, WaitForHeight},
        data::{ChainEvent, Data, EventProvableHeight},
    },
    plugin::Plugin,
    primitives::{ChainId, ClientType, QueryHeight},
    rpc::{FATAL_JSONRPC_ERROR_CODE, PluginServer, rpc_error, types::PluginInfo},
    vm::{Op, call, conc, data, noop, pass::PassResult, seq},
};

use crate::{
    call::{FetchBlock, FetchBlocks, MakeChainEvent, ModuleCall},
    ibc_events::IbcEvent,
};

pub mod ibc_events;

pub mod call;

const PER_PAGE_LIMIT: NonZeroU8 = NonZeroU8::new(100).unwrap();

#[tokio::main]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,
    // TODO: Remove chain revision
    pub chain_revision: u64,

    pub cometbft_client: cometbft_rpc::Client,

    pub chunk_block_fetch_size: u64,
    pub refetch_delay: u64,

    pub index_trivial_events: bool,

    pub ibc_host_contract_address: Option<Bech32<H256>>,
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
    pub ibc_host_contract_address: Option<Bech32<H256>>,
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
        let tm_client = cometbft_rpc::Client::new(config.rpc_url).await?;

        let chain_id = tm_client.status().await?.node_info.network;

        let chain_revision = chain_id
            .split('-')
            .next_back()
            .ok_or_else(|| ChainIdParseError {
                found: chain_id.clone(),
                source: None,
            })?
            .parse()
            .map_err(|err| ChainIdParseError {
                found: chain_id.clone(),
                source: Some(err),
            })?;

        Ok(Self {
            cometbft_client: tm_client,
            chain_id: ChainId::new(chain_id),
            chain_revision,
            chunk_block_fetch_size: config.chunk_block_fetch_size,
            refetch_delay: config.refetch_delay,
            index_trivial_events: config.index_trivial_events,
            ibc_host_contract_address: config.ibc_host_contract_address,
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

    #[must_use]
    pub fn make_height(&self, height: u64) -> Height {
        Height::new_with_revision(self.chain_revision, height)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("unable to parse chain id: expected format `<chain>-<revision-number>`, found `{found}`")]
pub struct ChainIdParseError {
    found: String,
    #[source]
    source: Option<ParseIntError>,
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
                return Err(ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("height {height} cannot be greater than the until height {until}"),
                    None::<()>,
                ));
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

        if !height.revision_matches(&latest_height) {
            return Err(ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                format!(
                    "revision number mismatch: fetching blocks from height \
                    {height}, but the latest height is {latest_height}"
                ),
                None::<()>,
            ));
        }

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

        enum EventState {
            SeenPreviously,
            SeenNow,
        }

        let mut already_seen_events = already_seen_events.map(|a| {
            a.into_iter()
                .map(|h| (h, EventState::SeenPreviously))
                .collect::<BTreeMap<H256, EventState>>()
        });

        // list of MakeChainEvent ops that will be queued in a conc
        let mut make_chain_event_ops = vec![];

        // event hashes found while fetching this block
        let mut found_events = BTreeSet::new();

        let mut page = const { NonZeroU32::new(1).unwrap() };

        let mut total_count = 0;

        let mut seen_batches = BTreeSet::new();

        loop {
            info!(%height, %page, "fetching page {page}");

            let response = self
                .cometbft_client
                .tx_search(
                    format!("tx.height={}", height.height()),
                    false,
                    page,
                    PER_PAGE_LIMIT,
                    cometbft_rpc::rpc_types::Order::Desc,
                )
                .await
                .map_err(rpc_error(
                    format_args!("error fetching transactions at height {height}"),
                    Some(json!({ "height": height })),
                ))?;

            total_count += response.txs.len();

            for tx_response in response.txs {
                let _span = info_span!("tx_result.events", tx_hash = %tx_response.hash).entered();
                for event in tx_response.tx_result.events {
                    trace!(%event.ty, "observed event");

                    let event = match CosmosSdkEvent::<IbcEvent>::new(event) {
                        Ok(event) => event,
                        Err(cosmos_sdk_event::Error::Deserialize(error)) => {
                            trace!("unable to parse event: {error}");
                            continue;
                        }
                        Err(err) => {
                            error!("error parsing event: {}", ErrorReporter(err));
                            continue;
                        }
                    };

                    match (&event.contract_address, &self.ibc_host_contract_address) {
                        (None, _) => {}
                        (Some(addr), None) => {
                            debug!(
                                "found ibc-union event for contract {addr}, but no contract address is configured",
                            );
                            continue;
                        }
                        (Some(event_addr), Some(configured_addr)) => {
                            if event_addr == configured_addr {
                            } else {
                                debug!(
                                    "found ibc-union event for contract {event_addr}, but the configured contract address is {configured_addr}",
                                );
                                continue;
                            }
                        }
                    }

                    let mut make_chain_event = || {
                        if event.event.is_trivial() && !self.index_trivial_events {
                            debug!("not indexing trivial event");
                            None
                        } else {
                            let event = match event.event {
                                IbcEvent::WasmBatchSend {
                                    channel_id,
                                    batch_hash,
                                    packet_hash,
                                } => {
                                    debug!(%packet_hash, %batch_hash, %channel_id, "found batch send event");
                                    if seen_batches.insert((channel_id, batch_hash)) {
                                        info!(%batch_hash, %channel_id, "found batch send event");
                                        event.clone()
                                    } else {
                                        return None;
                                    }
                                }
                                _ => event.clone(),
                            };
                            Some(call(PluginMessage::new(
                                self.plugin_name(),
                                ModuleCall::from(MakeChainEvent {
                                    height,
                                    tx_hash: tx_response.hash.into_encoding(),
                                    event: event.event,
                                }),
                            )))
                        }
                    };

                    if let Some(ref mut already_seen_events) = already_seen_events {
                        match already_seen_events.entry(event.event.hash()) {
                            Entry::Vacant(vacant_entry) => {
                                info!("found previously missed event");
                                vacant_entry.insert(EventState::SeenNow);
                                make_chain_event_ops.push(make_chain_event());
                            }
                            Entry::Occupied(mut occupied_entry) => match occupied_entry.get() {
                                EventState::SeenPreviously => {
                                    info!("found previously seen event");
                                    occupied_entry.insert(EventState::SeenNow);
                                }
                                EventState::SeenNow => {
                                    warn!(
                                        "found duplicate event, likely due to a load-balanced rpc with poor nodes. additional data may have been missed!"
                                    );
                                }
                            },
                        };
                    } else {
                        found_events.insert(event.event.hash());
                        make_chain_event_ops.push(make_chain_event());
                    }
                }
            }

            if total_count >= (response.total_count as usize) {
                break;
            } else {
                page = page
                    .checked_add(1)
                    .expect("how many events does this block have???");
            }
        }

        Ok(conc(make_chain_event_ops.into_iter().flatten().chain(
            already_seen_events.is_none().then(|| {
                seq([
                    call(WaitForHeight {
                        chain_id: self.chain_id.clone(),
                        height: Height::new(height.height() + self.refetch_delay),
                        finalized: true,
                    }),
                    call(PluginMessage::new(
                        self.plugin_name(),
                        ModuleCall::from(FetchBlock {
                            height,
                            already_seen_events: Some(found_events),
                        }),
                    )),
                ])
            }),
        )))
    }

    #[instrument(level = "info", skip_all, fields(%height, %tx_hash))]
    async fn make_chain_event(
        &self,
        voyager_client: &VoyagerClient,
        height: Height,
        tx_hash: H256,
        event: IbcEvent,
    ) -> RpcResult<Op<VoyagerMessage>> {
        // events at height N are provable at height N+k where k>0
        let provable_height = EventProvableHeight::Min(height.increment());

        debug!(?event, "raw event");

        match event {
            IbcEvent::WasmCreateClient {
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
            IbcEvent::WasmUpdateClient {
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
            IbcEvent::WasmConnectionOpenInit {
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
            IbcEvent::WasmConnectionOpenTry {
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
            IbcEvent::WasmConnectionOpenAck {
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
            IbcEvent::WasmConnectionOpenConfirm {
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
            IbcEvent::WasmChannelOpenInit {
                port_id,
                channel_id,
                counterparty_port_id,
                connection_id,
                version,
            } => {
                let connection = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        QueryHeight::Specific(height),
                        ibc_union_spec::path::ConnectionPath { connection_id },
                    )
                    .await?;

                let client_info = voyager_client
                    .client_info::<IbcUnion>(self.chain_id.clone(), connection.client_id)
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        self.chain_id.clone(),
                        height.into(),
                        connection.client_id,
                    )
                    .await?;

                let event = ChannelOpenInit {
                    port_id: port_id.to_string().into_bytes().into(),
                    channel_id,
                    counterparty_port_id: counterparty_port_id.into_encoding(),
                    connection,
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
            IbcEvent::WasmChannelOpenTry {
                port_id,
                channel_id,
                counterparty_port_id,
                counterparty_channel_id,
                connection_id,
                counterparty_version,
            } => {
                let connection = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        QueryHeight::Specific(height),
                        ibc_union_spec::path::ConnectionPath { connection_id },
                    )
                    .await?;

                let client_info = voyager_client
                    .client_info::<IbcUnion>(self.chain_id.clone(), connection.client_id)
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        self.chain_id.clone(),
                        height.into(),
                        connection.client_id,
                    )
                    .await?;

                let event = ChannelOpenTry {
                    port_id: port_id.to_string().into_bytes().into(),
                    channel_id,
                    counterparty_port_id: counterparty_port_id.into_encoding(),
                    counterparty_channel_id,
                    connection,
                    version: counterparty_version,
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
            IbcEvent::WasmChannelOpenAck {
                port_id,
                channel_id,
                counterparty_port_id,
                counterparty_channel_id,
                connection_id,
            } => {
                let connection = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        QueryHeight::Specific(height),
                        ibc_union_spec::path::ConnectionPath { connection_id },
                    )
                    .await?;

                let client_info = voyager_client
                    .client_info::<IbcUnion>(self.chain_id.clone(), connection.client_id)
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        self.chain_id.clone(),
                        height.into(),
                        connection.client_id,
                    )
                    .await?;

                let channel = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        QueryHeight::Specific(height),
                        ChannelPath { channel_id },
                    )
                    .await?;

                let event = ChannelOpenAck {
                    port_id: port_id.to_string().into_bytes().into(),
                    channel_id,
                    counterparty_port_id: counterparty_port_id.into_encoding(),
                    counterparty_channel_id,
                    connection,
                    version: channel.version,
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

            IbcEvent::WasmChannelOpenConfirm {
                port_id,
                channel_id,
                counterparty_port_id,
                counterparty_channel_id,
                connection_id,
            } => {
                let channel = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        QueryHeight::Specific(height),
                        ibc_union_spec::path::ChannelPath { channel_id },
                    )
                    .await?;

                let connection = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        QueryHeight::Specific(height),
                        ibc_union_spec::path::ConnectionPath { connection_id },
                    )
                    .await?;

                let client_info = voyager_client
                    .client_info::<IbcUnion>(self.chain_id.clone(), connection.client_id)
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        self.chain_id.clone(),
                        height.into(),
                        connection.client_id,
                    )
                    .await?;

                let event = ChannelOpenConfirm {
                    port_id: port_id.to_string().into_bytes().into(),
                    channel_id,
                    counterparty_port_id: counterparty_port_id.into_encoding(),
                    counterparty_channel_id,
                    connection,
                    version: channel.version,
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
            IbcEvent::WasmPacketSend {
                packet_source_channel_id,
                packet_destination_channel_id,
                packet_data,
                packet_timeout_height: _,
                packet_timeout_timestamp,
                channel_id: _,
                packet_hash: _,
            } => {
                let packet = Packet {
                    source_channel_id: packet_source_channel_id,
                    destination_channel_id: packet_destination_channel_id,
                    data: packet_data,
                    timeout_height: MustBeZero,
                    timeout_timestamp: packet_timeout_timestamp,
                };

                let state = voyager_client
                    .maybe_query_ibc_state(
                        self.chain_id.clone(),
                        QueryHeight::Latest,
                        ibc_union_spec::path::BatchPacketsPath::from_packets(slice::from_ref(
                            &packet,
                        )),
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
                                    .unwrap(),
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
            IbcEvent::WasmBatchSend {
                channel_id,
                packet_hash: _,
                batch_hash,
            } => {
                let source_channel = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        QueryHeight::Specific(height),
                        ibc_union_spec::path::ChannelPath { channel_id },
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

                let event = BatchSend {
                    batch_hash,
                    source_channel: ChannelMetadata {
                        channel_id,
                        version: source_channel.version.clone(),
                        connection: ConnectionMetadata {
                            client_id: source_connection.client_id,
                            connection_id: source_channel.connection_id,
                        },
                    },
                    destination_channel: CounterpartyChannelMetadata {
                        channel_id: source_channel
                            .counterparty_channel_id
                            .expect("channel is open"),
                        connection: ConnectionMetadata {
                            client_id: source_connection.counterparty_client_id,
                            connection_id: source_connection.counterparty_connection_id.unwrap(),
                        },
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
            IbcEvent::WasmPacketAck {
                acknowledgement,
                channel_id,
                packet_hash,
            } => {
                let packet = voyager_client
                    .query(
                        self.chain_id.clone(),
                        PacketByHash {
                            channel_id,
                            packet_hash,
                        },
                    )
                    .await?
                    .packet;

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

                let event = PacketAck {
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
                                    .unwrap(),
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
            IbcEvent::WasmPacketRecv {
                maker: _,
                maker_msg,
                channel_id,
                packet_hash,
            } => {
                let destination_channel = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        QueryHeight::Specific(height),
                        ibc_union_spec::path::ChannelPath { channel_id },
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
                            channel_id: destination_channel.counterparty_channel_id.unwrap(),
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
                                    .unwrap(),
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
            IbcEvent::WasmWriteAck {
                acknowledgement,
                channel_id,
                packet_hash,
            } => {
                let destination_channel = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        QueryHeight::Specific(height),
                        ibc_union_spec::path::ChannelPath { channel_id },
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
                            channel_id: destination_channel.counterparty_channel_id.unwrap(),
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
                                    .unwrap(),
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
        }
    }
}

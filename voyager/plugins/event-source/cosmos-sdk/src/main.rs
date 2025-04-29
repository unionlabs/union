// #![warn(clippy::unwrap_used)]

use std::{
    cmp::Ordering,
    collections::{btree_map::Entry, BTreeMap, BTreeSet, VecDeque},
    error::Error,
    fmt::{Debug, Display},
    num::{NonZeroU32, NonZeroU8, ParseIntError},
    sync::Arc,
};

use cosmos_sdk_event::CosmosSdkEvent;
use dashmap::DashMap;
use ibc_classic_spec::IbcClassic;
use ibc_union_spec::{path::ChannelPath, query::PacketByHash, IbcUnion, Packet};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::{ErrorObject, ErrorObjectOwned},
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, error, info, info_span, instrument, trace, warn};
use unionlabs::{
    bech32::Bech32,
    ibc::core::{
        channel::{self},
        client::height::Height,
    },
    id::{ChannelId, ConnectionId, PortId},
    never::Never,
    option_unwrap,
    primitives::H256,
    ErrorReporter,
};
use voyager_message::{
    call::{Call, WaitForHeight},
    data::{ChainEvent, Data},
    filter::simple_take_filter,
    into_value,
    module::{PluginInfo, PluginServer},
    primitives::{ChainId, ClientInfo, ClientType, IbcSpec, QueryHeight},
    ExtensionsExt, Plugin, PluginMessage, VoyagerClient, VoyagerMessage, FATAL_JSONRPC_ERROR_CODE,
};
use voyager_vm::{call, conc, data, noop, pass::PassResult, seq, BoxDynError, Op};
use wasm_client_type::WasmClientType;

use crate::{
    call::{FetchBlock, FetchBlocks, MakeChainEvent, ModuleCall},
    ibc_events::IbcEvent,
};

pub mod ibc_events;

pub mod call;

const PER_PAGE_LIMIT: NonZeroU8 = option_unwrap!(NonZeroU8::new(100));

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,
    pub chain_revision: u64,

    pub cometbft_client: cometbft_rpc::Client,

    pub chunk_block_fetch_size: u64,
    pub refetch_delay: u64,

    pub checksum_cache: Arc<DashMap<H256, WasmClientType>>,

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

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
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
            checksum_cache: Arc::new(DashMap::default()),
            ibc_host_contract_address: config.ibc_host_contract_address,
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

    #[allow(clippy::too_many_arguments)] // pls
    async fn make_packet_metadata(
        &self,
        event_height: Height,
        self_connection_id: ConnectionId,
        self_port_id: PortId,
        self_channel_id: ChannelId,
        other_port_id: PortId,
        other_channel_id: ChannelId,
        voyager_client: &VoyagerClient,
    ) -> RpcResult<(
        ChainId,
        ClientInfo,
        ibc_classic_spec::ChannelMetadata,
        ibc_classic_spec::ChannelMetadata,
        channel::order::Order,
    )> {
        let self_connection = voyager_client
            .query_ibc_state(
                self.chain_id.clone(),
                event_height,
                ibc_classic_spec::ConnectionPath {
                    connection_id: self_connection_id.clone(),
                },
            )
            .await?;

        let client_info = voyager_client
            .client_info::<IbcClassic>(self.chain_id.clone(), self_connection.client_id.clone())
            .await?;

        let client_state_meta = voyager_client
            .client_state_meta::<IbcClassic>(
                self.chain_id.clone(),
                event_height.into(),
                self_connection.client_id.clone(),
            )
            .await?;

        let this_channel = voyager_client
            .query_ibc_state(
                self.chain_id.clone(),
                event_height,
                ibc_classic_spec::ChannelEndPath {
                    port_id: self_port_id.clone(),
                    channel_id: self_channel_id.clone(),
                },
            )
            .await?;

        let counterparty_latest_height = voyager_client
            .query_latest_height(client_state_meta.counterparty_chain_id.clone(), false)
            .await?;

        let counterparty_channel = voyager_client
            .query_ibc_state(
                client_state_meta.counterparty_chain_id.clone(),
                counterparty_latest_height,
                ibc_classic_spec::ChannelEndPath {
                    port_id: other_port_id.clone(),
                    channel_id: other_channel_id.clone(),
                },
            )
            .await?;

        let source_channel = ibc_classic_spec::ChannelMetadata {
            port_id: self_port_id.clone(),
            channel_id: self_channel_id.clone(),
            version: this_channel.version,
            connection: ibc_classic_spec::ConnectionMetadata {
                client_id: self_connection.client_id,
                connection_id: self_connection_id.clone(),
            },
        };
        let destination_channel = ibc_classic_spec::ChannelMetadata {
            port_id: other_port_id.clone(),
            channel_id: other_channel_id.clone(),
            version: counterparty_channel.version,
            connection: ibc_classic_spec::ConnectionMetadata {
                client_id: self_connection.counterparty.client_id,
                connection_id: self_connection
                    .counterparty
                    .connection_id
                    .expect("counterparty connection id should be set"),
            },
        };

        Ok((
            client_state_meta.counterparty_chain_id,
            client_info,
            source_channel,
            destination_channel,
            this_channel.ordering,
        ))
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
                    Op::Call(Call::FetchBlocks(fetch)) if fetch.chain_id == self.chain_id => {
                        call(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(FetchBlocks {
                                height: fetch.start_height,
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
            ModuleCall::FetchBlocks(FetchBlocks { height }) => {
                self.fetch_blocks(e.try_get::<VoyagerClient>()?, height)
                    .await
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
                self.make_chain_event(e.try_get::<VoyagerClient>()?, height, tx_hash, event)
                    .await
            }
        }
    }
}

// NOTE: For both of the below functions, `message` as a field will override any actual message put in (i.e. `error!("foo", message = "bar")` will print as "bar", not "foo" with an extra field `message = "bar"`.

fn rpc_error<E: Error>(
    message: impl Display,
    data: Option<Value>,
) -> impl FnOnce(E) -> ErrorObjectOwned {
    move |e| {
        let message = format!("{message}: {}", ErrorReporter(e));
        error!(%message, data = %data.as_ref().unwrap_or(&serde_json::Value::Null));
        ErrorObject::owned(-1, message, data)
    }
}

impl Module {
    #[instrument(skip_all, fields(%height))]
    async fn fetch_blocks(
        &self,
        voyager_client: &VoyagerClient,
        height: Height,
    ) -> RpcResult<Op<VoyagerMessage>> {
        let latest_height = voyager_client
            .query_latest_height(self.chain_id.clone(), true)
            .await?;

        info!(%latest_height, %height, "fetching blocks");

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

        let continuation = |next_height| {
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

                info!(
                    from_height = height.height(),
                    to_height = next_height,
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
            // // height == latest_height
            //  => {
            //     info!("requested fetch height is latest finalized height ({height})");

            //     Ok(conc([
            //         call(PluginMessage::new(
            //             self.plugin_name(),
            //             ModuleCall::from(FetchBlock {
            //                 already_seen_events: None,
            //                 height,
            //             }),
            //         )),
            //         continuation(height.increment()),
            //     ]))
            // }
            // height > latest_height
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

        let mut page = const { option_unwrap!(NonZeroU32::new(1)) };

        let mut total_count = 0;

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

                    let event = match CosmosSdkEvent::<IbcEvent>::new(event.clone()) {
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

                    let make_chain_event = || {
                        call(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(MakeChainEvent {
                                height,
                                tx_hash: tx_response.hash.into_encoding(),
                                event: event.event.clone(),
                            }),
                        ))
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
                                    warn!("found duplicate event, likely due to a load-balanced rpc with poor nodes. additional data may have been missed!");
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

        Ok(conc(make_chain_event_ops.into_iter().chain(
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
        // events at height N are provable at height N+k where k<0
        let provable_height = height.increment();

        debug!(?event, "raw event");

        match event {
            IbcEvent::CreateClient { ref client_id, .. }
            | IbcEvent::UpdateClient { ref client_id, .. }
            | IbcEvent::ClientMisbehaviour { ref client_id, .. }
            | IbcEvent::ConnectionOpenInit { ref client_id, .. }
            | IbcEvent::ConnectionOpenTry { ref client_id, .. }
            | IbcEvent::ConnectionOpenAck { ref client_id, .. }
            | IbcEvent::ConnectionOpenConfirm { ref client_id, .. } => {
                let client_info = voyager_client
                    .client_info::<IbcClassic>(self.chain_id.clone(), client_id.clone())
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcClassic>(
                        self.chain_id.clone(),
                        height.into(),
                        client_id.clone(),
                    )
                    .await?;

                Ok(data(ChainEvent {
                    chain_id: self.chain_id.clone(),
                    client_info,
                    counterparty_chain_id: client_state_meta.counterparty_chain_id,
                    tx_hash,
                    provable_height,
                    ibc_spec_id: IbcClassic::ID,
                    event: into_value::<ibc_classic_spec::FullEvent>(match event {
                        IbcEvent::CreateClient {
                            client_id,
                            client_type,
                            consensus_height,
                        } => ibc_classic_spec::CreateClient {
                            client_id,
                            client_type: ClientType::new(client_type),
                            consensus_height,
                        }
                        .into(),
                        IbcEvent::UpdateClient {
                            client_id,
                            client_type,
                            consensus_heights,
                        } => ibc_classic_spec::UpdateClient {
                            client_id,
                            client_type: ClientType::new(client_type),
                            consensus_heights,
                        }
                        .into(),
                        IbcEvent::ConnectionOpenInit {
                            connection_id,
                            client_id,
                            counterparty_client_id,
                        } => {
                            ibc_classic_spec::ConnectionOpenInit {
                                client_id,
                                connection_id,
                                counterparty_client_id,
                            }
                        }
                        .into(),
                        IbcEvent::ConnectionOpenTry {
                            connection_id,
                            client_id,
                            counterparty_client_id,
                            counterparty_connection_id,
                        } => {
                            ibc_classic_spec::ConnectionOpenTry {
                                client_id,
                                connection_id,
                                counterparty_client_id,
                                counterparty_connection_id,
                            }
                        }
                        .into(),
                        IbcEvent::ConnectionOpenAck {
                            connection_id,
                            client_id,
                            counterparty_client_id,
                            counterparty_connection_id,
                        } => {
                            ibc_classic_spec::ConnectionOpenAck {
                                client_id,
                                connection_id,
                                counterparty_client_id,
                                counterparty_connection_id,
                            }
                        }
                        .into(),
                        IbcEvent::ConnectionOpenConfirm {
                            connection_id,
                            client_id,
                            counterparty_client_id,
                            counterparty_connection_id,
                        } => {
                            ibc_classic_spec::ConnectionOpenConfirm {
                                client_id,
                                connection_id,
                                counterparty_client_id,
                                counterparty_connection_id,
                            }
                        }
                        .into(),
                        _ => unreachable!("who needs flow typing"),
                    }),
                }))
            }

            IbcEvent::ChannelOpenInit {
                ref connection_id, ..
            }
            | IbcEvent::ChannelOpenTry {
                ref connection_id, ..
            } => {
                let connection = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        height,
                        ibc_classic_spec::ConnectionPath {
                            connection_id: connection_id.clone(),
                        },
                    )
                    .await?;

                let client_info = voyager_client
                    .client_info::<IbcClassic>(self.chain_id.clone(), connection.client_id.clone())
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcClassic>(
                        self.chain_id.clone(),
                        height.into(),
                        connection.client_id.clone(),
                    )
                    .await?;

                Ok(data(ChainEvent {
                    chain_id: self.chain_id.clone(),
                    client_info,
                    counterparty_chain_id: client_state_meta.counterparty_chain_id,
                    tx_hash,
                    provable_height,
                    ibc_spec_id: IbcClassic::ID,
                    event: into_value::<ibc_classic_spec::FullEvent>(match event {
                        IbcEvent::ChannelOpenInit {
                            port_id,
                            channel_id,
                            counterparty_port_id,
                            version,
                            ..
                        } => {
                            ibc_classic_spec::ChannelOpenInit {
                                port_id,
                                channel_id,
                                counterparty_port_id,
                                connection,
                                version,
                            }
                        }
                        .into(),
                        IbcEvent::ChannelOpenTry {
                            port_id,
                            channel_id,
                            counterparty_port_id,
                            counterparty_channel_id,
                            version,
                            ..
                        } => ibc_classic_spec::ChannelOpenTry {
                            port_id,
                            channel_id,
                            counterparty_port_id,
                            counterparty_channel_id,
                            connection,
                            version,
                        }
                        .into(),
                        _ => unreachable!("who needs flow typing"),
                    }),
                }))
            }

            IbcEvent::ChannelOpenAck {
                ref connection_id,
                ref port_id,
                ref channel_id,
                ..
            }
            | IbcEvent::ChannelOpenConfirm {
                ref connection_id,
                ref port_id,
                ref channel_id,
                ..
            } => {
                let connection = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        height,
                        ibc_classic_spec::ConnectionPath {
                            connection_id: connection_id.clone(),
                        },
                    )
                    .await?;

                let client_info = voyager_client
                    .client_info::<IbcClassic>(self.chain_id.clone(), connection.client_id.clone())
                    .await?;

                let client_state_meta = voyager_client
                    .client_state_meta::<IbcClassic>(
                        self.chain_id.clone(),
                        height.into(),
                        connection.client_id.clone(),
                    )
                    .await?;

                let channel = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        height,
                        ibc_classic_spec::ChannelEndPath {
                            port_id: port_id.to_owned(),
                            channel_id: channel_id.to_owned(),
                        },
                    )
                    .await?;

                Ok(data(ChainEvent {
                    chain_id: self.chain_id.clone(),
                    client_info,
                    counterparty_chain_id: client_state_meta.counterparty_chain_id,
                    tx_hash,
                    provable_height,
                    ibc_spec_id: IbcClassic::ID,
                    event: into_value::<ibc_classic_spec::FullEvent>(match event {
                        IbcEvent::ChannelOpenAck {
                            port_id,
                            channel_id,
                            counterparty_port_id,
                            counterparty_channel_id,
                            connection_id: _,
                        } => {
                            ibc_classic_spec::ChannelOpenAck {
                                port_id,
                                channel_id,
                                counterparty_port_id,
                                counterparty_channel_id,
                                connection,
                                version: channel.version,
                            }
                        }
                        .into(),
                        IbcEvent::ChannelOpenConfirm {
                            port_id,
                            channel_id,
                            counterparty_port_id,
                            counterparty_channel_id,
                            connection_id: _,
                        } => ibc_classic_spec::ChannelOpenConfirm {
                            port_id,
                            channel_id,
                            counterparty_port_id,
                            counterparty_channel_id,
                            connection,
                            version: channel.version,
                        }
                        .into(),
                        _ => unreachable!("who needs flow typing"),
                    }),
                }))
            }
            // packet origin is this chain
            IbcEvent::SendPacket {
                packet_data_hex,
                packet_timeout_height,
                packet_timeout_timestamp,
                packet_sequence,
                packet_src_port,
                packet_src_channel,
                packet_dst_port,
                packet_dst_channel,
                packet_channel_ordering: _,
                connection_id,
            } => {
                let (
                    counterparty_chain_id,
                    client_info,
                    source_channel,
                    destination_channel,
                    channel_ordering,
                ) = self
                    .make_packet_metadata(
                        height,
                        connection_id.to_owned(),
                        packet_src_port.to_owned(),
                        packet_src_channel.to_owned(),
                        packet_dst_port.to_owned(),
                        packet_dst_channel.to_owned(),
                        voyager_client,
                    )
                    .await?;

                Ok(data(ChainEvent {
                    chain_id: self.chain_id.clone(),
                    client_info,
                    counterparty_chain_id,
                    tx_hash,
                    provable_height,
                    ibc_spec_id: IbcClassic::ID,
                    event: into_value::<ibc_classic_spec::FullEvent>(
                        ibc_classic_spec::SendPacket {
                            packet_data: packet_data_hex.into_encoding(),
                            packet: ibc_classic_spec::PacketMetadata {
                                sequence: packet_sequence,
                                source_channel,
                                destination_channel,
                                channel_ordering,
                                timeout_height: packet_timeout_height,
                                timeout_timestamp: packet_timeout_timestamp.as_nanos(),
                            },
                        }
                        .into(),
                    ),
                }))
            }
            IbcEvent::TimeoutPacket {
                packet_timeout_height,
                packet_timeout_timestamp,
                packet_sequence,
                packet_src_port,
                packet_src_channel,
                packet_dst_port,
                packet_dst_channel,
                packet_channel_ordering: _,
                connection_id,
            } => {
                let (
                    counterparty_chain_id,
                    client_info,
                    source_channel,
                    destination_channel,
                    channel_ordering,
                ) = self
                    .make_packet_metadata(
                        height,
                        connection_id.to_owned(),
                        packet_src_port.to_owned(),
                        packet_src_channel.to_owned(),
                        packet_dst_port.to_owned(),
                        packet_dst_channel.to_owned(),
                        voyager_client,
                    )
                    .await?;

                Ok(data(ChainEvent {
                    chain_id: self.chain_id.clone(),
                    client_info,
                    counterparty_chain_id,
                    tx_hash,
                    provable_height,
                    ibc_spec_id: IbcClassic::ID,
                    event: into_value::<ibc_classic_spec::FullEvent>(
                        ibc_classic_spec::TimeoutPacket {
                            packet: ibc_classic_spec::PacketMetadata {
                                sequence: packet_sequence,
                                source_channel,
                                destination_channel,
                                channel_ordering,
                                timeout_height: packet_timeout_height,
                                timeout_timestamp: packet_timeout_timestamp.as_nanos(),
                            },
                        }
                        .into(),
                    ),
                }))
            }
            IbcEvent::AcknowledgePacket {
                packet_timeout_height,
                packet_timeout_timestamp,
                packet_sequence,
                packet_src_port,
                packet_src_channel,
                packet_dst_port,
                packet_dst_channel,
                packet_channel_ordering: _,
                connection_id,
            } => {
                let (
                    counterparty_chain_id,
                    client_info,
                    source_channel,
                    destination_channel,
                    channel_ordering,
                ) = self
                    .make_packet_metadata(
                        height,
                        connection_id.to_owned(),
                        packet_src_port.to_owned(),
                        packet_src_channel.to_owned(),
                        packet_dst_port.to_owned(),
                        packet_dst_channel.to_owned(),
                        voyager_client,
                    )
                    .await?;

                Ok(data(ChainEvent {
                    chain_id: self.chain_id.clone(),
                    client_info,
                    counterparty_chain_id,
                    tx_hash,
                    provable_height,
                    ibc_spec_id: IbcClassic::ID,
                    event: into_value::<ibc_classic_spec::FullEvent>(
                        ibc_classic_spec::AcknowledgePacket {
                            packet: ibc_classic_spec::PacketMetadata {
                                sequence: packet_sequence,
                                source_channel,
                                destination_channel,
                                channel_ordering,
                                timeout_height: packet_timeout_height,
                                timeout_timestamp: packet_timeout_timestamp.as_nanos(),
                            },
                        }
                        .into(),
                    ),
                }))
            }
            // packet origin is the counterparty chain (if i put this comment above this pattern rustfmt explodes)
            IbcEvent::WriteAcknowledgement {
                packet_data_hex,
                packet_timeout_height,
                packet_timeout_timestamp,
                packet_sequence,
                packet_src_port,
                packet_src_channel,
                packet_dst_port,
                packet_dst_channel,
                packet_ack_hex,
                connection_id,
            } => {
                let (
                    counterparty_chain_id,
                    client_info,
                    destination_channel,
                    source_channel,
                    channel_ordering,
                ) = self
                    .make_packet_metadata(
                        height,
                        connection_id.to_owned(),
                        packet_dst_port.to_owned(),
                        packet_dst_channel.to_owned(),
                        packet_src_port.to_owned(),
                        packet_src_channel.to_owned(),
                        voyager_client,
                    )
                    .await?;

                Ok(data(ChainEvent {
                    chain_id: self.chain_id.clone(),
                    client_info,
                    counterparty_chain_id,
                    tx_hash,
                    provable_height,
                    ibc_spec_id: IbcClassic::ID,
                    event: into_value::<ibc_classic_spec::FullEvent>(
                        ibc_classic_spec::WriteAcknowledgement {
                            packet_data: packet_data_hex.into_encoding(),
                            packet_ack: packet_ack_hex.into_encoding(),
                            packet: ibc_classic_spec::PacketMetadata {
                                sequence: packet_sequence,
                                source_channel,
                                destination_channel,
                                channel_ordering,
                                timeout_height: packet_timeout_height,
                                timeout_timestamp: packet_timeout_timestamp.as_nanos(),
                            },
                        }
                        .into(),
                    ),
                }))
            }
            IbcEvent::RecvPacket {
                packet_data_hex,
                packet_timeout_height,
                packet_timeout_timestamp,
                packet_sequence,
                packet_src_port,
                packet_src_channel,
                packet_dst_port,
                packet_dst_channel,
                packet_channel_ordering: _,
                connection_id,
            } => {
                let (
                    counterparty_chain_id,
                    client_info,
                    destination_channel,
                    source_channel,
                    channel_ordering,
                ) = self
                    .make_packet_metadata(
                        height,
                        connection_id.to_owned(),
                        packet_dst_port.to_owned(),
                        packet_dst_channel.to_owned(),
                        packet_src_port.to_owned(),
                        packet_src_channel.to_owned(),
                        voyager_client,
                    )
                    .await?;

                Ok(data(ChainEvent {
                    chain_id: self.chain_id.clone(),
                    client_info,
                    counterparty_chain_id,
                    tx_hash,
                    provable_height,
                    ibc_spec_id: IbcClassic::ID,
                    event: into_value::<ibc_classic_spec::FullEvent>(
                        ibc_classic_spec::RecvPacket {
                            packet_data: packet_data_hex.into_encoding(),
                            packet: ibc_classic_spec::PacketMetadata {
                                sequence: packet_sequence,
                                source_channel,
                                destination_channel,
                                channel_ordering,
                                timeout_height: packet_timeout_height,
                                timeout_timestamp: packet_timeout_timestamp.as_nanos(),
                            },
                        }
                        .into(),
                    ),
                }))
            }
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

                let event = ibc_union_spec::event::CreateClient {
                    client_id,
                    client_type: ClientType::new(client_type),
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
                    event: into_value::<ibc_union_spec::event::FullEvent>(event),
                }))
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

                let event = ibc_union_spec::event::UpdateClient {
                    client_id,
                    client_type: client_info.client_type.clone(),
                    height: counterparty_height,
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
                    event: into_value::<ibc_union_spec::event::FullEvent>(event),
                }))
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

                let event = ibc_union_spec::event::ConnectionOpenInit {
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
                    provable_height,
                    ibc_spec_id: IbcUnion::ID,
                    event: into_value::<ibc_union_spec::event::FullEvent>(event),
                }))
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

                let event = ibc_union_spec::event::ConnectionOpenTry {
                    connection_id,
                    counterparty_connection_id,
                    client_id,
                    counterparty_client_id,
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
                    event: into_value::<ibc_union_spec::event::FullEvent>(event),
                }))
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

                let event = ibc_union_spec::event::ConnectionOpenAck {
                    connection_id,
                    counterparty_connection_id,
                    client_id,
                    counterparty_client_id,
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
                    event: into_value::<ibc_union_spec::event::FullEvent>(event),
                }))
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

                let event = ibc_union_spec::event::ConnectionOpenConfirm {
                    connection_id,
                    counterparty_connection_id,
                    client_id,
                    counterparty_client_id,
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
                    event: into_value::<ibc_union_spec::event::FullEvent>(event),
                }))
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
                        height,
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

                let event = ibc_union_spec::event::ChannelOpenInit {
                    port_id: port_id.to_string().into_bytes().into(),
                    channel_id,
                    counterparty_port_id: counterparty_port_id.into_encoding(),
                    connection,
                    version,
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
                    event: into_value::<ibc_union_spec::event::FullEvent>(event),
                }))
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
                        height,
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

                let event = ibc_union_spec::event::ChannelOpenTry {
                    port_id: port_id.to_string().into_bytes().into(),
                    channel_id,
                    counterparty_port_id: counterparty_port_id.into_encoding(),
                    counterparty_channel_id,
                    connection,
                    version: counterparty_version,
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
                    event: into_value::<ibc_union_spec::event::FullEvent>(event),
                }))
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
                        height,
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
                    .query_ibc_state(self.chain_id.clone(), height, ChannelPath { channel_id })
                    .await?;

                let event = ibc_union_spec::event::ChannelOpenAck {
                    port_id: port_id.to_string().into_bytes().into(),
                    channel_id,
                    counterparty_port_id: counterparty_port_id.into_encoding(),
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
                    event: into_value::<ibc_union_spec::event::FullEvent>(event),
                }))
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
                        height,
                        ibc_union_spec::path::ChannelPath { channel_id },
                    )
                    .await?;

                let connection = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        height,
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

                let event = ibc_union_spec::event::ChannelOpenConfirm {
                    port_id: port_id.to_string().into_bytes().into(),
                    channel_id,
                    counterparty_port_id: counterparty_port_id.into_encoding(),
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
                    event: into_value::<ibc_union_spec::event::FullEvent>(event),
                }))
            }
            IbcEvent::WasmPacketSend {
                packet_source_channel_id,
                packet_destination_channel_id,
                packet_data,
                packet_timeout_height,
                packet_timeout_timestamp,
                channel_id: _,
                packet_hash: _,
            } => {
                let packet = Packet {
                    source_channel_id: packet_source_channel_id,
                    destination_channel_id: packet_destination_channel_id,
                    data: packet_data,
                    timeout_height: packet_timeout_height,
                    timeout_timestamp: packet_timeout_timestamp,
                };

                let state = voyager_client
                    .maybe_query_ibc_state(
                        self.chain_id.clone(),
                        QueryHeight::Latest,
                        ibc_union_spec::path::BatchPacketsPath::from_packets(&[packet.clone()]),
                    )
                    .await?;

                if state.state.is_none() {
                    info!("packet already acknowledged");
                    return Ok(noop());
                }

                let source_channel = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        height,
                        ibc_union_spec::path::ChannelPath {
                            channel_id: packet.source_channel_id,
                        },
                    )
                    .await?;

                let source_connection = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        height,
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

                let event = ibc_union_spec::event::PacketSend {
                    packet_data: packet.data,
                    packet: ibc_union_spec::event::PacketMetadata {
                        source_channel: ibc_union_spec::event::ChannelMetadata {
                            channel_id: packet.source_channel_id,
                            version: source_channel.version.clone(),
                            connection: ibc_union_spec::event::ConnectionMetadata {
                                client_id: source_connection.client_id,
                                connection_id: source_channel.connection_id,
                            },
                        },
                        destination_channel: ibc_union_spec::event::ChannelMetadata {
                            channel_id: packet.destination_channel_id,
                            version: source_channel.version,
                            connection: ibc_union_spec::event::ConnectionMetadata {
                                client_id: source_connection.counterparty_client_id,
                                connection_id: source_connection
                                    .counterparty_connection_id
                                    .unwrap(),
                            },
                        },
                        timeout_height: packet.timeout_height,
                        timeout_timestamp: packet.timeout_timestamp,
                    },
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
                    event: into_value::<ibc_union_spec::event::FullEvent>(event),
                }))
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
                    .await?;

                let source_channel = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        height,
                        ibc_union_spec::path::ChannelPath {
                            channel_id: packet.source_channel_id,
                        },
                    )
                    .await?;

                let source_connection = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        height,
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

                let event = ibc_union_spec::event::PacketAck {
                    packet_data: packet.data,
                    packet: ibc_union_spec::event::PacketMetadata {
                        source_channel: ibc_union_spec::event::ChannelMetadata {
                            channel_id: packet.source_channel_id,
                            version: source_channel.version.clone(),
                            connection: ibc_union_spec::event::ConnectionMetadata {
                                client_id: source_connection.client_id,
                                connection_id: source_channel.connection_id,
                            },
                        },
                        destination_channel: ibc_union_spec::event::ChannelMetadata {
                            channel_id: packet.destination_channel_id,
                            version: source_channel.version,
                            connection: ibc_union_spec::event::ConnectionMetadata {
                                client_id: source_connection.counterparty_client_id,
                                connection_id: source_connection
                                    .counterparty_connection_id
                                    .unwrap(),
                            },
                        },
                        timeout_height: packet.timeout_height,
                        timeout_timestamp: packet.timeout_timestamp,
                    },
                    acknowledgement: acknowledgement.into_encoding(),
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
                    event: into_value::<ibc_union_spec::event::FullEvent>(event),
                }))
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
                        height,
                        ibc_union_spec::path::ChannelPath { channel_id },
                    )
                    .await?;

                let destination_connection = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        height,
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

                let counterparty_latest_height = voyager_client
                    .query_latest_height(client_state_meta.counterparty_chain_id.clone(), false)
                    .await?;

                let source_channel = voyager_client
                    .query_ibc_state(
                        client_state_meta.counterparty_chain_id.clone(),
                        counterparty_latest_height,
                        ibc_union_spec::path::ChannelPath {
                            channel_id: destination_channel.counterparty_channel_id.unwrap(),
                        },
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
                    .await?;

                let event = ibc_union_spec::event::PacketRecv {
                    packet_data: packet.data,
                    packet: ibc_union_spec::event::PacketMetadata {
                        source_channel: ibc_union_spec::event::ChannelMetadata {
                            channel_id: packet.source_channel_id,
                            version: source_channel.version.clone(),
                            connection: ibc_union_spec::event::ConnectionMetadata {
                                client_id: destination_connection.counterparty_client_id,
                                connection_id: destination_connection
                                    .counterparty_connection_id
                                    .unwrap(),
                            },
                        },
                        destination_channel: ibc_union_spec::event::ChannelMetadata {
                            channel_id: packet.destination_channel_id,
                            version: destination_channel.version.clone(),
                            connection: ibc_union_spec::event::ConnectionMetadata {
                                client_id: destination_connection.client_id,
                                connection_id: destination_channel.connection_id,
                            },
                        },
                        timeout_height: packet.timeout_height,
                        timeout_timestamp: packet.timeout_timestamp,
                    },
                    maker_msg: maker_msg.into_encoding(),
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
                    event: into_value::<ibc_union_spec::event::FullEvent>(event),
                }))
            }
            IbcEvent::WasmWriteAck {
                acknowledgement,
                channel_id,
                packet_hash,
            } => {
                let destination_channel = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        height,
                        ibc_union_spec::path::ChannelPath { channel_id },
                    )
                    .await?;

                let destination_connection = voyager_client
                    .query_ibc_state(
                        self.chain_id.clone(),
                        height,
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

                let counterparty_latest_height = voyager_client
                    .query_latest_height(client_state_meta.counterparty_chain_id.clone(), false)
                    .await?;

                let source_channel = voyager_client
                    .query_ibc_state(
                        client_state_meta.counterparty_chain_id.clone(),
                        counterparty_latest_height,
                        ibc_union_spec::path::ChannelPath {
                            channel_id: destination_channel.counterparty_channel_id.unwrap(),
                        },
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
                    .await?;

                let event = ibc_union_spec::event::WriteAck {
                    packet_data: packet.data,
                    packet: ibc_union_spec::event::PacketMetadata {
                        source_channel: ibc_union_spec::event::ChannelMetadata {
                            channel_id: packet.source_channel_id,
                            version: source_channel.version.clone(),
                            connection: ibc_union_spec::event::ConnectionMetadata {
                                client_id: destination_connection.counterparty_client_id,
                                connection_id: destination_connection
                                    .counterparty_connection_id
                                    .unwrap(),
                            },
                        },
                        destination_channel: ibc_union_spec::event::ChannelMetadata {
                            channel_id: packet.destination_channel_id,
                            version: destination_channel.version.clone(),
                            connection: ibc_union_spec::event::ConnectionMetadata {
                                client_id: destination_connection.client_id,
                                connection_id: destination_channel.connection_id,
                            },
                        },
                        timeout_height: packet.timeout_height,
                        timeout_timestamp: packet.timeout_timestamp,
                    },
                    acknowledgement: acknowledgement.into_encoding(),
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
                    event: into_value::<ibc_union_spec::event::FullEvent>(event),
                }))
            }
        }
    }
}

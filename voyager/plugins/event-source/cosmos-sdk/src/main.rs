// #![warn(clippy::unwrap_used)]

use std::{
    cmp::Ordering,
    collections::VecDeque,
    error::Error,
    fmt::{Debug, Display},
    num::{NonZeroU32, NonZeroU8, ParseIntError},
    sync::Arc,
};

use cosmos_sdk_event::CosmosSdkEvent;
use dashmap::DashMap;
use ibc_classic_spec::IbcClassic;
use ibc_union_spec::{path::ChannelPath, IbcUnion};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::{ErrorObject, ErrorObjectOwned},
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, error, info, instrument};
use unionlabs::{
    bech32::Bech32,
    ibc::core::{
        channel::{self},
        client::height::Height,
    },
    id::{ChannelId, ClientId, ConnectionId, PortId},
    option_unwrap, parse_wasm_client_type,
    primitives::{encoding::HexUnprefixed, H256},
    ErrorReporter, WasmClientType,
};
use voyager_message::{
    call::{Call, WaitForHeight},
    core::{ChainId, ClientInfo, ClientType, IbcSpec, QueryHeight},
    data::{ChainEvent, Data},
    into_value,
    module::{PluginInfo, PluginServer},
    rpc::missing_state,
    ExtensionsExt, Plugin, PluginMessage, VoyagerClient, VoyagerMessage,
};
use voyager_vm::{call, conc, data, pass::PassResult, seq, BoxDynError, Op};

use crate::{
    call::{FetchBlocks, FetchTransactions, MakeChainEvent, ModuleCall},
    callback::ModuleCallback,
    ibc_events::IbcEvent,
};

pub mod ibc_events;

pub mod call;
pub mod callback;
pub mod data;

const PER_PAGE_LIMIT: NonZeroU8 = option_unwrap!(NonZeroU8::new(10));

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(clap::Subcommand)]
pub enum Cmd {
    ChainId,
    LatestHeight,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,
    pub chain_revision: u64,

    pub cometbft_client: cometbft_rpc::Client,
    pub grpc_url: String,

    pub chunk_block_fetch_size: u64,

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
    pub grpc_url: String,

    #[serde(default)]
    pub ibc_host_contract_address: Option<Bech32<H256>>,
}

fn default_chunk_block_fetch_size() -> u64 {
    10
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = Cmd;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        let tm_client = cometbft_rpc::Client::new(config.rpc_url).await?;

        let chain_id = tm_client.status().await?.node_info.network;

        let chain_revision = chain_id
            .split('-')
            .last()
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
            grpc_url: config.grpc_url,
            chunk_block_fetch_size: config.chunk_block_fetch_size,
            checksum_cache: Arc::new(DashMap::default()),

            ibc_host_contract_address: config.ibc_host_contract_address,
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

    async fn cmd(config: Self::Config, cmd: Self::Cmd) {
        let module = Self::new(config).await.unwrap();

        match cmd {
            Cmd::ChainId => println!("{}", module.chain_id),
            Cmd::LatestHeight => println!("{}", module.latest_height().await.unwrap()),
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

    async fn client_type_of_checksum(&self, checksum: H256) -> RpcResult<Option<WasmClientType>> {
        if let Some(ty) = self.checksum_cache.get(&checksum) {
            debug!(
                %checksum,
                ty = ?*ty,
                "cache hit for checksum"
            );

            return Ok(Some(*ty));
        };

        info!(
            %checksum,
            "cache miss for checksum"
        );

        let bz = protos::ibc::lightclients::wasm::v1::query_client::QueryClient::connect(
            self.grpc_url.clone(),
        )
        .await
        .map_err(rpc_error(
            "error connecting to grpc server",
            Some(json!({
                "grpc_url": self.grpc_url
            })),
        ))?
        .code(protos::ibc::lightclients::wasm::v1::QueryCodeRequest {
            checksum: checksum.into_encoding::<HexUnprefixed>().to_string(),
        })
        .await
        .map_err(rpc_error(
            "error querying wasm code",
            Some(json!({
                "checksum": checksum,
                "grpc_url": self.grpc_url
            })),
        ))?
        .into_inner()
        .data;

        match parse_wasm_client_type(bz) {
            Ok(Some(ty)) => {
                info!(
                    %checksum,
                    ?ty,
                    "parsed checksum"
                );

                self.checksum_cache.insert(checksum, ty);

                Ok(Some(ty))
            }
            Ok(None) => Ok(None),
            Err(err) => {
                error!(
                    %checksum,
                    %err,
                    "unable to parse wasm client type"
                );

                Ok(None)
            }
        }
    }

    #[instrument(skip_all, fields(%client_id))]
    async fn checksum_of_client_id(&self, client_id: ClientId) -> RpcResult<H256> {
        type WasmClientState = protos::ibc::lightclients::wasm::v1::ClientState;

        let client_state = protos::ibc::core::client::v1::query_client::QueryClient::connect(
            self.grpc_url.clone(),
        )
        .await
        .map_err(rpc_error(
            "error connecting to grpc server",
            Some(json!({ "client_id": client_id })),
        ))?
        .client_state(protos::ibc::core::client::v1::QueryClientStateRequest {
            client_id: client_id.to_string(),
        })
        .await
        .map_err(rpc_error(
            "error querying client state",
            Some(json!({ "client_id": client_id })),
        ))?
        .into_inner()
        .client_state
        .ok_or_else(|| {
            // lol
            rpc_error(
                "error fetching client state",
                Some(json!({ "client_id": client_id })),
            )(&*Box::<dyn Error>::from("client state field is empty"))
        })?;

        assert!(
            client_state.type_url == <WasmClientState as prost::Name>::type_url(),
            "attempted to get the wasm blob checksum of a non-wasm \
            light client. this is a bug, please report this at \
            `https://github.com/unionlabs/union`."
        );

        // NOTE: We only need the checksum, so we don't need to decode the inner state contained in .data
        <WasmClientState as prost::Message>::decode(&*client_state.value)
            .map_err(rpc_error(
                "error decoding client state",
                Some(json!({ "client_id": client_id })),
            ))?
            .checksum
            .try_into()
            .map_err(rpc_error(
                "invalid checksum",
                Some(json!({ "client_id": client_id })),
            ))
    }

    // TODO: Remove
    async fn latest_height(&self) -> Result<Height, cometbft_rpc::JsonRpcError> {
        let commit_response = self.cometbft_client.commit(None).await?;

        let mut height = commit_response
            .signed_header
            .header
            .height
            .inner()
            .try_into()
            .expect("value is >= 0; qed;");

        if !commit_response.canonical {
            debug!("commit is not canonical, latest finalized height is the previous block");
            height -= 1;
        }

        debug!(height, "latest height");

        Ok(self.make_height(height))
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
        voyager_rpc_client: &VoyagerClient,
    ) -> RpcResult<(
        ChainId,
        ClientInfo,
        ibc_classic_spec::ChannelMetadata,
        ibc_classic_spec::ChannelMetadata,
        channel::order::Order,
    )> {
        let self_connection = voyager_rpc_client
            .query_ibc_state(
                self.chain_id.clone(),
                event_height.into(),
                ibc_classic_spec::ConnectionPath {
                    connection_id: self_connection_id.clone(),
                },
            )
            .await?
            .state
            .ok_or_else(missing_state("connection must exist", None))?;

        let client_info = voyager_rpc_client
            .client_info::<IbcClassic>(self.chain_id.clone(), self_connection.client_id.clone())
            .await?;

        let client_meta = voyager_rpc_client
            .client_meta::<IbcClassic>(
                self.chain_id.clone(),
                event_height.into(),
                self_connection.client_id.clone(),
            )
            .await?;

        let this_channel = voyager_rpc_client
            .query_ibc_state(
                self.chain_id.clone(),
                event_height.into(),
                ibc_classic_spec::ChannelEndPath {
                    port_id: self_port_id.clone(),
                    channel_id: self_channel_id.clone(),
                },
            )
            .await?
            .state
            .ok_or_else(missing_state("channel must exist", None))?;

        let counterparty_channel = voyager_rpc_client
            .query_ibc_state(
                client_meta.chain_id.clone(),
                QueryHeight::Latest,
                ibc_classic_spec::ChannelEndPath {
                    port_id: other_port_id.clone(),
                    channel_id: other_channel_id.clone(),
                },
            )
            .await?
            .state
            .ok_or_else(missing_state("channel must exist", None))?;

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
            client_meta.chain_id,
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
        cb: ModuleCallback,
        _data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match cb {}
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn call(&self, e: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::FetchTransactions(FetchTransactions { height, page }) => {
                info!(%height, %page, "fetching events in block");

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

                Ok(conc(
                    response
                        .txs
                        .into_iter()
                        .flat_map(|txr| {
                            txr.tx_result.events.into_iter().filter_map(move |event| {
                                debug!(%event.ty, "observed event");

                                let event = CosmosSdkEvent::<IbcEvent>::new(event.clone())
                                    .inspect_err(|e| match e {
                                        cosmos_sdk_event::Error::Deserialize(error) => {
                                            debug!("unable to parse event: {error}")
                                        }
                                        _ => {
                                            error!("{e}");
                                        }
                                    })
                                    .ok()?;

                                match (&event.contract_address, &self.ibc_host_contract_address) {
                                    (None, None) => Some((event, txr.hash)),
                                    (None, Some(_)) => Some((event, txr.hash)),
                                    (Some(_), None) => None,
                                    (Some(a), Some(b)) => {
                                        if a == b {
                                            Some((event, txr.hash))
                                        } else {
                                            None
                                        }
                                    }
                                }
                            })
                        })
                        // .collect::<Result<Vec<_>, _>>()
                        // .map_err(|err| {
                        //     ErrorObject::owned(
                        //         -1,
                        //         ErrorReporter(err).to_string(),
                        //         Some(json!({
                        //             "height": height,
                        //             "page": page
                        //         })),
                        //     )
                        // })?
                        // .into_iter()
                        .map(|(ibc_event, tx_hash)| {
                            debug!(event = %ibc_event.event.name(), "observed IBC event");
                            call(PluginMessage::new(
                                self.plugin_name(),
                                ModuleCall::from(MakeChainEvent {
                                    height,
                                    tx_hash: tx_hash.into_encoding(),
                                    event: ibc_event.event,
                                }),
                            ))
                        })
                        .chain(
                            ((page.get() * PER_PAGE_LIMIT.get() as u32) < response.total_count)
                                .then(|| {
                                    call(PluginMessage::new(
                                        self.plugin_name(),
                                        ModuleCall::from(FetchTransactions {
                                            height,
                                            page: page.checked_add(1).expect("too many pages?"),
                                        }),
                                    ))
                                }),
                        ),
                ))
            }
            ModuleCall::FetchBlocks(FetchBlocks { height }) => {
                let latest_height = e
                    .try_get::<VoyagerClient>()?
                    .query_latest_height(self.chain_id.clone(), true)
                    .await?;

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

                match height.height().cmp(&latest_height.height()) {
                    Ordering::Less => {
                        let next_height = (latest_height.height() - height.height())
                            .clamp(1, self.chunk_block_fetch_size)
                            + height.height();

                        info!("batch fetching blocks in range {height}..{next_height}");

                        Ok(conc(
                            (height.height()..next_height)
                                .map(|h| {
                                    call(PluginMessage::new(
                                        self.plugin_name(),
                                        ModuleCall::from(FetchTransactions {
                                            height: Height::new_with_revision(height.revision(), h),
                                            page: const { option_unwrap!(NonZeroU32::new(1_u32)) },
                                        }),
                                    ))
                                })
                                .chain([continuation(Height::new_with_revision(
                                    height.revision(),
                                    next_height,
                                ))]),
                        ))
                    }
                    Ordering::Equal | Ordering::Greater => Ok(conc([
                        call(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(FetchTransactions {
                                height: height.increment(),
                                page: const { option_unwrap!(NonZeroU32::new(1_u32)) },
                            }),
                        )),
                        continuation(height.increment()),
                    ])),
                }
            }
            ModuleCall::MakeChainEvent(MakeChainEvent {
                height,
                tx_hash,
                event,
            }) => {
                // events at height N are provable at height N+k where k<0
                let provable_height = height.increment();
                let voyager_client = e.try_get::<VoyagerClient>()?;

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

                        let client_meta = voyager_client
                            .client_meta::<IbcClassic>(
                                self.chain_id.clone(),
                                height.into(),
                                client_id.clone(),
                            )
                            .await?;

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
                            client_info,
                            counterparty_chain_id: client_meta.chain_id,
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
                                height.into(),
                                ibc_classic_spec::ConnectionPath {
                                    connection_id: connection_id.clone(),
                                },
                            )
                            .await?
                            .state
                            .ok_or_else(missing_state("connection must exist", None))?;

                        let client_info = voyager_client
                            .client_info::<IbcClassic>(
                                self.chain_id.clone(),
                                connection.client_id.clone(),
                            )
                            .await?;

                        let client_meta = voyager_client
                            .client_meta::<IbcClassic>(
                                self.chain_id.clone(),
                                height.into(),
                                connection.client_id.clone(),
                            )
                            .await?;

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
                            client_info,
                            counterparty_chain_id: client_meta.chain_id,
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
                                height.into(),
                                ibc_classic_spec::ConnectionPath {
                                    connection_id: connection_id.clone(),
                                },
                            )
                            .await?
                            .state
                            .ok_or_else(missing_state("connection must exist", None))?;

                        let client_info = voyager_client
                            .client_info::<IbcClassic>(
                                self.chain_id.clone(),
                                connection.client_id.clone(),
                            )
                            .await?;

                        let client_meta = voyager_client
                            .client_meta::<IbcClassic>(
                                self.chain_id.clone(),
                                height.into(),
                                connection.client_id.clone(),
                            )
                            .await?;

                        let channel = voyager_client
                            .query_ibc_state(
                                self.chain_id.clone(),
                                height.into(),
                                ibc_classic_spec::ChannelEndPath {
                                    port_id: port_id.to_owned(),
                                    channel_id: channel_id.to_owned(),
                                },
                            )
                            .await?
                            .state
                            .ok_or_else(missing_state("channel must exist", None))?;

                        Ok(data(ChainEvent {
                            chain_id: self.chain_id.clone(),
                            client_info,
                            counterparty_chain_id: client_meta.chain_id,
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
                                        timeout_timestamp: packet_timeout_timestamp,
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
                                        timeout_timestamp: packet_timeout_timestamp,
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
                                        timeout_timestamp: packet_timeout_timestamp,
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
                                        timeout_timestamp: packet_timeout_timestamp,
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
                                        timeout_timestamp: packet_timeout_timestamp,
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

                        let client_meta = voyager_client
                            .client_meta::<IbcUnion>(
                                self.chain_id.clone(),
                                height.into(),
                                client_id,
                            )
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
                            counterparty_chain_id: client_meta.chain_id,
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

                        let client_meta = voyager_client
                            .client_meta::<IbcUnion>(
                                self.chain_id.clone(),
                                height.into(),
                                client_id,
                            )
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
                            counterparty_chain_id: client_meta.chain_id,
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

                        let client_meta = voyager_client
                            .client_meta::<IbcUnion>(
                                self.chain_id.clone(),
                                height.into(),
                                client_id,
                            )
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
                            counterparty_chain_id: client_meta.chain_id,
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

                        let client_meta = voyager_client
                            .client_meta::<IbcUnion>(
                                self.chain_id.clone(),
                                height.into(),
                                client_id,
                            )
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
                            counterparty_chain_id: client_meta.chain_id,
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

                        let client_meta = voyager_client
                            .client_meta::<IbcUnion>(
                                self.chain_id.clone(),
                                height.into(),
                                client_id,
                            )
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
                            counterparty_chain_id: client_meta.chain_id,
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

                        let client_meta = voyager_client
                            .client_meta::<IbcUnion>(
                                self.chain_id.clone(),
                                height.into(),
                                client_id,
                            )
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
                            counterparty_chain_id: client_meta.chain_id,
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
                                QueryHeight::Specific(height),
                                ibc_union_spec::path::ConnectionPath { connection_id },
                            )
                            .await?
                            .state
                            .unwrap();

                        let client_info = voyager_client
                            .client_info::<IbcUnion>(self.chain_id.clone(), connection.client_id)
                            .await?;

                        let client_meta = voyager_client
                            .client_meta::<IbcUnion>(
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
                            counterparty_chain_id: client_meta.chain_id,
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
                                QueryHeight::Specific(height),
                                ibc_union_spec::path::ConnectionPath { connection_id },
                            )
                            .await?
                            .state
                            .unwrap();

                        let client_info = voyager_client
                            .client_info::<IbcUnion>(self.chain_id.clone(), connection.client_id)
                            .await?;

                        let client_meta = voyager_client
                            .client_meta::<IbcUnion>(
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
                            counterparty_chain_id: client_meta.chain_id,
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
                                QueryHeight::Specific(height),
                                ibc_union_spec::path::ConnectionPath { connection_id },
                            )
                            .await?
                            .state
                            .unwrap();

                        let client_info = voyager_client
                            .client_info::<IbcUnion>(self.chain_id.clone(), connection.client_id)
                            .await?;

                        let client_meta = voyager_client
                            .client_meta::<IbcUnion>(
                                self.chain_id.clone(),
                                height.into(),
                                connection.client_id,
                            )
                            .await?;

                        let channel = voyager_client
                            .query_ibc_state(
                                self.chain_id.clone(),
                                height.into(),
                                ChannelPath { channel_id },
                            )
                            .await?
                            .state
                            .ok_or_else(missing_state("channel must exist", None))?;

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
                            counterparty_chain_id: client_meta.chain_id,
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
                                QueryHeight::Specific(height),
                                ibc_union_spec::path::ChannelPath { channel_id },
                            )
                            .await?
                            .state
                            .unwrap();

                        let connection = voyager_client
                            .query_ibc_state(
                                self.chain_id.clone(),
                                QueryHeight::Specific(height),
                                ibc_union_spec::path::ConnectionPath { connection_id },
                            )
                            .await?
                            .state
                            .unwrap();

                        let client_info = voyager_client
                            .client_info::<IbcUnion>(self.chain_id.clone(), connection.client_id)
                            .await?;

                        let client_meta = voyager_client
                            .client_meta::<IbcUnion>(
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
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height,
                            ibc_spec_id: IbcUnion::ID,
                            event: into_value::<ibc_union_spec::event::FullEvent>(event),
                        }))
                    }
                    IbcEvent::WasmPacketSend { packet } => {
                        let source_channel = voyager_client
                            .query_ibc_state(
                                self.chain_id.clone(),
                                QueryHeight::Specific(height),
                                ibc_union_spec::path::ChannelPath {
                                    channel_id: packet.source_channel_id,
                                },
                            )
                            .await?
                            .state
                            .unwrap();

                        let source_connection = voyager_client
                            .query_ibc_state(
                                self.chain_id.clone(),
                                QueryHeight::Specific(height),
                                ibc_union_spec::path::ConnectionPath {
                                    connection_id: source_channel.connection_id,
                                },
                            )
                            .await?
                            .state
                            .unwrap();

                        let client_info = voyager_client
                            .client_info::<IbcUnion>(
                                self.chain_id.clone(),
                                source_connection.client_id,
                            )
                            .await?;

                        let client_meta = voyager_client
                            .client_meta::<IbcUnion>(
                                self.chain_id.clone(),
                                height.into(),
                                source_connection.client_id,
                            )
                            .await?;

                        let event = ibc_union_spec::event::PacketSend {
                            packet_data: packet.data.into(),
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
                                        connection_id: source_connection.counterparty_connection_id,
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
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height,
                            ibc_spec_id: IbcUnion::ID,
                            event: into_value::<ibc_union_spec::event::FullEvent>(event),
                        }))
                    }
                    IbcEvent::WasmPacketAck {
                        packet,
                        acknowledgement,
                    } => {
                        let source_channel = voyager_client
                            .query_ibc_state(
                                self.chain_id.clone(),
                                QueryHeight::Specific(height),
                                ibc_union_spec::path::ChannelPath {
                                    channel_id: packet.source_channel_id,
                                },
                            )
                            .await?
                            .state
                            .unwrap();

                        let source_connection = voyager_client
                            .query_ibc_state(
                                self.chain_id.clone(),
                                QueryHeight::Specific(height),
                                ibc_union_spec::path::ConnectionPath {
                                    connection_id: source_channel.connection_id,
                                },
                            )
                            .await?
                            .state
                            .unwrap();

                        let client_info = voyager_client
                            .client_info::<IbcUnion>(
                                self.chain_id.clone(),
                                source_connection.client_id,
                            )
                            .await?;

                        let client_meta = voyager_client
                            .client_meta::<IbcUnion>(
                                self.chain_id.clone(),
                                height.into(),
                                source_connection.client_id,
                            )
                            .await?;

                        let event = ibc_union_spec::event::PacketAck {
                            packet_data: packet.data.into(),
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
                                        connection_id: source_connection.counterparty_connection_id,
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
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height,
                            ibc_spec_id: IbcUnion::ID,
                            event: into_value::<ibc_union_spec::event::FullEvent>(event),
                        }))
                    }
                    IbcEvent::WasmPacketRecv {
                        packet,
                        relayer_msg,
                    } => {
                        let destination_channel = voyager_client
                            .query_ibc_state(
                                self.chain_id.clone(),
                                QueryHeight::Specific(height),
                                ibc_union_spec::path::ChannelPath {
                                    channel_id: packet.destination_channel_id,
                                },
                            )
                            .await?
                            .state
                            .unwrap();

                        let destination_connection = voyager_client
                            .query_ibc_state(
                                self.chain_id.clone(),
                                QueryHeight::Specific(height),
                                ibc_union_spec::path::ConnectionPath {
                                    connection_id: destination_channel.connection_id,
                                },
                            )
                            .await?
                            .state
                            .unwrap();

                        let client_info = voyager_client
                            .client_info::<IbcUnion>(
                                self.chain_id.clone(),
                                destination_connection.client_id,
                            )
                            .await?;

                        let client_meta = voyager_client
                            .client_meta::<IbcUnion>(
                                self.chain_id.clone(),
                                height.into(),
                                destination_connection.client_id,
                            )
                            .await?;

                        let source_channel = voyager_client
                            .query_ibc_state(
                                client_meta.chain_id.clone(),
                                QueryHeight::Latest,
                                ibc_union_spec::path::ChannelPath {
                                    channel_id: packet.source_channel_id,
                                },
                            )
                            .await?
                            .state
                            .unwrap();

                        let event = ibc_union_spec::event::PacketRecv {
                            packet_data: packet.data.into(),
                            packet: ibc_union_spec::event::PacketMetadata {
                                source_channel: ibc_union_spec::event::ChannelMetadata {
                                    channel_id: packet.source_channel_id,
                                    version: source_channel.version.clone(),
                                    connection: ibc_union_spec::event::ConnectionMetadata {
                                        client_id: destination_connection.counterparty_client_id,
                                        connection_id: destination_connection
                                            .counterparty_connection_id,
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
                            relayer_msg: relayer_msg.into_encoding(),
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
                            event: into_value::<ibc_union_spec::event::FullEvent>(event),
                        }))
                    }
                    IbcEvent::WasmWriteAck {
                        packet,
                        acknowledgement,
                    } => {
                        let destination_channel = voyager_client
                            .query_ibc_state(
                                self.chain_id.clone(),
                                QueryHeight::Specific(height),
                                ibc_union_spec::path::ChannelPath {
                                    channel_id: packet.destination_channel_id,
                                },
                            )
                            .await?
                            .state
                            .unwrap();

                        let destination_connection = voyager_client
                            .query_ibc_state(
                                self.chain_id.clone(),
                                QueryHeight::Specific(height),
                                ibc_union_spec::path::ConnectionPath {
                                    connection_id: destination_channel.connection_id,
                                },
                            )
                            .await?
                            .state
                            .unwrap();

                        let client_info = voyager_client
                            .client_info::<IbcUnion>(
                                self.chain_id.clone(),
                                destination_connection.client_id,
                            )
                            .await?;

                        let client_meta = voyager_client
                            .client_meta::<IbcUnion>(
                                self.chain_id.clone(),
                                height.into(),
                                destination_connection.client_id,
                            )
                            .await?;

                        let source_channel = voyager_client
                            .query_ibc_state(
                                client_meta.chain_id.clone(),
                                QueryHeight::Latest,
                                ibc_union_spec::path::ChannelPath {
                                    channel_id: packet.source_channel_id,
                                },
                            )
                            .await?
                            .state
                            .unwrap();

                        let event = ibc_union_spec::event::WriteAck {
                            packet_data: packet.data.into(),
                            packet: ibc_union_spec::event::PacketMetadata {
                                source_channel: ibc_union_spec::event::ChannelMetadata {
                                    channel_id: packet.source_channel_id,
                                    version: source_channel.version.clone(),
                                    connection: ibc_union_spec::event::ConnectionMetadata {
                                        client_id: destination_connection.counterparty_client_id,
                                        connection_id: destination_connection
                                            .counterparty_connection_id,
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
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height,
                            ibc_spec_id: IbcUnion::ID,
                            event: into_value::<ibc_union_spec::event::FullEvent>(event),
                        }))
                    }
                }
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

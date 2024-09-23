// #![warn(clippy::unwrap_used)]

use std::{
    collections::VecDeque,
    error::Error,
    fmt::{Debug, Display},
    num::{NonZeroU32, NonZeroU8, ParseIntError},
    sync::Arc,
};

use dashmap::DashMap;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::{ErrorObject, ErrorObjectOwned},
};
use queue_msg::{call, conc, data, BoxDynError, Op};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use serde_utils::Hex;
use tracing::{debug, error, info, instrument, warn};
use unionlabs::{
    encoding::{DecodeAs, Proto},
    events::{
        ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit, ChannelOpenTry, ClientMisbehaviour,
        ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit, ConnectionOpenTry,
        CreateClient, IbcEvent, SubmitEvidence, UpdateClient,
    },
    hash::{H256, H64},
    ibc::core::{
        channel::{self, channel::Channel},
        client::height::Height,
        commitment::merkle_proof::MerkleProof,
        connection::connection_end::ConnectionEnd,
    },
    ics24::{
        AcknowledgementPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath,
        CommitmentPath, ConnectionPath, IbcPath, NextClientSequencePath,
        NextConnectionSequencePath, NextSequenceAckPath, NextSequenceRecvPath,
        NextSequenceSendPath, Path, ReceiptPath,
    },
    id::{ChannelId, ClientId, ConnectionId, PortId},
    option_unwrap, parse_wasm_client_type, ErrorReporter, QueryHeight, WasmClientType,
};
use voyager_message::{
    call::Call,
    core::{ChainId, ClientInfo, ClientType, IbcGo08WasmClientMetadata, IbcInterface},
    data::{ChainEvent, ChannelMetadata, ConnectionMetadata, Data, PacketMetadata},
    module::{
        ChainModuleInfo, ChainModuleServer, ModuleInfo, QueueInteractionsServer, RawClientState,
    },
    reconnecting_jsonrpc_ws_client,
    rpc::{json_rpc_error_to_rpc_error, missing_state, VoyagerRpcClient, VoyagerRpcClientExt},
    run_module_server, ModuleContext, ModuleServer, VoyagerMessage, FATAL_JSONRPC_ERROR_CODE,
};

use crate::{
    call::{FetchBlocks, FetchTransactions, MakeChainEvent, ModuleCall},
    callback::ModuleCallback,
    data::ModuleData,
};

pub mod call;
pub mod callback;
pub mod data;

const PER_PAGE_LIMIT: NonZeroU8 = option_unwrap!(NonZeroU8::new(10));

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_module_server::<Module, _, _, _>().await
}

#[derive(clap::Subcommand)]
pub enum Cmd {
    ChainId,
    LatestHeight,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId<'static>,
    pub chain_revision: u64,

    pub tm_client: cometbft_rpc::Client,
    pub grpc_url: String,

    pub checksum_cache: Arc<DashMap<H256, WasmClientType>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId<'static>,
    pub ws_url: String,
    pub grpc_url: String,
}

impl ModuleContext for Module {
    type Config = Config;
    type Cmd = Cmd;
    type Info = ChainModuleInfo;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        let tm_client = cometbft_rpc::Client::new(config.ws_url).await?;

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
            tm_client,
            chain_id: ChainId::new(chain_id),
            chain_revision,
            grpc_url: config.grpc_url,
            checksum_cache: Arc::new(DashMap::default()),
        })
    }

    fn info(config: Self::Config) -> ModuleInfo<Self::Info> {
        ModuleInfo {
            name: plugin_name(&config.chain_id),
            kind: ChainModuleInfo {
                chain_id: config.chain_id,
            },
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
            revision_number: self.chain_revision,
            revision_height: height,
        }
    }

    async fn client_type_of_checksum(&self, checksum: H256) -> RpcResult<Option<WasmClientType>> {
        if let Some(ty) = self.checksum_cache.get(&checksum) {
            debug!(
                checksum = %checksum.to_string_unprefixed(),
                ty = ?*ty,
                "cache hit for checksum"
            );

            return Ok(Some(*ty));
        };

        info!(
            checksum = %checksum.to_string_unprefixed(),
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
            checksum: checksum.to_string_unprefixed(),
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
                    checksum = %checksum.to_string_unprefixed(),
                    ?ty,
                    "parsed checksum"
                );

                self.checksum_cache.insert(checksum, ty);

                Ok(Some(ty))
            }
            Ok(None) => Ok(None),
            Err(err) => {
                error!(
                    checksum = %checksum.to_string_unprefixed(),
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

    // async fn fetch_connection(&self, connection_id: ConnectionId) -> (ConnectionEnd, Height) {
    //     let inner = protos::ibc::core::connection::v1::query_client::QueryClient::connect(
    //         self.grpc_url.clone(),
    //     )
    //     .await
    //     .unwrap()
    //     .connection(protos::ibc::core::connection::v1::QueryConnectionRequest {
    //         connection_id: connection_id.to_string(),
    //     })
    //     .await
    //     .unwrap()
    //     .into_inner();

    //     (
    //         inner.connection.unwrap().try_into().unwrap(),
    //         inner.proof_height.unwrap().into(),
    //     )
    // }

    // async fn fetch_client(&self, client_id: ClientId) -> (Vec<u8>, Height) {
    //     let inner = protos::ibc::core::client::v1::query_client::QueryClient::connect(
    //         self.grpc_url.clone(),
    //     )
    //     .await
    //     .unwrap()
    //     .client_state(protos::ibc::core::client::v1::QueryClientStateRequest {
    //         client_id: client_id.to_string(),
    //     })
    //     .await
    //     .unwrap()
    //     .into_inner();

    //     (
    //         inner.client_state.unwrap().try_into().unwrap(),
    //         inner.proof_height.unwrap().into(),
    //     )
    // }

    async fn latest_height(&self) -> Result<Height, cometbft_rpc::JsonRpcError> {
        let commit_response = self.tm_client.commit(None).await?;

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
        voyager_rpc_client: &reconnecting_jsonrpc_ws_client::Client,
    ) -> RpcResult<(
        ChainId<'static>,
        ClientInfo,
        ChannelMetadata,
        ChannelMetadata,
        channel::order::Order,
    )> {
        let self_connection = voyager_rpc_client
            .query_ibc_state_typed(
                self.chain_id.clone(),
                event_height.into(),
                ConnectionPath {
                    connection_id: self_connection_id.clone(),
                },
            )
            .await
            .map_err(json_rpc_error_to_rpc_error)?
            .state
            .ok_or_else(missing_state("connection must exist", None))?;

        let client_info = voyager_rpc_client
            .client_info(self.chain_id.clone(), self_connection.client_id.clone())
            .await
            .map_err(json_rpc_error_to_rpc_error)?;

        let client_meta = voyager_rpc_client
            .client_meta(
                self.chain_id.clone(),
                event_height.into(),
                self_connection.client_id.clone(),
            )
            .await
            .map_err(json_rpc_error_to_rpc_error)?;

        let this_channel = voyager_rpc_client
            .query_ibc_state_typed(
                self.chain_id.clone(),
                event_height.into(),
                ChannelEndPath {
                    port_id: self_port_id.clone(),
                    channel_id: self_channel_id.clone(),
                },
            )
            .await
            .map_err(json_rpc_error_to_rpc_error)?
            .state
            .ok_or_else(missing_state("channel must exist", None))?;

        let counterparty_channel = voyager_rpc_client
            .query_ibc_state_typed(
                client_meta.chain_id.clone(),
                QueryHeight::Latest,
                ChannelEndPath {
                    port_id: other_port_id.clone(),
                    channel_id: other_channel_id.clone(),
                },
            )
            .await
            .map_err(json_rpc_error_to_rpc_error)?
            .state
            .ok_or_else(missing_state("channel must exist", None))?;

        let source_channel = ChannelMetadata {
            port_id: self_port_id.clone(),
            channel_id: self_channel_id.clone(),
            version: this_channel.version,
            connection: ConnectionMetadata {
                client_id: self_connection.client_id,
                connection_id: self_connection_id.clone(),
            },
        };
        let destination_channel = ChannelMetadata {
            port_id: other_port_id.clone(),
            channel_id: other_channel_id.clone(),
            version: counterparty_channel.version,
            connection: ConnectionMetadata {
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
impl QueueInteractionsServer<ModuleData, ModuleCall, ModuleCallback> for ModuleServer<Module> {
    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn callback(
        &self,
        cb: ModuleCallback,
        _data: VecDeque<Data<ModuleData>>,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match cb {}
    }

    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn call(
        &self,
        msg: ModuleCall,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match msg {
            ModuleCall::FetchTransactions(FetchTransactions { height, page }) => {
                info!(%height, %page, "fetching events in block");

                let response = self
                    .ctx
                    .tm_client
                    .tx_search(
                        format!("tx.height={}", height.revision_height),
                        false,
                        page,
                        PER_PAGE_LIMIT,
                        cometbft_rpc::types::Order::Desc,
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
                                IbcEvent::try_from_tendermint_event(event)
                                    .map(|event| event.map(|event| (event, txr.hash)))
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|err| {
                            ErrorObject::owned(
                                -1,
                                ErrorReporter(err).to_string(),
                                Some(json!({
                                    "height": height,
                                    "page": page
                                })),
                            )
                        })?
                        .into_iter()
                        .map(|(ibc_event, tx_hash)| {
                            debug!(event = %ibc_event.name(), "observed IBC event");
                            call(Call::plugin(
                                self.ctx.plugin_name(),
                                MakeChainEvent {
                                    height,
                                    tx_hash,
                                    event: ibc_event,
                                },
                            ))
                        })
                        .chain(
                            ((page.get() * PER_PAGE_LIMIT.get() as u32) < response.total_count)
                                .then(|| {
                                    call(Call::plugin(
                                        self.ctx.plugin_name(),
                                        FetchTransactions {
                                            height,
                                            page: page.checked_add(1).expect("too many pages?"),
                                        },
                                    ))
                                }),
                        ),
                ))
            }
            ModuleCall::FetchBlocks(FetchBlocks {
                from_height,
                to_height,
            }) => {
                assert!(from_height.revision_height < to_height.revision_height);

                if to_height.revision_height - from_height.revision_height == 1 {
                    Ok(call(Call::plugin(
                        self.ctx.plugin_name(),
                        FetchTransactions {
                            height: from_height,
                            page: const { option_unwrap!(NonZeroU32::new(1_u32)) },
                        },
                    )))
                } else {
                    // this is exclusive on `to`, so fetch the `from` block and "discard" the `to` block
                    // the assumption is that another message with `to..N` will be queued, which then following
                    // this logic will fetch `to`.

                    let new_from_height = from_height.increment();

                    Ok(conc(
                        [call(Call::plugin(
                            self.ctx.plugin_name(),
                            FetchTransactions {
                                height: from_height,
                                page: const { option_unwrap!(NonZeroU32::new(1_u32)) },
                            },
                        ))]
                        .into_iter()
                        .chain((new_from_height != to_height).then(|| {
                            debug!("range not completed, requeueing fetch from {new_from_height} to {to_height}");

                            call(Call::plugin(
                                self.ctx.plugin_name(),
                                FetchBlocks {
                                    from_height: new_from_height,
                                    to_height,
                                },
                            ))
                        })),
                    ))
                }
            }
            ModuleCall::MakeChainEvent(MakeChainEvent {
                height,
                tx_hash,
                event,
            }) => {
                // events at height N are provable at height N+k where k<0
                let provable_height = height.increment();

                match event {
                    IbcEvent::SubmitEvidence(SubmitEvidence { .. }) => {
                        // TODO: Not sure how to handle this one, since it only contains the hash
                        panic!()
                    }
                    IbcEvent::CreateClient(CreateClient { ref client_id, .. })
                    | IbcEvent::UpdateClient(UpdateClient { ref client_id, .. })
                    | IbcEvent::ClientMisbehaviour(ClientMisbehaviour { ref client_id, .. })
                    | IbcEvent::ConnectionOpenInit(ConnectionOpenInit { ref client_id, .. })
                    | IbcEvent::ConnectionOpenTry(ConnectionOpenTry { ref client_id, .. })
                    | IbcEvent::ConnectionOpenAck(ConnectionOpenAck { ref client_id, .. })
                    | IbcEvent::ConnectionOpenConfirm(ConnectionOpenConfirm {
                        ref client_id,
                        ..
                    }) => {
                        let client_info = self
                            .voyager_rpc_client
                            .client_info(self.ctx.chain_id.clone(), client_id.clone())
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?;

                        let client_meta = self
                            .voyager_rpc_client
                            .client_meta(
                                self.ctx.chain_id.clone(),
                                height.into(),
                                client_id.clone(),
                            )
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?;

                        Ok(data(ChainEvent {
                            chain_id: self.ctx.chain_id.clone(),
                            client_info,
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height,
                            event: match event {
                                IbcEvent::CreateClient(event) => {
                                    voyager_message::data::CreateClient {
                                        client_id: event.client_id,
                                        client_type: ClientType::new(event.client_type),
                                        consensus_height: event.consensus_height,
                                    }
                                    .into()
                                }
                                IbcEvent::UpdateClient(event) => {
                                    voyager_message::data::UpdateClient {
                                        client_id: event.client_id,
                                        client_type: ClientType::new(event.client_type),
                                        consensus_heights: event.consensus_heights,
                                    }
                                    .into()
                                }
                                IbcEvent::ConnectionOpenInit(event) => {
                                    voyager_message::data::ConnectionOpenInit {
                                        client_id: event.client_id,
                                        connection_id: event.connection_id,
                                        counterparty_client_id: event.counterparty_client_id,
                                    }
                                }
                                .into(),
                                IbcEvent::ConnectionOpenTry(event) => {
                                    voyager_message::data::ConnectionOpenTry {
                                        client_id: event.client_id,
                                        connection_id: event.connection_id,
                                        counterparty_client_id: event.counterparty_client_id,
                                        counterparty_connection_id: event
                                            .counterparty_connection_id,
                                    }
                                }
                                .into(),
                                IbcEvent::ConnectionOpenAck(event) => {
                                    voyager_message::data::ConnectionOpenAck {
                                        client_id: event.client_id,
                                        connection_id: event.connection_id,
                                        counterparty_client_id: event.counterparty_client_id,
                                        counterparty_connection_id: event
                                            .counterparty_connection_id,
                                    }
                                }
                                .into(),
                                IbcEvent::ConnectionOpenConfirm(event) => {
                                    voyager_message::data::ConnectionOpenConfirm {
                                        client_id: event.client_id,
                                        connection_id: event.connection_id,
                                        counterparty_client_id: event.counterparty_client_id,
                                        counterparty_connection_id: event
                                            .counterparty_connection_id,
                                    }
                                }
                                .into(),
                                _ => unreachable!("who needs flow typing"),
                            },
                        }))
                    }
                    IbcEvent::ChannelOpenInit(ChannelOpenInit {
                        ref connection_id, ..
                    })
                    | IbcEvent::ChannelOpenTry(ChannelOpenTry {
                        ref connection_id, ..
                    }) => {
                        let connection = self
                            .voyager_rpc_client
                            .query_ibc_state_typed(
                                self.ctx.chain_id.clone(),
                                height.into(),
                                ConnectionPath {
                                    connection_id: connection_id.clone(),
                                },
                            )
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?
                            .state
                            .ok_or_else(missing_state("connection must exist", None))?;

                        let client_info = self
                            .voyager_rpc_client
                            .client_info(self.ctx.chain_id.clone(), connection.client_id.clone())
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?;

                        let client_meta = self
                            .voyager_rpc_client
                            .client_meta(
                                self.ctx.chain_id.clone(),
                                height.into(),
                                connection.client_id.clone(),
                            )
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?;

                        Ok(data(ChainEvent {
                            chain_id: self.ctx.chain_id.clone(),
                            client_info,
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height,
                            event: match event {
                                IbcEvent::ChannelOpenInit(event) => {
                                    voyager_message::data::ChannelOpenInit {
                                        port_id: event.port_id,
                                        channel_id: event.channel_id,
                                        counterparty_port_id: event.counterparty_port_id,
                                        connection,
                                        version: event.version,
                                    }
                                }
                                .into(),
                                IbcEvent::ChannelOpenTry(event) => {
                                    voyager_message::data::ChannelOpenTry {
                                        port_id: event.port_id,
                                        channel_id: event.channel_id,
                                        counterparty_port_id: event.counterparty_port_id,
                                        counterparty_channel_id: event.counterparty_channel_id,
                                        connection,
                                        version: event.version,
                                    }
                                    .into()
                                }
                                _ => unreachable!("who needs flow typing"),
                            },
                        }))
                    }
                    IbcEvent::ChannelOpenAck(ChannelOpenAck {
                        ref connection_id,
                        ref port_id,
                        ref channel_id,
                        ..
                    })
                    | IbcEvent::ChannelOpenConfirm(ChannelOpenConfirm {
                        ref connection_id,
                        ref port_id,
                        ref channel_id,
                        ..
                    }) => {
                        let connection = self
                            .voyager_rpc_client
                            .query_ibc_state_typed(
                                self.ctx.chain_id.clone(),
                                height.into(),
                                ConnectionPath {
                                    connection_id: connection_id.clone(),
                                },
                            )
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?
                            .state
                            .ok_or_else(missing_state("connection must exist", None))?;

                        let client_info = self
                            .voyager_rpc_client
                            .client_info(self.ctx.chain_id.clone(), connection.client_id.clone())
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?;

                        let client_meta = self
                            .voyager_rpc_client
                            .client_meta(
                                self.ctx.chain_id.clone(),
                                height.into(),
                                connection.client_id.clone(),
                            )
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?;

                        let channel = self
                            .voyager_rpc_client
                            .query_ibc_state_typed(
                                self.ctx.chain_id.clone(),
                                height.into(),
                                ChannelEndPath {
                                    port_id: port_id.to_owned(),
                                    channel_id: channel_id.to_owned(),
                                },
                            )
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?
                            .state
                            .ok_or_else(missing_state("channel must exist", None))?;

                        Ok(data(ChainEvent {
                            chain_id: self.ctx.chain_id.clone(),
                            client_info,
                            counterparty_chain_id: client_meta.chain_id,
                            tx_hash,
                            provable_height,
                            event: match event {
                                IbcEvent::ChannelOpenAck(event) => {
                                    voyager_message::data::ChannelOpenAck {
                                        port_id: event.port_id,
                                        channel_id: event.channel_id,
                                        counterparty_port_id: event.counterparty_port_id,
                                        counterparty_channel_id: event.counterparty_channel_id,
                                        connection,
                                        version: channel.version,
                                    }
                                }
                                .into(),
                                IbcEvent::ChannelOpenConfirm(event) => {
                                    voyager_message::data::ChannelOpenConfirm {
                                        port_id: event.port_id,
                                        channel_id: event.channel_id,
                                        counterparty_port_id: event.counterparty_port_id,
                                        counterparty_channel_id: event.counterparty_channel_id,
                                        connection,
                                        version: channel.version,
                                    }
                                    .into()
                                }
                                _ => unreachable!("who needs flow typing"),
                            },
                        }))
                    }
                    // packet origin is this chain
                    IbcEvent::SendPacket(event) => {
                        let (
                            counterparty_chain_id,
                            client_info,
                            source_channel,
                            destination_channel,
                            channel_ordering,
                        ) = self
                            .ctx
                            .make_packet_metadata(
                                height,
                                event.connection_id.to_owned(),
                                event.packet_src_port.to_owned(),
                                event.packet_src_channel.to_owned(),
                                event.packet_dst_port.to_owned(),
                                event.packet_dst_channel.to_owned(),
                                &self.voyager_rpc_client,
                            )
                            .await?;

                        Ok(data(ChainEvent {
                            chain_id: self.ctx.chain_id.clone(),
                            client_info,
                            counterparty_chain_id,
                            tx_hash,
                            provable_height,
                            event: voyager_message::data::SendPacket {
                                packet_data: event.packet_data_hex,
                                packet: PacketMetadata {
                                    sequence: event.packet_sequence,
                                    source_channel,
                                    destination_channel,
                                    channel_ordering,
                                    timeout_height: event.packet_timeout_height,
                                    timeout_timestamp: event.packet_timeout_timestamp,
                                },
                            }
                            .into(),
                        }))
                    }
                    IbcEvent::TimeoutPacket(event) => {
                        let (
                            counterparty_chain_id,
                            client_info,
                            source_channel,
                            destination_channel,
                            channel_ordering,
                        ) = self
                            .ctx
                            .make_packet_metadata(
                                height,
                                event.connection_id.to_owned(),
                                event.packet_src_port.to_owned(),
                                event.packet_src_channel.to_owned(),
                                event.packet_dst_port.to_owned(),
                                event.packet_dst_channel.to_owned(),
                                &self.voyager_rpc_client,
                            )
                            .await?;

                        Ok(data(ChainEvent {
                            chain_id: self.ctx.chain_id.clone(),
                            client_info,
                            counterparty_chain_id,
                            tx_hash,
                            provable_height,
                            event: voyager_message::data::TimeoutPacket {
                                packet: PacketMetadata {
                                    sequence: event.packet_sequence,
                                    source_channel,
                                    destination_channel,
                                    channel_ordering,
                                    timeout_height: event.packet_timeout_height,
                                    timeout_timestamp: event.packet_timeout_timestamp,
                                },
                            }
                            .into(),
                        }))
                    }
                    IbcEvent::AcknowledgePacket(event) => {
                        let (
                            counterparty_chain_id,
                            client_info,
                            source_channel,
                            destination_channel,
                            channel_ordering,
                        ) = self
                            .ctx
                            .make_packet_metadata(
                                height,
                                event.connection_id.to_owned(),
                                event.packet_src_port.to_owned(),
                                event.packet_src_channel.to_owned(),
                                event.packet_dst_port.to_owned(),
                                event.packet_dst_channel.to_owned(),
                                &self.voyager_rpc_client,
                            )
                            .await?;

                        Ok(data(ChainEvent {
                            chain_id: self.ctx.chain_id.clone(),
                            client_info,
                            counterparty_chain_id,
                            tx_hash,
                            provable_height,
                            event: voyager_message::data::AcknowledgePacket {
                                packet: PacketMetadata {
                                    sequence: event.packet_sequence,
                                    source_channel,
                                    destination_channel,
                                    channel_ordering,
                                    timeout_height: event.packet_timeout_height,
                                    timeout_timestamp: event.packet_timeout_timestamp,
                                },
                            }
                            .into(),
                        }))
                    }
                    // packet origin is the counterparty chain (if i put this comment above this pattern rustfmt explodes)
                    IbcEvent::WriteAcknowledgement(event) => {
                        let (
                            counterparty_chain_id,
                            client_info,
                            destination_channel,
                            source_channel,
                            channel_ordering,
                        ) = self
                            .ctx
                            .make_packet_metadata(
                                height,
                                event.connection_id.to_owned(),
                                event.packet_dst_port.to_owned(),
                                event.packet_dst_channel.to_owned(),
                                event.packet_src_port.to_owned(),
                                event.packet_src_channel.to_owned(),
                                &self.voyager_rpc_client,
                            )
                            .await?;

                        Ok(data(ChainEvent {
                            chain_id: self.ctx.chain_id.clone(),
                            client_info,
                            counterparty_chain_id,
                            tx_hash,
                            provable_height,
                            event: voyager_message::data::WriteAcknowledgement {
                                packet_data: event.packet_data_hex,
                                packet_ack: event.packet_ack_hex,
                                packet: PacketMetadata {
                                    sequence: event.packet_sequence,
                                    source_channel,
                                    destination_channel,
                                    channel_ordering,
                                    timeout_height: event.packet_timeout_height,
                                    timeout_timestamp: event.packet_timeout_timestamp,
                                },
                            }
                            .into(),
                        }))
                    }
                    IbcEvent::RecvPacket(event) => {
                        let (
                            counterparty_chain_id,
                            client_info,
                            destination_channel,
                            source_channel,
                            channel_ordering,
                        ) = self
                            .ctx
                            .make_packet_metadata(
                                height,
                                event.connection_id.to_owned(),
                                event.packet_dst_port.to_owned(),
                                event.packet_dst_channel.to_owned(),
                                event.packet_src_port.to_owned(),
                                event.packet_src_channel.to_owned(),
                                &self.voyager_rpc_client,
                            )
                            .await?;

                        Ok(data(ChainEvent {
                            chain_id: self.ctx.chain_id.clone(),
                            client_info,
                            counterparty_chain_id,
                            tx_hash,
                            provable_height,
                            event: voyager_message::data::RecvPacket {
                                packet_data: event.packet_data_hex,
                                packet: PacketMetadata {
                                    sequence: event.packet_sequence,
                                    source_channel,
                                    destination_channel,
                                    channel_ordering,
                                    timeout_height: event.packet_timeout_height,
                                    timeout_timestamp: event.packet_timeout_timestamp,
                                },
                            }
                            .into(),
                        }))
                    }
                }
            }
        }
    }
}

#[async_trait]
impl ChainModuleServer<ModuleData, ModuleCall, ModuleCallback> for ModuleServer<Module> {
    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    fn chain_id(&self) -> RpcResult<ChainId<'static>> {
        Ok(self.ctx.chain_id.clone())
    }

    /// Query the latest finalized height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn query_latest_height(&self) -> RpcResult<Height> {
        self.ctx
            .latest_height()
            .await
            // TODO: Add more context here
            .map_err(|err| ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>))
    }

    /// Query the latest (non-finalized) height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn query_latest_height_as_destination(&self) -> RpcResult<Height> {
        self.ctx
            .latest_height()
            .await
            // TODO: Add more context here
            .map_err(|err| ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>))
    }

    /// Query the latest finalized timestamp of this chain.
    // TODO: Use a better timestamp type here
    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn query_latest_timestamp(&self) -> RpcResult<i64> {
        let mut commit_response =
            self.ctx.tm_client.commit(None).await.map_err(|err| {
                ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>)
            })?;

        if commit_response.canonical {
            debug!("commit is not canonical, fetching commit at previous block");
            commit_response = self
                .ctx
                .tm_client
                .commit(Some(
                    (u64::try_from(commit_response.signed_header.header.height.inner() - 1)
                        .expect("should be fine"))
                    .try_into()
                    .expect("should be fine"),
                ))
                .await
                .map_err(|err| {
                    ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>)
                })?;

            if !commit_response.canonical {
                error!(
                    ?commit_response,
                    "commit for previous height is not canonical? continuing \
                    anyways, but this may cause issues downstream"
                );
            }
        }

        Ok(commit_response
            .signed_header
            .header
            .time
            .as_unix_nanos()
            .try_into()
            .expect("should be fine"))
    }

    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn fetch_block_range(
        &self,
        from_height: Height,
        to_height: Height,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        Ok(call(Call::plugin(
            self.ctx.plugin_name(),
            FetchBlocks {
                from_height,
                to_height,
            },
        )))
    }

    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn client_info(&self, client_id: ClientId) -> RpcResult<ClientInfo> {
        match client_id.to_string().rsplit_once('-') {
            Some(("07-tendermint", _)) => Ok(ClientInfo {
                client_type: ClientType::new(ClientType::TENDERMINT),
                ibc_interface: IbcInterface::new(IbcInterface::IBC_GO_V8_NATIVE),
                metadata: Default::default(),
            }),
            Some(("08-wasm", _)) => {
                let checksum = self.ctx.checksum_of_client_id(client_id.clone()).await?;

                Ok(ClientInfo {
                    client_type: match self.ctx.client_type_of_checksum(checksum).await? {
                        Some(ty) => match ty {
                            WasmClientType::EthereumMinimal => {
                                ClientType::new(ClientType::ETHEREUM_MINIMAL)
                            }
                            WasmClientType::EthereumMainnet => {
                                ClientType::new(ClientType::ETHEREUM_MAINNET)
                            }
                            WasmClientType::Cometbls => ClientType::new(ClientType::COMETBLS),
                            WasmClientType::Tendermint => ClientType::new(ClientType::TENDERMINT),
                            WasmClientType::Scroll => ClientType::new(ClientType::SCROLL),
                            WasmClientType::Arbitrum => ClientType::new(ClientType::ARBITRUM),
                            WasmClientType::Linea => todo!(),
                            WasmClientType::Berachain => ClientType::new(ClientType::BEACON_KIT),
                            WasmClientType::Movement => ClientType::new(ClientType::MOVEMENT),
                            WasmClientType::EvmInCosmos => todo!(),
                        },
                        None => {
                            warn!(%client_id, "unknown client type for 08-wasm client");
                            // this early return is kind of dirty but it works
                            return Err(ErrorObject::owned(
                                -1,
                                "unknown client type for 08-wasm client",
                                Some(json!({
                                    "client_id": client_id.to_string()
                                })),
                            ));
                        }
                    },
                    ibc_interface: IbcInterface::new(IbcInterface::IBC_GO_V8_08_WASM),
                    metadata: into_value(IbcGo08WasmClientMetadata { checksum }),
                })
            }
            _ => Err(ErrorObject::owned(
                -1,
                format!("unknown client type (client id `{client_id}`)"),
                Some(json!({
                    "client_id": client_id.to_string()
                })),
            )),
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn query_ibc_state(&self, at: Height, path: Path) -> RpcResult<Value> {
        const IBC_STORE_PATH: &str = "store/ibc/key";

        let path_string = path.to_string();

        let error_data = || Some(json!({ "height": at, "path": path }));

        let query_result = self
            .ctx
            .tm_client
            .abci_query(
                IBC_STORE_PATH,
                &path_string,
                Some(
                    i64::try_from(at.revision_height)
                        .expect("should be fine")
                        .try_into()
                        .expect("invalid height"),
                ),
                false,
            )
            .await
            .map_err(rpc_error(
                format_args!("error fetching abci query"),
                error_data(),
            ))?
            .response;

        // NOTE: At this point, we assume that if the node has given us a response that the data contained within said response is fully reflective of the actual state on-chain, and as such it is a fatal error if we fail to decode it
        type ValueOf<T> = <T as IbcPath>::Value;

        Ok(match path {
            Path::ClientState(_) => into_value::<ValueOf<ClientStatePath>>(Hex(query_result.value)),
            Path::ClientConsensusState(_) => {
                into_value::<ValueOf<ClientConsensusStatePath>>(Hex(query_result.value))
            }
            Path::Connection(_) => {
                into_value::<ValueOf<ConnectionPath>>(if query_result.value.is_empty() {
                    None
                } else {
                    Some(
                        ConnectionEnd::decode_as::<Proto>(&query_result.value).map_err(
                            fatal_rpc_error("error decoding connection end", error_data()),
                        )?,
                    )
                })
            }
            Path::ChannelEnd(_) => {
                into_value::<ValueOf<ChannelEndPath>>(if query_result.value.is_empty() {
                    None
                } else {
                    Some(
                        Channel::decode_as::<Proto>(&query_result.value)
                            .map_err(fatal_rpc_error("error decoding channel end", error_data()))?,
                    )
                })
            }
            Path::Commitment(_) => {
                into_value::<ValueOf<CommitmentPath>>(if query_result.value.is_empty() {
                    None
                } else {
                    Some(
                        H256::try_from(query_result.value)
                            .map_err(fatal_rpc_error("error decoding commitment", error_data()))?,
                    )
                })
            }
            Path::Acknowledgement(_) => {
                into_value::<ValueOf<AcknowledgementPath>>(if query_result.value.is_empty() {
                    None
                } else {
                    Some(H256::try_from(query_result.value).map_err(fatal_rpc_error(
                        "error decoding acknowledgement commitment",
                        error_data(),
                    ))?)
                })
            }
            Path::Receipt(_) => into_value::<ValueOf<ReceiptPath>>(match query_result.value[..] {
                [] => false,
                [1] => true,
                ref invalid => {
                    return Err(fatal_rpc_error("error decoding receipt", error_data())(
                        format!(
                            "value is neither empty nor the single byte 0x01, found {}",
                            serde_utils::to_hex(invalid)
                        ),
                    ))
                }
            }),
            // NOTE: For these branches, we use H64 as a mildly hacky way to have a better error message (since `<[T; N] as TryFrom<Vec<T>>>::Error = Vec<T>`)
            Path::NextSequenceSend(_) => {
                into_value::<ValueOf<NextSequenceSendPath>>(u64::from_be_bytes(
                    H64::try_from(query_result.value)
                        .map_err(fatal_rpc_error(
                            "error decoding next_sequence_send",
                            error_data(),
                        ))?
                        .0,
                ))
            }
            Path::NextSequenceRecv(_) => {
                into_value::<ValueOf<NextSequenceRecvPath>>(u64::from_be_bytes(
                    H64::try_from(query_result.value)
                        .map_err(fatal_rpc_error(
                            "error decoding next_sequence_recv",
                            error_data(),
                        ))?
                        .0,
                ))
            }
            Path::NextSequenceAck(_) => {
                into_value::<ValueOf<NextSequenceAckPath>>(u64::from_be_bytes(
                    H64::try_from(query_result.value)
                        .map_err(fatal_rpc_error(
                            "error decoding next_sequence_ack",
                            error_data(),
                        ))?
                        .0,
                ))
            }
            Path::NextConnectionSequence(_) => {
                into_value::<ValueOf<NextConnectionSequencePath>>(u64::from_be_bytes(
                    H64::try_from(query_result.value)
                        .map_err(fatal_rpc_error(
                            "error decoding next_connection_sequence",
                            error_data(),
                        ))?
                        .0,
                ))
            }
            Path::NextClientSequence(_) => {
                into_value::<ValueOf<NextClientSequencePath>>(u64::from_be_bytes(
                    H64::try_from(query_result.value)
                        .map_err(fatal_rpc_error(
                            "error decoding next_client_sequence",
                            error_data(),
                        ))?
                        .0,
                ))
            }
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn query_ibc_proof(&self, at: Height, path: Path) -> RpcResult<Value> {
        // TODO: This is also in the fn above, move this to somewhere more appropriate (chain-utils perhaps?)

        const IBC_STORE_PATH: &str = "store/ibc/key";

        let path_string = path.to_string();

        let query_result = self
            .ctx
            .tm_client
            .abci_query(
                IBC_STORE_PATH,
                &path_string,
                // a proof at height H is provable at height H + 1
                // we assume that the height passed in to this function is the intended height to prove against, thus we have to query the height - 1
                Some(
                    (i64::try_from(at.revision_height).expect("should be fine") - 1)
                        .try_into()
                        .expect("invalid height"),
                ),
                true,
            )
            .await
            .map_err(rpc_error(
                format_args!("error fetching abci query"),
                Some(json!({ "height": at, "path": path })),
            ))?;

        Ok(into_value(
            MerkleProof::try_from(protos::ibc::core::commitment::v1::MerkleProof {
                proofs: query_result
                    .response
                    .proof_ops
                    .unwrap()
                    .ops
                    .into_iter()
                    .map(|op| {
                        <protos::cosmos::ics23::v1::CommitmentProof as prost::Message>::decode(
                            op.data.as_slice(),
                        )
                        .unwrap()
                    })
                    .collect::<Vec<_>>(),
            })
            .unwrap(),
        ))
    }

    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn query_raw_unfinalized_trusted_client_state(
        &self,
        client_id: ClientId,
    ) -> RpcResult<RawClientState<'static>> {
        let height = self.query_latest_height().await?;

        let client_state = serde_json::from_value::<Hex<Vec<u8>>>(
            self.query_ibc_state(
                height,
                ClientStatePath {
                    client_id: client_id.clone(),
                }
                .into(),
            )
            .await?,
        )
        .expect("infallible");

        let ClientInfo {
            client_type,
            ibc_interface,
            metadata: _,
        } = self.client_info(client_id.clone()).await?;

        Ok(RawClientState {
            client_type,
            ibc_interface,
            bytes: client_state.0.into(),
        })
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

fn fatal_rpc_error<E: Into<Box<dyn Error>>>(
    message: impl Display,
    data: Option<Value>,
) -> impl FnOnce(E) -> ErrorObjectOwned {
    move |e| {
        let e = e.into();
        let message = format!("{message}: {}", ErrorReporter(&*e));
        error!(%message, data = %data.as_ref().unwrap_or(&serde_json::Value::Null));
        ErrorObject::owned(FATAL_JSONRPC_ERROR_CODE, message, data)
    }
}

#[track_caller]
fn into_value<T: Debug + Serialize>(t: T) -> Value {
    match serde_json::to_value(t) {
        Ok(ok) => ok,
        Err(err) => {
            error!(
                error = %ErrorReporter(err),
                "error serializing value of type {}",
                std::any::type_name::<T>()
            );

            panic!(
                "error serializing value of type {}",
                std::any::type_name::<T>()
            );
        }
    }
}

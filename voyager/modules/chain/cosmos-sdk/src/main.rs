use std::{
    collections::VecDeque,
    num::{NonZeroU32, NonZeroU8, ParseIntError},
    sync::Arc,
};

use dashmap::DashMap;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
};
use queue_msg::{call, conc, promise, BoxDynError, Op};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use serde_utils::Hex;
use tracing::{debug, error, info, instrument, warn};
use unionlabs::{
    encoding::{DecodeAs, Proto},
    events::{
        AcknowledgePacket, ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit, ChannelOpenTry,
        ClientMisbehaviour, ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit,
        ConnectionOpenTry, CreateClient, IbcEvent, RecvPacket, SendPacket, SubmitEvidence,
        TimeoutPacket, UpdateClient, WriteAcknowledgement,
    },
    hash::H256,
    ibc::core::{
        channel::channel::Channel, client::height::Height, commitment::merkle_proof::MerkleProof,
        connection::connection_end::ConnectionEnd,
    },
    ics24::{ChannelEndPath, ClientStatePath, ConnectionPath, Path},
    id::{ClientId, ConnectionId},
    option_unwrap, parse_wasm_client_type, ErrorReporter, QueryHeight, WasmClientType,
};
use voyager_message::{
    call::{compound::fetch_client_state_meta, Call, FetchClientInfo, FetchState},
    callback::{
        AggregateDecodeClientStateMetaFromConnection, AggregateFetchClientFromConnection,
        AggregateFetchCounterpartyChannelAndConnection, Callback, InfoOrMeta,
    },
    data::{ClientInfo, Data, IbcState},
    plugin::{
        ChainModuleServer, IbcGo08WasmClientMetadata, PluginInfo, PluginKind, PluginModuleServer,
        RawClientState,
    },
    run_module_server, ChainId, ClientType, IbcInterface, VoyagerMessage,
};

use crate::{
    call::{FetchBlocks, FetchClientFromConnectionId, FetchTransactions, ModuleCall},
    callback::{MakeFullEvent, ModuleCallback},
    data::ModuleData,
};

pub mod call;
pub mod callback;
pub mod data;

const PER_PAGE_LIMIT: NonZeroU8 = option_unwrap!(NonZeroU8::new(10));

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    run_module_server(
        Module::new,
        ChainModuleServer::into_rpc,
        |config, cmd| async move { Module::new(config).await?.cmd(cmd).await },
    )
    .await
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
    pub ws_url: String,
    pub grpc_url: String,
}

impl Module {
    pub async fn cmd(&self, cmd: Cmd) -> Result<(), BoxDynError> {
        match cmd {
            Cmd::ChainId => println!("{}", self.chain_id),
            Cmd::LatestHeight => println!("{}", self.query_latest_height().await?),
        }

        Ok(())
    }

    fn plugin_name(&self) -> String {
        pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

        format!("{PLUGIN_NAME}/{}", self.chain_id)
    }

    pub async fn new(config: Config) -> Result<Self, InitError> {
        let tm_client = cometbft_rpc::Client::new(config.ws_url).await?;

        let chain_id = tm_client.status().await?.node_info.network;

        let chain_revision = chain_id
            .split('-')
            .last()
            .ok_or_else(|| InitError::ChainIdParse {
                found: chain_id.clone(),
                source: None,
            })?
            .parse()
            .map_err(|err| InitError::ChainIdParse {
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

    #[must_use]
    pub fn make_height(&self, height: u64) -> Height {
        Height {
            revision_number: self.chain_revision,
            revision_height: height,
        }
    }

    async fn client_type_of_checksum(&self, checksum: H256) -> Option<WasmClientType> {
        if let Some(ty) = self.checksum_cache.get(&checksum) {
            debug!(
                checksum = %checksum.to_string_unprefixed(),
                ty = ?*ty,
                "cache hit for checksum"
            );

            return Some(*ty);
        };

        info!(
            checksum = %checksum.to_string_unprefixed(),
            "cache miss for checksum"
        );

        let bz = protos::ibc::lightclients::wasm::v1::query_client::QueryClient::connect(
            self.grpc_url.clone(),
        )
        .await
        .unwrap()
        .code(protos::ibc::lightclients::wasm::v1::QueryCodeRequest {
            checksum: checksum.to_string_unprefixed(),
        })
        .await
        .unwrap()
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

                Some(ty)
            }
            Ok(None) => None,
            Err(err) => {
                error!(
                    checksum = %checksum.to_string_unprefixed(),
                    %err,
                    "unable to parse wasm client type"
                );

                None
            }
        }
    }

    async fn checksum_of_client_id(&self, client_id: ClientId) -> H256 {
        let client_state = protos::ibc::core::client::v1::query_client::QueryClient::connect(
            self.grpc_url.clone(),
        )
        .await
        .unwrap()
        .client_state(protos::ibc::core::client::v1::QueryClientStateRequest {
            client_id: client_id.to_string(),
        })
        .await
        .unwrap()
        .into_inner()
        .client_state
        .unwrap();

        assert!(
            client_state.type_url
                == <protos::ibc::lightclients::wasm::v1::ClientState as prost::Name>::type_url()
        );

        // NOTE: We only need the checksum, so we don't need to decode the inner state contained in .data
        <protos::ibc::lightclients::wasm::v1::ClientState as prost::Message>::decode(
            &*client_state.value,
        )
        .unwrap()
        .checksum
        .try_into()
        .unwrap()
    }

    async fn fetch_connection(&self, connection_id: ConnectionId) -> (ConnectionEnd, Height) {
        let inner = protos::ibc::core::connection::v1::query_client::QueryClient::connect(
            self.grpc_url.clone(),
        )
        .await
        .unwrap()
        .connection(protos::ibc::core::connection::v1::QueryConnectionRequest {
            connection_id: connection_id.to_string(),
        })
        .await
        .unwrap()
        .into_inner();

        (
            inner.connection.unwrap().try_into().unwrap(),
            inner.proof_height.unwrap().into(),
        )
    }

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
}

#[derive(Debug, thiserror::Error)]
pub enum InitError {
    #[error("tendermint rpc error")]
    Cometbft(#[from] cometbft_rpc::JsonRpcError),
    #[error(
        "unable to parse chain id: expected format `<chain>-<revision-number>`, found `{found}`"
    )]
    ChainIdParse {
        found: String,
        #[source]
        source: Option<ParseIntError>,
    },
}

#[async_trait]
impl PluginModuleServer<ModuleData, ModuleCall, ModuleCallback> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn info(&self) -> RpcResult<PluginInfo> {
        Ok(PluginInfo {
            name: self.plugin_name(),
            kind: Some(PluginKind::Chain),
            interest_filter: None,
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    fn callback(
        &self,
        cb: ModuleCallback,
        data: VecDeque<Data<ModuleData>>,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        Ok(match cb {
            ModuleCallback::MakeFullEvent(aggregate) => aggregate.do_aggregate(data),
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn call(
        &self,
        msg: ModuleCall,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match msg {
            ModuleCall::FetchTransactions(FetchTransactions { height, page }) => {
                info!(%height, %page, "fetching events in block");

                let response = self
                    .tm_client
                    .tx_search(
                        format!("tx.height={}", height.revision_height),
                        false,
                        page,
                        PER_PAGE_LIMIT,
                        cometbft_rpc::Order::Desc,
                    )
                    .await
                    .unwrap();

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

                            match ibc_event {
                                IbcEvent::SubmitEvidence(SubmitEvidence { .. }) => {
                                    // TODO: Not sure how to handle this one, since it only contains the hash
                                    panic!()
                                }
                                IbcEvent::CreateClient(CreateClient { ref client_id, .. })
                                | IbcEvent::UpdateClient(UpdateClient { ref client_id, .. })
                                | IbcEvent::ClientMisbehaviour(ClientMisbehaviour {
                                    ref client_id,
                                    ..
                                })
                                | IbcEvent::ConnectionOpenInit(ConnectionOpenInit {
                                    ref client_id,
                                    ..
                                })
                                | IbcEvent::ConnectionOpenTry(ConnectionOpenTry {
                                    ref client_id,
                                    ..
                                })
                                | IbcEvent::ConnectionOpenAck(ConnectionOpenAck {
                                    ref client_id,
                                    ..
                                })
                                | IbcEvent::ConnectionOpenConfirm(ConnectionOpenConfirm {
                                    ref client_id,
                                    ..
                                }) => promise(
                                    [
                                        call(FetchClientInfo {
                                            chain_id: self.chain_id.clone(),
                                            client_id: client_id.clone(),
                                        }),
                                        fetch_client_state_meta(
                                            self.chain_id.clone(),
                                            client_id.clone(),
                                            QueryHeight::Specific(height)
                                        )],
                                    [],
                                    Callback::plugin(
                                        self.plugin_name(),
                                        MakeFullEvent {
                                            chain_id: self.chain_id.clone(),
                                            tx_hash,
                                            height,
                                            event: ibc_event,
                                        },
                                    ),
                                ),

                                IbcEvent::ChannelOpenInit(ChannelOpenInit {
                                    ref connection_id,
                                    ..
                                })
                                | IbcEvent::ChannelOpenTry(ChannelOpenTry {
                                    ref connection_id,
                                    ..
                                }) => promise(
                                    [
                                        call(Call::plugin(
                                            self.plugin_name(),
                                            FetchClientFromConnectionId {
                                                connection_id: connection_id.clone(),
                                                fetch_type: InfoOrMeta::Both,
                                            },
                                        )),
                                        call(FetchState {
                                            chain_id: self.chain_id.clone(),
                                            at: QueryHeight::Specific(height),
                                            path: ConnectionPath {
                                                connection_id: connection_id.clone(),
                                            }
                                            .into(),
                                        }),
                                    ],
                                    [],
                                    Callback::plugin(
                                        self.plugin_name(),
                                        MakeFullEvent {
                                            chain_id: self.chain_id.clone(),
                                            tx_hash,
                                            height,
                                            event: ibc_event,
                                        },
                                    ),
                                ),
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
                                }) => promise(
                                    [
                                        call(Call::plugin(
                                            self.plugin_name(),
                                            FetchClientFromConnectionId {
                                                connection_id: connection_id.clone(),
                                                fetch_type: InfoOrMeta::Both,
                                            },
                                        )),
                                        call(FetchState {
                                            chain_id: self.chain_id.clone(),
                                            at: QueryHeight::Specific(height),
                                            path: ConnectionPath {
                                                connection_id: connection_id.clone(),
                                            }
                                            .into(),
                                        }),
                                        call(FetchState {
                                            chain_id: self.chain_id.clone(),
                                            at: QueryHeight::Specific(height),
                                            path: ChannelEndPath {
                                                port_id: port_id.clone(),
                                                channel_id: channel_id.clone(),
                                            }
                                            .into(),
                                        }),
                                    ],
                                    [],
                                    Callback::plugin(
                                        self.plugin_name(),
                                        MakeFullEvent {
                                            chain_id: self.chain_id.clone(),
                                            tx_hash,
                                            height,
                                            event: ibc_event,
                                        },
                                    ),
                                ),

                                // packet origin is this chain
                                IbcEvent::SendPacket(SendPacket {
                                    packet_src_port:    ref self_port,
                                    packet_src_channel: ref self_channel,
                                    packet_dst_port:    ref other_port,
                                    packet_dst_channel: ref other_channel,
                                                        ref connection_id,
                                    ..
                                })
                                | IbcEvent::TimeoutPacket(TimeoutPacket {
                                    packet_src_port:    ref self_port,
                                    packet_src_channel: ref self_channel,
                                    packet_dst_port:    ref other_port,
                                    packet_dst_channel: ref other_channel,
                                                        ref connection_id,
                                    ..
                                })
                                | IbcEvent::AcknowledgePacket(AcknowledgePacket {
                                    packet_src_port:    ref self_port,
                                    packet_src_channel: ref self_channel,
                                    packet_dst_port:    ref other_port,
                                    packet_dst_channel: ref other_channel,
                                    ref connection_id,
                                    ..
                                })
                                // packet origin is the counterparty chain
                                | IbcEvent::WriteAcknowledgement(WriteAcknowledgement {
                                    packet_src_port:    ref other_port,
                                    packet_src_channel: ref other_channel,
                                    packet_dst_port:    ref self_port,
                                    packet_dst_channel: ref self_channel,
                                                        ref connection_id,
                                    ..
                                })
                                | IbcEvent::RecvPacket(RecvPacket {
                                    packet_src_port:    ref other_port,
                                    packet_src_channel: ref other_channel,
                                    packet_dst_port:    ref self_port,
                                    packet_dst_channel: ref self_channel,
                                                        ref connection_id,
                                    ..
                                }) => {
                                    // dbg!(&ibc_event, &self_port, &other_port);

                                    promise(
                                        [
                                            // client underlying the connection on this chain
                                            call(Call::plugin(
                                                self.plugin_name(),
                                                FetchClientFromConnectionId {
                                                    connection_id: connection_id.clone(),
                                                    fetch_type: InfoOrMeta::Both,
                                                },
                                            )),
                                            // channel on this chain
                                            call(FetchState {
                                                chain_id: self.chain_id.clone(),
                                                at: QueryHeight::Specific(height),
                                                path: ChannelEndPath {
                                                    port_id: self_port.clone(),
                                                    channel_id: self_channel.clone(),
                                                }
                                                .into(),
                                            }),
                                            // connection on this chain
                                            call(FetchState {
                                                chain_id: self.chain_id.clone(),
                                                at: QueryHeight::Specific(height),
                                                path: ConnectionPath {
                                                    connection_id: connection_id.clone(),
                                                }
                                                .into(),
                                            }),
                                            // fetching the counterparty channel and connection is a bit trickier - we need the counterparty chain id first, which can then be used to fetch the channel on the counterparty, which then contains the connection id as well.
                                            promise(
                                                [promise(
                                                    [call(FetchState {
                                                        chain_id: self.chain_id.clone(),
                                                        at: QueryHeight::Specific(height),
                                                        path: ConnectionPath {
                                                            connection_id: connection_id.clone(),
                                                        }
                                                        .into()
                                                    })],
                                                    [],
                                                    AggregateDecodeClientStateMetaFromConnection {},
                                                )],
                                                [],
                                                AggregateFetchCounterpartyChannelAndConnection {
                                                    counterparty_port_id: other_port.clone(),
                                                    counterparty_channel_id: other_channel.clone(),
                                                }
                                            ),
                                        ],
                                        [],
                                        Callback::plugin(
                                            self.plugin_name(),
                                            MakeFullEvent {
                                                chain_id: self.chain_id.clone(),
                                                tx_hash,
                                                height,
                                                event: ibc_event,
                                            },
                                        ),
                                    )
                                },
                            }
                        })
                        .chain(
                            ((page.get() * PER_PAGE_LIMIT.get() as u32) < response.total_count)
                                .then(|| {
                                    call(Call::plugin(
                                        self.plugin_name(),
                                        FetchTransactions {
                                            height,
                                            page: page.checked_add(1).expect("too many pages?"),
                                        },
                                    ))
                                }),
                        ),
                ))
            }
            ModuleCall::FetchClientFromConnectionId(FetchClientFromConnectionId {
                connection_id,
                fetch_type,
            }) => {
                let (connection, height) = self.fetch_connection(connection_id.clone()).await;

                Ok(promise(
                    [],
                    [IbcState {
                        chain_id: self.chain_id.clone(),
                        path: ConnectionPath { connection_id },
                        height,
                        state: connection,
                    }
                    .into()],
                    AggregateFetchClientFromConnection { fetch_type },
                ))
            }
            ModuleCall::FetchBlocks(FetchBlocks {
                from_height,
                to_height,
            }) => {
                assert!(from_height.revision_height < to_height.revision_height);

                if to_height.revision_height - from_height.revision_height == 1 {
                    Ok(call(Call::plugin(
                        self.plugin_name(),
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
                            self.plugin_name(),
                            FetchTransactions {
                                height: from_height,
                                page: const { option_unwrap!(NonZeroU32::new(1_u32)) },
                            },
                        ))]
                        .into_iter()
                        .chain((new_from_height != to_height).then(|| {
                            debug!("range not completed, requeueing fetch from {new_from_height} to {to_height}");

                            call(Call::plugin(
                                self.plugin_name(),
                                FetchBlocks {
                                    from_height: new_from_height,
                                    to_height,
                                },
                            ))
                        })),
                    ))
                }
            }
        }
    }
}

#[async_trait]
impl ChainModuleServer<ModuleData, ModuleCall, ModuleCallback> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    fn chain_id(&self) -> RpcResult<ChainId<'static>> {
        Ok(self.chain_id.clone())
    }

    /// Query the latest finalized height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_latest_height(&self) -> RpcResult<Height> {
        match self.tm_client.block(None).await {
            Ok(block_response) => {
                let height = block_response
                    .block
                    .header
                    .height
                    .inner()
                    .try_into()
                    .expect("value is >= 0; qed;");

                debug!(height, "latest height");

                Ok(self.make_height(height))
            }
            Err(err) => Err(ErrorObject::owned(
                -1,
                ErrorReporter(err).to_string(),
                None::<()>,
            )),
        }
    }

    /// Query the latest (non-finalized) height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_latest_height_as_destination(&self) -> RpcResult<Height> {
        self.query_latest_height().await
    }

    /// Query the latest finalized timestamp of this chain.
    // TODO: Use a better timestamp type here
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_latest_timestamp(&self) -> RpcResult<i64> {
        match self.tm_client.block(None).await {
            Ok(block_response) => Ok(block_response
                .block
                .header
                .time
                .as_unix_nanos()
                .try_into()
                .unwrap()),
            Err(err) => Err(ErrorObject::owned(
                -1,
                ErrorReporter(err).to_string(),
                None::<()>,
            )),
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn fetch_block_range(
        &self,
        from_height: Height,
        to_height: Height,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        Ok(call(Call::plugin(
            self.plugin_name(),
            FetchBlocks {
                from_height,
                to_height,
            },
        )))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn client_info(&self, client_id: ClientId) -> RpcResult<ClientInfo> {
        match client_id.to_string().rsplit_once('-') {
            Some(("07-tendermint", _)) => Ok(ClientInfo {
                client_type: ClientType::new(ClientType::TENDERMINT),
                ibc_interface: IbcInterface::new(IbcInterface::IBC_GO_V8_NATIVE),
                metadata: Default::default(),
            }),
            Some(("08-wasm", _)) => {
                let checksum = self.checksum_of_client_id(client_id.clone()).await;

                Ok(ClientInfo {
                    client_type: match self.client_type_of_checksum(checksum).await {
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
                    metadata: serde_json::to_value(IbcGo08WasmClientMetadata { checksum }).unwrap(),
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

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_ibc_state(&self, at: Height, path: Path) -> RpcResult<Value> {
        const IBC_STORE_PATH: &str = "store/ibc/key";

        let path_string = path.to_string();

        let query_result = self
            .tm_client
            .abci_query(
                IBC_STORE_PATH,
                &path_string,
                Some(
                    i64::try_from(at.revision_height)
                        .unwrap()
                        .try_into()
                        .expect("invalid height"),
                ),
                false,
            )
            .await
            .unwrap()
            .response;

        Ok(match path {
            Path::ClientState(_) => serde_json::to_value(Hex(query_result.value)).unwrap(),
            Path::ClientConsensusState(_) => serde_json::to_value(Hex(query_result.value)).unwrap(),
            Path::Connection(_) => serde_json::to_value(
                ConnectionEnd::decode_as::<Proto>(&query_result.value).unwrap(),
            )
            .unwrap(),
            Path::ChannelEnd(_) => {
                serde_json::to_value(Channel::decode_as::<Proto>(&query_result.value).unwrap())
                    .unwrap()
            }
            Path::Commitment(_) => {
                serde_json::to_value(H256::try_from(query_result.value).unwrap()).unwrap()
            }
            Path::Acknowledgement(_) => {
                serde_json::to_value(H256::try_from(query_result.value).unwrap()).unwrap()
            }
            Path::Receipt(_) => serde_json::to_value(match query_result.value[..] {
                [] => false,
                [1] => true,
                ref invalid => panic!("not a bool??? {invalid:?}"),
            })
            .unwrap(),
            Path::NextSequenceSend(_) => {
                serde_json::to_value(u64::from_be_bytes(query_result.value.try_into().unwrap()))
                    .unwrap()
            }
            Path::NextSequenceRecv(_) => {
                serde_json::to_value(u64::from_be_bytes(query_result.value.try_into().unwrap()))
                    .unwrap()
            }
            Path::NextSequenceAck(_) => {
                serde_json::to_value(u64::from_be_bytes(query_result.value.try_into().unwrap()))
                    .unwrap()
            }
            Path::NextConnectionSequence(_) => {
                serde_json::to_value(u64::from_be_bytes(query_result.value.try_into().unwrap()))
                    .unwrap()
            }
            Path::NextClientSequence(_) => {
                serde_json::to_value(u64::from_be_bytes(query_result.value.try_into().unwrap()))
                    .unwrap()
            }
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_ibc_proof(&self, at: Height, path: Path) -> RpcResult<Value> {
        const IBC_STORE_PATH: &str = "store/ibc/key";

        let path_string = path.to_string();

        let query_result = self
            .tm_client
            .abci_query(
                IBC_STORE_PATH,
                &path_string,
                // a proof at height H is provable at height H + 1
                // we assume that the height passed in to this function is the intended height to prove against, thus we have to query the height - 1
                Some(
                    (i64::try_from(at.revision_height).unwrap() - 1)
                        .try_into()
                        .expect("invalid height"),
                ),
                true,
            )
            .await
            .unwrap();

        Ok(serde_json::to_value(
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
        )
        .unwrap())
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
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
        .unwrap();

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

// #![warn(clippy::unwrap_used)]

use std::{
    error::Error,
    fmt::{Debug, Display},
    num::{NonZeroU64, ParseIntError},
    sync::Arc,
};

use cometbft_rpc::types::abci::response_query::QueryResponse;
use dashmap::DashMap;
use futures::{stream::FuturesUnordered, TryStreamExt};
use itertools::Itertools;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::{ErrorObject, ErrorObjectOwned},
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, error, info, instrument, warn};
use unionlabs::{
    bytes::Bytes,
    encoding::{DecodeAs, Proto},
    hash::{hash_v2::HexUnprefixed, H256, H64},
    ibc::core::{
        channel::channel::Channel, client::height::Height, commitment::merkle_proof::MerkleProof,
        connection::connection_end::ConnectionEnd,
    },
    ics24::{
        AcknowledgementPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath,
        CommitmentPath, ConnectionPath, NextClientSequencePath, NextConnectionSequencePath,
        NextSequenceAckPath, NextSequenceRecvPath, NextSequenceSendPath, ReceiptPath,
    },
    id::{ChannelId, ClientId, ConnectionId, PortId},
    parse_wasm_client_type, ErrorReporter, WasmClientType,
};
use voyager_message::{
    core::{ChainId, ClientInfo, ClientType, IbcGo08WasmClientMetadata, IbcInterface, IbcVersion},
    into_value,
    module::{ChainModuleInfo, ChainModuleServer, RawClientState},
    run_chain_module_server, ChainModule, FATAL_JSONRPC_ERROR_CODE,
};
use voyager_vm::BoxDynError;

const IBC_STORE_PATH: &str = "store/ibc/key";

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_chain_module_server::<Module>().await
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

impl ChainModule for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: ChainModuleInfo) -> Result<Self, BoxDynError> {
        let tm_client = cometbft_rpc::Client::new(config.ws_url).await?;

        let chain_id = tm_client.status().await?.node_info.network;

        info.ensure_chain_id(&chain_id)?;

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
}

impl Module {
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

    async fn abci_query(&self, path_string: &str, height: Height) -> RpcResult<QueryResponse> {
        self.tm_client
            .abci_query(
                IBC_STORE_PATH,
                &path_string,
                Some(
                    i64::try_from(height.height())
                        .expect("should be fine")
                        .try_into()
                        .expect("invalid height"),
                ),
                false,
            )
            .await
            .map_err(rpc_error(
                format_args!("error fetching abci query"),
                Some(json!({ "height": height, "path": path_string })),
            ))
            .map(|response| response.response)
    }

    async fn prefix_of_client_id(&self, raw_client_id: u32) -> RpcResult<&'static str> {
        // TODO: Make this a config param
        const KNOWN_PREFIXES: &[&str] = &["07-tendermint", "08-wasm"];

        KNOWN_PREFIXES
            .iter()
            .map(move |prefix| {
                let client_id = ClientId::new_static(prefix, raw_client_id).clone();
                async move {
                    protos::ibc::core::client::v1::query_client::QueryClient::connect(
                        self.grpc_url.clone(),
                    )
                    .await
                    .map_err(rpc_error(
                        "error connecting to grpc server",
                        Some(json!({ "client_id": client_id })),
                    ))?
                    .client_state(protos::ibc::core::client::v1::QueryClientStateRequest {
                        // NOTE: We assume this is a wasm client if we're fetching the checksum
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
                        )(&*Box::<dyn Error>::from(
                            "client state field is empty",
                        ))
                    })
                    .map(|_| prefix)
                }
            })
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<&'static str>>()
            .await?
            .into_iter()
            .exactly_one()
            .map_err(|e| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("error fetching prefix of client id: {e}"),
                    Some(json!({
                        "found_prefixes": e.collect::<Vec<_>>()
                    })),
                )
            })
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
impl ChainModuleServer for Module {
    #[instrument(skip_all, fields(raw_client_id))]
    async fn query_client_prefix(&self, _: &Extensions, raw_client_id: u32) -> RpcResult<String> {
        self.prefix_of_client_id(raw_client_id)
            .await
            .map(|s| s.to_owned())
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn client_info(&self, _: &Extensions, client_id: ClientId) -> RpcResult<ClientInfo> {
        match client_id.to_string().rsplit_once('-') {
            Some(("07-tendermint", _)) => Ok(ClientInfo {
                client_type: ClientType::new(ClientType::TENDERMINT),
                ibc_interface: IbcInterface::new(IbcInterface::IBC_GO_V8_NATIVE),
                ibc_version: IbcVersion::V1_0_0,
                metadata: Default::default(),
            }),
            Some(("08-wasm", _)) => {
                let checksum = self.checksum_of_client_id(client_id.clone()).await?;

                Ok(ClientInfo {
                    client_type: match self.client_type_of_checksum(checksum).await? {
                        Some(ty) => match ty {
                            WasmClientType::Ethereum => ClientType::new(ClientType::ETHEREUM),
                            WasmClientType::Cometbls => {
                                ClientType::new(ClientType::COMETBLS_GROTH16)
                            }
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
                    ibc_version: IbcVersion::V1_0_0,
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

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %client_id))]
    async fn query_client_state(
        &self,
        _: &Extensions,
        height: Height,
        client_id: ClientId,
    ) -> RpcResult<Bytes> {
        let path_string = ClientStatePath { client_id }.to_string();

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(query_result.value.unwrap().into_encoding())
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %client_id, %trusted_height))]
    async fn query_client_consensus_state(
        &self,
        _: &Extensions,
        height: Height,
        client_id: ClientId,
        trusted_height: Height,
    ) -> RpcResult<Bytes> {
        let path_string = ClientConsensusStatePath {
            client_id,
            height: trusted_height,
        }
        .to_string();

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(query_result.value.unwrap().into_encoding())
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %connection_id))]
    async fn query_connection(
        &self,
        _: &Extensions,
        height: Height,
        connection_id: ConnectionId,
    ) -> RpcResult<Option<ConnectionEnd>> {
        let path_string = ConnectionPath { connection_id }.to_string();

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(match query_result.value {
            Some(value) => Some(
                ConnectionEnd::decode_as::<Proto>(&value)
                    .map_err(fatal_rpc_error("error decoding connection end", None))?,
            ),
            None => None,
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %port_id, %channel_id))]
    async fn query_channel(
        &self,
        _: &Extensions,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> RpcResult<Option<Channel>> {
        let path_string = ChannelEndPath {
            channel_id,
            port_id,
        }
        .to_string();

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(match query_result.value {
            Some(value) => Some(
                Channel::decode_as::<Proto>(&value)
                    .map_err(fatal_rpc_error("error decoding channel end", None))?,
            ),
            None => None,
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %port_id, %channel_id, %sequence))]
    async fn query_commitment(
        &self,
        _: &Extensions,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<Option<H256>> {
        let path_string = CommitmentPath {
            port_id,
            channel_id,
            sequence,
        }
        .to_string();

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(match query_result.value {
            Some(value) => Some(
                H256::try_from(value)
                    .map_err(fatal_rpc_error("error decoding commitment", None))?,
            ),
            None => None,
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %port_id, %channel_id, %sequence))]
    async fn query_acknowledgement(
        &self,
        _: &Extensions,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<Option<H256>> {
        let path_string = AcknowledgementPath {
            port_id,
            channel_id,
            sequence,
        }
        .to_string();

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(match query_result.value {
            Some(value) => Some(H256::try_from(value).map_err(fatal_rpc_error(
                "error decoding acknowledgement commitment",
                None,
            ))?),
            None => None,
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %port_id, %channel_id, %sequence))]
    async fn query_receipt(
        &self,
        _: &Extensions,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<bool> {
        let path_string = ReceiptPath {
            port_id,
            channel_id,
            sequence,
        }
        .to_string();

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(match query_result.value {
            None => false,
            Some(b) if b == [1] => true,
            Some(invalid) => {
                return Err(fatal_rpc_error("error decoding receipt", None)(format!(
                    "value is neither empty nor the single byte 0x01, found {}",
                    serde_utils::to_hex(invalid)
                )))
            }
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %port_id, %channel_id))]
    async fn query_next_sequence_send(
        &self,
        _: &Extensions,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> RpcResult<u64> {
        let path_string = NextSequenceSendPath {
            port_id,
            channel_id,
        }
        .to_string();

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(u64::from_be_bytes(
            *<H64>::try_from(query_result.value.unwrap())
                .map_err(fatal_rpc_error("error decoding next_sequence_send", None))?
                .get(),
        ))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %port_id, %channel_id))]
    async fn query_next_sequence_recv(
        &self,
        _: &Extensions,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> RpcResult<u64> {
        let path_string = NextSequenceRecvPath {
            port_id,
            channel_id,
        }
        .to_string();

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(u64::from_be_bytes(
            *<H64>::try_from(query_result.value.unwrap())
                .map_err(fatal_rpc_error("error decoding next_sequence_recv", None))?
                .get(),
        ))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %port_id, %channel_id))]
    async fn query_next_sequence_ack(
        &self,
        _: &Extensions,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> RpcResult<u64> {
        let path_string = NextSequenceAckPath {
            port_id,
            channel_id,
        }
        .to_string();

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(u64::from_be_bytes(
            *<H64>::try_from(query_result.value.unwrap())
                .map_err(fatal_rpc_error("error decoding next_sequence_ack", None))?
                .get(),
        ))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height))]
    async fn query_next_connection_sequence(
        &self,
        _: &Extensions,
        height: Height,
    ) -> RpcResult<u64> {
        let path_string = NextConnectionSequencePath {}.to_string();

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(u64::from_be_bytes(
            *<H64>::try_from(query_result.value.unwrap())
                .map_err(fatal_rpc_error(
                    "error decoding next_connection_sequence",
                    None,
                ))?
                .get(),
        ))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height))]
    async fn query_next_client_sequence(&self, _: &Extensions, height: Height) -> RpcResult<u64> {
        let path_string = NextClientSequencePath {}.to_string();

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(u64::from_be_bytes(
            *<H64>::try_from(query_result.value.unwrap())
                .map_err(fatal_rpc_error("error decoding next_client_sequence", None))?
                .get(),
        ))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_ibc_proof(
        &self,
        _: &Extensions,
        at: Height,
        path: Bytes,
        ibc_version: IbcVersion,
    ) -> RpcResult<Value> {
        let path_string = path.to_string();

        let query_result = self
            .tm_client
            .abci_query(
                IBC_STORE_PATH,
                &path_string,
                // a proof at height H is provable at height H + 1
                // we assume that the height passed in to this function is the intended height to prove against, thus we have to query the height - 1
                Some(
                    (i64::try_from(at.height()).expect("should be fine") - 1)
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

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_raw_unfinalized_trusted_client_state(
        &self,
        _e: &Extensions,
        _client_id: ClientId,
    ) -> RpcResult<RawClientState> {
        todo!()
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

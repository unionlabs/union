// #![warn(clippy::unwrap_used)]

use std::{
    collections::HashMap,
    error::Error,
    fmt::{Debug, Display},
    num::{NonZeroU64, ParseIntError},
    sync::Arc,
};

use clap::{builder::TypedValueParser, value_parser};
use dashmap::DashMap;
use futures::{
    stream::{self, FuturesUnordered},
    StreamExt, TryStreamExt,
};
use itertools::Itertools;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::{ErrorObject, ErrorObjectOwned},
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use serde_utils::Hex;
use tracing::{debug, error, info, instrument, warn};
use unionlabs::{
    encoding::{DecodeAs, Proto},
    hash::{hash_v2::HexUnprefixed, H256, H64},
    ibc::core::{
        channel::channel::Channel, client::height::Height, commitment::merkle_proof::MerkleProof,
        connection::connection_end::ConnectionEnd,
    },
    ics24::{
        AcknowledgementPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath,
        CommitmentPath, ConnectionPath, NextClientSequencePath, NextConnectionSequencePath,
        NextSequenceAckPath, NextSequenceRecvPath, NextSequenceSendPath, Path, ReceiptPath,
    },
    id::{ChannelId, ClientId, ConnectionId, PortId},
    parse_wasm_client_type,
    tendermint::abci::response_query::ResponseQuery,
    ErrorReporter, WasmClientType,
};
use voyager_message::{
    core::{
        ChainId, ClientInfo, ClientType, IbcGo08WasmClientMetadata, IbcInterface, IbcStoreFormat,
    },
    into_value,
    module::{ChainModuleInfo, ChainModuleServer, RawClientState},
    rpc::{ChannelInfo, ConnectionInfo},
    run_chain_module_server,
    valuable::Valuable,
    ChainModule, FATAL_JSONRPC_ERROR_CODE,
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
    PrefixOfClientId {
        #[arg(value_parser = value_parser!(u32).map(ClientId::new))]
        client_id: ClientId,
    },
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
            Some(ty) => {
                info!(
                    %checksum,
                    ?ty,
                    "parsed checksum"
                );

                let ty = match &*ty {
                    "EthereumMinimal" => WasmClientType::EthereumMinimal,
                    "EthereumMainnet" => WasmClientType::EthereumMainnet,
                    "Cometbls" => WasmClientType::Cometbls,
                    "Tendermint" => WasmClientType::Tendermint,
                    "Scroll" => WasmClientType::Scroll,
                    "Arbitrum" => WasmClientType::Arbitrum,
                    "Linea" => WasmClientType::Linea,
                    // TODO: Rename to beacon-kit
                    "Berachain" => WasmClientType::Berachain,
                    "EvmInCosmos" => WasmClientType::EvmInCosmos,
                    "Movement" => WasmClientType::Movement,
                    _ => {
                        warn!("unknown wasm client type `{ty}` for checksum {checksum}");
                        return Ok(None);
                    }
                };

                self.checksum_cache.insert(checksum, ty);

                Ok(Some(ty))
            }
            None => Ok(None),
        }
    }

    async fn prefix_of_client_id(&self, client_id: &ClientId) -> RpcResult<&'static str> {
        // TODO: Make this a config param
        const KNOWN_PREFIXES: &[&str] = &["07-tendermint", "08-wasm"];

        KNOWN_PREFIXES
            .iter()
            .map(move |prefix| {
                let client_id = client_id.clone();
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
                        client_id: client_id.to_string_prefixed("08-wasm"),
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

    async fn port_id_of_channel_id(&self, channel_id: &ChannelId) -> RpcResult<PortId> {
        let client = protos::ibc::core::channel::v1::query_client::QueryClient::connect(
            self.grpc_url.clone(),
        )
        .await
        .map_err(rpc_error("error connecting to grpc server", None))?;

        let all = stream::unfold((client, None), |(mut client, page)| async move {
            let response = client
                .channels(protos::ibc::core::channel::v1::QueryChannelsRequest { pagination: page })
                .await
                .unwrap()
                .into_inner();

            let channels = response.channels;
            let page = response.pagination;

            Some((
                channels
                    .into_iter()
                    .map(|channel| (channel.channel_id, channel.port_id)),
                (
                    client,
                    page.map(|page| protos::cosmos::base::query::v1beta1::PageRequest {
                        key: page.next_key,
                        ..Default::default()
                    }),
                ),
            ))
        })
        .flat_map(stream::iter)
        .collect::<HashMap<_, _>>()
        .await;

        Ok(all
            .get(&channel_id.to_string_prefixed())
            .unwrap()
            .parse()
            .unwrap())
    }

    #[instrument(skip_all, fields(client_id = client_id.as_value()))]
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
            // NOTE: We assume this is a wasm client if we're fetching the checksum
            client_id: client_id.to_string_prefixed("08-wasm"),
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

    async fn abci_query(&self, path_string: &str, height: Height) -> RpcResult<ResponseQuery> {
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
    /// Query the latest finalized height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_latest_height(&self, _: &Extensions) -> RpcResult<Height> {
        self.latest_height()
            .await
            // TODO: Add more context here
            .map_err(|err| ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>))
    }

    /// Query the latest (non-finalized) height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_latest_height_as_destination(&self, _: &Extensions) -> RpcResult<Height> {
        self.latest_height()
            .await
            // TODO: Add more context here
            .map_err(|err| ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>))
    }

    /// Query the latest finalized timestamp of this chain.
    // TODO: Use a better timestamp type here
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_latest_timestamp(&self, _: &Extensions) -> RpcResult<i64> {
        let mut commit_response =
            self.tm_client.commit(None).await.map_err(|err| {
                ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>)
            })?;

        if commit_response.canonical {
            debug!("commit is not canonical, fetching commit at previous block");
            commit_response = self
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

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn client_info(&self, _: &Extensions, client_id: ClientId) -> RpcResult<ClientInfo> {
        let prefix = self.prefix_of_client_id(&client_id).await?;

        match prefix {
            "07-tendermint" => Ok(ClientInfo {
                client_type: ClientType::new(ClientType::TENDERMINT),
                ibc_interface: IbcInterface::new(IbcInterface::IBC_GO_V8_NATIVE),
                metadata: Default::default(),
            }),
            "08-wasm" => {
                let checksum = self.checksum_of_client_id(client_id.clone()).await?;

                Ok(ClientInfo {
                    client_type: match self.client_type_of_checksum(checksum).await? {
                        Some(ty) => match ty {
                            WasmClientType::EthereumMinimal => {
                                ClientType::new(ClientType::ETHEREUM_MINIMAL)
                            }
                            WasmClientType::EthereumMainnet => {
                                ClientType::new(ClientType::ETHEREUM_MAINNET)
                            }
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
                            warn!(
                                client_id = client_id.as_value(),
                                "unknown client type for 08-wasm client"
                            );
                            // this early return is kind of dirty but it works
                            return Err(ErrorObject::owned(
                                -1,
                                "unknown client type for 08-wasm client",
                                Some(json!({
                                    "client_id": client_id.to_string_prefixed(prefix)
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
                format!(
                    "unknown client type (prefix `{prefix}`, id {})",
                    client_id.id()
                ),
                Some(json!({
                    "client_id": client_id
                })),
            )),
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, client_id = client_id.as_value()))]
    async fn query_client_state(
        &self,
        _: &Extensions,
        height: Height,
        client_id: ClientId,
    ) -> RpcResult<Hex<Vec<u8>>> {
        let prefix = self.prefix_of_client_id(&client_id).await?;
        let path_string = ClientStatePath { client_id }.ics24_commitment_path(prefix);

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(Hex(query_result.value))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, client_id = client_id.as_value(), %trusted_height))]
    async fn query_client_consensus_state(
        &self,
        _: &Extensions,
        height: Height,
        client_id: ClientId,
        trusted_height: Height,
    ) -> RpcResult<Hex<Vec<u8>>> {
        let prefix = self.prefix_of_client_id(&client_id).await?;
        let path_string = ClientConsensusStatePath {
            client_id,
            height: trusted_height,
        }
        .ics24_commitment_path(prefix);

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(Hex(query_result.value))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, connection_id = connection_id.as_value()))]
    async fn query_connection(
        &self,
        _: &Extensions,
        height: Height,
        connection_id: ConnectionId,
    ) -> RpcResult<Option<ConnectionInfo>> {
        let path_string = ConnectionPath { connection_id }.ics24_commitment_path();

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(if query_result.value.is_empty() {
            None
        } else {
            Some(
                ConnectionEnd::decode_as::<Proto>(&query_result.value)
                    .map_err(fatal_rpc_error("error decoding connection end", None))?,
            )
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, channel_id = channel_id.as_value()))]
    async fn query_channel(
        &self,
        _: &Extensions,
        height: Height,
        channel_id: ChannelId,
    ) -> RpcResult<Option<ChannelInfo>> {
        let port_id = self.port_id_of_channel_id(&channel_id).await?;
        let path_string = ChannelEndPath { channel_id }.ics24_commitment_path(&port_id);

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(if query_result.value.is_empty() {
            None
        } else {
            let channel = Channel::decode_as::<Proto>(&query_result.value)
                .map_err(fatal_rpc_error("error decoding channel end", None))?;
            Some(ChannelInfo {
                port_id,
                state: channel.state,
                ordering: channel.ordering,
                counterparty_channel_id: channel.counterparty.channel_id,
                connection_hops: channel.connection_hops,
                version: channel.version,
            })
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, channel_id = channel_id.as_value(), %sequence))]
    async fn query_commitment(
        &self,
        _: &Extensions,
        height: Height,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<Option<H256>> {
        let port_id = self.port_id_of_channel_id(&channel_id).await?;
        let path_string = CommitmentPath {
            channel_id,
            sequence,
        }
        .ics24_commitment_path(&port_id);

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(if query_result.value.is_empty() {
            None
        } else {
            Some(
                H256::try_from(query_result.value)
                    .map_err(fatal_rpc_error("error decoding commitment", None))?,
            )
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, channel_id = channel_id.as_value(), %sequence))]
    async fn query_acknowledgement(
        &self,
        _: &Extensions,
        height: Height,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<Option<H256>> {
        let port_id = self.port_id_of_channel_id(&channel_id).await?;
        let path_string = AcknowledgementPath {
            channel_id,
            sequence,
        }
        .ics24_commitment_path(&port_id);

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(if query_result.value.is_empty() {
            None
        } else {
            Some(H256::try_from(query_result.value).map_err(fatal_rpc_error(
                "error decoding acknowledgement commitment",
                None,
            ))?)
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, channel_id = channel_id.as_value(), %sequence))]
    async fn query_receipt(
        &self,
        _: &Extensions,
        height: Height,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<bool> {
        let port_id = self.port_id_of_channel_id(&channel_id).await?;
        let path_string = ReceiptPath {
            channel_id,
            sequence,
        }
        .ics24_commitment_path(&port_id);

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(match query_result.value[..] {
            [] => false,
            [1] => true,
            ref invalid => {
                return Err(fatal_rpc_error("error decoding receipt", None)(format!(
                    "value is neither empty nor the single byte 0x01, found {}",
                    serde_utils::to_hex(invalid)
                )))
            }
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, channel_id = channel_id.as_value()))]
    async fn query_next_sequence_send(
        &self,
        _: &Extensions,
        height: Height,
        channel_id: ChannelId,
    ) -> RpcResult<u64> {
        let port_id = self.port_id_of_channel_id(&channel_id).await?;
        let path_string = NextSequenceSendPath { channel_id }.ics24_commitment_path(&port_id);

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(u64::from_be_bytes(
            *<H64>::try_from(query_result.value)
                .map_err(fatal_rpc_error("error decoding next_sequence_send", None))?
                .get(),
        ))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, channel_id = channel_id.as_value()))]
    async fn query_next_sequence_recv(
        &self,
        _: &Extensions,
        height: Height,
        channel_id: ChannelId,
    ) -> RpcResult<u64> {
        let port_id = self.port_id_of_channel_id(&channel_id).await?;
        let path_string = NextSequenceRecvPath { channel_id }.ics24_commitment_path(&port_id);

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(u64::from_be_bytes(
            *<H64>::try_from(query_result.value)
                .map_err(fatal_rpc_error("error decoding next_sequence_recv", None))?
                .get(),
        ))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, channel_id = channel_id.as_value()))]
    async fn query_next_sequence_ack(
        &self,
        _: &Extensions,
        height: Height,
        channel_id: ChannelId,
    ) -> RpcResult<u64> {
        let port_id = self.port_id_of_channel_id(&channel_id).await?;
        let path_string = NextSequenceAckPath { channel_id }.ics24_commitment_path(&port_id);

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(u64::from_be_bytes(
            *<H64>::try_from(query_result.value)
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
        let path_string = NextConnectionSequencePath {}.ics24_commitment_path();

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(u64::from_be_bytes(
            *<H64>::try_from(query_result.value)
                .map_err(fatal_rpc_error(
                    "error decoding next_connection_sequence",
                    None,
                ))?
                .get(),
        ))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height))]
    async fn query_next_client_sequence(&self, _: &Extensions, height: Height) -> RpcResult<u64> {
        let path_string = NextClientSequencePath {}.ics24_commitment_path();

        let query_result = self.abci_query(&path_string, height).await?;

        Ok(u64::from_be_bytes(
            *<H64>::try_from(query_result.value)
                .map_err(fatal_rpc_error("error decoding next_client_sequence", None))?
                .get(),
        ))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_ibc_proof(
        &self,
        _: &Extensions,
        at: Height,
        path: Path,
        ibc_store_format: IbcStoreFormat<'static>,
    ) -> RpcResult<Value> {
        let path_string = match &path {
            Path::ClientState(path) => {
                path.ics24_commitment_path(self.prefix_of_client_id(&path.client_id).await?)
            }
            Path::ClientConsensusState(path) => {
                path.ics24_commitment_path(self.prefix_of_client_id(&path.client_id).await?)
            }
            Path::Connection(path) => path.ics24_commitment_path(),
            Path::ChannelEnd(path) => {
                let port_id = self.port_id_of_channel_id(&path.channel_id).await?;
                path.ics24_commitment_path(&port_id)
            }
            Path::Commitment(path) => {
                let port_id = self.port_id_of_channel_id(&path.channel_id).await?;
                path.ics24_commitment_path(&port_id)
            }
            Path::Acknowledgement(path) => {
                let port_id = self.port_id_of_channel_id(&path.channel_id).await?;
                path.ics24_commitment_path(&port_id)
            }
            Path::Receipt(path) => {
                let port_id = self.port_id_of_channel_id(&path.channel_id).await?;
                path.ics24_commitment_path(&port_id)
            }
            Path::NextSequenceSend(path) => {
                let port_id = self.port_id_of_channel_id(&path.channel_id).await?;
                path.ics24_commitment_path(&port_id)
            }
            Path::NextSequenceRecv(path) => {
                let port_id = self.port_id_of_channel_id(&path.channel_id).await?;
                path.ics24_commitment_path(&port_id)
            }
            Path::NextSequenceAck(path) => {
                let port_id = self.port_id_of_channel_id(&path.channel_id).await?;
                path.ics24_commitment_path(&port_id)
            }
            Path::NextConnectionSequence(path) => path.ics24_commitment_path(),
            Path::NextClientSequence(path) => path.ics24_commitment_path(),
        };

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
        e: &Extensions,
        client_id: ClientId,
    ) -> RpcResult<RawClientState<'static>> {
        // let height = self.query_latest_height(e).await?;

        // let client_state = serde_json::from_value::<Hex<Vec<u8>>>(
        //     self.query_ibc_state(
        //         e,
        //         height,
        //         ClientStatePath {
        //             client_id: client_id.clone(),
        //         }
        //         .into(),
        //     )
        //     .await?,
        // )
        // .expect("infallible");

        // let ClientInfo {
        //     client_type,
        //     ibc_interface,
        //     metadata: _,
        // } = self.client_info(e, client_id.clone()).await?;

        // Ok(RawClientState {
        //     client_type,
        //     ibc_interface,
        //     bytes: client_state.0.into(),
        // })

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

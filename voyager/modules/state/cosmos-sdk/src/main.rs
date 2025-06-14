// #![warn(clippy::unwrap_used)]

use std::{
    error::Error,
    fmt::{Debug, Display},
    num::{NonZeroU32, NonZeroU64, NonZeroU8, ParseIntError},
};

use cometbft_rpc::{rpc_types::Order, types::abci::response_query::QueryResponse};
use cosmos_sdk_event::CosmosSdkEvent;
use futures::{stream::FuturesUnordered, TryStreamExt};
use ibc_classic_spec::{
    AcknowledgementPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath, CommitmentPath,
    ConnectionPath, IbcClassic, NextClientSequencePath, NextConnectionSequencePath,
    NextSequenceAckPath, NextSequenceRecvPath, NextSequenceSendPath, ReceiptPath,
    StorePath as ClassicStorePath,
};
use ibc_union_spec::{
    path::StorePath as UnionStorePath,
    query::{PacketByHash, PacketsByBatchHash, Query},
    Channel as UnionChannel, ChannelId as UnionChannelId, ClientId as UnionClientId,
    Connection as UnionConnection, ConnectionId as UnionConnectionId, IbcUnion, Packet, Timestamp,
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::{ErrorObject, ErrorObjectOwned},
    Extensions,
};
use protos::cosmwasm::wasm::v1::{QuerySmartContractStateRequest, QuerySmartContractStateResponse};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, instrument, trace};
use unionlabs::{
    bech32::Bech32,
    encoding::{DecodeAs, Proto},
    ibc::core::{
        channel::channel::Channel, client::height::Height,
        connection::connection_end::ConnectionEnd,
    },
    id::{ChannelId, ClientId, ConnectionId, PortId},
    never::Never,
    option_unwrap,
    primitives::{Bytes, H256, H64},
    ErrorReporter,
};
use voyager_sdk::{
    anyhow, into_value,
    plugin::StateModule,
    primitives::{ChainId, ClientInfo, ClientType, IbcInterface, IbcSpec, IbcSpecId},
    rpc::{types::StateModuleInfo, StateModuleServer, FATAL_JSONRPC_ERROR_CODE},
};

const IBC_STORE_PATH: &str = "store/ibc/key";

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    <Module as StateModule<SupportedIbcSpec>>::run().await;
}

#[derive(clap::Subcommand)]
pub enum Cmd {
    ChainId,
    LatestHeight,
}

#[derive(Debug, Clone, PartialEq, Copy, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum SupportedIbcSpec {
    IbcClassic,
    IbcUnion,
}

impl TryFrom<String> for SupportedIbcSpec {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match &*value {
            "ibc-classic" => Ok(SupportedIbcSpec::IbcClassic),
            "ibc-union" => Ok(SupportedIbcSpec::IbcUnion),
            _ => Err(format!("unsupported IBC spec: `{value}`")),
        }
    }
}

impl SupportedIbcSpec {
    fn as_str(&self) -> &'static str {
        match self {
            SupportedIbcSpec::IbcClassic => "ibc-classic",
            SupportedIbcSpec::IbcUnion => "ibc-union",
        }
    }
}

impl From<SupportedIbcSpec> for String {
    fn from(value: SupportedIbcSpec) -> Self {
        value.as_str().to_owned()
    }
}

impl IbcSpec for SupportedIbcSpec {
    type ClientId = ClientId;
    type Query = Never;
    type StorePath = Never;
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,
    pub chain_revision: u64,
    pub ibc_spec: SupportedIbcSpec,

    pub cometbft_client: cometbft_rpc::Client,
    
    // Optional field for Union IBC
    pub ibc_host_contract_address: Option<Bech32<H256>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub rpc_url: String,
    #[serde(default = "default_max_drift")]
    pub max_drift: u64,
    /// Required for Union IBC, optional for Classic IBC
    pub ibc_host_contract_address: Option<Bech32<H256>>,
}

fn default_max_drift() -> u64 {
    10
}

impl StateModule<SupportedIbcSpec> for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: StateModuleInfo) -> anyhow::Result<Self> {
        let cometbft_client = cometbft_rpc::Client::new(config.rpc_url).await?;

        let chain_id = cometbft_client.status().await?.node_info.network;

        info.ensure_chain_id(&chain_id)?;

        let ibc_spec = SupportedIbcSpec::try_from(info.ibc_spec_id.to_string())
            .map_err(|e| anyhow::anyhow!(e))?;

        // Validate configuration based on IBC spec
        match ibc_spec {
            SupportedIbcSpec::IbcClassic => {
                if config.ibc_host_contract_address.is_some() {
                    return Err(anyhow::anyhow!(
                        "ibc_host_contract_address should not be provided for Classic IBC"
                    ));
                }
            }
            SupportedIbcSpec::IbcUnion => {
                if config.ibc_host_contract_address.is_none() {
                    return Err(anyhow::anyhow!(
                        "ibc_host_contract_address is required for Union IBC"
                    ));
                }
            }
        }

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
            cometbft_client,
            chain_id: ChainId::new(chain_id),
            chain_revision,
            ibc_spec,
            ibc_host_contract_address: config.ibc_host_contract_address,
        })
    }
}

impl Module {
    #[must_use]
    pub fn make_height(&self, height: u64) -> Height {
        Height::new_with_revision(self.chain_revision, height)
    }

    // Classic IBC methods
    async fn abci_query(&self, path_string: &str, height: Height) -> RpcResult<QueryResponse> {
        self.cometbft_client
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

    // Union IBC methods
    #[instrument(skip_all, fields(?height))]
    pub async fn query_smart<Q: Serialize, R: DeserializeOwned>(
        &self,
        query: &Q,
        height: Option<Height>,
    ) -> RpcResult<Option<R>> {
        let ibc_host_contract_address = self.ibc_host_contract_address.as_ref()
            .ok_or_else(|| ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                "ibc_host_contract_address not configured for Union IBC",
                None::<()>,
            ))?;

        let query_data = serde_json::to_string(query).expect("serialization is infallible; qed;");
        let response = self
            .cometbft_client
            .grpc_abci_query::<_, QuerySmartContractStateResponse>(
                "/cosmwasm.wasm.v1.Query/SmartContractState",
                &QuerySmartContractStateRequest {
                    address: ibc_host_contract_address.to_string(),
                    query_data: query_data.clone().into_bytes(),
                },
                height.map(|height| {
                    i64::try_from(height.height())
                        .expect("should be fine")
                        .try_into()
                        .expect("invalid height")
                }),
                false,
            )
            .await
            .map_err(rpc_error(
                "error fetching abci query",
                Some(json!({
                    "height": height,
                    "query_data": query_data
                })),
            ))?;

        // https://github.com/cosmos/cosmos-sdk/blob/e2027bf62893bb5f82e8f7a8ea59d1a43eb6b78f/baseapp/abci.go#L1272-L1278
        if response
            .code
            .is_err_code(option_unwrap!(NonZeroU32::new(26)))
        {
            Err(ErrorObject::owned(
                -1,
                "attempted to query state at a nonexistent height, \
                potentially due to load balanced rpc endpoints",
                Some(json!({
                    "height": height,
                    "query_data": query_data
                })),
            ))
        } else {
            response
                .value
                .map(|value| {
                    trace!("raw response: {}", String::from_utf8_lossy(&value.data));
                    serde_json::from_slice(&value.data).map_err(|e| {
                        ErrorObject::owned(
                            -1,
                            ErrorReporter(e).with_message(&format!(
                                "unable to deserialize response ({})",
                                std::any::type_name::<R>()
                            )),
                            None::<()>,
                        )
                    })
                })
                .transpose()
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %channel_id, %packet_hash))]
    pub async fn query_packet_by_hash(
        &self,
        channel_id: UnionChannelId,
        packet_hash: H256,
    ) -> RpcResult<Packet> {
        if !matches!(self.ibc_spec, SupportedIbcSpec::IbcUnion) {
            return Err(ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                "query_packet_by_hash is only supported for Union IBC",
                None::<()>,
            ));
        }

        let ibc_host_contract_address = self.ibc_host_contract_address.as_ref()
            .ok_or_else(|| ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                "ibc_host_contract_address not configured",
                None::<()>,
            ))?;

        let query = format!("wasm-packet_send.packet_hash='{packet_hash}' AND wasm-packet_send.channel_id={channel_id}");

        let mut res = self
            .cometbft_client
            .tx_search(
                query,
                false,
                option_unwrap!(NonZeroU32::new(1)),
                option_unwrap!(NonZeroU8::new(1)),
                Order::Asc,
            )
            .await
            .map_err(rpc_error("error querying packet by packet hash", None))?;

        if res.total_count != 1 {
            return Err(ErrorObject::owned(
                -1,
                format!(
                    "error querying for packet {packet_hash}, \
                    expected 1 event but found {}",
                    res.total_count,
                ),
                None::<()>,
            ));
        }

        let res = res.txs.pop().unwrap();

        let Some(IbcEvent::WasmPacketSend {
            packet_source_channel_id,
            packet_destination_channel_id,
            packet_data,
            packet_timeout_height,
            packet_timeout_timestamp,
            channel_id: _,
            packet_hash: _,
        }) = res.tx_result.events.into_iter().find_map(|event| {
            CosmosSdkEvent::<IbcEvent>::new(event).ok().and_then(|e| {
                (e.contract_address.unwrap() == *ibc_host_contract_address).then_some(e.event)
            })
        })
        else {
            panic!()
        };

        Ok(Packet {
            source_channel_id: packet_source_channel_id,
            destination_channel_id: packet_destination_channel_id,
            data: packet_data,
            timeout_height: packet_timeout_height,
            timeout_timestamp: packet_timeout_timestamp,
        })
    }

    // Client state queries
    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %client_id))]
    async fn query_client_state(&self, height: Height, client_id: ClientId) -> RpcResult<Bytes> {
        match self.ibc_spec {
            SupportedIbcSpec::IbcClassic => {
                let path_string = ClientStatePath { client_id }.to_string();
                let query_result = self.abci_query(&path_string, height).await?;
                Ok(query_result.value.unwrap().into_encoding())
            }
            SupportedIbcSpec::IbcUnion => {
                let client_id = UnionClientId::new(client_id.to_string().parse::<u32>()
                    .map_err(|e| rpc_error("invalid client id", None)(e))?);
                let client_state = self
                    .query_smart::<_, Bytes>(
                        &ibc_union_msg::query::QueryMsg::GetClientState { client_id },
                        Some(height),
                    )
                    .await?;
                
                client_state.ok_or_else(|| {
                    ErrorObject::owned(
                        -1,
                        format!("client state not found for client {client_id}"),
                        None::<()>,
                    )
                }).map(Into::into)
            }
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %client_id, %trusted_height))]
    async fn query_client_consensus_state(
        &self,
        height: Height,
        client_id: ClientId,
        trusted_height: Height,
    ) -> RpcResult<Bytes> {
        match self.ibc_spec {
            SupportedIbcSpec::IbcClassic => {
                let path_string = ClientConsensusStatePath {
                    client_id,
                    height: trusted_height,
                }
                .to_string();
                let query_result = self.abci_query(&path_string, height).await?;
                Ok(query_result.value.unwrap().into_encoding())
            }
            SupportedIbcSpec::IbcUnion => {
                let client_id = UnionClientId::new(client_id.to_string().parse::<u32>()
                    .map_err(|e| rpc_error("invalid client id", None)(e))?);
                let consensus_state = self
                    .query_smart::<_, Bytes>(
                        &ibc_union_msg::query::QueryMsg::GetConsensusState {
                            client_id,
                            height: trusted_height.height(),
                        },
                        Some(height),
                    )
                    .await?;
                
                consensus_state.ok_or_else(|| {
                    ErrorObject::owned(
                        -1,
                        format!("consensus state not found for client {client_id} at height {trusted_height}"),
                        None::<()>,
                    )
                }).map(Into::into)
            }
        }
    }

    // Connection queries
    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %connection_id))]
    async fn query_connection(
        &self,
        height: Height,
        connection_id: ConnectionId,
    ) -> RpcResult<Option<Value>> {
        match self.ibc_spec {
            SupportedIbcSpec::IbcClassic => {
                let path_string = ConnectionPath { connection_id }.to_string();
                let query_result = self.abci_query(&path_string, height).await?;
                
                Ok(match query_result.value {
                    Some(value) => {
                        let connection_end = ConnectionEnd::decode_as::<Proto>(&value)
                            .map_err(fatal_rpc_error("error decoding connection end", None))?;
                        Some(into_value(connection_end))
                    }
                    None => None,
                })
            }
            SupportedIbcSpec::IbcUnion => {
                let connection_id = UnionConnectionId::new(connection_id.to_string().parse::<u32>()
                    .map_err(|e| rpc_error("invalid connection id", None)(e))?);
                let connection = self
                    .query_smart::<_, UnionConnection>(
                        &ibc_union_msg::query::QueryMsg::GetConnection { connection_id },
                        Some(height),
                    )
                    .await?;
                
                Ok(connection.map(|c| into_value(c)))
            }
        }
    }

    // Channel queries
    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %port_id, %channel_id))]
    async fn query_channel(
        &self,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> RpcResult<Option<Value>> {
        match self.ibc_spec {
            SupportedIbcSpec::IbcClassic => {
                let path_string = ChannelEndPath {
                    channel_id,
                    port_id,
                }
                .to_string();
                let query_result = self.abci_query(&path_string, height).await?;
                
                Ok(match query_result.value {
                    Some(value) => {
                        let channel = Channel::decode_as::<Proto>(&value)
                            .map_err(fatal_rpc_error("error decoding channel end", None))?;
                        Some(into_value(channel))
                    }
                    None => None,
                })
            }
            SupportedIbcSpec::IbcUnion => {
                let channel_id = UnionChannelId::new(channel_id.to_string().parse::<u32>()
                    .map_err(|e| rpc_error("invalid channel id", None)(e))?);
                let channel = self
                    .query_smart::<_, UnionChannel>(
                        &ibc_union_msg::query::QueryMsg::GetChannel { channel_id },
                        Some(height),
                    )
                    .await?;
                
                Ok(channel.map(|c| into_value(c)))
            }
        }
    }

    // Commitment queries (Classic IBC only)
    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %port_id, %channel_id, %sequence))]
    async fn query_commitment(
        &self,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<Option<H256>> {
        match self.ibc_spec {
            SupportedIbcSpec::IbcClassic => {
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
            SupportedIbcSpec::IbcUnion => {
                Err(ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    "commitment queries not supported for Union IBC",
                    None::<()>,
                ))
            }
        }
    }

    // Acknowledgement queries (Classic IBC only)
    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %port_id, %channel_id, %sequence))]
    async fn query_acknowledgement(
        &self,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<Option<H256>> {
        match self.ibc_spec {
            SupportedIbcSpec::IbcClassic => {
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
            SupportedIbcSpec::IbcUnion => {
                Err(ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    "acknowledgement queries not supported for Union IBC",
                    None::<()>,
                ))
            }
        }
    }

    // Receipt queries (Classic IBC only)
    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %port_id, %channel_id, %sequence))]
    async fn query_receipt(
        &self,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<bool> {
        match self.ibc_spec {
            SupportedIbcSpec::IbcClassic => {
                let path_string = ReceiptPath {
                    port_id,
                    channel_id,
                    sequence,
                }
                .to_string();
                let query_result = self.abci_query(&path_string, height).await?;
                
                Ok(query_result.value.is_some())
            }
            SupportedIbcSpec::IbcUnion => {
                Err(ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    "receipt queries not supported for Union IBC",
                    None::<()>,
                ))
            }
        }
    }

    // Sequence queries (Classic IBC only)
    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %port_id, %channel_id))]
    async fn query_next_sequence_send(
        &self,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> RpcResult<u64> {
        match self.ibc_spec {
            SupportedIbcSpec::IbcClassic => {
                let path_string = NextSequenceSendPath {
                    port_id,
                    channel_id,
                }
                .to_string();
                let query_result = self.abci_query(&path_string, height).await?;
                
                Ok(u64::from_be_bytes(
                    query_result.value.unwrap().try_into()
                        .map_err(fatal_rpc_error("error decoding next sequence send", None))?
                ))
            }
            SupportedIbcSpec::IbcUnion => {
                Err(ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    "sequence queries not supported for Union IBC",
                    None::<()>,
                ))
            }
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %port_id, %channel_id))]
    async fn query_next_sequence_recv(
        &self,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> RpcResult<u64> {
        match self.ibc_spec {
            SupportedIbcSpec::IbcClassic => {
                let path_string = NextSequenceRecvPath {
                    port_id,
                    channel_id,
                }
                .to_string();
                let query_result = self.abci_query(&path_string, height).await?;
                
                Ok(u64::from_be_bytes(
                    query_result.value.unwrap().try_into()
                        .map_err(fatal_rpc_error("error decoding next sequence recv", None))?
                ))
            }
            SupportedIbcSpec::IbcUnion => {
                Err(ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    "sequence queries not supported for Union IBC",
                    None::<()>,
                ))
            }
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %port_id, %channel_id))]
    async fn query_next_sequence_ack(
        &self,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> RpcResult<u64> {
        match self.ibc_spec {
            SupportedIbcSpec::IbcClassic => {
                let path_string = NextSequenceAckPath {
                    port_id,
                    channel_id,
                }
                .to_string();
                let query_result = self.abci_query(&path_string, height).await?;
                
                Ok(u64::from_be_bytes(
                    query_result.value.unwrap().try_into()
                        .map_err(fatal_rpc_error("error decoding next sequence ack", None))?
                ))
            }
            SupportedIbcSpec::IbcUnion => {
                Err(ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    "sequence queries not supported for Union IBC",
                    None::<()>,
                ))
            }
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height))]
    async fn query_next_connection_sequence(&self, height: Height) -> RpcResult<u64> {
        match self.ibc_spec {
            SupportedIbcSpec::IbcClassic => {
                let path_string = NextConnectionSequencePath {}.to_string();
                let query_result = self.abci_query(&path_string, height).await?;
                
                Ok(u64::from_be_bytes(
                    query_result.value.unwrap().try_into()
                        .map_err(fatal_rpc_error("error decoding next connection sequence", None))?
                ))
            }
            SupportedIbcSpec::IbcUnion => {
                Err(ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    "connection sequence queries not supported for Union IBC",
                    None::<()>,
                ))
            }
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height))]
    async fn query_next_client_sequence(&self, height: Height) -> RpcResult<u64> {
        match self.ibc_spec {
            SupportedIbcSpec::IbcClassic => {
                let path_string = NextClientSequencePath {}.to_string();
                let query_result = self.abci_query(&path_string, height).await?;
                
                Ok(u64::from_be_bytes(
                    query_result.value.unwrap().try_into()
                        .map_err(fatal_rpc_error("error decoding next client sequence", None))?
                ))
            }
            SupportedIbcSpec::IbcUnion => {
                Err(ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    "client sequence queries not supported for Union IBC",
                    None::<()>,
                ))
            }
        }
    }

    // Union IBC specific queries
    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %batch_hash))]
    async fn query_batch_packets(
        &self,
        height: Height,
        batch_hash: H256,
    ) -> RpcResult<Option<H256>> {
        match self.ibc_spec {
            SupportedIbcSpec::IbcUnion => {
                self.query_smart::<_, H256>(
                    &ibc_union_msg::query::QueryMsg::GetBatchPackets { batch_hash },
                    Some(height),
                )
                .await
            }
            SupportedIbcSpec::IbcClassic => {
                Err(ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    "batch packet queries not supported for Classic IBC",
                    None::<()>,
                ))
            }
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %batch_hash))]
    async fn query_batch_receipts(
        &self,
        height: Height,
        batch_hash: H256,
    ) -> RpcResult<Option<H256>> {
        match self.ibc_spec {
            SupportedIbcSpec::IbcUnion => {
                self.query_smart::<_, H256>(
                    &ibc_union_msg::query::QueryMsg::GetBatchReceipts { batch_hash },
                    Some(height),
                )
                .await
            }
            SupportedIbcSpec::IbcClassic => {
                Err(ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    "batch receipt queries not supported for Classic IBC",
                    None::<()>,
                ))
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub struct ChainIdParseError {
    found: String,
    #[source]
    source: Option<ParseIntError>,
}

impl Display for ChainIdParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unable to parse chain revision from {}", self.found)
    }
}

#[async_trait]
impl StateModuleServer<SupportedIbcSpec> for Module {
    #[instrument(skip_all)]
    async fn query(&self, _: &Extensions, query: Value) -> RpcResult<Value> {
        match self.ibc_spec {
            SupportedIbcSpec::IbcClassic => {
                let _query: Never = serde_json::from_value(query).map_err(|err| {
                    ErrorObject::owned(FATAL_JSONRPC_ERROR_CODE, err.to_string(), None::<()>)
                })?;
                Ok(Value::Null)
            }
            SupportedIbcSpec::IbcUnion => {
                let query: Query = serde_json::from_value(query).map_err(|err| {
                    ErrorObject::owned(FATAL_JSONRPC_ERROR_CODE, err.to_string(), None::<()>)
                })?;

                match query {
                    Query::PacketByHash(PacketByHash {
                        channel_id,
                        packet_hash,
                    }) => {
                        let packet = self.query_packet_by_hash(channel_id, packet_hash).await?;
                        Ok(into_value(packet))
                    }
                    Query::PacketsByBatchHash(PacketsByBatchHash { batch_hash, channel_id }) => {
                        let packets = self
                            .query_smart::<_, Vec<Packet>>(
                                &ibc_union_msg::query::QueryMsg::GetBatchPackets { batch_hash },
                                None,
                            )
                            .await?;
                        Ok(into_value(packets))
                    }
                }
            }
        }
    }

    #[instrument(skip_all)]
    async fn client_info(&self, _: &Extensions, client_id: ClientId) -> RpcResult<ClientInfo> {
        let latest_height = self
            .cometbft_client
            .status()
            .await
            .map_err(rpc_error("error fetching latest height", None))?
            .sync_info
            .latest_block_height;

        let height = self.make_height(latest_height.into());

        let client_state = self.query_client_state(height, client_id).await?;

        match self.ibc_spec {
            SupportedIbcSpec::IbcClassic => {
                // For classic IBC, we need to determine the client type from the client state
                // This is a simplified approach - in practice, you might need more sophisticated logic
                Ok(ClientInfo {
                    client_type: ClientType::new(ClientType::TENDERMINT.to_string()),
                    ibc_interface: IbcInterface::new(IbcInterface::IBC_GO_V8_NATIVE.to_string()),
                    metadata: Value::Null,
                })
            }
            SupportedIbcSpec::IbcUnion => {
                let union_client_id = UnionClientId::new(client_id.to_string().parse::<u32>()
                    .map_err(|e| rpc_error("invalid client id", None)(e))?);
                let client_type = self
                    .query_smart::<_, String>(
                        &ibc_union_msg::query::QueryMsg::GetClientType {
                            client_id: union_client_id,
                        },
                        Some(height),
                    )
                    .await?
                    .unwrap_or_else(|| "unknown".to_string());

                Ok(ClientInfo {
                    client_type: ClientType::new(client_type),
                    ibc_interface: IbcInterface::new(IbcInterface::IBC_COSMWASM.to_string()),
                    metadata: Value::Null,
                })
            }
        }
    }

    #[instrument(skip_all)]
    async fn query_ibc_state(
        &self,
        _: &Extensions,
        at: Height,
        path: Value,
    ) -> RpcResult<Value> {
        match self.ibc_spec {
            SupportedIbcSpec::IbcClassic => {
                let path: ClassicStorePath = serde_json::from_value(path).map_err(|err| {
                    ErrorObject::owned(FATAL_JSONRPC_ERROR_CODE, err.to_string(), None::<()>)
                })?;

                match path {
                    ClassicStorePath::ClientState(client_state_path) => {
                        let client_state = self
                            .query_client_state(at, client_state_path.client_id)
                            .await?;
                        Ok(into_value(client_state))
                    }
                    ClassicStorePath::ClientConsensusState(consensus_state_path) => {
                        let consensus_state = self
                            .query_client_consensus_state(
                                at,
                                consensus_state_path.client_id,
                                consensus_state_path.height,
                            )
                            .await?;
                        Ok(into_value(consensus_state))
                    }
                    ClassicStorePath::Connection(connection_path) => {
                        let connection = self
                            .query_connection(at, connection_path.connection_id)
                            .await?;
                        Ok(into_value(connection))
                    }
                    ClassicStorePath::ChannelEnd(channel_path) => {
                        let channel = self
                            .query_channel(at, channel_path.port_id, channel_path.channel_id)
                            .await?;
                        Ok(into_value(channel))
                    }
                    ClassicStorePath::Commitment(commitment_path) => {
                        let commitment = self
                            .query_commitment(
                                at,
                                commitment_path.port_id,
                                commitment_path.channel_id,
                                commitment_path.sequence,
                            )
                            .await?;
                        Ok(into_value(commitment))
                    }
                    ClassicStorePath::Acknowledgement(ack_path) => {
                        let ack = self
                            .query_acknowledgement(
                                at,
                                ack_path.port_id,
                                ack_path.channel_id,
                                ack_path.sequence,
                            )
                            .await?;
                        Ok(into_value(ack))
                    }
                    ClassicStorePath::Receipt(receipt_path) => {
                        let receipt = self
                            .query_receipt(
                                at,
                                receipt_path.port_id,
                                receipt_path.channel_id,
                                receipt_path.sequence,
                            )
                            .await?;
                        Ok(into_value(receipt))
                    }
                    ClassicStorePath::NextSequenceSend(seq_path) => {
                        let sequence = self
                            .query_next_sequence_send(at, seq_path.port_id, seq_path.channel_id)
                            .await?;
                        Ok(into_value(sequence))
                    }
                    ClassicStorePath::NextSequenceRecv(seq_path) => {
                        let sequence = self
                            .query_next_sequence_recv(at, seq_path.port_id, seq_path.channel_id)
                            .await?;
                        Ok(into_value(sequence))
                    }
                    ClassicStorePath::NextSequenceAck(seq_path) => {
                        let sequence = self
                            .query_next_sequence_ack(at, seq_path.port_id, seq_path.channel_id)
                            .await?;
                        Ok(into_value(sequence))
                    }
                    ClassicStorePath::NextConnectionSequence(_) => {
                        let sequence = self.query_next_connection_sequence(at).await?;
                        Ok(into_value(sequence))
                    }
                    ClassicStorePath::NextClientSequence(_) => {
                        let sequence = self.query_next_client_sequence(at).await?;
                        Ok(into_value(sequence))
                    }
                }
            }
            SupportedIbcSpec::IbcUnion => {
                let path: UnionStorePath = serde_json::from_value(path).map_err(|err| {
                    ErrorObject::owned(FATAL_JSONRPC_ERROR_CODE, err.to_string(), None::<()>)
                })?;

                match path {
                    UnionStorePath::ClientState(_) => {
                        // Union client state query - placeholder for now
                        Ok(Value::Null)
                    }
                    UnionStorePath::ConsensusState(_) => {
                        // Union consensus state query - placeholder for now
                        Ok(Value::Null)
                    }
                    UnionStorePath::Connection(_) => {
                        // Union connection query - placeholder for now
                        Ok(Value::Null)
                    }
                    UnionStorePath::Channel(_) => {
                        // Union channel query - placeholder for now
                        Ok(Value::Null)
                    }
                    UnionStorePath::BatchPackets(batch_path) => {
                        let batch_packets = self
                            .query_batch_packets(at, batch_path.batch_hash)
                            .await?;
                        Ok(into_value(batch_packets))
                    }
                    UnionStorePath::BatchReceipts(batch_path) => {
                        let batch_receipts = self
                            .query_batch_receipts(at, batch_path.batch_hash)
                            .await?;
                        Ok(into_value(batch_receipts))
                    }
                }
            }
        }
    }
}

fn rpc_error<E: Error>(
    message: impl Display,
    data: Option<Value>,
) -> impl FnOnce(E) -> ErrorObjectOwned {
    move |err| {
        error!("{message}: {}", ErrorReporter(err));
        ErrorObject::owned(-1, message.to_string(), data)
    }
}

fn fatal_rpc_error<E: Into<Box<dyn Error>>>(
    message: impl Display,
    data: Option<Value>,
) -> impl FnOnce(E) -> ErrorObjectOwned {
    move |err| {
        error!("{message}: {}", ErrorReporter(&err));
        ErrorObject::owned(FATAL_JSONRPC_ERROR_CODE, message.to_string(), data)
    }
}

// Union IBC Events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IbcEvent {
    #[serde(rename = "wasm-packet_send")]
    WasmPacketSend {
        #[serde(with = "serde_utils::string")]
        packet_source_channel_id: UnionChannelId,
        #[serde(with = "serde_utils::string")]
        packet_destination_channel_id: UnionChannelId,
        packet_data: Bytes,
        #[serde(with = "serde_utils::string")]
        packet_timeout_height: u64,
        #[serde(with = "serde_utils::string")]
        packet_timeout_timestamp: Timestamp,
        #[serde(with = "serde_utils::string")]
        channel_id: UnionChannelId,
        packet_hash: H256,
    },
    #[serde(rename = "wasm-batch_send")]
    WasmBatchSend {
        #[serde(with = "serde_utils::string")]
        channel_id: UnionChannelId,
        packet_hash: H256,
        batch_hash: H256,
    },
}

// #![warn(clippy::unwrap_used)]

use std::num::{NonZeroU8, NonZeroU32, ParseIntError};

use cometbft_rpc::rpc_types::Order;
use cosmos_sdk_event::CosmosSdkEvent;
use futures::{TryFutureExt, TryStreamExt, stream::FuturesUnordered};
use ibc_union_spec::{
    Channel, ChannelId, ClientId, Connection, ConnectionId, IbcUnion, MustBeZero, Packet, Status,
    Timestamp,
    path::StorePath,
    query::{
        ClientStatus, PacketAckByHash, PacketAckByHashResponse, PacketByHash, PacketByHashResponse,
        PacketsByBatchHash, PacketsByBatchHashResponse, Query,
    },
};
use jsonrpsee::{
    Extensions,
    core::{RpcResult, async_trait},
    types::ErrorObject,
};
use protos::cosmwasm::wasm::v1::{QuerySmartContractStateRequest, QuerySmartContractStateResponse};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use tracing::{debug, error, instrument, trace};
use unionlabs::{
    ErrorReporter,
    ibc::core::client::height::Height,
    option_unwrap,
    primitives::{Bech32, Bytes, H256, encoding::HexUnprefixed},
};
use voyager_sdk::{
    anyhow, into_value,
    plugin::StateModule,
    primitives::{ChainId, ClientInfo, ClientType, IbcInterface, IbcSpec},
    rpc::{FATAL_JSONRPC_ERROR_CODE, StateModuleServer, rpc_error, types::StateModuleInfo},
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    <Module as StateModule<IbcUnion>>::run().await;
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

    pub ibc_host_contract_address: Bech32<H256>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub rpc_url: String,
    pub ibc_host_contract_address: Bech32<H256>,
}

impl StateModule<IbcUnion> for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: StateModuleInfo) -> anyhow::Result<Self> {
        let cometbft_client = cometbft_rpc::Client::new(config.rpc_url).await?;

        let chain_id = cometbft_client.status().await?.node_info.network;

        info.ensure_chain_id(&chain_id)?;
        info.ensure_ibc_spec_id(IbcUnion::ID.as_str())?;

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
            ibc_host_contract_address: config.ibc_host_contract_address,
        })
    }
}

impl Module {
    #[must_use]
    pub fn make_height(&self, height: u64) -> Height {
        Height::new_with_revision(self.chain_revision, height)
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %channel_id, %packet_hash))]
    pub async fn query_packet_by_hash(
        &self,
        channel_id: ChannelId,
        packet_hash: H256,
    ) -> RpcResult<PacketByHashResponse> {
        let query = format!(
            "wasm-packet_send.packet_hash='{packet_hash}' AND wasm-packet_send.channel_id={channel_id}"
        );

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
            packet_timeout_height: _,
            packet_timeout_timestamp,
            channel_id: _,
            packet_hash: _,
        }) = res.tx_result.events.into_iter().find_map(|event| {
            CosmosSdkEvent::<IbcEvent>::new(event).ok().and_then(|e| {
                (e.contract_address.unwrap() == self.ibc_host_contract_address).then_some(e.event)
            })
        })
        else {
            panic!()
        };

        Ok(PacketByHashResponse {
            packet: Packet {
                source_channel_id: packet_source_channel_id,
                destination_channel_id: packet_destination_channel_id,
                data: packet_data,
                timeout_height: MustBeZero,
                timeout_timestamp: packet_timeout_timestamp,
            },
            tx_hash: res.hash.into_encoding(),
            provable_height: res.height.unwrap().get() + 1,
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %channel_id, %packet_hash))]
    pub async fn query_packet_ack_by_hash(
        &self,
        channel_id: ChannelId,
        packet_hash: H256,
    ) -> RpcResult<PacketAckByHashResponse> {
        let query = format!(
            "wasm-write_ack.packet_hash='{packet_hash}' AND wasm-write_ack.channel_id={channel_id}"
        );

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
                    "error querying for acknowledgement for \
                     packet {packet_hash}, expected 1 event \
                     but found {}",
                    res.total_count,
                ),
                None::<()>,
            ));
        }

        let res = res.txs.pop().unwrap();

        let Some(IbcEvent::WasmWriteAck {
            channel_id: _,
            packet_hash: _,
            acknowledgement,
        }) = res.tx_result.events.into_iter().find_map(|event| {
            CosmosSdkEvent::<IbcEvent>::new(event).ok().and_then(|e| {
                (e.contract_address.unwrap() == self.ibc_host_contract_address).then_some(e.event)
            })
        })
        else {
            panic!()
        };

        Ok(PacketAckByHashResponse {
            ack: acknowledgement.into_encoding(),
            tx_hash: res.hash.into_encoding(),
            provable_height: res.height.unwrap().get() + 1,
        })
    }

    #[instrument(skip_all, fields(?height))]
    pub async fn query_smart<Q: Serialize, R: DeserializeOwned>(
        &self,
        query: &Q,
        height: Option<Height>,
    ) -> RpcResult<Option<R>> {
        let query_data = serde_json::to_string(query).expect("serialization is infallible; qed;");
        let response = self
            .cometbft_client
            .grpc_abci_query::<_, QuerySmartContractStateResponse>(
                "/cosmwasm.wasm.v1.Query/SmartContractState",
                &QuerySmartContractStateRequest {
                    address: self.ibc_host_contract_address.to_string(),
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

    #[instrument(
        skip_all,
        fields(
            chain_id = %self.chain_id,
            %height,
            %client_id,
        )
    )]
    async fn query_client_state(
        &self,
        height: Height,
        client_id: ClientId,
    ) -> RpcResult<Option<Bytes>> {
        let client_state = self
            .query_smart::<_, Bytes>(
                &ibc_union_msg::query::QueryMsg::GetClientState { client_id },
                Some(height),
            )
            .await?;

        Ok(client_state.map(Bytes::into_encoding))
    }

    #[instrument(
        skip_all,
            fields(
            chain_id = %self.chain_id,
            %height,
            %client_id,
            %trusted_height
        )
    )]
    async fn query_consensus_state(
        &self,
        height: Height,
        client_id: ClientId,
        trusted_height: u64,
    ) -> RpcResult<Option<Bytes>> {
        let client_state = self
            .query_smart::<_, Bytes>(
                &ibc_union_msg::query::QueryMsg::GetConsensusState {
                    client_id,
                    height: trusted_height,
                },
                Some(height),
            )
            .await?;

        Ok(client_state.map(Bytes::into_encoding))
    }

    #[instrument(
        skip_all,
        fields(
            chain_id = %self.chain_id,
            %height,
            %connection_id
        )
    )]
    async fn query_connection(
        &self,
        height: Height,
        connection_id: ConnectionId,
    ) -> RpcResult<Option<Connection>> {
        let client_state = self
            .query_smart::<_, Connection>(
                &ibc_union_msg::query::QueryMsg::GetConnection { connection_id },
                Some(height),
            )
            .await?;

        Ok(client_state)
    }

    #[instrument(
        skip_all,
        fields(
            chain_id = %self.chain_id,
            %height,
            %channel_id
        )
    )]
    async fn query_channel(
        &self,
        height: Height,
        channel_id: ChannelId,
    ) -> RpcResult<Option<Channel>> {
        let channel = self
            .query_smart::<_, Channel>(
                &ibc_union_msg::query::QueryMsg::GetChannel { channel_id },
                Some(height),
            )
            .await?;

        Ok(channel)
    }

    #[instrument(
        skip_all,
        fields(
            chain_id = %self.chain_id,
            %height,
            %batch_hash
        )
    )]
    async fn query_batch_packets(
        &self,
        height: Height,
        batch_hash: H256,
    ) -> RpcResult<Option<H256>> {
        let commitment = self
            .query_smart::<_, Option<H256>>(
                &ibc_union_msg::query::QueryMsg::GetBatchPackets { batch_hash },
                Some(height),
            )
            .await?;

        Ok(commitment.flatten())
    }

    #[instrument(
        skip_all,
        fields(
            chain_id = %self.chain_id,
            %height,
            %batch_hash
        )
    )]
    async fn query_batch_receipts(
        &self,
        height: Height,
        batch_hash: H256,
    ) -> RpcResult<Option<H256>> {
        let commitment = self
            .query_smart::<_, Option<H256>>(
                &ibc_union_msg::query::QueryMsg::GetBatchReceipts { batch_hash },
                Some(height),
            )
            .await?;

        Ok(commitment.flatten())
    }

    #[instrument(
        skip_all,
        fields(
            chain_id = %self.chain_id,
            %height,
            %client_id,
            %proof_height,
            %path,
        )
    )]
    async fn query_membership_proof(
        &self,
        height: Height,
        client_id: ClientId,
        proof_height: u64,
        path: Bytes,
    ) -> RpcResult<Option<H256>> {
        let commitment = self
            .query_smart::<_, Option<H256>>(
                &ibc_union_msg::query::QueryMsg::GetCommittedMembershipProof {
                    client_id,
                    proof_height,
                    path,
                },
                Some(height),
            )
            .await?;

        Ok(commitment.flatten())
    }

    #[instrument(
        skip_all,
        fields(
            chain_id = %self.chain_id,
            %height,
            %client_id,
            %proof_height,
            %path,
        )
    )]
    async fn query_non_membership_proof(
        &self,
        height: Height,
        client_id: ClientId,
        proof_height: u64,
        path: Bytes,
    ) -> RpcResult<bool> {
        let commitment = self
            .query_smart::<_, bool>(
                &ibc_union_msg::query::QueryMsg::GetCommittedNonMembershipProof {
                    client_id,
                    proof_height,
                    path,
                },
                Some(height),
            )
            .await?;

        Ok(commitment.unwrap_or_default())
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
impl StateModuleServer<IbcUnion> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query(&self, _: &Extensions, query: Query) -> RpcResult<Value> {
        match query {
            Query::PacketByHash(PacketByHash {
                channel_id,
                packet_hash,
            }) => self
                .query_packet_by_hash(channel_id, packet_hash)
                .await
                .map(into_value),
            Query::PacketsByBatchHash(PacketsByBatchHash {
                channel_id,
                batch_hash,
            }) => {
                let query = format!(
                    "wasm-batch_send.batch_hash='{batch_hash}' AND wasm-batch_send.channel_id={channel_id}"
                );

                let res = self
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
                            "error querying for batch {batch_hash}, \
                            expected only 1 transaction but found {}",
                            res.total_count,
                        ),
                        None::<()>,
                    ));
                }

                let tx = res.txs[1].clone();

                if tx.tx_result.events.len() < 2 {
                    return Err(ErrorObject::owned(
                        -1,
                        format!(
                            "error querying for batch {batch_hash}, \
                            expected at least 2 events but found {}",
                            tx.tx_result.events.len()
                        ),
                        None::<()>,
                    ));
                }

                let packets = tx
                    .tx_result
                    .events
                    .into_iter()
                    .filter_map(|event| {
                        CosmosSdkEvent::<IbcEvent>::new(event).ok().and_then(|e| {
                            (e.contract_address.unwrap() == self.ibc_host_contract_address)
                                .then_some(e.event)
                        })
                    })
                    .map(|event| match event {
                        IbcEvent::WasmBatchSend {
                            channel_id,
                            packet_hash,
                            batch_hash: _,
                        } => self
                            .query_packet_by_hash(channel_id, packet_hash)
                            .map_ok(|res| res.packet),
                        _ => panic!(),
                    })
                    .collect::<FuturesUnordered<_>>()
                    .try_collect::<Vec<_>>()
                    .await?;

                Ok(into_value(PacketsByBatchHashResponse {
                    packets,
                    tx_hash: tx.hash.into_encoding(),
                    provable_height: tx.height.unwrap().get() + 1,
                }))
            }
            Query::PacketAckByHash(PacketAckByHash {
                channel_id,
                packet_hash,
            }) => self
                .query_packet_ack_by_hash(channel_id, packet_hash)
                .await
                .map(into_value),
            Query::ClientStatus(ClientStatus { client_id, height }) => {
                let status = self
                    .query_smart::<_, Status>(
                        &ibc_union_msg::query::QueryMsg::GetStatus { client_id },
                        height.map(Height::new),
                    )
                    .await?
                    .ok_or(ErrorObject::owned(
                        FATAL_JSONRPC_ERROR_CODE,
                        format!("client {client_id} not found at height {height:?}"),
                        None::<()>,
                    ))?;

                debug!(
                    %status,
                    %client_id,
                    height = height.map(|h| h.to_string()),
                    "fetched client status"
                );

                Ok(into_value(status))
            }
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn client_info(&self, _: &Extensions, client_id: ClientId) -> RpcResult<ClientInfo> {
        let client_type = self
            .query_smart::<_, String>(
                &ibc_union_msg::query::QueryMsg::GetClientType { client_id },
                None,
            )
            .await?
            .ok_or(ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                format!("client `{client_id}` not found"),
                None::<()>,
            ))?;

        Ok(ClientInfo {
            client_type: ClientType::new(client_type),
            ibc_interface: IbcInterface::new(IbcInterface::IBC_COSMWASM),
            metadata: Value::Null,
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_ibc_state(
        &self,
        _: &Extensions,
        at: Height,
        path: StorePath,
    ) -> RpcResult<Value> {
        match path {
            StorePath::ClientState(path) => self
                .query_client_state(at, path.client_id)
                .await
                .map(into_value),
            StorePath::ConsensusState(path) => self
                .query_consensus_state(at, path.client_id, path.height)
                .await
                .map(into_value),
            StorePath::Connection(path) => self
                .query_connection(at, path.connection_id)
                .await
                .map(into_value),
            StorePath::Channel(path) => self
                .query_channel(at, path.channel_id)
                .await
                .map(into_value),
            StorePath::BatchPackets(path) => self
                .query_batch_packets(at, path.batch_hash)
                .await
                .map(into_value),
            StorePath::BatchReceipts(path) => self
                .query_batch_receipts(at, path.batch_hash)
                .await
                .map(into_value),
            StorePath::MembershipProof(path) => self
                .query_membership_proof(at, path.client_id, path.proof_height, path.path)
                .await
                .map(into_value),
            StorePath::NonMembershipProof(path) => self
                .query_non_membership_proof(at, path.client_id, path.proof_height, path.path)
                .await
                .map(into_value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "attributes")]
pub enum IbcEvent {
    #[serde(rename = "wasm-packet_send")]
    WasmPacketSend {
        #[serde(with = "serde_utils::string")]
        packet_source_channel_id: ChannelId,
        #[serde(with = "serde_utils::string")]
        packet_destination_channel_id: ChannelId,
        packet_data: Bytes,
        #[serde(with = "serde_utils::string")]
        packet_timeout_height: u64,
        #[serde(with = "serde_utils::string")]
        packet_timeout_timestamp: Timestamp,
        #[serde(with = "serde_utils::string")]
        channel_id: ChannelId,
        packet_hash: H256,
    },
    #[serde(rename = "wasm-write_ack")]
    WasmWriteAck {
        #[serde(with = "serde_utils::string")]
        channel_id: ChannelId,
        packet_hash: H256,
        acknowledgement: Bytes<HexUnprefixed>,
    },
    #[serde(rename = "wasm-batch_send")]
    WasmBatchSend {
        #[serde(with = "serde_utils::string")]
        channel_id: ChannelId,
        packet_hash: H256,
        batch_hash: H256,
    },
}

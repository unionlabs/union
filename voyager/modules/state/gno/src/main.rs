#![doc = include_str!("../README.md")]
#![warn(clippy::unwrap_used)]

use std::fmt::Display;

use ibc_union_spec::{
    Channel, ChannelId, ClientId, Connection, ConnectionId, IbcUnion, MustBeZero, Packet, Status,
    path::{BatchPacketsPath, BatchReceiptsPath, StorePath},
    query::{
        ClientStatus, PacketAckByHash, PacketAckByHashResponse, PacketByHash, PacketByHashResponse,
        PacketsByBatchHash, Query,
    },
};
use jsonrpsee::{Extensions, core::async_trait};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tracing::{debug, instrument};
use unionlabs::{
    encoding::{DecodeAs, EthAbi},
    ibc::core::client::height::Height,
    primitives::{
        Bytes, H256,
        encoding::{Base64, HexUnprefixed},
    },
};
use voyager_event_source_plugin_gno::ibc_events::IbcEvent;
use voyager_sdk::{
    anyhow, into_value,
    plugin::StateModule,
    primitives::{ChainId, ClientInfo, ClientType, IbcInterface, IbcSpec},
    rpc::{RpcError, RpcErrorExt, RpcResult, StateModuleServer, types::StateModuleInfo},
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await;
}

#[derive(clap::Subcommand)]
pub enum Cmd {
    ChainId,
    LatestHeight,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub gno_client: gno_rpc::Client,
    pub tx_indexer_client: reqwest::Client,
    pub tx_indexer_rpc_url: String,

    pub ibc_core_realm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub rpc_url: String,
    pub tx_indexer_rpc_url: String,
    pub ibc_core_realm: String,
}

impl StateModule<IbcUnion> for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: StateModuleInfo) -> anyhow::Result<Self> {
        let gno_client = gno_rpc::Client::new(config.rpc_url).await?;

        let chain_id = gno_client.status(None).await?.node_info.network;

        info.ensure_chain_id(&chain_id)?;
        info.ensure_ibc_spec_id(IbcUnion::ID.as_str())?;

        Ok(Self {
            gno_client,
            tx_indexer_client: reqwest::Client::new(),
            tx_indexer_rpc_url: config.tx_indexer_rpc_url,
            chain_id: ChainId::new(chain_id),
            ibc_core_realm: config.ibc_core_realm,
        })
    }
}

impl Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id, %channel_id, %packet_hash))]
    pub async fn query_packet_by_hash(
        &self,
        channel_id: ChannelId,
        packet_hash: H256,
    ) -> RpcResult<PacketByHashResponse> {
        let query = format!(
            r#"query getEvents {{
  getTransactions(
    where: {{
      success: {{ eq: true }},
      response: {{
        events: {{
          _and: [
            {{
              GnoEvent: {{
                type: {{ eq: "PacketSend" }}
                pkg_path: {{ eq: "{}" }},
                attrs: {{
                  key: {{ eq: "source_channel_id" }}
                  value: {{ eq: "{channel_id}" }}
                }}
              }}
          	}}
            {{
              GnoEvent: {{
                type: {{ eq: "PacketSend" }}
                pkg_path: {{ eq: "gno.land/r/core/ibc/v1/core" }},
                attrs: {{
                  key: {{ eq:"packet_hash" }}
                  value: {{ eq:"{packet_hash}" }}
                }}
              }}
          	}}
          ]
        }}
      }}
    }}
  ) {{
    block_height
    hash
    response {{
      events {{
        ... on GnoEvent {{
          type
          pkg_path
          attrs {{
            key
            value
          }}
        }}
      }}
    }}
  }}
}}"#,
            self.ibc_core_realm
        );

        println!("{query}");

        let res = self
            .tx_indexer_client
            .post(&self.tx_indexer_rpc_url)
            .json(&json!({
                "operationName": "getEvents",
                "query": query
            }))
            .send()
            .await
            .map_err(RpcError::retryable("error sending graphql query"))?
            .json::<Value>()
            .await
            .map_err(RpcError::retryable(
                "invalid json returned from graphql query",
            ))?;

        let hash = res
            .pointer("/data/getTransactions/0/hash")
            .ok_or_else(|| RpcError::fatal_from_message("no tx hash in graphql response"))?
            .as_str()
            .ok_or_else(|| {
                RpcError::fatal_from_message("tx hash in graphql response is not a string")
            })?
            .parse::<H256<Base64>>()
            .map_err(RpcError::fatal("invalid tx hash in graphql response"))?;

        let height = res
            .pointer("/data/getTransactions/0/block_height")
            .ok_or_else(|| RpcError::fatal_from_message("no block height in graphql response"))?
            .as_number()
            .ok_or_else(|| {
                RpcError::fatal_from_message("block height in graphql response is not a number")
            })?
            .as_u64()
            .ok_or_else(|| {
                RpcError::fatal_from_message("block height in graphql response is not a u64")
            })?;

        let raw_event = res
            .pointer("/data/getTransactions/0/response/events/0")
            .ok_or_else(|| {
                RpcError::fatal_from_message("no response events in graphql response")
            })?;

        let event = serde_json::from_value(raw_event.clone())
            .map_err(RpcError::fatal("invalid attrs in graphql response"))?;

        let IbcEvent::PacketSend {
            packet_data,
            source_channel_id,
            destination_channel_id,
            timeout_timestamp,
            ..
        } = IbcEvent::from_gno_event(event)?.ok_or_else(|| {
            RpcError::fatal_from_message(
                "invalid graphql response: invalid event returned from query",
            )
        })?
        else {
            return Err(RpcError::fatal_from_message(
                "invalid graphql response: unexpected event returned from query",
            ));
        };

        Ok(PacketByHashResponse {
            packet: Packet {
                source_channel_id,
                destination_channel_id,
                data: packet_data,
                timeout_height: MustBeZero,
                timeout_timestamp,
            },
            tx_hash: Some(hash.into_encoding()),
            provable_height: height,
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %channel_id, %packet_hash))]
    pub async fn query_packet_ack_by_hash(
        &self,
        channel_id: ChannelId,
        packet_hash: H256,
    ) -> RpcResult<PacketAckByHashResponse> {
        // let query = format!(
        //     "wasm-write_ack.packet_hash='{packet_hash}' AND wasm-write_ack.channel_id={channel_id}"
        // );

        // let mut tx_result = self
        //     .gno_client
        //     .tx_search(
        //         &query,
        //         false,
        //         const { NonZeroU32::new(1).unwrap() },
        //         const { NonZeroU8::new(1).unwrap() },
        //         Order::Asc,
        //     )
        //     .await
        //     .map_err(RpcError::retryable(
        //         "error querying packet ack by packet hash in tx",
        //     ))?;

        // if tx_result.total_count != 1 {
        //     return Err(RpcError::retryable_from_message(format!(
        //         "error querying for acknowledgement for packet \
        //          {packet_hash}, expected 1 tx but found {}",
        //         tx_result.total_count,
        //     ))
        //     .with_data(json!({ "tx_result": tx_result })));
        // }

        // let tx = tx_result.txs.pop().expect("len is > 1; qed;");

        // let extract_event = |event: &gno_rpc::types::abci::event::Event| {
        //     CosmosSdkEvent::<IbcEvent>::new(event.clone())
        //         .ok()
        //         .and_then(|e| match e.event {
        //             IbcEvent::WasmWriteAck {
        //                 channel_id: found_channel_id,
        //                 packet_hash: found_packet_hash,
        //                 acknowledgement,
        //             } => (channel_id == found_channel_id
        //                 && packet_hash == found_packet_hash
        //                 && e.contract_address
        //                     .is_some_and(|a| a == self.ibc_host_contract_address))
        //             .then_some(acknowledgement),
        //             _ => None,
        //         })
        // };

        // let maybe_ack = tx.tx_result.events.iter().find_map(extract_event);

        // match maybe_ack {
        //     Some(ack) => {
        //         info!(%packet_hash, %channel_id, "queried packet");

        //         Ok(PacketAckByHashResponse {
        //             ack: ack.into_encoding(),
        //             tx_hash: Some(tx.hash.into_encoding()),
        //             provable_height: tx.height.expect("tx must have a height; qed;").get() + 1,
        //         })
        //     }
        //     None => {
        //         info!("packet not found in a tx, checking for block events");

        //         let mut block_search_response = self
        //             .gno_client
        //             .block_search(
        //                 query,
        //                 const { NonZeroU32::new(1).unwrap() },
        //                 const { NonZeroU8::new(1).unwrap() },
        //                 Order::Asc,
        //             )
        //             .await
        //             .map_err(RpcError::retryable(
        //                 "error querying packet ack by packet hash in block",
        //             ))?;

        //         if block_search_response.total_count != 1 {
        //             return Err(RpcError::retryable_from_message(format!(
        //                 "error querying for packet {packet_hash}, \
        //                 expected 1 block but found {}",
        //                 block_search_response.total_count,
        //             ))
        //             .with_data(json!({ "block_response": block_search_response })));
        //         }

        //         let block_response = block_search_response
        //             .blocks
        //             .pop()
        //             .expect("len is > 1; qed;");

        //         let block_results = self
        //             .gno_client
        //             .block_results(Some(
        //                 (block_response.block.header.height.inner() as u64)
        //                     .try_into()
        //                     .expect("block number is valid"),
        //             ))
        //             .await
        //             .map_err(RpcError::retryable(format!(
        //                 "error querying block {}",
        //                 block_response.block.header.height
        //             )))?;

        //         let ack = block_results
        //             .finalize_block_events
        //             .iter()
        //             .flatten()
        //             .find_map(extract_event)
        //             .ok_or_else(|| {
        //                 RpcError::retryable_from_message(format!(
        //                     "error querying for packet {packet_hash}, channel \
        //                     {channel_id}; the wasm-packet_ack event was not found",
        //                 ))
        //                 .with_data(json!({ "block_results": block_results }))
        //             })?;

        //         Ok(PacketAckByHashResponse {
        //             ack: ack.into_encoding(),
        //             tx_hash: Some(tx.hash.into_encoding()),
        //             provable_height: block_response.block.header.height.inner() as u64 + 1,
        //         })
        //     }
        // }

        todo!()
    }

    #[instrument(skip_all, fields(?height))]
    pub async fn qeval(&self, query: impl Display, height: Option<Height>) -> RpcResult<String> {
        let value = self
            .gno_client
            .eval_query(
                &self.ibc_core_realm,
                &query,
                height.map(|height| {
                    i64::try_from(height.height())
                        .expect("should be fine")
                        .try_into()
                        .expect("invalid height")
                }),
            )
            .await
            .map_err(RpcError::retryable("error fetching eval query"))
            .with_data(json!({
                "height": height,
                "query": query.to_string(),
            }))?
            .ok_or(RpcError::fatal_from_message(
                "no value returned from a successful qeval query",
            ))?;

        let response = String::from_utf8(value.into())
            .map_err(RpcError::fatal("invalid utf8 returned from qeval query"))?;

        debug!(%response, "vm/qeval response");

        Ok(response)
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
            .qeval(format!("QueryClientState({client_id})"), Some(height))
            .await
            .and_then(parse_gno_string_object)?;

        if client_state.is_empty() {
            Ok(None)
        } else {
            client_state
                .parse::<Bytes>()
                .map(Some)
                .map_err(RpcError::fatal("unable to parse response bytes"))
        }
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
        let consensus_state = self
            .qeval(
                format!("QueryConsensusState({client_id}, {trusted_height})"),
                Some(height),
            )
            .await
            .and_then(parse_gno_string_object)?;

        if consensus_state.is_empty() {
            Ok(None)
        } else {
            consensus_state
                .parse::<Bytes>()
                .map(Some)
                .map_err(RpcError::fatal("unable to parse response bytes"))
        }
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
        let connection = self
            .qeval(format!("QueryConnection({connection_id})"), Some(height))
            .await
            .and_then(parse_gno_string_object)?;

        if connection.is_empty() {
            Ok(None)
        } else {
            Connection::decode_as::<EthAbi>(
                &connection
                    .parse::<Bytes>()
                    .map_err(RpcError::fatal("unable to parse response bytes"))?,
            )
            .map(Some)
            .map_err(RpcError::fatal("invalid connection returned from contract"))
        }
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
            .qeval(format!("QueryChannel({channel_id})"), Some(height))
            .await
            .and_then(parse_gno_string_object)?;

        if channel.is_empty() {
            Ok(None)
        } else {
            Channel::ethabi_decode_prefixed(
                &channel
                    .parse::<Bytes>()
                    .map_err(RpcError::fatal("unable to parse response bytes"))?,
            )
            .map(Some)
            .map_err(RpcError::fatal("invalid channel returned from contract"))
        }
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
        // let commitment = self
        //     .qeval(
        //         format!("QueryBatchPackets({})", gno_bytes32(batch_hash)),
        //         Some(height),
        //     )
        //     .await
        //     .and_then(parse_gno_string_object)?;

        // if commitment.is_empty() {
        //     Ok(None)
        // } else {
        //     commitment
        //         .parse::<H256>()
        //         .map(Some)
        //         .map_err(RpcError::fatal("unable to parse response bytes"))
        // }

        let commitment = self
            .gno_client
            .abci_query(
                ".store/main/key",
                format!(
                    "/pv/vm:{}:{}",
                    self.ibc_core_realm,
                    BatchPacketsPath { batch_hash }
                        .key()
                        .into_encoding::<HexUnprefixed>()
                ),
                Some(
                    (height.height() as i64)
                        .try_into()
                        .map_err(RpcError::fatal("invalid gno height"))?,
                ),
                false,
            )
            .await?
            .response
            .value;

        match commitment {
            Some(commitment) => commitment
                .try_into()
                .map(Some)
                .map_err(RpcError::fatal("invalid commitment value")),
            None => Ok(None),
        }
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
        // let commitment = self
        //     .qeval(
        //         format!("QueryBatchReceipts({})", gno_bytes32(batch_hash)),
        //         Some(height),
        //     )
        //     .await
        //     .and_then(parse_gno_string_object)?;

        // if commitment.is_empty() {
        //     Ok(None)
        // } else {
        //     commitment
        //         .parse::<H256>()
        //         .map(Some)
        //         .map_err(RpcError::fatal("unable to parse response bytes"))
        // }

        let commitment = self
            .gno_client
            .abci_query(
                ".store/main/key",
                format!(
                    "/pv/vm:{}:{}",
                    self.ibc_core_realm,
                    BatchReceiptsPath { batch_hash }
                        .key()
                        .into_encoding::<HexUnprefixed>()
                ),
                Some(
                    (height.height() as i64)
                        .try_into()
                        .map_err(RpcError::fatal("invalid gno height"))?,
                ),
                false,
            )
            .await?
            .response
            .value;

        match commitment {
            Some(commitment) => commitment
                .try_into()
                .map(Some)
                .map_err(RpcError::fatal("invalid commitment value")),
            None => Ok(None),
        }
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
        Err(RpcError::fatal_from_message("not supported on gno"))
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
        Err(RpcError::fatal_from_message("not supported on gno"))
    }
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
            // TODO: Also query block events here
            Query::PacketsByBatchHash(PacketsByBatchHash {
                channel_id: _,
                batch_hash: _,
            }) => {
                todo!()
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
                    .qeval(
                        format!("GetClientStatus({client_id})"),
                        height.map(Height::new),
                    )
                    .await
                    .and_then(parse_gno_string_object)?
                    .parse::<u8>()
                    .map_err(RpcError::fatal(format!(
                        "invalid client status for client `{client_id}` \
                        at height {height:?}"
                    )))
                    .and_then(|n| match n {
                        0 => Err(RpcError::fatal_from_message(format!(
                            "client `{client_id}` not found at height {height:?}"
                        ))),
                        1 => Ok(Status::Active),
                        2 => Ok(Status::Expired),
                        3 => Ok(Status::Frozen),
                        _ => Err(RpcError::fatal_from_message(format!(
                            "invalid client status `{n}` for client \
                            `{client_id}` at height {height:?}"
                        ))),
                    })?;

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
            .qeval(format!("GetClientType({client_id})"), None)
            .await
            .and_then(parse_gno_string_object)?;

        if client_type.is_empty() {
            return Err(RpcError::fatal_from_message(format!(
                "client `{client_id}` not found"
            )));
        }

        Ok(ClientInfo {
            client_type: ClientType::new(client_type),
            ibc_interface: IbcInterface::new(IbcInterface::IBC_GNO),
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
            StorePath::BatchTimeouts(_) => unimplemented!(),
        }
    }
}

// REVIEW: Do I need to do unescaping here?
fn parse_gno_string_object(s: impl AsRef<str>) -> RpcResult<String> {
    s.as_ref()
        .strip_prefix("(\"")
        .and_then(|s| s.strip_suffix("\" string)"))
        .map(ToOwned::to_owned)
        .ok_or(RpcError::fatal_from_message("invalid string object"))
}

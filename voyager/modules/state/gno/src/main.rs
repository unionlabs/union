#![doc = include_str!("../README.md")]
#![warn(clippy::unwrap_used)]

use std::fmt::Display;

use futures::{TryFutureExt, TryStreamExt, stream::FuturesUnordered};
use ibc_union_spec::{
    Channel, ChannelId, ClientId, Connection, ConnectionId, IbcUnion, MustBeZero, Packet, Status,
    path::{BatchPacketsPath, BatchReceiptsPath, StorePath},
    query::{
        ClientStatus, PacketAckByHash, PacketAckByHashResponse, PacketByHash, PacketByHashResponse,
        PacketsByBatchHash, PacketsByBatchHashResponse, Query,
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
        let ibc_core_realm = &self.ibc_core_realm;
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
                pkg_path: {{ eq: "{ibc_core_realm}" }},
                attrs: {{
                  key: {{ eq: "source_channel_id" }}
                  value: {{ eq: "{channel_id}" }}
                }}
              }}
          	}}
            {{
              GnoEvent: {{
                type: {{ eq: "PacketSend" }}
                pkg_path: {{ eq: "{ibc_core_realm}" }},
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
}}"#
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

        let transactions = res
            .pointer("/data/getTransactions")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                RpcError::fatal_from_message("no getTransactions in graphql response")
            })?;

        if transactions.len() != 1 {
            return Err(RpcError::retryable_from_message(format!(
                "expected exactly one transaction matching channel_id {channel_id} and \
                 packet_hash {packet_hash}, found {}",
                transactions.len()
            )));
        }

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

        let events = res
            .pointer("/data/getTransactions/0/response/events")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                RpcError::fatal_from_message("no response events in graphql response")
            })?;

        let (source_channel_id, destination_channel_id, packet_data, timeout_timestamp) = events
            .iter()
            .filter(|event| {
                event.pointer("/type").and_then(Value::as_str) == Some("PacketSend")
                    && event.pointer("/pkg_path").and_then(Value::as_str)
                        == Some(ibc_core_realm.as_str())
            })
            .map(|raw_event| {
                let event = serde_json::from_value(raw_event.clone())
                    .map_err(RpcError::fatal("invalid attrs in graphql response"))?;

                let IbcEvent::PacketSend {
                    packet_hash: found_packet_hash,
                    packet_data,
                    source_channel_id: found_channel_id,
                    destination_channel_id,
                    timeout_timestamp,
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

                Ok((
                    found_packet_hash,
                    found_channel_id,
                    packet_data,
                    destination_channel_id,
                    timeout_timestamp,
                ))
            })
            .collect::<RpcResult<Vec<_>>>()?
            .into_iter()
            // the tx can bundle multiple gno msgs together, so the where clause only
            // guarantees *a* matching event is somewhere in the tx, not that every
            // packet_send event in it belongs to this packet
            .find_map(
                |(
                    found_packet_hash,
                    found_channel_id,
                    packet_data,
                    destination_channel_id,
                    timeout_timestamp,
                )| {
                    (found_packet_hash == packet_hash && found_channel_id == channel_id).then_some(
                        (
                            found_channel_id,
                            destination_channel_id,
                            packet_data,
                            timeout_timestamp,
                        ),
                    )
                },
            )
            .ok_or_else(|| {
                RpcError::fatal_from_message(format!(
                    "no packet_send event matching channel_id {channel_id} / packet_hash \
                     {packet_hash} found in graphql response",
                ))
            })?;

        Ok(PacketByHashResponse {
            packet: Packet {
                source_channel_id,
                destination_channel_id,
                data: packet_data,
                timeout_height: MustBeZero,
                timeout_timestamp,
            },
            tx_hash: Some(hash.into_encoding()),
            provable_height: height + 1,
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %channel_id, %batch_hash))]
    pub async fn query_packets_by_batch_hash(
        &self,
        channel_id: ChannelId,
        batch_hash: H256,
    ) -> RpcResult<PacketsByBatchHashResponse> {
        let ibc_core_realm = &self.ibc_core_realm;
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
                type: {{ eq: "BatchSend" }}
                pkg_path: {{ eq: "{ibc_core_realm}" }},
                attrs: {{
                  key: {{ eq: "channel_id" }}
                  value: {{ eq: "{channel_id}" }}
                }}
              }}
          	}}
            {{
              GnoEvent: {{
                type: {{ eq: "BatchSend" }}
                pkg_path: {{ eq: "{ibc_core_realm}" }},
                attrs: {{
                  key: {{ eq:"batch_hash" }}
                  value: {{ eq:"{batch_hash}" }}
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
}}"#
        );

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

        let transactions = res
            .pointer("/data/getTransactions")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                RpcError::fatal_from_message("no getTransactions in graphql response")
            })?;

        if transactions.len() != 1 {
            return Err(RpcError::retryable_from_message(format!(
                "expected exactly one transaction matching channel_id {channel_id} and \
                 batch_hash {batch_hash}, found {}",
                transactions.len()
            )));
        }

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

        let events = res
            .pointer("/data/getTransactions/0/response/events")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                RpcError::fatal_from_message("no response events in graphql response")
            })?;

        let packet_hashes = events
            .iter()
            .filter(|event| {
                event.pointer("/type").and_then(Value::as_str) == Some("BatchSend")
                    && event.pointer("/pkg_path").and_then(Value::as_str)
                        == Some(ibc_core_realm.as_str())
            })
            .map(|raw_event| {
                let event = serde_json::from_value(raw_event.clone())
                    .map_err(RpcError::fatal("invalid attrs in graphql response"))?;

                let IbcEvent::BatchSend {
                    packet_hash,
                    batch_hash: found_batch_hash,
                    channel_id: found_channel_id,
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

                Ok((found_batch_hash, found_channel_id, packet_hash))
            })
            .collect::<RpcResult<Vec<_>>>()?
            .into_iter()
            // the tx can bundle multiple gno msgs together (see
            // `do_send_transaction`/`broadcast_tx_commit` in the gno transaction
            // plugin), so the where clause only guarantees *a* matching event is
            // somewhere in the tx, not that every BatchSend event in it belongs to
            // this batch. events for other batches bundled into the same tx are
            // filtered out here rather than treated as an error.
            .filter_map(|(found_batch_hash, found_channel_id, packet_hash)| {
                (found_batch_hash == batch_hash && found_channel_id == channel_id)
                    .then_some(packet_hash)
            })
            .collect::<Vec<_>>();

        if packet_hashes.len() < 2 {
            return Err(RpcError::fatal_from_message(format!(
                "expected at least 2 packets in batch {batch_hash}, found {}",
                packet_hashes.len()
            )));
        }

        let packets = packet_hashes
            .into_iter()
            .map(|packet_hash| {
                self.query_packet_by_hash(channel_id, packet_hash)
                    .map_ok(|res| res.packet)
            })
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<_>>()
            .await?;

        Ok(PacketsByBatchHashResponse {
            packets,
            tx_hash: hash.into_encoding(),
            provable_height: height + 1,
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %channel_id, %packet_hash))]
    pub async fn query_packet_ack_by_hash(
        &self,
        channel_id: ChannelId,
        packet_hash: H256,
    ) -> RpcResult<PacketAckByHashResponse> {
        let ibc_core_realm = &self.ibc_core_realm;
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
                type: {{ eq: "WriteAck" }}
                pkg_path: {{ eq: "{ibc_core_realm}" }},
                attrs: {{
                  key: {{ eq: "destination_channel_id" }}
                  value: {{ eq: "{channel_id}" }}
                }}
              }}
          	}}
            {{
              GnoEvent: {{
                type: {{ eq: "WriteAck" }}
                pkg_path: {{ eq: "{ibc_core_realm}" }},
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
}}"#
        );

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

        let transactions = res
            .pointer("/data/getTransactions")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                RpcError::fatal_from_message("no getTransactions in graphql response")
            })?;

        if transactions.len() != 1 {
            return Err(RpcError::retryable_from_message(format!(
                "expected exactly one transaction matching channel_id {channel_id} and \
                 packet_hash {packet_hash}, found {}",
                transactions.len()
            )));
        }

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

        let events = res
            .pointer("/data/getTransactions/0/response/events")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                RpcError::fatal_from_message("no response events in graphql response")
            })?;

        let acknowledgement = events
            .iter()
            .filter(|event| {
                event.pointer("/type").and_then(Value::as_str) == Some("WriteAck")
                    && event.pointer("/pkg_path").and_then(Value::as_str)
                        == Some(ibc_core_realm.as_str())
            })
            .map(|raw_event| {
                let event = serde_json::from_value(raw_event.clone())
                    .map_err(RpcError::fatal("invalid attrs in graphql response"))?;

                let IbcEvent::WriteAck {
                    packet_hash: found_packet_hash,
                    destination_channel_id: found_channel_id,
                    acknowledgement,
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

                Ok((found_packet_hash, found_channel_id, acknowledgement))
            })
            .collect::<RpcResult<Vec<_>>>()?
            .into_iter()
            // the tx can bundle multiple gno msgs together, so the where clause only
            // guarantees *a* matching event is somewhere in the tx, not that every
            // write_ack event in it belongs to this packet
            .find_map(|(found_packet_hash, found_channel_id, acknowledgement)| {
                (found_packet_hash == packet_hash && found_channel_id == channel_id)
                    .then_some(acknowledgement)
            })
            .ok_or_else(|| {
                RpcError::fatal_from_message(format!(
                    "no write_ack event matching channel_id {channel_id} / packet_hash \
                     {packet_hash} found in graphql response",
                ))
            })?;

        Ok(PacketAckByHashResponse {
            ack: acknowledgement.into_encoding(),
            tx_hash: Some(hash.into_encoding()),
            provable_height: height + 1,
        })
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
            Query::PacketsByBatchHash(PacketsByBatchHash {
                channel_id,
                batch_hash,
            }) => self
                .query_packets_by_batch_hash(channel_id, batch_hash)
                .await
                .map(into_value),
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

fn parse_gno_string_object(s: impl AsRef<str>) -> RpcResult<String> {
    let s = s.as_ref();
    // Handles both old format ("value" string) and new format
    // ("value" gno.land/p/.../types.T, nil error)
    s.strip_prefix("(\"")
        .and_then(|s| s.find("\" ").map(|end| s[..end].to_owned()))
        .ok_or(RpcError::fatal_from_message("invalid string object"))
}

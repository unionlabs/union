#![warn(clippy::unwrap_used)]

use std::sync::Arc;

use alloy::{
    eips::BlockNumberOrTag,
    network::AnyNetwork,
    providers::{layers::CacheLayer, DynProvider, Provider, ProviderBuilder},
    rpc::types::{TransactionInput, TransactionRequest},
    serde::WithOtherFields,
    sol_types::{SolCall, SolValue},
};
use futures::{stream::FuturesUnordered, TryStreamExt};
use ibc_solidity::{
    ILightClient,
    Ibc::{self, IbcInstance},
};
use ibc_union_spec::{
    path::{BatchPacketsPath, BatchReceiptsPath, StorePath},
    query::Query,
    Channel, ChannelId, ChannelState, ClientId, Connection, ConnectionId, ConnectionState,
    IbcUnion, Packet,
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, info, instrument, trace};
use unionlabs::{
    ibc::core::client::height::Height,
    primitives::{Bytes, H160, H256},
    ErrorReporter,
};
use voyager_sdk::{
    self, anyhow, into_value,
    plugin::StateModule,
    primitives::{ChainId, ClientInfo, ClientType, IbcInterface},
    rpc::{types::StateModuleInfo, StateModuleServer, MISSING_STATE_ERROR_CODE},
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

pub struct Module {
    pub chain_id: ChainId,

    pub ibc_handler_address: H160,

    pub max_query_window: Option<u64>,

    pub provider: DynProvider<AnyNetwork>,

    pub channel_cache: moka::future::Cache<ChannelId, Channel>,
    pub connection_cache: moka::future::Cache<ConnectionId, Connection>,
    pub client_address_cache: moka::future::Cache<u32, alloy::primitives::Address>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    /// The RPC endpoint for the execution chain.
    pub rpc_url: String,

    #[serde(default)]
    pub max_query_window: Option<u64>,

    #[serde(default)]
    pub max_cache_size: u32,
}

impl StateModule<IbcUnion> for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: StateModuleInfo) -> anyhow::Result<Self> {
        let provider = DynProvider::new(
            ProviderBuilder::new()
                .layer(CacheLayer::new(config.max_cache_size))
                .network::<AnyNetwork>()
                .connect(&config.rpc_url)
                .await?,
        );

        let chain_id = provider.get_chain_id().await?;

        info.ensure_chain_id(chain_id.to_string())?;

        Ok(Module {
            chain_id: ChainId::new(chain_id.to_string()),
            ibc_handler_address: config.ibc_handler_address,
            max_query_window: config.max_query_window,
            // should probably be big enough
            channel_cache: moka::future::Cache::new(10_000),
            connection_cache: moka::future::Cache::new(10_000),
            client_address_cache: moka::future::Cache::new(10_000),
            provider,
        })
    }
}

impl Module {
    #[must_use]
    pub fn make_height(&self, height: u64) -> Height {
        Height::new(height)
    }

    fn ibc_handler(&self) -> IbcInstance<DynProvider<AnyNetwork>, AnyNetwork> {
        Ibc::new::<_, AnyNetwork>(self.ibc_handler_address.get().into(), self.provider.clone())
    }

    // TODO: This can definitely be cached
    #[instrument(skip(self))]
    pub async fn client_address(
        &self,
        client_id: u32,
        height: u64,
    ) -> RpcResult<alloy::primitives::Address> {
        self.client_address_cache
            .try_get_with(client_id, async {
                let client_address = self
                    .ibc_handler()
                    .clientImpls(client_id)
                    .block(height.into())
                    .call()
                    .await
                    .map_err(|err| {
                        ErrorObject::owned(
                            -1,
                            format!("error fetching client address: {}", ErrorReporter(err)),
                            None::<()>,
                        )
                    })?;

                debug!(%client_address, "fetched client address");

                Ok(client_address)
            })
            .await
            .map_err(|e: Arc<ErrorObject>| (*e).clone())
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %client_id))]
    async fn query_client_state(
        &self,
        height: Height,
        client_id: ClientId,
    ) -> RpcResult<Option<Bytes>> {
        let execution_height = height.height();

        let client_address = self
            .client_address(client_id.raw(), execution_height)
            .await?;

        let light_client = ILightClient::new(client_address, self.provider.clone());
        let client_state = light_client
            .getClientState(client_id.raw())
            .block(execution_height.into())
            .call()
            .await;

        match client_state {
            Ok(client_state) => Ok(Some(client_state.0.to_vec().into())),
            Err(alloy::contract::Error::AbiError(_) | alloy::contract::Error::ZeroData(_, _)) => {
                Ok(None)
            }
            Err(err) => Err(ErrorObject::owned(
                -1,
                format!("error fetching client state: {}", ErrorReporter(err)),
                None::<()>,
            )),
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %client_id, %trusted_height))]
    async fn query_consensus_state(
        &self,
        height: Height,
        client_id: ClientId,
        trusted_height: u64,
    ) -> RpcResult<Option<Bytes>> {
        let execution_height = height.height();

        let client_address = self
            .client_address(client_id.raw(), execution_height)
            .await?;

        let light_client = ILightClient::new(client_address, self.provider.clone());

        let consensus_state = light_client
            .getConsensusState(client_id.raw(), trusted_height)
            .block(execution_height.into())
            .call()
            .await;

        match consensus_state {
            Ok(consensus_state) => Ok(Some(consensus_state.0.to_vec().into())),
            Err(alloy::contract::Error::AbiError(_) | alloy::contract::Error::ZeroData(_, _)) => {
                Ok(None)
            }
            Err(err) => Err(ErrorObject::owned(
                -1,
                format!("error fetching consensus state: {}", ErrorReporter(err)),
                None::<()>,
            )),
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %connection_id))]
    async fn query_connection(
        &self,
        height: Height,
        connection_id: ConnectionId,
    ) -> RpcResult<Option<Connection>> {
        if let Some(connection) = self.connection_cache.get(&connection_id).await {
            trace!("cache hit");

            return Ok(Some(connection));
        }

        let execution_height = height.height();

        let ibc_handler = self.ibc_handler();

        let raw = ibc_handler
            .provider()
            .call(WithOtherFields::new(TransactionRequest {
                from: None,
                to: Some(alloy::primitives::Address::from(self.ibc_handler_address).into()),
                input: TransactionInput::new(
                    Ibc::connectionsCall(connection_id.raw())
                        .abi_encode()
                        .into(),
                ),
                ..Default::default()
            }))
            .block(execution_height.into())
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    format!("error querying channel: {}", ErrorReporter(e)),
                    None::<()>,
                )
            })?;

        let connection = Connection::abi_decode_params_validate(&raw).map_err(|e| {
            ErrorObject::owned(
                -1,
                format!("error decoding channel: {}", ErrorReporter(e)),
                None::<()>,
            )
        })?;

        if connection.state == ConnectionState::Open {
            info!("connection is open, caching");

            self.connection_cache
                .insert(connection_id, connection.clone())
                .await;
        }

        Ok(Some(connection))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %channel_id))]
    async fn query_channel(
        &self,
        height: Height,
        channel_id: ChannelId,
    ) -> RpcResult<Option<Channel>> {
        // NOTE: We will need to review this logic if/when we support channel closings
        if let Some(channel) = self.channel_cache.get(&channel_id).await {
            trace!("cache hit");

            return Ok(Some(channel));
        }

        let execution_height = height.height();

        let ibc_handler = self.ibc_handler();

        // https://github.com/alloy-rs/core/issues/811
        // let raw = ibc_handler
        //     .channels(channel_id)
        //     .block(execution_height.into())
        //     .call()
        //     .await
        //     .map_err(|err| {
        //         ErrorObject::owned(
        //             -1,
        //             format!("error fetching channel: {}", ErrorReporter(err)),
        //             None::<()>,
        //         )
        //     })?
        //     ._0;

        let raw = ibc_handler
            .provider()
            .call(WithOtherFields::new(TransactionRequest {
                from: None,
                to: Some(alloy::primitives::Address::from(self.ibc_handler_address).into()),
                input: TransactionInput::new(
                    Ibc::channelsCall(channel_id.raw()).abi_encode().into(),
                ),
                ..Default::default()
            }))
            .block(execution_height.into())
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    format!("error querying channel: {}", ErrorReporter(e)),
                    None::<()>,
                )
            })?;

        let channel = Channel::abi_decode_params_validate(&raw).map_err(|e| {
            ErrorObject::owned(
                -1,
                format!("error decoding channel: {}", ErrorReporter(e)),
                None::<()>,
            )
        })?;

        if channel.state == ChannelState::Open {
            info!("channel is open, caching");

            self.channel_cache.insert(channel_id, channel.clone()).await;
        }

        Ok(Some(channel))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height))]
    async fn query_batch_packets(
        &self,
        height: Height,
        batch_hash: H256,
    ) -> RpcResult<Option<H256>> {
        let execution_height = height.height();

        let ibc_handler = self.ibc_handler();

        let raw = ibc_handler
            .commitments(BatchPacketsPath { batch_hash }.key().into())
            .block(execution_height.into())
            .call()
            .await
            .map_err(|err| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching batch commitments: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })?;

        if <H256>::from(raw) == <H256>::default() {
            Ok(None)
        } else {
            Ok(Some(raw.into()))
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height))]
    async fn query_batch_receipts(
        &self,
        height: Height,
        batch_hash: H256,
    ) -> RpcResult<Option<H256>> {
        let execution_height = height.height();

        let ibc_handler = self.ibc_handler();

        let raw = ibc_handler
            .commitments(BatchReceiptsPath { batch_hash }.key().into())
            .block(execution_height.into())
            .call()
            .await
            .map_err(|err| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching batch receipts: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })?;

        if <H256>::from(raw) == <H256>::default() {
            Ok(None)
        } else {
            Ok(Some(raw.into()))
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %channel_id, %packet_hash))]
    async fn packet_by_packet_hash(
        &self,
        channel_id: ChannelId,
        packet_hash: H256,
    ) -> RpcResult<Packet> {
        let ibc_handler = self.ibc_handler();

        let windows = match self.max_query_window {
            Some(window) => {
                let latest_height = self.provider.get_block_number().await.map_err(|e| {
                    ErrorObject::owned(
                        -1,
                        format!(
                            "error querying latest height while constructing query windows for decoding packet send event for packet {packet_hash}: {}",
                            ErrorReporter(e)
                        ),
                        None::<()>,
                    )
                })?;
                mk_windows(latest_height, window)
            }
            None => vec![(BlockNumberOrTag::Earliest, BlockNumberOrTag::Latest)],
        };

        for (from, to) in windows {
            debug!(%from, %to, "querying range for packet");

            let query = ibc_handler
                .PacketSend_filter()
                .topic1(alloy::primitives::U256::from(channel_id.raw()))
                .topic2(alloy::primitives::U256::from_be_bytes(*(packet_hash.get())));

            trace!(?query, "raw query");

            let mut packet_logs =
                query
                    .from_block(from)
                    .to_block(to)
                    .query()
                    .await
                    .map_err(|e| {
                        ErrorObject::owned(
                            -1,
                            format!(
                                "error querying for packet {packet_hash}: {}",
                                ErrorReporter(e)
                            ),
                            None::<()>,
                        )
                    })?;

            if packet_logs.is_empty() {
                debug!(%from, %to, "packet not found in range");
                continue;
            } else if packet_logs.len() == 1 {
                // there's really no nicer way to do this without having multiple checks (we want to ensure there's only 1 item in the list)
                let (packet_log, _) = packet_logs.pop().expect("len is 1; qed;");

                return packet_log.packet.try_into().map_err(|e| {
                    ErrorObject::owned(
                        -1,
                        format!(
                            "error decoding packet send event \
                            for packet {packet_hash}: {}",
                            ErrorReporter(e)
                        ),
                        None::<()>,
                    )
                });
            } else {
                return Err(ErrorObject::owned(
                    -1,
                    format!(
                        "error querying for packet {packet_hash}, \
                        expected 1 event but found {}",
                        packet_logs.len()
                    ),
                    None::<()>,
                ));
            }
        }

        Err(ErrorObject::owned(
            MISSING_STATE_ERROR_CODE,
            format!("packet {packet_hash} not found"),
            None::<()>,
        ))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %channel_id, %batch_hash))]
    async fn packets_by_batch_hash(
        &self,
        channel_id: ChannelId,
        batch_hash: H256,
    ) -> RpcResult<Vec<Packet>> {
        let ibc_handler = self.ibc_handler();

        let windows = match self.max_query_window {
            Some(window) => {
                let latest_height = self.provider.get_block_number().await.map_err(|e| {
                    ErrorObject::owned(
                        -1,
                        format!(
                            "error querying latest height while constructing query windows for decoding packet send event for packet {batch_hash}: {}",
                            ErrorReporter(e)
                        ),
                        None::<()>,
                    )
                })?;
                mk_windows(latest_height, window)
            }
            None => vec![(BlockNumberOrTag::Earliest, BlockNumberOrTag::Latest)],
        };

        for (from, to) in windows {
            debug!(%from, %to, "querying range for packet");

            let query = ibc_handler
                .BatchedPreviouslySent_filter()
                .topic1(alloy::primitives::U256::from(channel_id.raw()))
                .topic2(alloy::primitives::U256::from_be_bytes(*(batch_hash.get())));

            trace!(?query, "raw query");

            let batch_logs = query
                .from_block(from)
                .to_block(to)
                .query()
                .await
                .map_err(|e| {
                    ErrorObject::owned(
                        -1,
                        format!(
                            "error querying for packet {batch_hash}: {}",
                            ErrorReporter(e)
                        ),
                        None::<()>,
                    )
                })?;

            if batch_logs.is_empty() {
                debug!(%from, %to, "batch not found in range");
                continue;
            } else {
                return batch_logs
                    .into_iter()
                    .map(|(event, _)| {
                        self.packet_by_packet_hash(
                            event
                                .channel_id
                                .try_into()
                                .expect("invalid channel id on event?"),
                            event.packet_hash.into(),
                        )
                    })
                    .collect::<FuturesUnordered<_>>()
                    .try_collect::<Vec<_>>()
                    .await;
            }
        }

        Err(ErrorObject::owned(
            MISSING_STATE_ERROR_CODE,
            format!("packet {batch_hash} not found"),
            None::<()>,
        ))
    }
}

fn mk_windows(mut latest_height: u64, window: u64) -> Vec<(BlockNumberOrTag, BlockNumberOrTag)> {
    std::iter::from_fn(|| {
        if latest_height == 0 {
            None
        } else {
            let upper_bound = latest_height;
            let lower_bound = latest_height.saturating_sub(window);
            latest_height = lower_bound;
            Some((lower_bound.into(), upper_bound.into()))
        }
    })
    .collect()
}

#[async_trait]
impl StateModuleServer<IbcUnion> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query(&self, _: &Extensions, query: Query) -> RpcResult<Value> {
        match query {
            Query::PacketByHash(packet_by_hash) => self
                .packet_by_packet_hash(packet_by_hash.channel_id, packet_by_hash.packet_hash)
                .await
                .map(into_value),
            Query::PacketsByBatchHash(packets_by_batch_hash) => self
                .packets_by_batch_hash(
                    packets_by_batch_hash.channel_id,
                    packets_by_batch_hash.batch_hash,
                )
                .await
                .map(into_value),
        }
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
            StorePath::BatchReceipts(path) => self
                .query_batch_receipts(at, path.batch_hash)
                .await
                .map(into_value),
            StorePath::BatchPackets(path) => self
                .query_batch_packets(at, path.batch_hash)
                .await
                .map(into_value),
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn client_info(&self, _: &Extensions, client_id: ClientId) -> RpcResult<ClientInfo> {
        let ibc_handler = self.ibc_handler();
        let client_type = ibc_handler
            .clientTypes(client_id.raw())
            .call()
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching client info: {}", ErrorReporter(e)),
                    None::<()>,
                )
            })?;

        Ok(ClientInfo {
            client_type: ClientType::new(client_type),
            ibc_interface: IbcInterface::new(IbcInterface::IBC_SOLIDITY),
            metadata: Default::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mk_windows() {
        assert_eq!(
            mk_windows(30, 10),
            vec![
                (20.into(), 30.into()),
                (10.into(), 20.into()),
                (0.into(), 10.into())
            ]
        );

        assert_eq!(
            mk_windows(29, 10),
            vec![
                (19.into(), 29.into()),
                (9.into(), 19.into()),
                (0.into(), 9.into())
            ]
        )
    }
}

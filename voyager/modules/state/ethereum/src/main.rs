#![warn(clippy::unwrap_used)]

use alloy::{
    eips::BlockNumberOrTag,
    providers::{layers::CacheLayer, DynProvider, Provider, ProviderBuilder},
    rpc::types::{TransactionInput, TransactionRequest},
    sol_types::{SolCall, SolValue},
};
use ibc_solidity::{
    ILightClient,
    Ibc::{self, IbcInstance},
};
use ibc_union_spec::{
    path::{BatchPacketsPath, BatchReceiptsPath, StorePath},
    query::Query,
    Channel, ChannelId, ClientId, Connection, ConnectionId, IbcUnion, Packet,
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, instrument, trace};
use unionlabs::{
    ibc::core::client::height::Height,
    primitives::{Bytes, H160, H256},
    ErrorReporter,
};
use voyager_message::{
    into_value,
    module::{StateModuleInfo, StateModuleServer},
    primitives::{ChainId, ClientInfo, ClientType, IbcInterface},
    StateModule,
};
use voyager_vm::BoxDynError;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

pub struct Module {
    pub chain_id: ChainId,

    pub ibc_handler_address: H160,

    pub provider: DynProvider,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    /// The RPC endpoint for the execution chain.
    pub rpc_url: String,

    #[serde(default)]
    pub max_cache_size: u32,
}

impl StateModule<IbcUnion> for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: StateModuleInfo) -> Result<Self, BoxDynError> {
        let provider = DynProvider::new(
            ProviderBuilder::new()
                .layer(CacheLayer::new(config.max_cache_size))
                .connect(&config.rpc_url)
                .await?,
        );

        let chain_id = provider.get_chain_id().await?;

        info.ensure_chain_id(chain_id.to_string())?;

        Ok(Module {
            chain_id: ChainId::new(chain_id.to_string()),
            ibc_handler_address: config.ibc_handler_address,
            provider,
        })
    }
}

impl Module {
    #[must_use]
    pub fn make_height(&self, height: u64) -> Height {
        Height::new(height)
    }

    fn ibc_handler(&self) -> IbcInstance<(), DynProvider> {
        Ibc::new(self.ibc_handler_address.get().into(), self.provider.clone())
    }

    // TODO: This can definitely be cached
    #[instrument(skip(self))]
    pub async fn client_address(
        &self,
        client_id: u32,
        height: u64,
    ) -> RpcResult<alloy::primitives::Address> {
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
            })?
            ._0;

        trace!(%client_address, "fetched client address");

        Ok(client_address)
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
            Ok(client_state) => Ok(Some(client_state._0.0.to_vec().into())),
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
            Ok(consensus_state) => Ok(Some(consensus_state._0.0.to_vec().into())),
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
        let execution_height = height.height();

        let ibc_handler = self.ibc_handler();

        let raw = ibc_handler
            .provider()
            .call(TransactionRequest {
                from: None,
                to: Some(alloy::primitives::Address::from(self.ibc_handler_address).into()),
                input: TransactionInput::new(
                    Ibc::connectionsCall {
                        _0: connection_id.raw(),
                    }
                    .abi_encode()
                    .into(),
                ),
                ..Default::default()
            })
            .block(execution_height.into())
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    format!("error querying channel: {}", ErrorReporter(e)),
                    None::<()>,
                )
            })?;

        let connection = Connection::abi_decode_params(&raw, true).map_err(|e| {
            ErrorObject::owned(
                -1,
                format!("error decoding channel: {}", ErrorReporter(e)),
                None::<()>,
            )
        })?;

        Ok(Some(connection))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %channel_id))]
    async fn query_channel(
        &self,
        height: Height,
        channel_id: ChannelId,
    ) -> RpcResult<Option<Channel>> {
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
            .call(TransactionRequest {
                from: None,
                to: Some(alloy::primitives::Address::from(self.ibc_handler_address).into()),
                input: TransactionInput::new(
                    Ibc::channelsCall {
                        _0: channel_id.raw(),
                    }
                    .abi_encode()
                    .into(),
                ),
                ..Default::default()
            })
            .block(execution_height.into())
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    format!("error querying channel: {}", ErrorReporter(e)),
                    None::<()>,
                )
            })?;

        let channel = Channel::abi_decode_params(&raw, true).map_err(|e| {
            ErrorObject::owned(
                -1,
                format!("error decoding channel: {}", ErrorReporter(e)),
                None::<()>,
            )
        })?;

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
            })?
            ._0;

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
            })?
            ._0;

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

        let query = ibc_handler
            .PacketSend_filter()
            .topic1(alloy::primitives::U256::from(channel_id.raw()))
            .topic2(alloy::primitives::U256::from_be_bytes(*(packet_hash.get())))
            .from_block(BlockNumberOrTag::Earliest)
            .to_block(BlockNumberOrTag::Latest);

        debug!(?query, "raw query");

        query
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
            })
            .and_then(|mut packet_logs| {
                if packet_logs.len() == 1 {
                    // there's really no nicer way to do this without having multiple checks (we want to ensure there's only 1 item in the list)
                    let (packet_log, _) = packet_logs.pop().expect("len is 1; qed;");

                    packet_log.packet.try_into().map_err(|e| {
                        ErrorObject::owned(
                            -1,
                            format!(
                                "error decoding packet send event for packet {packet_hash}: {}",
                                ErrorReporter(e)
                            ),
                            None::<()>,
                        )
                    })
                } else {
                    Err(ErrorObject::owned(
                        -1,
                        format!(
                            "error querying for packet {packet_hash}, expected 1 event but found {}",
                            packet_logs.len()
                        ),
                        None::<()>,
                    ))
                }
            })
    }
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
            })?
            ._0;

        Ok(ClientInfo {
            client_type: ClientType::new(client_type),
            ibc_interface: IbcInterface::new(IbcInterface::IBC_SOLIDITY),
            metadata: Default::default(),
        })
    }
}

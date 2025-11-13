#![warn(clippy::unwrap_used)]

use ibc_union_spec::{
    Channel, ChannelId, ClientId, Connection, ConnectionId, IbcUnion, Status,
    path::StorePath,
    query::{PacketAckByHashResponse, PacketByHashResponse, PacketsByBatchHashResponse, Query},
};
use jsonrpsee::{
    Extensions,
    core::{RpcResult, async_trait},
    types::ErrorObject,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use tracing::instrument;
use unionlabs::{
    ErrorReporter,
    ibc::core::client::height::Height,
    primitives::{Bytes, H160, H256},
};
use voyager_sdk::{
    ExtensionsExt, anyhow, into_value,
    plugin::StateModule,
    primitives::{ChainId, ClientInfo},
    rpc::{StateModuleServer, types::StateModuleInfo},
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

pub struct Module {
    pub chain_id: ChainId,

    pub ibc_handler_address: H160,

    pub rpc_url: String,
    // pub max_query_window: Option<u64>,
    // pub channel_cache: moka::future::Cache<ChannelId, Channel>,
    // pub connection_cache: moka::future::Cache<ConnectionId, Connection>,
    // pub client_address_cache: moka::future::Cache<u32, alloy::primitives::Address>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    /// The RPC endpoint for the execution chain.
    pub rpc_url: String,
    // #[serde(default)]
    // pub max_query_window: Option<u64>,

    // #[serde(default)]
    // pub max_cache_size: u32,
}

impl StateModule<IbcUnion> for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: StateModuleInfo) -> anyhow::Result<Self> {
        let provider =
            RpcClient::new_with_commitment(config.rpc_url.clone(), CommitmentConfig::finalized());

        let chain_id = provider.get_genesis_hash().await?.to_string();

        info.ensure_chain_id(&chain_id)?;

        Ok(Module {
            chain_id: ChainId::new(chain_id),
            ibc_handler_address: config.ibc_handler_address,
            rpc_url: config.rpc_url,
            // max_query_window: config.max_query_window,
            // should probably be big enough
            // channel_cache: moka::future::Cache::new(10_000),
            // connection_cache: moka::future::Cache::new(10_000),
            // client_address_cache: moka::future::Cache::new(10_000),
            // provider,
        })
    }
}

impl Module {
    #[must_use]
    pub fn make_height(&self, height: u64) -> Height {
        Height::new(height)
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %client_id))]
    async fn query_client_state(
        &self,
        height: Height,
        client_id: ClientId,
    ) -> RpcResult<Option<Bytes>> {
        todo!()
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %client_id, %trusted_height))]
    async fn query_consensus_state(
        &self,
        height: Height,
        client_id: ClientId,
        trusted_height: u64,
    ) -> RpcResult<Option<Bytes>> {
        todo!()
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %connection_id))]
    async fn query_connection(
        &self,
        height: Height,
        connection_id: ConnectionId,
    ) -> RpcResult<Option<Connection>> {
        let client =
            RpcClient::new_with_commitment(self.rpc_url.clone(), CommitmentConfig::finalized());

        client.get_account(todo!()).await.map_err(|e| {
            ErrorObject::owned(
                -1,
                ErrorReporter(e).with_message("error fetching connection account"),
                None::<()>,
            )
        })?;

        todo!()
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %channel_id))]
    async fn query_channel(
        &self,
        height: Height,
        channel_id: ChannelId,
    ) -> RpcResult<Option<Channel>> {
        todo!()
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height))]
    async fn query_batch_packets(
        &self,
        height: Height,
        batch_hash: H256,
    ) -> RpcResult<Option<H256>> {
        todo!()
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height))]
    async fn query_batch_receipts(
        &self,
        height: Height,
        batch_hash: H256,
    ) -> RpcResult<Option<H256>> {
        todo!()
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
        todo!()
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
        todo!()
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %channel_id, %packet_hash))]
    async fn packet_by_packet_hash(
        &self,
        channel_id: ChannelId,
        packet_hash: H256,
    ) -> RpcResult<PacketByHashResponse> {
        todo!()
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %channel_id, %batch_hash))]
    async fn packets_by_batch_hash(
        &self,
        channel_id: ChannelId,
        batch_hash: H256,
    ) -> RpcResult<PacketsByBatchHashResponse> {
        todo!()
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %channel_id, %packet_hash))]
    async fn packet_ack_by_packet_hash(
        &self,
        channel_id: ChannelId,
        packet_hash: H256,
    ) -> RpcResult<PacketAckByHashResponse> {
        todo!()
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %client_id, %height))]
    async fn client_status(&self, client_id: ClientId, height: u64) -> RpcResult<Status> {
        todo!()
    }
}

#[async_trait]
impl StateModuleServer<IbcUnion> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query(&self, e: &Extensions, query: Query) -> RpcResult<Value> {
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
            Query::PacketAckByHash(packet_ack_by_hash) => self
                .packet_ack_by_packet_hash(
                    packet_ack_by_hash.channel_id,
                    packet_ack_by_hash.packet_hash,
                )
                .await
                .map(into_value),
            Query::ClientStatus(client_status) => {
                let height = match client_status.height {
                    Some(height) => height,
                    None => e
                        .voyager_client()?
                        .query_latest_height(self.chain_id.clone(), false)
                        .await?
                        .height(),
                };

                self.client_status(client_status.client_id, height)
                    .await
                    .map(into_value)
            }
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
            StorePath::MembershipProof(path) => self
                .query_membership_proof(at, path.client_id, path.proof_height, path.path)
                .await
                .map(into_value),
            StorePath::NonMembershipProof(path) => self
                .query_non_membership_proof(at, path.client_id, path.proof_height, path.path)
                .await
                .map(into_value),
            StorePath::BatchTimeouts(path) => todo!(),
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn client_info(&self, _: &Extensions, client_id: ClientId) -> RpcResult<ClientInfo> {
        todo!()
    }
}

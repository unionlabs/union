use alloy::{
    eips::BlockNumberOrTag,
    network::AnyNetwork,
    providers::{DynProvider, Provider, ProviderBuilder},
};
use attested_light_client::query::LatestHeight;
use ibc_union_spec::Timestamp;
use jsonrpsee::{Extensions, core::async_trait};
use protos::cosmwasm::wasm::v1::{QuerySmartContractStateRequest, QuerySmartContractStateResponse};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use unionlabs::{
    ibc::core::client::height::Height,
    primitives::{Bech32, H256},
};
use voyager_sdk::{
    anyhow,
    plugin::FinalityModule,
    primitives::{ChainId, ConsensusType},
    rpc::{FinalityModuleServer, RpcError, RpcResult, types::FinalityModuleInfo},
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await;
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,
    pub attestation_client_address: Bech32<H256>,
    pub cometbft_client: cometbft_rpc::Client,
    pub provider: DynProvider<AnyNetwork>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub attestation_client_address: Bech32<H256>,
    pub cometbft_rpc_url: String,
    pub eth_rpc_url: String,
}

impl FinalityModule for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: FinalityModuleInfo) -> anyhow::Result<Self> {
        let provider = DynProvider::new(
            ProviderBuilder::new()
                .network::<AnyNetwork>()
                .connect(&config.eth_rpc_url)
                .await?,
        );

        info.ensure_chain_id(provider.get_chain_id().await?.to_string())?;
        info.ensure_consensus_type(ConsensusType::ATTESTED)?;

        Ok(Self {
            chain_id: info.chain_id,
            attestation_client_address: config.attestation_client_address,
            cometbft_client: cometbft_rpc::Client::new(config.cometbft_rpc_url).await?,
            provider,
        })
    }
}

impl Module {
    async fn query_latest_attested_height(&self) -> RpcResult<LatestHeight> {
        let req = QuerySmartContractStateRequest {
            address: self.attestation_client_address.to_string(),
            query_data: serde_json::to_vec(&attested_light_client::msg::QueryMsg::LatestHeight {
                chain_id: self.chain_id.to_string(),
            })
            .unwrap(),
        };

        let raw = self
            .cometbft_client
            .grpc_abci_query::<_, QuerySmartContractStateResponse>(
                "/cosmwasm.wasm.v1.Query/SmartContractState",
                &req,
                None,
                false,
            )
            .await
            .map_err(RpcError::retryable("error fetching latest attested height"))?
            .into_result()
            .map_err(RpcError::retryable("error fetching latest attested height"))?
            .unwrap()
            .data;

        Ok(serde_json::from_slice::<Option<LatestHeight>>(&raw)
            .map_err(RpcError::retryable("error fetching latest attested height"))?
            .unwrap())
    }
}

#[async_trait]
impl FinalityModuleServer for Module {
    /// Query the latest finalized height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id, finalized))]
    async fn query_latest_height(&self, _: &Extensions, finalized: bool) -> RpcResult<Height> {
        if finalized {
            Ok(Height::new(
                self.query_latest_attested_height().await?.height,
            ))
        } else {
            self.provider
                .get_block_number()
                .await
                .map(Height::new)
                .map_err(RpcError::retryable("error fetching latest block number"))
        }
    }

    /// Query the latest finalized timestamp of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id, finalized))]
    async fn query_latest_timestamp(
        &self,
        _: &Extensions,
        finalized: bool,
    ) -> RpcResult<Timestamp> {
        if finalized {
            Ok(self.query_latest_attested_height().await?.timestamp)
        } else {
            Ok(Timestamp::from_secs(
                self.provider
                    .get_block(BlockNumberOrTag::Latest.into())
                    .hashes()
                    .await
                    .map_err(RpcError::retryable("error fetching latest block"))?
                    .ok_or_else(|| RpcError::missing_state("latest block not found"))?
                    .header
                    .timestamp,
            ))
        }
    }
}

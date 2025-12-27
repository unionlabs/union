use std::{
    fmt::Debug,
    num::{NonZeroU64, ParseIntError},
};

use cometbls_light_client_types::{ClientState, ConsensusState};
use jsonrpsee::{Extensions, core::async_trait};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::instrument;
use unionlabs::{
    ibc::core::{client::height::Height, commitment::merkle_root::MerkleRoot},
    primitives::{Bech32, H256},
};
use voyager_sdk::{
    anyhow, ensure_null,
    plugin::ClientBootstrapModule,
    primitives::{ChainId, ClientType, Duration, Timestamp},
    rpc::{ClientBootstrapModuleServer, RpcError, RpcResult, types::ClientBootstrapModuleInfo},
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub cometbft_client: cometbft_rpc::Client,
    pub chain_revision: u64,

    pub ibc_host_contract_address: H256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub rpc_url: String,
    pub ibc_host_contract_address: Bech32<H256>,
}

impl ClientBootstrapModule for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: ClientBootstrapModuleInfo) -> anyhow::Result<Self> {
        let cometbft_client = cometbft_rpc::Client::new(config.rpc_url).await?;

        let chain_id = cometbft_client
            .status()
            .await?
            .node_info
            .network
            .to_string();

        info.ensure_chain_id(&chain_id)?;
        info.ensure_client_type(ClientType::COMETBLS)?;

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
            ibc_host_contract_address: *config.ibc_host_contract_address.data(),
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

impl Module {
    #[must_use]
    pub fn make_height(&self, height: u64) -> Height {
        Height::new_with_revision(self.chain_revision, height)
    }
}

#[async_trait]
impl ClientBootstrapModuleServer for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn self_client_state(
        &self,
        _: &Extensions,
        height: Height,
        config: Value,
    ) -> RpcResult<Value> {
        ensure_null(config)?;

        let params = self
            .cometbft_client
            .grpc_abci_query::<_, protos::cosmos::staking::v1beta1::QueryParamsResponse>(
                "/cosmos.staking.v1beta1.Query/Params",
                &protos::cosmos::staking::v1beta1::QueryParamsRequest {},
                None,
                false,
            )
            .await?
            .into_result()
            .map_err(RpcError::retryable("error fetching params"))?
            .ok_or_else(|| {
                RpcError::retryable_from_message("error fetching params: empty response")
            })?
            .params
            .unwrap_or_default();

        let commit = self
            .cometbft_client
            .commit(Some(NonZeroU64::new(height.height()).unwrap()))
            .await
            .unwrap();

        let height = commit.signed_header.header.height;

        // Expected to be nanos
        let unbonding_period =
            u64::try_from(params.unbonding_time.clone().unwrap().seconds).unwrap();

        // Avoid low unbonding period preventing relayer from submitting slightly old headers
        let unbonding_period = unbonding_period.max(3 * 24 * 3600);

        Ok(serde_json::to_value(ClientState {
            chain_id: cometbls_light_client_types::ChainId::from_string(self.chain_id.to_string())
                .unwrap(),
            trusting_period: Duration::from_secs(unbonding_period * 85 / 100),
            max_clock_drift: Duration::from_secs(60 * 20),
            frozen_height: Height::new(0),
            latest_height: Height::new_with_revision(
                self.chain_id
                    .as_str()
                    .split('-')
                    .next_back()
                    .unwrap()
                    .parse()
                    .unwrap(),
                height.inner().try_into().expect("value is >= 0; qed;"),
            ),
            contract_address: self.ibc_host_contract_address,
        })
        .unwrap())
    }

    /// The consensus state on this chain at the specified `Height`.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn self_consensus_state(
        &self,
        _: &Extensions,
        height: Height,
        config: Value,
    ) -> RpcResult<Value> {
        ensure_null(config)?;

        let commit = self
            .cometbft_client
            .commit(Some(NonZeroU64::new(height.height()).unwrap()))
            .await
            .unwrap();

        Ok(serde_json::to_value(ConsensusState {
            timestamp: Timestamp::from_nanos(commit.signed_header.header.time.as_unix_nanos()),
            app_hash: MerkleRoot {
                hash: commit.signed_header.header.app_hash.into_encoding(),
            },
            next_validators_hash: commit.signed_header.header.next_validators_hash,
        })
        .unwrap())
    }
}

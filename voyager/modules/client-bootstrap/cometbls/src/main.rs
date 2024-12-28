use std::{
    fmt::Debug,
    num::{NonZeroU64, ParseIntError},
};

use cometbls_light_client_types::{ClientState, ConsensusState};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{error, instrument};
use unionlabs::{
    bech32::Bech32,
    hash::H256,
    ibc::core::{client::height::Height, commitment::merkle_root::MerkleRoot},
    traits::Member,
};
use voyager_message::{
    core::{ChainId, ClientType},
    module::{ClientBootstrapModuleInfo, ClientBootstrapModuleServer},
    ClientBootstrapModule,
};
use voyager_vm::BoxDynError;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub tm_client: cometbft_rpc::Client,
    pub chain_revision: u64,
    pub grpc_url: String,

    pub ibc_host_contract_address: H256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ws_url: String,
    pub grpc_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ibc_host_contract_address: Option<Bech32<H256>>,
}

impl ClientBootstrapModule for Module {
    type Config = Config;

    async fn new(
        config: Self::Config,
        info: ClientBootstrapModuleInfo,
    ) -> Result<Self, BoxDynError> {
        let tm_client = cometbft_rpc::Client::new(config.ws_url).await?;

        let chain_id = tm_client.status().await?.node_info.network.to_string();

        info.ensure_chain_id(&chain_id)?;
        info.ensure_client_type(ClientType::COMETBLS)?;

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
            ibc_host_contract_address: config
                .ibc_host_contract_address
                .map(|a| *a.data())
                .unwrap_or_default(),
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
    async fn self_client_state(&self, _: &Extensions, height: Height) -> RpcResult<Value> {
        let params = protos::cosmos::staking::v1beta1::query_client::QueryClient::connect(
            self.grpc_url.clone(),
        )
        .await
        .unwrap()
        .params(protos::cosmos::staking::v1beta1::QueryParamsRequest {})
        .await
        .unwrap()
        .into_inner()
        .params
        .unwrap();

        let commit = self
            .tm_client
            .commit(Some(NonZeroU64::new(height.height()).unwrap()))
            .await
            .unwrap();

        let height = commit.signed_header.header.height;

        // Expected to be nanos
        let unbonding_period =
            u64::try_from(params.unbonding_time.clone().unwrap().seconds).unwrap() * 1_000_000_000;

        // Avoid low unbonding period preventing relayer from submitting slightly old headers
        let unbonding_period = unbonding_period.max(3 * 24 * 3600 * 1_000_000_000);

        Ok(serde_json::to_value(ClientState {
            chain_id: cometbls_light_client_types::ChainId::from_string(self.chain_id.to_string())
                .unwrap(),
            trusting_period: unbonding_period * 85 / 100,
            max_clock_drift: (60 * 20) * 1_000_000_000,
            frozen_height: Height::new(0),
            latest_height: Height::new_with_revision(
                self.chain_id
                    .as_str()
                    .split('-')
                    .last()
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
    async fn self_consensus_state(&self, _: &Extensions, height: Height) -> RpcResult<Value> {
        let commit = self
            .tm_client
            .commit(Some(NonZeroU64::new(height.height()).unwrap()))
            .await
            .unwrap();

        Ok(serde_json::to_value(ConsensusState {
            timestamp: commit.signed_header.header.time.as_unix_nanos(),
            app_hash: MerkleRoot {
                hash: commit.signed_header.header.app_hash.into_encoding(),
            },
            next_validators_hash: commit.signed_header.header.next_validators_hash,
        })
        .unwrap())
    }
}

#![warn(clippy::unwrap_used)]

use alloy::providers::{DynProvider, Provider, ProviderBuilder, layers::CacheLayer};
use ibc_union_spec::{IbcUnion, path::StorePath};
use jsonrpsee::{Extensions, core::async_trait};
use parlia_light_client_types::StateProof;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, instrument};
use unionlabs::{
    ethereum::ibc_commitment_key, ibc::core::client::height::Height, primitives::H160,
};
use voyager_sdk::{
    anyhow, into_value,
    plugin::ProofModule,
    primitives::ChainId,
    rpc::{ProofModuleServer, RpcError, RpcResult, types::ProofModuleInfo},
    types::ProofType,
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
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

impl ProofModule<IbcUnion> for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: ProofModuleInfo) -> anyhow::Result<Self> {
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
}

#[async_trait]
impl ProofModuleServer<IbcUnion> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id, %at, ?path))]
    async fn query_ibc_proof(
        &self,
        _: &Extensions,
        at: Height,
        path: StorePath,
    ) -> RpcResult<Option<(Value, ProofType)>> {
        let location = ibc_commitment_key(path.key());

        debug!(
            "querying proof for slot {location} for IBC handler contract {}",
            self.ibc_handler_address
        );

        let height = at.height();

        let proof = self
            .provider
            .get_proof(
                self.ibc_handler_address.get().into(),
                vec![location.to_be_bytes().into()],
            )
            .block_id(height.into())
            .await
            .map_err(RpcError::retryable("error fetching proof"))?;

        let storage_proof = match <[_; 1]>::try_from(proof.storage_proof) {
            Ok([proof]) => proof,
            Err(invalid) => {
                panic!(
                    "received invalid response from eth_getProof, expected length of 1 but got `{invalid:#?}`"
                );
            }
        };

        let proof = StateProof {
            account_proof: proof
                .account_proof
                .into_iter()
                .map(|bytes| bytes.into())
                .collect(),
            storage_hash: proof.storage_hash.into(),
            storage_proof: storage_proof
                .proof
                .into_iter()
                .map(|bytes| bytes.into())
                .collect(),
        };

        let proof_type = if storage_proof.value.is_zero() {
            ProofType::NonMembership
        } else {
            ProofType::Membership
        };

        Ok(Some((into_value(proof), proof_type)))
    }
}

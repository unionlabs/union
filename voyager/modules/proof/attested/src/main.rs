use attested_light_client::types::AttestationValue;
use attested_light_client_types::StorageProof;
use ibc_union_spec::{IbcUnion, path::StorePath};
use jsonrpsee::{Extensions, core::async_trait};
use protos::cosmwasm::wasm::v1::{QuerySmartContractStateRequest, QuerySmartContractStateResponse};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tracing::instrument;
use unionlabs::{
    ibc::core::client::height::Height,
    primitives::{Bech32, H256},
};
use voyager_sdk::{
    anyhow, into_value,
    plugin::ProofModule,
    primitives::ChainId,
    rpc::{ProofModuleServer, RpcError, RpcErrorExt, RpcResult, types::ProofModuleInfo},
    types::ProofType,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId,
    pub attestation_client_address: Bech32<H256>,
    pub rpc_url: String,
}

impl ProofModule<IbcUnion> for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: ProofModuleInfo) -> anyhow::Result<Self> {
        info.ensure_chain_id(config.chain_id.as_str())?;

        Ok(Self {
            chain_id: config.chain_id,
            attestation_client_address: config.attestation_client_address,
            cometbft_client: cometbft_rpc::Client::new(config.rpc_url).await?,
        })
    }
}

#[async_trait]
impl ProofModuleServer<IbcUnion> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_ibc_proof(
        &self,
        _: &Extensions,
        at: Height,
        path: StorePath,
    ) -> RpcResult<Option<(Value, ProofType)>> {
        let req = QuerySmartContractStateRequest {
            address: self.attestation_client_address.to_string(),
            query_data: serde_json::to_vec(&attested_light_client::msg::QueryMsg::AttestedValue {
                chain_id: self.chain_id.to_string(),
                height: at.height(),
                key: path.key().into(),
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
            .map_err(RpcError::retryable("error fetching attested value"))
            .with_data(json!({
                "chain_id": self.chain_id.to_string(),
                "height": at.height(),
                "key": path.key()
            }))?
            .into_result()
            .map_err(RpcError::retryable("error fetching attested value"))
            .with_data(json!({
                "chain_id": self.chain_id.to_string(),
                "height": at.height(),
                "key": path.key()
            }))?
            .unwrap()
            .data;

        Ok(
            match serde_json::from_slice::<AttestationValue>(&raw).unwrap() {
                AttestationValue::NonExistence => {
                    Some((into_value(StorageProof {}), ProofType::Membership))
                }
                AttestationValue::Existence(_) => {
                    Some((into_value(StorageProof {}), ProofType::NonMembership))
                }
            },
        )
    }
}

#![warn(clippy::unwrap_used)]

use alloy::{
    providers::{Provider, ProviderBuilder, RootProvider},
    transports::BoxTransport,
};
use ethereum_light_client_types::StorageProof;
use ibc_union_spec::{path::StorePath, IbcUnion};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, instrument};
use unionlabs::{
    ethereum::ibc_commitment_key,
    ibc::core::client::height::Height,
    primitives::{H160, U256},
    ErrorReporter,
};
use voyager_message::{
    core::ChainId,
    into_value,
    module::{ProofModuleInfo, ProofModuleServer},
    ProofModule,
};
use voyager_vm::BoxDynError;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub ibc_handler_address: H160,

    pub provider: RootProvider<BoxTransport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    /// The RPC endpoint for the execution chain.
    pub rpc_url: String,
}

impl ProofModule<IbcUnion> for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: ProofModuleInfo) -> Result<Self, BoxDynError> {
        let provider = ProviderBuilder::new().on_builtin(&config.rpc_url).await?;

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
    ) -> RpcResult<Value> {
        let location = ibc_commitment_key(path.key());

        debug!(
            "querying proof for slot {location} for IBC handler contract {}",
            self.ibc_handler_address
        );

        let execution_height = at.height();

        let proof = self
            .provider
            .get_proof(
                self.ibc_handler_address.get().into(),
                vec![location.to_be_bytes().into()],
            )
            .block_id(execution_height.into())
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching proof: {}", ErrorReporter(e)),
                    None::<()>,
                )
            })?;

        let proof = match <[_; 1]>::try_from(proof.storage_proof) {
            Ok([proof]) => proof,
            Err(invalid) => {
                panic!("received invalid response from eth_getProof, expected length of 1 but got `{invalid:#?}`");
            }
        };

        let proof = StorageProof {
            key: U256::from_be_bytes(proof.key.as_b256().0),
            value: U256::from_be_bytes(proof.value.to_be_bytes()),
            proof: proof.proof.into_iter().map(|bytes| bytes.into()).collect(),
        };

        Ok(into_value(proof))
    }
}

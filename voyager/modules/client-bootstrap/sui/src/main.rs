use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sui_light_client_types::{
    client_state::{ClientState, ClientStateV1},
    consensus_state::ConsensusState,
};
use sui_sdk::{types::base_types::ObjectID, SuiClient, SuiClientBuilder};
use tracing::instrument;
use unionlabs::ibc::core::client::height::Height;
use voyager_message::{
    ensure_null,
    module::{ClientBootstrapModuleInfo, ClientBootstrapModuleServer},
    primitives::{ChainId, ClientType},
    vm::BoxDynError,
    ClientBootstrapModule,
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub ibc_store: ObjectID,

    /// The address of the IBC smart contract.
    pub ibc_contract: ObjectID,

    pub sui_client: SuiClient,
}

impl ClientBootstrapModule for Module {
    type Config = Config;

    async fn new(
        config: Self::Config,
        info: ClientBootstrapModuleInfo,
    ) -> Result<Self, BoxDynError> {
        let sui_client = SuiClientBuilder::default().build(&config.rpc_url).await?;

        let chain_id = sui_client.read_api().get_chain_identifier().await?;

        info.ensure_chain_id(chain_id.to_string())?;
        info.ensure_client_type(ClientType::SUI)?;

        Ok(Self {
            chain_id: ChainId::new(chain_id.to_string()),
            ibc_contract: config.ibc_contract,
            ibc_store: config.ibc_store,
            sui_client,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The address of the `IBCHandler` smart contract.
    pub ibc_contract: ObjectID,

    pub ibc_store: ObjectID,

    pub rpc_url: String,
}

#[async_trait]
impl ClientBootstrapModuleServer for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn self_client_state(
        &self,
        _: &Extensions,
        _: Height,
        config: Value,
    ) -> RpcResult<Value> {
        ensure_null(config)?;

        let latest_checkpoint = self
            .sui_client
            .read_api()
            .get_latest_checkpoint_sequence_number()
            .await
            .unwrap();

        Ok(serde_json::to_value(ClientState::V1(ClientStateV1 {
            chain_id: self.chain_id.to_string(),
            latest_checkpoint,
            frozen_height: 0,
            initial_committee: None,
        }))
        .expect("infallible"))
    }

    /// The consensus state on this chain at the specified `Height`.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn self_consensus_state(
        &self,
        _: &Extensions,
        _height: Height,
        config: Value,
    ) -> RpcResult<Value> {
        ensure_null(config)?;

        Ok(serde_json::to_value(ConsensusState { timestamp: 1000 }).expect("infallible"))
    }
}

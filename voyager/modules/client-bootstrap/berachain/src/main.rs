use std::{fmt::Debug, num::ParseIntError};

use alloy::providers::{DynProvider, Provider, ProviderBuilder};
use berachain_light_client_types::{client_state::ClientStateV1, ClientState, ConsensusState};
use ibc_union_spec::ClientId;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{error, instrument};
use unionlabs::{ibc::core::client::height::Height, primitives::H160, ErrorReporter};
use voyager_message::{
    ensure_null, into_value,
    module::{ClientBootstrapModuleInfo, ClientBootstrapModuleServer},
    primitives::{ChainId, ClientType, Timestamp},
    ClientBootstrapModule, FATAL_JSONRPC_ERROR_CODE,
};
use voyager_vm::BoxDynError;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub provider: DynProvider,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    /// The RPC endpoint for the execution chain.
    pub rpc_url: String,
}

impl ClientBootstrapModule for Module {
    type Config = Config;

    async fn new(
        config: Self::Config,
        info: ClientBootstrapModuleInfo,
    ) -> Result<Self, BoxDynError> {
        let provider = DynProvider::new(ProviderBuilder::new().connect(&config.rpc_url).await?);

        let chain_id = provider.get_chain_id().await?;

        info.ensure_chain_id(chain_id.to_string())?;
        info.ensure_client_type(ClientType::BEACON_KIT)?;

        Ok(Self {
            chain_id: ChainId::new(chain_id.to_string()),
            provider,
            ibc_handler_address: config.ibc_handler_address,
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
        Height::new(height)
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

        Ok(into_value(ClientState::V1(ClientStateV1 {
            l1_client_id: config.l1_client_id,
            chain_id: self.chain_id.as_str().parse().unwrap(),
            latest_height: height.height(),
            ibc_contract_address: self.ibc_handler_address,
        })))
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

        let block = self
            .provider
            .get_block(height.height().into())
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    ErrorReporter(e).with_message("error fetching block"),
                    None::<()>,
                )
            })?
            .unwrap();

        Ok(into_value(ConsensusState {
            state_root: block.header.state_root.into(),
            storage_root: self
                .provider
                .get_proof(self.ibc_handler_address.into(), vec![])
                .block_id(block.header.number.into())
                .await
                .unwrap()
                .storage_hash
                .0
                .into(),
            timestamp: Timestamp::from_secs(block.header.timestamp),
        }))
    }
}

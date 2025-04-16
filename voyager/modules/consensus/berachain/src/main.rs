use std::fmt::Debug;

use alloy::providers::{DynProvider, Provider, ProviderBuilder};
use beacon_api_types::{chain_spec::Mainnet, deneb};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use unionlabs::{
    berachain::LATEST_EXECUTION_PAYLOAD_HEADER_PREFIX,
    encoding::{DecodeAs, Ssz},
    ibc::core::client::height::Height,
    primitives::H160,
};
use voyager_message::{
    module::{ConsensusModuleInfo, ConsensusModuleServer},
    primitives::{ChainId, ConsensusType, Timestamp},
    ConsensusModule, ExtensionsExt, VoyagerClient,
};
use voyager_vm::BoxDynError;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub l1_client_id: u32,
    pub l1_chain_id: ChainId,
    pub l2_chain_id: ChainId,
    pub ibc_handler_address: H160,
    pub eth_provider: DynProvider,
    pub tm_client: cometbft_rpc::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub l1_client_id: u32,
    pub l1_chain_id: ChainId,
    pub ibc_handler_address: H160,
    pub rpc_url: String,
    pub comet_ws_url: String,
}

impl ConsensusModule for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: ConsensusModuleInfo) -> Result<Self, BoxDynError> {
        let tm_client = cometbft_rpc::Client::new(config.comet_ws_url).await?;

        let eth_provider = DynProvider::new(ProviderBuilder::new().connect(&config.rpc_url).await?);

        let l2_chain_id = ChainId::new(eth_provider.get_chain_id().await?.to_string());

        info.ensure_chain_id(l2_chain_id.as_str())?;
        info.ensure_consensus_type(ConsensusType::BEACON_KIT)?;

        Ok(Self {
            l1_client_id: config.l1_client_id,
            l1_chain_id: config.l1_chain_id,
            l2_chain_id,
            ibc_handler_address: config.ibc_handler_address,
            eth_provider,
            tm_client,
        })
    }
}

#[async_trait]
impl ConsensusModuleServer for Module {
    /// Query the latest finalized height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.l2_chain_id))]
    async fn query_latest_height(&self, ext: &Extensions, finalized: bool) -> RpcResult<Height> {
        let voy_client = ext.try_get::<VoyagerClient>()?;
        if finalized {
            let l1_height = voy_client
                .query_latest_height(self.l1_chain_id.clone(), finalized)
                .await?;

            let raw_execution_header = self
                .tm_client
                .abci_query(
                    "store/beacon/key",
                    [LATEST_EXECUTION_PAYLOAD_HEADER_PREFIX],
                    // proof for height H must be queried at H-1
                    Some((l1_height.height() as i64 - 1).try_into().unwrap()),
                    false,
                )
                .await
                .unwrap();

            let execution_header = deneb::ExecutionPayloadHeaderSsz::<Mainnet>::decode_as::<Ssz>(
                raw_execution_header
                    .response
                    .value
                    .expect("big trouble")
                    .as_ref(),
            )
            .unwrap();

            Ok(Height::new(execution_header.block_number))
        } else {
            Ok(Height::new(
                self.eth_provider
                    .get_block_number()
                    .await
                    .expect("big trouble"),
            ))
        }
    }

    /// Query the latest finalized timestamp of this chain.
    #[instrument(skip_all, fields(chain_id = %self.l2_chain_id))]
    async fn query_latest_timestamp(
        &self,
        ext: &Extensions,
        finalized: bool,
    ) -> RpcResult<Timestamp> {
        let latest_height = self.query_latest_height(ext, finalized).await?;
        let latest_block = self
            .eth_provider
            .get_block_by_number(latest_height.height().into())
            .hashes()
            .await
            .expect("big trouble")
            .expect("big trouble");
        // Normalize to nanos in order to be compliant with cosmos
        Ok(Timestamp::from_secs(latest_block.header.timestamp))
    }

    // #[instrument(skip_all, fields(chain_id = %self.l2_chain_id))]
    // async fn self_client_state(&self, _: &Extensions, height: Height) -> RpcResult<Value> {
    //     Ok(into_value(ClientState {
    //         l1_client_id: self.l1_client_id,
    //         chain_id: self
    //             .l2_chain_id
    //             .as_str()
    //             .parse()
    //             .expect("self.chain_id is a valid u256"),
    //         latest_height: height.height(),
    //         ibc_contract_address: self.ibc_handler_address,
    //     }))
    // }

    // /// The consensus state on this chain at the specified `Height`.
    // #[instrument(skip_all, fields(chain_id = %self.l2_chain_id))]
    // async fn self_consensus_state(&self, _: &Extensions, height: Height) -> RpcResult<Value> {
    //     let block = self
    //         .eth_provider
    //         .get_block_by_number(
    //             height.height().into(),
    //             alloy::rpc::types::BlockTransactionsKind::Hashes,
    //         )
    //         .await
    //         .expect("big trouble")
    //         .expect("big trouble");
    //     Ok(into_value(&ConsensusState {
    //         // Normalize to nanos in order to be compliant with cosmos
    //         timestamp: block.header.timestamp * 1_000_000_000,
    //         state_root: block.header.state_root.into(),
    //         storage_root: self
    //             .eth_provider
    //             .get_proof(self.ibc_handler_address.into(), vec![])
    //             .block_id(height.height().into())
    //             .await
    //             .unwrap()
    //             .storage_hash
    //             .0
    //             .into(),
    //     }))
    // }
}

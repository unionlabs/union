// #![warn(clippy::unwrap_used)] // oh boy this will be a lot of work

use std::sync::Arc;

use alloy::{
    providers::{Provider, ProviderBuilder, RootProvider},
    transports::BoxTransport,
};
use beacon_api::client::BeaconApiClient;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, instrument};
use unionlabs::{
    ethereum::{ibc_commitment_key, IBC_HANDLER_COMMITMENTS_SLOT},
    hash::H160,
    ibc::{core::client::height::Height, lightclients::ethereum::storage_proof::StorageProof},
    ics24::{ClientStatePath, Path},
    id::ClientId,
    uint::U256,
    ErrorReporter,
};
use voyager_message::{
    core::{ChainId, ClientInfo, ClientType, IbcInterface},
    module::{ChainModuleInfo, ChainModuleServer, RawClientState},
    run_chain_module_server, ChainModule,
};
use voyager_vm::BoxDynError;

const ETHEREUM_REVISION_NUMBER: u64 = 0;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_chain_module_server::<Module>().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId<'static>,

    pub ibc_handler_address: H160,

    pub provider: RootProvider<BoxTransport>,
    pub beacon_api_client: BeaconApiClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    /// The RPC endpoint for the execution chain.
    pub eth_rpc_api: String,
    /// The RPC endpoint for the beacon chain.
    pub eth_beacon_rpc_api: String,
}

impl ChainModule for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: ChainModuleInfo) -> Result<Self, BoxDynError> {
        let provider = ProviderBuilder::new()
            .on_builtin(&config.eth_rpc_api)
            .await?;

        let chain_id = provider.get_chain_id().await?;

        info.ensure_chain_id(chain_id.to_string())?;

        Ok(Module {
            chain_id: ChainId::new(chain_id.to_string()),
            ibc_handler_address: config.ibc_handler_address,
            provider,
            beacon_api_client: BeaconApiClient::new(config.eth_beacon_rpc_api).await?,
        })
    }
}

impl Module {
    #[must_use]
    pub fn make_height(&self, height: u64) -> Height {
        Height::new(height)
    }

    fn ibc_handler(&self) -> IBCHandler<Provider<Ws>> {
        IBCHandler::new(self.ibc_handler_address, Arc::new(self.provider.clone()))
    }

    #[instrument(skip(self))]
    pub async fn execution_height_of_beacon_slot(&self, slot: u64) -> u64 {
        let execution_height = self
            .beacon_api_client
            .execution_height(beacon_api::client::BlockId::Slot(slot))
            .await
            .unwrap();

        debug!("beacon slot {slot} is execution height {execution_height}");

        execution_height
    }

    #[instrument(skip_all, fields(%path, %height))]
    pub async fn fetch_ibc_state(&self, path: Path, height: Height) -> Result<Value, BoxDynError> {
        let execution_height = self.execution_height_of_beacon_slot(height.height()).await;

        Ok(match path {
            Path::ClientState(path) => serde_json::to_value(
                self.ibc_handler()
                    .ibc_state_read(execution_height, path.clone())
                    .await
                    .unwrap(),
            )
            .unwrap(),
            Path::ClientConsensusState(path) => serde_json::to_value(
                self.ibc_handler()
                    .ibc_state_read(execution_height, path.clone())
                    .await
                    .unwrap(),
            )
            .unwrap(),
            Path::Connection(path) => serde_json::to_value(
                self.ibc_handler()
                    .ibc_state_read(execution_height, path.clone())
                    .await
                    .unwrap(),
            )
            .unwrap(),
            Path::ChannelEnd(path) => serde_json::to_value(
                self.ibc_handler()
                    .ibc_state_read(execution_height, path.clone())
                    .await
                    .unwrap(),
            )
            .unwrap(),
            Path::Commitment(path) => serde_json::to_value(
                self.ibc_handler()
                    .ibc_state_read(execution_height, path.clone())
                    .await
                    .unwrap(),
            )
            .unwrap(),
            Path::Acknowledgement(path) => serde_json::to_value(
                self.ibc_handler()
                    .ibc_state_read(execution_height, path.clone())
                    .await
                    .unwrap(),
            )
            .unwrap(),
            Path::Receipt(path) => serde_json::to_value(
                self.ibc_handler()
                    .ibc_state_read(execution_height, path.clone())
                    .await
                    .unwrap(),
            )
            .unwrap(),
            Path::NextSequenceSend(path) => serde_json::to_value(
                self.ibc_handler()
                    .ibc_state_read(execution_height, path.clone())
                    .await
                    .unwrap(),
            )
            .unwrap(),
            Path::NextSequenceRecv(path) => serde_json::to_value(
                self.ibc_handler()
                    .ibc_state_read(execution_height, path.clone())
                    .await
                    .unwrap(),
            )
            .unwrap(),
            Path::NextSequenceAck(path) => serde_json::to_value(
                self.ibc_handler()
                    .ibc_state_read(execution_height, path.clone())
                    .await
                    .unwrap(),
            )
            .unwrap(),
            Path::NextConnectionSequence(path) => serde_json::to_value(
                self.ibc_handler()
                    .ibc_state_read(execution_height, path.clone())
                    .await
                    .unwrap(),
            )
            .unwrap(),
            Path::NextClientSequence(path) => serde_json::to_value(
                self.ibc_handler()
                    .ibc_state_read(execution_height, path.clone())
                    .await
                    .unwrap(),
            )
            .unwrap(),
        })
    }
}

#[async_trait]
impl ChainModuleServer for Module {
    /// Query the latest finalized height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_latest_height(&self, _: &Extensions) -> RpcResult<Height> {
        self.beacon_api_client
            .finality_update()
            .await
            .map(|response| self.make_height(response.data.attested_header.beacon.slot))
            .map_err(|err| ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>))
    }

    /// Query the latest (non-finalized) height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_latest_height_as_destination(&self, _: &Extensions) -> RpcResult<Height> {
        let height = self
            .beacon_api_client
            .block(beacon_api::client::BlockId::Head)
            .await
            .map_err(|err| ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>))?
            .data
            .message
            .slot;

        // // HACK: we introduced this because we were using alchemy for the
        // // execution endpoint and our custom beacon endpoint that rely on
        // // its own execution chain. Alchemy was a bit delayed and the
        // // execution height for the beacon head wasn't existing for few
        // // secs. We wait for an extra beacon head to let alchemy catch up.
        // // We should be able to remove that once we rely on an execution
        // // endpoint that is itself used by the beacon endpoint (no different
        // // POV).
        // loop {
        //     let next_height = self
        //         .beacon_api_client
        //         .block(beacon_api::client::BlockId::Head)
        //         .await?
        //         .data
        //         .message
        //         .slot;
        //     if next_height > height {
        //         break;
        //     }

        //     tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        // }

        Ok(self.make_height(height))
    }

    /// Query the latest finalized timestamp of this chain.
    // TODO: Use a better timestamp type here
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_latest_timestamp(&self, _: &Extensions) -> RpcResult<i64> {
        Ok(self
            .beacon_api_client
            .finality_update()
            .await
            .map_err(|err| ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>))?
            .data
            .attested_header
            .execution
            .timestamp
            .try_into()
            .unwrap())
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn client_info(&self, _: &Extensions, client_id: ClientId) -> RpcResult<ClientInfo> {
        Ok(ClientInfo {
            client_type: ClientType::new(
                self.ibc_handler()
                    .client_types(client_id.to_string())
                    .await
                    .unwrap(),
            ),
            ibc_interface: IbcInterface::new(IbcInterface::IBC_SOLIDITY),
            metadata: Default::default(),
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_ibc_state(&self, _: &Extensions, at: Height, path: Path) -> RpcResult<Value> {
        self.fetch_ibc_state(path, at).await.map_err(|err| {
            ErrorObject::owned(
                -1,
                format!("error fetching ibc state: {}", ErrorReporter(&*err)),
                None::<()>,
            )
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_ibc_proof(&self, _: &Extensions, at: Height, path: Path) -> RpcResult<Value> {
        let location = ibc_commitment_key(&path.to_string(), IBC_HANDLER_COMMITMENTS_SLOT);

        let execution_height = self.execution_height_of_beacon_slot(at.height()).await;

        let proof = self
            .provider
            .get_proof(
                self.ibc_handler_address.into(),
                vec![location.to_be_bytes().into()],
            )
            .block_id(execution_height.into())
            .await
            .unwrap();

        let proof = match <[_; 1]>::try_from(proof.storage_proof) {
            Ok([proof]) => proof,
            Err(invalid) => {
                panic!("received invalid response from eth_getProof, expected length of 1 but got `{invalid:#?}`");
            }
        };

        let proof = StorageProof {
            key: proof.key.0.into(),
            value: proof.value.into(),
            proof: proof
                .proof
                .into_iter()
                .map(|bytes| bytes.to_vec())
                .collect(),
        };

        Ok(serde_json::to_value(proof).expect("serialization is infallible; qed;"))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_raw_unfinalized_trusted_client_state(
        &self,
        e: &Extensions,
        client_id: ClientId,
    ) -> RpcResult<RawClientState<'static>> {
        let latest_execution_height = self.provider.get_block_number().await.unwrap().as_u64();

        let ClientInfo {
            client_type,
            ibc_interface,
            metadata: _,
        } = self.client_info(e, client_id.clone()).await?;

        Ok(RawClientState {
            client_type,
            ibc_interface,
            bytes: self
                .ibc_handler()
                .ibc_state_read(latest_execution_height, ClientStatePath { client_id })
                .await
                .unwrap()
                .0
                .into(),
        })
    }
}

// type Pls = <(<Module as ModuleContext>::Info, Module) as voyager_message::module::IntoRpc<
//     ModuleData,
//     ModuleCall,
//     ModuleCallback,
//     // RpcModule = ModuleServer<ModuleContext>,
// >>::RpcModule;

// static_assertions::assert_type_eq_all!(Pls, Module);

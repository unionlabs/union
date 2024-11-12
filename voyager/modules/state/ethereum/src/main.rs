// #![warn(clippy::unwrap_used)] // oh boy this will be a lot of work

use alloy::{
    providers::{Provider, ProviderBuilder, RootProvider},
    transports::BoxTransport,
};
use beacon_api::client::BeaconApiClient;
use ibc_solidity::ibc::{
    Channel, Connection, ILightClient,
    Ibc::{self, IbcInstance},
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, info, instrument};
use unionlabs::{
    bytes::Bytes,
    hash::{H160, H256},
    ibc::core::client::height::Height,
    ics24::ethabi::{BatchPacketsPath, BatchReceiptsPath, Path},
    ErrorReporter,
};
use voyager_message::{
    core::{ChainId, ClientInfo, ClientType, IbcInterface},
    ibc_union::IbcUnion,
    into_value,
    module::{StateModuleInfo, StateModuleServer},
    StateModule, FATAL_JSONRPC_ERROR_CODE,
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

impl StateModule<IbcUnion> for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: StateModuleInfo) -> Result<Self, BoxDynError> {
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

    fn ibc_handler(&self) -> IbcInstance<BoxTransport, RootProvider<BoxTransport>> {
        Ibc::new(self.ibc_handler_address.get().into(), self.provider.clone())
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

    #[instrument(skip(self))]
    pub async fn client_address(
        &self,
        client_id: u32,
        height: u64,
    ) -> RpcResult<alloy::primitives::Address> {
        let client_address = self
            .ibc_handler()
            .clientImpls(client_id)
            .block(height.into())
            .call()
            .await
            .map_err(|err| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching client address: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })?
            ._0;

        info!(%client_address, "fetched client address");

        Ok(client_address)
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %client_id))]
    async fn query_client_state(&self, height: Height, client_id: u32) -> RpcResult<Bytes> {
        let execution_height = self.execution_height_of_beacon_slot(height.height()).await;

        let client_address = self.client_address(client_id, execution_height).await?;

        let light_client = ILightClient::new(client_address, self.provider.clone());
        let client_state = light_client
            .getClientState(client_id)
            .block(execution_height.into())
            .call()
            .await
            .map_err(|err| {
                ErrorObject::owned(
                    match err {
                        alloy::contract::Error::AbiError(_) => FATAL_JSONRPC_ERROR_CODE,
                        _ => -1,
                    },
                    format!("error fetching client state: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })?
            ._0
            .0;

        Ok(client_state.to_vec().into())
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %client_id, %trusted_height))]
    async fn query_consensus_state(
        &self,
        height: Height,
        client_id: u32,
        trusted_height: u64,
    ) -> RpcResult<Bytes> {
        let execution_height = self.execution_height_of_beacon_slot(height.height()).await;

        let client_address = self.client_address(client_id, execution_height).await?;

        let light_client = ILightClient::new(client_address, self.provider.clone());

        let consensus_state = light_client
            .getConsensusState(client_id, trusted_height)
            .block(execution_height.into())
            .call()
            .await
            .map_err(|err| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching consensus state: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })?
            ._0
            .0;

        Ok(consensus_state.to_vec().into())
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %connection_id))]
    async fn query_connection(
        &self,
        height: Height,
        connection_id: u32,
    ) -> RpcResult<Option<Connection>> {
        let execution_height = self.execution_height_of_beacon_slot(height.height()).await;

        let ibc_handler = self.ibc_handler();

        let raw = ibc_handler
            .connections(connection_id)
            .block(execution_height.into())
            .call()
            .await
            .map_err(|err| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching connection: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })?
            ._0;

        Ok(Some(raw))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %channel_id))]
    async fn query_channel(&self, height: Height, channel_id: u32) -> RpcResult<Option<Channel>> {
        let execution_height = self.execution_height_of_beacon_slot(height.height()).await;

        let ibc_handler = self.ibc_handler();

        let raw = ibc_handler
            .channels(channel_id)
            .block(execution_height.into())
            .call()
            .await
            .map_err(|err| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching channel: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })?
            ._0;

        Ok(Some(raw))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %channel_id))]
    async fn query_batch_packets(
        &self,
        height: Height,
        channel_id: u32,
        batch_hash: H256,
    ) -> RpcResult<Option<H256>> {
        let execution_height = self.execution_height_of_beacon_slot(height.height()).await;

        let ibc_handler = self.ibc_handler();

        let raw = ibc_handler
            .commitments(
                BatchPacketsPath {
                    channel_id,
                    batch_hash,
                }
                .key()
                .into(),
            )
            .block(execution_height.into())
            .call()
            .await
            .map_err(|err| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching batch commitments: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })?
            ._0;

        Ok(Some(raw.into()))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %channel_id))]
    async fn query_batch_receipts(
        &self,
        height: Height,
        channel_id: u32,
        batch_hash: H256,
    ) -> RpcResult<Option<H256>> {
        let execution_height = self.execution_height_of_beacon_slot(height.height()).await;

        let ibc_handler = self.ibc_handler();

        let raw = ibc_handler
            .commitments(
                BatchReceiptsPath {
                    channel_id,
                    batch_hash,
                }
                .key()
                .into(),
            )
            .block(execution_height.into())
            .call()
            .await
            .map_err(|err| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching batch receipts: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })?
            ._0;

        Ok(Some(raw.into()))
    }
}

#[async_trait]
impl StateModuleServer<IbcUnion> for Module {
    async fn query_ibc_state(&self, _: &Extensions, at: Height, path: Path) -> RpcResult<Value> {
        match path {
            Path::ClientState(path) => self
                .query_client_state(at, path.client_id)
                .await
                .map(into_value),
            Path::ConsensusState(path) => self
                .query_consensus_state(at, path.client_id, path.height)
                .await
                .map(into_value),
            Path::Connection(path) => self
                .query_connection(at, path.connection_id)
                .await
                .map(into_value),
            Path::Channel(path) => self
                .query_channel(at, path.channel_id)
                .await
                .map(into_value),
            Path::BatchReceipts(path) => self
                .query_batch_receipts(at, path.channel_id, path.batch_hash)
                .await
                .map(into_value),
            Path::BatchPackets(path) => self
                .query_batch_packets(at, path.channel_id, path.batch_hash)
                .await
                .map(into_value),
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn client_info(&self, _: &Extensions, client_id: u32) -> RpcResult<ClientInfo> {
        let ibc_handler = self.ibc_handler();
        let client_type = ibc_handler.clientTypes(client_id).call().await.unwrap()._0;
        Ok(ClientInfo {
            client_type: ClientType::new(client_type),
            ibc_interface: IbcInterface::new(IbcInterface::IBC_SOLIDITY),
            metadata: Default::default(),
        })
    }

    // #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    // async fn query_ibc_proof(
    //     &self,
    //     _: &Extensions,
    //     at: Height,
    //     path: Path,
    //     // ibc_store_format: IbcStoreFormat<'static>,
    // ) -> RpcResult<Value> {
    //     let location = ibc_commitment_key(match path {
    //         Path::ClientState(path) => ethabi::client_state_key(path.client_id.id()),
    //         Path::ClientConsensusState(path) => {
    //             ethabi::consensus_state_key(path.client_id.id(), path.height.height())
    //         }
    //         Path::Connection(path) => ethabi::connection_key(path.connection_id.id()),
    //         Path::ChannelEnd(path) => ethabi::channel_key(path.channel_id.id()),
    //         Path::Commitment(_path) => {
    //             todo!()
    //             // ethabi::commitments_key(path.channel_id.id(), path.sequence.get())
    //         }
    //         Path::Acknowledgement(_path) => {
    //             todo!()
    //             // ethabi::acknowledgements_key(path.channel_id.id(), path.sequence.get())
    //         }
    //         Path::Receipt(_path) => {
    //             todo!()
    //             // ethabi::receipts_key(path.channel_id.id(), path.sequence.get())
    //         }
    //         Path::NextSequenceSend(_path) => todo!(),
    //         Path::NextSequenceRecv(_path) => todo!(),
    //         Path::NextSequenceAck(_path) => todo!(),
    //         Path::NextConnectionSequence(_path) => todo!(),
    //         Path::NextClientSequence(_path) => todo!(),
    //     });

    //     let execution_height = self.execution_height_of_beacon_slot(at.height()).await;

    //     let proof = self
    //         .provider
    //         .get_proof(
    //             self.ibc_handler_address.get().into(),
    //             vec![location.to_be_bytes().into()],
    //         )
    //         .block_id(execution_height.into())
    //         .await
    //         .unwrap();

    //     let proof = match <[_; 1]>::try_from(proof.storage_proof) {
    //         Ok([proof]) => proof,
    //         Err(invalid) => {
    //             panic!("received invalid response from eth_getProof, expected length of 1 but got `{invalid:#?}`");
    //         }
    //     };

    //     let proof = StorageProof {
    //         key: U256::from_be_bytes(proof.key.0 .0),
    //         value: U256::from_be_bytes(proof.value.to_be_bytes()),
    //         proof: proof
    //             .proof
    //             .into_iter()
    //             .map(|bytes| bytes.to_vec())
    //             .collect(),
    //     };

    //     Ok(serde_json::to_value(proof).expect("serialization is infallible; qed;"))
    // }
}

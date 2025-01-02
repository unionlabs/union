// #![warn(clippy::unwrap_used)] // oh boy this will be a lot of work

use alloy::{
    providers::{Provider, ProviderBuilder, RootProvider},
    rpc::types::{TransactionInput, TransactionRequest},
    sol_types::{SolCall, SolValue},
    transports::BoxTransport,
};
use ibc_solidity::{
    Channel, Connection, ILightClient,
    Ibc::{self, IbcInstance},
};
use ibc_union_spec::{BatchPacketsPath, BatchReceiptsPath, IbcUnion, StorePath};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, instrument};
use unionlabs::{
    ibc::core::client::height::Height,
    primitives::{Bytes, H160, H256},
    ErrorReporter,
};
use voyager_message::{
    core::{ChainId, ClientInfo, ClientType, IbcInterface},
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    /// The RPC endpoint for the execution chain.
    pub eth_rpc_api: String,
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
        let execution_height = height.height();

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
        let execution_height = height.height();

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
        let execution_height = height.height();

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
        let execution_height = height.height();

        let ibc_handler = self.ibc_handler();

        // https://github.com/alloy-rs/core/issues/811
        // let raw = ibc_handler
        //     .channels(channel_id)
        //     .block(execution_height.into())
        //     .call()
        //     .await
        //     .map_err(|err| {
        //         ErrorObject::owned(
        //             -1,
        //             format!("error fetching channel: {}", ErrorReporter(err)),
        //             None::<()>,
        //         )
        //     })?
        //     ._0;

        let raw = ibc_handler
            .provider()
            .call(&TransactionRequest {
                from: None,
                to: Some(alloy::primitives::Address::from(self.ibc_handler_address).into()),
                input: TransactionInput::new(
                    Ibc::channelsCall { _0: channel_id }.abi_encode().into(),
                ),
                ..Default::default()
            })
            .block(execution_height.into())
            .await
            .unwrap();

        dbg!(&raw);

        let channel = ibc_solidity::Channel::abi_decode_params(&raw, true).unwrap();

        Ok(Some(channel))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %channel_id))]
    async fn query_batch_packets(
        &self,
        height: Height,
        channel_id: u32,
        batch_hash: H256,
    ) -> RpcResult<Option<H256>> {
        let execution_height = height.height();

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
        let execution_height = height.height();

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
    async fn query_ibc_state(
        &self,
        _: &Extensions,
        at: Height,
        path: StorePath,
    ) -> RpcResult<Value> {
        match path {
            StorePath::ClientState(path) => self
                .query_client_state(at, path.client_id)
                .await
                .map(into_value),
            StorePath::ConsensusState(path) => self
                .query_consensus_state(at, path.client_id, path.height)
                .await
                .map(into_value),
            StorePath::Connection(path) => self
                .query_connection(at, path.connection_id)
                .await
                .map(into_value),
            StorePath::Channel(path) => self
                .query_channel(at, path.channel_id)
                .await
                .map(into_value),
            StorePath::BatchReceipts(path) => self
                .query_batch_receipts(at, path.channel_id, path.batch_hash)
                .await
                .map(into_value),
            StorePath::BatchPackets(path) => self
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
}

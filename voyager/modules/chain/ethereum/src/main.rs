// #![warn(clippy::unwrap_used)] // oh boy this will be a lot of work

use std::{num::NonZeroU64, sync::Arc};

use alloy::{
    providers::{Provider, ProviderBuilder, RootProvider},
    transports::BoxTransport,
};
use beacon_api::client::BeaconApiClient;
use ibc_solidity::ibc::{
    ChannelOrder, ChannelState, ConnectionState, ILightClient,
    Ibc::{self, IbcInstance},
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_utils::Hex;
use tracing::{debug, instrument};
use unionlabs::{
    ethereum::{ibc_commitment_key, IBC_HANDLER_COMMITMENTS_SLOT},
    hash::{H160, H256},
    ibc::{
        core::{
            channel::{self, channel::Channel, order::Order},
            client::height::Height,
            connection::{self, connection_end::ConnectionEnd, version::Version},
        },
        lightclients::ethereum::storage_proof::StorageProof,
    },
    ics24::{ClientStatePath, CommitmentPath, Path, ReceiptPath},
    id::{ChannelId, ClientId, ConnectionId, PortId},
    uint::U256,
    ErrorReporter,
};
use voyager_message::{
    core::{ChainId, ClientInfo, ClientType, IbcInterface, IbcStoreFormat},
    module::{ChainModuleInfo, ChainModuleServer, RawClientState},
    rpc::{ChannelInfo, ConnectionInfo},
    run_chain_module_server,
    valuable::Valuable,
    ChainModule,
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
        let ibc_handler = self.ibc_handler();
        let client_type = ibc_handler
            .clientTypes(client_id.id())
            .call()
            .await
            .unwrap()
            ._0;
        Ok(ClientInfo {
            client_type: ClientType::new(client_type),
            ibc_interface: IbcInterface::new(IbcInterface::IBC_SOLIDITY),
            metadata: Default::default(),
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, client_id = client_id.as_value()))]
    async fn query_client_state(
        &self,
        _: &Extensions,
        height: Height,
        client_id: ClientId,
    ) -> RpcResult<Hex<Vec<u8>>> {
        let execution_height = self.execution_height_of_beacon_slot(height.height()).await;

        let ibc_handler = self.ibc_handler();

        let client_address = ibc_handler
            .clientImpls(client_id.id())
            .block(execution_height.into())
            .call()
            .await
            .map_err(|err| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching ibc state: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })?
            ._0;

        let light_client = ILightClient::new(client_address, self.provider.clone());
        let client_state = light_client
            .getClientState(client_id.id())
            .block(execution_height.into())
            .call()
            .await
            .map_err(|err| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching ibc state: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })?
            ._0
            .0;

        Ok(Hex(client_state.to_vec()))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, client_id = client_id.as_value(), %trusted_height))]
    async fn query_client_consensus_state(
        &self,
        _: &Extensions,
        height: Height,
        client_id: ClientId,
        trusted_height: Height,
    ) -> RpcResult<Hex<Vec<u8>>> {
        let execution_height = self.execution_height_of_beacon_slot(height.height()).await;

        let ibc_handler = self.ibc_handler();

        let client_address = ibc_handler
            .clientImpls(client_id.id())
            .block(execution_height.into())
            .call()
            .await
            .map_err(|err| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching ibc state: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })?
            ._0;

        let light_client = ILightClient::new(client_address, self.provider.clone());

        let consensus_state = light_client
            .getConsensusState(client_id.id(), trusted_height.height())
            .block(execution_height.into())
            .call()
            .await
            .map_err(|err| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching ibc state: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })?
            ._0
            .0;

        Ok(Hex(consensus_state.to_vec()))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, connection_id = connection_id.as_value()))]
    async fn query_connection(
        &self,
        _: &Extensions,
        height: Height,
        connection_id: ConnectionId,
    ) -> RpcResult<Option<ConnectionInfo>> {
        let execution_height = self.execution_height_of_beacon_slot(height.height()).await;

        let ibc_handler = self.ibc_handler();

        let raw = ibc_handler
            .connections(connection_id.id())
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

        Ok(Some(ConnectionInfo {
            client_id: ClientId::new(raw.clientId),
            versions: vec![Version {
                identifier: "1".to_owned(),
                features: vec![Order::Ordered, Order::Unordered],
            }],
            state: match raw.state {
                ConnectionState::Unspecified => connection::state::State::UninitializedUnspecified,
                ConnectionState::Init => connection::state::State::Init,
                ConnectionState::TryOpen => connection::state::State::Tryopen,
                ConnectionState::Open => connection::state::State::Open,
                _ => todo!(),
            },
            counterparty: connection::counterparty::Counterparty {
                client_id: ClientId::new(raw.counterparty.clientId),
                connection_id: Some(ConnectionId::new(raw.counterparty.connectionId)),
                prefix: todo!(),
            },
            delay_period: 0,
        }))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, channel_id = channel_id.as_value()))]
    async fn query_channel(
        &self,
        _: &Extensions,
        height: Height,
        channel_id: ChannelId,
    ) -> RpcResult<Option<ChannelInfo>> {
        let execution_height = self.execution_height_of_beacon_slot(height.height()).await;

        let ibc_handler = self.ibc_handler();

        let raw = ibc_handler
            .channels(channel_id.id())
            .block(execution_height.into())
            .call()
            .await
            .map_err(|err| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching ibc state: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })?
            ._0;

        let port_id = ibc_handler
            .channelOwner(channel_id.id())
            .block(execution_height.into())
            .call()
            .await
            .map_err(|err| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching ibc state: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })?
            ._0;

        Ok(Some(ChannelInfo {
            port_id: PortId::new(port_id.to_string()).unwrap(),
            state: match raw.state {
                ChannelState::Unspecified => channel::state::State::UninitializedUnspecified,
                ChannelState::Init => channel::state::State::Init,
                ChannelState::TryOpen => channel::state::State::Tryopen,
                ChannelState::Open => channel::state::State::Open,
                ChannelState::Closed => channel::state::State::Closed,
                _ => todo!(),
            },
            ordering: match raw.ordering {
                ChannelOrder::Unspecified => Order::NoneUnspecified,
                ChannelOrder::Unordered => Order::Unordered,
                ChannelOrder::Ordered => Order::Ordered,
                _ => todo!(),
            },
            counterparty_channel_id: Some(ChannelId::new(raw.counterparty.channelId)),
            connection_hops: vec![ConnectionId::new(raw.connectionId)],
            version: <H256>::from(raw.version).to_string(),
        }))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, channel_id = channel_id.as_value(), %sequence))]
    async fn query_commitment(
        &self,
        _: &Extensions,
        height: Height,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<Option<H256>> {
        let execution_height = self.execution_height_of_beacon_slot(height.height()).await;

        let ibc_handler = self.ibc_handler();

        let raw: H256 = ibc_handler
            .commitments(
                CommitmentPath {
                    channel_id,
                    sequence,
                }
                .commitments_key()
                .into(),
            )
            .block(execution_height.into())
            .call()
            .await
            .map_err(|err| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching ibc state: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })?
            ._0
            .into();

        Ok(if raw == <H256>::default() {
            None
        } else {
            Some(raw)
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, channel_id = channel_id.as_value(), %sequence))]
    async fn query_acknowledgement(
        &self,
        _: &Extensions,
        height: Height,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<Option<H256>> {
        let execution_height = self.execution_height_of_beacon_slot(height.height()).await;

        let ibc_handler = self.ibc_handler();

        let raw: H256 = ibc_handler
            .commitments(
                CommitmentPath {
                    channel_id,
                    sequence,
                }
                .commitments_key()
                .into(),
            )
            .block(execution_height.into())
            .call()
            .await
            .map_err(|err| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching ibc state: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })?
            ._0
            .into();

        Ok(if raw == <H256>::default() {
            None
        } else {
            Some(raw)
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, channel_id = channel_id.as_value(), %sequence))]
    async fn query_receipt(
        &self,
        _: &Extensions,
        height: Height,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<bool> {
        let execution_height = self.execution_height_of_beacon_slot(height.height()).await;

        let ibc_handler = self.ibc_handler();

        let raw: H256 = ibc_handler
            .commitments(
                ReceiptPath {
                    channel_id,
                    sequence,
                }
                .commitments_key()
                .into(),
            )
            .block(execution_height.into())
            .call()
            .await
            .map_err(|err| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching ibc state: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })?
            ._0
            .into();

        Ok(raw.get()[0] == 1)
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, channel_id = channel_id.as_value()))]
    async fn query_next_sequence_send(
        &self,
        _: &Extensions,
        height: Height,
        channel_id: ChannelId,
    ) -> RpcResult<u64> {
        todo!()
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, channel_id = channel_id.as_value()))]
    async fn query_next_sequence_recv(
        &self,
        _: &Extensions,
        height: Height,
        channel_id: ChannelId,
    ) -> RpcResult<u64> {
        todo!()
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, channel_id = channel_id.as_value()))]
    async fn query_next_sequence_ack(
        &self,
        _: &Extensions,
        height: Height,
        channel_id: ChannelId,
    ) -> RpcResult<u64> {
        todo!()
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height))]
    async fn query_next_connection_sequence(
        &self,
        _: &Extensions,
        height: Height,
    ) -> RpcResult<u64> {
        todo!()
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height))]
    async fn query_next_client_sequence(&self, _: &Extensions, height: Height) -> RpcResult<u64> {
        todo!()
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_ibc_proof(
        &self,
        _: &Extensions,
        at: Height,
        path: Path,
        ibc_store_format: IbcStoreFormat<'static>,
    ) -> RpcResult<Value> {
        let location = ibc_commitment_key(
            match path {
                Path::ClientState(path) => path.commitments_key(),
                Path::ClientConsensusState(path) => path.commitments_key(),
                Path::Connection(path) => path.commitments_key(),
                Path::ChannelEnd(path) => path.commitments_key(),
                Path::Commitment(path) => path.commitments_key(),
                Path::Acknowledgement(path) => path.commitments_key(),
                Path::Receipt(path) => path.commitments_key(),
                Path::NextSequenceSend(path) => todo!(),
                Path::NextSequenceRecv(path) => todo!(),
                Path::NextSequenceAck(path) => todo!(),
                Path::NextConnectionSequence(path) => todo!(),
                Path::NextClientSequence(path) => todo!(),
            },
            IBC_HANDLER_COMMITMENTS_SLOT,
        );

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
            key: U256::from_be_bytes(proof.key.0 .0),
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
        // let latest_execution_height = self.provider.get_block_number().await.unwrap().as_u64();

        // let ClientInfo {
        //     client_type,
        //     ibc_interface,
        //     metadata: _,
        // } = self.client_info(e, client_id.clone()).await?;

        // Ok(RawClientState {
        //     client_type,
        //     ibc_interface,
        //     bytes: self
        //         .ibc_handler()
        //         .ibc_state_read(latest_execution_height, ClientStatePath { client_id })
        //         .await
        //         .unwrap()
        //         .0
        //         .into(),
        // })

        todo!()
    }
}

// type Pls = <(<Module as ModuleContext>::Info, Module) as voyager_message::module::IntoRpc<
//     ModuleData,
//     ModuleCall,
//     ModuleCallback,
//     // RpcModule = ModuleServer<ModuleContext>,
// >>::RpcModule;

// static_assertions::assert_type_eq_all!(Pls, Module);

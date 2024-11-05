// #![warn(clippy::unwrap_used)] // oh boy this will be a lot of work

use std::num::NonZeroU64;

use alloy::{
    providers::{Provider, ProviderBuilder, RootProvider},
    transports::BoxTransport,
};
use beacon_api::client::BeaconApiClient;
use ethereum_light_client_types::StorageProof;
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
use tracing::{debug, info, instrument};
use unionlabs::{
    bytes::Bytes,
    ethereum::ibc_commitment_key,
    hash::{H160, H256},
    ibc::core::{
        channel::{self, channel::Channel, order::Order},
        client::height::Height,
        commitment::merkle_prefix::MerklePrefix,
        connection::{self, connection_end::ConnectionEnd, version::Version},
    },
    id::{ChannelId, ClientId, ConnectionId, PortId},
    uint::U256,
    ErrorReporter,
};
use voyager_message::{
    core::{ChainId, ClientInfo, ClientType, IbcInterface, IbcVersion, QueryHeight},
    module::{ChainModuleInfo, ChainModuleServer, RawClientState},
    rpc::{json_rpc_error_to_error_object, VoyagerRpcClient},
    run_chain_module_server, ChainModule, ExtensionsExt, VoyagerClient, FATAL_JSONRPC_ERROR_CODE,
};
use voyager_vm::BoxDynError;

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
}

#[async_trait]
impl ChainModuleServer for Module {
    #[instrument(skip_all, fields(raw_client_id))]
    async fn query_client_prefix(&self, _: &Extensions, raw_client_id: u32) -> RpcResult<String> {
        Ok(self
            .ibc_handler()
            .clientTypes(raw_client_id)
            .call()
            .await
            .unwrap()
            ._0)
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
            ibc_version: IbcVersion::UnionIbc,
            metadata: Default::default(),
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %client_id))]
    async fn query_client_state(
        &self,
        _: &Extensions,
        height: Height,
        client_id: ClientId,
    ) -> RpcResult<Bytes> {
        let execution_height = self.execution_height_of_beacon_slot(height.height()).await;

        let client_address = self
            .client_address(client_id.id(), execution_height)
            .await?;

        let light_client = ILightClient::new(client_address, self.provider.clone());
        let client_state = light_client
            .getClientState(client_id.id())
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
    async fn query_client_consensus_state(
        &self,
        _: &Extensions,
        height: Height,
        client_id: ClientId,
        trusted_height: Height,
    ) -> RpcResult<Bytes> {
        let execution_height = self.execution_height_of_beacon_slot(height.height()).await;

        let client_address = self
            .client_address(client_id.id(), execution_height)
            .await?;

        let light_client = ILightClient::new(client_address, self.provider.clone());

        let consensus_state = light_client
            .getConsensusState(client_id.id(), trusted_height.height())
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
        e: &Extensions,
        height: Height,
        connection_id: ConnectionId,
    ) -> RpcResult<Option<ConnectionEnd>> {
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

        let client_type = self
            .ibc_handler()
            .clientTypes(raw.clientId)
            .call()
            .await
            .unwrap()
            ._0;

        let client_id = ClientId::new(client_type, raw.clientId);

        let client_meta = e
            .try_get::<VoyagerClient>()?
            .client_meta(
                self.chain_id.clone(),
                QueryHeight::Latest,
                client_id.clone(),
            )
            .await
            .map_err(json_rpc_error_to_error_object)?;

        let counterparty_client_prefix = e
            .try_get::<VoyagerClient>()?
            .query_client_prefix(client_meta.chain_id, raw.counterpartyClientId)
            .await
            .map_err(json_rpc_error_to_error_object)?;

        Ok(Some(ConnectionEnd {
            client_id,
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
                client_id: ClientId::new(counterparty_client_prefix, raw.counterpartyClientId),
                connection_id: Some(ConnectionId::new(raw.counterpartyConnectionId)),
                prefix: MerklePrefix {
                    // TODO: Figure out what to put here
                    key_prefix: b"TODO".to_vec(),
                },
            },
            delay_period: 0,
        }))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %channel_id))]
    async fn query_channel(
        &self,
        _: &Extensions,
        height: Height,
        _port_id: PortId,
        channel_id: ChannelId,
    ) -> RpcResult<Option<Channel>> {
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
                    format!("error fetching channel: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })?
            ._0;

        // let port_id = ibc_handler
        //     .channelOwner(channel_id.id())
        //     .block(execution_height.into())
        //     .call()
        //     .await
        //     .map_err(|err| {
        //         ErrorObject::owned(
        //             -1,
        //             format!("error fetching ibc state: {}", ErrorReporter(err)),
        //             None::<()>,
        //         )
        //     })?
        //     ._0;

        Ok(Some(Channel {
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
            counterparty: channel::counterparty::Counterparty {
                port_id: PortId::new(raw.counterpartyPortId).unwrap(),
                channel_id: Some(ChannelId::new(raw.counterpartyChannelId)),
            },
            connection_hops: vec![ConnectionId::new(raw.connectionId)],
            version: raw.version,
            upgrade_sequence: 0,
        }))
    }

    // #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %channel_id, %sequence))]
    async fn query_commitment(
        &self,
        _: &Extensions,
        _height: Height,
        _port_id: PortId,
        _channel_id: ChannelId,
        _sequence: NonZeroU64,
    ) -> RpcResult<Option<H256>> {
        // let execution_height = self.execution_height_of_beacon_slot(height.height()).await;

        // let ibc_handler = self.ibc_handler();

        // let raw: H256 = ibc_handler
        //     .commitments(ethabi::commitments_key(channel_id.id(), sequence.get()).into())
        //     .block(execution_height.into())
        //     .call()
        //     .await
        //     .map_err(|err| {
        //         ErrorObject::owned(
        //             -1,
        //             format!("error fetching ibc state: {}", ErrorReporter(err)),
        //             None::<()>,
        //         )
        //     })?
        //     ._0
        //     .into();

        // Ok(if raw == <H256>::default() {
        //     None
        // } else {
        //     Some(raw)
        // })
        todo!()
    }

    // #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %channel_id, %sequence))]
    async fn query_acknowledgement(
        &self,
        _: &Extensions,
        _height: Height,
        _port_id: PortId,
        _channel_id: ChannelId,
        _sequence: NonZeroU64,
    ) -> RpcResult<Option<H256>> {
        // let execution_height = self.execution_height_of_beacon_slot(height.height()).await;

        // let ibc_handler = self.ibc_handler();

        // let raw: H256 = ibc_handler
        //     .commitments(ethabi::commitments_key(channel_id.id(), sequence.get()).into())
        //     .block(execution_height.into())
        //     .call()
        //     .await
        //     .map_err(|err| {
        //         ErrorObject::owned(
        //             -1,
        //             format!("error fetching ibc state: {}", ErrorReporter(err)),
        //             None::<()>,
        //         )
        //     })?
        //     ._0
        //     .into();

        // Ok(if raw == <H256>::default() {
        //     None
        // } else {
        //     Some(raw)
        // })
        todo!()
    }

    // #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %channel_id, %sequence))]
    async fn query_receipt(
        &self,
        _: &Extensions,
        _height: Height,
        _port_id: PortId,
        _channel_id: ChannelId,
        _sequence: NonZeroU64,
    ) -> RpcResult<bool> {
        // let execution_height = self.execution_height_of_beacon_slot(height.height()).await;

        // let ibc_handler = self.ibc_handler();

        // let raw: H256 = ibc_handler
        //     .commitments(ethabi::receipts_key(channel_id.id(), sequence.get()).into())
        //     .block(execution_height.into())
        //     .call()
        //     .await
        //     .map_err(|err| {
        //         ErrorObject::owned(
        //             -1,
        //             format!("error fetching ibc state: {}", ErrorReporter(err)),
        //             None::<()>,
        //         )
        //     })?
        //     ._0
        //     .into();

        // Ok(raw.get()[0] == 1)
        todo!()
    }

    // #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %channel_id))]
    async fn query_next_sequence_send(
        &self,
        _: &Extensions,
        _height: Height,
        _port_id: PortId,
        _channel_id: ChannelId,
    ) -> RpcResult<u64> {
        todo!()
    }

    // #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %channel_id))]
    async fn query_next_sequence_recv(
        &self,
        _: &Extensions,
        _height: Height,
        _port_id: PortId,
        _channel_id: ChannelId,
    ) -> RpcResult<u64> {
        todo!()
    }

    // #[instrument(skip_all, fields(chain_id = %self.chain_id, %height, %channel_id))]
    async fn query_next_sequence_ack(
        &self,
        _: &Extensions,
        _height: Height,
        _port_id: PortId,
        _channel_id: ChannelId,
    ) -> RpcResult<u64> {
        todo!()
    }

    // #[instrument(skip_all, fields(chain_id = %self.chain_id, %height))]
    async fn query_next_connection_sequence(
        &self,
        _: &Extensions,
        _height: Height,
    ) -> RpcResult<u64> {
        todo!()
    }

    // #[instrument(skip_all, fields(chain_id = %self.chain_id, %height))]
    async fn query_next_client_sequence(&self, _: &Extensions, _height: Height) -> RpcResult<u64> {
        todo!()
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_ibc_proof(
        &self,
        _: &Extensions,
        at: Height,
        path: Bytes,
        ibc_version: IbcVersion,
    ) -> RpcResult<Value> {
        // TODO: Don't panic
        assert!(matches!(ibc_version, IbcVersion::UnionIbc));

        // let location = ibc_commitment_key(match path {
        //     Path::ClientState(path) => ethabi::client_state_key(path.client_id.id()),
        //     Path::ClientConsensusState(path) => {
        //         ethabi::consensus_state_key(path.client_id.id(), path.height.height())
        //     }
        //     Path::Connection(path) => ethabi::connection_key(path.connection_id.id()),
        //     Path::ChannelEnd(path) => ethabi::channel_key(path.channel_id.id()),
        //     Path::Commitment(_path) => {
        //         todo!()
        //         // ethabi::commitments_key(path.channel_id.id(), path.sequence.get())
        //     }
        //     Path::Acknowledgement(_path) => {
        //         todo!()
        //         // ethabi::acknowledgements_key(path.channel_id.id(), path.sequence.get())
        //     }
        //     Path::Receipt(_path) => {
        //         todo!()
        //         // ethabi::receipts_key(path.channel_id.id(), path.sequence.get())
        //     }
        //     Path::NextSequenceSend(_path) => todo!(),
        //     Path::NextSequenceRecv(_path) => todo!(),
        //     Path::NextSequenceAck(_path) => todo!(),
        //     Path::NextConnectionSequence(_path) => todo!(),
        //     Path::NextClientSequence(_path) => todo!(),
        // });

        let location = ibc_commitment_key(path.try_into().unwrap());

        let execution_height = self.execution_height_of_beacon_slot(at.height()).await;

        let proof = self
            .provider
            .get_proof(
                self.ibc_handler_address.get().into(),
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
            value: U256::from_be_bytes(proof.value.to_be_bytes()),
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
        _e: &Extensions,
        _client_id: ClientId,
    ) -> RpcResult<RawClientState> {
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

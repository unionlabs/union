use std::fmt::Debug;

use aptos_move_ibc::{
    channel::Channel as AptosChannel, connection_end::ConnectionEnd, ibc::ClientExt as _,
};
use aptos_rest_client::{aptos_api_types::Address, error::RestError};
use aptos_types::state_store::state_value::PersistedStateValueMetadata;
use ibc_union_spec::{
    path::StorePath, query::Query, Channel, ChannelState, ClientId, Connection, ConnectionState,
    IbcUnion,
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::{ErrorObject, ErrorObjectOwned},
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, instrument, trace};
use unionlabs::{
    aptos::{
        sparse_merkle_proof::{SparseMerkleLeafNode, SparseMerkleProof},
        storage_proof::{StateValue, StateValueMetadata, StorageProof},
    },
    ibc::core::client::height::Height,
    primitives::{Bytes, H256},
    ErrorReporter,
};
use voyager_message::{
    into_value,
    module::{StateModuleInfo, StateModuleServer},
    primitives::{ChainId, ClientInfo, ClientType, IbcInterface, Timestamp},
    StateModule,
};
use voyager_vm::BoxDynError;

pub mod events;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(clap::Subcommand)]
pub enum Cmd {
    ChainId,
    VaultAddress,
    SubmitTx,
    FetchAbi,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub aptos_client: aptos_rest_client::Client,
    pub movement_rpc_url: String,

    pub ibc_handler_address: Address,
}

impl StateModule<IbcUnion> for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: StateModuleInfo) -> Result<Self, BoxDynError> {
        let aptos_client = aptos_rest_client::Client::new(config.rpc_url.parse()?);

        let chain_id = aptos_client.get_index().await?.inner().chain_id;

        info.ensure_chain_id(chain_id.to_string())?;

        Ok(Self {
            chain_id: ChainId::new(chain_id.to_string()),
            aptos_client,
            movement_rpc_url: config.movement_rpc_url,
            ibc_handler_address: config.ibc_handler_address,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub rpc_url: String,
    pub movement_rpc_url: String,
    pub ibc_handler_address: Address,
}

impl aptos_move_ibc::ibc::ClientExt for Module {
    fn client(&self) -> &aptos_rest_client::Client {
        &self.aptos_client
    }
}

impl Module {
    #[must_use]
    pub fn make_height(&self, height: u64) -> Height {
        Height::new(height)
    }

    pub async fn ledger_version_of_height(&self, height: u64) -> u64 {
        let ledger_version = self
            .aptos_client
            .get_block_by_height(height, false)
            .await
            // .map_err(rest_error_to_rpc_error)?
            .unwrap()
            .into_inner()
            .last_version
            .0;

        debug!("height {height} is ledger version {ledger_version}");

        ledger_version
    }

    /// Query the latest finalized height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    pub async fn query_latest_height(&self, _: &Extensions) -> RpcResult<Height> {
        match self.aptos_client.get_index().await {
            Ok(ledger_info) => {
                let height = ledger_info.inner().block_height.0;

                trace!(height, "latest height");

                Ok(self.make_height(height))
            }
            Err(err) => Err(ErrorObject::owned(
                -1,
                ErrorReporter(err).to_string(),
                None::<()>,
            )),
        }
    }

    /// Query the latest finalized timestamp of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    pub async fn query_latest_timestamp(&self, e: &Extensions) -> RpcResult<Timestamp> {
        let latest_height = self.query_latest_height(e).await?;

        match self
            .aptos_client
            .get_block_by_height(latest_height.height(), false)
            .await
        {
            Ok(block) => {
                let timestamp = block.inner().block_timestamp.0;

                debug!(%timestamp, %latest_height, "latest timestamp");

                Ok(Timestamp::from_nanos(timestamp))
            }
            Err(err) => Err(ErrorObject::owned(
                -1,
                ErrorReporter(err).to_string(),
                None::<()>,
            )),
        }
    }
}

#[async_trait]
impl StateModuleServer<IbcUnion> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query(&self, _: &Extensions, query: Query) -> RpcResult<Value> {
        match query {
            Query::PacketByHash(_packet_by_hash) => todo!(),
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn client_info(&self, _: &Extensions, client_id: ClientId) -> RpcResult<ClientInfo> {
        let client_type = self
            .client_id_to_type(self.ibc_handler_address.into(), None, (client_id.raw(),))
            .await
            .map_err(rest_error_to_rpc_error)?;
        Ok(ClientInfo {
            client_type: ClientType::new(client_type),
            ibc_interface: IbcInterface::new(IbcInterface::IBC_MOVE_APTOS),
            metadata: Default::default(),
        })
    }

    async fn query_ibc_state(
        &self,
        _: &Extensions,
        at: Height,
        path: StorePath,
    ) -> RpcResult<Value> {
        let ledger_version = self.ledger_version_of_height(at.height()).await;

        Ok(match path {
            StorePath::ClientState(path) => {
                let client_state_bytes: Bytes = self
                    .client_state(
                        self.ibc_handler_address.into(),
                        Some(ledger_version),
                        (path.client_id.raw(),),
                    )
                    .await
                    .map_err(rest_error_to_rpc_error)?
                    .into();

                into_value(client_state_bytes)
            }
            StorePath::ConsensusState(path) => {
                let consensus_state_bytes: Bytes = self
                    .consensus_state(
                        self.ibc_handler_address.into(),
                        Some(ledger_version),
                        (path.client_id.raw(), path.height),
                    )
                    .await
                    .map_err(rest_error_to_rpc_error)?
                    .into();

                into_value(consensus_state_bytes)
            }
            StorePath::Connection(path) => {
                let connection = self
                    .get_connection(
                        self.ibc_handler_address.into(),
                        Some(ledger_version),
                        (path.connection_id.raw(),),
                    )
                    .await
                    .map_err(rest_error_to_rpc_error)?
                    .unwrap();
                into_value(convert_connection(connection))
            }
            StorePath::Channel(path) => {
                let channel = self
                    .get_channel(
                        self.ibc_handler_address.into(),
                        Some(ledger_version),
                        (path.channel_id.raw(),),
                    )
                    .await
                    .map_err(rest_error_to_rpc_error)?
                    .unwrap();
                into_value(convert_channel(channel))
            }
            // TODO(aeryz): check if we have to do `TryInto<H256>` here
            StorePath::BatchPackets(path) => into_value(
                self.get_commitment(
                    self.ibc_handler_address.into(),
                    Some(ledger_version),
                    (path.key().into_bytes().into(),),
                )
                .await
                .map_err(rest_error_to_rpc_error)?,
            ),
            StorePath::BatchReceipts(path) => {
                let commitment = self
                    .get_commitment(
                        self.ibc_handler_address.into(),
                        Some(ledger_version),
                        (path.key().into_bytes().into(),),
                    )
                    .await
                    .map_err(rest_error_to_rpc_error)?;

                into_value(match &commitment[..] {
                    [] => false,
                    [1] => true,
                    _ => panic!("not a bool??? {commitment:?}"),
                })
            }
        })
    }
}

pub fn rest_error_to_rpc_error(e: RestError) -> ErrorObjectOwned {
    ErrorObject::owned(-1, format!("rest error: {}", ErrorReporter(e)), None::<()>)
}

pub async fn get_storage_proof(
    movement_rpc_url: &str,
    address: H256,
    key: String,
    height: u64,
) -> StorageProof {
    let client = reqwest::Client::new();

    let req =
        format!("{movement_rpc_url}/movement/v1/table-item-with-proof/{address}/{key}/{height}",);

    let (state_value, proof): (
        Option<aptos_types::state_store::state_value::StateValue>,
        aptos_types::proof::SparseMerkleProof,
    ) = client.get(req).send().await.unwrap().json().await.unwrap();

    StorageProof {
        state_value: state_value.map(|s| {
            let (metadata, data) = s.unpack();
            match metadata.into_persistable() {
                None => StateValue::V0(data.to_vec()),
                Some(PersistedStateValueMetadata::V0 {
                    deposit,
                    creation_time_usecs,
                }) => StateValue::WithMetadata {
                    data: data.to_vec(),
                    metadata: StateValueMetadata::V0 {
                        deposit,
                        creation_time_usecs,
                    },
                },
                Some(PersistedStateValueMetadata::V1 {
                    slot_deposit,
                    bytes_deposit,
                    creation_time_usecs,
                }) => StateValue::WithMetadata {
                    data: data.to_vec(),
                    metadata: StateValueMetadata::V1 {
                        slot_deposit,
                        bytes_deposit,
                        creation_time_usecs,
                    },
                },
            }
        }),
        proof: SparseMerkleProof {
            leaf: proof.leaf().map(|leaf| SparseMerkleLeafNode {
                key: (*leaf.key().as_ref()).into(),
                value_hash: (*leaf.value_hash().as_ref()).into(),
            }),
            siblings: proof
                .siblings()
                .iter()
                .map(AsRef::as_ref)
                .copied()
                .map(Into::into)
                .collect(),
        },
    }
}

fn convert_connection(connection: ConnectionEnd) -> Connection {
    Connection {
        state: match connection.state {
            1 => ConnectionState::Init,
            2 => ConnectionState::TryOpen,
            3 => ConnectionState::Open,
            _ => panic!("connection state must be 1..=3"),
        },
        client_id: connection.client_id.try_into().unwrap(),
        counterparty_client_id: connection.counterparty_client_id.try_into().unwrap(),
        counterparty_connection_id: connection.counterparty_connection_id.try_into().ok(),
    }
}

fn convert_channel(channel: AptosChannel) -> Channel {
    Channel {
        state: match channel.state {
            1 => ChannelState::Init,
            2 => ChannelState::TryOpen,
            3 => ChannelState::Open,
            4 => ChannelState::Closed,
            _ => panic!("channel state must be 1..=4"),
        },
        connection_id: channel.connection_id.try_into().unwrap(),
        counterparty_channel_id: channel.counterparty_channel_id.try_into().ok(),
        counterparty_port_id: channel.counterparty_port_id.into(),
        version: channel.version,
    }
}

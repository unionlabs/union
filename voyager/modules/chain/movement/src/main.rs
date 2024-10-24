use std::fmt::Debug;

use aptos_move_ibc::ibc::ClientExt as _;
use aptos_rest_client::{aptos_api_types::Address, error::RestError};
use aptos_types::state_store::state_value::PersistedStateValueMetadata;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::{ErrorObject, ErrorObjectOwned},
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use serde_utils::Hex;
use tracing::{debug, instrument};
use unionlabs::{
    aptos::{
        sparse_merkle_proof::{SparseMerkleLeafNode, SparseMerkleProof},
        storage_proof::{StateValue, StateValueMetadata, StorageProof},
    },
    hash::H256,
    ibc::core::{
        channel::{self, channel::Channel, order::Order},
        client::height::Height,
        commitment::merkle_prefix::MerklePrefix,
        connection::{self, connection_end::ConnectionEnd},
    },
    ics24::{ClientStatePath, Path},
    id::ClientId,
    uint::U256,
    ErrorReporter,
};
use voyager_message::{
    core::{ChainId, ClientInfo, ClientType, IbcInterface},
    into_value,
    module::{ChainModuleInfo, ChainModuleServer, RawClientState},
    run_chain_module_server, ChainModule,
};
use voyager_vm::BoxDynError;

pub mod events;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_chain_module_server::<Module>().await
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
    pub chain_id: ChainId<'static>,

    pub aptos_client: aptos_rest_client::Client,
    pub movement_rpc_url: String,

    pub ibc_handler_address: Address,
}

impl ChainModule for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: ChainModuleInfo) -> Result<Self, BoxDynError> {
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
        Height {
            // TODO: Make this a constant
            revision_number: 0,
            revision_height: height,
        }
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
}

#[async_trait]
impl ChainModuleServer for Module {
    /// Query the latest finalized height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_latest_height(&self, _: &Extensions) -> RpcResult<Height> {
        match self.aptos_client.get_index().await {
            Ok(ledger_info) => {
                let height = ledger_info.inner().block_height.0;

                debug!(height, "latest height");

                Ok(self.make_height(height))
            }
            Err(err) => Err(ErrorObject::owned(
                -1,
                ErrorReporter(err).to_string(),
                None::<()>,
            )),
        }
    }

    /// Query the latest (non-finalized) height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_latest_height_as_destination(&self, e: &Extensions) -> RpcResult<Height> {
        self.query_latest_height(e).await
    }

    /// Query the latest finalized timestamp of this chain.
    // TODO: Use a better timestamp type here
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_latest_timestamp(&self, e: &Extensions) -> RpcResult<i64> {
        let latest_height = self.query_latest_height(e).await?;

        match self
            .aptos_client
            .get_block_by_height(latest_height.revision_height, false)
            .await
        {
            Ok(block) => {
                let timestamp = block.inner().block_timestamp.0;

                debug!(%timestamp, %latest_height, "latest timestamp");

                Ok(timestamp.try_into().unwrap())
            }
            Err(err) => Err(ErrorObject::owned(
                -1,
                ErrorReporter(err).to_string(),
                None::<()>,
            )),
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn client_info(&self, _: &Extensions, client_id: ClientId) -> RpcResult<ClientInfo> {
        match client_id.to_string().rsplit_once('-') {
            Some(("cometbls", _)) => Ok(ClientInfo {
                client_type: ClientType::new(ClientType::COMETBLS_GROTH16),
                ibc_interface: IbcInterface::new(IbcInterface::IBC_MOVE_APTOS),
                metadata: Default::default(),
            }),
            _ => Err(ErrorObject::owned(
                -1,
                format!("unknown client type (client id `{client_id}`)"),
                Some(json!({
                    "client_id": client_id.to_string()
                })),
            )),
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %at, %path))]
    async fn query_ibc_state(&self, _: &Extensions, at: Height, path: Path) -> RpcResult<Value> {
        let ledger_version = self.ledger_version_of_height(at.revision_height).await;

        Ok(match path {
            Path::ClientState(path) => {
                let client_state_bytes = self
                    .client_state(
                        self.ibc_handler_address.into(),
                        (path.client_id.to_string(),),
                        Some(ledger_version),
                    )
                    .await
                    .map_err(rest_error_to_rpc_error)?;

                into_value(client_state_bytes)
            }
            Path::ClientConsensusState(path) => {
                let consensus_state_bytes = self
                    .consensus_state(
                        self.ibc_handler_address.into(),
                        (
                            path.client_id.to_string(),
                            path.height.revision_number,
                            path.height.revision_height,
                        ),
                        Some(ledger_version),
                    )
                    .await
                    .map_err(rest_error_to_rpc_error)?;

                into_value(consensus_state_bytes)
            }
            Path::Connection(path) => into_value(
                self.get_connection(
                    self.ibc_handler_address.into(),
                    (path.connection_id.to_string(),),
                    Some(ledger_version),
                )
                .await
                .map_err(rest_error_to_rpc_error)?
                .into_option()
                .map(convert_connection),
            ),
            Path::ChannelEnd(path) => into_value(
                self.get_channel(
                    self.ibc_handler_address.into(),
                    (path.port_id.to_string(), path.channel_id.to_string()),
                    Some(ledger_version),
                )
                .await
                .map_err(rest_error_to_rpc_error)?
                .into_option()
                .map(convert_channel),
            ),
            Path::Commitment(path) => {
                let commitment = self
                    .get_commitment(
                        self.ibc_handler_address.into(),
                        (path.to_string().into_bytes().into(),),
                        Some(ledger_version),
                    )
                    .await
                    .map_err(rest_error_to_rpc_error)?;

                into_value(<H256>::try_from(commitment.0).unwrap())
            }
            Path::Acknowledgement(path) => {
                let commitment = self
                    .get_commitment(
                        self.ibc_handler_address.into(),
                        (path.to_string().into_bytes().into(),),
                        Some(ledger_version),
                    )
                    .await
                    .map_err(rest_error_to_rpc_error)?;

                into_value(<H256>::try_from(commitment.0).unwrap())
            }
            Path::Receipt(path) => {
                let commitment = self
                    .get_commitment(
                        self.ibc_handler_address.into(),
                        (path.to_string().into_bytes().into(),),
                        Some(ledger_version),
                    )
                    .await
                    .map_err(rest_error_to_rpc_error)?;

                into_value(match &commitment.0[..] {
                    [] => false,
                    [1] => true,
                    _ => panic!("not a bool??? {commitment}"),
                })
            }
            Path::NextSequenceSend(_) => todo!(),
            Path::NextSequenceRecv(_) => todo!(),
            Path::NextSequenceAck(_) => todo!(),
            Path::NextConnectionSequence(_) => todo!(),
            Path::NextClientSequence(_) => todo!(),
        })

        // self.get_connection(, , )

        // const IBC_STORE_PATH: &str = "store/ibc/key";

        // let path_string = path.to_string();

        // let query_result = self
        //     .tm_client
        //     .abci_query(
        //         IBC_STORE_PATH,
        //         &path_string,
        //         Some(
        //             i64::try_from(at.revision_height)
        //                 .unwrap()
        //                 .try_into()
        //                 .expect("invalid height"),
        //         ),
        //         false,
        //     )
        //     .await
        //     .unwrap()
        //     .response;

        // Ok(match path {
        //     Path::ClientState(_) => serde_json::to_value(Hex(query_result.value)).unwrap(),
        //     Path::ClientConsensusState(_) => serde_json::to_value(Hex(query_result.value)).unwrap(),
        //     Path::Connection(_) => serde_json::to_value(
        //         ConnectionEnd::decode_as::<Proto>(&query_result.value).unwrap(),
        //     )
        //     .unwrap(),
        //     Path::ChannelEnd(_) => {
        //         serde_json::to_value(Channel::decode_as::<Proto>(&query_result.value).unwrap())
        //             .unwrap()
        //     }
        //     Path::Commitment(_) => {
        //         serde_json::to_value(H256::try_from(query_result.value).unwrap()).unwrap()
        //     }
        //     Path::Acknowledgement(_) => {
        //         serde_json::to_value(H256::try_from(query_result.value).unwrap()).unwrap()
        //     }
        //     Path::Receipt(_) => serde_json::to_value(match query_result.value[..] {
        //         [] => false,
        //         [1] => true,
        //         ref invalid => panic!("not a bool??? {invalid:?}"),
        //     })
        //     .unwrap(),
        //     Path::NextSequenceSend(_) => {
        //         serde_json::to_value(u64::from_be_bytes(query_result.value.try_into().unwrap()))
        //             .unwrap()
        //     }
        //     Path::NextSequenceRecv(_) => {
        //         serde_json::to_value(u64::from_be_bytes(query_result.value.try_into().unwrap()))
        //             .unwrap()
        //     }
        //     Path::NextSequenceAck(_) => {
        //         serde_json::to_value(u64::from_be_bytes(query_result.value.try_into().unwrap()))
        //             .unwrap()
        //     }
        //     Path::NextConnectionSequence(_) => {
        //         serde_json::to_value(u64::from_be_bytes(query_result.value.try_into().unwrap()))
        //             .unwrap()
        //     }
        //     Path::NextClientSequence(_) => {
        //         serde_json::to_value(u64::from_be_bytes(query_result.value.try_into().unwrap()))
        //             .unwrap()
        //     }
        // })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_ibc_proof(&self, _: &Extensions, at: Height, _path: Path) -> RpcResult<Value> {
        let ledger_version = self.ledger_version_of_height(at.revision_height).await;

        let vault_addr = self
            .get_vault_addr(self.ibc_handler_address.into(), Some(ledger_version))
            .await
            .unwrap();

        let _address_str = self
            .aptos_client
            .get_account_resource(
                vault_addr.into(),
                &format!("{}::ibc::IBCStore", self.ibc_handler_address),
            )
            .await
            .unwrap()
            .into_inner()
            .unwrap()
            .data["commitments"]["handle"]
            .clone()
            .as_str()
            .unwrap()
            .to_owned();
        let _address = <H256>::new(U256::from_be_hex(_address_str).unwrap().to_be_bytes());

        // NOTE(aeryz): This only works with Union's custom Movement node. When the following PR is merged,
        // we will uncomment this: https://github.com/movementlabsxyz/movement/pull/645
        // let storage_proof = get_storage_proof(
        //     &self.ctx.movement_rpc_url,
        //     address,
        //     hex::encode(bcs::to_bytes(&path.to_string().as_bytes()).expect("won't fail")),
        //     at.revision_height,
        // ).await;

        Ok(into_value(StorageProof {
            state_value: None,
            proof: SparseMerkleProof {
                leaf: None,
                siblings: Vec::new(),
            },
        }))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_raw_unfinalized_trusted_client_state(
        &self,
        e: &Extensions,
        client_id: ClientId,
    ) -> RpcResult<RawClientState<'static>> {
        let height = self.query_latest_height(e).await?;

        let client_state = serde_json::from_value::<Hex<Vec<u8>>>(
            self.query_ibc_state(
                e,
                height,
                ClientStatePath {
                    client_id: client_id.clone(),
                }
                .into(),
            )
            .await?,
        )
        .unwrap();

        let ClientInfo {
            client_type,
            ibc_interface,
            metadata: _,
        } = self.client_info(e, client_id.clone()).await?;

        Ok(RawClientState {
            client_type,
            ibc_interface,
            bytes: client_state.0.into(),
        })
    }
}

pub fn rest_error_to_rpc_error(e: RestError) -> ErrorObjectOwned {
    ErrorObject::owned(-1, format!("rest error: {}", ErrorReporter(e)), None::<()>)
}

pub fn convert_connection(
    connection: aptos_move_ibc::connection_end::ConnectionEnd,
) -> ConnectionEnd {
    ConnectionEnd {
        client_id: connection.client_id.parse().unwrap(),
        versions: connection
            .versions
            .into_iter()
            .map(|version| connection::version::Version {
                identifier: version.identifier,
                features: version
                    .features
                    .into_iter()
                    .map(|feature| Order::from_proto_str(&feature).expect("unknown feature"))
                    .collect(),
            })
            .collect(),
        state: connection::state::State::try_from(u8::try_from(connection.state.0).unwrap())
            .unwrap(),
        counterparty: connection::counterparty::Counterparty {
            client_id: connection.counterparty.client_id.parse().unwrap(),
            connection_id: if connection.counterparty.connection_id.is_empty() {
                None
            } else {
                Some(connection.counterparty.connection_id.parse().unwrap())
            },
            prefix: MerklePrefix {
                key_prefix: connection.counterparty.prefix.key_prefix.into(),
            },
        },
        delay_period: connection.delay_period.0,
    }
}

pub fn convert_channel(channel: aptos_move_ibc::channel::Channel) -> Channel {
    Channel {
        state: channel.state.try_into().unwrap(),
        ordering: channel.ordering.try_into().unwrap(),
        counterparty: channel::counterparty::Counterparty {
            port_id: channel.counterparty.port_id.parse().unwrap(),
            channel_id: channel.counterparty.channel_id.parse().unwrap(),
        },
        connection_hops: channel
            .connection_hops
            .into_iter()
            .map(|hop| hop.parse().unwrap())
            .collect(),
        version: channel.version,
    }
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

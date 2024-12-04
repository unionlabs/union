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
use tracing::{debug, instrument};
use unionlabs::{
    aptos::{
        sparse_merkle_proof::{SparseMerkleLeafNode, SparseMerkleProof},
        storage_proof::{StateValue, StateValueMetadata, StorageProof},
    },
    hash::H256,
    ibc::core::client::height::Height,
    ics24::ethabi::Path,
    ErrorReporter,
};
use voyager_message::{
    core::{ChainId, ClientInfo, ClientType, IbcInterface},
    ibc_union::IbcUnion,
    into_value,
    module::{StateModuleInfo, StateModuleServer},
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

    /// Query the latest finalized timestamp of this chain.
    // TODO: Use a better timestamp type here
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    pub async fn query_latest_timestamp(&self, e: &Extensions) -> RpcResult<i64> {
        let latest_height = self.query_latest_height(e).await?;

        match self
            .aptos_client
            .get_block_by_height(latest_height.height(), false)
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
}

#[async_trait]
impl StateModuleServer<IbcUnion> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn client_info(&self, _: &Extensions, client_id: u32) -> RpcResult<ClientInfo> {
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

    async fn query_ibc_state(&self, _: &Extensions, at: Height, path: Path) -> RpcResult<Value> {
        let ledger_version = self.ledger_version_of_height(at.height()).await;

        Ok(match path {
            Path::ClientState(path) => {
                let client_state_bytes = self
                    .client_state(
                        self.ibc_handler_address.into(),
                        Some(ledger_version),
                        (path.client_id,),
                    )
                    .await
                    .map_err(rest_error_to_rpc_error)?;

                into_value(client_state_bytes)
            }
            Path::ConsensusState(path) => {
                let consensus_state_bytes = self
                    .consensus_state(
                        self.ibc_handler_address.into(),
                        Some(ledger_version),
                        (path.client_id, path.height),
                    )
                    .await
                    .map_err(rest_error_to_rpc_error)?;

                into_value(consensus_state_bytes)
            }
            Path::Connection(path) => into_value(
                self.get_connection(
                    self.ibc_handler_address.into(),
                    Some(ledger_version),
                    (path.connection_id,),
                )
                .await
                .map_err(rest_error_to_rpc_error)?,
            ),
            Path::Channel(path) => into_value(
                self.get_channel(
                    self.ibc_handler_address.into(),
                    Some(ledger_version),
                    (path.channel_id,),
                )
                .await
                .map_err(rest_error_to_rpc_error)?,
            ),
            // TODO(aeryz): check if we have to do `TryInto<H256>` here
            Path::BatchPackets(path) => into_value(
                self.get_commitment(
                    self.ibc_handler_address.into(),
                    Some(ledger_version),
                    (path.key().into_bytes().into(),),
                )
                .await
                .map_err(rest_error_to_rpc_error)?,
            ),
            Path::BatchReceipts(path) => {
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

    // #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    // async fn query_ibc_proof(&self, _: &Extensions, at: Height, _path: Path) -> RpcResult<Value> {
    //     let ledger_version = self.ledger_version_of_height(at.revision_height).await;

    //     let vault_addr = self
    //         .get_vault_addr(self.ibc_handler_address.into(), Some(ledger_version))
    //         .await
    //         .unwrap();

    //     let _address_str = self
    //         .aptos_client
    //         .get_account_resource(
    //             vault_addr.into(),
    //             &format!("{}::ibc::IBCStore", self.ibc_handler_address),
    //         )
    //         .await
    //         .unwrap()
    //         .into_inner()
    //         .unwrap()
    //         .data["commitments"]["handle"]
    //         .clone()
    //         .as_str()
    //         .unwrap()
    //         .to_owned();
    //     let _address = <H256>::new(U256::from_be_hex(_address_str).unwrap().to_be_bytes());

    //     // NOTE(aeryz): This only works with Union's custom Movement node. When the following PR is merged,
    //     // we will uncomment this: https://github.com/movementlabsxyz/movement/pull/645
    //     // let storage_proof = get_storage_proof(
    //     //     &self.ctx.movement_rpc_url,
    //     //     address,
    //     //     hex::encode(bcs::to_bytes(&path.to_string().as_bytes()).expect("won't fail")),
    //     //     at.revision_height,
    //     // ).await;

    //     Ok(into_value(StorageProof {
    //         state_value: None,
    //         proof: SparseMerkleProof {
    //             leaf: None,
    //             siblings: Vec::new(),
    //         },
    //     }))
    // }

    // #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    // async fn query_raw_unfinalized_trusted_client_state(
    //     &self,
    //     e: &Extensions,
    //     client_id: ClientId,
    // ) -> RpcResult<RawClientState<'static>> {
    //     let height = self.query_latest_height(e).await?;

    //     let client_state = serde_json::from_value::<Bytes>(
    //         self.query_ibc_state(
    //             e,
    //             height,
    //             ClientStatePath {
    //                 client_id: client_id.clone(),
    //             }
    //             .into(),
    //         )
    //         .await?,
    //     )
    //     .unwrap();

    //     let ClientInfo {
    //         client_type,
    //         ibc_interface,
    //         metadata: _,
    //     } = self.client_info(e, client_id.clone()).await?;

    //     Ok(RawClientState {
    //         client_type,
    //         ibc_interface,
    //         bytes: client_state.0.into(),
    //     })
    // }
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

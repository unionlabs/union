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
use serde_json::Value;
use tracing::{debug, instrument};
use unionlabs::{
    aptos::{
        sparse_merkle_proof::{SparseMerkleLeafNode, SparseMerkleProof},
        storage_proof::{StateValue, StateValueMetadata, StorageProof},
    },
    hash::H256,
    ibc::core::client::height::Height,
    ics24::ethabi::Path,
    uint::U256,
    ErrorReporter,
};
use voyager_message::{
    core::ChainId,
    ibc_union::IbcUnion,
    into_value,
    module::{ProofModuleInfo, ProofModuleServer},
    ProofModule,
};
use voyager_vm::BoxDynError;

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

impl ProofModule<IbcUnion> for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: ProofModuleInfo) -> Result<Self, BoxDynError> {
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
}

#[async_trait]
impl ProofModuleServer<IbcUnion> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_ibc_proof(&self, _: &Extensions, at: Height, _path: Path) -> RpcResult<Value> {
        let ledger_version = self.ledger_version_of_height(at.height()).await;

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

use std::fmt::Debug;

use ibc_union_spec::{path::StorePath, IbcUnion};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sui_light_client_types::{
    checkpoint_summary::CheckpointContents,
    digest::Digest,
    object::{Data, MoveObject, MoveObjectType, ObjectInner, StructTag, TypeTag},
    storage_proof::StorageProof,
    transaction_effects::TransactionEffects,
    Authenticator, ObjectID, Owner,
};
use sui_sdk::{
    rpc_types::{SuiObjectDataOptions, SuiTransactionBlockResponseOptions},
    types::{
        base_types::ObjectID as SuiObjectID, effects::TransactionEvents,
        messages_checkpoint::CertifiedCheckpointSummary, object::Object, transaction::Transaction,
    },
    SuiClientBuilder,
};
use tracing::instrument;
use unionlabs::{ibc::core::client::height::Height, ErrorReporter};
use voyager_message::{
    into_value,
    module::{ProofModuleInfo, ProofModuleServer},
    primitives::ChainId,
    rpc::ProofType,
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

#[derive(Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub sui_client: sui_sdk::SuiClient,

    pub sui_object_store_rpc_url: String,

    pub ibc_commitments_object_id: ObjectID,
}

impl ProofModule<IbcUnion> for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: ProofModuleInfo) -> Result<Self, BoxDynError> {
        let sui_client = SuiClientBuilder::default().build(&config.rpc_url).await?;

        let chain_id = sui_client.read_api().get_chain_identifier().await?;

        info.ensure_chain_id(chain_id.to_string())?;

        Ok(Self {
            chain_id: ChainId::new(chain_id.to_string()),
            sui_client,
            ibc_commitments_object_id: config.ibc_commitments_object_id,
            sui_object_store_rpc_url: config.sui_object_store_rpc_url,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub rpc_url: String,

    pub ibc_commitments_object_id: ObjectID,

    pub sui_object_store_rpc_url: String,
}

impl Module {
    #[must_use]
    pub fn make_height(&self, height: u64) -> Height {
        Height::new(height)
    }
}

fn err<T: core::error::Error>(e: T, msg: &str) -> ErrorObject {
    ErrorObject::owned(-1, ErrorReporter(e).with_message(msg), None::<()>)
}

#[async_trait]
impl ProofModuleServer<IbcUnion> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_ibc_proof(
        &self,
        _: &Extensions,
        _: Height,
        path: StorePath,
    ) -> RpcResult<Option<(Value, ProofType)>> {
        let key = path.key();

        let target_object_id = sui_verifier::calculate_dynamic_field_object_id(
            *self.ibc_commitments_object_id.get(),
            key.get().as_slice(),
        );

        let target_object = self
            .sui_client
            .read_api()
            .get_object_with_options(
                SuiObjectID::new(*target_object_id.get()),
                SuiObjectDataOptions {
                    show_type: false,
                    show_owner: true,
                    show_previous_transaction: true,
                    show_display: false,
                    show_content: false,
                    show_bcs: true,
                    show_storage_rebate: true,
                },
            )
            .await
            .map_err(|e| err(e, "error fetching the object"))?
            .data
            .expect("data is fetched");

        let previous_tx = target_object
            .previous_transaction
            .expect("tx info is requested");

        let checkpoint_number = self
            .sui_client
            .read_api()
            .get_transaction_with_options(previous_tx, SuiTransactionBlockResponseOptions::new())
            .await
            .map_err(|e| err(e, "error fetching the tx"))?
            .checkpoint
            .expect("checkpoint is fetched");

        let client = reqwest::Client::new();
        let req = format!("{}/{checkpoint_number}.chk", self.sui_object_store_rpc_url);
        let res = client
            .get(req)
            .send()
            .await
            .map_err(|e| err(e, "error fetching the tx"))?
            .bytes()
            .await
            .map_err(|e| err(e, "error fetching the tx"))?;

        let (_, checkpoint) = bcs::from_bytes::<(u8, CheckpointData)>(&res)
            .map_err(|e| err(e, "invalid checkpoint data"))?;

        let tx = checkpoint
            .transactions
            .iter()
            .find(|tx| *tx.transaction.digest() == previous_tx)
            .unwrap();

        let object = convert_object(target_object.try_into().expect(
            "target object should have all the info needed to be able to make it into an object",
        ));

        Ok(Some((
            into_value(StorageProof {
                checkpoint_contents: checkpoint.checkpoint_contents,
                transaction_effects: tx.effects.clone(),
                object,
            }),
            ProofType::Membership,
        )))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckpointData {
    pub checkpoint_summary: CertifiedCheckpointSummary,
    pub checkpoint_contents: CheckpointContents,
    pub transactions: Vec<CheckpointTransaction>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckpointTransaction {
    /// The input Transaction
    pub transaction: Transaction,
    /// The effects produced by executing this transaction
    pub effects: TransactionEffects,
    /// The events, if any, emitted by this transactions during execution
    pub events: Option<TransactionEvents>,
    /// The state of all inputs to this transaction as they were prior to execution.
    pub input_objects: Vec<Object>,
    /// The state of all output objects created or mutated or unwrapped by this transaction.
    pub output_objects: Vec<Object>,
}

/// convert an object into the local `ObjectInner` type
fn convert_object(object: Object) -> ObjectInner {
    let sui_sdk::types::object::Data::Move(object_data) = &object.data else {
        panic!("package objects are never fetched");
    };

    ObjectInner {
        data: Data::Move(MoveObject {
            type_: if object_data.type_().is_gas_coin() {
                MoveObjectType::GasCoin
            } else if object_data.type_().is_staked_sui() {
                MoveObjectType::StakedSui
            } else if object_data.type_().is_coin() {
                MoveObjectType::Coin(convert_type_tag(
                    object_data.type_().coin_type_maybe().expect("type is coin"),
                ))
            } else {
                if let Some(struct_tag) = object_data.type_().other() {
                    MoveObjectType::Other(convert_struct_tag(struct_tag))
                } else {
                    panic!("no other possible states");
                }
            },
            has_public_transfer: object_data.has_public_transfer(),
            version: object_data.version().into(),
            contents: object_data.contents().into(),
        }),
        owner: match &object.owner {
            sui_sdk::types::object::Owner::AddressOwner(sui_address) => {
                Owner::AddressOwner(sui_address.to_inner().into())
            }
            sui_sdk::types::object::Owner::ObjectOwner(sui_address) => {
                Owner::ObjectOwner(sui_address.to_inner().into())
            }
            sui_sdk::types::object::Owner::Shared {
                initial_shared_version,
            } => Owner::Shared {
                initial_shared_version: (*initial_shared_version).into(),
            },
            sui_sdk::types::object::Owner::Immutable => Owner::Immutable,
            sui_sdk::types::object::Owner::ConsensusV2 {
                start_version,
                authenticator,
            } => Owner::ConsensusV2 {
                start_version: (*start_version).into(),
                authenticator: Box::new(Authenticator::SingleOwner(
                    authenticator.as_single_owner().to_inner().into(),
                )),
            },
        },
        previous_transaction: Digest(object.previous_transaction.into_inner().into()),
        storage_rebate: object.storage_rebate,
    }
}

fn convert_type_tag(tag: sui_sdk::types::TypeTag) -> TypeTag {
    match tag {
        sui_sdk::types::TypeTag::Bool => TypeTag::Bool,
        sui_sdk::types::TypeTag::U8 => TypeTag::U8,
        sui_sdk::types::TypeTag::U64 => TypeTag::U64,
        sui_sdk::types::TypeTag::U128 => TypeTag::U128,
        sui_sdk::types::TypeTag::Address => TypeTag::Address,
        sui_sdk::types::TypeTag::Signer => TypeTag::Signer,
        sui_sdk::types::TypeTag::Vector(type_tag) => {
            TypeTag::Vector(Box::new(convert_type_tag(*type_tag)))
        }
        sui_sdk::types::TypeTag::Struct(struct_tag) => {
            TypeTag::Struct(Box::new(convert_struct_tag(*struct_tag)))
        }
        sui_sdk::types::TypeTag::U16 => TypeTag::U16,
        sui_sdk::types::TypeTag::U32 => TypeTag::U32,
        sui_sdk::types::TypeTag::U256 => TypeTag::U256,
    }
}

fn convert_struct_tag(tag: move_core_types::language_storage::StructTag) -> StructTag {
    StructTag {
        address: tag.address.into_bytes().into(),
        module: tag.module.into_string(),
        name: tag.name.into_string(),
        type_params: tag.type_params.into_iter().map(convert_type_tag).collect(),
    }
}

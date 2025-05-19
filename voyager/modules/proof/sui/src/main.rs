use std::fmt::Debug;

use ibc_union_spec::{path::StorePath, IbcUnion};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sui_light_client_types::{
    checkpoint_summary::CheckpointContents,
    crypto::AuthorityStrongQuorumSignInfo,
    object::{Data, MoveObject, MoveObjectType, ObjectInner, StructTag, TypeTag},
    storage_proof::StorageProof,
    transaction_effects::{TransactionEffects, TransactionEffectsV1, TransactionEffectsV2},
    ObjectID,
};
use sui_sdk::{
    rpc_types::SuiTransactionBlockResponseOptions,
    types::{
        base_types::ObjectID as SuiObjectID,
        effects::TransactionEvents,
        full_checkpoint_content::{
            CheckpointData as SuiCheckpointData, CheckpointTransaction as SuiCheckpointTransaction,
        },
        messages_checkpoint::{
            CertifiedCheckpointSummary, CheckpointContents as SuiCheckpointContents,
        },
        object::Object,
        transaction::Transaction,
    },
    SuiClientBuilder,
};
use tracing::instrument;
use unionlabs::{ibc::core::client::height::Height, primitives::FixedBytes};
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

#[async_trait]
impl ProofModuleServer<IbcUnion> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_ibc_proof(
        &self,
        _: &Extensions,
        at: Height,
        path: StorePath,
    ) -> RpcResult<(Value, ProofType)> {
        let key = path.key();

        let target_object_id = sui_verifier::calculate_dynamic_field_object_id(
            *self.ibc_commitments_object_id.get(),
            key.get().as_slice(),
        );

        println!("target object id: {}", target_object_id);

        let target_object = self
            .sui_client
            .read_api()
            .get_object_with_options(
                SuiObjectID::new(*target_object_id.get()),
                sui_sdk::rpc_types::SuiObjectDataOptions {
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
            .unwrap()
            .data
            .unwrap();

        let previous_tx = target_object.previous_transaction.unwrap();
        let checkpoint_number = self
            .sui_client
            .read_api()
            .get_transaction_with_options(previous_tx, SuiTransactionBlockResponseOptions::new())
            .await
            .unwrap()
            .checkpoint
            .unwrap();

        println!("height: {at}, num: {checkpoint_number}");

        let client = reqwest::Client::new();
        let req = format!("{}/{checkpoint_number}.chk", self.sui_object_store_rpc_url);
        let res = client.get(req).send().await.unwrap().bytes().await.unwrap();

        let (_, checkpoint) = bcs::from_bytes::<(u8, CheckpointData)>(&res).unwrap();

        let tx = checkpoint
            .transactions
            .iter()
            .find(|tx| *tx.transaction.digest() == previous_tx)
            .unwrap();

        let object: Object = target_object.try_into().unwrap();

        let mut buf = vec![];
        bcs::serialize_into(&mut buf, &object).unwrap();

        let object: ObjectInner = bcs::from_bytes(&buf).unwrap();

        Ok((
            into_value(StorageProof {
                checkpoint_contents: checkpoint.checkpoint_contents,
                transaction_effects: tx.effects.clone(),
                object,
            }),
            ProofType::Membership,
        ))
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

use jsonrpsee::{
    Extensions,
    core::{RpcResult, async_trait},
    types::ErrorObject,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sui_light_client_types::{
    CertifiedCheckpointSummary, U64,
    checkpoint_summary::CheckpointContents,
    client_state::{ClientState, ClientStateV1},
    committee::Committee,
    consensus_state::ConsensusState,
    fixed_bytes::SuiFixedBytes,
};
use sui_sdk::{
    SuiClient, SuiClientBuilder,
    rpc_types::{CheckpointId, SuiCommittee},
    types::{base_types::ObjectID, full_checkpoint_content::CheckpointTransaction},
};
use tracing::instrument;
use unionlabs::{ErrorReporter, ibc::core::client::height::Height};
use voyager_sdk::{
    anyhow, ensure_null,
    plugin::ClientBootstrapModule,
    primitives::{ChainId, ClientType},
    rpc::{ClientBootstrapModuleServer, types::ClientBootstrapModuleInfo},
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub ibc_store: ObjectID,

    /// The address of the IBC smart contract.
    pub ibc_contract: ObjectID,

    pub sui_client: SuiClient,

    pub sui_object_store_rpc_url: String,

    pub ibc_commitments_object_id: ObjectID,
}

impl ClientBootstrapModule for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: ClientBootstrapModuleInfo) -> anyhow::Result<Self> {
        let sui_client = SuiClientBuilder::default().build(&config.rpc_url).await?;

        let chain_id = sui_client.read_api().get_chain_identifier().await?;

        info.ensure_chain_id(&chain_id)?;
        info.ensure_client_type(ClientType::SUI)?;

        Ok(Self {
            chain_id: ChainId::new(chain_id.to_string()),
            ibc_contract: config.ibc_contract,
            ibc_store: config.ibc_store,
            sui_object_store_rpc_url: config.sui_object_store_rpc_url,
            sui_client,
            ibc_commitments_object_id: config.ibc_commitments_object_id,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The address of the `IBCHandler` smart contract.
    pub ibc_contract: ObjectID,

    pub ibc_store: ObjectID,

    pub rpc_url: String,

    pub sui_object_store_rpc_url: String,

    pub ibc_commitments_object_id: ObjectID,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckpointData {
    pub checkpoint_summary: CertifiedCheckpointSummary,
    pub checkpoint_contents: CheckpointContents,
    pub transactions: Vec<CheckpointTransaction>,
}

#[async_trait]
impl ClientBootstrapModuleServer for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn self_client_state(
        &self,
        _: &Extensions,
        height: Height,
        config: Value,
    ) -> RpcResult<Value> {
        ensure_null(config)?;

        let latest_checkpoint = self
            .sui_client
            .read_api()
            .get_checkpoint(CheckpointId::SequenceNumber(height.height()))
            .await
            .map_err(|e| ErrorObject::owned(-1, ErrorReporter(e).to_string(), None::<()>))?;

        let committee = self
            .sui_client
            .governance_api()
            .get_committee_info(Some(latest_checkpoint.epoch.into()))
            .await
            .map_err(|e| ErrorObject::owned(-1, ErrorReporter(e).to_string(), None::<()>))?;

        Ok(serde_json::to_value(ClientState::V1(ClientStateV1 {
            chain_id: self.chain_id.to_string(),
            latest_checkpoint: height.height(),
            frozen_height: 0,
            initial_committee: Some(convert_committee(committee)),
            ibc_commitments_object_id: sui_light_client_types::ObjectID::new(
                self.ibc_commitments_object_id.into_bytes(),
            ),
        }))
        .expect("infallible"))
    }

    /// The consensus state on this chain at the specified `Height`.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn self_consensus_state(
        &self,
        _: &Extensions,
        height: Height,
        config: Value,
    ) -> RpcResult<Value> {
        ensure_null(config)?;

        // TODO(aeryz): imma fix it bro chill
        let client = reqwest::Client::new();
        let req = format!("{}/{}.chk", self.sui_object_store_rpc_url, height.height());
        let res = client
            .get(req)
            .send()
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    ErrorReporter(e).with_message("error fetching the checkpoint"),
                    None::<()>,
                )
            })?
            .bytes()
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    ErrorReporter(e).with_message("error fetching the checkpoint"),
                    None::<()>,
                )
            })?;

        let (_, checkpoint) =
            bcs::from_bytes::<(u8, CheckpointData)>(&res).expect("can decode checkpoint data");

        Ok(serde_json::to_value(ConsensusState {
            timestamp: checkpoint.checkpoint_summary.data.sequence_number,
            content_digest: checkpoint.checkpoint_summary.data.content_digest,
        })
        .expect("infallible"))
    }
}

fn convert_committee(committee: SuiCommittee) -> Committee {
    Committee {
        epoch: U64(committee.epoch),
        voting_rights: committee
            .validators
            .into_iter()
            .map(|(pubkey, power)| (SuiFixedBytes(pubkey.0.into()), U64(power)))
            .collect(),
    }
}

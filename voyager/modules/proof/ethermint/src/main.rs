#![warn(clippy::unwrap_used)]

use std::{
    error::Error,
    fmt::{Debug, Display},
    num::ParseIntError,
};

use ibc_union_spec::{path::StorePath, IbcUnion};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::{ErrorObject, ErrorObjectOwned},
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, instrument};
use unionlabs::{
    bounded::BoundedI64,
    cosmos::ics23::commitment_proof::CommitmentProof,
    ethereum::ibc_commitment_key,
    ibc::core::{client::height::Height, commitment::merkle_proof::MerkleProof},
    primitives::{Bytes, H160},
    ErrorReporter,
};
use voyager_message::{
    into_value,
    module::{ProofModuleInfo, ProofModuleServer},
    primitives::ChainId,
    rpc::ProofType,
    ProofModule, FATAL_JSONRPC_ERROR_CODE, MISSING_STATE_ERROR_CODE,
};
use voyager_vm::BoxDynError;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    <Module as ProofModule<IbcUnion>>::run().await;
}

#[derive(clap::Subcommand)]
pub enum Cmd {
    ChainId,
    LatestHeight,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,
    pub chain_revision: u64,

    pub cometbft_client: cometbft_rpc::Client,

    pub ibc_handler_address: H160,
    pub store_key: Bytes,
    pub key_prefix_storage: Bytes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub rpc_url: String,
    pub ibc_handler_address: H160,
    pub store_key: Bytes,
    pub key_prefix_storage: Bytes,
}

impl ProofModule<IbcUnion> for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: ProofModuleInfo) -> Result<Self, BoxDynError> {
        let tm_client = cometbft_rpc::Client::new(config.rpc_url).await?;

        let chain_id = tm_client.status().await?.node_info.network;

        info.ensure_chain_id(&chain_id)?;

        let chain_revision = chain_id
            .split('-')
            .next_back()
            .ok_or_else(|| ChainIdParseError {
                found: chain_id.clone(),
                source: None,
            })?
            .parse()
            .map_err(|err| ChainIdParseError {
                found: chain_id.clone(),
                source: Some(err),
            })?;

        Ok(Self {
            cometbft_client: tm_client,
            chain_id: ChainId::new(chain_id),
            chain_revision,
            ibc_handler_address: config.ibc_handler_address,
            store_key: config.store_key,
            key_prefix_storage: config.key_prefix_storage,
        })
    }
}

impl Module {
    #[must_use]
    pub fn make_height(&self, height: u64) -> Height {
        Height::new_with_revision(self.chain_revision, height)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("unable to parse chain id: expected format `<chain>-<revision-number>`, found `{found}`")]
pub struct ChainIdParseError {
    found: String,
    #[source]
    source: Option<ParseIntError>,
}

#[async_trait]
impl ProofModuleServer<IbcUnion> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_ibc_proof(
        &self,
        _: &Extensions,
        at: Height,
        path: StorePath,
    ) -> RpcResult<Option<(Value, ProofType)>> {
        let data = self
            .key_prefix_storage
            .clone()
            .into_iter()
            .chain(self.ibc_handler_address)
            .chain(ibc_commitment_key(path.key()).to_be_bytes())
            .collect::<Vec<_>>();

        let query_result = self
            .cometbft_client
            .abci_query(
                format!(
                    "store/{}/key",
                    String::from_utf8(self.store_key.to_vec()).expect("idk how to handle this")
                ),
                data,
                // THIS -1 IS VERY IMPORTANT!!!
                //
                // a proof at height H is provable at height H + 1
                // we assume that the height passed in to this function is the intended height to prove against, thus we have to query the height - 1
                Some(BoundedI64::new(at.height() - 1).map_err(|e| {
                    let message = format!("invalid height value: {}", ErrorReporter(e));
                    error!(%message);
                    ErrorObject::owned(
                        FATAL_JSONRPC_ERROR_CODE,
                        message,
                        Some(json!({ "height": at })),
                    )
                })?),
                true,
            )
            .await
            .map_err(rpc_error("error querying ibc proof", None))?;

        // if this field is none, the proof is not available at this height
        let Some(proofs) = query_result.response.proof_ops else {
            return Ok(None);
        };

        let proofs = proofs
            .ops
            .into_iter()
            .map(|op| {
                <protos::cosmos::ics23::v1::CommitmentProof as prost::Message>::decode(&*op.data)
                    .map_err(|e| {
                        ErrorObject::owned(
                            FATAL_JSONRPC_ERROR_CODE,
                            format!("invalid height value: {}", ErrorReporter(e)),
                            Some(json!({ "height": at })),
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let proof =
            MerkleProof::try_from(protos::ibc::core::commitment::v1::MerkleProof { proofs })
                .map_err(|e| {
                    ErrorObject::owned(
                        FATAL_JSONRPC_ERROR_CODE,
                        format!("invalid merkle proof value: {}", ErrorReporter(e)),
                        Some(json!({ "height": at })),
                    )
                })?;

        let proof_type = if proof
            .proofs
            .iter()
            .any(|p| matches!(&p, CommitmentProof::Nonexist(_)))
        {
            ProofType::NonMembership
        } else {
            ProofType::Membership
        };

        Ok(Some((into_value(proof), proof_type)))
    }
}

// NOTE: For both of the below functions, `message` as a field will override any actual message put in (i.e. `error!("foo", message = "bar")` will print as "bar", not "foo" with an extra field `message = "bar"`.

fn rpc_error<E: Error>(
    message: impl Display,
    data: Option<Value>,
) -> impl FnOnce(E) -> ErrorObjectOwned {
    move |e| {
        let message = format!("{message}: {}", ErrorReporter(e));
        error!(%message, data = %data.as_ref().unwrap_or(&serde_json::Value::Null));
        ErrorObject::owned(-1, message, data)
    }
}

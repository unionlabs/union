#![warn(clippy::unwrap_used)]

use std::num::ParseIntError;

use ibc_union_spec::{
    IbcUnion,
    path::{IBC_UNION_COSMWASM_COMMITMENT_PREFIX, StorePath},
};
use jsonrpsee::{
    Extensions,
    core::{RpcResult, async_trait},
    types::ErrorObject,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tracing::{error, instrument, warn};
use unionlabs::{
    ErrorReporter,
    bounded::BoundedI64,
    cosmos::ics23::commitment_proof::CommitmentProof,
    ibc::core::{client::height::Height, commitment::merkle_proof::MerkleProof},
    primitives::{Bech32, H256},
};
use voyager_sdk::{
    anyhow, into_value,
    plugin::ProofModule,
    primitives::ChainId,
    rpc::{FATAL_JSONRPC_ERROR_CODE, ProofModuleServer, rpc_error, types::ProofModuleInfo},
    types::ProofType,
};

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

    pub ibc_host_contract_address: Bech32<H256>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub rpc_url: String,
    pub ibc_host_contract_address: Bech32<H256>,
}

impl ProofModule<IbcUnion> for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: ProofModuleInfo) -> anyhow::Result<Self> {
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
            ibc_host_contract_address: config.ibc_host_contract_address,
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
        // TODO: Extract this into a function somewhere, reuse in lightclients
        let data = [0x03]
            .into_iter()
            .chain(*self.ibc_host_contract_address.data())
            .chain(IBC_UNION_COSMWASM_COMMITMENT_PREFIX)
            .chain(path.key())
            .collect::<Vec<_>>();

        let query_result = self
            .cometbft_client
            .abci_query(
                "store/wasm/key",
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

#![warn(clippy::unwrap_used)]

use std::num::ParseIntError;

use cometbft_rpc::types::CometbftHeight;
use ibc_union_spec::{IbcUnion, path::StorePath};
use jsonrpsee::{Extensions, core::async_trait};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{instrument, warn};
use unionlabs::{
    bounded::BoundedI64,
    cosmos::ics23::commitment_proof::CommitmentProof,
    ethereum::ibc_commitment_key,
    ibc::core::{client::height::Height, commitment::merkle_proof::MerkleProof},
    primitives::{Bytes, H160},
};
use voyager_sdk::{
    anyhow, into_value,
    plugin::ProofModule,
    primitives::ChainId,
    rpc::{ProofModuleServer, RpcError, RpcResult, types::ProofModuleInfo},
    types::ProofType,
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await;
}

#[derive(clap::Subcommand)]
pub enum Cmd {
    ChainId,
    LatestHeight,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

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

    async fn new(config: Self::Config, info: ProofModuleInfo) -> anyhow::Result<Self> {
        let tm_client = cometbft_rpc::Client::new(config.rpc_url).await?;

        let chain_id = tm_client.status().await?.node_info.network;

        info.ensure_chain_id(&chain_id)?;

        Ok(Self {
            cometbft_client: tm_client,
            chain_id: ChainId::new(chain_id),
            ibc_handler_address: config.ibc_handler_address,
            store_key: config.store_key,
            key_prefix_storage: config.key_prefix_storage,
        })
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
                Some(
                    CometbftHeight::try_from(at.height() - 1)
                        .map_err(RpcError::fatal("invalid height value: {at}"))?,
                ),
                true,
            )
            .await
            .map_err(RpcError::retryable("error querying ibc proof"))?;

        // if this field is none, the proof is not available at this height
        let Some(proofs) = query_result.response.proof_ops else {
            return Ok(None);
        };

        let proofs = proofs
            .ops
            .into_iter()
            .map(|op| {
                <protos::cosmos::ics23::v1::CommitmentProof as prost::Message>::decode(&*op.data)
                    .map_err(RpcError::fatal("invalid commitment proof value"))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let proof =
            MerkleProof::try_from(protos::ibc::core::commitment::v1::MerkleProof { proofs })
                .map_err(RpcError::fatal("invalid merkle proof value"))?;

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

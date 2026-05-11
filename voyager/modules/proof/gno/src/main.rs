#![warn(clippy::unwrap_used)]

use ibc_union_spec::{IbcUnion, path::StorePath};
use jsonrpsee::{Extensions, core::async_trait};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{instrument, warn};
use unionlabs::{
    bounded::BoundedI64,
    cosmos::ics23::commitment_proof::CommitmentProof,
    ibc::core::client::height::Height,
    primitives::{Bytes, encoding::HexUnprefixed},
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

    pub gno_client: gno_rpc::Client,

    pub ibc_core_realm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub rpc_url: String,
    pub ibc_core_realm: String,
}

impl ProofModule<IbcUnion> for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: ProofModuleInfo) -> anyhow::Result<Self> {
        let gno_client = gno_rpc::Client::new(config.rpc_url).await?;

        let chain_id = gno_client.status(None).await?.node_info.network;

        info.ensure_chain_id(&chain_id)?;

        Ok(Self {
            gno_client,
            chain_id: ChainId::new(chain_id),
            ibc_core_realm: config.ibc_core_realm,
        })
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
    ) -> RpcResult<Option<(Value, ProofType)>> {
        let data = gnovm_store_key(&self.ibc_core_realm, path.key().into());

        let query_result = self
            .gno_client
            .abci_query(
                ".store/main/key",
                data,
                // THIS -1 IS VERY IMPORTANT!!!
                //
                // a proof at height H is provable at height H + 1
                // we assume that the height passed in to this function is the intended height to prove against, thus we have to query the height - 1
                Some(
                    BoundedI64::new(at.height() - 1)
                        .map_err(RpcError::fatal("invalid height value: {at}"))?,
                ),
                true,
            )
            .await
            .map_err(RpcError::retryable("error querying ibc proof"))?;

        // if this field is none, the proof is not available at this height
        if query_result.response.proof.is_none() {
            return Ok(None);
        };

        let proof = query_result
            .decode_merkle_proof()
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

fn gnovm_store_key(realm: &str, key: Vec<u8>) -> Vec<u8> {
    format!("/pv/vm:{realm}:{}", <Bytes<HexUnprefixed>>::new(key)).into_bytes()
}

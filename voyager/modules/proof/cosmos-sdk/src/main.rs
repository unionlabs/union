// #![warn(clippy::unwrap_used)]

use std::{
    error::Error,
    fmt::{Debug, Display},
    num::{NonZeroU32, ParseIntError},
};

use ibc_classic_spec::{IbcClassic, StorePath};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::{ErrorObject, ErrorObjectOwned},
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, instrument};
use unionlabs::{
    cosmos::ics23::commitment_proof::CommitmentProof,
    ibc::core::{client::height::Height, commitment::merkle_proof::MerkleProof},
    option_unwrap, ErrorReporter,
};
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
    <Module as ProofModule<IbcClassic>>::run().await;
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub rpc_url: String,
}

impl ProofModule<IbcClassic> for Module {
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
impl ProofModuleServer<IbcClassic> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_ibc_proof(
        &self,
        _: &Extensions,
        at: Height,
        path: StorePath,
    ) -> RpcResult<(Value, ProofType)> {
        const IBC_STORE_PATH: &str = "store/ibc/key";

        let path_string = path.to_string();

        let query_result = self
            .cometbft_client
            .abci_query(
                IBC_STORE_PATH,
                &path_string,
                // a proof at height H is provable at height H + 1
                // we assume that the height passed in to this function is the intended height to prove against, thus we have to query the height - 1
                Some(
                    (i64::try_from(at.height()).expect("should be fine") - 1)
                        .try_into()
                        .expect("invalid height"),
                ),
                true,
            )
            .await
            .map_err(rpc_error(
                format_args!("error fetching abci query"),
                Some(json!({ "height": at, "path": path })),
            ))?;

        // https://github.com/cosmos/cosmos-sdk/blob/e2027bf62893bb5f82e8f7a8ea59d1a43eb6b78f/baseapp/abci.go#L1272-L1278
        if query_result
            .response
            .code
            .is_err_code(option_unwrap!(NonZeroU32::new(26)))
        {
            return Err(ErrorObject::owned(
                -1,
                "attempted to query state at a nonexistent height, \
                potentially due to load balanced rpc endpoints",
                Some(json!({
                    "height": at,
                    "path": path
                })),
            ));
        }

        let proofs = query_result
            .response
            .proof_ops
            .unwrap()
            .ops
            .into_iter()
            .map(|op| {
                <protos::cosmos::ics23::v1::CommitmentProof as prost::Message>::decode(&*op.data)
                    .unwrap()
            })
            .collect::<Vec<_>>();

        let proof =
            MerkleProof::try_from(protos::ibc::core::commitment::v1::MerkleProof { proofs })
                .unwrap();

        let proof_type = if proof
            .proofs
            .iter()
            .any(|p| matches!(&p, CommitmentProof::Nonexist(_)))
        {
            ProofType::NonMembership
        } else {
            ProofType::Membership
        };

        Ok((into_value(proof), proof_type))
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

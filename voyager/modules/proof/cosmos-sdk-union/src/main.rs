// #![warn(clippy::unwrap_used)]

use std::{
    error::Error,
    fmt::{Debug, Display},
    num::ParseIntError,
};

use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::{ErrorObject, ErrorObjectOwned},
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{error, instrument};
use union_ibc::state::CONNECTIONS;
use unionlabs::{
    bounded::BoundedI64,
    ibc::core::{client::height::Height, commitment::merkle_proof::MerkleProof},
    ics24::ethabi::Path,
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

    pub tm_client: cometbft_rpc::Client,
    pub grpc_url: String,

    pub ibc_union_contract_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub ws_url: String,
    pub grpc_url: String,
    pub ibc_union_contract_address: String,
}

impl ProofModule<IbcUnion> for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: ProofModuleInfo) -> Result<Self, BoxDynError> {
        let tm_client = cometbft_rpc::Client::new(config.ws_url).await?;

        let chain_id = tm_client.status().await?.node_info.network;

        info.ensure_chain_id(&chain_id)?;

        let chain_revision = chain_id
            .split('-')
            .last()
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
            tm_client,
            chain_id: ChainId::new(chain_id),
            chain_revision,
            grpc_url: config.grpc_url,
            ibc_union_contract_address: config.ibc_union_contract_address,
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
    async fn query_ibc_proof(&self, _: &Extensions, at: Height, path: Path) -> RpcResult<Value> {
        let key = match path {
            // Path::ClientState(path) => self
            //     .query_client_state(at, path.client_id)
            //     .await
            //     .map(into_value),
            // Path::ConsensusState(path) => self
            //     .query_consensus_state(at, path.client_id, path.height)
            //     .await
            //     .map(into_value),
            Path::Connection(path) => CONNECTIONS.key(path.connection_id),
            Path::Channel(_path) => todo!(),
            _ => {
                todo!()
            }
        };

        let data = [0x03]
            .into_iter()
            .chain(
                subtle_encoding::bech32::decode(&self.ibc_union_contract_address)
                    .unwrap()
                    .1,
            )
            .chain(key.to_vec())
            .collect::<Vec<_>>();

        let query_result = self
            .tm_client
            .abci_query(
                "store/wasm/key",
                data,
                Some(BoundedI64::new(at.height()).unwrap()),
                true,
            )
            .await
            .map_err(rpc_error("error querying connection proof", None))?;

        Ok(into_value(
            MerkleProof::try_from(protos::ibc::core::commitment::v1::MerkleProof {
                proofs: query_result
                    .response
                    .proof_ops
                    .unwrap()
                    .ops
                    .into_iter()
                    .map(|op| {
                        <protos::cosmos::ics23::v1::CommitmentProof as prost::Message>::decode(
                            &*op.data,
                        )
                        .unwrap()
                    })
                    .collect::<Vec<_>>(),
            })
            .unwrap(),
        ))
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

// #[test]
// fn commitment_key() {
//     let key = [0x03].into_iter().chain()

//     println!("{}", serde_utils::Hex(&*CONNECTIONS.key(2)))
// }

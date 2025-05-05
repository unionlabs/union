use std::{
    collections::VecDeque,
    error::Error,
    fmt::{Debug, Display},
    num::ParseIntError,
};

use alloy::providers::{DynProvider, Provider, ProviderBuilder};
use beacon_api_types::{chain_spec::Mainnet, deneb};
use berachain_light_client_types::Header;
use cometbft_types::types::{validator::Validator, validator_set::ValidatorSet};
use ethereum_light_client_types::AccountProof;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::{ErrorObject, ErrorObjectOwned},
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::instrument;
use unionlabs::{
    berachain::LATEST_EXECUTION_PAYLOAD_HEADER_PREFIX,
    encoding::{DecodeAs, Ssz},
    ibc::core::{client::height::Height, commitment::merkle_proof::MerkleProof},
    never::Never,
    primitives::{encoding::HexUnprefixed, H160},
    ErrorReporter,
};
use voyager_message::{
    call::Call,
    data::{Data, DecodedHeaderMeta, OrderedHeaders},
    hook::UpdateHook,
    into_value,
    module::{PluginInfo, PluginServer},
    primitives::{ChainId, ClientType},
    DefaultCmd, Plugin, PluginMessage, VoyagerMessage,
};
use voyager_vm::{data, pass::PassResult, BoxDynError, Op, Visit};

use crate::call::{FetchUpdate, ModuleCall};

pub mod call;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,
    pub ibc_handler_address: H160,
    pub eth_provider: DynProvider,
    pub cometbft_client: cometbft_rpc::Client,
    pub chain_revision: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId,
    pub ibc_handler_address: H160,
    pub comet_rpc_url: String,
    pub eth_rpc_url: String,
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = Never;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        let eth_provider =
            DynProvider::new(ProviderBuilder::new().connect(&config.eth_rpc_url).await?);

        let chain_id = ChainId::new(eth_provider.get_chain_id().await?.to_string());

        if chain_id != config.chain_id {
            return Err(format!(
                "incorrect chain id: expected `{}`, but found `{}`",
                config.chain_id, chain_id
            )
            .into());
        }

        let tm_client = cometbft_rpc::Client::new(config.comet_rpc_url).await?;

        Ok(Self {
            chain_id: config.chain_id,
            ibc_handler_address: config.ibc_handler_address,
            eth_provider,
            cometbft_client: tm_client,
            chain_revision: chain_id
                .as_str()
                .parse()
                .expect("expected a numeric chain id"),
        })
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.chain_id),
            interest_filter: UpdateHook::filter(
                &config.chain_id,
                &ClientType::new(ClientType::BERACHAIN),
            ),
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

fn plugin_name(chain_id: &ChainId) -> String {
    pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

    format!("{PLUGIN_NAME}/{}", chain_id)
}

impl Module {
    fn plugin_name(&self) -> String {
        plugin_name(&self.chain_id)
    }

    // TODO: deduplicate with eth client update module?
    pub async fn fetch_account_update(&self, block_number: u64) -> RpcResult<AccountProof> {
        let account_update = self
            .eth_provider
            .get_proof(self.ibc_handler_address.into(), vec![])
            .block_id(block_number.into())
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    ErrorReporter(e).with_message("error fetching account update"),
                    None::<()>,
                )
            })?;

        Ok(AccountProof {
            storage_root: account_update.storage_hash.into(),
            proof: account_update
                .account_proof
                .into_iter()
                .map(|x| x.into())
                .collect(),
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
impl PluginServer<ModuleCall, Never> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn run_pass(
        &self,
        _: &Extensions,
        msgs: Vec<Op<VoyagerMessage>>,
    ) -> RpcResult<PassResult<VoyagerMessage>> {
        Ok(PassResult {
            optimize_further: vec![],
            ready: msgs
                .into_iter()
                .map(|mut op| {
                    UpdateHook::new(
                        &self.chain_id,
                        &ClientType::new(ClientType::BERACHAIN),
                        |fetch| {
                            Call::Plugin(PluginMessage::new(
                                self.plugin_name(),
                                ModuleCall::from(FetchUpdate {
                                    update_from: fetch.update_from,
                                    update_to: fetch.update_to,
                                }),
                            ))
                        },
                    )
                    .visit_op(&mut op);

                    op
                })
                .enumerate()
                .map(|(i, op)| (vec![i], op))
                .collect(),
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn call(&self, _: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::FetchUpdate(FetchUpdate {
                update_from,
                update_to,
            }) => {
                let trusted_height = update_from
                    .increment()
                    .height()
                    .try_into()
                    .expect("valid height");
                let untrusted_height = update_to.height().try_into().expect("valid height");

                let trusted_commit = self
                    .cometbft_client
                    .commit(Some(trusted_height))
                    .await
                    .map_err(rpc_error("trusted commit", None))?;

                let untrusted_commit = self
                    .cometbft_client
                    .commit(Some(untrusted_height))
                    .await
                    .map_err(rpc_error("untrusted commit", None))?;

                let trusted_validators = self
                    .cometbft_client
                    .all_validators(Some(trusted_height))
                    .await
                    .map_err(rpc_error("trusted validators", None))?;

                let untrusted_validators = self
                    .cometbft_client
                    .all_validators(Some(untrusted_height))
                    .await
                    .map_err(rpc_error("untrusted validators", None))?;

                // NOTE: The implementation is simple because the cometbft/beacon/execution heights are guaranteed to be the same (i.e. we don't need to find the consensus height to match the execution height)
                let query_result = self
                    .cometbft_client
                    .abci_query(
                        "store/beacon/key",
                        [LATEST_EXECUTION_PAYLOAD_HEADER_PREFIX],
                        // proof for height H must be queried at H-1
                        Some((update_to.height() as i64 - 1).try_into().unwrap()),
                        true,
                    )
                    .await
                    .unwrap();

                let execution_header =
                    deneb::ExecutionPayloadHeaderSsz::<Mainnet>::decode_as::<Ssz>(
                        query_result.response.value.expect("big trouble").as_ref(),
                    )
                    .expect("big trouble");

                let account_proof = self.fetch_account_update(update_to.height()).await?;

                let header = Header {
                    tm_header: tendermint_light_client_types::header::Header {
                        validator_set: mk_validator_set(
                            untrusted_validators.validators,
                            untrusted_commit.signed_header.header.proposer_address,
                        ),
                        signed_header: untrusted_commit.signed_header,
                        trusted_height: Height::new_with_revision(
                            self.chain_revision,
                            update_from.height(),
                        ),
                        trusted_validators: mk_validator_set(
                            trusted_validators.validators,
                            trusted_commit.signed_header.header.proposer_address,
                        ),
                    },
                    execution_header: execution_header.into(),
                    execution_header_proof:
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
                        }).unwrap(),
                    account_proof,
                };

                Ok(data(OrderedHeaders {
                    headers: vec![(DecodedHeaderMeta { height: update_to }, into_value(header))],
                }))
            }
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn callback(
        &self,
        _: &Extensions,
        callback: Never,
        _data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match callback {}
    }
}

fn mk_validator_set(
    validators: Vec<Validator>,
    proposer_address: H160<HexUnprefixed>,
) -> ValidatorSet {
    let proposer = validators
        .iter()
        .find(|val| val.address == proposer_address)
        .expect("proposer must exist in set")
        .clone();

    let total_voting_power = validators
        .iter()
        .map(|v| v.voting_power.inner())
        .sum::<i64>();

    ValidatorSet {
        validators,
        proposer,
        total_voting_power,
    }
}

fn rpc_error<E: Error>(
    message: impl Display,
    data: Option<Value>,
) -> impl FnOnce(E) -> ErrorObjectOwned {
    move |e| {
        let message = format!("{message}: {}", ErrorReporter(e));
        // error!(%message, data = %data.as_ref().unwrap_or(&serde_json::Value::Null));
        ErrorObject::owned(-1, message, data)
    }
}

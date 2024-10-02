use std::{
    collections::{HashMap, VecDeque},
    num::ParseIntError,
};

use itertools::Itertools;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use num_bigint::BigUint;
use protos::union::galois::api::v3::union_prover_api_client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, instrument, trace};
use unionlabs::{
    bounded::BoundedI64,
    cometbls::types::canonical_vote::CanonicalVote,
    tendermint::{
        crypto::public_key::PublicKey,
        types::{
            canonical_block_header::CanonicalPartSetHeader, canonical_block_id::CanonicalBlockId,
            commit_sig::CommitSig, signed_msg_type::SignedMsgType,
            simple_validator::SimpleValidator,
        },
    },
    union::galois::{
        poll_request::PollRequest,
        poll_response::{PollResponse, ProveRequestDone, ProveRequestFailed},
        prove_request::ProveRequest,
        validator_set_commit::ValidatorSetCommit,
    },
};
use voyager_message::{
    call::{Call, WaitForHeight},
    core::ChainId,
    data::Data,
    hook::UpdateHook,
    module::{PluginInfo, PluginServer},
    run_plugin_server, DefaultCmd, Plugin, PluginMessage, VoyagerMessage,
};
use voyager_vm::{
    call, data, defer, now, optimize::OptimizationResult, promise, seq, void, BoxDynError, Op,
};

use crate::{
    call::{FetchProveRequest, FetchUpdate, ModuleCall},
    callback::{AggregateHeader, ModuleCallback},
    data::{ModuleData, ProveResponse},
};

pub mod call;
pub mod callback;
pub mod data;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_plugin_server::<Module>().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId<'static>,

    pub tm_client: cometbft_rpc::Client,
    pub chain_revision: u64,
    pub grpc_url: String,

    pub prover_endpoints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub chain_id: ChainId<'static>,

    pub ws_url: String,
    pub grpc_url: String,

    pub prover_endpoints: Vec<String>,
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        let tm_client = cometbft_rpc::Client::new(config.ws_url).await?;

        let chain_id = tm_client.status().await?.node_info.network.to_string();

        if chain_id != config.chain_id.as_str() {
            return Err(format!(
                "incorrect chain id: expected `{}`, but found `{}`",
                config.chain_id, chain_id
            )
            .into());
        }

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
            prover_endpoints: config.prover_endpoints,
            grpc_url: config.grpc_url,
        })
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.chain_id),
            interest_filter: UpdateHook::filter(&config.chain_id),
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

fn plugin_name(chain_id: &ChainId<'_>) -> String {
    pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

    format!("{PLUGIN_NAME}/{}", chain_id)
}

impl Module {
    fn plugin_name(&self) -> String {
        plugin_name(&self.chain_id)
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
impl PluginServer<ModuleCall, ModuleCallback> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn run_pass(
        &self,
        _: &Extensions,
        msgs: Vec<Op<VoyagerMessage>>,
    ) -> RpcResult<OptimizationResult<VoyagerMessage>> {
        Ok(OptimizationResult {
            optimize_further: vec![],
            ready: msgs
                .into_iter()
                .map(|mut op| {
                    op.visit(&mut UpdateHook::new(&self.chain_id, |fetch| {
                        Call::Plugin(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(FetchUpdate {
                                update_from: fetch.update_from,
                                update_to: fetch.update_to,
                            }),
                        ))
                    }));

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
                let trusted_validators = self
                    .tm_client
                    .all_validators(Some(update_from.revision_height.try_into().unwrap()))
                    .await
                    .unwrap()
                    .validators;

                let untrusted_validators = self
                    .tm_client
                    .all_validators(Some(update_to.revision_height.try_into().unwrap()))
                    .await
                    .unwrap()
                    .validators;

                let signed_header = self
                    .tm_client
                    .commit(Some(update_to.revision_height.try_into().unwrap()))
                    .await
                    .unwrap()
                    .signed_header;

                let make_validators_commit = |mut validators: Vec<
                    unionlabs::tendermint::types::validator::Validator,
                >| {
                    // Validators must be sorted to match the root, by token then address
                    validators.sort_by(|a, b| {
                        // TODO: Double check how these comparisons are supposed to work
                        #[allow(clippy::collapsible_else_if)]
                        if a.voting_power == b.voting_power {
                            if a.address < b.address {
                                std::cmp::Ordering::Less
                            } else {
                                std::cmp::Ordering::Greater
                            }
                        } else {
                            if a.voting_power > b.voting_power {
                                std::cmp::Ordering::Less
                            } else {
                                std::cmp::Ordering::Greater
                            }
                        }
                    });

                    // The bitmap is a public input of the circuit, it must fit in Fr (scalar field) bn254
                    let mut bitmap = BigUint::default();
                    // REVIEW: This will over-allocate for the trusted validators; should be benchmarked
                    let mut signatures = Vec::<Vec<u8>>::with_capacity(validators.len());

                    let validators_map = validators
                        .iter()
                        .enumerate()
                        .map(|(i, v)| (v.address, i))
                        .collect::<HashMap<_, _>>();

                    // For each validator signature, we search for the actual validator
                    // in the set and set it's signed bit to 1. We then push the
                    // signature only if the validator signed. It's possible that we
                    // don't find a validator for a given signature as the validator set
                    // may have drifted (trusted validator set).
                    for sig in signed_header.commit.signatures.iter() {
                        match sig {
                            CommitSig::Absent => {
                                debug!("validator did not sign");
                            }
                            CommitSig::Commit {
                                validator_address,
                                timestamp: _,
                                signature,
                            } => {
                                if let Some(validator_index) = validators_map.get(validator_address)
                                {
                                    bitmap.set_bit(*validator_index as u64, true);
                                    signatures.push(signature.clone());
                                    trace!(
                                        %validator_address,
                                        %validator_index,
                                        "validator signed"
                                    );
                                } else {
                                    trace!(
                                        %validator_address,
                                        "validator set drifted, could not find validator signature"
                                    );
                                }
                            }
                            CommitSig::Nil {
                                validator_address, ..
                            } => {
                                trace!(
                                    %validator_address,
                                    "validator commit is nil"
                                );
                            }
                        }
                    }

                    let simple_validators = validators
                        .iter()
                        .map(|v| {
                            let PublicKey::Bn254(ref key) = v.pub_key else {
                                panic!("must be bn254")
                            };
                            SimpleValidator {
                                pub_key: PublicKey::Bn254(key.to_vec()),
                                voting_power: v.voting_power.into(),
                            }
                        })
                        .collect::<Vec<_>>();

                    ValidatorSetCommit {
                        validators: simple_validators,
                        signatures,
                        bitmap: bitmap.to_bytes_be(),
                    }
                };

                let trusted_validators_commit = make_validators_commit(trusted_validators);
                let untrusted_validators_commit = make_validators_commit(untrusted_validators);

                Ok(seq([
                    void(call(WaitForHeight {
                        chain_id: self.chain_id.clone(),
                        height: update_to,
                    })),
                    promise(
                        [call(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(FetchProveRequest {
                                request: ProveRequest {
                                    vote: CanonicalVote {
                                        // REVIEW: Should this be hardcoded to precommit?
                                        ty: SignedMsgType::Precommit,
                                        height: signed_header.commit.height,
                                        round: BoundedI64::new(
                                            signed_header.commit.round.inner().into(),
                                        )
                                        .expect(
                                            "0..=i32::MAX can be converted to 0..=i64::MAX safely",
                                        ),
                                        block_id: CanonicalBlockId {
                                            hash: signed_header
                                                .commit
                                                .block_id
                                                .hash
                                                .unwrap_or_default(),
                                            part_set_header: CanonicalPartSetHeader {
                                                total: signed_header
                                                    .commit
                                                    .block_id
                                                    .part_set_header
                                                    .total,
                                                hash: signed_header
                                                    .commit
                                                    .block_id
                                                    .part_set_header
                                                    .hash
                                                    .unwrap_or_default(),
                                            },
                                        },
                                        chain_id: signed_header.header.chain_id.clone(),
                                    },
                                    untrusted_header: signed_header.header.clone(),
                                    trusted_commit: trusted_validators_commit,
                                    untrusted_commit: untrusted_validators_commit,
                                },
                            }),
                        ))],
                        [],
                        PluginMessage::new(
                            self.plugin_name(),
                            ModuleCallback::from(AggregateHeader {
                                chain_id: self.chain_id.clone(),
                                signed_header,
                                update_from,
                                update_to,
                            }),
                        ),
                    ),
                ]))
            }
            ModuleCall::FetchProveRequest(FetchProveRequest { request }) => {
                debug!("submitting prove request");

                let prover_endpoint = &self.prover_endpoints[usize::try_from(
                    request.untrusted_header.height.inner(),
                )
                .expect("never going to happen bro")
                    % self.prover_endpoints.len()];

                let response =
                    union_prover_api_client::UnionProverApiClient::connect(prover_endpoint.clone())
                        .await
                        .unwrap()
                        .poll(protos::union::galois::api::v3::PollRequest::from(
                            PollRequest {
                                request: request.clone(),
                            },
                        ))
                        .await
                        .map(|x| x.into_inner().try_into().unwrap());

                debug!("submitted prove request");

                let retry = || {
                    debug!("proof pending");

                    seq([
                        // REVIEW: How long should we wait between polls?
                        defer(now() + 1),
                        call(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(FetchProveRequest { request }),
                        )),
                    ])
                };
                match response {
                    Ok(PollResponse::Pending) => Ok(retry()),
                    Err(status) if status.message() == "busy_building" => Ok(retry()),
                    Err(err) => panic!("prove request failed: {:?}", err),
                    Ok(PollResponse::Failed(ProveRequestFailed { message })) => {
                        error!(%message, "prove request failed");
                        panic!()
                    }
                    Ok(PollResponse::Done(ProveRequestDone { response })) => {
                        info!(prover = %prover_endpoint, "proof generated");

                        Ok(data(PluginMessage::new(
                            self.plugin_name(),
                            ModuleData::from(ProveResponse {
                                prove_response: response,
                            }),
                        )))
                    }
                }
            }
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn callback(
        &self,
        _: &Extensions,
        callback: ModuleCallback,
        data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        Ok(match callback {
            ModuleCallback::AggregateHeader(aggregate) => self.aggregate_header(
                aggregate,
                data.into_iter()
                    .exactly_one()
                    .unwrap()
                    .as_plugin::<ModuleData>(self.plugin_name())
                    .unwrap()
                    .try_into()
                    .unwrap(),
            ),
        })
    }
}

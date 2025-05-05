use std::{
    collections::{HashMap, VecDeque},
    num::ParseIntError,
};

use call::FetchUpdateBoot;
use cometbft_types::{
    crypto::public_key::PublicKey,
    types::{
        block_id_flag::BlockIdFlag, canonical_block_id::CanonicalBlockId,
        canonical_part_set_header::CanonicalPartSetHeader, commit_sig::CommitSig,
        signed_msg_type::SignedMsgType, simple_validator::SimpleValidator, validator::Validator,
    },
};
use galois_rpc::{
    canonical_vote::CanonicalVote,
    poll_request::PollRequest,
    poll_response::{PollResponse, ProveRequestDone, ProveRequestFailed},
    prove_request::ProveRequest,
    validator_set_commit::ValidatorSetCommit,
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, instrument, trace};
use unionlabs::{bounded::BoundedI64, ibc::core::client::height::Height};
use voyager_message::{
    call::{Call, WaitForHeight},
    data::Data,
    hook::UpdateHook,
    module::{PluginInfo, PluginServer},
    primitives::{ChainId, ClientType},
    DefaultCmd, Plugin, PluginMessage, VoyagerMessage, FATAL_JSONRPC_ERROR_CODE,
};
use voyager_vm::{
    call, data, defer, noop, now, pass::PassResult, promise, seq, void, BoxDynError, Op, Visit,
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
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub cometbft_client: cometbft_rpc::Client,
    pub chain_revision: u64,

    pub prover_endpoints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId,

    pub rpc_url: String,

    pub prover_endpoints: Vec<String>,
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        let cometbft_client = cometbft_rpc::Client::new(config.rpc_url).await?;

        let chain_id = cometbft_client
            .status()
            .await?
            .node_info
            .network
            .to_string();

        if chain_id != config.chain_id.as_str() {
            return Err(format!(
                "incorrect chain id: expected `{}`, but found `{}`",
                config.chain_id, chain_id
            )
            .into());
        }

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
            cometbft_client,
            chain_id: ChainId::new(chain_id),
            chain_revision,
            prover_endpoints: config.prover_endpoints,
        })
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.chain_id),
            interest_filter: UpdateHook::filter(
                &config.chain_id,
                &ClientType::new(ClientType::COMETBLS),
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

    #[instrument(skip_all, fields(%from, %to))]
    async fn find_highest_update_height(&self, from: Height, to: Height) -> Height {
        let sort = |mut validators: Vec<Validator>| {
            validators.sort_by(|a, b| {
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
            validators
                .into_iter()
                .map(|v| (v.address, v))
                .collect::<HashMap<_, _>>()
        };

        let trusted_validators = self
            .cometbft_client
            .all_validators(Some(from.increment().height().try_into().unwrap()))
            .await
            .unwrap()
            .validators;

        let trusted_map = sort(trusted_validators);

        // 1/3 of the trusted power must remain at H+k
        let trusted_power_threshold = trusted_map
            .values()
            .map(|v| v.voting_power.inner())
            .sum::<i64>()
            / 3;

        let mut low = from;
        let mut high = to.increment();
        while low < high {
            let mid = Height::new((low.height() + high.height()) / 2);

            info!("fetching between {low} and {high}, mid = {mid}");

            // 1. fetch commit
            let signed_header = self
                .cometbft_client
                .commit(Some(mid.height().try_into().unwrap()))
                .await
                .unwrap()
                .signed_header;

            // 2. fetch untrusted validators
            let untrusted_validators = self
                .cometbft_client
                .all_validators(Some(mid.height().try_into().unwrap()))
                .await
                .unwrap()
                .validators;

            let untrusted_map = sort(untrusted_validators);

            // 2. compute trusted power
            let mut trusted_power = 0;
            for sig in signed_header.commit.signatures.iter() {
                if sig.block_id_flag == (BlockIdFlag::Commit as i32) {
                    let address = sig.validator_address.as_encoding();
                    match (trusted_map.get(address), untrusted_map.get(address)) {
                        (Some(trusted_validator), Some(untrusted_validator))
                            if trusted_validator.voting_power
                                == untrusted_validator.voting_power =>
                        {
                            trusted_power += trusted_validator.voting_power.inner();
                        }
                        _ => {}
                    }
                }
            }

            info!(%trusted_power, %trusted_power_threshold);

            // 3. ensure trusted power is higher than threshold
            if trusted_power > trusted_power_threshold {
                low = mid.increment();
            } else {
                high = mid;
            }
        }

        Height::new(low.height() - 1)
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
    ) -> RpcResult<PassResult<VoyagerMessage>> {
        Ok(PassResult {
            optimize_further: vec![],
            ready: msgs
                .into_iter()
                .map(|mut op| {
                    UpdateHook::new(
                        &self.chain_id,
                        &ClientType::new(ClientType::COMETBLS),
                        |fetch| {
                            Call::Plugin(PluginMessage::new(
                                self.plugin_name(),
                                ModuleCall::from(FetchUpdateBoot {
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
            ModuleCall::FetchUpdateBoot(FetchUpdateBoot {
                update_from,
                update_to,
            }) => Ok(promise(
                [call(PluginMessage::new(
                    self.plugin_name(),
                    ModuleCall::FetchUpdate(FetchUpdate {
                        update_from,
                        update_to,
                    }),
                ))],
                [],
                PluginMessage::new(self.plugin_name(), ModuleCallback::from(AggregateHeader {})),
            )),
            ModuleCall::FetchUpdate(FetchUpdate {
                update_from,
                update_to,
            }) => {
                if update_from.height() == update_to.height() {
                    info!("update from {update_from} to {update_to} is a noop");
                    return Ok(noop());
                }

                let update_to_highest = self
                    .find_highest_update_height(update_from, update_to)
                    .await;
                if update_to_highest != update_to {
                    let intermediate = call(PluginMessage::new(
                        self.plugin_name(),
                        ModuleCall::from(FetchUpdate {
                            update_from,
                            update_to: update_to_highest,
                        }),
                    ));
                    let continuation = call(PluginMessage::new(
                        self.plugin_name(),
                        ModuleCall::from(FetchUpdate {
                            update_from: update_to_highest,
                            update_to,
                        }),
                    ));
                    return Ok(seq([intermediate, continuation]));
                }

                let trusted_validators = self
                    .cometbft_client
                    .all_validators(Some(update_from.increment().height().try_into().unwrap()))
                    .await
                    .unwrap()
                    .validators;

                let untrusted_validators = self
                    .cometbft_client
                    .all_validators(Some(update_to.height().try_into().unwrap()))
                    .await
                    .unwrap()
                    .validators;

                let signed_header = self
                    .cometbft_client
                    .commit(Some(update_to.height().try_into().unwrap()))
                    .await
                    .unwrap()
                    .signed_header;

                let make_validators_commit = |mut validators: Vec<Validator>| {
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
                        let sig = sig.clone().try_into().unwrap();
                        match sig {
                            CommitSig::Absent => {
                                debug!("validator did not sign");
                            }
                            CommitSig::Commit {
                                validator_address,
                                timestamp: _,
                                signature,
                            } => {
                                if let Some(validator_index) =
                                    validators_map.get(validator_address.as_encoding())
                                {
                                    bitmap.set_bit(*validator_index as u64, true);
                                    signatures.push(signature.clone().into());
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
                                pub_key: PublicKey::Bn254(key.clone()),
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
                        finalized: true,
                    })),
                    call(PluginMessage::new(
                        self.plugin_name(),
                        ModuleCall::from(FetchProveRequest {
                            update_from,
                            request: ProveRequest {
                                vote: CanonicalVote {
                                    // REVIEW: Should this be hardcoded to precommit?
                                    ty: SignedMsgType::Precommit,
                                    height: signed_header.commit.height,
                                    round: BoundedI64::new_const(
                                        signed_header.commit.round.inner().into(),
                                    )
                                    .expect("0..=i32::MAX can be converted to 0..=i64::MAX safely"),
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
                    )),
                ]))
            }
            ModuleCall::FetchProveRequest(FetchProveRequest {
                update_from,
                request,
            }) => {
                debug!("submitting prove request");

                let prover_endpoint = &self.prover_endpoints[usize::try_from(
                    request.untrusted_header.height.inner(),
                )
                .expect("never going to happen bro")
                    % self.prover_endpoints.len()];

                let response = galois_rpc::Client::connect(prover_endpoint)
                    .await
                    .unwrap()
                    .poll(PollRequest {
                        request: request.clone(),
                    })
                    .await;

                debug!("submitted prove request");

                let retry = || {
                    debug!("proof pending");

                    seq([
                        // REVIEW: How long should we wait between polls?
                        defer(now() + 1),
                        call(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(FetchProveRequest {
                                update_from,
                                request: request.clone(),
                            }),
                        )),
                    ])
                };
                match response {
                    Ok(PollResponse::Pending) => Ok(retry()),
                    Err(status) if status.message() == "busy_building" => Ok(retry()),
                    Err(err) => panic!("prove request failed: {:?}", err),
                    Ok(PollResponse::Failed(ProveRequestFailed { message })) => {
                        error!(%message, "prove request failed");

                        Err(ErrorObject::owned(
                            FATAL_JSONRPC_ERROR_CODE,
                            format!("prove request failed: {message}"),
                            None::<()>,
                        ))
                    }
                    Ok(PollResponse::Done(ProveRequestDone { response })) => {
                        info!(prover = %prover_endpoint, "proof generated");

                        Ok(data(PluginMessage::new(
                            self.plugin_name(),
                            ModuleData::from(ProveResponse {
                                prove_response: response,
                                update_from,
                                header: request.untrusted_header,
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
                data.into_iter().map(|x| {
                    x.as_plugin::<ModuleData>(self.plugin_name())
                        .unwrap()
                        .try_into()
                        .unwrap()
                }),
            ),
        })
    }
}

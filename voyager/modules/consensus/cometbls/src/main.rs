use std::{
    collections::VecDeque,
    fmt::Debug,
    num::{NonZeroU64, ParseIntError},
};

use jsonrpsee::core::{async_trait, RpcResult};
use protos::union::galois::api::v3::union_prover_api_client;
use queue_msg::{call, data, defer, now, promise, seq, void, BoxDynError, Op};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, error, info, instrument};
use unionlabs::{
    ibc::{
        core::{client::height::Height, commitment::merkle_root::MerkleRoot},
        lightclients::cometbls::{client_state::ClientState, consensus_state::ConsensusState},
    },
    traits::Member,
    union::galois::{
        poll_request::PollRequest,
        poll_response::{PollResponse, ProveRequestDone, ProveRequestFailed},
    },
};
use voyager_message::{
    call::{Call, WaitForHeight},
    callback::Callback,
    data::Data,
    module::{ConsensusModuleInfo, ConsensusModuleServer, ModuleInfo, QueueInteractionsServer},
    run_module_server, ChainId, ClientType, DefaultCmd, ModuleContext, ModuleServer,
    VoyagerMessage,
};

use crate::{
    call::{
        FetchProveRequest, FetchTrustedValidators, FetchUntrustedCommit, FetchUntrustedValidators,
        ModuleCall,
    },
    callback::{AggregateProveRequest, ModuleCallback},
    data::{ModuleData, ProveResponse, TrustedValidators, UntrustedCommit, UntrustedValidators},
};

pub mod call;
pub mod callback;
pub mod data;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_module_server::<Module, _, _, _>().await
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

impl ModuleContext for Module {
    type Config = Config;
    type Cmd = DefaultCmd;
    type Info = ConsensusModuleInfo;

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

    fn info(config: Self::Config) -> ModuleInfo<Self::Info> {
        ModuleInfo {
            name: plugin_name(&config.chain_id),
            kind: ConsensusModuleInfo {
                chain_id: config.chain_id,
                client_type: ClientType::new(ClientType::COMETBLS),
            },
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
impl QueueInteractionsServer<ModuleData, ModuleCall, ModuleCallback> for ModuleServer<Module> {
    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn call(
        &self,
        msg: ModuleCall,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match msg {
            ModuleCall::FetchUntrustedCommit(FetchUntrustedCommit { height }) => {
                let commit = self
                    .ctx
                    .tm_client
                    .commit(Some(height.revision_height.try_into().unwrap()))
                    .await
                    .unwrap();

                Ok(data(Data::plugin(
                    self.ctx.plugin_name(),
                    UntrustedCommit {
                        height,
                        signed_header: commit.signed_header,
                    },
                )))
            }
            ModuleCall::FetchTrustedValidators(FetchTrustedValidators { height }) => {
                let validators = self
                    .ctx
                    .tm_client
                    .all_validators(Some(height.revision_height.try_into().unwrap()))
                    .await
                    .unwrap()
                    .validators;

                Ok(data(Data::plugin(
                    self.ctx.plugin_name(),
                    TrustedValidators { height, validators },
                )))
            }
            ModuleCall::FetchUntrustedValidators(FetchUntrustedValidators { height }) => {
                let validators = self
                    .ctx
                    .tm_client
                    .all_validators(Some(height.revision_height.try_into().unwrap()))
                    .await
                    .unwrap()
                    .validators;

                Ok(data(Data::plugin(
                    self.ctx.plugin_name(),
                    UntrustedValidators { height, validators },
                )))
            }
            ModuleCall::FetchProveRequest(FetchProveRequest { request }) => {
                debug!("submitting prove request");

                let prover_endpoint = &self.ctx.prover_endpoints[usize::try_from(
                    request.untrusted_header.height.inner(),
                )
                .expect("never going to happen bro")
                    % self.ctx.prover_endpoints.len()];

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
                        call(Call::plugin(
                            self.ctx.plugin_name(),
                            FetchProveRequest { request },
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

                        Ok(data(Data::plugin(
                            self.ctx.plugin_name(),
                            ProveResponse {
                                prove_response: response,
                            },
                        )))
                    }
                }
            }
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn callback(
        &self,
        callback: ModuleCallback,
        data: VecDeque<Data<ModuleData>>,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        Ok(match callback {
            ModuleCallback::AggregateProveRequest(aggregate) => {
                queue_msg::aggregation::do_callback(aggregate, data)
            }
            ModuleCallback::AggregateHeader(aggregate) => {
                queue_msg::aggregation::do_callback(aggregate, data)
            }
        })
    }
}

#[async_trait]
impl ConsensusModuleServer<ModuleData, ModuleCall, ModuleCallback> for ModuleServer<Module> {
    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn consensus_info(&self) -> RpcResult<ConsensusModuleInfo> {
        Ok(ConsensusModuleInfo {
            chain_id: self.ctx.chain_id.clone(),
            client_type: ClientType::new(ClientType::COMETBLS),
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn self_client_state(&self, height: Height) -> RpcResult<Value> {
        let params = protos::cosmos::staking::v1beta1::query_client::QueryClient::connect(
            self.ctx.grpc_url.clone(),
        )
        .await
        .unwrap()
        .params(protos::cosmos::staking::v1beta1::QueryParamsRequest {})
        .await
        .unwrap()
        .into_inner()
        .params
        .unwrap();

        let commit = self
            .ctx
            .tm_client
            .commit(Some(NonZeroU64::new(height.revision_height).unwrap()))
            .await
            .unwrap();

        let height = commit.signed_header.header.height;

        // Expected to be nanos
        let unbonding_period =
            u64::try_from(params.unbonding_time.clone().unwrap().seconds).unwrap() * 1_000_000_000;

        Ok(serde_json::to_value(ClientState {
            chain_id: self.ctx.chain_id.to_string(),
            trusting_period: unbonding_period * 85 / 100,
            unbonding_period,
            max_clock_drift: (60 * 20) * 1_000_000_000,
            frozen_height: Height {
                revision_number: 0,
                revision_height: 0,
            },
            latest_height: Height {
                revision_number: self
                    .ctx
                    .chain_id
                    .as_str()
                    .split('-')
                    .last()
                    .unwrap()
                    .parse()
                    .unwrap(),
                revision_height: height.inner().try_into().expect("value is >= 0; qed;"),
            },
        })
        .unwrap())
    }

    /// The consensus state on this chain at the specified `Height`.
    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn self_consensus_state(&self, height: Height) -> RpcResult<Value> {
        let commit = self
            .ctx
            .tm_client
            .commit(Some(NonZeroU64::new(height.revision_height).unwrap()))
            .await
            .unwrap();

        Ok(serde_json::to_value(ConsensusState {
            timestamp: commit.signed_header.header.time.as_unix_nanos(),
            app_hash: MerkleRoot {
                hash: commit.signed_header.header.app_hash,
            },
            next_validators_hash: commit.signed_header.header.next_validators_hash,
        })
        .unwrap())
    }

    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn fetch_update_headers(
        &self,
        update_from: Height,
        update_to: Height,
        _counterparty_chain_id: ChainId<'static>,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        Ok(seq([
            void(call(WaitForHeight {
                chain_id: self.ctx.chain_id.clone(),
                height: update_to,
            })),
            promise(
                [
                    call(Call::plugin(
                        self.ctx.plugin_name(),
                        FetchUntrustedCommit { height: update_to },
                    )),
                    call(Call::plugin(
                        self.ctx.plugin_name(),
                        FetchUntrustedValidators { height: update_to },
                    )),
                    call(Call::plugin(
                        self.ctx.plugin_name(),
                        FetchTrustedValidators {
                            height: update_from.increment(),
                        },
                    )),
                ],
                [],
                Callback::plugin(
                    self.ctx.plugin_name(),
                    AggregateProveRequest {
                        chain_id: self.ctx.chain_id.clone(),
                        update_from,
                        update_to,
                    },
                ),
            ),
        ]))
    }
}

use std::{
    collections::VecDeque,
    fmt::Debug,
    num::{NonZeroU64, ParseIntError},
};

use jsonrpsee::core::{async_trait, RpcResult};
use protos::union::galois::api::v3::union_prover_api_client;
use queue_msg::{call, data, defer_relative, promise, seq, void, Op};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, error, info, instrument, warn};
use unionlabs::{
    ibc::{
        core::{
            client::height::{Height, IsHeight},
            commitment::merkle_root::MerkleRoot,
        },
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
    plugin::{
        ConsensusModuleInfo, ConsensusModuleServer, PluginInfo, PluginKind, PluginModuleServer,
    },
    run_module_server, ClientType, VoyagerMessage,
};

use crate::{
    aggregate::{AggregateProveRequest, ModuleAggregate},
    data::{ModuleData, ProveResponse, TrustedValidators, UntrustedCommit, UntrustedValidators},
    fetch::{
        FetchProveRequest, FetchTrustedValidators, FetchUntrustedCommit, FetchUntrustedValidators,
        ModuleFetch,
    },
};

pub mod aggregate;
pub mod data;
pub mod fetch;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    run_module_server(Module::new, ConsensusModuleServer::into_rpc).await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: String,

    pub tm_client: cometbft_rpc::Client,
    pub chain_revision: u64,
    pub grpc_url: String,

    pub prover_endpoints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ws_url: String,
    pub grpc_url: String,

    pub prover_endpoints: Vec<String>,
}

impl Module {
    fn plugin_name(&self) -> String {
        pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

        format!("{PLUGIN_NAME}/{}", self.chain_id)
    }

    pub async fn new(config: Config) -> Result<Self, UnionInitError> {
        let tm_client = cometbft_rpc::Client::new(config.ws_url).await?;

        let chain_id = tm_client.status().await?.node_info.network.to_string();

        let chain_revision = chain_id
            .split('-')
            .last()
            .ok_or_else(|| UnionInitError::ChainIdParse {
                found: chain_id.clone(),
                source: None,
            })?
            .parse()
            .map_err(|err| UnionInitError::ChainIdParse {
                found: chain_id.clone(),
                source: Some(err),
            })?;

        Ok(Self {
            tm_client,
            chain_id,
            chain_revision,
            prover_endpoints: config.prover_endpoints,
            grpc_url: config.grpc_url,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UnionInitError {
    #[error("tendermint rpc error")]
    Cometbft(#[from] cometbft_rpc::JsonRpcError),
    #[error(
        "unable to parse chain id: expected format \
        `<chain>-<revision-number>`, found `{found}`"
    )]
    // TODO: Once the `Id` trait in unionlabs is cleaned up to no longer use static id types, this error should just wrap `IdParseError`
    ChainIdParse {
        found: String,
        #[source]
        source: Option<ParseIntError>,
    },
}

type D = ModuleData;
type F = ModuleFetch;
type A = ModuleAggregate;

#[async_trait]
impl PluginModuleServer<D, F, A> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn info(&self) -> RpcResult<PluginInfo> {
        Ok(PluginInfo {
            name: self.plugin_name(),
            kind: Some(PluginKind::Consensus),
            interest_filter: None,
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn handle_fetch(&self, msg: ModuleFetch) -> RpcResult<Op<VoyagerMessage<D, F, A>>> {
        match msg {
            ModuleFetch::FetchUntrustedCommit(FetchUntrustedCommit { height }) => {
                let commit = self
                    .tm_client
                    .commit(Some(height.revision_height.try_into().unwrap()))
                    .await
                    .unwrap();

                Ok(data(Data::plugin(
                    self.plugin_name(),
                    UntrustedCommit {
                        height,
                        signed_header: commit.signed_header,
                    },
                )))
            }
            ModuleFetch::FetchTrustedValidators(FetchTrustedValidators { height }) => {
                let validators = self
                    .tm_client
                    .all_validators(Some(height.revision_height.try_into().unwrap()))
                    .await
                    .unwrap()
                    .validators;

                Ok(data(Data::plugin(
                    self.plugin_name(),
                    TrustedValidators { height, validators },
                )))
            }
            ModuleFetch::FetchUntrustedValidators(FetchUntrustedValidators { height }) => {
                let validators = self
                    .tm_client
                    .all_validators(Some(height.revision_height.try_into().unwrap()))
                    .await
                    .unwrap()
                    .validators;

                Ok(data(Data::plugin(
                    self.plugin_name(),
                    UntrustedValidators { height, validators },
                )))
            }
            ModuleFetch::FetchProveRequest(FetchProveRequest { request }) => {
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
                        defer_relative(1),
                        call(Call::plugin(
                            self.plugin_name(),
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
                            self.plugin_name(),
                            ProveResponse {
                                prove_response: response,
                            },
                        )))
                    }
                }
            }
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    fn handle_aggregate(
        &self,
        aggregate: A,
        data: VecDeque<Data<D>>,
    ) -> RpcResult<Op<VoyagerMessage<D, F, A>>> {
        Ok(match aggregate {
            ModuleAggregate::AggregateProveRequest(aggregate) => {
                queue_msg::aggregation::do_aggregate(aggregate, data)
            }
            ModuleAggregate::AggregateHeader(aggregate) => {
                queue_msg::aggregation::do_aggregate(aggregate, data)
            }
        })
    }
}

#[async_trait]
impl ConsensusModuleServer<D, F, A> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn consensus_info(&self) -> RpcResult<ConsensusModuleInfo> {
        Ok(ConsensusModuleInfo {
            chain_id: self.chain_id.clone(),
            client_type: ClientType::new(ClientType::COMETBLS),
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn self_client_state(&self, height: Height) -> RpcResult<Value> {
        let params = protos::cosmos::staking::v1beta1::query_client::QueryClient::connect(
            self.grpc_url.clone(),
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
            .tm_client
            .commit(Some(NonZeroU64::new(height.revision_height).unwrap()))
            .await
            .unwrap();

        let height = commit.signed_header.header.height;

        // Expected to be nanos
        let unbonding_period =
            u64::try_from(params.unbonding_time.clone().unwrap().seconds).unwrap() * 1_000_000_000;

        Ok(serde_json::to_value(ClientState {
            chain_id: self.chain_id.clone(),
            trusting_period: unbonding_period * 85 / 100,
            unbonding_period,
            max_clock_drift: (60 * 20) * 1_000_000_000,
            frozen_height: Height {
                revision_number: 0,
                revision_height: 0,
            },
            latest_height: Height {
                revision_number: self.chain_id.split('-').last().unwrap().parse().unwrap(),
                revision_height: height.inner().try_into().expect("value is >= 0; qed;"),
            },
        })
        .unwrap())
    }

    /// The consensus state on this chain at the specified `Height`.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn self_consensus_state(&self, height: Height) -> RpcResult<Value> {
        let commit = self
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

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    fn fetch_update_headers(
        &self,
        update_from: Height,
        update_to: Height,
    ) -> RpcResult<Op<VoyagerMessage<D, F, A>>> {
        Ok(seq([
            void(call(WaitForHeight {
                chain_id: self.chain_id.clone(),
                height: update_to,
            })),
            promise(
                [
                    call(Call::plugin(
                        self.plugin_name(),
                        FetchUntrustedCommit { height: update_to },
                    )),
                    call(Call::plugin(
                        self.plugin_name(),
                        FetchUntrustedValidators { height: update_to },
                    )),
                    call(Call::plugin(
                        self.plugin_name(),
                        FetchTrustedValidators {
                            height: update_from.increment(),
                        },
                    )),
                ],
                [],
                Callback::plugin(
                    self.plugin_name(),
                    AggregateProveRequest {
                        chain_id: self.chain_id.clone(),
                        update_from,
                        update_to,
                    },
                ),
            ),
        ]))
    }
}

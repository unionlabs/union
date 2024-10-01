use std::{collections::VecDeque, num::ParseIntError};

use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use protos::union::galois::api::v3::union_prover_api_client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, instrument};
use unionlabs::union::galois::{
    poll_request::PollRequest,
    poll_response::{PollResponse, ProveRequestDone, ProveRequestFailed},
};
use voyager_message::{
    call::{Call, WaitForHeight},
    callback::Callback,
    core::ChainId,
    data::Data,
    module::{ModuleInfo, PluginInfo, PluginServer, PluginTypes},
    run_module_server, DefaultCmd, ModuleContext, VoyagerMessage,
};
use voyager_vm::{
    call, data, defer, now, optimize::OptimizationResult, promise, seq, void, BoxDynError, Op,
};

use crate::{
    call::{
        FetchProveRequest, FetchTrustedValidators, FetchUntrustedCommit, FetchUntrustedValidators,
        FetchUpdate, ModuleCall,
    },
    callback::{AggregateProveRequest, ModuleCallback},
    data::{ModuleData, ProveResponse, TrustedValidators, UntrustedCommit, UntrustedValidators},
};

pub mod call;
pub mod callback;
pub mod data;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_module_server::<Module>().await
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
    type Info = PluginInfo;

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
            kind: PluginInfo {
                name: plugin_name(&config.chain_id),
                interest_filter: format!(
                    r#"[.. | ."@type"? == "fetch_update_headers" and ."@value".chain_id == "{}"] | any"#,
                    config.chain_id
                ),
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

impl PluginTypes for Module {
    type D = ModuleData;
    type C = ModuleCall;
    type Cb = ModuleCallback;
}

#[async_trait]
impl PluginServer<ModuleData, ModuleCall, ModuleCallback> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn run_pass(
        &self,
        _: &Extensions,
        msgs: Vec<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>>,
    ) -> RpcResult<OptimizationResult<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        Ok(OptimizationResult {
            optimize_further: vec![],
            ready: msgs
                .into_iter()
                .map(|op| match op {
                    Op::Call(Call::FetchUpdateHeaders(fetch))
                        if fetch.chain_id == self.chain_id =>
                    {
                        call(Call::plugin(
                            self.plugin_name(),
                            FetchUpdate {
                                update_from: fetch.update_from,
                                update_to: fetch.update_to,
                            },
                        ))
                    }
                    op => op,
                })
                .enumerate()
                .map(|(i, op)| (vec![i], op))
                .collect(),
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn call(
        &self,
        _: &Extensions,
        msg: ModuleCall,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match msg {
            ModuleCall::FetchUpdate(FetchUpdate {
                update_from,
                update_to,
            }) => Ok(seq([
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
            ])),
            ModuleCall::FetchUntrustedCommit(FetchUntrustedCommit { height }) => {
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
            ModuleCall::FetchTrustedValidators(FetchTrustedValidators { height }) => {
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
            ModuleCall::FetchUntrustedValidators(FetchUntrustedValidators { height }) => {
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
    async fn callback(
        &self,
        _: &Extensions,
        callback: ModuleCallback,
        data: VecDeque<Data<ModuleData>>,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        Ok(match callback {
            ModuleCallback::AggregateProveRequest(aggregate) => {
                voyager_vm::aggregation::do_callback(aggregate, data)
            }
            ModuleCallback::AggregateHeader(aggregate) => {
                voyager_vm::aggregation::do_callback(aggregate, data)
            }
        })
    }
}

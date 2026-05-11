#![warn(clippy::unwrap_used)]

use std::collections::VecDeque;

use gno_light_client_types::Header;
use gno_types::{Validator, ValidatorSet};
use jsonrpsee::{Extensions, core::async_trait};
use serde::{Deserialize, Serialize};
use tracing::{instrument, warn};
use unionlabs::{
    bounded::BoundedI64,
    never::Never,
    primitives::{Bech32, H160},
};
use voyager_sdk::{
    DefaultCmd,
    anyhow::{self, bail},
    hook::UpdateHook,
    into_value,
    message::{
        PluginMessage, VoyagerMessage,
        call::Call,
        data::{Data, DecodedHeaderMeta, OrderedHeaders},
    },
    plugin::Plugin,
    primitives::{ChainId, ClientType},
    rpc::{PluginServer, RpcError, RpcResult, types::PluginInfo},
    vm::{Op, Visit, data, pass::PassResult},
};

use crate::call::{FetchUpdate, ModuleCall};

pub mod call;

#[tokio::main]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub gno_client: gno_rpc::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId,

    pub rpc_url: String,
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = Never;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> anyhow::Result<Self> {
        let gno_client = gno_rpc::Client::new(config.rpc_url).await?;

        let chain_id = gno_client.status(None).await?.node_info.network.to_string();

        if chain_id != config.chain_id.as_str() {
            bail!(
                "incorrect chain id: expected `{}`, but found `{}`",
                config.chain_id,
                chain_id
            );
        }

        Ok(Self {
            gno_client,
            chain_id: ChainId::new(chain_id),
        })
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.chain_id),
            interest_filter: UpdateHook::filter(
                &config.chain_id,
                &ClientType::new(ClientType::GNO),
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
                    UpdateHook::new(&self.chain_id, &ClientType::new(ClientType::GNO), |fetch| {
                        Call::Plugin(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(FetchUpdate {
                                update_from: fetch.update_from,
                                update_to: fetch.update_to,
                            }),
                        ))
                    })
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
                let trusted_height: BoundedI64<0> = (update_from.increment().height() as i64)
                    .try_into()
                    .expect("valid height");
                let untrusted_height: BoundedI64<0> = (update_to.height() as i64)
                    .try_into()
                    .expect("valid height");

                let trusted_commit = self
                    .gno_client
                    .commit(trusted_height)
                    .await
                    .map_err(RpcError::retryable("trusted commit"))?;

                let untrusted_commit = self
                    .gno_client
                    .commit(untrusted_height)
                    .await
                    .map_err(RpcError::retryable("untrusted commit"))?;

                let trusted_validators = self
                    .gno_client
                    .validators(trusted_height)
                    .await
                    .map_err(RpcError::retryable("trusted validators"))?;

                let untrusted_validators = self
                    .gno_client
                    .validators(untrusted_height)
                    .await
                    .map_err(RpcError::retryable("untrusted validators"))?;

                let header = Header {
                    validator_set: mk_validator_set(
                        untrusted_validators.validators,
                        untrusted_commit
                            .signed_header
                            .header
                            .proposer_address
                            .clone(),
                    ),
                    signed_header: untrusted_commit.signed_header,
                    trusted_height: update_from,
                    trusted_validators: mk_validator_set(
                        trusted_validators.validators,
                        trusted_commit.signed_header.header.proposer_address,
                    ),
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

fn mk_validator_set(validators: Vec<Validator>, proposer_address: Bech32<H160>) -> ValidatorSet {
    let proposer = validators
        .iter()
        .find(|val| val.address == proposer_address)
        .expect("proposer must exist in set")
        .clone();

    ValidatorSet {
        validators,
        proposer,
    }
}

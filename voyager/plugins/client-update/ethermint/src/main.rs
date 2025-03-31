use std::{collections::VecDeque, fmt::Debug, num::ParseIntError};

use cometbft_types::types::{validator::Validator, validator_set::ValidatorSet};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use serde::{Deserialize, Serialize};
use tendermint_light_client_types::Header;
use tracing::instrument;
use unionlabs::{
    ibc::core::client::height::Height,
    primitives::{encoding::HexUnprefixed, H160},
};
use voyager_message::{
    call::Call,
    core::{ChainId, ClientType},
    data::{Data, DecodedHeaderMeta, OrderedHeaders},
    hook::UpdateHook,
    module::{PluginInfo, PluginServer},
    DefaultCmd, Plugin, PluginMessage, VoyagerMessage,
};
use voyager_vm::{data, pass::PassResult, BoxDynError, Op, Visit};

use crate::{
    call::{FetchUpdate, ModuleCall},
    callback::ModuleCallback,
};

pub mod call;
pub mod callback;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub cometbft_client: cometbft_rpc::Client,
    pub chain_revision: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId,

    pub rpc_url: String,
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        let tm_client = cometbft_rpc::Client::new(config.rpc_url).await?;

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

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.chain_id),
            interest_filter: UpdateHook::filter(
                &config.chain_id,
                &ClientType::new(ClientType::ETHERMINT),
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
                        &ClientType::new(ClientType::ETHERMINT),
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
                let trusted_commit = self
                    .cometbft_client
                    .commit(Some(update_from.increment().height().try_into().unwrap()))
                    .await
                    .unwrap();

                let untrusted_commit = self
                    .cometbft_client
                    .commit(Some(update_to.height().try_into().unwrap()))
                    .await
                    .unwrap();

                let trusted_validators = self
                    .cometbft_client
                    .all_validators(Some(update_from.increment().height().try_into().unwrap()))
                    .await
                    .unwrap();

                let untrusted_validators = self
                    .cometbft_client
                    .all_validators(Some(update_to.height().try_into().unwrap()))
                    .await
                    .unwrap();

                let header = Header {
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
                };

                Ok(data(OrderedHeaders {
                    headers: vec![(
                        DecodedHeaderMeta { height: update_to },
                        serde_json::to_value(header).unwrap(),
                    )],
                }))
            }
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn callback(
        &self,
        _: &Extensions,
        callback: ModuleCallback,
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
        .unwrap()
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

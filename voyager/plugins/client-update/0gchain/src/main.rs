use std::{collections::VecDeque, fmt::Debug, num::ParseIntError};

use alloy::{
    providers::{Provider, ProviderBuilder, RootProvider},
    transports::BoxTransport,
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use unionlabs::primitives::H160;
use voyager_message::{
    call::{Call, FetchUpdateHeaders, WaitForTrustedHeight},
    core::{ChainId, ClientType, IbcSpecId},
    data::Data,
    hook::UpdateHook,
    module::{PluginInfo, PluginServer},
    DefaultCmd, Plugin, PluginMessage, RawClientId, VoyagerMessage,
};
use voyager_vm::{call, conc, pass::PassResult, seq, BoxDynError, Op, Visit};

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
    pub l1_client_id: u32,
    pub l1_chain_id: ChainId,
    pub l2_chain_id: ChainId,
    pub ibc_handler_address: H160,
    pub eth_provider: RootProvider<BoxTransport>,
    pub cometbft_client: cometbft_rpc::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub l1_client_id: u32,
    pub l1_chain_id: ChainId,
    pub l2_chain_id: ChainId,
    pub ibc_handler_address: H160,
    pub comet_rpc_url: String,
    pub eth_rpc_url: String,
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        let eth_provider = ProviderBuilder::new()
            .on_builtin(&config.eth_rpc_url)
            .await?;

        let chain_id = ChainId::new(eth_provider.get_chain_id().await?.to_string());

        if chain_id != config.l2_chain_id {
            return Err(format!(
                "incorrect chain id: expected `{}`, but found `{}`",
                config.l2_chain_id, chain_id
            )
            .into());
        }

        let tm_client = cometbft_rpc::Client::new(config.comet_rpc_url).await?;

        Ok(Self {
            l1_client_id: config.l1_client_id,
            l2_chain_id: config.l2_chain_id,
            l1_chain_id: config.l1_chain_id,
            ibc_handler_address: config.ibc_handler_address,
            eth_provider,
            cometbft_client: tm_client,
        })
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.l2_chain_id),
            interest_filter: UpdateHook::filter(
                &config.l2_chain_id,
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
        plugin_name(&self.l2_chain_id)
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
    #[instrument(skip_all, fields(chain_id = %self.l2_chain_id))]
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
                        &self.l2_chain_id,
                        &ClientType::new(ClientType::ETHERMINT),
                        |fetch| {
                            Call::Plugin(PluginMessage::new(
                                self.plugin_name(),
                                ModuleCall::from(FetchUpdate {
                                    counterparty_chain_id: fetch.counterparty_chain_id.clone(),
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

    #[instrument(skip_all, fields(chain_id = %self.l2_chain_id))]
    async fn call(&self, _: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::FetchUpdate(FetchUpdate {
                counterparty_chain_id,
                update_from,
                update_to,
            }) => {
                // TODO(aeryz): check whether tm already have the `update_to` state?
                // Recursively dispatch a L1 update before dispatching the L2 update.
                Ok(conc([
                    call(FetchUpdateHeaders {
                        client_type: ClientType::new(ClientType::TENDERMINT),
                        counterparty_chain_id: counterparty_chain_id.clone(),
                        chain_id: self.l1_chain_id.clone(),
                        client_id: RawClientId::new(self.l1_client_id),
                        update_from,
                        update_to,
                    }),
                    seq([call(WaitForTrustedHeight {
                        chain_id: counterparty_chain_id,
                        ibc_spec_id: IbcSpecId::new(IbcSpecId::UNION),
                        // TODO: abstract away the L1 client id and read it from
                        // the L2 client state (l2_client_id) on the
                        // `counterparty_chain_id`
                        client_id: RawClientId::new(self.l1_client_id),
                        height: update_to,
                        finalized: true,
                    })]),
                ]))
            }
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.l2_chain_id))]
    async fn callback(
        &self,
        _: &Extensions,
        callback: ModuleCallback,
        _data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match callback {}
    }
}

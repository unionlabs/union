use std::collections::VecDeque;

use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info, instrument};
use unionlabs::never::Never;
use voyager_message::{
    call::{FetchUpdateHeaders, WaitForTrustedHeight},
    callback::AggregateSubmitTxFromOrderedHeaders,
    data::Data,
    into_value,
    module::{PluginInfo, PluginServer},
    primitives::{ChainId, IbcSpecId, QueryHeight},
    ExtensionsExt, Plugin, PluginMessage, RawClientId, VoyagerClient, VoyagerMessage,
};
use voyager_vm::{call, conc, defer, now, pass::PassResult, promise, seq, BoxDynError, Op};

use crate::call::{CheckForClientAge, ModuleCall};

pub mod call;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

pub struct Module {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = Never;

    type Config = Config;
    type Cmd = Cmd;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        Ok(Module::new(config))
    }

    fn info(config: Self::Config) -> PluginInfo {
        let module = Module::new(config);

        PluginInfo {
            name: module.plugin_name(),
            // never interested in any messages since this plugin does not utilize a queue
            interest_filter: "null".to_owned(),
        }
    }

    async fn cmd(config: Self::Config, cmd: Self::Cmd) {
        let module = Self::new(config);

        match cmd {
            Cmd::MakeMessage(msg) => {
                let op = call::<VoyagerMessage>(PluginMessage::new(
                    module.plugin_name(),
                    ModuleCall::CheckForClientAge(msg),
                ));

                println!("{}", into_value(op));
            }
        }
    }
}

#[derive(clap::Parser)]
pub enum Cmd {
    MakeMessage(CheckForClientAge),
}

impl Module {
    fn plugin_name(&self) -> String {
        pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

        PLUGIN_NAME.to_owned()
    }

    pub fn new(_: Config) -> Self {
        Self {}
    }

    #[instrument(
        skip_all,
        fields(
            %chain_id,
            %ibc_spec_id,
            client_id = %client_id.as_raw(),
            max_age
        )
    )]
    async fn check_for_client_age(
        &self,
        voyager_client: &VoyagerClient,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        client_id: RawClientId,
        max_age: u64,
    ) -> RpcResult<Op<VoyagerMessage>> {
        let client_state_meta = voyager_client
            .client_state_meta_raw(
                chain_id.clone(),
                ibc_spec_id.clone(),
                QueryHeight::Latest,
                client_id.clone(),
            )
            .await?;

        let client_info = voyager_client
            .client_info_raw(chain_id.clone(), ibc_spec_id.clone(), client_id.clone())
            .await?;

        let latest_finalized_height = voyager_client
            .query_latest_height(client_state_meta.counterparty_chain_id.clone(), true)
            .await?;

        if client_state_meta.counterparty_height.height() + max_age
            < latest_finalized_height.height()
        {
            info!("client is older than threshold");

            Ok(conc([
                promise(
                    [call(FetchUpdateHeaders {
                        client_type: client_info.client_type,
                        chain_id: client_state_meta.counterparty_chain_id,
                        counterparty_chain_id: chain_id.clone(),
                        client_id: client_id.clone(),
                        update_from: client_state_meta.counterparty_height,
                        update_to: latest_finalized_height,
                    })],
                    [],
                    AggregateSubmitTxFromOrderedHeaders {
                        ibc_spec_id: ibc_spec_id.clone(),
                        chain_id: chain_id.clone(),
                        client_id: client_id.clone(),
                    },
                ),
                seq([
                    call(WaitForTrustedHeight {
                        chain_id: chain_id.clone(),
                        ibc_spec_id: ibc_spec_id.clone(),
                        client_id: client_id.clone(),
                        height: client_state_meta.counterparty_height.increment_by(max_age),
                        finalized: false,
                    }),
                    call(PluginMessage::new(
                        self.plugin_name(),
                        ModuleCall::CheckForClientAge(CheckForClientAge {
                            chain_id,
                            ibc_spec_id,
                            client_id,
                            max_age,
                        }),
                    )),
                ]),
            ]))
        } else {
            Ok(seq([
                defer(now() + 60),
                call(PluginMessage::new(
                    self.plugin_name(),
                    ModuleCall::CheckForClientAge(CheckForClientAge {
                        chain_id,
                        ibc_spec_id,
                        client_id,
                        max_age,
                    }),
                )),
            ]))
        }
    }
}

#[async_trait]
impl PluginServer<ModuleCall, Never> for Module {
    async fn run_pass(
        &self,
        _: &Extensions,
        msgs: Vec<Op<VoyagerMessage>>,
    ) -> RpcResult<PassResult<VoyagerMessage>> {
        error!(?msgs, "this plugin does not utilize a queue");

        Ok(PassResult::default())
    }

    async fn call(&self, e: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::CheckForClientAge(CheckForClientAge {
                chain_id,
                ibc_spec_id,
                client_id,
                max_age,
            }) => {
                self.check_for_client_age(e.try_get()?, chain_id, ibc_spec_id, client_id, max_age)
                    .await
            }
        }
    }

    async fn callback(
        &self,
        _: &Extensions,
        cb: Never,
        _data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match cb {}
    }
}

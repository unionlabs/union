use std::{collections::VecDeque, ops::Deref, panic::AssertUnwindSafe, path::PathBuf, sync::Arc};

use alloy::{
    network::AnyNetwork,
    providers::{DynProvider, Provider, ProviderBuilder},
};
use clap::Subcommand;
use ibc_solidity::Ibc::{self, IbcInstance};
use ibc_union_spec::{
    IbcUnion,
    event::FullEvent,
    path::{ConnectionPath, StorePath},
};
use jsonrpsee::{
    Extensions,
    core::{RpcResult, async_trait},
    types::ErrorObject,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::instrument;
use unionlabs::{
    ErrorReporter,
    never::Never,
    primitives::{H160, H256},
};
use voyager_sdk::{
    anyhow::{self, bail},
    hook::{SubmitTxHook, simple_take_filter},
    message::{PluginMessage, VoyagerMessage, data::Data},
    plugin::Plugin,
    primitives::{ChainId, IbcSpec},
    rpc::{FATAL_JSONRPC_ERROR_CODE, PluginServer, types::PluginInfo},
    vm::{Op, Visit, call, defer, now, pass::PassResult, seq},
};

use crate::call::{ModuleCall, SubmitAttestation};

pub mod call;

#[tokio::main]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module(Arc<ModuleInner>);

impl Deref for Module {
    type Target = ModuleInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct ModuleInner {
    pub chain_id: ChainId,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    pub provider: DynProvider<AnyNetwork>,

    pub attestation_key: (),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId,
    #[serde(default)]
    pub additional_chain_ids: Vec<ChainId>,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    /// The RPC endpoint for the execution chain.
    pub rpc_url: String,

    pub attestation_key_path: PathBuf,
}

#[derive(Subcommand)]
pub enum Cmd {}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = Never;

    type Config = Config;
    type Cmd = Cmd;

    async fn new(config: Self::Config) -> anyhow::Result<Self> {
        let provider = DynProvider::new(
            ProviderBuilder::new()
                .network::<AnyNetwork>()
                .connect(&config.rpc_url)
                .await?,
        );

        let raw_chain_id = provider.get_chain_id().await?;
        let chain_id = ChainId::new(raw_chain_id.to_string());

        if chain_id != config.chain_id {
            bail!(
                "incorrect chain id: expected `{}`, but found `{}`",
                config.chain_id,
                chain_id
            );
        }

        Ok(Self(Arc::new(ModuleInner {
            chain_id,
            ibc_handler_address: config.ibc_handler_address,
            provider,
            attestation_key: (),
        })))
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.chain_id),
            interest_filter: simple_take_filter(format!(
                r#"
if ."@type" == "data" then
    ."@value" as $data |

    # pull all ibc events from this chain, they will be verified and attested to

    $data."@type" == "ibc_event" and $data."@value".chain_id == "{chain_id}" and $data."@value".ibc_spec_id == "{ibc_union_id}"
else
    false
end
"#,
                chain_id = config.chain_id,
                ibc_union_id = IbcUnion::ID,
            )),
        }
    }

    async fn cmd(config: Self::Config, cmd: Self::Cmd) {
        let plugin = Self::new(config).await.unwrap();

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
    async fn run_pass(
        &self,
        _: &Extensions,
        msgs: Vec<Op<VoyagerMessage>>,
    ) -> RpcResult<PassResult<VoyagerMessage>> {
        Ok(PassResult {
            optimize_further: vec![],
            ready: msgs
                .into_iter()
                .enumerate()
                .map(|(idx, mut op)| {
                    let op = match op.into_data().unwrap() {
                        Data::IbcEvent(chain_event) => call(PluginMessage::new(
                            plugin_name(&self.chain_id),
                            ModuleCall::SubmitAttestation(SubmitAttestation {
                                event: chain_event.decode_event::<IbcUnion>().unwrap().unwrap(),
                                tx_hash: chain_event.tx_hash,
                                height: chain_event.provable_height.height().height(),
                            }),
                        )),
                        _ => todo!(),
                    };

                    (vec![idx], op)
                })
                .collect(),
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn call(&self, _: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::SubmitAttestation(SubmitAttestation {
                event,
                tx_hash,
                height,
            }) => {
                self.submit_attestation(event, tx_hash, height).await?;

                Ok(noop())
            }
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn callback(
        &self,
        _: &Extensions,
        cb: Never,
        _data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match cb {}
    }
}

impl Module {
    fn ibc_handler(&self) -> IbcInstance<DynProvider<AnyNetwork>, AnyNetwork> {
        Ibc::new::<_, AnyNetwork>(self.ibc_handler_address.get().into(), self.provider.clone())
    }

    async fn submit_attestation(
        &self,
        event: FullEvent,
        tx_hash: H256,
        height: u64,
    ) -> RpcResult<()> {
        let (k, v) = match event {
            FullEvent::CreateClient(event) => return Ok(()),
            FullEvent::UpdateClient(event) => return Ok(()),
            FullEvent::ConnectionOpenInit(event) => {
                // ConnectionPath {
                //     connection_id: event.connection_id,
                // }
                // .key()

                let connection = self
                    .ibc_handler()
                    .connections(event.connection_id.raw())
                    .call()
                    .await?;
            }
            FullEvent::ConnectionOpenTry(event) => todo!(),
            FullEvent::ConnectionOpenAck(event) => todo!(),
            FullEvent::ConnectionOpenConfirm(event) => todo!(),
            FullEvent::ChannelOpenInit(event) => todo!(),
            FullEvent::ChannelOpenTry(event) => todo!(),
            FullEvent::ChannelOpenAck(event) => todo!(),
            FullEvent::ChannelOpenConfirm(event) => todo!(),
            FullEvent::ChannelCloseInit(event) => todo!(),
            FullEvent::ChannelCloseConfirm(event) => todo!(),
            FullEvent::PacketSend(event) => todo!(),
            FullEvent::BatchSend(event) => todo!(),
            FullEvent::PacketRecv(event) => todo!(),
            FullEvent::IntentPacketRecv(event) => todo!(),
            FullEvent::WriteAck(event) => todo!(),
            FullEvent::PacketAck(event) => todo!(),
            FullEvent::PacketTimeout(event) => todo!(),
        };

        let tx = self
            .provider
            .get_transaction_by_hash(tx_hash.get().into())
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    ErrorReporter(e).with_message("error fetching source transaction"),
                    None::<()>,
                )
            })?
            .ok_or_else(|| ErrorObject::owned(-1, format!("tx {tx_hash} not found"), None::<()>))?;

        let tx_height = tx.block_number.unwrap();

        if tx_height != height {
            return Err(ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                format!(
                    "block number is inconsistent; event height \
                    is {height} but tx height is {tx_height}"
                ),
                None::<()>,
            ));
        }

        Ok(())
    }
}

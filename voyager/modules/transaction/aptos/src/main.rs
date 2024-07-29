use std::{collections::VecDeque, sync::Arc};

use aptos_crypto::{ed25519::Ed25519PrivateKey, PrivateKey};
use aptos_rest_client::aptos_api_types::{Address, MoveModuleId};
use aptos_types::{
    account_address::AccountAddress,
    transaction::{EntryFunction, RawTransaction},
};
use chain_utils::{
    keyring::{ConcurrentKeyring, KeyringConfig, KeyringEntry},
    BoxDynError,
};
use jsonrpsee::core::{async_trait, RpcResult};
use queue_msg::{call, noop, optimize::OptimizationResult, Op};
use serde::{Deserialize, Serialize};
use sha3::Digest;
use tracing::instrument;
use unionlabs::hash::H256;
use voyager_message::{
    call::Call,
    data::{Data, WithChainId},
    module::{ModuleInfo, PluginModuleInfo, PluginModuleServer, QueueInteractionsServer},
    run_module_server, ChainId, DefaultCmd, ModuleContext, ModuleServer, VoyagerMessage,
};

use crate::{call::ModuleCall, callback::ModuleCallback, data::ModuleData};

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

    pub ibc_handler_address: Address,

    pub aptos_client: aptos_rest_client::Client,

    pub keyring: ConcurrentKeyring<AccountAddress, Arc<Ed25519PrivateKey>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId<'static>,

    pub rpc_url: String,
    pub ibc_handler_address: Address,

    pub keyring: KeyringConfig,
}

impl ModuleContext for Module {
    type Config = Config;
    type Cmd = DefaultCmd;
    type Info = PluginModuleInfo;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        let aptos_client = aptos_rest_client::Client::new(config.rpc_url.parse().unwrap());

        let chain_id = aptos_client.get_index().await?.inner().chain_id;

        Ok(Self {
            chain_id: ChainId::new(chain_id.to_string()),
            ibc_handler_address: config.ibc_handler_address,
            aptos_client,
            keyring: ConcurrentKeyring::new(
                config.keyring.name,
                config.keyring.keys.into_iter().map(|config| {
                    let pk = aptos_crypto::ed25519::Ed25519PrivateKey::try_from(&*config.value())
                        .unwrap();

                    let address = H256::from(
                        sha3::Sha3_256::new()
                            .chain_update(pk.public_key().to_bytes())
                            .chain_update([0])
                            .finalize(),
                    )
                    .0
                    .into();

                    KeyringEntry {
                        name: config.name(),
                        address,
                        signer: Arc::new(pk),
                    }
                }),
            ),
        })
    }

    fn info(config: Self::Config) -> ModuleInfo<Self::Info> {
        ModuleInfo {
            name: plugin_name(&config.chain_id),
            kind: PluginModuleInfo {
                interest_filter: format!(
                    r#"
if ."@type" == "data" then
    ."@value" as $data |

    # pull all transaction data messages
    ($data."@type" == "identified_ibc_message_batch" or $data."@type" == "identified_ibc_message")
        and $data."@value".chain_id == "{chain_id}"
else
    false
end
"#,
                    chain_id = config.chain_id,
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
        pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

        format!("{PLUGIN_NAME}/{}", self.chain_id)
    }
}

#[async_trait]
impl QueueInteractionsServer<ModuleData, ModuleCall, ModuleCallback> for ModuleServer<Module> {
    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn call(
        &self,
        msg: ModuleCall,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match msg {
            ModuleCall::SubmitTransaction(msgs) => {
                self.ctx
                    .keyring
                    .with(|pk| async move {
                        let sender = H256::from(
                            sha3::Sha3_256::new()
                                .chain_update(pk.public_key().to_bytes())
                                .chain_update([0])
                                .finalize(),
                        )
                        .0
                        .into();

                        let account = self
                            .ctx
                            .aptos_client
                            .get_account(sender)
                            .await
                            .unwrap()
                            .into_inner();

                        dbg!(&account);

                        let raw = RawTransaction::new_entry_function(
                            sender,
                            account.sequence_number,
                            EntryFunction::new(
                                MoveModuleId {
                                    address: self.ctx.ibc_handler_address,
                                    name: "Core".parse().unwrap(),
                                }
                                .into(),
                                "hackerman".parse().unwrap(),
                                vec![],
                                vec![],
                            ),
                            400000,
                            100,
                            queue_msg::now() + 10,
                            self.ctx.chain_id.as_str().parse().unwrap(),
                        );

                        // let hash = raw.test_only_hash()

                        let sig = raw.sign(pk, pk.public_key()).unwrap();

                        dbg!(&sig);

                        let res = self.ctx.aptos_client.submit_and_wait(&sig).await.unwrap();

                        dbg!(&res);

                        Ok(noop())
                    })
                    .await
                    .unwrap_or_else(|| {
                        Ok(call(Call::plugin(
                            self.ctx.plugin_name(),
                            ModuleCall::SubmitTransaction(msgs),
                        )))
                    })
            }
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn callback(
        &self,
        cb: ModuleCallback,
        _data: VecDeque<Data<ModuleData>>,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match cb {}
    }
}

#[async_trait]
impl PluginModuleServer<ModuleData, ModuleCall, ModuleCallback> for ModuleServer<Module> {
    async fn run_pass(
        &self,
        msgs: Vec<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>>,
    ) -> RpcResult<OptimizationResult<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        Ok(OptimizationResult {
            optimize_further: vec![],
            ready: msgs
                .into_iter()
                .enumerate()
                .map(|(idx, msg)| {
                    (
                        vec![idx],
                        match msg {
                            Op::Data(Data::IdentifiedIbcMessage(WithChainId {
                                chain_id,
                                message,
                            })) => {
                                assert_eq!(chain_id, self.ctx.chain_id);

                                call(Call::plugin(
                                    self.ctx.plugin_name(),
                                    ModuleCall::SubmitTransaction(vec![message]),
                                ))
                            }
                            Op::Data(Data::IdentifiedIbcMessageBatch(WithChainId {
                                chain_id,
                                message,
                            })) => {
                                assert_eq!(chain_id, self.ctx.chain_id);

                                call(Call::plugin(
                                    self.ctx.plugin_name(),
                                    ModuleCall::SubmitTransaction(message),
                                ))
                            }
                            _ => panic!("unexpected message: {msg:?}"),
                        },
                    )
                })
                .collect(),
        })
    }
}

// #[allow(clippy::type_complexity)]
// fn process_msgs(
//     msgs: Vec<IbcMessage>,
//     sender: AccountAddress,
//     relayer: H160,
// ) -> Vec<(IbcMessage, EntryFunction)> {
//     let _ = (msgs, sender, relayer);

//     // msgs.clone()
//     //     .into_iter()
//     //     .map(|msg| match msg.clone() {
//     //         IbcMessage::CreateClient(MsgCreateClientData {
//     //             msg: data,
//     //             client_type,
//     //         }) => (
//     //             msg,
//     //             EntryFunction::new(
//     //                 MoveModuleId {
//     //                     address: (),
//     //                     name: (),
//     //                 }
//     //                 .into(),
//     //                 "create_client".parse().unwrap(),
//     //                 vec![],
//     //                 vec![client_type, data.client_state, data.consensus_state],
//     //             ),
//     //         ),
//     //         IbcMessage::UpdateClient(data) => (
//     //             msg,
//     //             mk_function_call(
//     //                 ibc_handler,
//     //                 UpdateClientCall(contracts::shared_types::MsgUpdateClient {
//     //                     client_id: data.client_id.to_string(),
//     //                     client_message: data.client_message.into(),
//     //                     relayer: relayer.into(),
//     //                 }),
//     //             ),
//     //         ),
//     //         _ => todo!(),
//     //     })
//     //     .collect()

//     todo!()
// }

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
use tracing::{instrument, warn};
use unionlabs::{
    hash::{H160, H256},
    ibc::core::connection::msg_connection_open_init::MsgConnectionOpenInit,
};
use voyager_message::{
    call::Call,
    data::{Data, IbcMessage, MsgCreateClientData, WithChainId},
    default_subcommand_handler,
    plugin::{OptimizationPassPluginServer, PluginInfo, PluginModuleServer},
    reth_ipc::client::IpcClientBuilder,
    run_module_server, ChainId, VoyagerMessage,
};

use crate::{call::ModuleCall, callback::ModuleCallback, data::ModuleData};

pub mod call;
pub mod callback;
pub mod data;

mod client;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_module_server(
        Module::new,
        OptimizationPassPluginServer::into_rpc,
        default_subcommand_handler,
    )
    .await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub client: Arc<jsonrpsee::ws_client::WsClient>,

    pub chain_id: ChainId<'static>,

    pub ibc_handler_address: Address,

    pub aptos_client: aptos_rest_client::Client,

    pub keyring: ConcurrentKeyring<AccountAddress, Arc<Ed25519PrivateKey>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub rpc_url: String,
    pub ibc_handler_address: Address,

    pub keyring: KeyringConfig,
}

impl client::Core::ClientExt for Module {
    fn client(&self) -> &aptos_rest_client::Client {
        &self.aptos_client
    }

    fn module_address(&self) -> AccountAddress {
        self.ibc_handler_address.into()
    }
}

impl Module {
    fn plugin_name(&self) -> String {
        pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

        format!("{PLUGIN_NAME}/{}", self.chain_id)
    }

    pub async fn new(config: Config, voyager_socket: String) -> Result<Self, BoxDynError> {
        let client = Arc::new(IpcClientBuilder::default().build(&voyager_socket).await?);

        let aptos_client = aptos_rest_client::Client::new(config.rpc_url.parse().unwrap());

        let chain_id = aptos_client.get_index().await?.inner().chain_id;

        Ok(Self {
            client,
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
}

#[async_trait]
impl PluginModuleServer<ModuleData, ModuleCall, ModuleCallback> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn info(&self) -> RpcResult<PluginInfo> {
        Ok(PluginInfo {
            name: self.plugin_name(),
            kind: None,
            interest_filter: Some(
                format!(
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
                    chain_id = self.chain_id,
                )
                .to_string(),
            ),
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn call(
        &self,
        msg: ModuleCall,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match msg {
            ModuleCall::SubmitTransaction(msgs) => self
                .keyring
                .with(|pk| {
                    let msgs = msgs.clone();
                    async move {
                        let sender = H256::from(
                            sha3::Sha3_256::new()
                                .chain_update(pk.public_key().to_bytes())
                                .chain_update([0])
                                .finalize(),
                        )
                        .0
                        .into();

                        let account = self
                            .aptos_client
                            .get_account(sender)
                            .await
                            .unwrap()
                            .into_inner();

                        dbg!(&account);

                        let msgs = process_msgs(self, msgs.clone());

                        for (msg, entry_fn) in msgs {
                            dbg!(msg);

                            let raw = RawTransaction::new_entry_function(
                                sender,
                                account.sequence_number,
                                entry_fn,
                                400000,
                                100,
                                queue_msg::now() + 10,
                                self.chain_id.as_str().parse().unwrap(),
                            );

                            let sig = raw.sign(pk, pk.public_key()).unwrap();

                            dbg!(&sig);

                            let res = self.aptos_client.submit_and_wait(&sig).await.unwrap();

                            dbg!(&res);
                        }

                        Ok(noop())
                    }
                })
                .await
                .unwrap_or_else(|| {
                    Ok(call(Call::plugin(
                        self.plugin_name(),
                        ModuleCall::SubmitTransaction(msgs),
                    )))
                }),
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn callback(
        &self,
        cb: ModuleCallback,
        _data: VecDeque<Data<ModuleData>>,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match cb {}
    }
}

#[async_trait]
impl OptimizationPassPluginServer<ModuleData, ModuleCall, ModuleCallback> for Module {
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
                                assert_eq!(chain_id, self.chain_id);

                                call(Call::plugin(
                                    self.plugin_name(),
                                    ModuleCall::SubmitTransaction(vec![message]),
                                ))
                            }
                            Op::Data(Data::IdentifiedIbcMessageBatch(WithChainId {
                                chain_id,
                                message,
                            })) => {
                                assert_eq!(chain_id, self.chain_id);

                                call(Call::plugin(
                                    self.plugin_name(),
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

#[allow(clippy::type_complexity)]
fn process_msgs(
    client: &impl client::Core::ClientExt,
    msgs: Vec<IbcMessage>,
) -> Vec<(IbcMessage, EntryFunction)> {
    msgs.clone()
        .into_iter()
        .map(|msg| match msg.clone() {
            IbcMessage::CreateClient(MsgCreateClientData {
                msg: data,
                client_type,
            }) => (msg, {
                let this = &client;
                let (_0, _1, _2) = (
                    client_type.to_string(),
                    data.client_state,
                    data.consensus_state,
                );
                ::aptos_types::transaction::EntryFunction::new(
                    ::aptos_rest_client::aptos_api_types::MoveModuleId {
                        address: this.module_address().into(),
                        name: stringify!(Core).parse().unwrap(),
                    }
                    .into(),
                    stringify!(create_client).parse().unwrap(),
                    vec![],
                    vec![
                        bcs::to_bytes(&_0).unwrap(),
                        bcs::to_bytes(&_1).unwrap(),
                        bcs::to_bytes(&_2).unwrap(),
                    ],
                )
            }),
            IbcMessage::UpdateClient(data) => (
                msg,
                client.update_client((data.client_id.to_string(), data.client_message)),
            ),
            IbcMessage::ConnectionOpenInit(data) => (
                msg,
                client.connection_open_init((
                    data.client_id.to_string(),
                    data.version.identifier,
                    data.version
                        .features
                        .into_iter()
                        .map(|f| f.to_string())
                        .collect::<Vec<String>>(),
                    data.counterparty.client_id.to_string(),
                    if let Some(conn) = data.counterparty.connection_id {
                        conn.to_string()
                    } else {
                        String::new()
                    },
                    data.counterparty.prefix.key_prefix,
                    data.delay_period,
                )),
            ),

            IbcMessage::ConnectionOpenTry(data) => (
                msg,
                client.connection_open_try((
                    data.counterparty.client_id.to_string(),
                    if let Some(conn) = data.counterparty.connection_id {
                        conn.to_string()
                    } else {
                        String::new()
                    },
                    data.counterparty.prefix.key_prefix,
                    data.delay_period,
                    data.client_id.to_string(),
                    data.client_state,
                    data.counterparty_versions
                        .iter()
                        .map(|v| v.identifier.clone())
                        .collect::<Vec<String>>(),
                    data.counterparty_versions
                        .iter()
                        .map(|v| {
                            v.features
                                .iter()
                                .map(|f| f.to_string())
                                .collect::<Vec<String>>()
                        })
                        .collect::<Vec<Vec<String>>>(),
                    data.proof_init,
                    data.proof_client,
                    data.proof_height.revision_number,
                    data.proof_height.revision_height,
                )),
            ),
            IbcMessage::ConnectionOpenAck(data) => (
                msg,
                client.connection_open_ack((
                    data.connection_id.to_string(),
                    data.client_state,
                    data.version.identifier,
                    data.version
                        .features
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>(),
                    data.proof_try,
                    data.proof_client,
                    data.counterparty_connection_id.to_string(),
                    data.proof_height.revision_number,
                    data.proof_height.revision_height,
                )),
            ),
            IbcMessage::ConnectionOpenConfirm(data) => (
                msg,
                client.connection_open_confirm((
                    data.connection_id.to_string(),
                    data.proof_ack,
                    data.proof_height.revision_number,
                    data.proof_height.revision_height,
                )),
            ),
            _ => todo!(),
        })
        .collect()
}

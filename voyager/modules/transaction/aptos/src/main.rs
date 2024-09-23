use std::{collections::VecDeque, sync::Arc};

use aptos_crypto::{ed25519::Ed25519PrivateKey, PrivateKey};
use aptos_rest_client::aptos_api_types::Address;
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
    core::ChainId,
    data::{log_msg, Data, IbcMessage, WithChainId},
    module::{ModuleInfo, PluginModuleInfo, PluginModuleServer, QueueInteractionsServer},
    run_module_server, DefaultCmd, ModuleContext, ModuleServer, VoyagerMessage,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId<'static>,
    pub rpc_url: String,
    pub ibc_handler_address: Address,

    pub keyring: KeyringConfig,
}

impl aptos_move_ibc::ibc::ClientExt for Module {
    fn client(&self) -> &aptos_rest_client::Client {
        &self.aptos_client
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

#[async_trait]
impl QueueInteractionsServer<ModuleData, ModuleCall, ModuleCallback> for ModuleServer<Module> {
    #[instrument(skip_all, fields(chain_id = %self.ctx.chain_id))]
    async fn call(
        &self,
        msg: ModuleCall,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match msg {
            ModuleCall::SubmitTransaction(msgs) => self
                .ctx
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
                            .ctx
                            .aptos_client
                            .get_account(sender)
                            .await
                            .unwrap()
                            .into_inner();

                        dbg!(&account);

                        let msgs = process_msgs(
                            self.ctx.ibc_handler_address.into(),
                            &self.ctx,
                            msgs.clone(),
                        );

                        let mut txs = vec![];

                        for (i, (msg, entry_fn)) in msgs.into_iter().enumerate() {
                            log_msg(&self.ctx.chain_id.to_string(), &msg);
                            // dbg!(msg);

                            let raw = RawTransaction::new_entry_function(
                                sender,
                                account.sequence_number + i as u64,
                                entry_fn,
                                400000,
                                100,
                                queue_msg::now() + 100,
                                self.ctx.chain_id.as_str().parse().unwrap(),
                            );

                            let signed_tx = raw.sign(pk, pk.public_key()).unwrap();

                            dbg!(&signed_tx);

                            txs.push(signed_tx.into_inner());
                        }

                        dbg!(&txs);

                        let res = self.ctx.aptos_client.submit_batch(&txs).await.unwrap();

                        dbg!(&res);

                        // res.into_inner().transaction_failures

                        Ok(noop())
                    }
                })
                .await
                .unwrap_or_else(|| {
                    Ok(call(Call::plugin(
                        self.ctx.plugin_name(),
                        ModuleCall::SubmitTransaction(msgs),
                    )))
                }),
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

#[allow(clippy::type_complexity)]
fn process_msgs<T: aptos_move_ibc::ibc::ClientExt>(
    ibc_handler_address: AccountAddress,
    client: &T,
    msgs: Vec<IbcMessage>,
) -> Vec<(IbcMessage, EntryFunction)> {
    msgs.clone()
        .into_iter()
        .map(|msg| match msg.clone() {
            IbcMessage::CreateClient(data) => (
                msg,
                client.create_client(
                    ibc_handler_address,
                    (
                        data.client_type.to_string(),
                        data.msg.client_state,
                        data.msg.consensus_state,
                    ),
                ),
            ),
            IbcMessage::UpdateClient(data) => (
                msg,
                client.update_client(
                    ibc_handler_address,
                    (data.client_id.to_string(), data.client_message),
                ),
            ),
            IbcMessage::ConnectionOpenInit(data) => (
                msg,
                client.connection_open_init(
                    ibc_handler_address,
                    (
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
                    ),
                ),
            ),

            IbcMessage::ConnectionOpenTry(data) => (
                msg,
                client.connection_open_try(
                    ibc_handler_address,
                    (
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
                    ),
                ),
            ),
            IbcMessage::ConnectionOpenAck(data) => (
                msg,
                client.connection_open_ack(
                    ibc_handler_address,
                    (
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
                    ),
                ),
            ),
            IbcMessage::ConnectionOpenConfirm(data) => (
                msg,
                client.connection_open_confirm(
                    ibc_handler_address,
                    (
                        data.connection_id.to_string(),
                        data.proof_ack,
                        data.proof_height.revision_number,
                        data.proof_height.revision_height,
                    ),
                ),
            ),
            IbcMessage::ChannelOpenInit(data) => (
                msg,
                client.channel_open_init(
                    data.port_id.to_string().parse().unwrap(),
                    (
                        data.channel
                            .connection_hops
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<String>>(),
                        data.channel.ordering as u8,
                        data.channel.counterparty.port_id.to_string(),
                        data.channel.counterparty.channel_id,
                        data.channel.version,
                    ),
                ),
            ),
            IbcMessage::ChannelOpenTry(data) => (
                msg,
                client.channel_open_try(
                    data.port_id.to_string().parse().unwrap(),
                    (
                        data.channel
                            .connection_hops
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<String>>(),
                        data.channel.ordering as u8,
                        data.channel.counterparty.port_id.to_string(),
                        data.channel.counterparty.channel_id,
                        data.counterparty_version,
                        data.channel.version,
                        data.proof_init,
                        data.proof_height.revision_number,
                        data.proof_height.revision_height,
                    ),
                ),
            ),
            IbcMessage::ChannelOpenAck(data) => (
                msg,
                client.channel_open_ack(
                    data.port_id.to_string().parse().unwrap(),
                    (
                        data.channel_id.to_string(),
                        data.counterparty_channel_id.to_string(),
                        data.counterparty_version,
                        data.proof_try,
                        data.proof_height.revision_number,
                        data.proof_height.revision_height,
                    ),
                ),
            ),
            IbcMessage::ChannelOpenConfirm(data) => (
                msg,
                client.channel_open_confirm(
                    data.port_id.to_string().parse().unwrap(),
                    (
                        data.channel_id.to_string(),
                        data.proof_ack,
                        data.proof_height.revision_number,
                        data.proof_height.revision_height,
                    ),
                ),
            ),
            IbcMessage::RecvPacket(data) => (
                msg,
                client.recv_packet(
                    data.packet.destination_port.to_string().parse().unwrap(),
                    (
                        data.packet.sequence.get(),
                        data.packet.source_port.to_string(),
                        data.packet.source_channel.to_string(),
                        data.packet.destination_port.to_string(),
                        data.packet.destination_channel.to_string(),
                        data.packet.data,
                        data.packet.timeout_height.revision_number,
                        data.packet.timeout_height.revision_height,
                        data.packet.timeout_timestamp,
                        data.proof_commitment,
                        data.proof_height.revision_number,
                        data.proof_height.revision_height,
                    ),
                ),
            ),
            IbcMessage::AcknowledgePacket(data) => (
                msg,
                client.acknowledge_packet(
                    data.packet.source_port.to_string().parse().unwrap(),
                    (
                        data.packet.sequence.get(),
                        data.packet.source_port.to_string(),
                        data.packet.source_channel.to_string(),
                        data.packet.destination_port.to_string(),
                        data.packet.destination_channel.to_string(),
                        data.packet.data,
                        data.packet.timeout_height.revision_number,
                        data.packet.timeout_height.revision_height,
                        data.packet.timeout_timestamp,
                        data.acknowledgement,
                        data.proof_acked,
                        data.proof_height.revision_number,
                        data.proof_height.revision_height,
                    ),
                ),
            ),
            _ => todo!(),
        })
        .collect()
}

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
use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use serde::{Deserialize, Serialize};
use sha3::Digest;
use tracing::{info, instrument};
use unionlabs::{
    hash::H256,
    id::{ChannelId, ConnectionId},
};
use voyager_message::{
    core::ChainId,
    data::{Data, IbcMessage, WithChainId},
    module::{PluginInfo, PluginServer},
    run_plugin_server, DefaultCmd, Plugin, PluginMessage, VoyagerMessage,
};
use voyager_vm::{call, noop, pass::PassResult, Op};

use crate::{call::ModuleCall, callback::ModuleCallback};

pub mod call;
pub mod callback;
pub mod data;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_plugin_server::<Module>().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId<'static>,

    pub ibc_handler_address: Address,

    pub aptos_client: aptos_rest_client::Client,

    pub keyring: ConcurrentKeyring<AccountAddress, Arc<Ed25519PrivateKey>>,
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

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

                    let address = (*<H256>::from(
                        sha3::Sha3_256::new()
                            .chain_update(pk.public_key().to_bytes())
                            .chain_update([0])
                            .finalize(),
                    )
                    .get())
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

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.chain_id),
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
impl PluginServer<ModuleCall, ModuleCallback> for Module {
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
                .map(|(idx, op)| {
                    (
                        vec![idx],
                        match op {
                            Op::Data(Data::IdentifiedIbcMessage(WithChainId {
                                chain_id,
                                message,
                            })) => {
                                assert_eq!(chain_id, self.chain_id);

                                call(PluginMessage::new(
                                    self.plugin_name(),
                                    ModuleCall::SubmitTransaction(vec![message]),
                                ))
                            }
                            Op::Data(Data::IdentifiedIbcMessageBatch(WithChainId {
                                chain_id,
                                message,
                            })) => {
                                assert_eq!(chain_id, self.chain_id);

                                call(PluginMessage::new(
                                    self.plugin_name(),
                                    ModuleCall::SubmitTransaction(message),
                                ))
                            }
                            _ => panic!("unexpected message: {op:?}"),
                        },
                    )
                })
                .collect(),
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn call(&self, _: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::SubmitTransaction(msgs) => self
                .keyring
                .with(|pk| {
                    let msgs = msgs.clone();
                    async move {
                        let sender = (*<H256>::from(
                            sha3::Sha3_256::new()
                                .chain_update(pk.public_key().to_bytes())
                                .chain_update([0])
                                .finalize(),
                        )
                        .get())
                        .into();

                        let account = self
                            .aptos_client
                            .get_account(sender)
                            .await
                            .unwrap()
                            .into_inner();

                        dbg!(&account);

                        let msgs =
                            process_msgs(self.ibc_handler_address.into(), self, msgs.clone());

                        let mut txs = vec![];

                        for (i, (msg, entry_fn)) in msgs.into_iter().enumerate() {
                            info!(chain_id = %self.chain_id, msg = msg.as_value());
                            // log_msg(&self.chain_id.to_string(), &msg);
                            // dbg!(msg);

                            let raw = RawTransaction::new_entry_function(
                                sender,
                                account.sequence_number + i as u64,
                                entry_fn,
                                400000,
                                100,
                                voyager_vm::now() + 100,
                                self.chain_id.as_str().parse().unwrap(),
                            );

                            let signed_tx = raw.sign(pk, pk.public_key()).unwrap();

                            dbg!(&signed_tx);

                            txs.push(signed_tx.into_inner());
                        }

                        dbg!(&txs);

                        let res = self.aptos_client.submit_batch(&txs).await.unwrap();

                        dbg!(&res);

                        // res.into_inner().transaction_failures

                        Ok(noop())
                    }
                })
                .await
                .unwrap_or_else(|| {
                    Ok(call(PluginMessage::new(
                        self.plugin_name(),
                        ModuleCall::SubmitTransaction(msgs),
                    )))
                }),
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn callback(
        &self,
        _: &Extensions,
        cb: ModuleCallback,
        _data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match cb {}
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
                    (
                        data.client_id.to_string_prefixed("PREFIX"),
                        data.client_message,
                    ),
                ),
            ),
            IbcMessage::ConnectionOpenInit(data) => (
                msg,
                client.connection_open_init(
                    ibc_handler_address,
                    (
                        data.client_id.to_string_prefixed("PREFIX"),
                        data.version.identifier,
                        data.version
                            .features
                            .into_iter()
                            .map(|f| f.to_string())
                            .collect::<Vec<String>>(),
                        data.counterparty.client_id.to_string_prefixed("PREFIX"),
                        if let Some(conn) = data.counterparty.connection_id {
                            conn.to_string_prefixed()
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
                        // TODO: Figure this out
                        data.counterparty.client_id.to_string_prefixed("PREFIX"),
                        if let Some(conn) = data.counterparty.connection_id {
                            conn.to_string_prefixed()
                        } else {
                            String::new()
                        },
                        data.counterparty.prefix.key_prefix,
                        data.delay_period,
                        // TODO: Figure this out
                        data.client_id.to_string_prefixed("PREFIX"),
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
                        data.proof_height.revision(),
                        data.proof_height.height(),
                    ),
                ),
            ),
            IbcMessage::ConnectionOpenAck(data) => (
                msg,
                client.connection_open_ack(
                    ibc_handler_address,
                    (
                        data.connection_id.to_string_prefixed(),
                        data.client_state,
                        data.version.identifier,
                        data.version
                            .features
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<String>>(),
                        data.proof_try,
                        data.proof_client,
                        data.counterparty_connection_id.to_string_prefixed(),
                        data.proof_height.revision(),
                        data.proof_height.height(),
                    ),
                ),
            ),
            IbcMessage::ConnectionOpenConfirm(data) => (
                msg,
                client.connection_open_confirm(
                    ibc_handler_address,
                    (
                        data.connection_id.to_string_prefixed(),
                        data.proof_ack,
                        data.proof_height.revision(),
                        data.proof_height.height(),
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
                            .map(ConnectionId::to_string_prefixed)
                            .collect::<Vec<String>>(),
                        data.channel.ordering as u8,
                        data.channel.counterparty.port_id.to_string(),
                        data.channel
                            .counterparty
                            .channel_id
                            .as_ref()
                            .map(ChannelId::to_string_prefixed)
                            .unwrap_or_default(),
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
                            .map(ConnectionId::to_string_prefixed)
                            .collect::<Vec<String>>(),
                        data.channel.ordering as u8,
                        data.channel.counterparty.port_id.to_string(),
                        data.channel
                            .counterparty
                            .channel_id
                            .as_ref()
                            .map(ChannelId::to_string_prefixed)
                            .unwrap_or_default(),
                        data.counterparty_version,
                        data.channel.version,
                        data.proof_init,
                        data.proof_height.revision(),
                        data.proof_height.height(),
                    ),
                ),
            ),
            IbcMessage::ChannelOpenAck(data) => (
                msg,
                client.channel_open_ack(
                    data.port_id.to_string().parse().unwrap(),
                    (
                        data.channel_id.to_string_prefixed(),
                        data.counterparty_channel_id.to_string_prefixed(),
                        data.counterparty_version,
                        data.proof_try,
                        data.proof_height.revision(),
                        data.proof_height.height(),
                    ),
                ),
            ),
            IbcMessage::ChannelOpenConfirm(data) => (
                msg,
                client.channel_open_confirm(
                    data.port_id.to_string().parse().unwrap(),
                    (
                        data.channel_id.to_string_prefixed(),
                        data.proof_ack,
                        data.proof_height.revision(),
                        data.proof_height.height(),
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
                        data.packet.source_channel.to_string_prefixed(),
                        data.packet.destination_port.to_string(),
                        data.packet.destination_channel.to_string_prefixed(),
                        data.packet.data,
                        data.packet.timeout_height.revision(),
                        data.packet.timeout_height.height(),
                        data.packet.timeout_timestamp,
                        data.proof_commitment,
                        data.proof_height.revision(),
                        data.proof_height.height(),
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
                        data.packet.source_channel.to_string_prefixed(),
                        data.packet.destination_port.to_string(),
                        data.packet.destination_channel.to_string_prefixed(),
                        data.packet.data,
                        data.packet.timeout_height.revision(),
                        data.packet.timeout_height.height(),
                        data.packet.timeout_timestamp,
                        data.acknowledgement,
                        data.proof_acked,
                        data.proof_height.revision(),
                        data.proof_height.height(),
                    ),
                ),
            ),
            _ => todo!(),
        })
        .collect()
}

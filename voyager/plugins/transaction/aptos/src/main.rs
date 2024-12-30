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
use ibc_union_spec::{Datagram, IbcUnion};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use move_core_types::{
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
};
use serde::{Deserialize, Serialize};
use sha3::Digest;
use tracing::instrument;
use unionlabs::hash::H256;
use voyager_message::{
    core::ChainId,
    data::Data,
    hook::SubmitTxHook,
    module::{PluginInfo, PluginServer},
    DefaultCmd, Plugin, PluginMessage, VoyagerMessage,
};
use voyager_vm::{call, noop, pass::PassResult, Op, Visit};

use crate::{call::ModuleCall, callback::ModuleCallback};

pub mod call;
pub mod callback;
pub mod data;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

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
            interest_filter: SubmitTxHook::filter(&config.chain_id),
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId,
    pub rpc_url: String,
    pub ibc_handler_address: Address,

    pub keyring: KeyringConfig,
}

impl aptos_move_ibc::ibc::ClientExt for Module {
    fn client(&self) -> &aptos_rest_client::Client {
        &self.aptos_client
    }
}

impl aptos_move_ibc::recv_packet::ClientExt for Module {
    fn client(&self) -> &aptos_rest_client::Client {
        &self.aptos_client
    }
}

impl aptos_move_ibc::acknowledge_packet::ClientExt for Module {
    fn client(&self) -> &aptos_rest_client::Client {
        &self.aptos_client
    }
}

impl aptos_move_ibc::channel_handshake::ClientExt for Module {
    fn client(&self) -> &aptos_rest_client::Client {
        &self.aptos_client
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
                .map(|(idx, mut op)| {
                    SubmitTxHook::new(&self.chain_id, |submit_tx| {
                        PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::SubmitTransaction(
                                submit_tx
                                    .datagrams
                                    .iter()
                                    .map(|message| {
                                        message.decode_datagram::<IbcUnion>().unwrap().unwrap()
                                    })
                                    .collect(),
                                // .collect::<Result<_, _>>()?,
                            ),
                        )
                        .into()
                    })
                    .visit_op(&mut op);

                    (vec![idx], op)
                })
                .collect(),
            // .collect::<RpcResult<_>>()?,
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
                            process_msgs(self.ibc_handler_address.into(), self, msgs.clone()).await;

                        let mut txs = vec![];

                        for (i, (_, entry_fn)) in msgs.into_iter().enumerate() {
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

fn ibc_app_witness(module: AccountAddress) -> TypeTag {
    TypeTag::Struct(Box::new(StructTag {
        address: module,
        module: Identifier::new("ibc_app").unwrap(),
        name: Identifier::new("IbcAppWitness").unwrap(),
        type_args: vec![],
    }))
}

#[allow(clippy::type_complexity)]
async fn process_msgs<
    T: aptos_move_ibc::ibc::ClientExt
        + aptos_move_ibc::recv_packet::ClientExt
        + aptos_move_ibc::acknowledge_packet::ClientExt
        + aptos_move_ibc::channel_handshake::ClientExt,
>(
    ibc_handler_address: AccountAddress,
    client: &T,
    msgs: Vec<Datagram>,
) -> Vec<(Datagram, EntryFunction)> {
    let mut data = vec![];
    for msg in msgs {
        let item = match msg.clone() {
            Datagram::CreateClient(data) => (
                msg,
                client.create_client(
                    ibc_handler_address,
                    (
                        data.client_type.to_string(),
                        data.client_state_bytes.into_vec(),
                        data.consensus_state_bytes.into_vec(),
                    ),
                ),
            ),
            Datagram::UpdateClient(data) => (
                msg,
                client.update_client(
                    ibc_handler_address,
                    (data.client_id, data.client_message.into_vec()),
                ),
            ),
            Datagram::ConnectionOpenInit(data) => (
                msg,
                client.connection_open_init(
                    ibc_handler_address,
                    (data.client_id, data.counterparty_client_id),
                ),
            ),

            Datagram::ConnectionOpenTry(data) => (
                msg,
                client.connection_open_try(
                    ibc_handler_address,
                    (
                        data.counterparty_client_id,
                        data.counterparty_connection_id,
                        data.client_id,
                        data.proof_init.into_vec(),
                        data.proof_height,
                    ),
                ),
            ),
            Datagram::ConnectionOpenAck(data) => (
                msg,
                client.connection_open_ack(
                    ibc_handler_address,
                    (
                        data.connection_id,
                        data.counterparty_connection_id,
                        data.proof_try.into_vec(),
                        data.proof_height,
                    ),
                ),
            ),
            Datagram::ConnectionOpenConfirm(data) => (
                msg,
                client.connection_open_confirm(
                    ibc_handler_address,
                    (
                        data.connection_id,
                        data.proof_ack.into_vec(),
                        data.proof_height,
                    ),
                ),
            ),
            Datagram::ChannelOpenInit(data) => (
                msg,
                client.channel_open_init(
                    ibc_handler_address,
                    (
                        AccountAddress::try_from(data.port_id.as_ref()).unwrap(),
                        data.counterparty_port_id.into_vec(),
                        data.connection_id,
                        data.version,
                    ),
                    (ibc_app_witness(data.port_id.as_ref().try_into().unwrap()),),
                ),
            ),
            Datagram::ChannelOpenTry(data) => (
                msg,
                client.channel_open_try(
                    ibc_handler_address,
                    (
                        AccountAddress::try_from(data.port_id.as_ref()).unwrap(),
                        data.channel.connection_id,
                        data.channel.counterparty_channel_id,
                        data.channel.counterparty_port_id.to_vec(),
                        data.channel.version,
                        data.counterparty_version,
                        data.proof_init.into_vec(),
                        data.proof_height,
                    ),
                    (ibc_app_witness(data.port_id.as_ref().try_into().unwrap()),),
                ),
            ),
            Datagram::ChannelOpenAck(data) => {
                let port_id = client
                    .get_module(ibc_handler_address, None, (data.channel_id,))
                    .await
                    .unwrap();
                (
                    msg,
                    client.channel_open_ack(
                        ibc_handler_address,
                        (
                            port_id.into(),
                            data.channel_id,
                            data.counterparty_version,
                            data.counterparty_channel_id,
                            data.proof_try.into_vec(),
                            data.proof_height,
                        ),
                        (ibc_app_witness(port_id.into()),),
                    ),
                )
            }
            Datagram::ChannelOpenConfirm(data) => {
                let port_id = client
                    .get_module(ibc_handler_address, None, (data.channel_id,))
                    .await
                    .unwrap();
                (
                    msg,
                    client.channel_open_confirm(
                        ibc_handler_address,
                        (
                            port_id.into(),
                            data.channel_id,
                            data.proof_ack.into_vec(),
                            data.proof_height,
                        ),
                        (ibc_app_witness(port_id.into()),),
                    ),
                )
            }
            Datagram::PacketRecv(data) => {
                let (
                    source_channels,
                    (destination_channels, (packet_data, (timeout_heights, timeout_timestamps))),
                ): (Vec<_>, (Vec<_>, (Vec<_>, (Vec<_>, Vec<_>)))) = data
                    .packets
                    .into_iter()
                    .map(|p| {
                        (
                            p.source_channel,
                            (
                                p.destination_channel,
                                (p.data.to_vec(), (p.timeout_height, p.timeout_timestamp)),
                            ),
                        )
                    })
                    .unzip();

                let port_id = client
                    .get_module(ibc_handler_address, None, (destination_channels[0],))
                    .await
                    .unwrap();

                (
                    msg,
                    client.recv_packet(
                        ibc_handler_address,
                        (
                            port_id.into(),
                            source_channels,
                            destination_channels,
                            packet_data,
                            timeout_heights,
                            timeout_timestamps,
                            data.proof.into_vec(),
                            data.proof_height,
                        ),
                        (ibc_app_witness(port_id.into()),),
                    ),
                )
            }
            Datagram::PacketAcknowledgement(data) => {
                let (source_channels, destination_channels) = data
                    .packets
                    .into_iter()
                    .map(|p| (p.source_channel, p.destination_channel))
                    .collect::<(Vec<u32>, Vec<u32>)>();

                let port_id = client
                    .get_module(ibc_handler_address, None, (source_channels[0],))
                    .await
                    .unwrap();

                (
                    msg,
                    client.acknowledge_packet(
                        ibc_handler_address,
                        (
                            port_id.into(),
                            source_channels,
                            destination_channels,
                            vec![],
                            vec![],
                            vec![],
                            vec![],
                            vec![],
                            0,
                        ),
                        (ibc_app_witness(port_id.into()),),
                    ),
                )
            }
            _ => todo!(),
        };
        data.push(item);
    }

    data
}

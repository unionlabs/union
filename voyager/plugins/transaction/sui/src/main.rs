use std::{collections::VecDeque, panic::AssertUnwindSafe, str::FromStr, sync::Arc};

use concurrent_keyring::{ConcurrentKeyring, KeyringConfig, KeyringEntry};
use fastcrypto::{hash::HashFunction, traits::Signer};
use ibc_union_spec::{datagram::Datagram, IbcUnion};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use shared_crypto::intent::{Intent, IntentMessage};
use sui_sdk::{
    rpc_types::{SuiObjectDataOptions, SuiTransactionBlockResponseOptions, SuiTypeTag},
    types::{
        base_types::{ObjectID, SequenceNumber, SuiAddress},
        crypto::{DefaultHash, SignatureScheme, SuiKeyPair, SuiSignature},
        programmable_transaction_builder::ProgrammableTransactionBuilder,
        signature::GenericSignature,
        transaction::{
            Argument, CallArg, Command, ObjectArg, Transaction, TransactionData, TransactionKind,
        },
        Identifier,
    },
    SuiClient, SuiClientBuilder,
};
use tracing::{info, instrument};
use unionlabs::{
    primitives::{encoding::HexPrefixed, Bytes},
    ErrorReporter,
};
use voyager_message::{
    data::Data,
    hook::SubmitTxHook,
    module::{PluginInfo, PluginServer},
    primitives::ChainId,
    vm::{call, noop, pass::PassResult, Op, Visit},
    DefaultCmd, Plugin, PluginMessage, VoyagerMessage,
};
use voyager_vm::BoxDynError;

use crate::{call::ModuleCall, callback::ModuleCallback};

pub mod call;
pub mod callback;
pub mod data;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub ibc_handler_address: SuiAddress,

    pub ibc_store: SuiAddress,

    pub sui_client: sui_sdk::SuiClient,

    pub keyring: ConcurrentKeyring<SuiAddress, Arc<SuiKeyPair>>,

    pub ibc_store_initial_seq: SequenceNumber,
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        let sui_client = SuiClientBuilder::default().build(&config.rpc_url).await?;

        let chain_id = sui_client.read_api().get_chain_identifier().await?;

        let ibc_store_initial_seq = sui_client
            .read_api()
            .get_object_with_options(
                ObjectID::new(config.ibc_store.to_inner()),
                SuiObjectDataOptions::default().with_owner(),
            )
            .await
            .map_err(|e| ErrorObject::owned(-1, ErrorReporter(e).to_string(), None::<()>))?
            .data
            .expect("ibc store object exists on chain")
            .owner
            .expect("owner will be present")
            .start_version()
            .expect("ibc store is shared, hence it has a start version");

        Ok(Self {
            chain_id: ChainId::new(chain_id.to_string()),
            ibc_handler_address: config.ibc_handler_address,
            sui_client,
            ibc_store_initial_seq,
            keyring: ConcurrentKeyring::new(
                config.keyring.name,
                config.keyring.keys.into_iter().map(|config| {
                    println!("{}", Bytes::<HexPrefixed>::new(config.value()));
                    let pk = SuiKeyPair::decode(
                        &String::from_utf8(config.value()).expect("priv keys are utf8 strings"),
                    )
                    .expect("private key is valid");

                    let address = SuiAddress::from(&pk.public());

                    KeyringEntry {
                        address,
                        signer: Arc::new(pk),
                    }
                }),
            ),
            ibc_store: config.ibc_store,
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
    pub ibc_handler_address: SuiAddress,
    pub ibc_store: SuiAddress,

    pub keyring: KeyringConfig,
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
                            ),
                        )
                        .into()
                    })
                    .visit_op(&mut op);

                    (vec![idx], op)
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
                    let sender = SuiAddress::from(&pk.public());
                    let msgs = msgs.clone();
                    AssertUnwindSafe(async move {
                        let gas_coin = self
                            .sui_client
                            .coin_read_api()
                            .get_coins(sender, None, None, None)
                            .await
                            .expect("sender is broke")
                            .data
                            .into_iter()
                            .next()
                            .expect("sender has a gas token");

                        let gas_budget = 20_000_000;
                        let gas_price = self
                            .sui_client
                            .read_api()
                            .get_reference_gas_price()
                            .await
                            .map_err(|e| {
                                ErrorObject::owned(
                                    -1,
                                    ErrorReporter(e)
                                        .with_message("error fetching the reference gas price"),
                                    None::<()>,
                                )
                            })?;

                        // create the transaction data that will be sent to the network.
                        //
                        let msgs = process_msgs(self, msgs.clone(), sender).await;

                        let mut ptb = ProgrammableTransactionBuilder::new();

                        for (_, (contract_addr, _, module, entry_fn, arguments)) in
                            msgs.into_iter().enumerate()
                        {
                            let arguments = arguments
                                .into_iter()
                                .map(|arg| ptb.input(arg).expect("input works"))
                                .collect();
                            ptb.command(Command::move_call(
                                contract_addr.into(),
                                module,
                                entry_fn,
                                vec![],
                                arguments,
                            ));
                        }

                        let builder = ptb.finish();

                        let tx_data = TransactionData::new_programmable(
                            sender,
                            vec![gas_coin.object_ref()],
                            builder,
                            gas_budget,
                            gas_price,
                        );

                        let intent_msg = IntentMessage::new(Intent::sui_transaction(), tx_data);
                        let raw_tx = bcs::to_bytes(&intent_msg).expect("bcs should not fail");
                        let mut hasher = DefaultHash::default();
                        hasher.update(raw_tx.clone());
                        let digest = hasher.finalize().digest;

                        // use SuiKeyPair to sign the digest.
                        let sui_sig = pk.sign(&digest);

                        sui_sig
                            .verify_secure(&intent_msg, sender, SignatureScheme::ED25519)
                            .expect("sender has a valid signature");

                        info!("submitting sui tx");

                        let transaction_response = self
                            .sui_client
                            .quorum_driver_api()
                            .execute_transaction_block(
                                Transaction::from_generic_sig_data(
                                    intent_msg.value,
                                    vec![GenericSignature::Signature(sui_sig)],
                                ),
                                SuiTransactionBlockResponseOptions::default(),
                                None,
                            )
                            .await
                            .map_err(|e| {
                                ErrorObject::owned(
                                    -1,
                                    ErrorReporter(e).with_message("error executing a tx"),
                                    None::<()>,
                                )
                            })?;

                        info!("{transaction_response:?}");

                        Ok(noop())
                    })
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

// module: Identifier,
// function: Identifier,
// type_arguments: Vec<TypeTag>,

#[allow(clippy::type_complexity)]
async fn process_msgs(
    module: &Module,
    msgs: Vec<Datagram>,
    fee_recipient: SuiAddress,
) -> Vec<(SuiAddress, Datagram, Identifier, Identifier, Vec<CallArg>)> {
    let mut data = vec![];
    for msg in msgs {
        let item = match msg.clone() {
            Datagram::CreateClient(data) => (
                module.ibc_handler_address,
                msg,
                Identifier::new("ibc").unwrap(),
                Identifier::new("create_client").unwrap(),
                vec![
                    CallArg::Object(ObjectArg::SharedObject {
                        id: module.ibc_store.into(),
                        initial_shared_version: module.ibc_store_initial_seq,
                        mutable: true,
                    }),
                    CallArg::Pure(bcs::to_bytes(&data.client_type.to_string()).unwrap()),
                    CallArg::Pure(bcs::to_bytes(&data.client_state_bytes).unwrap()),
                    CallArg::Pure(bcs::to_bytes(&data.consensus_state_bytes).unwrap()),
                ],
            ),
            Datagram::UpdateClient(data) => (
                module.ibc_handler_address,
                msg,
                Identifier::new("ibc").unwrap(),
                Identifier::new("update_client").unwrap(),
                vec![
                    CallArg::Object(ObjectArg::SharedObject {
                        id: module.ibc_store.into(),
                        initial_shared_version: module.ibc_store_initial_seq,
                        mutable: true,
                    }),
                    CallArg::Pure(bcs::to_bytes(&data.client_id).unwrap()),
                    CallArg::Pure(bcs::to_bytes(&data.client_message).unwrap()),
                ],
            ),
            Datagram::ConnectionOpenInit(data) => (
                module.ibc_handler_address,
                msg,
                Identifier::new("ibc").unwrap(),
                Identifier::new("connection_open_init").unwrap(),
                vec![
                    CallArg::Object(ObjectArg::SharedObject {
                        id: module.ibc_store.into(),
                        initial_shared_version: module.ibc_store_initial_seq,
                        mutable: true,
                    }),
                    CallArg::Pure(bcs::to_bytes(&data.client_id).unwrap()),
                    CallArg::Pure(bcs::to_bytes(&data.counterparty_client_id).unwrap()),
                ],
            ),
            Datagram::ConnectionOpenTry(data) => (
                module.ibc_handler_address,
                msg,
                Identifier::new("ibc").unwrap(),
                Identifier::new("connection_open_try").unwrap(),
                vec![
                    CallArg::Object(ObjectArg::SharedObject {
                        id: module.ibc_store.into(),
                        initial_shared_version: module.ibc_store_initial_seq,
                        mutable: true,
                    }),
                    CallArg::Pure(bcs::to_bytes(&data.counterparty_client_id).unwrap()),
                    CallArg::Pure(bcs::to_bytes(&data.counterparty_connection_id).unwrap()),
                    CallArg::Pure(bcs::to_bytes(&data.client_id).unwrap()),
                    CallArg::Pure(bcs::to_bytes(&data.proof_init).unwrap()),
                    CallArg::Pure(bcs::to_bytes(&data.proof_height).unwrap()),
                ],
            ),
            Datagram::ConnectionOpenAck(data) => (
                module.ibc_handler_address,
                msg,
                Identifier::new("ibc").unwrap(),
                Identifier::new("connection_open_ack").unwrap(),
                vec![
                    CallArg::Object(ObjectArg::SharedObject {
                        id: module.ibc_store.into(),
                        initial_shared_version: module.ibc_store_initial_seq,
                        mutable: true,
                    }),
                    CallArg::Pure(bcs::to_bytes(&data.connection_id).unwrap()),
                    CallArg::Pure(bcs::to_bytes(&data.counterparty_connection_id).unwrap()),
                    CallArg::Pure(bcs::to_bytes(&data.proof_try).unwrap()),
                    CallArg::Pure(bcs::to_bytes(&data.proof_height).unwrap()),
                ],
            ),
            Datagram::ConnectionOpenConfirm(data) => (
                module.ibc_handler_address,
                msg,
                Identifier::new("ibc").unwrap(),
                Identifier::new("connection_open_confirm").unwrap(),
                vec![
                    CallArg::Object(ObjectArg::SharedObject {
                        id: module.ibc_store.into(),
                        initial_shared_version: module.ibc_store_initial_seq,
                        mutable: true,
                    }),
                    CallArg::Pure(bcs::to_bytes(&data.connection_id).unwrap()),
                    CallArg::Pure(bcs::to_bytes(&data.proof_ack).unwrap()),
                    CallArg::Pure(bcs::to_bytes(&data.proof_height).unwrap()),
                ],
            ),
            Datagram::ChannelOpenInit(data) => {
                let port_id = String::from_utf8(data.port_id.to_vec()).expect("port id is String");

                let module_info = port_id.split("::").collect::<Vec<&str>>();
                if module_info.len() != 3 {
                    panic!("invalid port id");
                }

                let addr = SuiAddress::from_str(module_info[0]).expect("module string is correct");

                (
                    addr,
                    msg,
                    Identifier::new(module_info[1]).unwrap(),
                    Identifier::new("channel_open_init").unwrap(),
                    vec![
                        CallArg::Object(ObjectArg::SharedObject {
                            id: module.ibc_store.into(),
                            initial_shared_version: module.ibc_store_initial_seq,
                            mutable: true,
                        }),
                        CallArg::Pure(bcs::to_bytes(&data.counterparty_port_id).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.connection_id).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.version).unwrap()),
                    ],
                )
            }
            Datagram::ChannelOpenTry(data) => {
                let port_id = String::from_utf8(data.port_id.to_vec()).expect("port id is String");

                let module_info = port_id.split("::").collect::<Vec<&str>>();
                if module_info.len() != 3 {
                    panic!("invalid port id");
                }

                let addr = SuiAddress::from_str(module_info[0]).expect("module string is correct");

                (
                    addr,
                    msg,
                    Identifier::new(module_info[1]).unwrap(),
                    Identifier::new("channel_open_try").unwrap(),
                    vec![
                        CallArg::Object(ObjectArg::SharedObject {
                            id: module.ibc_store.into(),
                            initial_shared_version: module.ibc_store_initial_seq,
                            mutable: true,
                        }),
                        CallArg::Pure(bcs::to_bytes(&data.channel.connection_id).unwrap()),
                        CallArg::Pure(
                            bcs::to_bytes(&data.channel.counterparty_channel_id).unwrap(),
                        ),
                        CallArg::Pure(bcs::to_bytes(&data.channel.counterparty_port_id).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.channel.version).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.counterparty_version).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.proof_init).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.proof_height).unwrap()),
                    ],
                )
            }
            Datagram::ChannelOpenAck(data) => {
                let query = SuiQuery::new(&module.sui_client, module.ibc_store.into()).await;

                let res = query
                    .add_param(data.channel_id.raw())
                    .call(module.ibc_handler_address.into(), "get_port_id")
                    .await
                    .unwrap();

                if res.len() != 1 {
                    panic!("expected a single encoded connection end")
                }

                let port_id = bcs::from_bytes::<String>(&res[0].0).unwrap();

                let module_info = port_id.split("::").collect::<Vec<&str>>();
                if module_info.len() != 3 {
                    panic!("invalid port id");
                }

                let addr = SuiAddress::from_str(module_info[0]).expect("module string is correct");
                (
                    addr,
                    msg,
                    Identifier::new(module_info[1]).unwrap(),
                    Identifier::new("channel_open_ack").unwrap(),
                    vec![
                        CallArg::Object(ObjectArg::SharedObject {
                            id: module.ibc_store.into(),
                            initial_shared_version: module.ibc_store_initial_seq,
                            mutable: true,
                        }),
                        CallArg::Pure(bcs::to_bytes(&data.channel_id).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.counterparty_version).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.counterparty_channel_id).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.proof_try).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.proof_height).unwrap()),
                    ],
                )
            }
            Datagram::ChannelOpenConfirm(data) => {
                let query = SuiQuery::new(&module.sui_client, module.ibc_store.into()).await;

                let res = query
                    .add_param(data.channel_id.raw())
                    .call(module.ibc_handler_address.into(), "get_port_id")
                    .await
                    .unwrap();

                if res.len() != 1 {
                    panic!("expected a single encoded connection end")
                }

                let port_id = bcs::from_bytes::<String>(&res[0].0).unwrap();

                let module_info = port_id.split("::").collect::<Vec<&str>>();
                if module_info.len() != 3 {
                    panic!("invalid port id");
                }

                let addr = SuiAddress::from_str(module_info[0]).expect("module string is correct");
                (
                    addr,
                    msg,
                    Identifier::new(module_info[1]).unwrap(),
                    Identifier::new("channel_open_confirm").unwrap(),
                    vec![
                        CallArg::Object(ObjectArg::SharedObject {
                            id: module.ibc_store.into(),
                            initial_shared_version: module.ibc_store_initial_seq,
                            mutable: true,
                        }),
                        CallArg::Pure(bcs::to_bytes(&data.channel_id).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.proof_ack).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.proof_height).unwrap()),
                    ],
                )
            }
            Datagram::PacketRecv(data) => {
                let query = SuiQuery::new(&module.sui_client, module.ibc_store.into()).await;

                let res = query
                    .add_param(data.packets[0].destination_channel_id.raw())
                    .call(module.ibc_handler_address.into(), "get_port_id")
                    .await
                    .unwrap();

                if res.len() != 1 {
                    panic!("expected a single encoded connection end")
                }

                let port_id = bcs::from_bytes::<String>(&res[0].0).unwrap();

                let module_info = port_id.split("::").collect::<Vec<&str>>();
                if module_info.len() != 3 {
                    panic!("invalid port id");
                }

                let addr = SuiAddress::from_str(module_info[0]).expect("module string is correct");

                let (
                    source_channels,
                    dest_channels,
                    packet_data,
                    timeout_heights,
                    timeout_timestamps,
                ) = data
                    .packets
                    .iter()
                    .map(|p| {
                        (
                            p.source_channel_id,
                            p.destination_channel_id,
                            p.data.clone(),
                            p.timeout_height,
                            p.timeout_timestamp,
                        )
                    })
                    .collect::<(Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>)>();

                (
                    addr,
                    msg,
                    Identifier::new(module_info[1]).unwrap(),
                    Identifier::new("recv_packet").unwrap(),
                    vec![
                        CallArg::Object(ObjectArg::SharedObject {
                            id: module.ibc_store.into(),
                            initial_shared_version: module.ibc_store_initial_seq,
                            mutable: true,
                        }),
                        CallArg::Object(ObjectArg::SharedObject {
                            id: ObjectID::from_str("0xabb3475a06c67d42a3b380de38e9bc1f04557ad46a43d797a46c5425b81db09b").unwrap(),
                            initial_shared_version: 349179545.into(),
                            mutable: true,
                        }),
                        CallArg::Object(ObjectArg::SharedObject {
                            id: ObjectID::from_str("0x6").unwrap(),
                            initial_shared_version: 1.into(),
                            mutable: false,
                        }),
                        CallArg::Pure(bcs::to_bytes(&source_channels).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&dest_channels).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&packet_data).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&timeout_heights).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&timeout_timestamps).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.proof).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.proof_height).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&fee_recipient).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.relayer_msgs).unwrap()),
                    ],
                )
            }
            _ => todo!(),
        };
        data.push(item);
    }

    data
}

struct SuiQuery<'a> {
    client: &'a SuiClient,
    params: Vec<CallArg>,
}

impl<'a> SuiQuery<'a> {
    async fn new(client: &'a SuiClient, ibc_store_id: ObjectID) -> Self {
        let object_ref = client
            .read_api()
            .get_object_with_options(ibc_store_id, SuiObjectDataOptions::new())
            .await
            .unwrap()
            .object_ref_if_exists()
            .unwrap();
        Self {
            client,
            params: vec![CallArg::Object(ObjectArg::ImmOrOwnedObject(object_ref))],
        }
    }

    fn add_param<T>(mut self, param: T) -> Self
    where
        T: serde::Serialize,
    {
        self.params
            .push(CallArg::Pure(bcs::to_bytes(&param).unwrap()));
        self
    }

    async fn call(
        self,
        package: ObjectID,
        function: &str,
    ) -> Result<Vec<(Vec<u8>, SuiTypeTag)>, String> {
        let mut ptb = ProgrammableTransactionBuilder::new();
        ptb.command(Command::move_call(
            package,
            Identifier::new("ibc").unwrap(),
            Identifier::new(function).unwrap(),
            vec![],
            self.params
                .iter()
                .enumerate()
                .map(|(i, _)| Argument::Input(i as u16))
                .collect(),
        ));

        for arg in self.params {
            ptb.input(arg).unwrap();
        }

        let res = self
            .client
            .read_api()
            .dev_inspect_transaction_block(
                SuiAddress::ZERO,
                TransactionKind::ProgrammableTransaction(ptb.finish()),
                None,
                None,
                None,
            )
            .await
            .unwrap();

        match (res.results, res.error) {
            (Some(res), _) => Ok(res[0].clone().return_values),
            (_, Some(err)) => Err(err),
            _ => panic!("invalid"),
        }
    }
}

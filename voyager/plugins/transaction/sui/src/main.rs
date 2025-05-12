use std::{collections::VecDeque, panic::AssertUnwindSafe, sync::Arc};

use concurrent_keyring::{ConcurrentKeyring, KeyringConfig, KeyringEntry};
use fastcrypto::{hash::HashFunction, traits::Signer};
use ibc_union_spec::{datagram::Datagram, IbcUnion};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use serde::{Deserialize, Serialize};
use shared_crypto::intent::{Intent, IntentMessage};
use sui_sdk::{
    rpc_types::{SuiObjectDataOptions, SuiTransactionBlockResponseOptions},
    types::{
        base_types::{ObjectID, SequenceNumber, SuiAddress},
        crypto::{DefaultHash, SignatureScheme, SuiKeyPair, SuiSignature},
        programmable_transaction_builder::ProgrammableTransactionBuilder,
        signature::GenericSignature,
        transaction::{Argument, CallArg, Command, ObjectArg, Transaction, TransactionData},
        Identifier,
    },
    SuiClientBuilder,
};
use tracing::instrument::{self, WithSubscriber};
use unionlabs::primitives::{encoding::HexPrefixed, Bytes};
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

    pub ibc_handler_address: ObjectID,

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
            .unwrap()
            .data
            .unwrap()
            .owner
            .unwrap()
            .start_version()
            .unwrap();

        Ok(Self {
            chain_id: ChainId::new(chain_id.to_string()),
            ibc_handler_address: config.ibc_handler_address,
            sui_client,
            ibc_store_initial_seq,
            keyring: ConcurrentKeyring::new(
                config.keyring.name,
                config.keyring.keys.into_iter().map(|config| {
                    println!("{}", Bytes::<HexPrefixed>::new(config.value()));
                    let pk =
                        SuiKeyPair::decode(&String::from_utf8(config.value()).unwrap()).unwrap();

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
    pub ibc_handler_address: ObjectID,
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
                                    .collect(), // .collect::<Result<_, _>>()?,
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

    // #[instrument(skip_all, fields(chain_id = %self.chain_id))]
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
                            .unwrap()
                            .data
                            .into_iter()
                            .next()
                            .unwrap();

                        let gas_budget = 20_000_000;
                        let gas_price = self
                            .sui_client
                            .read_api()
                            .get_reference_gas_price()
                            .await
                            .unwrap();

                        println!("GAS PRICE: {}", gas_price);

                        // create the transaction data that will be sent to the network.
                        //
                        let msgs =
                            process_msgs(self.ibc_store, self.ibc_store_initial_seq, msgs.clone())
                                .await;

                        let mut ptb = ProgrammableTransactionBuilder::new();

                        for (_, (_, module, entry_fn, arguments)) in msgs.into_iter().enumerate() {
                            let arguments = arguments
                                .into_iter()
                                .map(|arg| ptb.input(arg).unwrap())
                                .collect();
                            ptb.command(Command::move_call(
                                self.ibc_handler_address,
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

                        // if you would like to verify the signature locally before submission, use this function.
                        // if it fails to verify locally, the transaction will fail to execute in Sui.
                        sui_sig
                            .verify_secure(&intent_msg, sender, SignatureScheme::ED25519)
                            .unwrap();

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
                            .unwrap();

                        println!("{transaction_response:?}");

                        // res.into_inner().transaction_failures

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

    // #[instrument(skip_all, fields(chain_id = %self.chain_id))]
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
    ibc_store: SuiAddress,
    initial_shared_version: SequenceNumber,
    msgs: Vec<Datagram>,
) -> Vec<(Datagram, Identifier, Identifier, Vec<CallArg>)> {
    let mut data = vec![];
    for msg in msgs {
        let item = match msg.clone() {
            Datagram::CreateClient(data) => (
                msg,
                Identifier::new("ibc").unwrap(),
                Identifier::new("create_client").unwrap(),
                vec![
                    CallArg::Object(ObjectArg::SharedObject {
                        id: ibc_store.into(),
                        initial_shared_version,
                        mutable: true,
                    }),
                    CallArg::Pure(bcs::to_bytes(&data.client_type.to_string()).unwrap()),
                    CallArg::Pure(bcs::to_bytes(&data.client_state_bytes).unwrap()),
                    CallArg::Pure(bcs::to_bytes(&data.consensus_state_bytes).unwrap()),
                ],
            ),
            Datagram::UpdateClient(data) => (
                msg,
                Identifier::new("ibc").unwrap(),
                Identifier::new("update_client").unwrap(),
                vec![
                    CallArg::Object(ObjectArg::SharedObject {
                        id: ibc_store.into(),
                        initial_shared_version,
                        mutable: true,
                    }),
                    CallArg::Pure(bcs::to_bytes(&data.client_id).unwrap()),
                    CallArg::Pure(bcs::to_bytes(&data.client_message).unwrap()),
                ],
            ),
            Datagram::ConnectionOpenInit(data) => (
                msg,
                Identifier::new("ibc").unwrap(),
                Identifier::new("connection_open_init").unwrap(),
                vec![
                    CallArg::Object(ObjectArg::SharedObject {
                        id: ibc_store.into(),
                        initial_shared_version,
                        mutable: true,
                    }),
                    CallArg::Pure(bcs::to_bytes(&data.client_id).unwrap()),
                    CallArg::Pure(bcs::to_bytes(&data.counterparty_client_id).unwrap()),
                ],
            ),
            Datagram::ConnectionOpenTry(data) => (
                msg,
                Identifier::new("ibc").unwrap(),
                Identifier::new("connection_open_try").unwrap(),
                vec![
                    CallArg::Object(ObjectArg::SharedObject {
                        id: ibc_store.into(),
                        initial_shared_version,
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
                msg,
                Identifier::new("ibc").unwrap(),
                Identifier::new("connection_open_ack").unwrap(),
                vec![
                    CallArg::Object(ObjectArg::SharedObject {
                        id: ibc_store.into(),
                        initial_shared_version,
                        mutable: true,
                    }),
                    CallArg::Pure(bcs::to_bytes(&data.connection_id).unwrap()),
                    CallArg::Pure(bcs::to_bytes(&data.counterparty_connection_id).unwrap()),
                    CallArg::Pure(bcs::to_bytes(&data.proof_try).unwrap()),
                    CallArg::Pure(bcs::to_bytes(&data.proof_height).unwrap()),
                ],
            ),
            Datagram::ConnectionOpenConfirm(data) => (
                msg,
                Identifier::new("ibc").unwrap(),
                Identifier::new("connection_open_confirm").unwrap(),
                vec![
                    CallArg::Object(ObjectArg::SharedObject {
                        id: ibc_store.into(),
                        initial_shared_version,
                        mutable: true,
                    }),
                    CallArg::Pure(bcs::to_bytes(&data.connection_id).unwrap()),
                    CallArg::Pure(bcs::to_bytes(&data.proof_ack).unwrap()),
                    CallArg::Pure(bcs::to_bytes(&data.proof_height).unwrap()),
                ],
            ),
            _ => todo!(),
        };
        data.push(item);
    }

    data
}

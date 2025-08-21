use std::{
    collections::{hash_map::Entry, HashMap, VecDeque},
    fmt::Debug,
    panic::AssertUnwindSafe,
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use alloy::sol_types::SolValue;
use concurrent_keyring::{ConcurrentKeyring, KeyringConfig, KeyringEntry};
use fastcrypto::{hash::HashFunction, traits::Signer};
use hex_literal::hex;
use ibc_union_spec::{datagram::Datagram, ChannelId, IbcUnion, Packet};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    proc_macros::rpc,
    types::ErrorObject,
    Extensions,
};
use move_core_types_sui::{
    account_address::AccountAddress,
    ident_str,
    language_storage::{StructTag, TypeTag},
};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use shared_crypto::intent::{Intent, IntentMessage};
use sui_sdk::{
    rpc_types::{
        ObjectChange, SuiMoveValue, SuiObjectDataOptions, SuiParsedData,
        SuiTransactionBlockResponse, SuiTransactionBlockResponseOptions, SuiTypeTag,
    },
    types::{
        base_types::{ObjectID, ObjectRef, SequenceNumber, SuiAddress},
        crypto::{DefaultHash, SignatureScheme, SuiKeyPair, SuiSignature},
        dynamic_field::DynamicFieldName,
        programmable_transaction_builder::ProgrammableTransactionBuilder,
        signature::GenericSignature,
        transaction::{
            Argument, CallArg, Command, ObjectArg, ProgrammableTransaction, Transaction,
            TransactionData, TransactionKind,
        },
        Identifier,
    },
    SuiClient, SuiClientBuilder,
};
use tracing::{debug, info, instrument};
use ucs03_zkgm::com::{
    Batch, TokenMetadata, TokenOrderV2, ZkgmPacket, OP_BATCH, OP_TOKEN_ORDER,
    TOKEN_ORDER_KIND_INITIALIZE,
};
use unionlabs::{
    primitives::{encoding::HexPrefixed, Bytes, H256},
    ErrorReporter,
};
use voyager_sdk::{
    anyhow::{self, anyhow},
    hook::SubmitTxHook,
    message::{data::Data, PluginMessage, VoyagerMessage},
    plugin::Plugin,
    primitives::ChainId,
    rpc::{types::PluginInfo, PluginServer},
    serde_json::{self, json},
    vm::{call, noop, pass::PassResult, Op, Visit},
    DefaultCmd, ExtensionsExt, VoyagerClient,
};

use crate::{call::ModuleCall, callback::ModuleCallback};

pub mod call;
pub mod callback;
pub mod data;

const TOKEN_BYTECODE: [&[u8]; 2] = [
    hex!("a11ceb0b060000000a01000e020e1e032c27045308055b5607b101d1010882036006e2034b0aad04050cb2042b000a010d020602070212021302140001020001020701000003000c01000103030c0100010504020006050700000b000100010c010601000211030400030808090102040e0b01010c040f0e01010c05100c030001050307040a050d02080007080400020b020108000b030108000105010f010805010b01010900010800070900020a020a020a020b01010805070804020b030109000b02010900010b0201080001090001060804010b03010800020900050c436f696e4d657461646174610e46554e4749424c455f544f4b454e064f7074696f6e0b5472656173757279436170095478436f6e746578740355726c076164647265737304636f696e0f6372656174655f63757272656e63790b64756d6d795f6669656c640e66756e6769626c655f746f6b656e04696e6974046e6f6e65066f7074696f6e137075626c69635f73686172655f6f626a6563740f7075626c69635f7472616e736665720673656e64657207746f5f75323536087472616e736665720a74785f636f6e746578740375726c0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020520").as_slice(),
    hex!("0a0205046d756e6f0a021e1d7a6b676d20746f6b656e206372656174656420627920766f796167657200020109010000000002140b00070011023307010701070238000a0138010c020c030b0238020b030b012e110638030200").as_slice()
];

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZkgmConfig {
    /// ID of the `wrapped_token_to_t` mapping
    wrapped_token_to_t: ObjectID,
}

#[derive(Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub ibc_handler_address: SuiAddress,

    pub ibc_store: SuiAddress,

    pub graphql_url: String,

    pub sui_client: sui_sdk::SuiClient,

    pub keyring: ConcurrentKeyring<SuiAddress, Arc<SuiKeyPair>>,

    pub ibc_store_initial_seq: SequenceNumber,

    pub zkgm_config: ZkgmConfig,
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> anyhow::Result<Self> {
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
            ibc_handler_address: config.ibc_contract,
            sui_client,
            graphql_url: config.graphql_url,
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
            zkgm_config: config.zkgm_config,
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
    pub graphql_url: String,
    pub ibc_contract: SuiAddress,
    pub ibc_store: SuiAddress,

    pub keyring: KeyringConfig,

    pub zkgm_config: ZkgmConfig,
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
    async fn call(&self, e: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::SubmitTransaction(msgs) => self
                .keyring
                .with(|pk| {
                    let sender = SuiAddress::from(&pk.public());
                    let msgs = msgs.clone();
                    AssertUnwindSafe(async move {
                        let mut ptb = ProgrammableTransactionBuilder::new();

                        process_msgs(self, e.voyager_client()?, &mut ptb, pk, msgs, sender).await;

                        let builder = ptb.finish();
                        let _ = send_transactions(self, pk, builder).await?;
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

#[allow(clippy::type_complexity)]
async fn process_msgs(
    module: &Module,
    _voyager_client: &VoyagerClient,
    ptb: &mut ProgrammableTransactionBuilder,
    pk: &Arc<SuiKeyPair>,
    msgs: Vec<Datagram>,
    fee_recipient: SuiAddress,
) {
    for msg in msgs {
        match msg {
            Datagram::CreateClient(data) => {
                ptb.move_call(
                    module.ibc_handler_address.into(),
                    ident_str!("ibc").into(),
                    ident_str!("create_client").into(),
                    vec![],
                    vec![
                        CallArg::Object(ObjectArg::SharedObject {
                            id: module.ibc_store.into(),
                            initial_shared_version: module.ibc_store_initial_seq,
                            mutable: true,
                        }),
                        CallArg::Pure(bcs::to_bytes(&data.client_type.to_string()).unwrap()),
                        (&data.client_state_bytes.into_vec()).into(),
                        (&data.consensus_state_bytes.into_vec()).into(),
                    ],
                )
                .unwrap();
            }
            Datagram::UpdateClient(data) => {
                ptb.move_call(
                    module.ibc_handler_address.into(),
                    ident_str!("ibc").into(),
                    ident_str!("update_client").into(),
                    vec![],
                    vec![
                        CallArg::Object(ObjectArg::SharedObject {
                            id: module.ibc_store.into(),
                            initial_shared_version: module.ibc_store_initial_seq,
                            mutable: true,
                        }),
                        CallArg::Object(ObjectArg::SharedObject {
                            id: ObjectID::from_str("0x6").unwrap(),
                            initial_shared_version: 1.into(),
                            mutable: false,
                        }),
                        data.client_id.raw().into(),
                        (&data.client_message.into_vec()).into(),
                        CallArg::Pure(H256::<HexPrefixed>::default().into_bytes().to_vec()),
                    ],
                )
                .unwrap();
            }
            Datagram::ConnectionOpenInit(data) => {
                ptb.move_call(
                    module.ibc_handler_address.into(),
                    ident_str!("ibc").into(),
                    ident_str!("connection_open_init").into(),
                    vec![],
                    vec![
                        CallArg::Object(ObjectArg::SharedObject {
                            id: module.ibc_store.into(),
                            initial_shared_version: module.ibc_store_initial_seq,
                            mutable: true,
                        }),
                        CallArg::Pure(bcs::to_bytes(&data.client_id).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.counterparty_client_id).unwrap()),
                    ],
                )
                .unwrap();
            }
            Datagram::ConnectionOpenTry(data) => {
                ptb.move_call(
                    module.ibc_handler_address.into(),
                    ident_str!("ibc").into(),
                    ident_str!("connection_open_try").into(),
                    vec![],
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
                )
                .unwrap();
            }
            Datagram::ConnectionOpenAck(data) => {
                ptb.move_call(
                    module.ibc_handler_address.into(),
                    ident_str!("ibc").into(),
                    ident_str!("connection_open_ack").into(),
                    vec![],
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
                )
                .unwrap();
            }
            Datagram::ConnectionOpenConfirm(data) => {
                ptb.move_call(
                    module.ibc_handler_address.into(),
                    ident_str!("ibc").into(),
                    ident_str!("connection_open_confirm").into(),
                    vec![],
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
                )
                .unwrap();
            }
            Datagram::ChannelOpenInit(data) => {
                let port_id = String::from_utf8(data.port_id.to_vec()).expect("port id is String");

                let module_info = parse_port(&module.graphql_url, &port_id).await;

                ptb.move_call(
                    module_info.latest_address.into(),
                    Identifier::new(module_info.module_name).unwrap(),
                    ident_str!("channel_open_init").into(),
                    vec![],
                    vec![
                        CallArg::Object(ObjectArg::SharedObject {
                            id: module.ibc_store.into(),
                            initial_shared_version: module.ibc_store_initial_seq,
                            mutable: true,
                        }),
                        CallArg::Pure(bcs::to_bytes(&port_id).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.counterparty_port_id).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.connection_id).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.version).unwrap()),
                    ],
                )
                .unwrap();
            }
            Datagram::ChannelOpenTry(data) => {
                let port_id = String::from_utf8(data.port_id.to_vec()).expect("port id is String");

                let module_info = parse_port(&module.graphql_url, &port_id).await;

                ptb.move_call(
                    module_info.latest_address.into(),
                    Identifier::new(module_info.module_name).unwrap(),
                    ident_str!("channel_open_try").into(),
                    vec![],
                    vec![
                        CallArg::Object(ObjectArg::SharedObject {
                            id: module.ibc_store.into(),
                            initial_shared_version: module.ibc_store_initial_seq,
                            mutable: true,
                        }),
                        CallArg::Pure(bcs::to_bytes(&port_id).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.channel.connection_id).unwrap()),
                        CallArg::Pure(
                            bcs::to_bytes(&data.channel.counterparty_channel_id.unwrap()).unwrap(),
                        ),
                        CallArg::Pure(bcs::to_bytes(&data.channel.counterparty_port_id).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.channel.version).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.counterparty_version).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.proof_init).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.proof_height).unwrap()),
                    ],
                )
                .unwrap();
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

                let module_info = parse_port(&module.graphql_url, &port_id).await;

                ptb.move_call(
                    module_info.latest_address.into(),
                    Identifier::new(module_info.module_name).unwrap(),
                    ident_str!("channel_open_ack").into(),
                    vec![],
                    vec![
                        CallArg::Object(ObjectArg::SharedObject {
                            id: module.ibc_store.into(),
                            initial_shared_version: module.ibc_store_initial_seq,
                            mutable: true,
                        }),
                        CallArg::Pure(bcs::to_bytes(&port_id).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.channel_id).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.counterparty_version).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.counterparty_channel_id).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.proof_try).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.proof_height).unwrap()),
                    ],
                )
                .unwrap();
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

                let module_info = parse_port(&module.graphql_url, &port_id).await;
                ptb.move_call(
                    module_info.latest_address.into(),
                    Identifier::new(module_info.module_name).unwrap(),
                    ident_str!("channel_open_confirm").into(),
                    vec![],
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
                .unwrap();
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

                let module_info = parse_port(&module.graphql_url, &port_id).await;

                let store_initial_seq = module
                    .sui_client
                    .read_api()
                    .get_object_with_options(
                        module_info.stores[0].into(),
                        SuiObjectDataOptions::new().with_owner(),
                    )
                    .await
                    .unwrap()
                    .data
                    .expect("object exists on chain")
                    .owner
                    .expect("owner will be present")
                    .start_version()
                    .expect("object is shared, hence it has a start version");

                // let commands = voyager_client
                //     .plugin_client("plugin-name")
                //     .on_packet_recv(data.packets[0].clone())
                //     .await
                //     .map_err(json_rpc_error_to_queue_error)
                //     .unwrap();

                // If the module is ZKGM, then we register the tokens if needed. Otherwise,
                // the registered tokens are returned.
                let coin_ts = register_tokens_if_zkgm(
                    module,
                    ptb,
                    pk,
                    &data.packets[0],
                    &module_info,
                    store_initial_seq,
                )
                .await
                .unwrap();

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
                            0 as u64,
                            p.timeout_timestamp,
                        )
                    })
                    .collect::<(Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>)>();

                let module_name = Identifier::new(module_info.module_name).unwrap();

                let arguments = vec![
                    CallArg::Object(ObjectArg::SharedObject {
                        id: module_info.stores[0].into(),
                        initial_shared_version: store_initial_seq,
                        mutable: true,
                    }),
                    CallArg::Pure(bcs::to_bytes(&source_channels[0]).unwrap()),
                    CallArg::Pure(bcs::to_bytes(&dest_channels[0]).unwrap()),
                    CallArg::Pure(bcs::to_bytes(&packet_data[0]).unwrap()),
                    CallArg::Pure(bcs::to_bytes(&timeout_heights[0]).unwrap()),
                    CallArg::Pure(bcs::to_bytes(&timeout_timestamps[0]).unwrap()),
                ]
                .into_iter()
                .map(|a| ptb.input(a))
                .collect::<Result<_, _>>()
                .unwrap();

                // We start the session by calling `begin_recv`. The returned `session` has no drop nor store,
                // which means, we have to consume it within the same PTB via `end_recv`.
                let session = ptb.command(Command::move_call(
                    module_info.latest_address.into(),
                    module_name.clone(),
                    ident_str!("begin_recv").into(),
                    vec![],
                    arguments,
                ));

                // // SUI code partitions the instructions by the instructions that need coin. And the `recv_packet`
                // // endpoint must be called as many times as the partitions. Since the number of coins will be the
                // // same as the number of partitions, we are calling `recv_packet` based on the number of coins.
                for coin_t in coin_ts {
                    ptb.move_call(
                        module_info.latest_address.into(),
                        module_name.clone(),
                        ident_str!("recv_packet_2").into(),
                        vec![coin_t],
                        vec![
                            CallArg::Object(ObjectArg::SharedObject {
                                id: module.ibc_store.into(),
                                initial_shared_version: module.ibc_store_initial_seq,
                                mutable: true,
                            }),
                            CallArg::Object(ObjectArg::SharedObject {
                                id: module_info.stores[0].into(),
                                initial_shared_version: store_initial_seq,
                                mutable: true,
                            }),
                            CallArg::Object(ObjectArg::SharedObject {
                                id: ObjectID::from_str("0x6").unwrap(),
                                initial_shared_version: 1.into(),
                                mutable: false,
                            }),
                            CallArg::Pure(bcs::to_bytes(&fee_recipient).unwrap()),
                            CallArg::Pure(bcs::to_bytes(&data.relayer_msgs).unwrap()),
                        ],
                    )
                    .unwrap();
                }

                // // `end_recv` is done to consume the `session`, and do the recv commitment. Very important thing
                // // to note here is that, the fact that `session` have to be consumed makes it s.t. if we don't consume
                // // it, this PTB will fail and no partial state will be persisted.
                let arguments = vec![
                    CallArg::Object(ObjectArg::SharedObject {
                        id: module_info.stores[0].into(),
                        initial_shared_version: store_initial_seq,
                        mutable: true,
                    }),
                    CallArg::Object(ObjectArg::SharedObject {
                        id: module.ibc_store.into(),
                        initial_shared_version: module.ibc_store_initial_seq,
                        mutable: true,
                    }),
                    CallArg::Object(ObjectArg::SharedObject {
                        id: ObjectID::from_str("0x6").unwrap(),
                        initial_shared_version: 1.into(),
                        mutable: false,
                    }),
                    CallArg::Pure(bcs::to_bytes(&data.proof).unwrap()),
                    CallArg::Pure(bcs::to_bytes(&data.proof_height).unwrap()),
                    CallArg::Pure(bcs::to_bytes(&fee_recipient).unwrap()),
                    CallArg::Pure(bcs::to_bytes(&data.relayer_msgs).unwrap()),
                ]
                .into_iter()
                .map(|a| ptb.input(a).unwrap())
                .chain(vec![session])
                .collect();

                ptb.command(Command::move_call(
                    module_info.latest_address.into(),
                    module_name.clone(),
                    ident_str!("end_recv").into(),
                    vec![],
                    arguments,
                ));

                // ptb.move_call(
                //     module_info.latest_address.into(),
                //     Identifier::new(module_info.module_name).unwrap(),
                //     ident_str!("recv_packet").into(),
                //     vec![coin_t.unwrap()],
                //     vec![
                //         CallArg::Object(ObjectArg::SharedObject {
                //             id: module.ibc_store.into(),
                //             initial_shared_version: module.ibc_store_initial_seq,
                //             mutable: true,
                //         }),
                //         CallArg::Object(ObjectArg::SharedObject {
                //             id: module_info.stores[0].into(),
                //             initial_shared_version: store_initial_seq,
                //             mutable: true,
                //         }),
                //         CallArg::Object(ObjectArg::SharedObject {
                //             id: ObjectID::from_str("0x6").unwrap(),
                //             initial_shared_version: 1.into(),
                //             mutable: false,
                //         }),
                //         CallArg::Pure(bcs::to_bytes(&source_channels).unwrap()),
                //         CallArg::Pure(bcs::to_bytes(&dest_channels).unwrap()),
                //         CallArg::Pure(bcs::to_bytes(&packet_data).unwrap()),
                //         CallArg::Pure(bcs::to_bytes(&timeout_heights).unwrap()),
                //         CallArg::Pure(bcs::to_bytes(&timeout_timestamps).unwrap()),
                //         CallArg::Pure(bcs::to_bytes(&data.proof).unwrap()),
                //         CallArg::Pure(bcs::to_bytes(&data.proof_height).unwrap()),
                //         CallArg::Pure(bcs::to_bytes(&fee_recipient).unwrap()),
                //         CallArg::Pure(bcs::to_bytes(&data.relayer_msgs).unwrap()),
                //     ],
                // )
                // .unwrap();
            }
            _ => todo!(),
        }
    }
}

fn predict_wrapped_denom(
    path: H256,
    channel: ChannelId,
    base_token: Vec<u8>,
    metadata_image: Vec<u8>,
) -> Vec<u8> {
    let mut buf = vec![];
    bcs::serialize_into(&mut buf, &path).expect("works");
    bcs::serialize_into(&mut buf, &channel.raw()).expect("works");
    buf.extend_from_slice(&base_token);
    buf.extend_from_slice(&metadata_image);

    Keccak256::new().chain_update(buf).finalize().to_vec()
}

#[derive(Deserialize)]
struct SuiFungibleAssetMetadata {
    name: String,
    symbol: String,
    decimals: u8,
    owner: H256,
    icon_url: Option<String>,
    description: String,
}

async fn register_tokens_if_zkgm(
    module: &Module,
    ptb: &mut ProgrammableTransactionBuilder,
    pk: &Arc<SuiKeyPair>,
    packet: &ibc_union_spec::Packet,
    module_info: &ModuleInfo,
    store_initial_seq: SequenceNumber,
) -> anyhow::Result<Vec<TypeTag>> {
    let Ok(zkgm_packet) = ZkgmPacket::abi_decode_params(&packet.data) else {
        return Ok(vec![]);
    };

    let mut coin_ts = vec![];

    match zkgm_packet.instruction.opcode {
        OP_BATCH => {
            let Ok(batch) = Batch::abi_decode_params(&zkgm_packet.instruction.operand) else {
                panic!("impossible");
            };

            let mut base_tokens: HashMap<alloy::primitives::Bytes, TypeTag> = HashMap::new();

            for instr in batch.instructions {
                let Ok(fao) = TokenOrderV2::abi_decode_params(&instr.operand) else {
                    continue;
                };

                let base_token = fao.base_token.clone();

                match base_tokens.entry(base_token) {
                    Entry::Occupied(e) => {
                        coin_ts.push(e.get().clone());
                    }
                    Entry::Vacant(e) => {
                        if let Some(type_tag) = register_token_if_zkgm(
                            module,
                            ptb,
                            pk,
                            packet,
                            &zkgm_packet,
                            fao,
                            module_info,
                            store_initial_seq,
                        )
                        .await?
                        {
                            coin_ts.push(type_tag.clone());
                            e.insert(type_tag);
                        }
                    }
                }
            }
        }
        OP_TOKEN_ORDER => {
            let fao = TokenOrderV2::abi_decode_params(&zkgm_packet.instruction.operand)
                .expect("impossible");
            let mut coin_ts = vec![];
            if let Some(type_tag) = register_token_if_zkgm(
                module,
                ptb,
                pk,
                packet,
                &zkgm_packet,
                fao,
                module_info,
                store_initial_seq,
            )
            .await?
            {
                coin_ts.push(type_tag);
            }
        }
        _ => {}
    }

    Ok(coin_ts)
}

/// Deploy and register the token if needed in `ZKGM`
async fn register_token_if_zkgm(
    module: &Module,
    ptb: &mut ProgrammableTransactionBuilder,
    pk: &Arc<SuiKeyPair>,
    packet: &ibc_union_spec::Packet,
    zkgm_packet: &ZkgmPacket,
    fao: TokenOrderV2,
    module_info: &ModuleInfo,
    store_initial_seq: SequenceNumber,
) -> anyhow::Result<Option<TypeTag>> {
    let (metadata_image, coin_metadata) = if fao.kind == TOKEN_ORDER_KIND_INITIALIZE {
        // TODO(aeryz): we could drop this packet as well since we know that its gonna fail
        let Ok(metadata) = TokenMetadata::abi_decode_params(&fao.metadata) else {
            return Ok(None);
        };

        println!("{}", metadata.initializer);

        // TODO(aeryz): we can also drop here
        let sui_metadata: SuiFungibleAssetMetadata =
            bcs::from_bytes(&metadata.initializer).unwrap();

        if sui_metadata.owner != H256::<HexPrefixed>::default() {
            return Ok(None);
        }

        (
            Keccak256::new()
                .chain_update(&fao.metadata)
                .finalize()
                .to_vec(),
            Some(sui_metadata),
        )
    }
    // TODO(aeryz): figure out how this will work
    // else if fao.kind == FUNGIBLE_ASSET_METADATA_TYPE_IMAGE {
    //     // otherwise, the metadata must be an image
    //     if fao.metadata.len() != 32 {
    //         return Err(anyhow!("invalid metadata"));
    //     }

    //     (fao.metadata.into(), None)
    // }
    else {
        // This means the transfer is an unwrap. Hence the `quote_token` must already be in the form `address::module::name`
        // which defines the coin type `T`.
        let quote_token = String::from_utf8(fao.quote_token.into())
            .map_err(|_| anyhow!("in the unwrap case, the quote token must be a utf8 string"))?;
        let fields: Vec<&str> = quote_token.split("::").collect();
        if fields.len() != 3 {
            panic!("a registered token must be always in `address::module_name::name` form");
        }

        return Ok(Some(
            StructTag {
                address: AccountAddress::from_str(fields[0]).expect("address is valid"),
                module: Identifier::new(fields[1]).expect("module name is valid"),
                name: Identifier::new(fields[2]).expect("name is valid"),
                type_params: vec![],
            }
            .into(),
        ));
    };

    let wrapped_token = predict_wrapped_denom(
        zkgm_packet.path.to_le_bytes().into(),
        packet.destination_channel_id,
        fao.base_token.to_vec(),
        metadata_image,
    );

    // A wrapped token is only registered once, and once it's being received in the SUI side.
    // `wrapped_token` is set to the given coin type. If there's already a coin type with this
    // `wrapped_token`, we have to use that.
    if let Some(wrapped_token_t) = get_registered_wrapped_token(module, &wrapped_token).await? {
        return Ok(Some(wrapped_token_t));
    }

    let Some(coin_metadata) = coin_metadata else {
        return Err(anyhow!(
            "the coin is going to be received for the first time, so the metadata must be provided"
        ));
    };
    let (treasury_ref, metadata_ref, coin_t) =
        publish_new_coin(module, pk, coin_metadata.decimals).await?;

    // updating name, symbol, icon_url and the description since we don't have these in the published binary right now
    // TODO(aeryz): we should generate the move binary to contain the necessary data and don't do these calls
    call_coin_setter(
        ptb,
        "update_name",
        treasury_ref,
        metadata_ref,
        coin_t.clone(),
        coin_metadata.name,
    )
    .await?;

    call_coin_setter(
        ptb,
        "update_symbol",
        treasury_ref,
        metadata_ref,
        coin_t.clone(),
        coin_metadata.symbol,
    )
    .await?;

    call_coin_setter(
        ptb,
        "update_description",
        treasury_ref,
        metadata_ref,
        coin_t.clone(),
        coin_metadata.description,
    )
    .await?;

    if let Some(icon_url) = coin_metadata.icon_url {
        call_coin_setter(
            ptb,
            "update_icon_url",
            treasury_ref,
            metadata_ref,
            coin_t.clone(),
            icon_url,
        )
        .await?;
    }

    // We are finally registering the token before calling the recv
    register_capability(
        ptb,
        module_info,
        store_initial_seq,
        treasury_ref,
        metadata_ref,
        coin_t.clone(),
    )
    .await?;

    Ok(Some(coin_t))
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

pub struct ModuleInfo {
    pub original_address: SuiAddress,
    pub latest_address: SuiAddress,
    pub module_name: String,
    pub stores: Vec<SuiAddress>,
}

// original_address::module_name::store_address
// TODO(aeryz): we can also choose to include store_name here
pub async fn parse_port(graphql_url: &str, port_id: &str) -> ModuleInfo {
    let module_info = port_id.split("::").collect::<Vec<&str>>();
    if module_info.len() < 3 {
        panic!("invalid port id");
    }

    let original_address = module_info[0].parse().unwrap();

    let query = json!({
        "query": "query ($address: SuiAddress) { latestPackage(address: $address) { address } }",
        "variables": { "address": original_address }
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(graphql_url)
        .header("Content-Type", "application/json")
        .body(query.to_string())
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let v: serde_json::Value = serde_json::from_str(resp.as_str()).unwrap();
    let latest_address =
        SuiAddress::from_str(v["data"]["latestPackage"]["address"].as_str().unwrap()).unwrap();

    ModuleInfo {
        original_address,
        latest_address,
        module_name: module_info[1].to_string(),
        stores: module_info[2..]
            .iter()
            .map(|s| s.parse().unwrap())
            .collect(),
    }
}

pub async fn send_transactions(
    module: &Module,
    pk: &Arc<SuiKeyPair>,
    ptb: ProgrammableTransaction,
) -> RpcResult<SuiTransactionBlockResponse> {
    let sender = SuiAddress::from(&pk.public());
    let gas_coin = module
        .sui_client
        .coin_read_api()
        .get_coins(sender, None, None, None)
        .await
        .expect("sender is broke")
        .data
        .into_iter()
        .next()
        .expect("sender has a gas token");

    let gas_budget = 180_000_000; //TODO: change it later
    let gas_price = module
        .sui_client
        .read_api()
        .get_reference_gas_price()
        .await
        .map_err(|e| {
            ErrorObject::owned(
                -1,
                ErrorReporter(e).with_message("error fetching the reference gas price"),
                None::<()>,
            )
        })?;
    println!(
        "{}",
        serde_json::to_string(
            &module
                .sui_client
                .read_api()
                .dev_inspect_transaction_block(
                    sender,
                    TransactionKind::ProgrammableTransaction(ptb.clone()),
                    None,
                    None,
                    None
                )
                .await
                .unwrap()
        )
        .unwrap()
    );

    let tx_data = TransactionData::new_programmable(
        sender,
        vec![gas_coin.object_ref()],
        ptb,
        gas_budget,
        gas_price,
    );

    /*
        GasCostSummary { computation_cost: 693000000, storage_cost: 35013200, storage_rebate: 15506964, non_refundable_storage_fee: 156636 }


    693_000_000
    35_013_200
    15_506_964
        */

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

    let transaction_response = module
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
        .await;

    info!("{transaction_response:?}");

    let transaction_response = transaction_response.map_err(|e| {
        ErrorObject::owned(
            -1,
            ErrorReporter(e).with_message("error executing a tx"),
            None::<()>,
        )
    })?;

    Ok(transaction_response)
}

async fn get_registered_wrapped_token(
    module: &Module,
    wrapped_token: &[u8],
) -> anyhow::Result<Option<TypeTag>> {
    if let Some(wrapped_token_t) = module
        .sui_client
        .read_api()
        .get_dynamic_field_object(
            module.zkgm_config.wrapped_token_to_t,
            DynamicFieldName {
                type_: TypeTag::Vector(Box::new(TypeTag::U8)),
                value: serde_json::to_value(&wrapped_token).expect("serde will work"),
            },
        )
        .await
        .map_err(|_| anyhow!("wrapped_token_to_t is expected to return some data"))?
        .data
    {
        match wrapped_token_t.content.expect("content always exists") {
            SuiParsedData::MoveObject(object) => {
                let SuiMoveValue::String(field_value) = object
                    .fields
                    .field_value("value")
                    .expect("token has a `value` field")
                else {
                    panic!("token must have the type `String`, this voyager might be outdated");
                };

                debug!("the token is already registered");

                let fields: Vec<&str> = field_value.split("::").collect();
                if fields.len() != 3 {
                    panic!(
                        "a registered token must be always in `address::module_name::name` form"
                    );
                }

                return Ok(Some(
                    StructTag {
                        address: AccountAddress::from_str(fields[0]).expect("address is valid"),
                        module: Identifier::new(fields[1]).expect("module name is valid"),
                        name: Identifier::new(fields[2]).expect("name is valid"),
                        type_params: vec![],
                    }
                    .into(),
                ));
            }
            SuiParsedData::Package(_) => panic!("this should never be a package"),
        }
    } else {
        Ok(None)
    }
}

async fn publish_new_coin(
    module: &Module,
    pk: &Arc<SuiKeyPair>,
    decimals: u8,
) -> anyhow::Result<(ObjectRef, ObjectRef, TypeTag)> {
    // There is no wrapped token
    let mut bytecode = TOKEN_BYTECODE[0].to_vec();
    // 31 because it will be followed by a u8 (decimals)
    bytecode.extend_from_slice(&[0; 31]);
    bytecode.extend_from_slice(&decimals.to_be_bytes());
    bytecode.extend_from_slice(TOKEN_BYTECODE[1]);

    let mut ptb = ProgrammableTransactionBuilder::new();

    let res = ptb.command(Command::Publish(
        vec![bytecode],
        vec![
            ObjectID::from_str("0x1").unwrap(),
            ObjectID::from_str("0x2").unwrap(),
        ],
    ));

    let arg = ptb
        .input(CallArg::Pure(
            bcs::to_bytes(&SuiAddress::from(&pk.public())).unwrap(),
        ))
        .unwrap();
    let _ = ptb.command(Command::TransferObjects(vec![res.clone()], arg));

    let transaction_response = send_transactions(module, pk, ptb.finish()).await.unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;
    let object_changes = module
        .sui_client
        .read_api()
        .get_transaction_with_options(
            transaction_response.digest,
            SuiTransactionBlockResponseOptions::new().with_object_changes(),
        )
        .await
        .unwrap()
        .object_changes
        .unwrap();
    let (treasury_ref, coin_t) = object_changes
        .iter()
        .find_map(|o| match &o {
            ObjectChange::Created {
                object_type: StructTag {
                    name, type_params, ..
                },
                ..
            } => {
                if name.as_ident_str() == ident_str!("TreasuryCap") {
                    Some((o.object_ref(), type_params[0].clone()))
                } else {
                    None
                }
            }
            _ => None,
        })
        .unwrap();

    let metadata_ref = object_changes
        .iter()
        .find_map(|o| match &o {
            ObjectChange::Created {
                object_type: StructTag { name, .. },
                ..
            } => {
                if name.as_ident_str() == ident_str!("CoinMetadata") {
                    Some(o.object_ref())
                } else {
                    None
                }
            }
            _ => None,
        })
        .unwrap();

    Ok((treasury_ref, metadata_ref, coin_t))
}

async fn call_coin_setter<T: Serialize>(
    ptb: &mut ProgrammableTransactionBuilder,
    function: &'static str,
    treasury_ref: ObjectRef,
    metadata_ref: ObjectRef,
    coin_t: TypeTag,
    data: T,
) -> anyhow::Result<()> {
    let arguments: Vec<Argument> = [
        CallArg::Object(ObjectArg::ImmOrOwnedObject(treasury_ref)),
        CallArg::Object(ObjectArg::SharedObject {
            id: metadata_ref.0,
            initial_shared_version: metadata_ref.1,
            mutable: true,
        }),
        CallArg::Pure(bcs::to_bytes(&data).unwrap()),
    ]
    .into_iter()
    .map(|arg| ptb.input(arg).unwrap())
    .collect();

    let _ = ptb.command(Command::move_call(
        ObjectID::from_str("0x2").unwrap(),
        ident_str!("coin").into(),
        ident_str!(function).into(),
        vec![coin_t],
        arguments.to_vec(),
    ));

    Ok(())
}

async fn register_capability(
    ptb: &mut ProgrammableTransactionBuilder,
    module_info: &ModuleInfo,
    initial_seq: SequenceNumber,
    treasury_ref: ObjectRef,
    metadata_ref: ObjectRef,
    coin_t: TypeTag,
) -> anyhow::Result<()> {
    let arguments = [
        ptb.input(CallArg::Object(ObjectArg::SharedObject {
            id: module_info.stores[0].into(),
            initial_shared_version: initial_seq,
            mutable: true,
        }))
        .unwrap(),
        ptb.input(CallArg::Object(ObjectArg::ImmOrOwnedObject(treasury_ref)))
            .unwrap(),
        ptb.input(CallArg::Object(ObjectArg::SharedObject {
            id: metadata_ref.0,
            initial_shared_version: metadata_ref.1,
            mutable: true,
        }))
        .unwrap(),
        // owner is 0x0
        ptb.input(CallArg::Pure(
            H256::<HexPrefixed>::default().into_bytes().to_vec(),
        ))
        .unwrap(),
    ];
    ptb.command(Command::move_call(
        module_info.latest_address.into(),
        Identifier::new(module_info.module_name.clone()).unwrap(),
        ident_str!("register_capability").into(),
        vec![coin_t.clone()],
        arguments.to_vec(),
    ));

    Ok(())
}

/*
╭───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╮
│ Object Changes                                                                                                                                    │
├───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Created Objects:                                                                                                                                  │
│  ┌──                                                                                                                                              │
│  │ ObjectID: 0x3f40f5ee083b947a74dfbf0071e8ae051d6a09366c065b720930f6ccae86f75e                                                                   │
│  │ Sender: 0x232a4f7eb4c2abf5061316704373fd4bffbd297729406d9d04012931405f590b                                                                     │
│  │ Owner: Account Address ( 0x232a4f7eb4c2abf5061316704373fd4bffbd297729406d9d04012931405f590b )                                                  │
│  │ ObjectType: 0x2::package::UpgradeCap                                                                                                           │
│  │ Version: 349179655                                                                                                                             │
│  │ Digest: 6KSt3SNKKfFLAqupM5DP3Q5Wxo8QA3AEq1fLamqHTvn5                                                                                           │
│  └──                                                                                                                                              │
│  ┌──                                                                                                                                              │
│  │ ObjectID: 0x50fe8c5faed80bef58c6a6243689f03a36000852f5aed8efbff50278ae887a71                                                                   │
│  │ Sender: 0x232a4f7eb4c2abf5061316704373fd4bffbd297729406d9d04012931405f590b                                                                     │
│  │ Owner: Shared( 349179655 )                                                                                                                     │
│  │ ObjectType: 0x3b305eaf161580056178f6e375624a406fcad0eebb99ebb802788d6c47e2b367::ibc::IBCStore                                                  │
│  │ Version: 349179655                                                                                                                             │
│  │ Digest: ANSraQpcpxLMee2uP3Uq5kXxEgh3z7BsT6z3EwfbquYU                                                                                           │
│  └──                                                                                                                                              │
│ Mutated Objects:                                                                                                                                  │
│  ┌──                                                                                                                                              │
│  │ ObjectID: 0xe1fbe13ed5e81d9dba74e2819c6a4cfaba6be25bdadb7ac5321def4eaab5bf09                                                                   │
│  │ Sender: 0x232a4f7eb4c2abf5061316704373fd4bffbd297729406d9d04012931405f590b                                                                     │
│  │ Owner: Account Address ( 0x232a4f7eb4c2abf5061316704373fd4bffbd297729406d9d04012931405f590b )                                                  │
│  │ ObjectType: 0x2::coin::Coin<0x2::sui::SUI>                                                                                                     │
│  │ Version: 349179655                                                                                                                             │
│  │ Digest: 7y8JWpcE4eirC8CHKdGeJQmYX7wny1yq3VrhJLEsXGvQ                                                                                           │
│  └──                                                                                                                                              │
│ Published Objects:                                                                                                                                │
│  ┌──                                                                                                                                              │
│  │ PackageID: 0x3b305eaf161580056178f6e375624a406fcad0eebb99ebb802788d6c47e2b367                                                                  │
│  │ Version: 1                                                                                                                                     │
│  │ Digest: 44KR1h95cn5gFYF4QZDhTrWg3jytXgiwxnFrxZqbtgBq                                                                                           │
│  │ Modules: bcs_utils, channel, commitment, connection_end, create_lens_client_event, ethabi, groth16_verifier, height, ibc, light_client, packet │
│  └──                                                                                                                                              │
╰─

commitments: 0xaef6807dd959db193c6dd56b54aea5ebab70e93acf7053f231014c6f93f0ca77
*/

#[rpc(server, client)]
trait SuiIbcAppPlugin {
    #[method(name = "onPacketRecv")]
    async fn on_packet_recv(&self, packet: Packet) -> RpcResult<Vec<Command>>;
}

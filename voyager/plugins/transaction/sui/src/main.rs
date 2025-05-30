use std::{
    collections::VecDeque, fmt::Debug, panic::AssertUnwindSafe, str::FromStr, sync::Arc,
    time::Duration,
};

use alloy::sol_types::SolValue;
use concurrent_keyring::{ConcurrentKeyring, KeyringConfig, KeyringEntry};
use fastcrypto::{hash::HashFunction, traits::Signer};
use hex_literal::hex;
use ibc_union_spec::{datagram::Datagram, ChannelId, IbcUnion};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use move_core_types_sui::{
    account_address::AccountAddress,
    ident_str,
    identifier::Identifier as MoveIdentifier,
    language_storage::{StructTag, TypeTag},
};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use shared_crypto::intent::{Intent, IntentMessage};
// use sui_json_rpc_api::MoveUtilsClient;
use sui_sdk::{
    rpc_types::{
        ObjectChange, SuiData, SuiObjectDataOptions, SuiTransactionBlockResponse,
        SuiTransactionBlockResponseOptions, SuiTypeTag,
    },
    types::{
        base_types::{ObjectID, SequenceNumber, SuiAddress},
        crypto::{DefaultHash, SignatureScheme, SuiKeyPair, SuiSignature},
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
use tracing::{info, instrument};
use ucs03_zkgm::com::{FungibleAssetOrder, ZkgmPacket};
use unionlabs::{
    primitives::{encoding::HexPrefixed, Bytes, U256},
    ErrorReporter,
};
use voyager_sdk::{
    anyhow,
    hook::SubmitTxHook,
    message::{data::Data, PluginMessage, VoyagerMessage},
    plugin::Plugin,
    primitives::ChainId,
    rpc::{types::PluginInfo, PluginServer},
    vm::{call, noop, pass::PassResult, Op, Visit},
    DefaultCmd,
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
    // let sui_client = SuiClientBuilder::default()
    //     .build("https://fullnode.testnet.sui.io")
    //     .await
    //     .unwrap();

    // let res = sui_client
    //     .http()
    //     .get_normalized_move_function(
    //         ObjectID::from_str(
    //             "0xf02e69bb76b03820e27ddb2908f8dace2efa3d69924ee1842ddbf3df1287b917",
    //         )
    //         .unwrap(),
    //         "zkgm_relay".into(),
    //         "recv_packet".into(),
    //     )
    //     .await
    //     .unwrap();

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
                        let msgs = process_msgs(self, pk, msgs, sender).await;

                        let mut ptb = ProgrammableTransactionBuilder::new();

                        for (_, (contract_addr, _, module, entry_fn, arguments, type_args)) in
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
                                type_args,
                                arguments,
                            ));
                        }

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

// module: Identifier,
// function: Identifier,
// type_arguments: Vec<TypeTag>,

#[allow(clippy::type_complexity)]
async fn process_msgs(
    module: &Module,
    pk: &Arc<SuiKeyPair>,
    msgs: Vec<Datagram>,
    fee_recipient: SuiAddress,
) -> Vec<(
    SuiAddress,
    Datagram,
    Identifier,
    Identifier,
    Vec<CallArg>,
    Vec<TypeTag>,
)> {
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
                vec![],
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
                vec![],
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
                vec![],
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
                vec![],
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
                vec![],
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
                vec![],
            ),
            Datagram::ChannelOpenInit(data) => {
                let port_id = String::from_utf8(data.port_id.to_vec()).expect("port id is String");

                let module_info = parse_port(&module.sui_client, &port_id).await;

                (
                    module_info.latest_address,
                    msg,
                    Identifier::new(module_info.module_name).unwrap(),
                    Identifier::new("channel_open_init").unwrap(),
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
                    vec![],
                )
            }
            Datagram::ChannelOpenTry(data) => {
                let port_id = String::from_utf8(data.port_id.to_vec()).expect("port id is String");

                let module_info = parse_port(&module.sui_client, &port_id).await;

                (
                    module_info.latest_address,
                    msg,
                    Identifier::new(module_info.module_name).unwrap(),
                    Identifier::new("channel_open_try").unwrap(),
                    vec![
                        CallArg::Object(ObjectArg::SharedObject {
                            id: module.ibc_store.into(),
                            initial_shared_version: module.ibc_store_initial_seq,
                            mutable: true,
                        }),
                        CallArg::Pure(bcs::to_bytes(&port_id).unwrap()),
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
                    vec![],
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

                let module_info = parse_port(&module.sui_client, &port_id).await;

                (
                    module_info.latest_address,
                    msg,
                    Identifier::new(module_info.module_name).unwrap(),
                    Identifier::new("channel_open_ack").unwrap(),
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
                    vec![],
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

                let module_info = parse_port(&module.sui_client, &port_id).await;
                (
                    module_info.latest_address,
                    msg,
                    Identifier::new(module_info.module_name).unwrap(),
                    Identifier::new("channel_open_confirm").unwrap(),
                    vec![
                        CallArg::Object(ObjectArg::SharedObject {
                            id: module.ibc_store.into(),
                            initial_shared_version: module.ibc_store_initial_seq,
                            mutable: true,
                        }),
                        CallArg::Pure(bcs::to_bytes(&port_id).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.channel_id).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.proof_ack).unwrap()),
                        CallArg::Pure(bcs::to_bytes(&data.proof_height).unwrap()),
                    ],
                    vec![],
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

                let module_info: ModuleInfo = parse_port(&module.sui_client, &port_id).await;

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

                register_token_if_zkgm(
                    &module,
                    pk,
                    &data.packets[0],
                    &module_info,
                    store_initial_seq,
                )
                .await;

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

                println!(
                    "{:?}, {:?}, {:?}, {:?}, {:?}",
                    source_channels,
                    dest_channels,
                    packet_data,
                    timeout_heights,
                    timeout_timestamps
                );

                (
                    module_info.latest_address,
                    msg,
                    Identifier::new(module_info.module_name).unwrap(),
                    Identifier::new("recv_packet").unwrap(),
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
                    vec![TypeTag::Struct(Box::new(StructTag {
                        address: AccountAddress::from_str(
                            "0xacc51178ffc547cdfa36a8ab4a6ae3823edaa8f07ff9177d9d520aad080b28fd",
                        )
                        .unwrap(),
                        module: MoveIdentifier::new("fungible_token").unwrap(),
                        name: MoveIdentifier::new("FUNGIBLE_TOKEN").unwrap(),
                        type_params: vec![],
                    }))],
                )
            }
            _ => todo!(),
        };
        data.push(item);
    }

    data
}

fn predict_wrapped_denom(path: U256, channel: ChannelId, base_token: Vec<u8>) -> Vec<u8> {
    let mut buf = vec![];
    bcs::serialize_into(&mut buf, &path).expect("works");
    bcs::serialize_into(&mut buf, &channel.raw()).expect("works");
    buf.extend_from_slice(&base_token);

    Keccak256::new().chain_update(buf).finalize().to_vec()
}

async fn register_token_if_zkgm(
    module: &Module,
    pk: &Arc<SuiKeyPair>,
    packet: &ibc_union_spec::Packet,
    module_info: &ModuleInfo,
    store_initial_seq: SequenceNumber,
) {
    let Ok(zkgm_packet) = ZkgmPacket::abi_decode_params(&packet.data) else {
        return;
    };

    let Ok(fao) = FungibleAssetOrder::abi_decode_params(&zkgm_packet.instruction.operand) else {
        return;
    };

    let wrapped_token = predict_wrapped_denom(
        fao.base_token_path.into(),
        packet.destination_channel_id,
        fao.base_token.to_vec(),
    );

    if fao.quote_token != wrapped_token {
        return;
    }

    let mut bytecode = TOKEN_BYTECODE[0].to_vec();
    bytecode.extend_from_slice(&hex!(
        "0000000000000000000000000000000000000000000000000000000000000001"
    ));
    bytecode.extend_from_slice(TOKEN_BYTECODE[1]);

    let mut ptb = ProgrammableTransactionBuilder::new();

    let res = ptb.command(Command::Publish(
        vec![bytecode],
        vec![
            ObjectID::from_str(
                "0x0000000000000000000000000000000000000000000000000000000000000001",
            )
            .unwrap(),
            ObjectID::from_str(
                "0x0000000000000000000000000000000000000000000000000000000000000002",
            )
            .unwrap(),
        ],
    ));
    let arg = ptb
        .input(CallArg::Pure(
            bcs::to_bytes(&SuiAddress::from(&pk.public())).unwrap(),
        ))
        .unwrap();
    let _ = ptb.command(Command::TransferObjects(vec![res], arg));

    let transaction_response = send_transactions(module, pk, ptb.finish()).await.unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;
    let (treasury_ref, coin_t) = module
        .sui_client
        .read_api()
        .get_transaction_with_options(
            transaction_response.digest,
            SuiTransactionBlockResponseOptions::new().with_object_changes(),
        )
        .await
        .unwrap()
        .object_changes
        .unwrap()
        .into_iter()
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

    let mut ptb = ProgrammableTransactionBuilder::new();

    let arguments = [
        ptb.input(CallArg::Object(ObjectArg::SharedObject {
            id: module_info.stores[0].into(),
            initial_shared_version: store_initial_seq,
            mutable: true,
        }))
        .unwrap(),
        ptb.input(CallArg::Object(ObjectArg::ImmOrOwnedObject(treasury_ref)))
            .unwrap(),
    ]
    .to_vec();

    ptb.command(Command::move_call(
        module_info.latest_address.into(),
        Identifier::new(module_info.module_name.clone()).unwrap(),
        ident_str!("register_capability").into(),
        vec![coin_t],
        arguments,
    ));
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

pub async fn parse_port(sui_client: &SuiClient, port_id: &str) -> ModuleInfo {
    let module_info = port_id.split("::").collect::<Vec<&str>>();
    if module_info.len() < 4 {
        panic!("invalid port id");
    }

    let upgrade_cap_address: SuiAddress = module_info[2].parse().unwrap();

    let sui_sdk::rpc_types::SuiMoveValue::Address(addr) = sui_client
        .read_api()
        .get_object_with_options(
            upgrade_cap_address.into(),
            SuiObjectDataOptions::new().with_content(),
        )
        .await
        .unwrap()
        .into_object()
        .unwrap()
        .content
        .unwrap()
        .try_into_move()
        .unwrap()
        .fields
        .field_value("package")
        .unwrap()
    else {
        panic!("this can't be the case");
    };

    ModuleInfo {
        original_address: module_info[0].parse().unwrap(),
        latest_address: addr,
        module_name: module_info[1].to_string(),
        stores: module_info[3..]
            .into_iter()
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

    let gas_budget = 200_000_000; //TODO: change it later
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

    let tx_data = TransactionData::new_programmable(
        sender,
        vec![gas_coin.object_ref()],
        ptb,
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
        .await
        .map_err(|e| {
            ErrorObject::owned(
                -1,
                ErrorReporter(e).with_message("error executing a tx"),
                None::<()>,
            )
        })?;

    Ok(transaction_response)
}

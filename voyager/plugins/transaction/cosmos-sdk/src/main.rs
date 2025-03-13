// #![warn(clippy::unwrap_used)]
#![feature(if_let_guard)]

use std::{
    collections::{HashMap, HashSet, VecDeque},
    num::NonZeroU32,
    panic::AssertUnwindSafe,
    sync::LazyLock,
};

use cometbft_rpc::rpc_types::GrpcAbciQueryError;
use concurrent_keyring::{ConcurrentKeyring, KeyringConfig, KeyringEntry};
use cosmos_client::{
    gas::GasConfig,
    rpc::{Rpc, RpcT},
    wallet::{LocalSigner, WalletT},
    BroadcastTxCommitError, FetchAccountInfoError, SimulateTxError, TxClient,
};
use ibc_union::ContractErrorKind;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use prost::Message;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, info_span, instrument, trace, warn};
use unionlabs::{
    self,
    bech32::Bech32,
    google::protobuf::any::mk_any,
    option_unwrap,
    primitives::{Bytes, H160, H256},
    ErrorReporter,
};
use voyager_message::{
    core::ChainId,
    data::Data,
    hook::SubmitTxHook,
    module::{PluginInfo, PluginServer},
    vm::{call, noop, pass::PassResult, seq, BoxDynError, Op, Visit},
    DefaultCmd, Plugin, PluginMessage, VoyagerMessage, FATAL_JSONRPC_ERROR_CODE,
};

use crate::{
    call::{IbcMessage, ModuleCall},
    callback::ModuleCallback,
};

pub mod call;
pub mod callback;
pub mod data;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug)]
pub struct Module {
    pub chain_id: ChainId,
    pub ibc_host_contract_address: Bech32<H256>,
    pub keyring: ConcurrentKeyring<Bech32<H160>, LocalSigner>,
    pub rpc: Rpc,
    pub gas_config: GasConfig,
    pub bech32_prefix: String,
    pub fatal_errors: HashMap<(String, NonZeroU32), Option<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId,
    pub ibc_host_contract_address: Bech32<H256>,
    pub keyring: KeyringConfig,
    pub rpc_url: String,
    pub gas_config: GasConfig,
    /// A list of (codespace, code) tuples that are to be considered non-recoverable.
    #[serde(default)]
    pub fatal_errors: HashMap<(String, NonZeroU32), Option<String>>,
}

const FATAL_ERRORS: &[(&str, NonZeroU32)] = &[
    // https://github.com/cosmos/ibc-go/blob/main/modules/light-clients/08-wasm/types/errors.go
    ("08-wasm", option_unwrap!(NonZeroU32::new(4))),
    // https://github.com/cosmos/ibc-go/blob/7f89b7dd8796eca1bfe07f8a7833f3ce2d7a8e04/modules/core/02-client/types/errors.go
    ("client", option_unwrap!(NonZeroU32::new(4))),
];

static ACCOUNT_SEQUENCE_ERRORS: LazyLock<HashSet<(&str, NonZeroU32)>> = LazyLock::new(|| {
    [
        // ("sdk", option_unwrap!(NonZeroU32::new(6))),
        ("sdk", option_unwrap!(NonZeroU32::new(32))),
    ]
    .into_iter()
    .collect()
});

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        let rpc = Rpc::new(config.rpc_url).await?;

        let chain_id = rpc.client().status().await?.node_info.network.to_string();

        let bech32_prefix = rpc
            .client()
            .grpc_abci_query::<_, protos::cosmos::auth::v1beta1::Bech32PrefixResponse>(
                "/cosmos.auth.v1beta1.Query/Bech32Prefix",
                &protos::cosmos::auth::v1beta1::Bech32PrefixRequest {},
                None,
                false,
            )
            .await?
            .into_result()?
            .unwrap()
            .bech32_prefix;

        Ok(Self {
            ibc_host_contract_address: config.ibc_host_contract_address,
            keyring: ConcurrentKeyring::new(
                config.keyring.name,
                config.keyring.keys.into_iter().map(|entry| {
                    let signer =
                        LocalSigner::new(entry.value().try_into().unwrap(), bech32_prefix.clone());

                    KeyringEntry {
                        name: entry.name(),
                        address: signer.address(),
                        signer,
                    }
                }),
            ),
            rpc,
            chain_id: ChainId::new(chain_id),
            gas_config: config.gas_config,
            bech32_prefix,
            fatal_errors: config
                .fatal_errors
                .into_iter()
                .chain(
                    FATAL_ERRORS
                        .iter()
                        .map(|(codespace, code)| (((*codespace).to_owned(), *code), None)),
                )
                .collect(),
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

fn plugin_name(chain_id: &ChainId) -> String {
    pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

    format!("{PLUGIN_NAME}/{}", chain_id)
}

impl Module {
    fn plugin_name(&self) -> String {
        plugin_name(&self.chain_id)
    }

    pub async fn do_send_transaction(
        &self,
        msgs: Vec<IbcMessage>,
    ) -> Option<Result<(), BroadcastTxCommitError>> {
        self.keyring
            .with(|signer| {
                let msgs = msgs.clone();

                trace!(?msgs);

                // TODO: Figure out a way to thread this value through
                let memo = format!("Voyager {}", env!("CARGO_PKG_VERSION"));

                let ibc_host_contract_address = self.ibc_host_contract_address.clone();
                let msgs = process_msgs(msgs, signer, ibc_host_contract_address);

                let msgs = msgs
                    .into_iter()
                    .filter_map(|msg| match msg {
                        Ok(msg) => Some(msg),
                        Err(err) => {
                            error!("invalid msg: {}", ErrorReporter(err));
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                let tx_client = TxClient::new(signer, &self.rpc, &self.gas_config);

                let batch_size = msgs.len();
                let msg_names = msgs.iter().map(|x| x.0.name()).collect::<Vec<_>>();

                AssertUnwindSafe(async move {
                    if msgs.is_empty() {
                        info!("no msgs left to submit after filtering out invalid msgs");
                        return Ok(());
                    }

                    match tx_client
                        .broadcast_tx_commit(
                            msgs.iter().map(move |x| x.1.clone()).collect::<Vec<_>>(),
                            memo,
                            true,
                        )
                        .await
                    {
                        Ok((tx_hash, tx_response)) => {
                            info!(
                                %tx_hash,
                                gas_used = %tx_response.tx_result.gas_used,
                                batch.size = %batch_size,
                                "submitted cosmos transaction"
                            );

                            for msg in msg_names {
                                info!(%tx_hash, %msg, "cosmos tx");
                            }

                            Ok(())
                        }
                        Err(err) => {
                            info!(error = %ErrorReporter(&err), "cosmos tx failed");
                            Err(err)
                        }
                    }
                })
            })
            .await
    }
}

// {
//     Ok((tx_hash, gas_used)) => {
//         info!(
//             %tx_hash,
//             %gas_used,
//             batch.size = %batch_size,
//             "submitted cosmos transaction"
//         );

//         for msg in msg_names {
//             info!(%tx_hash, %msg, "cosmos tx");
//         }

//         Ok(())
//     }
//     Err(err) => match err {
//         BroadcastTxCommitError::Tx(CosmosSdkError::ChannelError(
//             ChannelError::ErrRedundantTx,
//         )) => {
//             info!("packet messages are redundant");
//             Ok(())
//         }
//         // BroadcastTxCommitError::Tx(CosmosSdkError::SdkError(
//         //     SdkError::ErrOutOfGas
//         // )) => {
//         //     error!("out of gas");
//         //     Err(BroadcastTxCommitError::OutOfGas)
//         // }
//         BroadcastTxCommitError::Tx(CosmosSdkError::SdkError(
//             SdkError::ErrWrongSequence
//         )) => {
//             warn!("account sequence mismatch on tx submission, message will be requeued and retried");
//             Err(BroadcastTxCommitError::AccountSequenceMismatch(None))
//         }
//         BroadcastTxCommitError::SimulateTx(err) if err.message().contains("account sequence mismatch") => {
//             warn!("account sequence mismatch on simulation, message will be requeued and retried");
//             Err(BroadcastTxCommitError::AccountSequenceMismatch(Some(err)))
//         }
//         err => Err(err),
//     },
// }

#[async_trait]
impl PluginServer<ModuleCall, ModuleCallback> for Module {
    #[instrument(skip_all)]
    async fn run_pass(
        &self,
        _: &Extensions,
        msgs: Vec<Op<VoyagerMessage>>,
    ) -> RpcResult<PassResult<VoyagerMessage>> {
        let mut hook = SubmitTxHook::new(&self.chain_id, |submit_tx| {
            PluginMessage::new(
                self.plugin_name(),
                ModuleCall::SubmitTransaction(
                    submit_tx
                        .datagrams
                        .clone()
                        .into_iter()
                        .map(IbcMessage::from_raw_datagram)
                        .collect::<Result<_, _>>()
                        .unwrap(),
                ),
            )
            .into()
        });

        debug!(msgs = msgs.len(), "optimizing messages");

        Ok(PassResult {
            optimize_further: vec![],
            ready: msgs
                .into_iter()
                .enumerate()
                .map(|(idx, mut op)| {
                    hook.visit_op(&mut op);

                    (vec![idx], op)
                })
                .collect(),
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    #[allow(clippy::collapsible_match)]
    async fn call(&self, _: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::SubmitTransaction(mut msgs) => {
                let batch_submission_result = self.do_send_transaction(msgs.clone()).await;

                match batch_submission_result {
                    None => return Err(ErrorObject::owned(-1, "no signers available", None::<()>)),
                    Some(Ok(())) => return Ok(noop()),
                    Some(Err(err)) => {
                        // dbg!(&err);

                        // let mut split_msgs = false;

                        match err {
                            _ if let Some(err) = err.as_json_rpc_error() => {
                                return Err(ErrorObject::owned(
                                    -1,
                                    ErrorReporter(err).with_message("jsonrpc error"),
                                    None::<()>,
                                ))
                            }

                            BroadcastTxCommitError::FetchAccountInfo(
                                FetchAccountInfoError::Query(GrpcAbciQueryError {
                                    error_code,
                                    codespace,
                                    log,
                                }),
                            )
                            | BroadcastTxCommitError::SimulateTx(SimulateTxError::Query(
                                GrpcAbciQueryError {
                                    error_code,
                                    codespace,
                                    log,
                                },
                            ))
                            | BroadcastTxCommitError::TxFailed {
                                codespace,
                                error_code,
                                log,
                            } if ACCOUNT_SEQUENCE_ERRORS.contains(&(&codespace, error_code))
                                || log.contains("account sequence mismatch") =>
                            {
                                return Err(ErrorObject::owned(
                                    -1,
                                    format!("account sequence mismatch ({codespace}, {error_code}): {log}"),
                                    None::<()>,
                                ));
                            }

                            BroadcastTxCommitError::FetchAccountInfo(
                                FetchAccountInfoError::Query(GrpcAbciQueryError {
                                    error_code,
                                    codespace,
                                    log,
                                }),
                            )
                            | BroadcastTxCommitError::SimulateTx(SimulateTxError::Query(
                                GrpcAbciQueryError {
                                    error_code,
                                    codespace,
                                    log,
                                },
                            ))
                            | BroadcastTxCommitError::TxFailed {
                                codespace,
                                error_code,
                                log,
                            } => {
                                if let Some((msg_idx, log)) = parse_msg_idx_from_log(&log) {
                                    let _span = info_span!("cosmos msg failed", msg_idx).entered();
                                    info!(%log);

                                    match self.fatal_errors.get(&(codespace.clone(), error_code)) {
                                        // no msg
                                        Some(None) => {
                                            error!(codespace, error_code, %log, "fatal error");
                                        }
                                        // provided msg
                                        Some(Some(msg)) => {
                                            error!(codespace, error_code, %log, "fatal error: {msg}");
                                        }
                                        // unknown error, retry
                                        None => match parse_wasm_failure(log) {
                                            Some(err) => match err {
                                                ContractErrorKind::ReceivedTimedOutPacketHeight => {
                                                    info!("packet timed out (height)");
                                                }
                                                ContractErrorKind::ReceivedTimedOutPacketTimestamp => {
                                                    info!("packet timed out (timestamp)");
                                                }
                                                // ContractErrorKind::PacketNotReceived => {}
                                                ContractErrorKind::AlreadyAcknowledged => {
                                                    info!("packet already acknowledged");
                                                }
                                                ContractErrorKind::PacketCommitmentNotFound => {
                                                    info!("packet commitment not found");
                                                }
                                                _ => {
                                                    warn!("ibc-union error ({err}): {log}");
                                                    // split_msgs = true;
                                                }
                                            },
                                            None => {
                                                warn!("error submitting transaction ({codespace}, {error_code}): {log}");
                                                // split_msgs = true;
                                            }
                                        },
                                    }

                                    // if !split_msgs {
                                    //     msgs.remove(msg_idx);

                                    //     if msgs.is_empty() {
                                    //         Ok(noop())
                                    //     } else {
                                    //         Ok(call(PluginMessage::new(
                                    //             self.plugin_name(),
                                    //             ModuleCall::SubmitTransaction(msgs),
                                    //         )))
                                    //     }
                                    // } else
                                    if msgs.len() == 1 {
                                        warn!("cosmos msg failed");

                                        Ok(noop())
                                    } else {
                                        // Ok(seq(msgs.into_iter().map(|msg| {
                                        //     call(PluginMessage::new(
                                        //         self.plugin_name(),
                                        //         ModuleCall::SubmitTransaction(vec![msg]),
                                        //     ))
                                        // })))
                                        msgs.remove(msg_idx);

                                        if msgs.is_empty() {
                                            Ok(noop())
                                        } else {
                                            Ok(call(PluginMessage::new(
                                                self.plugin_name(),
                                                ModuleCall::SubmitTransaction(msgs),
                                            )))
                                        }
                                    }
                                } else if log.contains("insufficient funds") {
                                    warn!("out of gas");

                                    return Err(ErrorObject::owned(-1, "out of gas", None::<()>));
                                } else {
                                    warn!("unable to parse message index from tx failure ({codespace}, {error_code}): {log}");

                                    if msgs.len() == 1 {
                                        warn!("cosmos msg failed");
                                        Ok(noop())
                                    } else {
                                        Ok(seq(msgs.into_iter().map(|msg| {
                                            call(PluginMessage::new(
                                                self.plugin_name(),
                                                ModuleCall::SubmitTransaction(vec![msg]),
                                            ))
                                        })))
                                    }
                                }
                            }
                            _ => Err(ErrorObject::owned(
                                -1,
                                ErrorReporter(err).with_message("error submitting tx"),
                                None::<()>,
                            )),
                        }
                    }
                }
            }
        }
    }

    #[instrument(skip_all)]
    async fn callback(
        &self,
        _: &Extensions,
        cb: ModuleCallback,
        _data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match cb {}
    }
}

fn process_msgs(
    msgs: Vec<IbcMessage>,
    signer: &LocalSigner,
    ibc_host_contract_address: Bech32<H256>,
) -> Vec<RpcResult<(IbcMessage, protos::google::protobuf::Any)>> {
    msgs.into_iter()
        .map(|msg| {
            let signer = signer.address().to_string();

            let encoded = match msg.clone() {
                IbcMessage::IbcV1(msg) => match msg {
                    ibc_classic_spec::Datagram::ConnectionOpenInit(message) => {
                        mk_any(&protos::ibc::core::connection::v1::MsgConnectionOpenInit {
                            client_id: message.client_id.to_string(),
                            counterparty: Some(message.counterparty.into()),
                            version: Some(message.version.into()),
                            signer,
                            delay_period: message.delay_period,
                        })
                    }
                    ibc_classic_spec::Datagram::ConnectionOpenTry(message) => {
                        mk_any(&protos::ibc::core::connection::v1::MsgConnectionOpenTry {
                            client_id: message.client_id.to_string(),
                            counterparty: Some(message.counterparty.into()),
                            delay_period: message.delay_period,
                            counterparty_versions: message
                                .counterparty_versions
                                .into_iter()
                                .map(Into::into)
                                .collect(),
                            proof_height: Some(message.proof_height.into()),
                            proof_init: message.proof_init.into(),
                            signer,
                            ..Default::default()
                        })
                    }
                    #[allow(deprecated)]
                    ibc_classic_spec::Datagram::ConnectionOpenAck(message) => {
                        mk_any(&protos::ibc::core::connection::v1::MsgConnectionOpenAck {
                            client_state: Some(
                                protos::google::protobuf::Any::decode(&*message.client_state)
                                    .expect("value should be encoded as an `Any`"),
                            ),
                            proof_height: Some(message.proof_height.into()),
                            proof_client: message.proof_client.into(),
                            proof_consensus: message.proof_consensus.into(),
                            consensus_height: Some(message.consensus_height.into()),
                            signer,
                            host_consensus_state_proof: vec![],
                            connection_id: message.connection_id.to_string(),
                            counterparty_connection_id: message
                                .counterparty_connection_id
                                .to_string(),
                            version: Some(message.version.into()),
                            proof_try: message.proof_try.into(),
                        })
                    }
                    ibc_classic_spec::Datagram::ConnectionOpenConfirm(message) => mk_any(
                        &protos::ibc::core::connection::v1::MsgConnectionOpenConfirm {
                            connection_id: message.connection_id.to_string(),
                            proof_ack: message.proof_ack.into(),
                            proof_height: Some(message.proof_height.into()),
                            signer,
                        },
                    ),
                    ibc_classic_spec::Datagram::ChannelOpenInit(message) => {
                        mk_any(&protos::ibc::core::channel::v1::MsgChannelOpenInit {
                            port_id: message.port_id.to_string(),
                            channel: Some(message.channel.into()),
                            signer,
                        })
                    }
                    ibc_classic_spec::Datagram::ChannelOpenTry(message) => {
                        mk_any(&protos::ibc::core::channel::v1::MsgChannelOpenTry {
                            port_id: message.port_id.to_string(),
                            channel: Some(message.channel.into()),
                            counterparty_version: message.counterparty_version,
                            proof_init: message.proof_init.into(),
                            proof_height: Some(message.proof_height.into()),
                            signer,
                            ..Default::default()
                        })
                    }
                    ibc_classic_spec::Datagram::ChannelOpenAck(message) => {
                        mk_any(&protos::ibc::core::channel::v1::MsgChannelOpenAck {
                            port_id: message.port_id.to_string(),
                            channel_id: message.channel_id.to_string(),
                            counterparty_version: message.counterparty_version,
                            counterparty_channel_id: message.counterparty_channel_id.to_string(),
                            proof_try: message.proof_try.into(),
                            proof_height: Some(message.proof_height.into()),
                            signer,
                        })
                    }
                    ibc_classic_spec::Datagram::ChannelOpenConfirm(message) => {
                        mk_any(&protos::ibc::core::channel::v1::MsgChannelOpenConfirm {
                            port_id: message.port_id.to_string(),
                            channel_id: message.channel_id.to_string(),
                            proof_height: Some(message.proof_height.into()),
                            signer,
                            proof_ack: message.proof_ack.into(),
                        })
                    }
                    ibc_classic_spec::Datagram::RecvPacket(message) => {
                        mk_any(&protos::ibc::core::channel::v1::MsgRecvPacket {
                            packet: Some(message.packet.into()),
                            proof_height: Some(message.proof_height.into()),
                            signer,
                            proof_commitment: message.proof_commitment.into(),
                        })
                    }
                    ibc_classic_spec::Datagram::AcknowledgePacket(message) => {
                        mk_any(&protos::ibc::core::channel::v1::MsgAcknowledgement {
                            packet: Some(message.packet.into()),
                            acknowledgement: message.acknowledgement.into(),
                            proof_acked: message.proof_acked.into(),
                            proof_height: Some(message.proof_height.into()),
                            signer,
                        })
                    }
                    ibc_classic_spec::Datagram::TimeoutPacket(message) => {
                        mk_any(&protos::ibc::core::channel::v1::MsgTimeout {
                            packet: Some(message.packet.into()),
                            proof_unreceived: message.proof_unreceived,
                            proof_height: Some(message.proof_height.into()),
                            next_sequence_recv: message.next_sequence_recv.get(),
                            signer,
                        })
                    }
                    ibc_classic_spec::Datagram::CreateClient(message) => {
                        mk_any(&protos::ibc::core::client::v1::MsgCreateClient {
                            client_state: Some(
                                protos::google::protobuf::Any::decode(&*message.msg.client_state)
                                    .expect("value should be encoded as an `Any`"),
                            ),
                            consensus_state: Some(
                                protos::google::protobuf::Any::decode(
                                    &*message.msg.consensus_state,
                                )
                                .expect("value should be encoded as an `Any`"),
                            ),
                            signer,
                        })
                    }
                    ibc_classic_spec::Datagram::UpdateClient(message) => {
                        mk_any(&protos::ibc::core::client::v1::MsgUpdateClient {
                            signer,
                            client_id: message.client_id.to_string(),
                            client_message: Some(
                                protos::google::protobuf::Any::decode(&*message.client_message)
                                    .expect("value should be encoded as an `Any`"),
                            ),
                        })
                    }
                },
                IbcMessage::IbcUnion(msg) => match msg {
                    ibc_union_spec::datagram::Datagram::CreateClient(msg_create_client) => {
                        mk_any(&protos::cosmwasm::wasm::v1::MsgExecuteContract {
                            sender: signer.to_string(),
                            contract: ibc_host_contract_address.to_string(),
                            msg: serde_json::to_vec(&ibc_union_msg::msg::ExecuteMsg::CreateClient(
                                ibc_union_msg::msg::MsgCreateClient {
                                    client_type: msg_create_client.client_type.to_string(),
                                    client_state_bytes: msg_create_client.client_state_bytes,
                                    consensus_state_bytes: msg_create_client.consensus_state_bytes,
                                    relayer: signer.to_string(),
                                },
                            ))
                            .unwrap(),
                            funds: vec![],
                        })
                    }
                    ibc_union_spec::datagram::Datagram::UpdateClient(msg_update_client) => {
                        mk_any(&protos::cosmwasm::wasm::v1::MsgExecuteContract {
                            sender: signer.to_string(),
                            contract: ibc_host_contract_address.to_string(),
                            msg: serde_json::to_vec(&ibc_union_msg::msg::ExecuteMsg::UpdateClient(
                                ibc_union_msg::msg::MsgUpdateClient {
                                    client_id: msg_update_client.client_id,
                                    client_message: msg_update_client.client_message,
                                    relayer: signer.to_string(),
                                },
                            ))
                            .unwrap(),
                            funds: vec![],
                        })
                    }
                    ibc_union_spec::datagram::Datagram::ConnectionOpenInit(
                        msg_connection_open_init,
                    ) => mk_any(&protos::cosmwasm::wasm::v1::MsgExecuteContract {
                        sender: signer.to_string(),
                        contract: ibc_host_contract_address.to_string(),
                        msg: serde_json::to_vec(
                            &ibc_union_msg::msg::ExecuteMsg::ConnectionOpenInit(
                                ibc_union_msg::msg::MsgConnectionOpenInit {
                                    client_id: msg_connection_open_init.client_id,
                                    counterparty_client_id: msg_connection_open_init
                                        .counterparty_client_id,
                                    relayer: signer.to_string(),
                                },
                            ),
                        )
                        .unwrap(),
                        funds: vec![],
                    }),
                    ibc_union_spec::datagram::Datagram::ConnectionOpenTry(
                        msg_connection_open_try,
                    ) => mk_any(&protos::cosmwasm::wasm::v1::MsgExecuteContract {
                        sender: signer.to_string(),
                        contract: ibc_host_contract_address.to_string(),
                        msg: serde_json::to_vec(
                            &ibc_union_msg::msg::ExecuteMsg::ConnectionOpenTry(
                                ibc_union_msg::msg::MsgConnectionOpenTry {
                                    counterparty_client_id: msg_connection_open_try
                                        .counterparty_client_id,
                                    counterparty_connection_id: msg_connection_open_try
                                        .counterparty_connection_id,
                                    client_id: msg_connection_open_try.client_id,
                                    proof_init: msg_connection_open_try.proof_init,
                                    proof_height: msg_connection_open_try.proof_height,
                                    relayer: signer.to_string(),
                                },
                            ),
                        )
                        .unwrap(),
                        funds: vec![],
                    }),
                    ibc_union_spec::datagram::Datagram::ConnectionOpenAck(
                        msg_connection_open_ack,
                    ) => mk_any(&protos::cosmwasm::wasm::v1::MsgExecuteContract {
                        sender: signer.to_string(),
                        contract: ibc_host_contract_address.to_string(),
                        msg: serde_json::to_vec(
                            &ibc_union_msg::msg::ExecuteMsg::ConnectionOpenAck(
                                ibc_union_msg::msg::MsgConnectionOpenAck {
                                    connection_id: msg_connection_open_ack.connection_id,
                                    counterparty_connection_id: msg_connection_open_ack
                                        .counterparty_connection_id,
                                    proof_try: msg_connection_open_ack.proof_try,
                                    proof_height: msg_connection_open_ack.proof_height,
                                    relayer: signer.to_string(),
                                },
                            ),
                        )
                        .unwrap(),
                        funds: vec![],
                    }),
                    ibc_union_spec::datagram::Datagram::ConnectionOpenConfirm(
                        msg_connection_open_confirm,
                    ) => mk_any(&protos::cosmwasm::wasm::v1::MsgExecuteContract {
                        sender: signer.to_string(),
                        contract: ibc_host_contract_address.to_string(),
                        msg: serde_json::to_vec(
                            &ibc_union_msg::msg::ExecuteMsg::ConnectionOpenConfirm(
                                ibc_union_msg::msg::MsgConnectionOpenConfirm {
                                    connection_id: msg_connection_open_confirm.connection_id,
                                    proof_ack: msg_connection_open_confirm.proof_ack,
                                    proof_height: msg_connection_open_confirm.proof_height,
                                    relayer: signer.to_string(),
                                },
                            ),
                        )
                        .unwrap(),
                        funds: vec![],
                    }),
                    ibc_union_spec::datagram::Datagram::ChannelOpenInit(msg_channel_open_init) => {
                        let channel_open_init = ibc_union_msg::msg::ExecuteMsg::ChannelOpenInit(
                            ibc_union_msg::msg::MsgChannelOpenInit {
                                port_id: parse_port_id(msg_channel_open_init.port_id.to_vec())?,
                                relayer: signer.to_string(),
                                counterparty_port_id: msg_channel_open_init.counterparty_port_id,
                                connection_id: msg_channel_open_init.connection_id,
                                version: msg_channel_open_init.version,
                            },
                        );

                        mk_any(&protos::cosmwasm::wasm::v1::MsgExecuteContract {
                            sender: signer.to_string(),
                            contract: ibc_host_contract_address.to_string(),
                            msg: serde_json::to_vec(&channel_open_init).unwrap(),
                            funds: vec![],
                        })
                    }
                    ibc_union_spec::datagram::Datagram::ChannelOpenTry(msg_channel_open_try) => {
                        let channel_open_try = ibc_union_msg::msg::ExecuteMsg::ChannelOpenTry(
                            ibc_union_msg::msg::MsgChannelOpenTry {
                                port_id: parse_port_id(msg_channel_open_try.port_id.to_vec())?,
                                channel: msg_channel_open_try.channel,
                                counterparty_version: msg_channel_open_try.counterparty_version,
                                proof_init: msg_channel_open_try.proof_init,
                                proof_height: msg_channel_open_try.proof_height,
                                relayer: signer.to_string(),
                            },
                        );

                        mk_any(&protos::cosmwasm::wasm::v1::MsgExecuteContract {
                            sender: signer.to_string(),
                            contract: ibc_host_contract_address.to_string(),
                            msg: serde_json::to_vec(&channel_open_try).unwrap(),
                            funds: vec![],
                        })
                    }
                    ibc_union_spec::datagram::Datagram::ChannelOpenAck(msg_channel_open_ack) => {
                        let channel_open_ack = ibc_union_msg::msg::ExecuteMsg::ChannelOpenAck(
                            ibc_union_msg::msg::MsgChannelOpenAck {
                                channel_id: msg_channel_open_ack.channel_id,
                                counterparty_version: msg_channel_open_ack.counterparty_version,
                                counterparty_channel_id: msg_channel_open_ack
                                    .counterparty_channel_id,
                                proof_try: msg_channel_open_ack.proof_try,
                                proof_height: msg_channel_open_ack.proof_height,
                                relayer: signer.to_string(),
                            },
                        );

                        mk_any(&protos::cosmwasm::wasm::v1::MsgExecuteContract {
                            sender: signer.to_string(),
                            contract: ibc_host_contract_address.to_string(),
                            msg: serde_json::to_vec(&channel_open_ack).unwrap(),
                            funds: vec![],
                        })
                    }
                    ibc_union_spec::datagram::Datagram::ChannelOpenConfirm(
                        msg_channel_open_confirm,
                    ) => {
                        let channel_open_confirm =
                            ibc_union_msg::msg::ExecuteMsg::ChannelOpenConfirm(
                                ibc_union_msg::msg::MsgChannelOpenConfirm {
                                    channel_id: msg_channel_open_confirm.channel_id,
                                    proof_ack: msg_channel_open_confirm.proof_ack,
                                    proof_height: msg_channel_open_confirm.proof_height,
                                    relayer: signer.to_string(),
                                },
                            );

                        mk_any(&protos::cosmwasm::wasm::v1::MsgExecuteContract {
                            sender: signer.to_string(),
                            contract: ibc_host_contract_address.to_string(),
                            msg: serde_json::to_vec(&channel_open_confirm).unwrap(),
                            funds: vec![],
                        })
                    }
                    ibc_union_spec::datagram::Datagram::ChannelCloseInit(
                        _msg_channel_close_init,
                    ) => todo!(),
                    ibc_union_spec::datagram::Datagram::ChannelCloseConfirm(
                        _msg_channel_close_confirm,
                    ) => {
                        todo!()
                    }
                    ibc_union_spec::datagram::Datagram::PacketRecv(msg_packet_recv) => {
                        let packet_recv = ibc_union_msg::msg::ExecuteMsg::PacketRecv(
                            ibc_union_msg::msg::MsgPacketRecv {
                                packets: msg_packet_recv
                                    .packets
                                    .into_iter()
                                    .map(Into::into)
                                    .collect(),
                                relayer_msgs: msg_packet_recv.relayer_msgs,
                                proof: msg_packet_recv.proof,
                                proof_height: msg_packet_recv.proof_height,
                                relayer: signer.to_string(),
                            },
                        );

                        mk_any(&protos::cosmwasm::wasm::v1::MsgExecuteContract {
                            sender: signer.to_string(),
                            contract: ibc_host_contract_address.to_string(),
                            msg: serde_json::to_vec(&packet_recv).unwrap(),
                            funds: vec![],
                        })
                    }
                    ibc_union_spec::datagram::Datagram::PacketAcknowledgement(
                        msg_packet_acknowledgement,
                    ) => {
                        let packet_recv = ibc_union_msg::msg::ExecuteMsg::PacketAck(
                            ibc_union_msg::msg::MsgPacketAcknowledgement {
                                packets: msg_packet_acknowledgement
                                    .packets
                                    .into_iter()
                                    .map(Into::into)
                                    .collect(),
                                acknowledgements: msg_packet_acknowledgement.acknowledgements,
                                proof: msg_packet_acknowledgement.proof,
                                proof_height: msg_packet_acknowledgement.proof_height,
                                relayer: signer.to_string(),
                            },
                        );

                        mk_any(&protos::cosmwasm::wasm::v1::MsgExecuteContract {
                            sender: signer.to_string(),
                            contract: ibc_host_contract_address.to_string(),
                            msg: serde_json::to_vec(&packet_recv).unwrap(),
                            funds: vec![],
                        })
                    }
                    ibc_union_spec::datagram::Datagram::PacketTimeout(_msg_packet_timeout) => {
                        todo!()
                    }
                    ibc_union_spec::datagram::Datagram::IntentPacketRecv(
                        _msg_intent_packet_recv,
                    ) => todo!(),
                    ibc_union_spec::datagram::Datagram::BatchSend(_msg_batch_send) => todo!(),
                    ibc_union_spec::datagram::Datagram::BatchAcks(_msg_batch_acks) => todo!(),
                },
            };

            Ok((msg, encoded))
        })
        .collect()
}

fn parse_port_id(bz: Vec<u8>) -> RpcResult<String> {
    String::from_utf8(bz).map_err(|e| {
        ErrorObject::owned(
            FATAL_JSONRPC_ERROR_CODE,
            format!("invalid utf8: {}", <Bytes>::new(e.into_bytes())),
            None::<()>,
        )
    })
}

// rpc error: code = Unknown desc = failed to execute message; message index: 0: IBC_UNION_ERR_PACKET_COMMITMENT_NOT_FOUND packet commitment not found: execute wasm contract failed [CosmWasm/wasmd@v0.53.2/x/wasm/keeper/keeper.go:436] with gas used: '287090'
fn parse_wasm_failure(log: &str) -> Option<ContractErrorKind> {
    log.split(' ').find_map(ContractErrorKind::parse)
}

fn parse_msg_idx_from_log(log: &str) -> Option<(usize, &str)> {
    let (_, log) = log.split_once("message index: ")?;
    let (idx, log) = log.split_once(':')?;
    Some((idx.parse().ok()?, log))
}

#[test]
fn test_parse_wasm_failure() {
    let (idx, log) = parse_msg_idx_from_log("rpc error: code = Unknown desc = failed to execute message; message index: 0: IBC_UNION_ERR_PACKET_COMMITMENT_NOT_FOUND packet commitment not found: execute wasm contract failed [CosmWasm/wasmd@v0.53.2/x/wasm/keeper/keeper.go:436] with gas used: '287090'").unwrap();

    dbg!(idx, parse_wasm_failure(log));
}

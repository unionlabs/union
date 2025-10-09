// #![warn(clippy::unwrap_used)]
#![feature(if_let_guard)]

use std::{
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    num::NonZeroU32,
    ops::Deref,
    panic::AssertUnwindSafe,
    sync::{Arc, LazyLock},
};

use cometbft_rpc::rpc_types::GrpcAbciQueryError;
use concurrent_keyring::{ConcurrentKeyring, KeyringConfig, KeyringEntry};
use cosmos_client::{
    BroadcastTxCommitError, TxClient,
    gas::{GasFillerT, any, feemarket, fixed, osmosis_eip1559_feemarket},
    rpc::{Rpc, RpcT},
    wallet::{LocalSigner, WalletT},
};
use ibc_union::ContractErrorKind;
use jsonrpsee::{
    Extensions, MethodsError,
    core::{RpcResult, async_trait},
    proc_macros::rpc,
    types::ErrorObject,
};
use prost::Message;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, error, info, info_span, instrument, trace, warn};
use unionlabs::{
    self, ErrorReporter,
    cosmos::base::coin::Coin,
    google::protobuf::any::mk_any,
    never::Never,
    option_unwrap,
    primitives::{Bech32, Bytes, H160, H256},
};
use voyager_sdk::{
    DefaultCmd,
    anyhow::{self, anyhow, bail},
    hook::SubmitTxHook,
    into_value,
    message::{PluginMessage, VoyagerMessage, data::Data},
    plugin::Plugin,
    primitives::ChainId,
    rpc::{FATAL_JSONRPC_ERROR_CODE, PluginServer, types::PluginInfo},
    vm::{BoxDynError, Op, Visit, call, defer_relative, noop, pass::PassResult, seq},
};

use crate::call::{IbcMessage, ModuleCall};

pub mod call;

#[tokio::main]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module(Arc<ModuleInner>);

#[derive(Debug)]
pub struct ModuleInner {
    pub chain_id: ChainId,
    pub ibc_host_contract_address: Bech32<H256>,
    pub keyring: ConcurrentKeyring<Bech32<H160>, LocalSigner>,
    pub rpc: Rpc,
    pub gas_config: any::GasFiller,
    pub bech32_prefix: String,
    pub fatal_errors: HashMap<(String, NonZeroU32), Option<String>>,
    pub gas_station_config: Vec<Coin>,
    pub fee_recipient: Option<Bech32<Bytes>>,
    pub max_tx_size: u32,
}

impl Deref for Module {
    type Target = ModuleInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId,
    pub ibc_host_contract_address: Bech32<H256>,
    pub keyring: KeyringConfig,
    pub rpc_url: String,
    pub gas_config: GasFillerConfig,
    /// A list of (codespace, code) tuples that are to be considered non-recoverable.
    #[serde(default)]
    pub fatal_errors: HashMap<(String, NonZeroU32), Option<String>>,
    #[serde(default)]
    pub gas_station_config: Vec<Coin>,
    #[serde(default)]
    pub fee_recipient: Option<Bech32<Bytes>>,
    pub max_tx_size: u32,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "config")]
pub enum GasFillerConfig {
    // fixed gas filler is it's own config
    Fixed(fixed::GasFiller),
    Feemarket(FeemarketConfig),
    OsmosisEip1559Feemarket(OsmosisEip1559FeemarketConfig),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FeemarketConfig {
    pub max_gas: u64,
    #[serde(with = "::serde_utils::string_opt")]
    pub gas_multiplier: Option<f64>,
    pub denom: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OsmosisEip1559FeemarketConfig {
    pub max_gas: u64,
    #[serde(with = "::serde_utils::string_opt")]
    pub gas_multiplier: Option<f64>,
    #[serde(with = "::serde_utils::string_opt")]
    pub base_fee_multiplier: Option<f64>,
    pub denom: Option<String>,
}

impl GasFillerConfig {
    async fn into_gas_filler(self, rpc_url: String) -> Result<any::GasFiller, BoxDynError> {
        Ok(match self {
            GasFillerConfig::Fixed(config) => any::GasFiller::Fixed(config),
            GasFillerConfig::Feemarket(config) => any::GasFiller::Feemarket(
                feemarket::GasFiller::new(feemarket::Config {
                    rpc_url,
                    max_gas: config.max_gas,
                    gas_multiplier: config.gas_multiplier,
                    denom: config.denom,
                })
                .await?,
            ),
            GasFillerConfig::OsmosisEip1559Feemarket(config) => {
                any::GasFiller::OsmosisEip1559Feemarket(
                    osmosis_eip1559_feemarket::GasFiller::new(osmosis_eip1559_feemarket::Config {
                        rpc_url,
                        max_gas: config.max_gas,
                        gas_multiplier: config.gas_multiplier,
                        base_fee_multiplier: config.base_fee_multiplier,
                        denom: config.denom,
                    })
                    .await?,
                )
            }
        })
    }
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
    type Callback = Never;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> anyhow::Result<Self> {
        let rpc = Rpc::new(config.rpc_url.clone()).await?;

        let chain_id = rpc.client().status().await?.node_info.network.to_string();

        if chain_id != config.chain_id.as_str() {
            bail!(
                "incorrect chain id: expected `{}`, but found `{}`",
                config.chain_id,
                chain_id
            );
        }

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

        Ok(Self(Arc::new(ModuleInner {
            ibc_host_contract_address: config.ibc_host_contract_address,
            keyring: ConcurrentKeyring::new(
                config.keyring.name,
                config.keyring.keys.into_iter().map(|entry| {
                    let signer =
                        LocalSigner::new(entry.value().try_into().unwrap(), bech32_prefix.clone());

                    KeyringEntry {
                        address: signer.address(),
                        signer,
                    }
                }),
            ),
            rpc,
            chain_id: ChainId::new(chain_id),
            gas_config: config
                .gas_config
                .into_gas_filler(config.rpc_url.clone())
                .await
                .map_err(|e| anyhow!(e))?,
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
            gas_station_config: config.gas_station_config,
            fee_recipient: config.fee_recipient,
            max_tx_size: config.max_tx_size,
        })))
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

// TODO: Currently duplicated between here and the ethereum tx plugin, deduplicate
#[rpc(server)]
trait TransactionPlugin {
    #[method(name = "signerAddresses")]
    async fn signer_addresses(&self) -> RpcResult<Vec<Bech32<H160>>>;

    #[method(name = "signerBalances")]
    async fn signer_balances(&self) -> RpcResult<BTreeMap<Bech32<H160>, String>>;
}

#[async_trait]
impl TransactionPluginServer for Module {
    async fn signer_addresses(&self) -> RpcResult<Vec<Bech32<H160>>> {
        Ok(self.keyring.keys().cloned().collect())
    }

    async fn signer_balances(&self) -> RpcResult<BTreeMap<Bech32<H160>, String>> {
        let mut out = BTreeMap::new();

        for address in self.keyring.keys() {
            let balance = self
                .rpc
                .client()
                .grpc_abci_query::<_, protos::cosmos::bank::v1beta1::QueryBalanceResponse>(
                    "/cosmos.bank.v1beta1.Query/Balance",
                    &protos::cosmos::bank::v1beta1::QueryBalanceRequest {
                        address: address.to_string(),
                        denom: self.gas_config.mk_fee(0).await.amount[0].denom.clone(),
                    },
                    None,
                    false,
                )
                .await
                .map_err(|e| {
                    ErrorObject::owned(
                        -1,
                        ErrorReporter(e).with_message("error fetching balance"),
                        None::<()>,
                    )
                })?
                .into_result()
                .map_err(|e| {
                    ErrorObject::owned(
                        -1,
                        ErrorReporter(e).with_message("error fetching balance"),
                        None::<()>,
                    )
                })?
                .ok_or_else(|| {
                    ErrorObject::owned(-1, "empty response when fetching balance", None::<()>)
                })?
                .balance
                .ok_or_else(|| {
                    ErrorObject::owned(-1, "empty balance when fetching balance", None::<()>)
                })?
                .amount;

            out.insert(address.clone(), balance);
        }

        Ok(out)
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
    ) -> Option<Result<Option<Op<VoyagerMessage>>, BroadcastTxCommitError>> {
        self.keyring
            .with(|signer| {
                let msgs = msgs.clone();

                trace!(?msgs);

                // TODO: Figure out a way to thread this value through
                let memo = format!("Voyager {}", env!("CARGO_PKG_VERSION"));

                let ibc_host_contract_address = self.ibc_host_contract_address.clone();
                let msgs = process_msgs(
                    msgs,
                    signer,
                    ibc_host_contract_address,
                    self.gas_station_config.clone(),
                    self.fee_recipient.as_ref(),
                );

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

                let approximate_size = msgs.iter().map(|x| x.1.encoded_len()).sum::<usize>();

                info!(
                    %approximate_size,
                    max_tx_size = %self.max_tx_size,
                    "approximate tx size"
                );

                AssertUnwindSafe(async move {
                    if msgs.is_empty() {
                        info!("no msgs left to submit after filtering out invalid msgs");
                        return Ok(None);
                    }

                    if approximate_size > self.max_tx_size as usize {
                        if msgs.len() == 1 {
                            error!(
                                %approximate_size,
                                max_tx_size = %self.max_tx_size,
                                msg = msgs.first().unwrap().0.name(),
                                "message is too large, dropping as it cannot be submitted"
                            );
                            return Ok(None);
                        } else {
                            warn!(
                                %approximate_size,
                                max_tx_size = %self.max_tx_size,
                                "tx is too large, splitting messages"
                            );

                            let mut msgs = msgs.into_iter().map(|x| x.0).collect::<Vec<_>>();

                            let new_msgs = msgs.split_off(msgs.len().div_ceil(2));

                            return Ok(Some(seq([
                                call(PluginMessage::new(
                                    self.plugin_name(),
                                    ModuleCall::SubmitTransaction(msgs),
                                )),
                                // ensure that the first half gets included
                                defer_relative(3),
                                call(PluginMessage::new(
                                    self.plugin_name(),
                                    ModuleCall::SubmitTransaction(new_msgs),
                                )),
                            ])));
                        }
                    };

                    match tx_client
                        .broadcast_tx_commit(
                            msgs.iter().map(move |x| x.1.clone()).collect::<Vec<_>>(),
                            memo,
                            true,
                        )
                        .await
                    {
                        Ok(tx_response) => {
                            info!(
                                tx_hash = %tx_response.hash,
                                gas_used = %tx_response.tx_result.gas_used,
                                batch.size = %batch_size,
                                "submitted cosmos transaction"
                            );

                            for msg in msg_names {
                                info!(tx_hash = %tx_response.hash, %msg, "cosmos msg");
                            }

                            Ok(None)
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
impl PluginServer<ModuleCall, Never> for Module {
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
    async fn call(&self, _: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::SubmitTransaction(mut msgs) => {
                let batch_submission_result = self.do_send_transaction(msgs.clone()).await;

                match batch_submission_result {
                    None => Err(ErrorObject::owned(-1, "no signers available", None::<()>)),
                    Some(Ok(None)) => {
                        for (idx, msg) in msgs.into_iter().enumerate() {
                            info!(
                                msg = msg.name(),
                                %idx,
                                data = %into_value(&msg),
                                "cosmos tx",
                            );
                        }
                        Ok(noop())
                    }
                    Some(Ok(Some(op))) => Ok(op),
                    Some(Err(err)) => {
                        match err {
                            _ if let Some(err) = err.as_json_rpc_error() => {
                                return Err(ErrorObject::owned(
                                    -1,
                                    ErrorReporter(err).with_message("jsonrpc error"),
                                    None::<()>,
                                ));
                            }

                            BroadcastTxCommitError::Query(GrpcAbciQueryError {
                                error_code,
                                codespace,
                                log,
                            })
                            | BroadcastTxCommitError::TxFailed {
                                codespace,
                                error_code,
                                log,
                            } if ACCOUNT_SEQUENCE_ERRORS.contains(&(&codespace, error_code))
                                || log.contains("account sequence mismatch") =>
                            {
                                return Err(ErrorObject::owned(
                                    -1,
                                    format!(
                                        "account sequence mismatch ({codespace}, {error_code}): {log}"
                                    ),
                                    None::<()>,
                                ));
                            }

                            BroadcastTxCommitError::Query(GrpcAbciQueryError {
                                error_code,
                                codespace,
                                log,
                            })
                            | BroadcastTxCommitError::TxFailed {
                                codespace,
                                error_code,
                                log,
                            } => {
                                info!(%log, "error submitting cosmos tx");

                                if let Some((msg_idx, log)) = parse_msg_idx_from_log(&log) {
                                    let _span = info_span!("cosmos msg failed", msg_idx).entered();
                                    info!(%log, "tx log");

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
                                                ContractErrorKind::AlreadyAcknowledged => {
                                                    info!("packet already acknowledged");
                                                }
                                                ContractErrorKind::PacketCommitmentNotFound => {
                                                    info!("packet commitment not found");
                                                }
                                                _ => {
                                                    warn!("ibc-union error ({err}): {log}");
                                                }
                                            },
                                            None => {
                                                warn!("error submitting transaction ({codespace}, {error_code}): {log}");
                                            }
                                        },
                                    }

                                    if msgs.len() == 1 {
                                        warn!(msg = %into_value(msgs.pop().unwrap()), "cosmos msg failed");

                                        Ok(noop())
                                    } else {
                                        let failed_msg = msgs.remove(msg_idx);

                                        if matches!(
                                            failed_msg,
                                            IbcMessage::IbcClassic(
                                                ibc_classic_spec::Datagram::UpdateClient(_)
                                            ) | IbcMessage::IbcUnion(
                                                ibc_union_spec::datagram::Datagram::UpdateClient(_)
                                            )
                                        ) {
                                            warn!(
                                                "update client failed, this may cause other messages to fail as well"
                                            );
                                        }

                                        warn!(msg = %into_value(failed_msg), "dropping failed msg");

                                        if msgs.is_empty() {
                                            info!(
                                                "no messages to submit after dropping failed messages"
                                            );

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
                                    warn!(
                                        "unable to parse message index from tx failure ({codespace}, {error_code}): {log}"
                                    );

                                    if msgs.len() == 1 {
                                        warn!(msg = %into_value(msgs.pop().unwrap()), "cosmos msg failed");
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
        cb: Never,
        _data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match cb {}
    }

    #[instrument(skip_all, fiellds(chain_id = %self.chain_id))]
    async fn custom(&self, _: &Extensions, method: String, params: Vec<Value>) -> RpcResult<Value> {
        TransactionPluginServer::into_rpc(self.clone())
            .call::<Vec<Value>, Value>(&method, params)
            .await
            .map_err(|e| match e {
                MethodsError::Parse(error) => ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    ErrorReporter(error).with_message("error parsing ergs"),
                    None::<()>,
                ),
                MethodsError::JsonRpc(error_object) => error_object,
                MethodsError::InvalidSubscriptionId(_) => ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    "subscriptions are not supported",
                    None::<()>,
                ),
            })
    }
}

fn process_msgs(
    msgs: Vec<IbcMessage>,
    signer: &LocalSigner,
    ibc_host_contract_address: Bech32<H256>,
    _gas_station_config: Vec<Coin>,
    fee_recipient: Option<&Bech32<Bytes>>,
) -> Vec<RpcResult<(IbcMessage, protos::google::protobuf::Any)>> {
    msgs.into_iter()
        .map(|msg| {
            let signer = signer.address().to_string();

            let encoded = match msg.clone() {
                IbcMessage::IbcClassic(msg) => {
                    use ibc_classic_spec::Datagram;
                    use protos::ibc::core::{channel::v1::*, client::v1::*, connection::v1::*};

                    match msg {
                        Datagram::ConnectionOpenInit(message) => mk_any(&MsgConnectionOpenInit {
                            client_id: message.client_id.to_string(),
                            counterparty: Some(message.counterparty.into()),
                            version: Some(message.version.into()),
                            signer,
                            delay_period: message.delay_period,
                        }),
                        Datagram::ConnectionOpenTry(message) => mk_any(&MsgConnectionOpenTry {
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
                        }),
                        #[allow(deprecated)]
                        Datagram::ConnectionOpenAck(message) => mk_any(&MsgConnectionOpenAck {
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
                        }),
                        Datagram::ConnectionOpenConfirm(message) => {
                            mk_any(&MsgConnectionOpenConfirm {
                                connection_id: message.connection_id.to_string(),
                                proof_ack: message.proof_ack.into(),
                                proof_height: Some(message.proof_height.into()),
                                signer,
                            })
                        }
                        Datagram::ChannelOpenInit(message) => mk_any(&MsgChannelOpenInit {
                            port_id: message.port_id.to_string(),
                            channel: Some(message.channel.into()),
                            signer,
                        }),
                        Datagram::ChannelOpenTry(message) => mk_any(&MsgChannelOpenTry {
                            port_id: message.port_id.to_string(),
                            channel: Some(message.channel.into()),
                            counterparty_version: message.counterparty_version,
                            proof_init: message.proof_init.into(),
                            proof_height: Some(message.proof_height.into()),
                            signer,
                            ..Default::default()
                        }),
                        Datagram::ChannelOpenAck(message) => mk_any(&MsgChannelOpenAck {
                            port_id: message.port_id.to_string(),
                            channel_id: message.channel_id.to_string(),
                            counterparty_version: message.counterparty_version,
                            counterparty_channel_id: message.counterparty_channel_id.to_string(),
                            proof_try: message.proof_try.into(),
                            proof_height: Some(message.proof_height.into()),
                            signer,
                        }),
                        Datagram::ChannelOpenConfirm(message) => mk_any(&MsgChannelOpenConfirm {
                            port_id: message.port_id.to_string(),
                            channel_id: message.channel_id.to_string(),
                            proof_height: Some(message.proof_height.into()),
                            signer,
                            proof_ack: message.proof_ack.into(),
                        }),
                        Datagram::RecvPacket(message) => mk_any(&MsgRecvPacket {
                            packet: Some(message.packet.into()),
                            proof_height: Some(message.proof_height.into()),
                            signer,
                            proof_commitment: message.proof_commitment.into(),
                        }),
                        Datagram::AcknowledgePacket(message) => mk_any(&MsgAcknowledgement {
                            packet: Some(message.packet.into()),
                            acknowledgement: message.acknowledgement.into(),
                            proof_acked: message.proof_acked.into(),
                            proof_height: Some(message.proof_height.into()),
                            signer,
                        }),
                        Datagram::TimeoutPacket(message) => mk_any(&MsgTimeout {
                            packet: Some(message.packet.into()),
                            proof_unreceived: message.proof_unreceived,
                            proof_height: Some(message.proof_height.into()),
                            next_sequence_recv: message.next_sequence_recv.get(),
                            signer,
                        }),
                        Datagram::CreateClient(message) => mk_any(&MsgCreateClient {
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
                        }),
                        Datagram::UpdateClient(message) => mk_any(&MsgUpdateClient {
                            signer,
                            client_id: message.client_id.to_string(),
                            client_message: Some(
                                protos::google::protobuf::Any::decode(&*message.client_message)
                                    .expect("value should be encoded as an `Any`"),
                            ),
                        }),
                    }
                }
                IbcMessage::IbcUnion(msg) => {
                    use ibc_union_msg::msg::*;
                    use ibc_union_spec::datagram::Datagram;

                    let mk_msg = |msg| {
                        mk_any(&protos::cosmwasm::wasm::v1::MsgExecuteContract {
                            sender: signer.to_string(),
                            contract: ibc_host_contract_address.to_string(),
                            msg: serde_json::to_vec(&msg).unwrap(),
                            funds: vec![],
                        })
                    };

                    match msg {
                        Datagram::CreateClient(msg_create_client) => {
                            mk_msg(ExecuteMsg::CreateClient(MsgCreateClient {
                                client_type: msg_create_client.client_type.to_string(),
                                client_state_bytes: msg_create_client.client_state_bytes,
                                consensus_state_bytes: msg_create_client.consensus_state_bytes,
                                relayer: fee_recipient
                                    .map_or(signer.to_string(), |s| s.to_string()),
                            }))
                        }
                        Datagram::UpdateClient(msg_update_client) => {
                            mk_msg(ExecuteMsg::UpdateClient(MsgUpdateClient {
                                client_id: msg_update_client.client_id,
                                client_message: msg_update_client.client_message,
                                relayer: fee_recipient
                                    .map_or(signer.to_string(), |s| s.to_string()),
                            }))
                        }
                        Datagram::ConnectionOpenInit(msg_connection_open_init) => {
                            mk_msg(ExecuteMsg::ConnectionOpenInit(MsgConnectionOpenInit {
                                client_id: msg_connection_open_init.client_id,
                                counterparty_client_id: msg_connection_open_init
                                    .counterparty_client_id,
                            }))
                        }
                        Datagram::ConnectionOpenTry(msg_connection_open_try) => {
                            mk_msg(ExecuteMsg::ConnectionOpenTry(MsgConnectionOpenTry {
                                counterparty_client_id: msg_connection_open_try
                                    .counterparty_client_id,
                                counterparty_connection_id: msg_connection_open_try
                                    .counterparty_connection_id,
                                client_id: msg_connection_open_try.client_id,
                                proof_init: msg_connection_open_try.proof_init,
                                proof_height: msg_connection_open_try.proof_height,
                            }))
                        }
                        Datagram::ConnectionOpenAck(msg_connection_open_ack) => {
                            mk_msg(ExecuteMsg::ConnectionOpenAck(MsgConnectionOpenAck {
                                connection_id: msg_connection_open_ack.connection_id,
                                counterparty_connection_id: msg_connection_open_ack
                                    .counterparty_connection_id,
                                proof_try: msg_connection_open_ack.proof_try,
                                proof_height: msg_connection_open_ack.proof_height,
                            }))
                        }
                        Datagram::ConnectionOpenConfirm(msg_connection_open_confirm) => mk_msg(
                            ExecuteMsg::ConnectionOpenConfirm(MsgConnectionOpenConfirm {
                                connection_id: msg_connection_open_confirm.connection_id,
                                proof_ack: msg_connection_open_confirm.proof_ack,
                                proof_height: msg_connection_open_confirm.proof_height,
                            }),
                        ),
                        Datagram::ChannelOpenInit(msg_channel_open_init) => {
                            mk_msg(ExecuteMsg::ChannelOpenInit(MsgChannelOpenInit {
                                port_id: parse_port_id(msg_channel_open_init.port_id.to_vec())?,
                                relayer: fee_recipient
                                    .map_or(signer.to_string(), |s| s.to_string()),
                                counterparty_port_id: msg_channel_open_init.counterparty_port_id,
                                connection_id: msg_channel_open_init.connection_id,
                                version: msg_channel_open_init.version,
                            }))
                        }
                        Datagram::ChannelOpenTry(msg_channel_open_try) => {
                            mk_msg(ExecuteMsg::ChannelOpenTry(MsgChannelOpenTry {
                                port_id: parse_port_id(msg_channel_open_try.port_id.to_vec())?,
                                channel: msg_channel_open_try.channel,
                                counterparty_version: msg_channel_open_try.counterparty_version,
                                proof_init: msg_channel_open_try.proof_init,
                                proof_height: msg_channel_open_try.proof_height,
                                relayer: fee_recipient
                                    .map_or(signer.to_string(), |s| s.to_string()),
                            }))
                        }
                        Datagram::ChannelOpenAck(msg_channel_open_ack) => {
                            mk_msg(ExecuteMsg::ChannelOpenAck(MsgChannelOpenAck {
                                channel_id: msg_channel_open_ack.channel_id,
                                counterparty_version: msg_channel_open_ack.counterparty_version,
                                counterparty_channel_id: msg_channel_open_ack
                                    .counterparty_channel_id,
                                proof_try: msg_channel_open_ack.proof_try,
                                proof_height: msg_channel_open_ack.proof_height,
                                relayer: fee_recipient
                                    .map_or(signer.to_string(), |s| s.to_string()),
                            }))
                        }
                        Datagram::ChannelOpenConfirm(msg_channel_open_confirm) => {
                            mk_msg(ExecuteMsg::ChannelOpenConfirm(MsgChannelOpenConfirm {
                                channel_id: msg_channel_open_confirm.channel_id,
                                proof_ack: msg_channel_open_confirm.proof_ack,
                                proof_height: msg_channel_open_confirm.proof_height,
                                relayer: fee_recipient
                                    .map_or(signer.to_string(), |s| s.to_string()),
                            }))
                        }
                        Datagram::ChannelCloseInit(_msg_channel_close_init) => todo!(),
                        Datagram::ChannelCloseConfirm(_msg_channel_close_confirm) => {
                            todo!()
                        }
                        Datagram::PacketRecv(msg_packet_recv) => {
                            mk_msg(ExecuteMsg::PacketRecv(MsgPacketRecv {
                                packets: msg_packet_recv.packets.into_iter().collect(),
                                relayer_msgs: msg_packet_recv.relayer_msgs,
                                proof: msg_packet_recv.proof,
                                proof_height: msg_packet_recv.proof_height,
                                relayer: fee_recipient
                                    .map_or(signer.to_string(), |s| s.to_string()),
                            }))
                        }
                        Datagram::PacketAcknowledgement(msg_packet_acknowledgement) => {
                            mk_msg(ExecuteMsg::PacketAck(MsgPacketAcknowledgement {
                                packets: msg_packet_acknowledgement.packets.into_iter().collect(),
                                acknowledgements: msg_packet_acknowledgement.acknowledgements,
                                proof: msg_packet_acknowledgement.proof,
                                proof_height: msg_packet_acknowledgement.proof_height,
                                relayer: fee_recipient
                                    .map_or(signer.to_string(), |s| s.to_string()),
                            }))
                        }
                        Datagram::PacketTimeout(msg_packet_timeout) => {
                            mk_msg(ExecuteMsg::PacketTimeout(MsgPacketTimeout {
                                packet: msg_packet_timeout.packet,
                                proof: msg_packet_timeout.proof,
                                proof_height: msg_packet_timeout.proof_height,
                                relayer: fee_recipient
                                    .map_or(signer.to_string(), |s| s.to_string()),
                            }))
                        }
                        Datagram::IntentPacketRecv(msg_intent_packet_recv) => {
                            mk_msg(ExecuteMsg::IntentPacketRecv(MsgIntentPacketRecv {
                                packets: msg_intent_packet_recv.packets.into_iter().collect(),
                                market_maker_msgs: msg_intent_packet_recv.market_maker_messages,
                                market_maker: fee_recipient
                                    .map_or(signer.to_string(), |s| s.to_string()),
                            }))
                        }
                        Datagram::BatchSend(msg_batch_send) => {
                            mk_msg(ExecuteMsg::BatchSend(MsgBatchSend {
                                packets: msg_batch_send.packets,
                            }))
                        }
                        Datagram::BatchAcks(_msg_batch_acks) => todo!(),
                        Datagram::CommitMembershipProof(msg_commit_membership_proof) => mk_msg(
                            ExecuteMsg::CommitMembershipProof(MsgCommitMembershipProof {
                                client_id: msg_commit_membership_proof.client_id,
                                proof_height: msg_commit_membership_proof.proof_height,
                                proof: msg_commit_membership_proof.proof,
                                path: msg_commit_membership_proof.path,
                                value: msg_commit_membership_proof.value,
                            }),
                        ),
                        Datagram::CommitNonMembershipProof(msg_commit_non_membership_proof) => {
                            mk_msg(ExecuteMsg::CommitNonMembershipProof(
                                MsgCommitNonMembershipProof {
                                    client_id: msg_commit_non_membership_proof.client_id,
                                    proof_height: msg_commit_non_membership_proof.proof_height,
                                    proof: msg_commit_non_membership_proof.proof,
                                    path: msg_commit_non_membership_proof.path,
                                },
                            ))
                        }
                    }
                }
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

#[cfg(test)]
mod tests {
    use concurrent_keyring::KeyringConfigEntry;

    use super::*;

    #[test]
    fn test_parse_wasm_failure() {
        let (idx, log) = parse_msg_idx_from_log("rpc error: code = Unknown desc = failed to execute message; message index: 0: IBC_UNION_ERR_PACKET_COMMITMENT_NOT_FOUND packet commitment not found: execute wasm contract failed [CosmWasm/wasmd@v0.53.2/x/wasm/keeper/keeper.go:436] with gas used: '287090'").unwrap();

        dbg!(idx, parse_wasm_failure(log));
    }

    #[test]
    fn config_parse() {
        let json = r#"{
            "chain_id": "chain_id",
            "gas_config": {
              "config": {
                "max_gas": 123456789,
                "gas_multiplier": "1.4"
              },
              "type": "feemarket"
            },
            "ibc_host_contract_address": "union1hnuj8f6d3wy3fcprt55vddv7v2650t6uudnvd2hukqrteeam8wjqvcmecf",
            "keyring": {
              "keys": [
                {
                  "key": "0x0000000000000000000000000000000000000000000000000000000000000000",
                  "name": "name",
                  "type": "raw"
                }
              ],
              "name": "name"
            },
            "rpc_url": "rpc_url",
            "max_tx_size": 1000000
          }"#;

        let config = serde_json::from_str::<Config>(json).unwrap();

        assert_eq!(
            config,
            Config {
                chain_id: ChainId::new("chain_id"),
                ibc_host_contract_address:
                    "union1hnuj8f6d3wy3fcprt55vddv7v2650t6uudnvd2hukqrteeam8wjqvcmecf"
                        .parse()
                        .unwrap(),
                keyring: KeyringConfig {
                    name: "name".to_string(),
                    keys: vec![KeyringConfigEntry::Raw {
                        name: "name".to_string(),
                        key: vec![0; 32],
                    }]
                },
                rpc_url: "rpc_url".to_string(),
                gas_config: GasFillerConfig::Feemarket(FeemarketConfig {
                    max_gas: 123456789,
                    gas_multiplier: Some(1.4),
                    denom: None
                }),
                fatal_errors: HashMap::default(),
                gas_station_config: vec![],
                fee_recipient: None,
                max_tx_size: 1000000
            }
        );
    }
}

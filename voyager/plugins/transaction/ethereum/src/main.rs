use std::{
    collections::{BTreeMap, VecDeque},
    ops::Deref,
    panic::AssertUnwindSafe,
    sync::Arc,
};

use alloy::{
    contract::{Error, RawCallBuilder},
    network::{AnyNetwork, EthereumWallet},
    primitives::Address,
    providers::{
        fillers::RecommendedFillers, layers::CacheLayer, DynProvider, PendingTransactionError,
        Provider, ProviderBuilder,
    },
    signers::local::LocalSigner,
    sol_types::{SolEvent, SolInterface},
    transports::TransportError,
};
use bip32::secp256k1::ecdsa::{self, SigningKey};
use clap::Subcommand;
use concurrent_keyring::{ConcurrentKeyring, KeyringConfig, KeyringEntry};
use ibc_solidity::Ibc::{self, IbcErrors};
use ibc_union_spec::{datagram::Datagram, IbcUnion};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    proc_macros::rpc,
    types::{ErrorObject, ErrorObjectOwned},
    Extensions, MethodsError,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{error, info, info_span, instrument, trace, warn, Instrument};
use unionlabs::{
    primitives::{H160, H256, U256},
    ErrorReporter,
};
use voyager_message::{
    data::Data,
    hook::SubmitTxHook,
    into_value,
    module::{PluginInfo, PluginServer},
    primitives::ChainId,
    vm::{call, defer, now, pass::PassResult, seq, BoxDynError, Op, Visit},
    Plugin, PluginMessage, VoyagerMessage, FATAL_JSONRPC_ERROR_CODE,
};

use crate::{
    call::ModuleCall,
    callback::ModuleCallback,
    multicall::{Call3, Multicall, MulticallResult},
};

pub mod call;
pub mod callback;
pub mod data;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module(Arc<ModuleInner>);

impl Deref for Module {
    type Target = ModuleInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct ModuleInner {
    pub chain_id: ChainId,
    pub additional_chain_ids: Vec<ChainId>,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,
    pub multicall_address: H160,

    pub provider: DynProvider<AnyNetwork>,

    pub keyring: ConcurrentKeyring<alloy::primitives::Address, LocalSigner<SigningKey>>,

    pub max_gas_price: Option<u128>,

    pub fixed_gas_price: Option<u128>,

    pub legacy: bool,

    pub fee_recipient: Option<alloy::primitives::Address>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId,
    #[serde(default)]
    pub additional_chain_ids: Vec<ChainId>,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,
    pub multicall_address: H160,

    /// The RPC endpoint for the execution chain.
    pub rpc_url: String,

    pub keyring: KeyringConfig,

    #[serde(default)]
    pub max_gas_price: Option<u128>,

    /// Temporary fix for 0g until they fix their eth_feeHistory endpoint
    #[serde(default)]
    pub fixed_gas_price: Option<u128>,

    #[serde(default)]
    pub legacy: bool,

    #[serde(default)]
    pub max_cache_size: u32,

    #[serde(default)]
    pub fee_recipient: Option<alloy::primitives::Address>,
}

#[derive(Subcommand)]
pub enum Cmd {
    SignerAddresses,
    SignerBalances,
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = Cmd;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        let provider = DynProvider::new(
            ProviderBuilder::new()
                .network::<AnyNetwork>()
                .layer(CacheLayer::new(config.max_cache_size))
                .connect(&config.rpc_url)
                .await?,
        );

        let raw_chain_id = provider.get_chain_id().await?;
        let chain_id = ChainId::new(raw_chain_id.to_string());

        if chain_id != config.chain_id {
            return Err(format!(
                "incorrect chain id: expected `{}`, but found `{}`",
                config.chain_id, chain_id
            )
            .into());
        }

        Ok(Self(Arc::new(ModuleInner {
            chain_id,
            additional_chain_ids: config.additional_chain_ids,
            ibc_handler_address: config.ibc_handler_address,
            multicall_address: config.multicall_address,
            provider,
            keyring: ConcurrentKeyring::new(
                config.keyring.name,
                config.keyring.keys.into_iter().map(|config| {
                    let signing_key = <ecdsa::SigningKey as bip32::PrivateKey>::from_bytes(
                        &config.value().as_slice().try_into().unwrap(),
                    )
                    .unwrap();

                    let signer = LocalSigner::from_signing_key(signing_key);

                    KeyringEntry {
                        address: signer.address(),
                        signer,
                    }
                }),
            ),
            max_gas_price: config.max_gas_price,
            fixed_gas_price: config.fixed_gas_price,
            legacy: config.legacy,
            fee_recipient: config.fee_recipient,
        })))
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.chain_id),
            interest_filter: SubmitTxHook::filter_many(
                [&config.chain_id]
                    .into_iter()
                    .chain(&config.additional_chain_ids),
            ),
        }
    }

    async fn cmd(config: Self::Config, cmd: Self::Cmd) {
        let plugin = Self::new(config).await.unwrap();

        match cmd {
            Cmd::SignerAddresses => {
                println!("{}", into_value(plugin.keyring.keys().collect::<Vec<_>>()))
            }
            Cmd::SignerBalances => {
                let mut out = BTreeMap::new();

                for address in plugin.keyring.keys() {
                    let balance = plugin.provider.get_balance(*address).await.unwrap();

                    out.insert(address, balance);
                }

                println!("{}", into_value(out))
            }
        }
    }
}

#[rpc(server)]
trait TransactionPlugin {
    #[method(name = "signerAddresses")]
    async fn signer_addresses(&self) -> RpcResult<Vec<Address>>;

    #[method(name = "signerBalances")]
    async fn signer_balances(&self) -> RpcResult<BTreeMap<Address, U256>>;
}

#[async_trait]
impl TransactionPluginServer for Module {
    async fn signer_addresses(&self) -> RpcResult<Vec<Address>> {
        Ok(self.keyring.keys().cloned().collect())
    }

    async fn signer_balances(&self) -> RpcResult<BTreeMap<Address, U256>> {
        let mut out = BTreeMap::new();

        for address in self.keyring.keys() {
            let balance = self.provider.get_balance(*address).await.map_err(|e| {
                ErrorObject::owned(
                    -1,
                    ErrorReporter(e).with_message("error fetching balance"),
                    None::<()>,
                )
            })?;

            out.insert(*address, balance.into());
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
}

#[derive(Debug, thiserror::Error)]
pub enum TxSubmitError {
    #[error(transparent)]
    Error(#[from] Error),
    #[error("error estimating gas")]
    Estimate(#[source] Error),
    #[error("error waiting for transaction")]
    PendingTransactionError(#[from] PendingTransactionError),
    #[error("out of gas")]
    OutOfGas,
    #[error("0x revert")]
    EmptyRevert(Vec<Datagram>),
    #[error("gas price is too high: max {max}, price {price}")]
    GasPriceTooHigh { max: u128, price: u128 },
    #[error("rpc error (this is just the IbcDatagram conversion functions but i need to make those errors better)")]
    RpcError(#[from] ErrorObjectOwned),
    #[error("batch too large")]
    BatchTooLarge,
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
                            ModuleCall::SubmitMulticall(
                                submit_tx
                                    .datagrams
                                    .clone()
                                    .into_iter()
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
            ModuleCall::SubmitMulticall(mut msgs) => {
                let res = self
                    .keyring
                    .with({
                        let msgs = msgs.clone();
                        move |wallet| -> _ {
                            // let call = if self.legacy { call.legacy() } else { call };
                            AssertUnwindSafe(self.submit_transaction(wallet, msgs))
                        }
                    })
                    .await;

                let rewrap_msg = || {
                    PluginMessage::new(
                        self.plugin_name(),
                        ModuleCall::SubmitMulticall(msgs.clone()),
                    )
                };

                match res {
                    Some(Ok(())) => Ok(Op::Noop),
                    Some(Err(TxSubmitError::GasPriceTooHigh { .. })) => {
                        Ok(seq([defer(now() + 6), call(rewrap_msg())]))
                    }
                    Some(Err(TxSubmitError::OutOfGas)) => {
                        Ok(seq([defer(now() + 12), call(rewrap_msg())]))
                    }
                    Some(Err(TxSubmitError::EmptyRevert(msgs))) => Ok(seq([
                        defer(now() + 12),
                        call(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::SubmitMulticall(msgs),
                        )),
                    ])),
                    Some(Err(TxSubmitError::BatchTooLarge)) => {
                        let new = msgs.split_off(msgs.len() / 2);
                        Ok(seq([
                            call(PluginMessage::new(
                                self.plugin_name(),
                                ModuleCall::SubmitMulticall(msgs),
                            )),
                            call(PluginMessage::new(
                                self.plugin_name(),
                                ModuleCall::SubmitMulticall(new),
                            )),
                        ]))
                    }
                    Some(Err(err)) => Err(ErrorObject::owned(
                        -1,
                        ErrorReporter(err).to_string(),
                        None::<()>,
                    )),
                    None => Ok(call(rewrap_msg())),
                }
            }
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

impl Module {
    async fn submit_transaction(
        &self,
        wallet: &LocalSigner<SigningKey>,
        ibc_messages: Vec<Datagram>,
    ) -> Result<(), TxSubmitError> {
        let signer = DynProvider::new(
            ProviderBuilder::new()
                .network::<AnyNetwork>()
                .filler(AnyNetwork::recommended_fillers())
                // .filler(<NonceFiller>::default())
                // .filler(ChainIdFiller::default())
                .wallet(EthereumWallet::new(wallet.clone()))
                .on_provider(self.provider.clone()),
        );

        if let Some(max_gas_price) = self.max_gas_price {
            let gas_price = self
                .provider
                .get_gas_price()
                .await
                .expect("unable to fetch gas price");

            if gas_price > max_gas_price {
                warn!(%max_gas_price, %gas_price, "gas price is too high");

                return Err(TxSubmitError::GasPriceTooHigh {
                    max: max_gas_price,
                    price: gas_price,
                });
            }
        }

        let multicall = Multicall::new(self.multicall_address.into(), signer.clone());

        let ibc = Ibc::new(self.ibc_handler_address.into(), &self.provider);

        let msgs = process_msgs(
            &ibc,
            ibc_messages,
            self.fee_recipient.unwrap_or(wallet.address()).into(),
        )?;

        trace!(?msgs);

        let msg_names = msgs
            .iter()
            // .map(|x| (x.0.clone(), x.1.function.name.clone()))
            .map(|x| (x.0.clone(), x.0.name()))
            .collect::<Vec<_>>();

        let mut call = multicall.multicall(
            msgs.clone()
                .into_iter()
                .map(|(_, call)| Call3 {
                    target: self.ibc_handler_address.into(),
                    allowFailure: true,
                    callData: call.calldata().clone(),
                })
                .collect(),
        );

        info!("submitting evm tx");

        let gas_estimate = call.estimate_gas().await.map_err(|e| {
            if ErrorReporter(&e)
                .to_string()
                .contains("gas required exceeds")
            {
                TxSubmitError::BatchTooLarge
            } else {
                TxSubmitError::Estimate(e)
            }
        })?;
        //     .map_err(|e| {
        //     ErrorObject::owned(
        //         -1,
        //         format!("error estimating gas: {}", ErrorReporter(e), None::<()>),
        //     )
        // })?;

        let gas_to_use = gas_estimate + (gas_estimate / 2);

        info!(gas_estimate, gas_to_use, "gas estimatation successful");

        if let Some(fixed_gas_price) = self.fixed_gas_price {
            call = call.gas_price(fixed_gas_price);
        }

        match call.gas(gas_to_use).send().await {
            Ok(ok) => {
                let tx_hash = <H256>::from(*ok.tx_hash());
                async move {
                    let receipt = ok.get_receipt().await?;

                    info!(%tx_hash, "tx included");

                    let result = MulticallResult::decode_log_data(
                        receipt
                            .inner
                            .inner
                            .logs()
                            .last()
                            .expect("multicall event should be last log")
                            .data(),
                        true,
                    )
                    .expect("unable to decode multicall result log");

                    info!(
                        gas_used = %receipt.gas_used,
                        batch.size = msg_names.len(),
                        "submitted batched evm messages"
                    );

                    for (idx, (result, (msg, msg_name))) in
                        result._0.into_iter().zip(msg_names).enumerate()
                    {
                        if result.success {
                            info!(
                                msg = msg_name,
                                %idx,
                                data = %serde_json::to_string(&msg).unwrap(),
                                "evm tx",
                            );
                        } else if let Ok(known_revert) =
                            IbcErrors::abi_decode(&result.returnData, true)
                        {
                            error!(
                                msg = %msg_name,
                                %idx,
                                revert = ?known_revert,
                                well_known = true,
                                data = %serde_json::to_string(&msg).unwrap(),
                                "evm message failed",
                            );
                        } else if result.returnData.is_empty() {
                            error!(
                                msg = %msg_name,
                                %idx,
                                revert = %result.returnData,
                                well_known = false,
                                data = %serde_json::to_string(&msg).unwrap(),
                                "evm message failed with 0x revert, likely an ABI issue",
                            );
                        } else {
                            error!(
                                msg = %msg_name,
                                %idx,
                                revert = %result.returnData,
                                well_known = false,
                                data = %serde_json::to_string(&msg).unwrap(),
                                "evm message failed",
                            );
                        }
                    }

                    Ok(())
                }
                .instrument(info_span!(
                    "evm tx",
                    %tx_hash,
                ))
                .await
            }
            Err(
                Error::PendingTransactionError(PendingTransactionError::TransportError(
                    TransportError::ErrorResp(e),
                ))
                | Error::TransportError(TransportError::ErrorResp(e)),
            ) if e
                .message
                .contains("insufficient funds for gas * price + value") =>
            {
                error!("out of gas");
                Err(TxSubmitError::OutOfGas)
            }
            Err(
                Error::PendingTransactionError(PendingTransactionError::TransportError(
                    TransportError::ErrorResp(e),
                ))
                | Error::TransportError(TransportError::ErrorResp(e)),
            ) if e.message.contains("oversized data")
                || e.message.contains("exceeds block gas limit")
                || e.message.contains("gas required exceeds") =>
            {
                if msgs.len() == 1 {
                    error!(error = %e.message, msg = ?msgs[0], "message is too large");
                    Ok(()) // drop the message
                } else {
                    warn!(error = %e.message, "batch is too large");
                    Err(TxSubmitError::BatchTooLarge)
                }
            }
            Err(err) => Err(TxSubmitError::Error(err)),
        }
    }
}

#[allow(clippy::type_complexity)]
fn process_msgs<'a>(
    ibc_handler: &'a ibc_solidity::Ibc::IbcInstance<(), &'a DynProvider<AnyNetwork>, AnyNetwork>,
    msgs: Vec<Datagram>,
    relayer: H160,
) -> RpcResult<
    Vec<(
        Datagram,
        RawCallBuilder<(), &'a &'a DynProvider<AnyNetwork>, AnyNetwork>,
    )>,
> {
    trace!(?msgs);

    msgs.clone()
        .into_iter()
        .map(|msg| {
            Ok(match msg.clone() {
                Datagram::CreateClient(data) => (
                    msg,
                    ibc_handler
                        .createClient(ibc_solidity::MsgCreateClient {
                            client_type: data.client_type.to_string(),
                            client_state_bytes: data.client_state_bytes.into(),
                            consensus_state_bytes: data.consensus_state_bytes.into(),
                            relayer: relayer.into(),
                        })
                        .clear_decoder(),
                ),
                Datagram::UpdateClient(data) => (
                    msg,
                    ibc_handler
                        .updateClient(ibc_solidity::MsgUpdateClient {
                            client_id: data.client_id.raw(),
                            client_message: data.client_message.into(),
                            relayer: relayer.into(),
                        })
                        .clear_decoder(),
                ),
                Datagram::ConnectionOpenInit(data) => (
                    msg,
                    ibc_handler
                        .connectionOpenInit(ibc_solidity::MsgConnectionOpenInit {
                            client_id: data.client_id.raw(),
                            counterparty_client_id: data.counterparty_client_id.raw(),
                        })
                        .clear_decoder(),
                ),
                Datagram::ConnectionOpenTry(data) => (
                    msg,
                    ibc_handler
                        .connectionOpenTry(ibc_solidity::MsgConnectionOpenTry {
                            counterparty_client_id: data.counterparty_client_id.raw(),
                            counterparty_connection_id: data.counterparty_connection_id.raw(),
                            client_id: data.client_id.raw(),
                            proof_init: data.proof_init.into(),
                            proof_height: data.proof_height,
                        })
                        .clear_decoder(),
                ),
                Datagram::ConnectionOpenAck(data) => (
                    msg,
                    ibc_handler
                        .connectionOpenAck(ibc_solidity::MsgConnectionOpenAck {
                            connection_id: data.connection_id.raw(),
                            counterparty_connection_id: data.counterparty_connection_id.raw(),
                            proof_height: data.proof_height,
                            proof_try: data.proof_try.into(),
                        })
                        .clear_decoder(),
                ),
                Datagram::ConnectionOpenConfirm(data) => (
                    msg,
                    ibc_handler
                        .connectionOpenConfirm(ibc_solidity::MsgConnectionOpenConfirm {
                            connection_id: data.connection_id.raw(),
                            proof_ack: data.proof_ack.into(),
                            proof_height: data.proof_height,
                        })
                        .clear_decoder(),
                ),
                Datagram::ChannelOpenInit(data) => (
                    msg,
                    ibc_handler
                        .channelOpenInit(ibc_solidity::MsgChannelOpenInit {
                            port_id: data.port_id.try_into().unwrap(),
                            relayer: relayer.into(),
                            counterparty_port_id: data.counterparty_port_id.into(),
                            connection_id: data.connection_id.raw(),
                            version: data.version,
                        })
                        .clear_decoder(),
                ),
                Datagram::ChannelOpenTry(data) => (
                    msg,
                    ibc_handler
                        .channelOpenTry(ibc_solidity::MsgChannelOpenTry {
                            port_id: data.port_id.try_into().unwrap(),
                            channel: data.channel.into(),
                            counterparty_version: data.counterparty_version,
                            proof_init: data.proof_init.into(),
                            proof_height: data.proof_height,
                            relayer: relayer.into(),
                        })
                        .clear_decoder(),
                ),
                Datagram::ChannelOpenAck(data) => (
                    msg,
                    ibc_handler
                        .channelOpenAck(ibc_solidity::MsgChannelOpenAck {
                            channel_id: data.channel_id.raw(),
                            counterparty_version: data.counterparty_version,
                            counterparty_channel_id: data.counterparty_channel_id.raw(),
                            proof_try: data.proof_try.into(),
                            proof_height: data.proof_height,
                            relayer: relayer.into(),
                        })
                        .clear_decoder(),
                ),
                Datagram::ChannelOpenConfirm(data) => (
                    msg,
                    ibc_handler
                        .channelOpenConfirm(ibc_solidity::MsgChannelOpenConfirm {
                            channel_id: data.channel_id.raw(),
                            proof_ack: data.proof_ack.into(),
                            proof_height: data.proof_height,
                            relayer: relayer.into(),
                        })
                        .clear_decoder(),
                ),
                Datagram::PacketRecv(data) => (
                    msg,
                    ibc_handler
                        .recvPacket(ibc_solidity::MsgPacketRecv {
                            packets: data.packets.into_iter().map(Into::into).collect(),
                            proof: data.proof.into(),
                            proof_height: data.proof_height,
                            relayer: relayer.into(),
                            relayer_msgs: data.relayer_msgs.into_iter().map(Into::into).collect(),
                        })
                        .clear_decoder(),
                ),
                Datagram::PacketAcknowledgement(data) => (
                    msg,
                    ibc_handler
                        .acknowledgePacket(ibc_solidity::MsgPacketAcknowledgement {
                            packets: data.packets.into_iter().map(Into::into).collect(),
                            acknowledgements: data
                                .acknowledgements
                                .into_iter()
                                .map(Into::into)
                                .collect(),
                            proof: data.proof.into(),
                            proof_height: data.proof_height,
                            relayer: relayer.into(),
                        })
                        .clear_decoder(),
                ),
                Datagram::PacketTimeout(data) => (
                    msg,
                    ibc_handler
                        .timeoutPacket(ibc_solidity::MsgPacketTimeout {
                            packet: data.packet.into(),
                            proof: data.proof.into(),
                            proof_height: data.proof_height,
                            relayer: relayer.into(),
                        })
                        .clear_decoder(),
                ),
                _ => todo!(),
            })
        })
        .collect()
}

pub mod multicall {
    alloy::sol! {
        #![sol(rpc)]

        struct Call3 {
            address target;
            bool allowFailure;
            bytes callData;
        }

        #[derive(Debug)]
        struct Result {
            bool success;
            bytes returnData;
        }

        #[derive(Debug)]
        event MulticallResult(Result[]);

        contract Multicall {
            function multicall(
                Call3[] calldata calls
            ) public payable returns (Result[] memory returnData);
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy::{
        hex,
        primitives::{fixed_bytes, LogData},
    };

    use super::*;

    #[test]
    fn multicall_result_decode() {
        let bz = hex::decode("0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000004").unwrap();

        let result = MulticallResult::decode_log_data(
            &LogData::new(
                [fixed_bytes!(
                    "798f59b5fbedbc6b92c366aebbe4ef378956a3a1b9ff4a1ba0760f3d0752a883"
                )]
                .to_vec(),
                bz.into(),
            )
            .unwrap(),
            true,
        )
        .unwrap();

        dbg!(result);
    }

    // TODO: rename of the event broke the test indeed
    // #[test]
    // fn create_client_decode() {
    //     let bz = hex::decode("0x000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008636f6d6574626c73000000000000000000000000000000000000000000000000").unwrap();

    //     let result = CreateClient::decode_log_data(
    //         &LogData::new(
    //             [fixed_bytes!(
    //                 "04e9540749029ffe9d24e5bd373d2e18bf4fab8f13c60b4a62b9ae8562920cc8"
    //             )]
    //             .to_vec(),
    //             bz.into(),
    //         )
    //         .unwrap(),
    //         true,
    //     )
    //     .unwrap();

    //     dbg!(result);
    // }
}

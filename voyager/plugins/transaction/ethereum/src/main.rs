use std::collections::VecDeque;

use alloy::{
    contract::{Error, RawCallBuilder},
    network::EthereumWallet,
    providers::{PendingTransactionError, Provider, ProviderBuilder, RootProvider},
    signers::local::LocalSigner,
    sol_types::{SolEvent, SolInterface},
    transports::{BoxTransport, Transport, TransportError},
};
use bip32::secp256k1::ecdsa::{self, SigningKey};
use chain_utils::{
    keyring::{ConcurrentKeyring, KeyringConfig, KeyringEntry},
    BoxDynError,
};
use ibc_solidity::Ibc::{self, IbcErrors};
use ibc_union_spec::{Datagram, IbcUnion};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::{ErrorObject, ErrorObjectOwned},
    Extensions,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info, info_span, instrument, warn, Instrument};
use unionlabs::{
    hash::{H160, H256},
    ErrorReporter,
};
use voyager_message::{
    core::{ChainId, IbcSpec},
    data::{Data, WithChainId},
    module::{PluginInfo, PluginServer},
    DefaultCmd, Plugin, PluginMessage, VoyagerMessage, FATAL_JSONRPC_ERROR_CODE,
};
use voyager_vm::{call, defer, now, pass::PassResult, seq, Op};

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
pub struct Module {
    pub chain_id: ChainId,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,
    pub multicall_address: H160,

    pub provider: RootProvider<BoxTransport>,

    pub keyring: ConcurrentKeyring<alloy::primitives::Address, LocalSigner<SigningKey>>,

    pub max_gas_price: Option<u128>,
    pub legacy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,
    pub multicall_address: H160,

    /// The RPC endpoint for the execution chain.
    pub eth_rpc_api: String,

    pub keyring: KeyringConfig,

    #[serde(default)]
    pub max_gas_price: Option<u128>,

    #[serde(default)]
    pub legacy: bool,
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        let provider = ProviderBuilder::new()
            .on_builtin(&config.eth_rpc_api)
            .await?;

        let raw_chain_id = provider.get_chain_id().await?;
        let chain_id = ChainId::new(raw_chain_id.to_string());

        if chain_id != config.chain_id {
            return Err(format!(
                "incorrect chain id: expected `{}`, but found `{}`",
                config.chain_id, chain_id
            )
            .into());
        }

        Ok(Self {
            chain_id,
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
                        name: config.name(),
                        address: signer.address(),
                        signer,
                    }
                }),
            ),
            max_gas_price: config.max_gas_price,
            legacy: config.legacy,
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
    ($data."@type" == "identified_ibc_datagram"
        and $data."@value".chain_id == "{chain_id}"
        and $data."@value".message.ibc_spec_id == "{ibc_spec_id}")
    or ($data."@type" == "identified_ibc_datagram_batch"
        and $data."@value".chain_id == "{chain_id}"
        and all($data."@value".message[] | select(.ibc_spec_id == "{ibc_spec_id}")))
else
    false
end
"#,
                chain_id = config.chain_id,
                ibc_spec_id = IbcUnion::ID,
            ),
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
}

#[derive(Debug, thiserror::Error)]
pub enum TxSubmitError {
    #[error(transparent)]
    Error(#[from] Error),
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
                .map(|(idx, msg)| {
                    Ok((
                        vec![idx],
                        match msg {
                            Op::Data(Data::IdentifiedIbcDatagram(WithChainId {
                                chain_id,
                                message,
                            })) => {
                                assert_eq!(chain_id, self.chain_id);

                                call(PluginMessage::new(
                                    self.plugin_name(),
                                    ModuleCall::SubmitMulticall(vec![message
                                        .decode_datagram::<IbcUnion>()
                                        .unwrap()
                                        .map_err(|e| {
                                            ErrorObject::owned(
                                                FATAL_JSONRPC_ERROR_CODE,
                                                format!(
                                                    "unable to deserialize datagram: {}",
                                                    ErrorReporter(e)
                                                ),
                                                None::<()>,
                                            )
                                        })?]),
                                ))
                            }
                            Op::Data(Data::IdentifiedIbcDatagramBatch(WithChainId {
                                chain_id,
                                message,
                            })) => {
                                assert_eq!(chain_id, self.chain_id);

                                call(PluginMessage::new(
                                    self.plugin_name(),
                                    ModuleCall::SubmitMulticall(
                                        message
                                            .into_iter()
                                            .map(|message| {
                                                message
                                                    .decode_datagram::<IbcUnion>()
                                                    .unwrap()
                                                    .map_err(|e| {
                                                        ErrorObject::owned(
                                                            FATAL_JSONRPC_ERROR_CODE,
                                                            format!(
                                                            "unable to deserialize datagram: {}",
                                                            ErrorReporter(e)
                                                        ),
                                                            None::<()>,
                                                        )
                                                    })
                                            })
                                            .collect::<Result<_, _>>()?,
                                    ),
                                ))
                            }
                            _ => panic!("unexpected message: {msg:?}"),
                        },
                    ))
                })
                .collect::<RpcResult<_>>()?,
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn call(&self, _: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::SubmitMulticall(msgs) => {
                let res = self
                    .keyring
                    .with({
                        let msgs = msgs.clone();
                        move |wallet| -> _ {
                            // let call = if self.legacy { call.legacy() } else { call };
                            self.submit_transaction(wallet, msgs)
                        }
                    })
                    .await;

                let rewrap_msg =
                    || PluginMessage::new(self.plugin_name(), ModuleCall::SubmitMulticall(msgs));

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
}

impl Module {
    async fn submit_transaction(
        &self,
        wallet: &LocalSigner<SigningKey>,
        ibc_messages: Vec<Datagram>,
    ) -> Result<(), TxSubmitError> {
        let signer = ProviderBuilder::new()
            .with_recommended_fillers()
            // .filler(<NonceFiller>::default())
            // .filler(ChainIdFiller::default())
            .wallet(EthereumWallet::new(wallet.clone()))
            .on_provider(self.provider.clone());

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

        let msgs = process_msgs(&ibc, ibc_messages, wallet.address().0.into())?;

        dbg!(&msgs);

        let msg_names = msgs
            .iter()
            // .map(|x| (x.0.clone(), x.1.function.name.clone()))
            .map(|x| (x.0.clone(), x.0.name()))
            .collect::<Vec<_>>();

        let call = multicall.multicall(
            msgs.into_iter()
                .map(|(_, x)| Call3 {
                    target: self.ibc_handler_address.into(),
                    allowFailure: true,
                    callData: x.calldata().clone(),
                })
                .collect(),
        );

        info!("submitting evm tx");

        match call.gas(15_000_000).send().await {
            Ok(ok) => {
                let tx_hash = <H256>::from(*ok.tx_hash());
                async move {
                    let receipt = ok.get_receipt().await?;

                    info!(%tx_hash, "tx included");

                    let result = MulticallResult::decode_log_data(
                        receipt
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

                    let mut retry_msgs = vec![];

                    for (idx, (result, (msg, msg_name))) in
                        result._0.into_iter().zip(msg_names).enumerate()
                    {
                        if result.success {
                            info_span!(
                                "evm tx",
                                msg = msg_name,
                                %idx,
                                data = %serde_json::to_string(&msg).unwrap(),
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
                                "evm message failed",
                            );

                            retry_msgs.push((true, msg));
                        } else {
                            error!(
                                msg = %msg_name,
                                %idx,
                                revert = %result.returnData,
                                well_known = false,
                                data = %serde_json::to_string(&msg).unwrap(),
                                "evm message failed",
                            );

                            retry_msgs.push((false, msg));
                        }
                    }

                    // NOTE: An empty iterator returns false
                    if retry_msgs
                        .iter()
                        .any(|(is_empty_revert, _)| *is_empty_revert)
                    {
                        Err(TxSubmitError::EmptyRevert(
                            retry_msgs.into_iter().map(|(_, msg)| msg).collect(),
                        ))
                    } else {
                        Ok(())
                    }
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
            Err(err) => Err(TxSubmitError::Error(err)),
        }
    }
}

#[allow(clippy::type_complexity)]
fn process_msgs<T: Transport + Clone, P: Provider<T>>(
    ibc_handler: &ibc_solidity::Ibc::IbcInstance<T, P>,
    msgs: Vec<Datagram>,
    relayer: H160,
) -> RpcResult<Vec<(Datagram, RawCallBuilder<T, &P>)>> {
    dbg!(&msgs);

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
                            client_id: data.client_id,
                            client_message: data.client_message.into(),
                            relayer: relayer.into(),
                        })
                        .clear_decoder(),
                ),
                Datagram::ConnectionOpenInit(data) => (
                    msg,
                    ibc_handler
                        .connectionOpenInit(ibc_solidity::MsgConnectionOpenInit {
                            client_id: data.client_id,
                            counterparty_client_id: data.counterparty_client_id,
                            relayer: relayer.into(),
                        })
                        .clear_decoder(),
                ),
                Datagram::ConnectionOpenTry(data) => (
                    msg,
                    ibc_handler
                        .connectionOpenTry(ibc_solidity::MsgConnectionOpenTry {
                            counterparty_client_id: data.counterparty_client_id,
                            counterparty_connection_id: data.counterparty_connection_id,
                            client_id: data.client_id,
                            proof_init: data.proof_init.into(),
                            proof_height: data.proof_height,
                            relayer: relayer.into(),
                        })
                        .clear_decoder(),
                ),
                Datagram::ConnectionOpenAck(data) => (
                    msg,
                    ibc_handler
                        .connectionOpenAck(ibc_solidity::MsgConnectionOpenAck {
                            connection_id: data.connection_id,
                            counterparty_connection_id: data.counterparty_connection_id,
                            proof_height: data.proof_height,
                            proof_try: data.proof_try.into(),
                            relayer: relayer.into(),
                        })
                        .clear_decoder(),
                ),
                Datagram::ConnectionOpenConfirm(data) => (
                    msg,
                    ibc_handler
                        .connectionOpenConfirm(ibc_solidity::MsgConnectionOpenConfirm {
                            connection_id: data.connection_id,
                            proof_ack: data.proof_ack.into(),
                            proof_height: data.proof_height,
                            relayer: relayer.into(),
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
                            connection_id: data.connection_id,
                            version: data.version,
                        })
                        .clear_decoder(),
                ),
                Datagram::ChannelOpenTry(data) => (
                    msg,
                    ibc_handler
                        .channelOpenTry(ibc_solidity::MsgChannelOpenTry {
                            channel: data.channel,
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
                            channel_id: data.channel_id,
                            counterparty_version: data.counterparty_version,
                            counterparty_channel_id: data.counterparty_channel_id,
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
                            channel_id: data.channel_id,
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
                            packets: data.packets,
                            proof: data.proof.into(),
                            proof_height: data.proof_height,
                            relayer: relayer.into(),
                            relayer_msgs: data.relayer_msgs.into_iter().map(Into::into).collect(),
                        })
                        .clear_decoder(),
                ),
                // Datagram::AcknowledgePacket(data) => (
                //     msg,
                //     ibc_handler
                //         .acknowledgePacket(MsgPacketAcknowledgement {
                //             packets: vec![convert_packet(data.packet)?],
                //             acknowledgements: vec![data.acknowledgement.into()],
                //             proof: data.proof_acked.into(),
                //             proofHeight: data.proof_height.height(),
                //             relayer: relayer.into(),
                //         })
                //         .clear_decoder(),
                // ),
                // Datagram::TimeoutPacket(data) => (
                //     msg,
                //     ibc_handler
                //         .timeoutPacket(MsgPacketTimeout {
                //             packet: convert_packet(data.packet)?,
                //             proof: data.proof_unreceived.into(),
                //             proofHeight: data.proof_height.height(),
                //             relayer: relayer.into(),
                //         })
                //         .clear_decoder(),
                // ),
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
    use ibc_solidity::Ibc::ClientCreated;

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

    #[test]
    fn create_client_decode() {
        let bz = hex::decode("0x000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008636f6d6574626c73000000000000000000000000000000000000000000000000").unwrap();

        let result = ClientCreated::decode_log_data(
            &LogData::new(
                [fixed_bytes!(
                    "04e9540749029ffe9d24e5bd373d2e18bf4fab8f13c60b4a62b9ae8562920cc8"
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
}

use std::collections::VecDeque;

use alloy::{
    contract::{Error, RawCallBuilder},
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
use ibc_solidity::ibc::{
    ChannelCounterparty, ChannelOrder, ChannelState, ConnectionCounterparty,
    Ibc::{self, IbcErrors},
    MsgChannelOpenAck, MsgChannelOpenConfirm, MsgChannelOpenInit, MsgChannelOpenTry,
    MsgConnectionOpenAck, MsgConnectionOpenConfirm, MsgConnectionOpenInit, MsgConnectionOpenTry,
    MsgCreateClient, MsgPacketAcknowledgement, MsgPacketRecv, MsgPacketTimeout, MsgUpdateClient,
};
use itertools::Itertools;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::{ErrorObject, ErrorObjectOwned},
    Extensions,
};
use serde::{Deserialize, Serialize};
use tracing::{error, error_span, info, info_span, instrument, warn, Instrument};
use unionlabs::{
    ethereum::keccak256,
    hash::{H160, H256},
    ibc::core::channel::{channel::Channel, order::Order, packet::Packet, state::State},
    ErrorReporter,
};
use voyager_message::{
    core::ChainId,
    data::{log_msg, Data, IbcMessage, MsgCreateClientData, WithChainId},
    module::{PluginInfo, PluginServer},
    run_plugin_server, DefaultCmd, Plugin, PluginMessage, VoyagerMessage, FATAL_JSONRPC_ERROR_CODE,
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
    run_plugin_server::<Module>().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId<'static>,

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
    pub chain_id: ChainId<'static>,

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

fn plugin_name(chain_id: &ChainId<'_>) -> String {
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
    EmptyRevert(Vec<IbcMessage>),
    #[error("gas price is too high: max {max}, price {price}")]
    GasPriceTooHigh { max: u128, price: u128 },
    #[error("rpc error (this is jsut the IbcMessage conversion functions but i need to make those errors better)")]
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
                    (
                        vec![idx],
                        match msg {
                            Op::Data(Data::IdentifiedIbcMessage(WithChainId {
                                chain_id,
                                message,
                            })) => {
                                assert_eq!(chain_id, self.chain_id);

                                call(PluginMessage::new(
                                    self.plugin_name(),
                                    ModuleCall::SubmitMulticall(vec![message]),
                                ))
                            }
                            Op::Data(Data::IdentifiedIbcMessageBatch(WithChainId {
                                chain_id,
                                message,
                            })) => {
                                assert_eq!(chain_id, self.chain_id);

                                call(PluginMessage::new(
                                    self.plugin_name(),
                                    ModuleCall::SubmitMulticall(message),
                                ))
                            }
                            _ => panic!("unexpected message: {msg:?}"),
                        },
                    )
                })
                .collect(),
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
        ibc_messages: Vec<IbcMessage>,
    ) -> Result<(), TxSubmitError> {
        let signer = ProviderBuilder::new()
            .with_recommended_fillers()
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

        let msgs = process_msgs(&ibc, ibc_messages, wallet.address().into())?;

        let msg_names = msgs
            .iter()
            // .map(|x| (x.0.clone(), x.1.function.name.clone()))
            .map(|x| (x.0.clone(), "todo".to_owned()))
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

        match call.send().await {
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
                            )
                            .in_scope(|| log_msg(&self.chain_id.to_string(), &msg));
                        } else if let Ok(known_revert) =
                            IbcErrors::abi_decode(&result.returnData, true)
                        {
                            error_span!(
                                "evm message failed",
                                msg = %msg_name,
                                %idx,
                                revert = ?known_revert,
                                well_known = true,
                            )
                            .in_scope(|| log_msg(&self.chain_id.to_string(), &msg));
                        } else if result.returnData.is_empty() {
                            error_span!(
                                "evm message failed",
                                msg = %msg_name,
                                %idx,
                                revert = %result.returnData,
                                well_known = false,
                            )
                            .in_scope(|| log_msg(&self.chain_id.to_string(), &msg));

                            retry_msgs.push((true, msg));
                        } else {
                            error_span!(
                                "evm message failed",
                                msg = %msg_name,
                                %idx,
                                revert = %result.returnData,
                                well_known = false,
                            )
                            .in_scope(|| log_msg(&self.chain_id.to_string(), &msg));

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
    ibc_handler: &ibc_solidity::ibc::Ibc::IbcInstance<T, P>,
    msgs: Vec<IbcMessage>,
    relayer: H160,
) -> RpcResult<Vec<(IbcMessage, RawCallBuilder<T, &P>)>> {
    msgs.clone()
        .into_iter()
        .map(|msg| {
            Ok(match msg.clone() {
                IbcMessage::ConnectionOpenInit(data) => (
                    msg,
                    ibc_handler
                        .connectionOpenInit(MsgConnectionOpenInit {
                            clientId: parse_numeric_identifier(&*data.client_id)?,
                            counterparty: ConnectionCounterparty {
                                clientId: parse_numeric_identifier(&*data.counterparty.client_id)?,
                                connectionId: data
                                    .counterparty
                                    .connection_id
                                    .as_deref()
                                    .map(parse_numeric_identifier)
                                    .transpose()?
                                    .unwrap_or_default(),
                            },
                            relayer: relayer.into(),
                        })
                        .clear_decoder(),
                ),
                IbcMessage::ConnectionOpenTry(data) => (
                    msg,
                    ibc_handler
                        .connectionOpenTry(MsgConnectionOpenTry {
                            counterparty: ConnectionCounterparty {
                                clientId: parse_numeric_identifier(&*data.counterparty.client_id)?,
                                connectionId: data
                                    .counterparty
                                    .connection_id
                                    .as_deref()
                                    .map(parse_numeric_identifier)
                                    .transpose()?
                                    .unwrap_or_default(),
                            },
                            clientId: parse_numeric_identifier(&*data.client_id)?,
                            proofInit: data.proof_init.into(),
                            proofHeight: data.proof_height.height(),
                            relayer: relayer.into(),
                        })
                        .clear_decoder(),
                ),
                IbcMessage::ConnectionOpenAck(data) => (
                    msg,
                    ibc_handler
                        .connectionOpenAck(MsgConnectionOpenAck {
                            connectionId: parse_numeric_identifier(&*data.connection_id)?,
                            counterpartyConnectionId: parse_numeric_identifier(
                                &*data.counterparty_connection_id,
                            )?,
                            proofHeight: data.proof_height.height(),
                            proofTry: data.proof_try.into(),
                            relayer: relayer.into(),
                        })
                        .clear_decoder(),
                ),
                IbcMessage::ConnectionOpenConfirm(data) => (
                    msg,
                    ibc_handler
                        .connectionOpenConfirm(MsgConnectionOpenConfirm {
                            connectionId: parse_numeric_identifier(&*data.connection_id)?,
                            proofAck: data.proof_ack.into(),
                            proofHeight: data.proof_height.height(),
                            relayer: relayer.into(),
                        })
                        .clear_decoder(),
                ),
                IbcMessage::ChannelOpenInit(data) => (
                    msg,
                    ibc_handler
                        .channelOpenInit(MsgChannelOpenInit {
                            portId: parse_port_id(&*data.port_id)?,
                            channel: convert_channel(data.channel)?,
                            relayer: relayer.into(),
                        })
                        .clear_decoder(),
                ),
                IbcMessage::ChannelOpenTry(data) => (
                    msg,
                    ibc_handler
                        .channelOpenTry(MsgChannelOpenTry {
                            portId: parse_port_id(&*data.port_id)?,
                            channel: convert_channel(data.channel)?,
                            counterpartyVersion: string_to_bytes32(&data.counterparty_version)?,
                            proofInit: data.proof_init.into(),
                            proofHeight: data.proof_height.height(),
                            relayer: relayer.into(),
                        })
                        .clear_decoder(),
                ),
                IbcMessage::ChannelOpenAck(data) => (
                    msg,
                    ibc_handler
                        .channelOpenAck(MsgChannelOpenAck {
                            portId: parse_port_id(&*data.port_id)?,
                            channelId: parse_numeric_identifier(&*data.channel_id)?,
                            counterpartyVersion: keccak256(&data.counterparty_version).into(),
                            counterpartyChannelId: parse_numeric_identifier(
                                &*data.counterparty_channel_id,
                            )?,
                            proofTry: data.proof_try.into(),
                            proofHeight: data.proof_height.height(),
                            relayer: relayer.into(),
                        })
                        .clear_decoder(),
                ),
                IbcMessage::ChannelOpenConfirm(data) => (
                    msg,
                    ibc_handler
                        .channelOpenConfirm(MsgChannelOpenConfirm {
                            portId: parse_port_id(&*data.port_id)?,
                            channelId: parse_numeric_identifier(&*data.channel_id)?,
                            proofAck: data.proof_ack.into(),
                            proofHeight: data.proof_height.height(),
                            relayer: relayer.into(),
                        })
                        .clear_decoder(),
                ),
                IbcMessage::RecvPacket(data) => (
                    msg,
                    ibc_handler
                        .recvPacket(MsgPacketRecv {
                            packets: vec![convert_packet(data.packet)?],
                            proof: data.proof_commitment.into(),
                            proofHeight: data.proof_height.height(),
                            relayer: relayer.into(),
                            relayerMsgs: vec![],
                        })
                        .clear_decoder(),
                ),
                IbcMessage::AcknowledgePacket(data) => (
                    msg,
                    ibc_handler
                        .acknowledgePacket(MsgPacketAcknowledgement {
                            packets: vec![convert_packet(data.packet)?],
                            acknowledgements: vec![data.acknowledgement.into()],
                            proof: data.proof_acked.into(),
                            proofHeight: data.proof_height.height(),
                            relayer: relayer.into(),
                        })
                        .clear_decoder(),
                ),
                IbcMessage::TimeoutPacket(data) => (
                    msg,
                    ibc_handler
                        .timeoutPacket(MsgPacketTimeout {
                            packet: convert_packet(data.packet)?,
                            proof: data.proof_unreceived.into(),
                            proofHeight: data.proof_height.height(),
                            nextSequenceRecv: data.next_sequence_recv.get(),
                            relayer: relayer.into(),
                        })
                        .clear_decoder(),
                ),
                IbcMessage::CreateClient(MsgCreateClientData {
                    msg: data,
                    client_type,
                }) => (
                    msg,
                    ibc_handler
                        .createClient(MsgCreateClient {
                            clientType: keccak256(client_type.as_str()).into(),
                            clientStateBytes: data.client_state.into(),
                            consensusStateBytes: data.consensus_state.into(),
                            relayer: relayer.into(),
                        })
                        .clear_decoder(),
                ),
                IbcMessage::UpdateClient(data) => (
                    msg,
                    ibc_handler
                        .updateClient(MsgUpdateClient {
                            clientId: parse_numeric_identifier(&*data.client_id)?,
                            clientMessage: data.client_message.into(),
                            relayer: relayer.into(),
                        })
                        .clear_decoder(),
                ),
            })
        })
        .collect()
}

fn parse_numeric_identifier(s: impl AsRef<str>) -> RpcResult<u32> {
    let s = s.as_ref();

    s.split_once('-')
        .and_then(|(_, id)| id.parse().ok())
        .ok_or(ErrorObject::owned(
            FATAL_JSONRPC_ERROR_CODE,
            format!("identifier is not in required format (found `{s}`)"),
            None::<()>,
        ))
}

fn parse_port_id(s: impl AsRef<str>) -> RpcResult<alloy::primitives::Address> {
    let s = s.as_ref();

    s.parse().map_err(|e| {
        ErrorObject::owned(
            FATAL_JSONRPC_ERROR_CODE,
            format!(
                "port id is not an address (found `{s}`): {}",
                ErrorReporter(e)
            ),
            None::<()>,
        )
    })
}

fn convert_packet(packet: Packet) -> RpcResult<ibc_solidity::ibc::Packet> {
    Ok(ibc_solidity::ibc::Packet {
        sequence: packet.sequence.get(),
        sourceChannel: parse_numeric_identifier(&*packet.source_channel)?,
        destinationChannel: parse_numeric_identifier(&*packet.destination_channel)?,
        data: packet.data.into(),
        timeoutHeight: packet.timeout_height.height(),
        timeoutTimestamp: packet.timeout_timestamp,
    })
}

fn convert_channel(channel: Channel) -> RpcResult<ibc_solidity::ibc::Channel> {
    Ok(ibc_solidity::ibc::Channel {
        state: match channel.state {
            State::UninitializedUnspecified => {
                return Err(ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    "channel is in an invalid state",
                    None::<()>,
                ))
            }
            State::Init => ChannelState::Init,
            State::Tryopen => ChannelState::TryOpen,
            State::Open => ChannelState::Open,
            State::Closed => ChannelState::Closed,
        },
        ordering: match channel.ordering {
            Order::NoneUnspecified => {
                return Err(ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    "channel order is invalid",
                    None::<()>,
                ))
            }
            Order::Unordered => ChannelOrder::Unordered,
            Order::Ordered => ChannelOrder::Ordered,
        },
        connectionId: channel
            .connection_hops
            .into_iter()
            .exactly_one()
            .as_deref()
            .map_err(|e| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("channel connection hops are invalid: {}", ErrorReporter(e)),
                    None::<()>,
                )
            })
            .and_then(parse_numeric_identifier)?,
        counterparty: ChannelCounterparty {
            channelId: parse_numeric_identifier(&*channel.counterparty.channel_id)?,
        },
        version: string_to_bytes32(&*channel.version)?,
    })
}

fn string_to_bytes32(s: impl AsRef<str>) -> RpcResult<alloy::primitives::FixedBytes<32>> {
    let s = s.as_ref();

    if s.len() > 32 {
        return Err(ErrorObject::owned(
            FATAL_JSONRPC_ERROR_CODE,
            format!("string value `{s}` does not fit in a bytes32 value"),
            None::<()>,
        ));
    }

    Ok(alloy::primitives::FixedBytes::right_padding_from(
        s.as_bytes(),
    ))
}

pub mod multicall {
    alloy::sol! {
        #![sol(rpc)]

        struct Call3 {
            address target;
            bool allowFailure;
            bytes callData;
        }

        struct Result {
            bool success;
            bytes returnData;
        }

        event MulticallResult(Result[]);

        contract Multicall {
            function multicall(
                Call3[] calldata calls
            ) public payable returns (Result[] memory returnData);
        }
    }
}

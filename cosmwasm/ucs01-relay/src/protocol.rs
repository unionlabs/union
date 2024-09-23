use base58::{FromBase58, ToBase58};
use cosmwasm_std::{
    wasm_execute, Addr, AnyMsg, Attribute, BankMsg, Binary, Coin, CosmosMsg, DepsMut, Env,
    HexBinary, IbcEndpoint, IbcOrder, IbcPacket, IbcReceiveResponse, MessageInfo, Uint128, Uint512,
};
use prost::{Message, Name};
use protos::deferredack::v1beta1::{DeferredPacketInfo, MsgWriteDeferredAck};
use sha2::{Digest, Sha256};
use token_factory_api::TokenFactoryMsg;
use ucs01_relay_api::{
    middleware::{InFlightPfmPacket, Memo, MiddlewareError, PacketForward, PacketForwardError},
    protocol::{
        AddrOf, ProtocolSwitch, TransferProtocol, ATTR_ERROR, ATTR_PFM, ATTR_SUCCESS,
        ATTR_VALUE_PFM_ACK, IBC_SEND_ID,
    },
    types::{
        make_factory_denom, make_foreign_denom, DenomOrigin, EncodingError, GenericAck, Ics20Ack,
        Ics20Packet, JsonWasm, NormalizedTransferToken, TransferToken, Ucs01Ack,
        Ucs01TransferPacket,
    },
};
use unionlabs::{encoding, ibc::core::client::height::Height};

use crate::{
    contract::execute_transfer,
    error::ContractError,
    msg::{ExecuteMsg, TransferMsg},
    state::{
        ChannelInfo, DenomHash, PfmRefundPacketKey, CHANNEL_INFO, CHANNEL_STATE,
        FOREIGN_DENOM_TO_HASH, HASH_TO_FOREIGN_DENOM, IN_FLIGHT_PFM_PACKETS, MAX_SUBDENOM_LENGTH,
    },
};

pub trait TransferProtocolExt<'a>:
    TransferProtocol<Error: From<ContractError>, CustomMsg = TokenFactoryMsg>
{
    fn common(&self) -> &ProtocolCommon<'a>;

    fn common_mut(&mut self) -> &mut ProtocolCommon<'a>;

    fn do_get_in_flight_packet(&self, forward_packet: IbcPacket) -> Option<InFlightPfmPacket> {
        let refund_key = PfmRefundPacketKey {
            channel_id: forward_packet.src.channel_id,
            port_id: forward_packet.src.port_id,
            sequence: forward_packet.sequence,
        };

        IN_FLIGHT_PFM_PACKETS
            .load(self.common().deps.storage, refund_key.clone())
            .ok()
    }

    #[allow(clippy::type_complexity)]
    fn do_pfm_ack(
        &mut self,
        ibc_packet: IbcPacket,
        refund_info: InFlightPfmPacket,
        ack: Result<Vec<u8>, Vec<u8>>,
        sender: &AddrOf<Self::Packet>,
        tokens: Vec<TransferToken>,
    ) -> Result<(Vec<CosmosMsg<Self::CustomMsg>>, Vec<Attribute>), Self::Error> {
        let ack =
            self.convert_ack_to_foreign_protocol(&refund_info.origin_protocol_version, ack)?;

        let (mut ack_msgs, mut ack_attrs, ack_bytes) = match ack {
            Ok(value) => {
                let value_string = Binary::from(value.clone()).to_string();
                (
                    self.send_tokens_success(sender, &Default::default(), tokens)?,
                    Vec::from_iter(
                        (!value_string.is_empty())
                            .then_some(Attribute::new(ATTR_SUCCESS, value_string)),
                    ),
                    value.to_vec(),
                )
            }
            Err(error) => (
                self.send_tokens_failure(sender, &Default::default(), tokens)?,
                Vec::from_iter((!error.is_empty()).then_some(Attribute::new(
                    ATTR_ERROR,
                    Binary::from(error.clone()).to_string(),
                ))),
                error.to_vec(),
            ),
        };

        let packet_timeout_timestamp: u64 = refund_info
            .origin_packet
            .timeout
            .timestamp()
            .unwrap_or_default()
            .nanos();
        let packet_timeout_height = match refund_info.origin_packet.timeout.block() {
            Some(timeout_block) => Height {
                revision_number: timeout_block.revision,
                revision_height: timeout_block.height,
            },
            None => Height {
                revision_number: 0,
                revision_height: 0,
            },
        };

        let deferred_packet_into = DeferredPacketInfo {
            refund_channel_id: refund_info.origin_packet.dest.channel_id,
            refund_port_id: refund_info.origin_packet.dest.port_id,
            packet_src_channel_id: refund_info.origin_packet.src.channel_id,
            packet_src_port_id: refund_info.origin_packet.src.port_id,
            packet_timeout_timestamp,
            packet_timeout_height: packet_timeout_height.to_string(),
            packet_data: refund_info.origin_packet.data.to_vec(),
            sequence: refund_info.origin_packet.sequence,
        };

        let deferred_ack_msg = CosmosMsg::<Self::CustomMsg>::Any(AnyMsg {
            type_url: MsgWriteDeferredAck::type_url(),
            value: MsgWriteDeferredAck {
                sender: self.self_addr().to_string(),
                deferred_packet_info: Some(deferred_packet_into),
                ack: ack_bytes,
            }
            .encode_to_vec()
            .into(),
        });

        ack_msgs.push(deferred_ack_msg);
        ack_attrs.push(Attribute::new(ATTR_PFM, ATTR_VALUE_PFM_ACK.to_string()));

        IN_FLIGHT_PFM_PACKETS.remove(
            self.common_mut().deps.storage,
            PfmRefundPacketKey {
                channel_id: ibc_packet.src.channel_id,
                port_id: ibc_packet.src.port_id,
                sequence: ibc_packet.sequence,
            },
        );

        Ok((ack_msgs, ack_attrs))
    }

    fn do_forward_transfer_packet(
        &mut self,
        tokens: Vec<Coin>,
        original_packet: IbcPacket,
        forward: PacketForward,
        sender: Addr,
    ) -> Result<IbcReceiveResponse<Self::CustomMsg>, Self::Error> {
        // Prepare forward message
        let msg_info = MessageInfo {
            sender,
            funds: tokens,
        };

        let timeout = forward.get_effective_timeout()?;

        // TODO: persist full memo
        let memo = match forward.next {
            Some(next) => serde_json_wasm::to_string(&Memo::Forward { forward: *next })
                .expect("can convert pfm memo to json string"),
            None => "".to_owned(),
        };

        let transfer_msg = TransferMsg {
            channel: forward.channel.clone().value(),
            receiver: forward.receiver.value(),
            timeout: Some(timeout),
            memo,
            fees: forward.fees,
        };

        // Send forward message
        let common = self.common_mut();
        let mut transfer = execute_transfer(
            common.deps.branch(),
            common.env.clone(),
            msg_info,
            transfer_msg,
        )?;

        let in_flight_packet = InFlightPfmPacket {
            origin_sender_addr: self.common().info.sender.clone(),
            origin_packet: original_packet,
            forward_timeout: timeout,
            forward_src_channel_id: forward.channel.value(),
            forward_src_port_id: forward.port.value(),
            origin_protocol_version: Self::VERSION.to_string(),
        };

        if let Some(reply_sub) = transfer
            .messages
            .iter_mut()
            .find(|sub| sub.id == IBC_SEND_ID)
        {
            *reply_sub = reply_sub
                .clone()
                .with_payload(serde_json_wasm::to_vec(&in_flight_packet).expect("can serialize"));
        } else {
            return Err(
                ContractError::MiddlewareError(MiddlewareError::PacketForward(
                    PacketForwardError::NoReplyMessageInStack,
                ))
                .into(),
            );
        }

        Ok(IbcReceiveResponse::without_ack()
            .add_submessages(transfer.messages)
            .add_events(transfer.events))
    }
}

impl<'a> TransferProtocolExt<'a> for Ucs01Protocol<'a> {
    fn common(&self) -> &ProtocolCommon<'a> {
        &self.common
    }

    fn common_mut(&mut self) -> &mut ProtocolCommon<'a> {
        &mut self.common
    }
}

impl<'a> TransferProtocolExt<'a> for Ics20Protocol<'a> {
    fn common(&self) -> &ProtocolCommon<'a> {
        &self.common
    }

    fn common_mut(&mut self) -> &mut ProtocolCommon<'a> {
        &mut self.common
    }
}

pub fn hash_denom(denom: &str) -> DenomHash {
    let mut hasher = Sha256::new();
    hasher.update(denom);
    DenomHash(<[u8; 32]>::from(hasher.finalize()).into())
}

pub fn encode_denom_hash(denom_hash: DenomHash) -> String {
    let result = denom_hash.0.get().to_base58().to_string();
    // https://en.wikipedia.org/wiki/Binary-to-text_encoding
    // Luckily, base58 encoding has ~0.73 efficiency:
    // (1 / 0.73) * 32 = 43.8356164384
    assert!(result.len() <= MAX_SUBDENOM_LENGTH);
    result
}

pub fn protocol_ordering(version: &str) -> Option<IbcOrder> {
    match version {
        Ics20Protocol::VERSION => Some(Ics20Protocol::ORDERING),
        Ucs01Protocol::VERSION => Some(Ucs01Protocol::ORDERING),
        _ => None,
    }
}

fn batch_submessages(
    self_addr: &cosmwasm_std::Addr,
    msgs: Vec<CosmosMsg<TokenFactoryMsg>>,
) -> Result<Vec<CosmosMsg<TokenFactoryMsg>>, ContractError> {
    Ok(vec![wasm_execute(
        self_addr,
        &ExecuteMsg::BatchExecute { msgs },
        vec![],
    )?
    .into()])
}

fn update_outstanding<F>(
    deps: DepsMut,
    channel_id: &str,
    denom: &str,
    f: F,
) -> Result<(), ContractError>
where
    F: FnOnce(Option<Uint512>) -> Result<Uint512, ContractError>,
{
    CHANNEL_STATE.update(
        deps.storage,
        (channel_id, denom),
        |state| -> Result<_, ContractError> {
            let new_outstanding = f(state.as_ref().map(|x| x.outstanding))?;
            let mut state = state.unwrap_or_default();
            state.outstanding = new_outstanding;
            Ok(state)
        },
    )?;
    Ok(())
}

fn increase_outstanding(
    deps: DepsMut,
    channel_id: &str,
    denom: &str,
    amount: Uint128,
) -> Result<(), ContractError> {
    update_outstanding(deps, channel_id, denom, |outstanding| {
        let new_outstanding = outstanding.unwrap_or_default().checked_add(amount.into())?;
        Ok(new_outstanding)
    })
}

fn decrease_outstanding(
    deps: DepsMut,
    channel_id: &str,
    denom: &str,
    amount: Uint128,
) -> Result<(), ContractError> {
    update_outstanding(deps, channel_id, denom, |outstanding| {
        let new_outstanding = outstanding
            .ok_or(ContractError::InsufficientFunds)?
            .checked_sub(amount.into())?;
        Ok(new_outstanding)
    })
}

fn normalize_for_ibc_transfer(
    mut hash_to_denom: impl FnMut(DenomHash) -> Result<Option<String>, ContractError>,
    contract_address: &str,
    endpoint: &IbcEndpoint,
    token: TransferToken,
) -> Result<TransferToken, ContractError> {
    let normalized_denom = match token
        .denom
        .strip_prefix("factory/")
        .and_then(|denom| denom.strip_prefix(contract_address))
        .and_then(|denom| denom.strip_prefix("/"))
    {
        Some(denom_hash) => {
            if let Some(normalized_denom) = hash_to_denom(DenomHash(unionlabs::hash::H256::new(
                denom_hash
                    .from_base58()
                    .expect("impossible")
                    .try_into()
                    .expect("impossible"),
            )))? {
                // This is the POV of the counterparty chain, where we transfer from A to B. It's a similar check than in receive_phase1.
                // If the denom is prefixed by the source chain path (A), it means it was local (originating from B and minted on A).
                // If the denom isn't prefixed by the source chain path (A), it means it was remote (originating from A chain).
                match DenomOrigin::from((normalized_denom.as_ref(), endpoint)) {
                    // If the token is from B, send back the preimage
                    DenomOrigin::Local { .. } => normalized_denom,
                    // If the token is from A, send the image
                    DenomOrigin::Remote { .. } => token.denom,
                }
            } else {
                token.denom
            }
        }
        None => token.denom,
    };
    Ok(TransferToken {
        denom: normalized_denom,
        amount: token.amount,
        fee: token.fee,
    })
}
trait OnReceive {
    fn foreign_toggle(
        &mut self,
        contract_address: &Addr,
        local_endpoint: &IbcEndpoint,
        denom: &str,
    ) -> Result<(bool, DenomHash, CosmosMsg<TokenFactoryMsg>), ContractError>;

    fn local_unescrow(
        &mut self,
        channel_id: &str,
        denom: &str,
        amount: Uint128,
    ) -> Result<(), ContractError>;

    #[allow(clippy::too_many_arguments)]
    fn receive_phase1_transfer(
        &mut self,
        contract_address: &Addr,
        endpoint: &IbcEndpoint,
        counterparty_endpoint: &IbcEndpoint,
        receiver: &str,
        relayer: &str,
        tokens: Vec<TransferToken>,
        cut_fees: bool,
    ) -> Result<
        (
            Vec<NormalizedTransferToken>,
            Vec<CosmosMsg<TokenFactoryMsg>>,
        ),
        ContractError,
    > {
        tokens
            .into_iter()
            .map(
                |ref token @ TransferToken {
                     ref denom,
                     amount,
                     fee,
                 }| {
                    let (actual_amount, fee_amount) = if cut_fees {
                        token.amounts()?
                    } else {
                        (token.amount, Uint128::zero())
                    };
                    let origin_denom = denom;
                    match DenomOrigin::from((denom.as_str(), counterparty_endpoint)) {
                        // The denom was prefixed by the counterparty endpoint,
                        // meaning it's a voucher that has been burnt on the
                        // remote chain. We must unescrow and transfer the
                        // native token.
                        DenomOrigin::Local { denom } => {
                            let total_amount = token.amount;
                            self.local_unescrow(&endpoint.channel_id, denom, total_amount)?;
                            let mut bank_msgs = Vec::with_capacity(2);
                            if !actual_amount.is_zero() {
                                bank_msgs.push(
                                    BankMsg::Send {
                                        to_address: receiver.to_string(),
                                        amount: vec![Coin {
                                            denom: denom.to_string(),
                                            amount: actual_amount,
                                        }],
                                    }
                                    .into(),
                                );
                            }
                            if !fee_amount.is_zero() {
                                bank_msgs.push(
                                    BankMsg::Send {
                                        to_address: relayer.to_string(),
                                        amount: vec![Coin {
                                            denom: denom.to_string(),
                                            amount: fee_amount,
                                        }],
                                    }
                                    .into(),
                                );
                            }
                            Ok((
                                NormalizedTransferToken {
                                    origin_denom: origin_denom.into(),
                                    token: TransferToken {
                                        denom: denom.to_string(),
                                        amount,
                                        fee,
                                    },
                                },
                                bank_msgs,
                            ))
                        }
                        // The denom wasn't prefixed with the remote endpoint,
                        // meaning it's a remote token. We must create a voucher
                        // if we didn't already and mint the according amount.
                        DenomOrigin::Remote { denom } => {
                            // We prefix the denom as per the spec with `source_port/source_channel/denom`.
                            let foreign_denom = make_foreign_denom(endpoint, denom);
                            // Check whether we need to register the new denom.
                            let (exists, hashed_foreign_denom, register_msg) =
                                self.foreign_toggle(contract_address, endpoint, &foreign_denom)?;
                            let normalized_foreign_denom = encode_denom_hash(hashed_foreign_denom);
                            let factory_denom = format!(
                                "factory/{}/{}",
                                contract_address, normalized_foreign_denom
                            );
                            let mut msgs = Vec::with_capacity(4);
                            // Create and register the asset if not already present.
                            if !exists {
                                msgs.push(register_msg);
                                msgs.push(
                                    TokenFactoryMsg::CreateDenom {
                                        subdenom: normalized_foreign_denom.clone(),
                                        metadata: None,
                                    }
                                    .into(),
                                );
                            }
                            // Only ever yield mint messages if the amount are non zero.
                            if !actual_amount.is_zero() {
                                msgs.push(
                                    TokenFactoryMsg::MintTokens {
                                        denom: factory_denom.clone(),
                                        amount: actual_amount,
                                        mint_to_address: receiver.to_string(),
                                    }
                                    .into(),
                                );
                            }
                            if !fee_amount.is_zero() {
                                msgs.push(
                                    TokenFactoryMsg::MintTokens {
                                        denom: factory_denom.clone(),
                                        amount: fee_amount,
                                        mint_to_address: relayer.to_string(),
                                    }
                                    .into(),
                                );
                            }
                            Ok((
                                NormalizedTransferToken {
                                    origin_denom: origin_denom.into(),
                                    token: TransferToken {
                                        denom: factory_denom,
                                        amount,
                                        fee,
                                    },
                                },
                                msgs,
                            ))
                        }
                    }
                },
            )
            .collect::<Result<(_, Vec<_>), _>>()
            .map(|(x, y)| (x, y.into_iter().flatten().collect()))
    }
}

pub struct StatefulOnReceive<'a> {
    deps: DepsMut<'a>,
}
impl<'a> OnReceive for StatefulOnReceive<'a> {
    fn foreign_toggle(
        &mut self,
        contract_address: &Addr,
        local_endpoint: &IbcEndpoint,
        denom: &str,
    ) -> Result<(bool, DenomHash, CosmosMsg<TokenFactoryMsg>), ContractError> {
        let exists = FOREIGN_DENOM_TO_HASH.has(
            self.deps.storage,
            (local_endpoint.clone().into(), denom.to_string()),
        );
        let hash = hash_denom(denom);
        Ok((
            exists,
            hash,
            wasm_execute(
                contract_address,
                &ExecuteMsg::RegisterDenom {
                    local_endpoint: local_endpoint.clone(),
                    denom: denom.to_string(),
                    hash: hash.0.get().into(),
                },
                Default::default(),
            )?
            .into(),
        ))
    }

    fn local_unescrow(
        &mut self,
        channel_id: &str,
        denom: &str,
        amount: Uint128,
    ) -> Result<(), ContractError> {
        decrease_outstanding(self.deps.branch(), channel_id, denom, amount)?;
        Ok(())
    }
}

trait ForTokens {
    fn on_local(
        &mut self,
        channel_id: &str,
        denom: &str,
        amount: Uint128,
        fee_amount: Uint128,
    ) -> Result<Vec<CosmosMsg<TokenFactoryMsg>>, ContractError>;

    fn on_remote(
        &mut self,
        channel_id: &str,
        denom: &str,
        amount: Uint128,
        fee_amount: Uint128,
    ) -> Result<Vec<CosmosMsg<TokenFactoryMsg>>, ContractError>;

    fn execute(
        &mut self,
        contract_address: &Addr,
        endpoint: &IbcEndpoint,
        tokens: Vec<TransferToken>,
    ) -> Result<Vec<CosmosMsg<TokenFactoryMsg>>, ContractError> {
        let mut messages = Vec::with_capacity(tokens.len());
        for token in tokens {
            // This is the origin from the counterparty POV
            let (actual_amount, fee_amount) = token.amounts()?;
            match DenomOrigin::from((token.denom.as_str(), endpoint)) {
                DenomOrigin::Local { denom } => {
                    // The denom has been previously normalized (factory/{}/ prefix removed), we must reconstruct to burn.
                    let foreign_denom =
                        encode_denom_hash(hash_denom(&make_foreign_denom(endpoint, denom)));
                    let factory_denom = make_factory_denom(contract_address, &foreign_denom);
                    messages.append(&mut self.on_remote(
                        &endpoint.channel_id,
                        &factory_denom,
                        actual_amount,
                        fee_amount,
                    )?);
                }
                DenomOrigin::Remote { denom } => {
                    messages.append(&mut self.on_local(
                        &endpoint.channel_id,
                        denom,
                        actual_amount,
                        fee_amount,
                    )?);
                }
            }
        }
        Ok(messages)
    }
}

struct StatefulSendTokens<'a> {
    deps: DepsMut<'a>,
    contract_address: String,
}

impl<'a> ForTokens for StatefulSendTokens<'a> {
    fn on_local(
        &mut self,
        channel_id: &str,
        denom: &str,
        amount: Uint128,
        fee_amount: Uint128,
    ) -> Result<Vec<CosmosMsg<TokenFactoryMsg>>, ContractError> {
        let total_amount = amount
            .checked_add(fee_amount)
            .expect("impossible; fee must be split from the base amount");
        increase_outstanding(self.deps.branch(), channel_id, denom, total_amount)?;
        Ok(Default::default())
    }

    fn on_remote(
        &mut self,
        _channel_id: &str,
        denom: &str,
        amount: Uint128,
        fee_amount: Uint128,
    ) -> Result<Vec<CosmosMsg<TokenFactoryMsg>>, ContractError> {
        Ok(vec![TokenFactoryMsg::BurnTokens {
            denom: denom.into(),
            amount: amount
                .checked_add(fee_amount)
                .expect("impossible; fee must be split from the base amount"),
            burn_from_address: self.contract_address.clone(),
        }
        .into()])
    }
}

struct StatefulRefundTokens<'a> {
    deps: DepsMut<'a>,
    receiver: String,
}

impl<'a> ForTokens for StatefulRefundTokens<'a> {
    fn on_local(
        &mut self,
        channel_id: &str,
        denom: &str,
        amount: Uint128,
        fee_amount: Uint128,
    ) -> Result<Vec<CosmosMsg<TokenFactoryMsg>>, ContractError> {
        let total_amount = amount
            .checked_add(fee_amount)
            .expect("impossible; fee must be split from the base amount");
        decrease_outstanding(self.deps.branch(), channel_id, denom, total_amount)?;
        Ok(vec![BankMsg::Send {
            to_address: self.receiver.clone(),
            amount: vec![Coin {
                denom: denom.into(),
                amount: total_amount,
            }],
        }
        .into()])
    }

    fn on_remote(
        &mut self,
        _channel_id: &str,
        denom: &str,
        amount: Uint128,
        fee_amount: Uint128,
    ) -> Result<Vec<CosmosMsg<TokenFactoryMsg>>, ContractError> {
        let total_amount = amount
            .checked_add(fee_amount)
            .expect("impossible; fee must be split from the base amount");
        Ok(vec![TokenFactoryMsg::MintTokens {
            denom: denom.into(),
            amount: total_amount,
            mint_to_address: self.receiver.clone(),
        }
        .into()])
    }
}

pub struct ProtocolCommon<'a> {
    pub deps: DepsMut<'a>,
    pub env: Env,
    pub info: MessageInfo,
    pub channel: ChannelInfo,
}

pub struct Ics20Protocol<'a> {
    pub common: ProtocolCommon<'a>,
}

impl<'a> TransferProtocol for Ics20Protocol<'a> {
    const VERSION: &'static str = "ics20-1";
    const ORDERING: IbcOrder = IbcOrder::Unordered;
    const RECEIVE_REPLY_ID: u64 = 0;

    type Packet = Ics20Packet;
    type Ack = Ics20Ack;
    type CustomMsg = TokenFactoryMsg;
    type Error = ContractError;
    type Encoding = JsonWasm;

    fn channel_endpoint(&self) -> &cosmwasm_std::IbcEndpoint {
        &self.common.channel.endpoint
    }

    fn caller(&self) -> &cosmwasm_std::Addr {
        &self.common.info.sender
    }

    fn self_addr(&self) -> &cosmwasm_std::Addr {
        &self.common.env.contract.address
    }

    fn self_addr_canonical(&self) -> Result<AddrOf<Self::Packet>, Self::Error> {
        Ok(self.self_addr().to_string())
    }

    fn ack_success() -> Self::Ack {
        Ics20Ack::Result(vec![1].into())
    }

    fn ack_failure(error: String) -> Self::Ack {
        Ics20Ack::Error(error)
    }

    fn send_tokens(
        &mut self,
        _sender: &String,
        _receiver: &String,
        tokens: Vec<TransferToken>,
    ) -> Result<Vec<CosmosMsg<Self::CustomMsg>>, Self::Error> {
        StatefulSendTokens {
            deps: self.common.deps.branch(),
            contract_address: self.common.env.contract.address.to_string(),
        }
        .execute(
            &self.common.env.contract.address,
            &self.common.channel.endpoint,
            tokens,
        )
    }

    fn send_tokens_success(
        &mut self,
        _sender: &String,
        _receiver: &String,
        _tokens: Vec<TransferToken>,
    ) -> Result<Vec<CosmosMsg<Self::CustomMsg>>, Self::Error> {
        Ok(Default::default())
    }

    fn send_tokens_failure(
        &mut self,
        sender: &String,
        _receiver: &String,
        tokens: Vec<TransferToken>,
    ) -> Result<Vec<CosmosMsg<Self::CustomMsg>>, Self::Error> {
        StatefulRefundTokens {
            deps: self.common.deps.branch(),
            receiver: sender.into(),
        }
        .execute(
            &self.common.env.contract.address,
            &self.common.channel.endpoint,
            tokens,
        )
    }

    #[allow(clippy::type_complexity)]
    fn receive_transfer(
        &mut self,
        receiver: &AddrOf<Self::Packet>,
        tokens: Vec<TransferToken>,
        cut_fees: bool,
    ) -> Result<
        (
            Vec<NormalizedTransferToken>,
            Vec<CosmosMsg<Self::CustomMsg>>,
        ),
        Self::Error,
    > {
        let (tokens, msgs) = StatefulOnReceive {
            deps: self.common.deps.branch(),
        }
        .receive_phase1_transfer(
            &self.common.env.contract.address,
            &self.common.channel.endpoint,
            &self.common.channel.counterparty_endpoint,
            receiver.as_str(),
            self.common.info.sender.as_str(),
            tokens,
            cut_fees,
        )?;
        Ok((tokens, batch_submessages(self.self_addr(), msgs)?))
    }

    fn normalize_for_ibc_transfer(
        &mut self,
        token: TransferToken,
    ) -> Result<TransferToken, Self::Error> {
        normalize_for_ibc_transfer(
            |hash| {
                HASH_TO_FOREIGN_DENOM
                    .may_load(self.common.deps.storage, hash)
                    .map_err(Into::into)
            },
            self.common.env.contract.address.as_str(),
            &self.common.channel.endpoint,
            token,
        )
    }

    fn common_to_protocol_packet(
        &self,
        packet: ucs01_relay_api::types::TransferPacketCommon<
            ucs01_relay_api::protocol::PacketExtensionOf<Self>,
        >,
    ) -> Result<Self::Packet, ucs01_relay_api::types::EncodingError> {
        Ics20Packet::try_from(packet)
    }

    fn forward_transfer_packet(
        &mut self,
        tokens: Vec<Coin>,
        original_packet: IbcPacket,
        forward: PacketForward,
        sender: Addr,
    ) -> Result<IbcReceiveResponse<Self::CustomMsg>, ContractError> {
        self.do_forward_transfer_packet(tokens, original_packet, forward, sender)
    }

    fn pfm_ack(
        &mut self,
        ack: GenericAck,
        ibc_packet: IbcPacket,
        refund_info: InFlightPfmPacket,
        sender: &AddrOf<Self::Packet>,
        tokens: Vec<TransferToken>,
    ) -> Result<(Vec<CosmosMsg<Self::CustomMsg>>, Vec<Attribute>), Self::Error> {
        self.do_pfm_ack(ibc_packet, refund_info, ack, sender, tokens)
    }

    fn convert_ack_to_foreign_protocol(
        &self,
        foreign_protocol: &str,
        ack: GenericAck,
    ) -> Result<GenericAck, ContractError> {
        match foreign_protocol {
            Ucs01Protocol::VERSION => Ok(match ack {
                Ok(_) => Ucs01Protocol::ack_success(),
                // REVIEW: Why do we discard the error here?
                Err(_) => Ucs01Protocol::ack_failure("".into()),
            }
            .into()),
            Ics20Protocol::VERSION => Ok(ack),
            v => Err(ContractError::UnknownProtocol {
                channel_id: String::new(),
                protocol_version: v.to_string(),
            }),
        }
    }

    fn get_in_flight_packet(&self, forward_packet: IbcPacket) -> Option<InFlightPfmPacket> {
        self.do_get_in_flight_packet(forward_packet)
    }

    fn load_channel_protocol_version(&self, channel_id: &str) -> Result<String, Self::Error> {
        Ok(CHANNEL_INFO
            .load(self.common.deps.storage, channel_id)?
            .protocol_version)
    }

    fn protocol_switch_result(
        &self,
        counterparty_protocol_version: &str,
    ) -> ucs01_relay_api::protocol::ProtocolSwitch {
        match counterparty_protocol_version {
            Ucs01Protocol::VERSION => ProtocolSwitch::Upgrade,
            Ics20Protocol::VERSION => ProtocolSwitch::Stable,
            x => panic!("impossible, unknown protocol: {}", x),
        }
    }
}

pub struct Ucs01Protocol<'a> {
    pub common: ProtocolCommon<'a>,
}

impl<'a> TransferProtocol for Ucs01Protocol<'a> {
    const VERSION: &'static str = "ucs01-relay-1";
    const ORDERING: IbcOrder = IbcOrder::Unordered;
    const RECEIVE_REPLY_ID: u64 = 1;

    type Packet = Ucs01TransferPacket;
    type Ack = Ucs01Ack;
    type CustomMsg = TokenFactoryMsg;
    type Error = ContractError;
    type Encoding = encoding::EthAbi;

    fn channel_endpoint(&self) -> &cosmwasm_std::IbcEndpoint {
        &self.common.channel.endpoint
    }

    fn caller(&self) -> &cosmwasm_std::Addr {
        &self.common.info.sender
    }

    fn self_addr(&self) -> &cosmwasm_std::Addr {
        &self.common.env.contract.address
    }

    fn self_addr_canonical(&self) -> Result<AddrOf<Self::Packet>, Self::Error> {
        Ok(self
            .common
            .deps
            .api
            .addr_canonicalize(self.self_addr().as_str())?
            .into())
    }

    fn ack_success() -> Self::Ack {
        Ucs01Ack::Success
    }

    fn ack_failure(_: String) -> Self::Ack {
        Ucs01Ack::Failure
    }

    fn send_tokens(
        &mut self,
        _sender: &HexBinary,
        _receiver: &HexBinary,
        tokens: Vec<TransferToken>,
    ) -> Result<Vec<CosmosMsg<Self::CustomMsg>>, Self::Error> {
        StatefulSendTokens {
            deps: self.common.deps.branch(),
            contract_address: self.common.env.contract.address.to_string(),
        }
        .execute(
            &self.common.env.contract.address,
            &self.common.channel.endpoint,
            tokens,
        )
    }

    fn send_tokens_success(
        &mut self,
        _sender: &HexBinary,
        _receiver: &HexBinary,
        _tokens: Vec<TransferToken>,
    ) -> Result<Vec<CosmosMsg<Self::CustomMsg>>, Self::Error> {
        Ok(Default::default())
    }

    // TODO: Remove receiver? here and on send_tokens_success?
    fn send_tokens_failure(
        &mut self,
        sender: &HexBinary,
        _receiver: &HexBinary,
        tokens: Vec<TransferToken>,
    ) -> Result<Vec<CosmosMsg<Self::CustomMsg>>, Self::Error> {
        let addr = self.common.deps.api.addr_humanize(&sender.clone().into())?;
        StatefulRefundTokens {
            deps: self.common.deps.branch(),
            receiver: addr.to_string(),
        }
        .execute(
            &self.common.env.contract.address,
            &self.common.channel.endpoint,
            tokens,
        )
    }

    #[allow(clippy::type_complexity)]
    fn receive_transfer(
        &mut self,
        receiver: &AddrOf<Self::Packet>,
        tokens: Vec<TransferToken>,
        cut_fees: bool,
    ) -> Result<
        (
            Vec<NormalizedTransferToken>,
            Vec<CosmosMsg<Self::CustomMsg>>,
        ),
        Self::Error,
    > {
        let receiver = self
            .common
            .deps
            .api
            .addr_humanize(&receiver.clone().into())?;
        // TODO(aeryz): call `addr_validate` here
        let (tokens, msgs) = StatefulOnReceive {
            deps: self.common.deps.branch(),
        }
        .receive_phase1_transfer(
            &self.common.env.contract.address,
            &self.common.channel.endpoint,
            &self.common.channel.counterparty_endpoint,
            receiver.as_str(),
            self.common.info.sender.as_str(),
            tokens,
            cut_fees,
        )?;

        Ok((tokens, batch_submessages(self.self_addr(), msgs)?))
    }

    fn normalize_for_ibc_transfer(
        &mut self,
        token: TransferToken,
    ) -> Result<TransferToken, Self::Error> {
        normalize_for_ibc_transfer(
            |hash| {
                HASH_TO_FOREIGN_DENOM
                    .may_load(self.common.deps.storage, hash)
                    .map_err(Into::into)
            },
            self.common.env.contract.address.as_str(),
            &self.common.channel.endpoint,
            token,
        )
    }

    fn common_to_protocol_packet(
        &self,
        packet: ucs01_relay_api::types::TransferPacketCommon<
            ucs01_relay_api::protocol::PacketExtensionOf<Self>,
        >,
    ) -> Result<Self::Packet, EncodingError> {
        Ok(Ucs01TransferPacket::new(
            self.common
                .deps
                .api
                .addr_canonicalize(&packet.sender)
                .map_err(|err| EncodingError::InvalidSender {
                    value: packet.sender,
                    err,
                })?
                .into(),
            HexBinary::from_hex(&packet.receiver).map_err(|err| {
                EncodingError::InvalidReceiver {
                    value: packet.receiver,
                    err,
                }
            })?,
            packet.tokens,
            packet.extension,
        ))
    }

    fn forward_transfer_packet(
        &mut self,
        tokens: Vec<Coin>,
        original_packet: IbcPacket,
        forward: PacketForward,
        sender: Addr,
    ) -> Result<IbcReceiveResponse<Self::CustomMsg>, ContractError> {
        self.do_forward_transfer_packet(tokens, original_packet, forward, sender)
    }

    #[allow(clippy::type_complexity)]
    fn pfm_ack(
        &mut self,
        ack: GenericAck,
        ibc_packet: IbcPacket,
        refund_info: InFlightPfmPacket,
        sender: &AddrOf<Self::Packet>,
        tokens: Vec<TransferToken>,
    ) -> Result<(Vec<CosmosMsg<Self::CustomMsg>>, Vec<Attribute>), Self::Error> {
        self.do_pfm_ack(ibc_packet, refund_info, ack, sender, tokens)
    }

    fn convert_ack_to_foreign_protocol(
        &self,
        foreign_protocol: &str,
        ack: GenericAck,
    ) -> Result<GenericAck, Self::Error> {
        match foreign_protocol {
            Ucs01Protocol::VERSION => Ok(ack),
            Ics20Protocol::VERSION => Ok(match ack {
                Ok(_) => Ics20Protocol::ack_success(),
                Err(_) => Ics20Protocol::ack_failure("ucs01 ack failure".to_string()),
            }
            .into()),
            v => Err(ContractError::UnknownProtocol {
                channel_id: String::new(),
                protocol_version: v.to_string(),
            }),
        }
    }

    fn get_in_flight_packet(&self, forward_packet: IbcPacket) -> Option<InFlightPfmPacket> {
        self.do_get_in_flight_packet(forward_packet)
    }

    fn load_channel_protocol_version(&self, channel_id: &str) -> Result<String, Self::Error> {
        Ok(CHANNEL_INFO
            .load(self.common.deps.storage, channel_id)?
            .protocol_version)
    }

    fn protocol_switch_result(
        &self,
        counterparty_protocol_version: &str,
    ) -> ucs01_relay_api::protocol::ProtocolSwitch {
        match counterparty_protocol_version {
            Ucs01Protocol::VERSION => ProtocolSwitch::Stable,
            Ics20Protocol::VERSION => ProtocolSwitch::Downgrade,
            x => panic!("impossible, unknown protocol: {}", x),
        }
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        testing::{message_info, mock_dependencies, mock_env},
        wasm_execute, Addr, BankMsg, Coin, CosmosMsg, IbcEndpoint, Uint128,
    };
    use token_factory_api::TokenFactoryMsg;
    use ucs01_relay_api::{
        protocol::TransferProtocol,
        types::{FeePerU128, TransferToken},
    };

    use super::{hash_denom, ForTokens, OnReceive, StatefulOnReceive};
    use crate::{
        error::ContractError,
        msg::ExecuteMsg,
        protocol::{encode_denom_hash, normalize_for_ibc_transfer, Ics20Protocol},
        state::{ChannelInfo, DenomHash},
    };

    #[test]
    fn test_ack() {
        let mut deps = mock_dependencies();
        println!(
            "ACK: {:?}",
            Ics20Protocol {
                common: super::ProtocolCommon {
                    deps: deps.as_mut(),
                    env: mock_env(),
                    info: message_info(&Addr::unchecked(""), &[]),
                    channel: ChannelInfo {
                        endpoint: IbcEndpoint {
                            port_id: "".to_string(),
                            channel_id: String::new(),
                        },
                        counterparty_endpoint: IbcEndpoint {
                            port_id: "".to_string(),
                            channel_id: String::new(),
                        },
                        connection_id: "".into(),
                        protocol_version: "".into(),
                    },
                },
            }
            .convert_ack_to_foreign_protocol("ucs01-relay-1", Ok(vec![1]))
            .unwrap()
            .unwrap()
        );
    }

    struct TestOnReceive {
        toggle: bool,
    }
    impl OnReceive for TestOnReceive {
        fn foreign_toggle(
            &mut self,
            contract_address: &Addr,
            local_endpoint: &IbcEndpoint,
            denom: &str,
        ) -> Result<(bool, DenomHash, CosmosMsg<TokenFactoryMsg>), ContractError> {
            let mut deps = mock_dependencies();
            let (_, hash, msg) = StatefulOnReceive {
                deps: deps.as_mut(),
            }
            .foreign_toggle(contract_address, local_endpoint, denom)?;
            Ok((self.toggle, hash, msg))
        }

        fn local_unescrow(
            &mut self,
            _channel_id: &str,
            _denom: &str,
            _amount: Uint128,
        ) -> Result<(), crate::error::ContractError> {
            Ok(())
        }
    }

    #[test]
    fn receive_transfer_create_foreign() {
        let denom_str =
            encode_denom_hash(hash_denom("wasm.0xDEADC0DE/channel-1/from-counterparty"));
        assert_eq!(
            TestOnReceive { toggle: false }
                .receive_phase1_transfer(
                    &Addr::unchecked("0xDEADC0DE"),
                    &IbcEndpoint {
                        port_id: "wasm.0xDEADC0DE".into(),
                        channel_id: "channel-1".into(),
                    },
                    &IbcEndpoint {
                        port_id: "transfer".into(),
                        channel_id: "channel-34".into(),
                    },
                    "receiver",
                    "relayer",
                    vec![TransferToken {
                        denom: "from-counterparty".into(),
                        amount: Uint128::from(100u128),
                        fee: FeePerU128::percent(10u128.try_into().unwrap()).unwrap(),
                    }],
                    true
                )
                .unwrap()
                .1,
            vec![
                wasm_execute(
                    Addr::unchecked("0xDEADC0DE"),
                    &ExecuteMsg::RegisterDenom {
                        local_endpoint: IbcEndpoint {
                            port_id: "wasm.0xDEADC0DE".into(),
                            channel_id: "channel-1".into(),
                        },
                        denom: "wasm.0xDEADC0DE/channel-1/from-counterparty".into(),
                        hash: hash_denom("wasm.0xDEADC0DE/channel-1/from-counterparty")
                            .0
                            .get()
                            .into(),
                    },
                    Default::default()
                )
                .unwrap()
                .into(),
                TokenFactoryMsg::CreateDenom {
                    subdenom: denom_str.clone(),
                    metadata: None
                }
                .into(),
                TokenFactoryMsg::MintTokens {
                    denom: format!("factory/0xDEADC0DE/{}", denom_str),
                    amount: Uint128::from(91u128),
                    mint_to_address: "receiver".into()
                }
                .into(),
                TokenFactoryMsg::MintTokens {
                    denom: format!("factory/0xDEADC0DE/{}", denom_str),
                    amount: Uint128::from(9u128),
                    mint_to_address: "relayer".into()
                }
                .into(),
            ]
        );
    }

    #[test]
    fn receive_transfer_destination_collision_yields_different_hashes() {
        let source_endpoint_1 = IbcEndpoint {
            port_id: "wasm.0xDEADC0DE".into(),
            channel_id: "channel-1".into(),
        };
        let source_endpoint_2 = IbcEndpoint {
            port_id: "wasm.0xDEADC0DE".into(),
            channel_id: "channel-2".into(),
        };
        let conflicting_destination = IbcEndpoint {
            port_id: "transfer".into(),
            channel_id: "channel-34".into(),
        };

        let CosmosMsg::Custom(TokenFactoryMsg::MintTokens { denom: denom1, .. }) =
            &TestOnReceive { toggle: true }
                .receive_phase1_transfer(
                    &Addr::unchecked("0xDEADC0DE"),
                    &source_endpoint_1,
                    &conflicting_destination,
                    "receiver",
                    "relayer",
                    vec![TransferToken {
                        denom: "from-counterparty".into(),
                        amount: Uint128::from(100u128),
                        fee: FeePerU128::zero(),
                    }],
                    true,
                )
                .unwrap()
                .1[0]
        else {
            panic!("invalid msg");
        };

        let CosmosMsg::Custom(TokenFactoryMsg::MintTokens { denom: denom2, .. }) =
            &TestOnReceive { toggle: true }
                .receive_phase1_transfer(
                    &Addr::unchecked("0xDEADC0DE"),
                    &source_endpoint_2,
                    &conflicting_destination,
                    "receiver",
                    "relayer",
                    vec![TransferToken {
                        denom: "from-counterparty".into(),
                        amount: Uint128::from(100u128),
                        fee: FeePerU128::zero(),
                    }],
                    true,
                )
                .unwrap()
                .1[0]
        else {
            panic!("invalid msg");
        };

        assert_ne!(denom1, denom2);
    }

    #[test]
    fn receive_transfer_foreign() {
        assert_eq!(
            TestOnReceive { toggle: true }
                .receive_phase1_transfer(
                    &Addr::unchecked("0xDEADC0DE"),
                    &IbcEndpoint {
                        port_id: "wasm.0xDEADC0DE".into(),
                        channel_id: "channel-1".into(),
                    },
                    &IbcEndpoint {
                        port_id: "transfer".into(),
                        channel_id: "channel-34".into(),
                    },
                    "receiver",
                    "relayer",
                    vec![TransferToken {
                        denom: "from-counterparty".into(),
                        amount: Uint128::from(100u128),
                        fee: FeePerU128::percent(5u128.try_into().unwrap()).unwrap(),
                    }],
                    true
                )
                .unwrap()
                .1,
            vec![
                TokenFactoryMsg::MintTokens {
                    denom: format!(
                        "factory/0xDEADC0DE/{}",
                        encode_denom_hash(hash_denom(
                            "wasm.0xDEADC0DE/channel-1/from-counterparty"
                        ))
                    ),
                    amount: Uint128::from(96u128),
                    mint_to_address: "receiver".into()
                }
                .into(),
                TokenFactoryMsg::MintTokens {
                    denom: format!(
                        "factory/0xDEADC0DE/{}",
                        encode_denom_hash(hash_denom(
                            "wasm.0xDEADC0DE/channel-1/from-counterparty"
                        ))
                    ),
                    amount: Uint128::from(4u128),
                    mint_to_address: "relayer".into()
                }
                .into()
            ]
        );
    }

    #[test]
    fn receive_transfer_unwraps_local() {
        assert_eq!(
            TestOnReceive { toggle: true }
                .receive_phase1_transfer(
                    &Addr::unchecked("0xDEADC0DE"),
                    &IbcEndpoint {
                        port_id: "wasm.0xDEADC0DE".into(),
                        channel_id: "channel-1".into(),
                    },
                    &IbcEndpoint {
                        port_id: "transfer".into(),
                        channel_id: "channel-34".into(),
                    },
                    "receiver",
                    "relayer",
                    vec![TransferToken {
                        denom: "transfer/channel-34/local-denom".into(),
                        amount: Uint128::from(119u128),
                        fee: FeePerU128::percent(10u128.try_into().unwrap()).unwrap()
                    }],
                    true
                )
                .unwrap()
                .1,
            vec![
                BankMsg::Send {
                    to_address: "receiver".into(),
                    amount: vec![Coin {
                        denom: "local-denom".into(),
                        amount: Uint128::from(108u128)
                    }]
                }
                .into(),
                BankMsg::Send {
                    to_address: "relayer".into(),
                    amount: vec![Coin {
                        denom: "local-denom".into(),
                        amount: Uint128::from(11u128)
                    }]
                }
                .into()
            ]
        );
    }

    #[test]
    fn send_tokens_channel_remote_burn() {
        struct OnRemoteOnly;
        impl ForTokens for OnRemoteOnly {
            fn on_local(
                &mut self,
                _channel_id: &str,
                _denom: &str,
                _amount: Uint128,
                _fee_amount: Uint128,
            ) -> Result<
                Vec<cosmwasm_std::CosmosMsg<token_factory_api::TokenFactoryMsg>>,
                crate::error::ContractError,
            > {
                Ok(vec![])
            }

            fn on_remote(
                &mut self,
                _channel_id: &str,
                denom: &str,
                amount: Uint128,
                fee_amount: Uint128,
            ) -> Result<
                Vec<cosmwasm_std::CosmosMsg<token_factory_api::TokenFactoryMsg>>,
                crate::error::ContractError,
            > {
                Ok(vec![TokenFactoryMsg::BurnTokens {
                    denom: denom.into(),
                    amount: amount + fee_amount,
                    burn_from_address: "0xCAFEBABE".into(),
                }
                .into()])
            }
        }

        assert_eq!(
            OnRemoteOnly
                .execute(
                    &Addr::unchecked("0xCAFEBABE"),
                    &IbcEndpoint {
                        port_id: "transfer-source".into(),
                        channel_id: "blabla".into()
                    },
                    vec![
                        TransferToken {
                            denom: "transfer-source/blabla/remote-denom".into(),
                            amount: Uint128::from(119u128),
                            fee: FeePerU128::zero()
                        },
                        TransferToken {
                            denom: "transfer-source/blabla-2/remote-denom".into(),
                            amount: Uint128::from(10u128),
                            fee: FeePerU128::zero()
                        },
                        TransferToken {
                            denom: "transfer-source/blabla/remote-denom2".into(),
                            amount: Uint128::from(129u128),
                            fee: FeePerU128::zero()
                        },
                    ],
                )
                .unwrap(),
            vec![
                TokenFactoryMsg::BurnTokens {
                    denom: format!(
                        "factory/0xCAFEBABE/{}",
                        encode_denom_hash(hash_denom("transfer-source/blabla/remote-denom"))
                    ),
                    amount: Uint128::from(119u128),
                    burn_from_address: "0xCAFEBABE".into()
                }
                .into(),
                TokenFactoryMsg::BurnTokens {
                    denom: format!(
                        "factory/0xCAFEBABE/{}",
                        encode_denom_hash(hash_denom("transfer-source/blabla/remote-denom2"))
                    ),
                    amount: Uint128::from(129u128),
                    burn_from_address: "0xCAFEBABE".into()
                }
                .into()
            ]
        );
    }

    #[test]
    fn send_tokens_channel_local_escrow() {
        struct OnLocalOnly {
            total: Uint128,
        }
        impl ForTokens for OnLocalOnly {
            fn on_local(
                &mut self,
                _channel_id: &str,
                _denom: &str,
                amount: Uint128,
                fee_amount: Uint128,
            ) -> Result<
                Vec<cosmwasm_std::CosmosMsg<token_factory_api::TokenFactoryMsg>>,
                crate::error::ContractError,
            > {
                self.total += amount + fee_amount;
                Ok(Default::default())
            }

            fn on_remote(
                &mut self,
                _channel_id: &str,
                _denom: &str,
                _amount: Uint128,
                _fee_amount: Uint128,
            ) -> Result<
                Vec<cosmwasm_std::CosmosMsg<token_factory_api::TokenFactoryMsg>>,
                crate::error::ContractError,
            > {
                todo!()
            }
        }
        let mut state = OnLocalOnly { total: 0u8.into() };
        assert_eq!(
            state
                .execute(
                    &Addr::unchecked("0xCAFEBABE"),
                    &IbcEndpoint {
                        port_id: "transfer-source".into(),
                        channel_id: "blabla".into()
                    },
                    vec![
                        TransferToken {
                            denom: "transfer/channel-2/remote-denom".into(),
                            amount: Uint128::from(119u128),
                            fee: FeePerU128::zero()
                        },
                        TransferToken {
                            denom: "transfer/channel-2/remote-denom2".into(),
                            amount: Uint128::from(129u128),
                            fee: FeePerU128::zero()
                        }
                    ],
                )
                .unwrap(),
            vec![]
        );
        assert_eq!(state.total, Uint128::from(119u128 + 129u128));
    }

    #[test]
    fn normalize_identity() {
        assert_eq!(
            normalize_for_ibc_transfer(
                |_| Ok(Some("transfer/channel-331/from-counterparty".into())),
                "0xDEADC0DE",
                &IbcEndpoint {
                    port_id: "transfer".into(),
                    channel_id: "channel-332".into()
                },
                TransferToken {
                    denom: "factory/0xDEADC0DE/Fr4cnL94KoBkpvid2B4EQpoiLA4MnSjUhQUMGLDJ1Jf4".into(),
                    amount: Uint128::MAX,
                    fee: FeePerU128::zero()
                }
            )
            .unwrap(),
            TransferToken {
                denom: "factory/0xDEADC0DE/Fr4cnL94KoBkpvid2B4EQpoiLA4MnSjUhQUMGLDJ1Jf4".into(),
                amount: Uint128::MAX,
                fee: FeePerU128::zero()
            }
        );
        assert_eq!(
            normalize_for_ibc_transfer(
                |_| Ok(Some("transfer1/channel-332/from-counterparty".into())),
                "0xDEADC0DE",
                &IbcEndpoint {
                    port_id: "transfer".into(),
                    channel_id: "channel-332".into()
                },
                TransferToken {
                    denom: "factory/0xDEADC0DE/Fr4cnL94KoBkpvid2B4EQpoiLA4MnSjUhQUMGLDJ1Jf4".into(),
                    amount: Uint128::MAX,
                    fee: FeePerU128::zero()
                }
            )
            .unwrap(),
            TransferToken {
                denom: "factory/0xDEADC0DE/Fr4cnL94KoBkpvid2B4EQpoiLA4MnSjUhQUMGLDJ1Jf4".into(),
                amount: Uint128::MAX,
                fee: FeePerU128::zero()
            }
        );
    }

    #[test]
    fn normalize_strips() {
        assert_eq!(
            normalize_for_ibc_transfer(
                |_| Ok(Some("transfer/channel-332/blabla-1".into())),
                "0xDEADC0DE",
                &IbcEndpoint {
                    port_id: "transfer".into(),
                    channel_id: "channel-332".into()
                },
                TransferToken {
                    denom: "factory/0xDEADC0DE/Fr4cnL94KoBkpvid2B4EQpoiLA4MnSjUhQUMGLDJ1Jf4".into(),
                    amount: Uint128::MAX,
                    fee: FeePerU128::zero()
                }
            )
            .unwrap(),
            TransferToken {
                denom: "transfer/channel-332/blabla-1".into(),
                amount: Uint128::MAX,
                fee: FeePerU128::zero()
            }
        );
    }
}

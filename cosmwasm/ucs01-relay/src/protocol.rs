use cosmwasm_std::{
    wasm_execute, Addr, AnyMsg, Attribute, BankMsg, Binary, Coin, CosmosMsg, DepsMut, Env, Event,
    HexBinary, IbcEndpoint, IbcOrder, IbcPacket, IbcReceiveResponse, MessageInfo, Uint128, Uint512,
};
use prost::{Message, Name};
use protos::deferredack::v1beta1::{DeferredPacketInfo, MsgWriteDeferredAck};
use sha2::{Digest, Sha256};
use token_factory_api::TokenFactoryMsg;
use ucs01_relay_api::{
    middleware::{
        InFlightPfmPacket, Memo, MiddlewareError, PacketForward, PacketForwardError,
        PFM_ERROR_EVENT,
    },
    protocol::{
        AddrOf, TransferProtocol, ATTR_ERROR, ATTR_PFM, ATTR_SUCCESS, ATTR_VALUE_PFM_ACK,
        IBC_SEND_ID,
    },
    types::{
        make_foreign_denom, DenomOrigin, EncodingError, GenericAck, Ics20Ack, Ics20Packet,
        JsonWasm, TransferPacket, TransferToken, Ucs01Ack, Ucs01TransferPacket,
    },
};
use unionlabs::{
    encoding::{self, Encode, EncodeAs},
    ibc::core::client::height::Height,
};

use crate::{
    contract::execute_transfer,
    error::ContractError,
    msg::{ExecuteMsg, TransferMsg},
    state::{
        ChannelInfo, Hash, PfmRefundPacketKey, CHANNEL_STATE, FOREIGN_DENOM_TO_HASH, HASH_LENGTH,
        HASH_TO_FOREIGN_DENOM, IN_FLIGHT_PFM_PACKETS,
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
        sender: &AddrOf<Self::TokenPacket>,
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
        receiver: Addr,
    ) -> Result<IbcReceiveResponse<Self::CustomMsg>, Self::Error> {
        // Prepare forward message
        let msg_info = MessageInfo {
            sender: receiver,
            funds: tokens,
        };

        let timeout = forward.get_effective_timeout();

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

        let add_events = IbcReceiveResponse::without_ack()
            .add_submessages(transfer.messages)
            .add_events(transfer.events);
        Ok(add_events)
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

pub fn hash_denom(denom: &str) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(denom);
    hasher.finalize()[..HASH_LENGTH]
        .try_into()
        .expect("impossible")
}

pub fn hash_denom_str(denom: &str) -> String {
    format!("0x{}", hex::encode(hash_denom(denom)))
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
    mut hash_to_denom: impl FnMut(Hash) -> Result<Option<String>, ContractError>,
    contract_address: &str,
    endpoint: &IbcEndpoint,
    token: TransferToken,
) -> Result<TransferToken, ContractError> {
    let normalized_denom = match token
        .denom
        .strip_prefix("factory/")
        .and_then(|denom| denom.strip_prefix(contract_address))
        .and_then(|denom| denom.strip_prefix("/0x"))
    {
        Some(denom_hash) => {
            if let Some(normalized_denom) = hash_to_denom(
                hex::decode(denom_hash)
                    .expect("impossible")
                    .try_into()
                    .expect("impossible"),
            )? {
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
    })
}
trait OnReceive {
    fn foreign_toggle(
        &mut self,
        contract_address: &Addr,
        local_endpoint: &IbcEndpoint,
        denom: &str,
    ) -> Result<(bool, Hash, CosmosMsg<TokenFactoryMsg>), ContractError>;

    fn local_unescrow(
        &mut self,
        channel_id: &str,
        denom: &str,
        amount: Uint128,
    ) -> Result<(), ContractError>;

    fn receive_phase1_transfer(
        &mut self,
        contract_address: &Addr,
        endpoint: &IbcEndpoint,
        counterparty_endpoint: &IbcEndpoint,
        receiver: &str,
        tokens: Vec<TransferToken>,
    ) -> Result<(Vec<TransferToken>, Vec<CosmosMsg<TokenFactoryMsg>>), ContractError> {
        tokens
            .into_iter()
            .map(|TransferToken { denom, amount }| {
                match DenomOrigin::from((denom.as_str(), counterparty_endpoint)) {
                    DenomOrigin::Local { denom } => {
                        self.local_unescrow(&endpoint.channel_id, denom, amount)?;
                        Ok((
                            TransferToken {
                                denom: denom.to_string(),
                                amount,
                            },
                            vec![BankMsg::Send {
                                to_address: receiver.to_string(),
                                amount: vec![Coin {
                                    denom: denom.to_string(),
                                    amount,
                                }],
                            }
                            .into()],
                        ))
                    }
                    DenomOrigin::Remote { denom } => {
                        let foreign_denom = make_foreign_denom(endpoint, denom);
                        let (exists, hashed_foreign_denom, register_msg) =
                            self.foreign_toggle(contract_address, endpoint, &foreign_denom)?;
                        let normalized_foreign_denom =
                            format!("0x{}", hex::encode(hashed_foreign_denom));
                        let factory_denom =
                            format!("factory/{}/{}", contract_address, normalized_foreign_denom);
                        let mint = TokenFactoryMsg::MintTokens {
                            denom: factory_denom.clone(),
                            amount,
                            mint_to_address: receiver.to_string(),
                        };
                        Ok((
                            TransferToken {
                                denom: factory_denom,
                                amount,
                            },
                            if exists {
                                vec![mint.into()]
                            } else {
                                vec![
                                    register_msg,
                                    TokenFactoryMsg::CreateDenom {
                                        subdenom: normalized_foreign_denom.clone(),
                                        metadata: None,
                                    }
                                    .into(),
                                    mint.into(),
                                ]
                            },
                        ))
                    }
                }
            })
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
    ) -> Result<(bool, Hash, CosmosMsg<TokenFactoryMsg>), ContractError> {
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
                    hash: hash.into(),
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
    ) -> Result<Vec<CosmosMsg<TokenFactoryMsg>>, ContractError>;

    fn on_remote(
        &mut self,
        channel_id: &str,
        denom: &str,
        amount: Uint128,
    ) -> Result<Vec<CosmosMsg<TokenFactoryMsg>>, ContractError>;

    fn execute(
        &mut self,
        contract_address: &Addr,
        endpoint: &IbcEndpoint,
        tokens: Vec<TransferToken>,
    ) -> Result<Vec<CosmosMsg<TokenFactoryMsg>>, ContractError> {
        let mut messages = Vec::with_capacity(tokens.len());
        for TransferToken { denom, amount } in tokens {
            // This is the origin from the counterparty POV
            match DenomOrigin::from((denom.as_str(), endpoint)) {
                DenomOrigin::Local { denom } => {
                    // the denom has been previously normalized (factory/{}/ prefix removed), we must reconstruct to burn
                    let foreign_denom = hash_denom_str(&make_foreign_denom(endpoint, denom));
                    let factory_denom = format!("factory/{}/{}", contract_address, foreign_denom);
                    messages.append(&mut self.on_remote(
                        &endpoint.channel_id,
                        &factory_denom,
                        amount,
                    )?);
                }
                DenomOrigin::Remote { denom } => {
                    messages.append(&mut self.on_local(&endpoint.channel_id, denom, amount)?);
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
    ) -> Result<Vec<CosmosMsg<TokenFactoryMsg>>, ContractError> {
        increase_outstanding(self.deps.branch(), channel_id, denom, amount)?;
        Ok(Default::default())
    }

    fn on_remote(
        &mut self,
        _channel_id: &str,
        denom: &str,
        amount: Uint128,
    ) -> Result<Vec<CosmosMsg<TokenFactoryMsg>>, ContractError> {
        Ok(vec![TokenFactoryMsg::BurnTokens {
            denom: denom.into(),
            amount,
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
    ) -> Result<Vec<CosmosMsg<TokenFactoryMsg>>, ContractError> {
        decrease_outstanding(self.deps.branch(), channel_id, denom, amount)?;
        Ok(vec![BankMsg::Send {
            to_address: self.receiver.clone(),
            amount: vec![Coin {
                denom: denom.into(),
                amount,
            }],
        }
        .into()])
    }

    fn on_remote(
        &mut self,
        _channel_id: &str,
        denom: &str,
        amount: Uint128,
    ) -> Result<Vec<CosmosMsg<TokenFactoryMsg>>, ContractError> {
        Ok(vec![TokenFactoryMsg::MintTokens {
            denom: denom.into(),
            amount,
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

    type TokenPacket = Ics20Packet;
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
        receiver: &AddrOf<Self::TokenPacket>,
        tokens: Vec<TransferToken>,
    ) -> Result<(Vec<TransferToken>, Vec<CosmosMsg<Self::CustomMsg>>), ContractError> {
        let (tokens, msgs) = StatefulOnReceive {
            deps: self.common.deps.branch(),
        }
        .receive_phase1_transfer(
            &self.common.env.contract.address,
            &self.common.channel.endpoint,
            &self.common.channel.counterparty_endpoint,
            receiver.as_str(),
            tokens,
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
                    .may_load(
                        self.common.deps.storage,
                        (self.common.channel.endpoint.clone().into(), hash),
                    )
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
    ) -> Result<Self::TokenPacket, ucs01_relay_api::types::EncodingError> {
        Ics20Packet::try_from(packet)
    }

    fn packet_forward(
        &mut self,
        packet: Self::TokenPacket,
        original_packet: IbcPacket,
        forward: PacketForward,
        processed: bool,
    ) -> cosmwasm_std::IbcReceiveResponse<Self::CustomMsg> {
        let mut msgs: Vec<CosmosMsg<Self::CustomMsg>> = Vec::new();
        // Override the receiving address for intermediate transfers on this chain with the contract address
        let override_addr = self.self_addr().to_owned();

        // If not already processed by other middleware, receive tokens into the contract address.
        let mut tokens: Vec<Coin> = Vec::new();
        if !processed {
            msgs.append(&mut match self
                .receive_transfer(&override_addr.to_string(), packet.tokens().to_vec())
            {
                Ok((t, msgs)) => {
                    t.into_iter().for_each(|t| {
                        tokens.push(Coin {
                            denom: t.denom,
                            amount: t.amount,
                        })
                    });
                    msgs
                }
                Err(error) => {
                    return Self::receive_error(error);
                }
            });
        }

        // Forward the packet
        let forward_response = match self.forward_transfer_packet(
            tokens,
            original_packet.clone(),
            forward,
            override_addr,
        ) {
            Ok(forward_response) => forward_response,
            Err(e) => {
                return IbcReceiveResponse::new(
                    Self::ack_failure(e.to_string()).encode_as::<JsonWasm>(),
                )
                .add_event(Event::new(PFM_ERROR_EVENT).add_attribute("error", e.to_string()))
            }
        };

        IbcReceiveResponse::without_ack()
            .add_messages(msgs)
            .add_submessages(forward_response.messages)
            .add_events(forward_response.events)
    }

    fn forward_transfer_packet(
        &mut self,
        tokens: Vec<Coin>,
        original_packet: IbcPacket,
        forward: PacketForward,
        receiver: Addr,
    ) -> Result<IbcReceiveResponse<Self::CustomMsg>, ContractError> {
        self.do_forward_transfer_packet(tokens, original_packet, forward, receiver)
    }

    fn pfm_ack(
        &mut self,
        ack: GenericAck,
        ibc_packet: IbcPacket,
        refund_info: InFlightPfmPacket,
        sender: &AddrOf<Self::TokenPacket>,
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
}

pub struct Ucs01Protocol<'a> {
    pub common: ProtocolCommon<'a>,
}

impl<'a> TransferProtocol for Ucs01Protocol<'a> {
    const VERSION: &'static str = "ucs01-relay-1";
    const ORDERING: IbcOrder = IbcOrder::Unordered;
    const RECEIVE_REPLY_ID: u64 = 1;

    type TokenPacket = Ucs01TransferPacket;
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
        receiver: &AddrOf<Self::TokenPacket>,
        tokens: Vec<TransferToken>,
    ) -> Result<(Vec<TransferToken>, Vec<CosmosMsg<Self::CustomMsg>>), ContractError> {
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
            tokens,
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
                    .may_load(
                        self.common.deps.storage,
                        (self.common.channel.endpoint.clone().into(), hash),
                    )
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
    ) -> Result<Self::TokenPacket, EncodingError> {
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

    fn packet_forward(
        &mut self,
        packet: Self::TokenPacket,
        original_packet: IbcPacket,
        forward: PacketForward,
        processed: bool,
    ) -> cosmwasm_std::IbcReceiveResponse<Self::CustomMsg> {
        let mut msgs: Vec<CosmosMsg<Self::CustomMsg>> = Vec::new();
        // Override the receiving address for intermediate transfers on this chain with the contract address
        let override_addr = self.self_addr().to_owned();
        let override_con_addr = match self
            .common
            .deps
            .api
            .addr_canonicalize(override_addr.as_str())
        {
            Ok(addr) => addr,
            Err(error) => {
                return Self::receive_error(error);
            }
        };

        // If not already processed by other middleware, receive tokens into the contract address.
        let mut tokens: Vec<Coin> = Vec::new();
        if !processed {
            msgs.append(&mut match self
                .receive_transfer(&override_con_addr.into(), packet.tokens().to_vec())
            {
                Ok((t, msgs)) => {
                    t.into_iter().for_each(|t| {
                        tokens.push(Coin {
                            denom: t.denom,
                            amount: t.amount,
                        })
                    });
                    msgs
                }
                Err(error) => {
                    return Self::receive_error(error);
                }
            });
        }

        // Forward the packet
        let forward_response = match self.forward_transfer_packet(
            tokens,
            original_packet.clone(),
            forward,
            override_addr,
        ) {
            Ok(forward_response) => forward_response,
            Err(e) => {
                return IbcReceiveResponse::new(Self::ack_failure(e.to_string()).encode())
                    .add_event(Event::new(PFM_ERROR_EVENT).add_attribute("error", e.to_string()))
            }
        };

        IbcReceiveResponse::without_ack()
            .add_messages(msgs)
            .add_submessages(forward_response.messages)
            .add_events(forward_response.events)
    }

    fn forward_transfer_packet(
        &mut self,
        tokens: Vec<Coin>,
        original_packet: IbcPacket,
        forward: PacketForward,
        receiver: Addr,
    ) -> Result<IbcReceiveResponse<Self::CustomMsg>, ContractError> {
        self.do_forward_transfer_packet(tokens, original_packet, forward, receiver)
    }

    #[allow(clippy::type_complexity)]
    fn pfm_ack(
        &mut self,
        ack: GenericAck,
        ibc_packet: IbcPacket,
        refund_info: InFlightPfmPacket,
        sender: &AddrOf<Self::TokenPacket>,
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
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        wasm_execute, Addr, BankMsg, Coin, CosmosMsg, IbcEndpoint, Uint128,
    };
    use token_factory_api::TokenFactoryMsg;
    use ucs01_relay_api::{protocol::TransferProtocol, types::TransferToken};

    use super::{hash_denom, ForTokens, OnReceive, StatefulOnReceive};
    use crate::{
        error::ContractError,
        msg::ExecuteMsg,
        protocol::{hash_denom_str, normalize_for_ibc_transfer, Ics20Protocol},
        state::{ChannelInfo, Hash},
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
                    info: mock_info("", &[]),
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
        ) -> Result<(bool, Hash, CosmosMsg<TokenFactoryMsg>), ContractError> {
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
        let denom_str = hash_denom_str("wasm.0xDEADC0DE/channel-1/from-counterparty");
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
                    vec![TransferToken {
                        denom: "from-counterparty".into(),
                        amount: Uint128::from(100u128)
                    },],
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
                        hash: hash_denom("wasm.0xDEADC0DE/channel-1/from-counterparty").into(),
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
                    amount: Uint128::from(100u128),
                    mint_to_address: "receiver".into()
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
                    vec![TransferToken {
                        denom: "from-counterparty".into(),
                        amount: Uint128::from(100u128),
                    }],
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
                    vec![TransferToken {
                        denom: "from-counterparty".into(),
                        amount: Uint128::from(100u128),
                    }],
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
                    vec![TransferToken {
                        denom: "from-counterparty".into(),
                        amount: Uint128::from(100u128)
                    },],
                )
                .unwrap()
                .1,
            vec![TokenFactoryMsg::MintTokens {
                denom: format!(
                    "factory/0xDEADC0DE/{}",
                    hash_denom_str("wasm.0xDEADC0DE/channel-1/from-counterparty")
                ),
                amount: Uint128::from(100u128),
                mint_to_address: "receiver".into()
            }
            .into()]
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
                    vec![TransferToken {
                        denom: "transfer/channel-34/local-denom".into(),
                        amount: Uint128::from(119u128)
                    }],
                )
                .unwrap()
                .1,
            vec![BankMsg::Send {
                to_address: "receiver".into(),
                amount: vec![Coin {
                    denom: "local-denom".into(),
                    amount: Uint128::from(119u128)
                }]
            }
            .into()]
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
            ) -> Result<
                Vec<cosmwasm_std::CosmosMsg<token_factory_api::TokenFactoryMsg>>,
                crate::error::ContractError,
            > {
                Ok(vec![TokenFactoryMsg::BurnTokens {
                    denom: denom.into(),
                    amount,
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
                            amount: Uint128::from(119u128)
                        },
                        TransferToken {
                            denom: "transfer-source/blabla-2/remote-denom".into(),
                            amount: Uint128::from(10u128)
                        },
                        TransferToken {
                            denom: "transfer-source/blabla/remote-denom2".into(),
                            amount: Uint128::from(129u128)
                        },
                    ],
                )
                .unwrap(),
            vec![
                TokenFactoryMsg::BurnTokens {
                    denom: format!(
                        "factory/0xCAFEBABE/{}",
                        hash_denom_str("transfer-source/blabla/remote-denom")
                    ),
                    amount: Uint128::from(119u128),
                    burn_from_address: "0xCAFEBABE".into()
                }
                .into(),
                TokenFactoryMsg::BurnTokens {
                    denom: format!(
                        "factory/0xCAFEBABE/{}",
                        hash_denom_str("transfer-source/blabla/remote-denom2")
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
            ) -> Result<
                Vec<cosmwasm_std::CosmosMsg<token_factory_api::TokenFactoryMsg>>,
                crate::error::ContractError,
            > {
                self.total += amount;
                Ok(Default::default())
            }

            fn on_remote(
                &mut self,
                _channel_id: &str,
                _denom: &str,
                _amount: Uint128,
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
                            amount: Uint128::from(119u128)
                        },
                        TransferToken {
                            denom: "transfer/channel-2/remote-denom2".into(),
                            amount: Uint128::from(129u128)
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
                    denom: "factory/0xDEADC0DE/0xaf30fd00576e1d27471a4d2b0c0487dc6876e0589e".into(),
                    amount: Uint128::MAX
                }
            )
            .unwrap(),
            TransferToken {
                denom: "factory/0xDEADC0DE/0xaf30fd00576e1d27471a4d2b0c0487dc6876e0589e".into(),
                amount: Uint128::MAX
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
                    denom: "factory/0xDEADC0DE/0xaf30fd00576e1d27471a4d2b0c0487dc6876e0589e".into(),
                    amount: Uint128::MAX
                }
            )
            .unwrap(),
            TransferToken {
                denom: "factory/0xDEADC0DE/0xaf30fd00576e1d27471a4d2b0c0487dc6876e0589e".into(),
                amount: Uint128::MAX
            }
        );
    }

    #[test]
    fn normalize_strip() {
        assert_eq!(
            normalize_for_ibc_transfer(
                |_| Ok(Some("transfer/channel-332/blabla-1".into())),
                "0xDEADC0DE",
                &IbcEndpoint {
                    port_id: "transfer".into(),
                    channel_id: "channel-332".into()
                },
                TransferToken {
                    denom: "factory/0xDEADC0DE/0xaf30fd00576e1d27471a4d2b0c0487dc6876e0589e".into(),
                    amount: Uint128::MAX
                }
            )
            .unwrap(),
            TransferToken {
                denom: "transfer/channel-332/blabla-1".into(),
                amount: Uint128::MAX
            }
        );
    }
}

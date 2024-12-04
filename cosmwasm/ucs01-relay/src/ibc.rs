#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_json, wasm_execute, Addr, DepsMut, Env, Ibc3ChannelOpenResponse, IbcAcknowledgement,
    IbcBasicResponse, IbcChannel, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg,
    IbcEndpoint, IbcPacket, IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg,
    IbcReceiveResponse, IbcTimeout, IbcTimeoutBlock, MessageInfo, Reply, Response, SubMsgResult,
    Timestamp,
};
use ibc_solidity::{Channel, Packet};
use prost::{Message, Name};
use protos::cosmwasm::wasm::v1::MsgIbcSendResponse;
use token_factory_api::TokenFactoryMsg;
use ucs01_relay_api::{
    middleware::InFlightPfmPacket,
    protocol::{TransferProtocol, IBC_SEND_ID},
};
use union_ibc_msg::{
    module::UnionIbcMsg,
    msg::{ExecuteMsg as UnionIbcHostMsg, MsgWriteAcknowledgement},
    query::QueryMsg as UnionIbcQuery,
};

pub type IbcResponse = IbcBasicResponse<TokenFactoryMsg>;

use crate::{
    contract::query_ibc_channel,
    error::ContractError,
    protocol::{packet_key, protocol_ordering, Ics20Protocol, ProtocolCommon, Ucs01Protocol},
    state::{CONFIG, IN_FLIGHT_PFM_PACKETS},
};

fn to_response<T>(
    IbcReceiveResponse {
        acknowledgement,
        messages,
        attributes,
        events,
        ..
    }: IbcReceiveResponse<T>,
) -> Response<T> {
    let response = Response::<T>::new()
        .add_submessages(messages)
        .add_attributes(attributes)
        .add_events(events);

    if let Some(ack) = acknowledgement {
        response.set_data(ack)
    } else {
        response
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(
    deps: DepsMut,
    _: Env,
    reply: Reply,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    match (reply.id, reply.result) {
        // RECEIVE_REPLY_ID is associated with submessages emitted during handling of `ibc_packet_receive`
        (Ics20Protocol::RECEIVE_REPLY_ID, SubMsgResult::Err(err)) => {
            Ok(to_response(Ics20Protocol::receive_error(err)))
        }
        (Ucs01Protocol::RECEIVE_REPLY_ID, SubMsgResult::Err(err)) => {
            Ok(to_response(Ucs01Protocol::receive_error(err)))
        }
        // IBC_SEND_ID is associated with submessages emitted during handling of `send`, which is called via `execute_transfer`, which is used both in PFM and non-PFM contexts
        (IBC_SEND_ID, SubMsgResult::Ok(value)) => {
            // this means this is not pfm
            if reply.payload.is_empty() {
                return Ok(Response::new());
            }

            let mut in_flight_packet =
                serde_json_wasm::from_slice::<InFlightPfmPacket>(reply.payload.as_slice())
                    .expect("binary is type");

            match value
                .msg_responses
                .iter()
                .find(|msg_response| msg_response.type_url == MsgIbcSendResponse::type_url())
            {
                Some(msg_response) => {
                    let send_response = MsgIbcSendResponse::decode(msg_response.value.as_slice())
                        .expect("is type url");
                    in_flight_packet.forward_packet.sequence = send_response.sequence;
                }
                None =>
                {
                    #[allow(deprecated)]
                    if from_json::<Packet>(value.data.unwrap_or_default()).is_err() {
                        return Err(ContractError::InvalidReply);
                    }
                }
            };

            let refund_packet_key = packet_key(&in_flight_packet.forward_packet);

            IN_FLIGHT_PFM_PACKETS
                .save(deps.storage, refund_packet_key, &in_flight_packet)
                .expect("infallible update");

            Ok(Response::new().add_event(in_flight_packet.create_hop_event()?))
        }
        (IBC_SEND_ID, SubMsgResult::Err(err)) => {
            // this means this is not pfm
            if reply.payload.is_empty() {
                return Err(ContractError::PfmSendPacketError { err });
            }

            // decode the payload to figure out the source channel
            let in_flight_packet =
                serde_json_wasm::from_slice::<InFlightPfmPacket>(reply.payload.as_slice())
                    .expect("binary is type");

            match &*in_flight_packet.origin_protocol_version {
                Ucs01Protocol::VERSION => Ok(to_response(Ucs01Protocol::receive_error(err))),
                Ics20Protocol::VERSION => Ok(to_response(Ics20Protocol::receive_error(err))),
                // in_flight_packet.origin_protocol_version is only ever set by us, so if it is set incorrectly then it is a bug
                version => unreachable!("unknown protocol version: {version}"),
            }
        }
        (_, result) => Err(ContractError::UnknownReply {
            id: reply.id,
            variant: result,
        }),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// enforces ordering and versioning constraints
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<Option<Ibc3ChannelOpenResponse>, ContractError> {
    enforce_order_and_version(msg.channel(), msg.counterparty_version())?;
    Ok(None)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_connect(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> Result<IbcResponse, ContractError> {
    enforce_order_and_version(msg.channel(), msg.counterparty_version())?;
    Ok(IbcResponse::default())
}

pub(crate) fn enforce_order_and_version(
    channel: &IbcChannel,
    counterparty_version: Option<&str>,
) -> Result<(), ContractError> {
    let channel_ordering =
        protocol_ordering(&channel.version).ok_or(ContractError::UnknownProtocol {
            channel_id: channel.endpoint.channel_id.clone(),
            protocol_version: channel.version.clone(),
        })?;
    if let Some(version) = counterparty_version {
        if protocol_ordering(version).is_none() {
            return Err(ContractError::UnknownProtocol {
                channel_id: channel.endpoint.channel_id.clone(),
                protocol_version: version.to_string(),
            });
        }
        if version != channel.version {
            return Err(ContractError::ProtocolMismatch {
                channel_id: channel.endpoint.channel_id.clone(),
                protocol_version: channel.version.clone(),
                counterparty_protocol_version: version.to_string(),
            });
        }
    }
    if channel.order != channel_ordering {
        return Err(ContractError::InvalidChannelOrdering {
            expected: channel_ordering,
            actual: channel.order.clone(),
        });
    }
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_close(
    _deps: DepsMut,
    _env: Env,
    _channel: IbcChannelCloseMsg,
) -> Result<IbcResponse, ContractError> {
    Err(ContractError::Unauthorized)
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// Check to see if we have any balance here
/// We should not return an error if possible, but rather an acknowledgement of failure
pub fn ibc_packet_receive(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse<TokenFactoryMsg>, ContractError> {
    let channel = query_ibc_channel(deps.as_ref(), msg.packet.dest.channel_id.clone())?;

    let info = MessageInfo {
        sender: msg.relayer,
        funds: Default::default(),
    };

    match channel.version.as_ref() {
        Ics20Protocol::VERSION => Ok((Ics20Protocol {
            common: ProtocolCommon {
                deps,
                env,
                info,
                channel,
            },
        })
        .receive(msg.packet)),
        Ucs01Protocol::VERSION => Ok((Ucs01Protocol {
            common: ProtocolCommon {
                deps,
                env,
                info,
                channel,
            },
        })
        .receive(msg.packet)),
        v => Err(ContractError::UnknownProtocol {
            channel_id: msg.packet.dest.channel_id,
            protocol_version: v.into(),
        }),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// check if success or failure and update balance, or return funds
pub fn ibc_packet_ack(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketAckMsg,
) -> Result<IbcResponse, ContractError> {
    let channel = query_ibc_channel(deps.as_ref(), msg.original_packet.src.channel_id.clone())?;

    let info = MessageInfo {
        sender: msg.relayer.clone(),
        funds: Default::default(),
    };

    match channel.version.as_str() {
        Ics20Protocol::VERSION => (Ics20Protocol {
            common: ProtocolCommon {
                deps,
                env,
                info,
                channel,
            },
        })
        .send_ack(msg),
        Ucs01Protocol::VERSION => (Ucs01Protocol {
            common: ProtocolCommon {
                deps,
                env,
                info,
                channel,
            },
        })
        .send_ack(msg),
        v => Err(ContractError::UnknownProtocol {
            channel_id: msg.original_packet.dest.channel_id,
            protocol_version: v.into(),
        }),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// return fund to original sender (same as failure in ibc_packet_ack)
pub fn ibc_packet_timeout(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketTimeoutMsg,
) -> Result<IbcResponse, ContractError> {
    let channel = query_ibc_channel(deps.as_ref(), msg.packet.src.channel_id.clone())?;

    let info = MessageInfo {
        sender: msg.relayer,
        funds: Default::default(),
    };

    match channel.version.as_str() {
        Ics20Protocol::VERSION => (Ics20Protocol {
            common: ProtocolCommon {
                deps,
                env,
                info,
                channel,
            },
        })
        .send_timeout(msg.packet),
        Ucs01Protocol::VERSION => (Ucs01Protocol {
            common: ProtocolCommon {
                deps,
                env,
                info,
                channel,
            },
        })
        .send_timeout(msg.packet),
        v => Err(ContractError::UnknownProtocol {
            channel_id: msg.packet.dest.channel_id,
            protocol_version: v.into(),
        }),
    }
}

pub(crate) fn execute_union_ibc(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: UnionIbcMsg,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    let ibc_host = CONFIG.load(deps.storage)?.ibc_host;
    if info.sender != ibc_host {
        return Err(ContractError::OnlyIBCHost);
    }
    match msg {
        UnionIbcMsg::OnChannelOpenInit {
            channel_id,
            version,
            ..
        } => {
            if version != Ucs01Protocol::VERSION {
                return Err(ContractError::UnknownProtocol {
                    channel_id: channel_id.to_string(),
                    protocol_version: version.to_string(),
                });
            }
            Ok(Response::new())
        }
        UnionIbcMsg::OnChannelOpenTry {
            channel_id,
            version,
            counterparty_version,
            ..
        } => {
            if version != Ucs01Protocol::VERSION {
                return Err(ContractError::UnknownProtocol {
                    channel_id: channel_id.to_string(),
                    protocol_version: version.to_string(),
                });
            }
            if counterparty_version != Ucs01Protocol::VERSION {
                return Err(ContractError::UnknownProtocol {
                    channel_id: channel_id.to_string(),
                    protocol_version: counterparty_version.to_string(),
                });
            }
            Ok(Response::new())
        }
        UnionIbcMsg::OnChannelOpenAck { .. } => Ok(Response::new()),
        UnionIbcMsg::OnChannelOpenConfirm { .. } => Ok(Response::new()),
        UnionIbcMsg::OnChannelCloseInit { .. } => Err(ContractError::Unauthorized),
        UnionIbcMsg::OnChannelCloseConfirm { .. } => Err(ContractError::Unauthorized),
        UnionIbcMsg::OnIntentRecvPacket { .. } => Err(ContractError::Unsupported),
        UnionIbcMsg::OnRecvPacket {
            packet, relayer, ..
        } => {
            let channel = deps.querier.query_wasm_smart::<Channel>(
                &ibc_host,
                &UnionIbcQuery::GetChannel {
                    channel_id: packet.destination_channel,
                },
            )?;

            let info = MessageInfo {
                sender: Addr::unchecked(relayer),
                funds: Default::default(),
            };

            let ibc_endpoint_src = IbcEndpoint {
                port_id: hex::encode(channel.counterparty_port_id),
                channel_id: packet.source_channel.to_string(),
            };

            let ibc_endpoint_dst = IbcEndpoint {
                port_id: format!("wasm.{}", env.contract.address),
                channel_id: packet.destination_channel.to_string(),
            };

            let ibc_channel = IbcChannel::new(
                ibc_endpoint_dst.clone(),
                ibc_endpoint_src.clone(),
                cosmwasm_std::IbcOrder::Ordered,
                channel.version.clone(),
                channel.connection_id.to_string(),
            );

            let ibc_packet = IbcPacket::new(
                packet.data.to_vec(),
                ibc_endpoint_src,
                ibc_endpoint_dst,
                0,
                IbcTimeout::with_both(
                    IbcTimeoutBlock {
                        revision: 0,
                        height: packet.timeout_height,
                    },
                    Timestamp::from_nanos(packet.timeout_timestamp),
                ),
            );

            let msg = match channel.version.as_ref() {
                Ics20Protocol::VERSION => Ok((Ics20Protocol {
                    common: ProtocolCommon {
                        deps,
                        env,
                        info,
                        channel: ibc_channel,
                    },
                })
                .receive(ibc_packet)),
                Ucs01Protocol::VERSION => Ok((Ucs01Protocol {
                    common: ProtocolCommon {
                        deps,
                        env,
                        info,
                        channel: ibc_channel,
                    },
                })
                .receive(ibc_packet)),
                v => Err(ContractError::UnknownProtocol {
                    channel_id: packet.destination_channel.to_string(),
                    protocol_version: v.into(),
                }),
            }?;

            // Marshal back the message to a base Response
            let mut response = Response::new()
                .add_submessages(msg.messages)
                .add_events(msg.events)
                .add_attributes(msg.attributes);

            // Instantly dispatches the acknowledgement back to the host if not async.
            if let Some(ack) = msg.acknowledgement {
                response = response.add_message(wasm_execute(
                    &ibc_host,
                    &UnionIbcHostMsg::WriteAcknowledgement(MsgWriteAcknowledgement {
                        channel_id: packet.destination_channel,
                        packet,
                        acknowledgement: Vec::from(ack).into(),
                    }),
                    vec![],
                )?);
            }

            Ok(response)
        }
        UnionIbcMsg::OnAcknowledgementPacket {
            packet,
            relayer,
            acknowledgement,
        } => {
            let relayer = Addr::unchecked(relayer);

            let channel = deps.querier.query_wasm_smart::<Channel>(
                &ibc_host,
                &UnionIbcQuery::GetChannel {
                    channel_id: packet.destination_channel,
                },
            )?;

            let info = MessageInfo {
                sender: relayer.clone(),
                funds: Default::default(),
            };

            let ibc_endpoint_src = IbcEndpoint {
                port_id: hex::encode(channel.counterparty_port_id),
                channel_id: packet.source_channel.to_string(),
            };

            let ibc_endpoint_dst = IbcEndpoint {
                port_id: format!("wasm.{}", env.contract.address),
                channel_id: packet.destination_channel.to_string(),
            };

            let ibc_channel = IbcChannel::new(
                ibc_endpoint_src.clone(),
                ibc_endpoint_dst.clone(),
                cosmwasm_std::IbcOrder::Ordered,
                channel.version.clone(),
                channel.connection_id.to_string(),
            );

            let ibc_packet = IbcPacket::new(
                packet.data.to_vec(),
                ibc_endpoint_src,
                ibc_endpoint_dst,
                0,
                IbcTimeout::with_both(
                    IbcTimeoutBlock {
                        revision: 0,
                        height: packet.timeout_height,
                    },
                    Timestamp::from_nanos(packet.timeout_timestamp),
                ),
            );

            let msg = IbcPacketAckMsg::new(
                IbcAcknowledgement::new(acknowledgement.to_vec()),
                ibc_packet,
                relayer,
            );

            let response = match channel.version.as_ref() {
                Ics20Protocol::VERSION => Ok((Ics20Protocol {
                    common: ProtocolCommon {
                        deps,
                        env,
                        info,
                        channel: ibc_channel,
                    },
                })
                .send_ack(msg)),
                Ucs01Protocol::VERSION => Ok((Ucs01Protocol {
                    common: ProtocolCommon {
                        deps,
                        env,
                        info,
                        channel: ibc_channel,
                    },
                })
                .send_ack(msg)),
                v => Err(ContractError::UnknownProtocol {
                    channel_id: packet.destination_channel.to_string(),
                    protocol_version: v.into(),
                }),
            }??;

            Ok(Response::new()
                .add_submessages(response.messages)
                .add_events(response.events)
                .add_attributes(response.attributes))
        }
        UnionIbcMsg::OnTimeoutPacket { packet, relayer } => {
            let channel = deps.querier.query_wasm_smart::<Channel>(
                &ibc_host,
                &UnionIbcQuery::GetChannel {
                    channel_id: packet.destination_channel,
                },
            )?;

            let info = MessageInfo {
                sender: Addr::unchecked(relayer),
                funds: Default::default(),
            };

            let ibc_endpoint_src = IbcEndpoint {
                port_id: hex::encode(channel.counterparty_port_id),
                channel_id: packet.source_channel.to_string(),
            };

            let ibc_endpoint_dst = IbcEndpoint {
                port_id: format!("wasm.{}", env.contract.address),
                channel_id: packet.destination_channel.to_string(),
            };

            let ibc_channel = IbcChannel::new(
                ibc_endpoint_src.clone(),
                ibc_endpoint_dst.clone(),
                cosmwasm_std::IbcOrder::Ordered,
                channel.version.clone(),
                channel.connection_id.to_string(),
            );

            let ibc_packet = IbcPacket::new(
                packet.data.to_vec(),
                ibc_endpoint_src,
                ibc_endpoint_dst,
                0,
                IbcTimeout::with_both(
                    IbcTimeoutBlock {
                        revision: 0,
                        height: packet.timeout_height,
                    },
                    Timestamp::from_nanos(packet.timeout_timestamp),
                ),
            );

            let response = match channel.version.as_ref() {
                Ics20Protocol::VERSION => Ok((Ics20Protocol {
                    common: ProtocolCommon {
                        deps,
                        env,
                        info,
                        channel: ibc_channel,
                    },
                })
                .send_timeout(ibc_packet)),
                Ucs01Protocol::VERSION => Ok((Ucs01Protocol {
                    common: ProtocolCommon {
                        deps,
                        env,
                        info,
                        channel: ibc_channel,
                    },
                })
                .send_timeout(ibc_packet)),
                v => Err(ContractError::UnknownProtocol {
                    channel_id: packet.destination_channel.to_string(),
                    protocol_version: v.into(),
                }),
            }??;

            Ok(Response::new()
                .add_submessages(response.messages)
                .add_events(response.events)
                .add_attributes(response.attributes))
        }
    }
}

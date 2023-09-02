use cosmwasm_std::{
    entry_point, DepsMut, Env, Ibc3ChannelOpenResponse, IbcBasicResponse, IbcChannel,
    IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcPacketAckMsg,
    IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse, MessageInfo, Reply, Response,
    SubMsgResult,
};
use token_factory_api::TokenFactoryMsg;
use ucs01_relay_api::protocol::TransferProtocol;

use crate::{
    error::ContractError,
    protocol::{protocol_ordering, Ics20Protocol, ProtocolCommon, Ucs01Protocol},
    state::{ChannelInfo, CHANNEL_INFO},
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
    Response::<T>::new()
        .add_submessages(messages)
        .add_attributes(attributes)
        .add_events(events)
        .set_data(acknowledgement)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_: DepsMut, _: Env, reply: Reply) -> Result<Response<TokenFactoryMsg>, ContractError> {
    match (reply.id, reply.result) {
        (Ics20Protocol::RECEIVE_REPLY_ID, SubMsgResult::Err(err)) => {
            Ok(to_response(Ics20Protocol::receive_error(err)))
        }
        (Ucs01Protocol::RECEIVE_REPLY_ID, SubMsgResult::Err(err)) => {
            Ok(to_response(Ucs01Protocol::receive_error(err)))
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
/// record the channel in CHANNEL_INFO
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> Result<IbcBasicResponse, ContractError> {
    enforce_order_and_version(msg.channel(), msg.counterparty_version())?;
    let channel: IbcChannel = msg.into();
    let info = ChannelInfo {
        endpoint: channel.endpoint,
        counterparty_endpoint: channel.counterparty_endpoint,
        connection_id: channel.connection_id,
        protocol_version: channel.version,
    };
    CHANNEL_INFO.save(deps.storage, &info.endpoint.channel_id, &info)?;
    Ok(IbcBasicResponse::default())
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
) -> Result<IbcBasicResponse, ContractError> {
    // Not allowed.
    unimplemented!();
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// Check to see if we have any balance here
/// We should not return an error if possible, but rather an acknowledgement of failure
pub fn ibc_packet_receive(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse<TokenFactoryMsg>, ContractError> {
    let channel_info = CHANNEL_INFO.load(deps.storage, &msg.packet.dest.channel_id)?;

    let info = MessageInfo {
        sender: msg.relayer,
        funds: Default::default(),
    };

    match channel_info.protocol_version {
        ref v if v == Ics20Protocol::VERSION => Ok(Ics20Protocol {
            common: ProtocolCommon {
                deps,
                env,
                info,
                channel: channel_info,
            },
        }
        .receive_phase0(msg.packet.data)),
        ref v if v == Ucs01Protocol::VERSION => Ok(Ucs01Protocol {
            common: ProtocolCommon {
                deps,
                env,
                info,
                channel: channel_info,
            },
        }
        .receive_phase0(msg.packet.data)),
        v => Err(ContractError::UnknownProtocol {
            channel_id: msg.packet.dest.channel_id,
            protocol_version: v,
        }),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// check if success or failure and update balance, or return funds
pub fn ibc_packet_ack(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketAckMsg,
) -> Result<IbcBasicResponse<TokenFactoryMsg>, ContractError> {
    let channel_info = CHANNEL_INFO.load(deps.storage, &msg.original_packet.dest.channel_id)?;

    let info = MessageInfo {
        sender: msg.relayer,
        funds: Default::default(),
    };

    match channel_info.protocol_version {
        ref v if v == Ics20Protocol::VERSION => Ics20Protocol {
            common: ProtocolCommon {
                deps,
                env,
                info,
                channel: channel_info,
            },
        }
        .send_ack(msg.acknowledgement.data, msg.original_packet.data),
        ref v if v == Ucs01Protocol::VERSION => Ucs01Protocol {
            common: ProtocolCommon {
                deps,
                env,
                info,
                channel: channel_info,
            },
        }
        .send_ack(msg.acknowledgement.data, msg.original_packet.data),
        v => Err(ContractError::UnknownProtocol {
            channel_id: msg.original_packet.dest.channel_id,
            protocol_version: v,
        }),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// return fund to original sender (same as failure in ibc_packet_ack)
pub fn ibc_packet_timeout(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketTimeoutMsg,
) -> Result<IbcBasicResponse<TokenFactoryMsg>, ContractError> {
    let channel_info = CHANNEL_INFO.load(deps.storage, &msg.packet.dest.channel_id)?;

    let info = MessageInfo {
        sender: msg.relayer,
        funds: Default::default(),
    };

    match channel_info.protocol_version {
        ref v if v == Ics20Protocol::VERSION => Ics20Protocol {
            common: ProtocolCommon {
                deps,
                env,
                info,
                channel: channel_info,
            },
        }
        .send_timeout(msg.packet.data),
        ref v if v == Ucs01Protocol::VERSION => Ucs01Protocol {
            common: ProtocolCommon {
                deps,
                env,
                info,
                channel: channel_info,
            },
        }
        .send_timeout(msg.packet.data),
        v => Err(ContractError::UnknownProtocol {
            channel_id: msg.packet.dest.channel_id,
            protocol_version: v,
        }),
    }
}

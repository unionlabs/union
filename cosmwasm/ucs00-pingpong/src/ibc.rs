use crate::{msg::UCS00PingPong, ContractError};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Binary, DepsMut, Env, IbcBasicResponse, IbcChannel,
    IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcMsg, IbcOrder, IbcPacket,
    IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse, IbcTimeout,
    IbcTimeoutBlock, Reply, Response,
};

pub const PINGPONG_VERSION: &str = "ucs00-pingpong-1";
pub const PINGPONG_ORDERING: IbcOrder = IbcOrder::Unordered;

#[cw_serde]
pub enum Ack {
    Result(Binary),
    Error(String),
}

fn ack_success() -> Binary {
    let res = Ack::Result(b"1".into());
    to_binary(&res).unwrap()
}

fn ack_fail(err: String) -> Binary {
    let res = Ack::Error(err);
    to_binary(&res).unwrap()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _reply: Reply) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<(), ContractError> {
    enforce_order_and_version(msg.channel(), msg.counterparty_version())?;
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_connect(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> Result<IbcBasicResponse, ContractError> {
    enforce_order_and_version(msg.channel(), msg.counterparty_version())?;
    Ok(IbcBasicResponse::default())
}

fn enforce_order_and_version(
    channel: &IbcChannel,
    counterparty_version: Option<&str>,
) -> Result<(), ContractError> {
    if channel.version != PINGPONG_VERSION {
        return Err(ContractError::InvalidIbcVersion {
            version: channel.version.clone(),
        });
    }
    if let Some(version) = counterparty_version {
        if version != PINGPONG_VERSION {
            return Err(ContractError::InvalidIbcVersion {
                version: version.to_string(),
            });
        }
    }
    if channel.order != PINGPONG_ORDERING {
        return Err(ContractError::OnlyOrderedChannel {});
    }
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_close(
    _deps: DepsMut,
    _env: Env,
    _channel: IbcChannelCloseMsg,
) -> Result<IbcBasicResponse, ContractError> {
    // We don't allow it.
    unimplemented!();
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_receive(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, ContractError> {
    let packet = msg.packet;
    let ping_packet: UCS00PingPong = packet.data.try_into()?;
    do_ibc_packet_receive(deps, packet.dest.channel_id, ping_packet).or_else(|err| {
        Ok(IbcReceiveResponse::new()
            .set_ack(ack_fail(err.to_string()))
            .add_attributes(vec![
                attr("success", "false"),
                attr("error", err.to_string()),
            ]))
    })
}

fn do_ibc_packet_receive(
    _: DepsMut,
    dest_channel_id: String,
    ping_packet: UCS00PingPong,
) -> Result<IbcReceiveResponse, ContractError> {
    let pong_packet = ping_packet.clone().reverse();
    let ibc_packet = IbcMsg::SendPacket {
        channel_id: dest_channel_id,
        data: pong_packet.into(),
        timeout: IbcTimeout::with_block(IbcTimeoutBlock {
            revision: 0,
            height: u64::MAX,
        }),
    };
    let res = IbcReceiveResponse::new()
        .set_ack(ack_success())
        .add_message(ibc_packet)
        .add_attribute("action", if ping_packet.ping { "ping" } else { "pong" })
        .add_attribute("success", "true");
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ContractError> {
    let ack: Ack = from_binary(&msg.acknowledgement.data)?;
    match ack {
        Ack::Result(_) => on_packet_success(deps, msg.original_packet),
        Ack::Error(err) => on_packet_failure(deps, msg.original_packet, err),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_timeout(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketTimeoutMsg,
) -> Result<IbcBasicResponse, ContractError> {
    on_packet_failure(deps, msg.packet, "timeout".to_string())
}

fn on_packet_success(
    _deps: DepsMut,
    _packet: IbcPacket,
) -> Result<IbcBasicResponse, ContractError> {
    let attributes = vec![attr("action", "acknowledge"), attr("success", "true")];
    Ok(IbcBasicResponse::new().add_attributes(attributes))
}

fn on_packet_failure(
    _deps: DepsMut,
    _packet: IbcPacket,
    err: String,
) -> Result<IbcBasicResponse, ContractError> {
    let res = IbcBasicResponse::new()
        .add_attribute("action", "acknowledge")
        .add_attribute("success", "false")
        .add_attribute("error", err);
    Ok(res)
}

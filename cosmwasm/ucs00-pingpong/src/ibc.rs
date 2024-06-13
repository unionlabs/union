use cosmwasm_std::{
    attr, entry_point, DepsMut, Env, IbcBasicResponse, IbcChannel, IbcChannelCloseMsg,
    IbcChannelConnectMsg, IbcChannelOpenMsg, IbcOrder, IbcPacket, IbcPacketAckMsg,
    IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse, Reply, Response, StdError,
};

use crate::{msg::UCS00PingPong, state::CONFIG, ContractError};

pub const PROTOCOL_VERSION: &str = "ucs00-pingpong-1";
pub const PROTOCOL_ORDERING: IbcOrder = IbcOrder::Unordered;

fn ack_success() -> Vec<u8> {
    [1].into()
}

fn ack_fail() -> Vec<u8> {
    [0].into()
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
    if channel.version != PROTOCOL_VERSION {
        return Err(ContractError::InvalidIbcVersion {
            version: channel.version.clone(),
        });
    }
    if let Some(version) = counterparty_version {
        if version != PROTOCOL_VERSION {
            return Err(ContractError::InvalidIbcVersion {
                version: version.to_string(),
            });
        }
    }
    if channel.order != PROTOCOL_ORDERING {
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
    Err(StdError::generic_err("The game is infinite").into())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_receive(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, ContractError> {
    let packet = msg.packet;
    let ping_packet = UCS00PingPong::decode(packet.data)?;
    do_ibc_packet_receive(deps, env, packet.dest.channel_id, ping_packet).or_else(|err| {
        Ok(IbcReceiveResponse::new()
            .set_ack(ack_fail())
            .add_attributes(vec![
                attr("success", "false"),
                attr("error", err.to_string()),
            ]))
    })
}

fn do_ibc_packet_receive(
    deps: DepsMut,
    env: Env,
    dest_channel_id: String,
    packet: UCS00PingPong,
) -> Result<IbcReceiveResponse, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let ibc_packet = packet.reverse(&config, env.block.height, dest_channel_id);
    let res = IbcReceiveResponse::new()
        .set_ack(ack_success())
        .add_message(ibc_packet)
        .add_attribute("action", if packet.ping { "ping" } else { "pong" })
        .add_attribute("success", "true");
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ContractError> {
    let attributes = vec![attr("action", "acknowledge")];
    Ok(IbcBasicResponse::new().add_attributes(attributes))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_timeout(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketTimeoutMsg,
) -> Result<IbcBasicResponse, ContractError> {
    on_packet_failure(deps, msg.packet, "timeout".to_string())
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

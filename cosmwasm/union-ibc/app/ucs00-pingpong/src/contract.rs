use cosmwasm_schema::cw_serde;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{wasm_execute, DepsMut, Env, MessageInfo, Response, StdError};
use union_ibc_msg::{module::UnionIbcMsg, msg::MsgWriteAcknowledgement};

use crate::{
    msg::{ExecuteMsg, InitMsg, UCS00PingPong},
    state::CONFIG,
    ContractError,
};

pub const PROTOCOL_VERSION: &str = "ucs00-pingpong-1";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InitMsg,
) -> Result<Response, ContractError> {
    CONFIG.save(deps.storage, &msg.config)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Initiate { channel_id, packet } => {
            let config = CONFIG.load(deps.storage)?;
            let msg = packet.reverse(&config, env.block.time.nanos(), channel_id);

            Ok(Response::default().add_message(wasm_execute(config.ibc_host, &msg, vec![])?))
        }
        ExecuteMsg::UnionIbcMsg(UnionIbcMsg::OnChannelOpenInit { version, .. }) => {
            enforce_version(&version, None)?;
            Ok(Response::default())
        }
        ExecuteMsg::UnionIbcMsg(UnionIbcMsg::OnChannelOpenTry {
            version,
            counterparty_version,
            ..
        }) => {
            enforce_version(&version, Some(&counterparty_version))?;
            Ok(Response::default())
        }
        ExecuteMsg::UnionIbcMsg(UnionIbcMsg::OnRecvPacket { packet, .. }) => {
            let ping_packet = UCS00PingPong::decode(&packet.data)?;
            let config = CONFIG.load(deps.storage)?;
            let msg =
                ping_packet.reverse(&config, env.block.time.nanos(), packet.destination_channel);

            Ok(Response::default()
                .add_message(wasm_execute(
                    &config.ibc_host,
                    &union_ibc_msg::msg::ExecuteMsg::WriteAcknowledgement(
                        MsgWriteAcknowledgement {
                            channel_id: packet.destination_channel,
                            packet,
                            acknowledgement: ack_success().into(),
                        },
                    ),
                    vec![],
                )?)
                .add_message(wasm_execute(&config.ibc_host, &msg, vec![])?)
                .add_attribute("action", if ping_packet.ping { "ping" } else { "pong" })
                .add_attribute("success", "true"))
        }
        ExecuteMsg::UnionIbcMsg(UnionIbcMsg::OnTimeoutPacket { .. }) => Ok(Response::default()
            .add_attribute("action", "acknowledge")
            .add_attribute("success", "false")
            .add_attribute("error", "timeout")),
        ExecuteMsg::UnionIbcMsg(
            UnionIbcMsg::OnChannelCloseInit { .. } | UnionIbcMsg::OnChannelCloseConfirm { .. },
        ) => Err(StdError::generic_err("the show must go on").into()),
        _ => Ok(Response::default()),
    }
}

#[cw_serde]
pub struct MigrateMsg {
    seconds_before_timeout: u64,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    CONFIG.update(deps.storage, |mut c| {
        c.seconds_before_timeout = msg.seconds_before_timeout;
        Ok::<_, ContractError>(c)
    })?;

    Ok(Response::new())
}

fn enforce_version(version: &str, counterparty_version: Option<&str>) -> Result<(), ContractError> {
    if version != PROTOCOL_VERSION {
        return Err(ContractError::InvalidIbcVersion {
            version: version.to_string(),
        });
    }
    if let Some(version) = counterparty_version {
        if version != PROTOCOL_VERSION {
            return Err(ContractError::InvalidIbcVersion {
                version: version.to_string(),
            });
        }
    }
    Ok(())
}

fn ack_success() -> Vec<u8> {
    [1].into()
}

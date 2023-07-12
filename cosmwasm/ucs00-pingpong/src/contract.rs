#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{DepsMut, Env, IbcMsg, IbcTimeout, IbcTimeoutBlock, MessageInfo, Response};

use crate::{
    msg::{ExecuteMsg, InitMsg},
    ContractError,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InitMsg,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Initiate { channel_id, packet } => {
            let ibc_packet = IbcMsg::SendPacket {
                channel_id,
                data: packet.into(),
                timeout: IbcTimeout::with_block(IbcTimeoutBlock {
                    revision: 0,
                    height: u64::MAX,
                }),
            };

            Ok(Response::default().add_message(ibc_packet))
        }
    }
}

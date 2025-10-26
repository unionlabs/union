#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![cfg_attr(not(test), warn(clippy::unwrap_used))]

use cosmwasm_std::{DepsMut, Env, Event, MessageInfo, Response, WasmMsg, to_json_binary};
use frissitheto::UpgradeMsg;

use crate::{error::ContractError, msg::ExecuteMsg};

pub mod error;
pub mod msg;

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
#[expect(clippy::needless_pass_by_value, reason = "required for entry_point")]
pub fn execute(
    _: DepsMut,
    env: Env,
    _: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Upgrade { new_code_id, msg } => Ok(Response::new()
            .add_event(
                Event::new("upgrade")
                    .add_attribute("new_code_id", new_code_id.to_string())
                    .add_attribute("msg", msg.to_string()),
            )
            .add_message(WasmMsg::Migrate {
                contract_addr: env.contract.address.to_string(),
                new_code_id,
                msg: to_json_binary(&UpgradeMsg::<(), _>::Migrate(msg))?,
            })),
    }
}

#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![cfg_attr(not(test), warn(clippy::unwrap_used))]

use cosmwasm_std::{Env, Response, WasmMsg, to_json_binary};
use frissitheto::UpgradeMsg;

use crate::{error::ContractError, event::Upgrade, msg::Upgradable};

pub mod error;
pub mod event;
pub mod msg;

pub fn execute(env: &Env, msg: Upgradable) -> Result<Response, ContractError> {
    match msg {
        Upgradable::Upgrade { new_code_id, msg } => Ok(Response::new()
            .add_event(Upgrade {
                new_code_id,
                msg: &msg,
            })
            .add_message(WasmMsg::Migrate {
                contract_addr: env.contract.address.to_string(),
                new_code_id: new_code_id.get(),
                msg: to_json_binary(&UpgradeMsg::<(), _>::Migrate(msg))?,
            })),
    }
}

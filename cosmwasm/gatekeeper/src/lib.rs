#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![warn(clippy::pedantic)]
#![allow(
    clippy::used_underscore_items,
    clippy::missing_errors_doc,
    clippy::enum_glob_use,
    clippy::doc_markdown
)]
#![cfg_attr(not(test), warn(clippy::unwrap_used))]
#![cfg_attr(test, allow(clippy::too_many_lines))]

use access_manager::{context::ExecCtx, contract::only_authorized};
use access_manager_types::manager::msg::{InitMsg, MigrateMsg, QueryMsg};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, entry_point};
use frissitheto::UpgradeMsg;

use crate::{error::ContractError, msg::ExecuteMsg};

pub mod error;
pub mod msg;

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests;

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AccessManager(msg) => {
            access_manager::execute(deps, env, info, msg).map_err(Into::into)
        }
        ExecuteMsg::Upgradable(msg) => {
            let mut ctx = ExecCtx::new(deps, &env, &info, &msg);

            only_authorized(&mut ctx)?;

            upgradable::execute(&env, msg).map_err(Into::into)
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    access_manager::query(deps, env, msg).map_err(Into::into)
}

#[entry_point]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    access_manager::reply(deps, env, reply).map_err(Into::into)
}

#[entry_point]
pub fn migrate(
    deps: DepsMut,
    env: Env,
    msg: UpgradeMsg<InitMsg, MigrateMsg>,
) -> Result<Response, ContractError> {
    access_manager::migrate(deps, env, msg).map_err(Into::into)
}

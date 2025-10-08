//! CosmWasm implementation of [OpenZeppelin][oz]'s [`AccessManaged.sol`][am].
//!
//! This contract module makes available a [`Restricted<T>`] wrapper. This should wrap the
//! `ExecuteMsg` for the contract. All entrypoints will be permissioned according to an "authority":
//! a contract like `access_manager` that follows the [`ExecuteMsg`][me]/[`QueryMsg`][mq] interface,
//! implementing a policy that allows certain callers to access certain functions.
//!
//! NOTE: The [`Restricted<T>`] wrapper will apply access control to *all* methods. Methods that
//! should be public must be configured as such on the manager.
//!
//! [me]: access_manager_types::manager::msg::ExecuteMsg
//! [mq]: access_manager_types::manager::msg::QueryMsg
//! [oz]: https://www.openzeppelin.com
//! [am]: https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManaged.sol
//! [et]: https://serde.rs/enum-representations.html#externally-tagged
//! [selector]: https://docs.soliditylang.org/en/latest/abi-spec.html#function-selector

#![warn(clippy::pedantic)]
#![allow(
    clippy::used_underscore_items,
    clippy::missing_errors_doc,
    clippy::enum_glob_use,
    clippy::doc_markdown
)]
#![cfg_attr(not(test), warn(clippy::unwrap_used))]
#![cfg_attr(test, allow(clippy::too_many_lines))]

use access_manager_types::managed::msg::{ExecuteMsg, InitMsg, MigrateMsg, QueryMsg};
use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError, to_json_binary,
};
use depolama::StorageExt;
use frissitheto::UpgradeMsg;

use crate::{
    contract::{authority, is_consuming_scheduled_op, set_authority},
    error::ContractError,
    state::{Authority, ConsumingSchedule},
};

pub mod contract;
pub mod error;
mod restricted;
pub mod state;

pub use restricted::{
    ACCESS_MANAGED_CONSUME_SCHEDULED_OP_REPLY_ID, EnsureCanCallResult, Restricted,
};

/// Initializes the contract connected to an initial authority.
///
/// ```solidity
/// constructor(address initialAdmin)
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManaged.sol#L27>
#[expect(clippy::needless_pass_by_value, reason = "required for entry_point")]
pub fn init(deps: DepsMut, msg: InitMsg) -> Result<Response, ContractError> {
    let InitMsg { initial_authority } = msg;

    deps.storage.write_item::<Authority>(&initial_authority);
    deps.storage.write_item::<ConsumingSchedule>(&false);

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    _: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetAuthority { new_authority } => {
            let event = set_authority(deps, info, new_authority)?;

            Ok(Response::new().add_event(event))
        }
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
#[expect(clippy::needless_pass_by_value, reason = "required for entry_point")]
pub fn query(deps: Deps, _: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Authority {} => Ok(to_json_binary(&authority(deps)?)?),
        QueryMsg::IsConsumingScheduledOp {} => {
            Ok(to_json_binary(&is_consuming_scheduled_op(deps)?)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn reply(deps: DepsMut, _: Env, reply: Reply) -> Result<Response, ContractError> {
    if let Some(reply) = handle_consume_scheduled_op_reply(deps, reply)? {
        Err(StdError::generic_err(format!("unknown reply: {reply:?}")).into())
    } else {
        Ok(Response::new())
    }
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "DepsMut should be passed by value"
)]
pub fn handle_consume_scheduled_op_reply(
    deps: DepsMut<'_>,
    reply: Reply,
) -> Result<Option<Reply>, ContractError> {
    match reply {
        Reply {
            id: ACCESS_MANAGED_CONSUME_SCHEDULED_OP_REPLY_ID,
            result,
            ..
        } => {
            result.unwrap();

            deps.storage.write_item::<ConsumingSchedule>(&false);

            Ok(None)
        }
        _ => Ok(Some(reply)),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(
    deps: DepsMut,
    _: Env,
    msg: UpgradeMsg<InitMsg, MigrateMsg>,
) -> Result<Response, ContractError> {
    msg.run(
        deps,
        |deps, msg| {
            let res = init(deps, msg)?;
            Ok((res, None))
        },
        |_, _, _| Ok((Response::default(), None)),
    )
}

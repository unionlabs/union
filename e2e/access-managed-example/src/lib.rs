use std::num::NonZero;

use access_managed::{
    EnsureCanCallResult, Restricted, handle_consume_scheduled_op_reply, state::Authority,
};
use access_manager_types::{
    managed::msg::{InitMsg, MigrateMsg},
    manager,
};
use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, Event, MessageInfo, Reply, Response, StdError, SubMsg, entry_point,
    to_json_binary, wasm_execute,
};
use depolama::StorageExt;
use frissitheto::{InitStateVersionError, UpgradeError, UpgradeMsg};

use crate::{
    msg::{ExecuteMsg, QueryMsg},
    state::{Counter, Executing, IncrementInReplyValue},
};

pub mod msg;
pub mod state;

pub const INCREMENT_IN_REPLY_REPLY_ID: u64 = 1;

pub fn init(deps: DepsMut, msg: InitMsg) -> Result<Response, ContractError> {
    deps.storage.write_item::<Counter>(&0);

    access_managed::init(deps, msg)?;

    Ok(Response::new())
}

#[entry_point]
pub fn instantiate(
    mut deps: DepsMut,
    _: Env,
    _: MessageInfo,
    msg: InitMsg,
) -> Result<Response, ContractError> {
    frissitheto::init_state_version(&mut deps, const { <NonZero<u32>>::new(1).unwrap() })?;

    init(deps, msg)
}

#[entry_point]
pub fn execute(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Restricted<ExecuteMsg>,
) -> Result<Response, ContractError> {
    let msg = match msg.ensure_can_call::<Authority>(deps.branch(), &env, &info)? {
        EnsureCanCallResult::Msg(msg) => msg,
        EnsureCanCallResult::Scheduled(sub_msgs) => {
            return Ok(Response::new().add_submessages(sub_msgs));
        }
    };

    match msg {
        ExecuteMsg::Increment { by } => {
            let new_value = deps
                .storage
                .update_item::<Counter, ContractError, _>(|counter| {
                    *counter = counter.saturating_add(by);
                    Ok(*counter)
                })?;

            Ok(Response::new()
                .add_event(Event::new("increment").add_attribute("by", by.to_string()))
                .add_event(Event::new("counter").add_attribute("value", new_value.to_string())))
        }
        ExecuteMsg::IncrementInReply { by } => {
            deps.storage.write_item::<IncrementInReplyValue>(&by);

            Ok(Response::new().add_submessage(SubMsg::reply_on_success(
                wasm_execute(env.contract.address, &ExecuteMsg::Noop {}, vec![])?,
                INCREMENT_IN_REPLY_REPLY_ID,
            )))
        }
        ExecuteMsg::Decrement { by, in_sub_msg } => {
            if in_sub_msg {
                deps.storage.write_item::<Executing>(&());
                Ok(Response::new().add_message(wasm_execute(
                    env.contract.address,
                    &ExecuteMsg::DecrementInSubMsg { by },
                    vec![],
                )?))
            } else {
                let new_value =
                    deps.storage
                        .update_item::<Counter, ContractError, _>(|counter| {
                            *counter = counter.saturating_sub(by);
                            Ok(*counter)
                        })?;

                Ok(Response::new()
                    .add_event(Event::new("decrement").add_attribute("by", by.to_string()))
                    .add_event(Event::new("counter").add_attribute("value", new_value.to_string())))
            }
        }
        ExecuteMsg::DecrementInSubMsg { by } => {
            if deps.storage.take_item::<Executing>()?.is_none() {
                return Err(ContractError::NotExecuting);
            }

            let new_value = deps
                .storage
                .update_item::<Counter, ContractError, _>(|counter| {
                    *counter = counter.saturating_sub(by);
                    Ok(*counter)
                })?;

            Ok(Response::new()
                .add_event(Event::new("decrement_in_sub_msg").add_attribute("by", by.to_string()))
                .add_event(Event::new("counter").add_attribute("value", new_value.to_string())))
        }
        ExecuteMsg::Noop {} => Ok(Response::new().add_event(Event::new("noop"))),
        ExecuteMsg::DelegateExecute { target, data } => {
            Ok(Response::new().add_message(wasm_execute(
                deps.storage.read_item::<Authority>()?,
                &manager::msg::ExecuteMsg::Execute { target, data },
                vec![],
            )?))
        }
        ExecuteMsg::DelegateSchedule { target, data, when } => {
            Ok(Response::new().add_message(wasm_execute(
                deps.storage.read_item::<Authority>()?,
                &manager::msg::ExecuteMsg::Schedule { target, data, when },
                vec![],
            )?))
        }
        ExecuteMsg::AccessManaged(msg) => {
            access_managed::execute(deps, env, info, msg).map_err(Into::into)
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::CurrentValue {} => Ok(to_json_binary(&deps.storage.read_item::<Counter>()?)?),
        QueryMsg::AccessManaged(msg) => access_managed::query(deps, env, msg).map_err(Into::into),
    }
}

#[entry_point]
pub fn reply(mut deps: DepsMut, _: Env, reply: Reply) -> Result<Response, ContractError> {
    if let Some(reply) = handle_consume_scheduled_op_reply(deps.branch(), reply)? {
        match reply.id {
            INCREMENT_IN_REPLY_REPLY_ID => {
                let by = deps.storage.take_item::<IncrementInReplyValue>()?.unwrap();

                let new_value =
                    deps.storage
                        .update_item::<Counter, ContractError, _>(|counter| {
                            *counter = counter.saturating_add(by);
                            Ok(*counter)
                        })?;

                Ok(Response::new()
                    .add_event(Event::new("increment_in_reply").add_attribute("by", by.to_string()))
                    .add_event(Event::new("counter").add_attribute("value", new_value.to_string())))
            }
            _ => Err(StdError::generic_err(format!("unknown reply: {reply:?}")).into()),
        }
    } else {
        Ok(Response::new())
    }
}

#[entry_point]
pub fn migrate(
    deps: DepsMut,
    _: Env,
    msg: UpgradeMsg<InitMsg, MigrateMsg>,
) -> Result<Response, ContractError> {
    msg.run(
        deps,
        |deps, init_msg| {
            let res = init(deps, init_msg)?;
            Ok((res, None))
        },
        |_, _, _| Ok((Response::default(), None)),
    )
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Migrate(#[from] UpgradeError),

    #[error(transparent)]
    InitStateVersion(#[from] InitStateVersionError),

    #[error(transparent)]
    AccessManaged(#[from] access_managed::error::ContractError),

    #[error("not executing")]
    NotExecuting,
}

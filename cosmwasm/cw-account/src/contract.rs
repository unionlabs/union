use cosmwasm_schema::cw_serde;
use cosmwasm_std::{entry_point, DepsMut, Env, MessageInfo, Response, StdResult};
use frissitheto::UpgradeMsg;

use crate::{
    error::Error,
    msg::{ExecuteMsg, InstantiateMsg},
    state::{Config, CONFIG},
};

fn init(deps: DepsMut, _: Env, msg: InstantiateMsg) -> Result<Response, Error> {
    CONFIG.save(deps.storage, &Config { owner: msg.owner })?;
    Ok(Response::default())
}

#[entry_point]
pub fn instantiate(_: DepsMut, _: Env, _: MessageInfo, _: ()) -> StdResult<Response> {
    panic!("this contract cannot be instantiated directly, but must be migrated from an existing instantiated contract.");
}

#[cw_serde]
pub struct MigrateMsg {}

#[entry_point]
pub fn migrate(
    deps: DepsMut,
    env: Env,
    msg: UpgradeMsg<InstantiateMsg, MigrateMsg>,
) -> Result<Response, Error> {
    msg.run(
        deps,
        |deps, init_msg| {
            let res = init(deps, env, init_msg)?;
            Ok((res, None))
        },
        |_, _, _| Ok((Response::default(), None)),
    )
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, Error> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(Error::OnlyOwner);
    }
    Ok(Response::new().add_messages(msg.messages))
}

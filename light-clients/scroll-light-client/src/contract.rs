use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use union_ibc_light_client::state::IBC_HOST;

use crate::{
    client::ScrollLightClient,
    errors::Error,
    msg::{InstantiateMsg, QueryMsg},
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, Error> {
    IBC_HOST.save(deps.storage, &msg.ibc_host)?;
    Ok(Response::default())
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    union_ibc_light_client::query::<ScrollLightClient>(deps, env, msg).map_err(Into::into)
}

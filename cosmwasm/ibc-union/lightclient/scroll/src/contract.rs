use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, entry_point};
use ibc_union_light_client::{
    IbcClientError,
    msg::{InstantiateMsg, QueryMsg},
};

use crate::client::ScrollLightClient;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, IbcClientError<ScrollLightClient>> {
    ibc_union_light_client::instantiate(deps, env, info, msg)
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    ibc_union_light_client::query::<ScrollLightClient>(deps, env, msg).map_err(Into::into)
}

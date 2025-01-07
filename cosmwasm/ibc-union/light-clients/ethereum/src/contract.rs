#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use ibc_union_light_client::{
    msg::{InstantiateMsg, QueryMsg},
    IbcClientError,
};
use serde::{Deserialize, Serialize};

use crate::client::EthereumLightClient;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, IbcClientError<EthereumLightClient>> {
    ibc_union_light_client::instantiate(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    ibc_union_light_client::query::<EthereumLightClient>(deps, env, msg).map_err(Into::into)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrateMsg {}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    _deps: DepsMut,
    _env: Env,
    _msg: MigrateMsg,
) -> Result<Response, IbcClientError<EthereumLightClient>> {
    Ok(Response::new())
}

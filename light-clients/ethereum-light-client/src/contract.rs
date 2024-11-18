use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use serde::{Deserialize, Serialize};
use unionlabs::cosmwasm::wasm::union::custom_query::UnionCustomQuery;

use crate::{
    client::EthereumLightClient,
    errors::Error,
    msg::{InstantiateMsg, QueryMsg},
    state::IBC_HOST,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut<UnionCustomQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, Error> {
    IBC_HOST.save(deps.storage, &msg.ibc_host)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<UnionCustomQuery>, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    union_ibc_light_client::query::<EthereumLightClient>(deps, env, msg).map_err(Into::into)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrateMsg {}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, Error> {
    Ok(Response::new())
}

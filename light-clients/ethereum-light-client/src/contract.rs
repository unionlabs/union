use cosmwasm_std::{
    entry_point, to_binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response,
};
use ics008_wasm_client::{ExecuteMsg, IBCClient, QueryMsg};

use crate::{client::EthereumLightClient, custom_query::CustomQuery, errors::Error};

pub struct InstantiateMsg {}

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, Error> {
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut<CustomQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, Error> {
    let result = EthereumLightClient::execute(deps, env, info, msg)?;
    Ok(Response::default().set_data(result.encode()?))
}

#[entry_point]
pub fn query(deps: Deps<CustomQuery>, env: Env, msg: QueryMsg) -> Result<QueryResponse, Error> {
    let response = EthereumLightClient::query(deps, env, msg)?;

    to_binary(&response).map_err(Into::into)
}

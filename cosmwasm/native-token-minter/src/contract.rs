use cosmwasm_schema::cw_serde;
use cosmwasm_std::{DepsMut, Env, MessageInfo, StdResult};
use token_factory_api::TokenFactoryMsg;

#[cw_serde]
pub struct InitMsg {}

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InitMsg,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[cw_serde]
pub struct MigrateMsg {}

#[entry_point]
pub fn migrate(_: DepsMut, _: Env, _: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::new())
}

#[entry_point]
pub fn execute(
    _deps: DepsMut<TokenFactoryMsg>,
    _env: Env,
    _info: MessageInfo,
    msg: TokenFactoryMsg,
) -> StdResult<Response<TokenFactoryMsg>> {
    Ok(Response::default().add_message(CosmosMsg::Custom(msg)))
}

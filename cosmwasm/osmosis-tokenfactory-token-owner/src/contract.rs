use cosmwasm_std::{DepsMut, Empty, Env, MessageInfo, Response, StdResult, entry_point};
use osmosis_tokenfactory_token_minter::msg::TokenFactoryAdminOperation;
use token_factory_api::TokenFactoryMsg;

#[entry_point]
pub fn instantiate(_: DepsMut, _: Env, _: MessageInfo, _: Empty) -> StdResult<Response> {
    Ok(Response::new())
}

#[entry_point]
pub fn execute(
    _: DepsMut,
    _: Env,
    _: MessageInfo,
    msg: TokenFactoryAdminOperation,
) -> StdResult<Response<TokenFactoryMsg>> {
    Ok(Response::new().add_message(msg.into_cosmos_msg()))
}

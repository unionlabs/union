use cosmwasm_schema::{cw_serde, QueryResponses};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};

use crate::ContractError;

#[cw_serde]
struct InstantiateMsg {}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Balances {
            address: _,
            tokens: _,
        } => to_json_binary::<Vec<Option<Uint128>>>(&vec![]), // TODO: insert responses
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(BalancesResponse)]
    Balances {
        address: String,
        tokens: Vec<String>,
    },
}

#[cw_serde]
pub struct BalancesResponse {
    pub balances: Vec<Option<Uint128>>,
}

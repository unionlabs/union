use cosmwasm_schema::{cw_serde, QueryResponses};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, QueryRequest, Response,
    StdResult, Uint128, WasmQuery,
};
use cw20::Cw20QueryMsg;

use crate::ContractError;

#[cw_serde]
pub struct InstantiateMsg {}

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
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Balances { address, tokens } => {
            let balances = tokens
                .iter()
                .map(|token| query_cw20_balance(deps, token, &address).ok())
                .collect();
            to_json_binary(&BalancesResponse { balances })
        }
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
    pub balances: Vec<Option<Uint128>>, // None if contract is invalid or no balance exists
}

#[cw_serde]
pub struct BalanceResponse {
    pub balance: Uint128,
}

fn query_cw20_balance(deps: Deps, cw20_contract: &str, address: &str) -> StdResult<Uint128> {
    let contract_addr = Addr::unchecked(cw20_contract);
    let query_msg = cosmwasm_std::to_json_binary(&Cw20QueryMsg::Balance {
        address: address.to_string(),
    })?;

    let res: BalanceResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: contract_addr.to_string(),
        msg: query_msg,
    }))?;

    Ok(res.balance)
}

use crate::errors::Error;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_std::{
    entry_point, to_binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response, StdError,
    StdResult,
};
use wasm_lc_types::msg::{Status, StatusResponse};

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
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, Error> {
    match msg {
        ExecuteMsg::VerifyMembership { .. } => {
            todo!()
        }
        _ => Err(StdError::not_found("Not implemented").into()),
    }
}

#[entry_point]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Status {} => to_binary(&query_status()),
    }
}

fn query_status() -> StatusResponse {
    StatusResponse {
        status: Status::Active.to_string(),
        genesis_metadata: vec![],
    }
}

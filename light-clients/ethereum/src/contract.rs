use crate::errors::Error;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::extract_client_state_from_wasm;
use cosmwasm_std::{
    entry_point, to_binary, to_vec, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response,
    StdError, StdResult,
};
use wasm_lc_types::msg::StatusResponse;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, Error> {
    let client_state = extract_client_state_from_wasm(deps.as_ref())?;
    Ok(Response::new().add_attribute(
        "Client state",
        String::from_utf8_lossy(&to_vec(&client_state).unwrap()),
    ))
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
        status: "Active".into(),
        genesis_metadata: vec![],
    }
}

use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response,
    StdError,
};
use unionlabs::ibc::{core::client::height::Height, lightclients::cometbls::header::Header};
use wasm_light_client_types::msg::{
    ClientMessage, ContractResult, MerklePath, Status, StatusResponse,
};

use crate::{
    errors::Error,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{read_client_state, read_consensus_state},
};

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
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, Error> {
    let result = match msg {
        ExecuteMsg::VerifyMembership {
            height,
            delay_time_period,
            delay_block_period,
            proof,
            path,
            value,
        } => verify_membership(
            deps.as_ref(),
            height,
            delay_time_period,
            delay_block_period,
            proof,
            path,
            value,
        ),
        ExecuteMsg::UpdateState {
            client_message: ClientMessage { header, .. },
        } => {
            if let Some(header) = header {
                // TODO(aeryz): How we want to pass the data? Which format?
                let header = todo!();
                // let header = Header::try_from_proto_bytes(&header.data).map_err(|err| {
                //     Error::decode(format!(
                //         "when converting proto header to header in update: {err:#?}"
                //     ))
                // })?;
                update_header(deps, header)
            } else {
                Err(StdError::not_found("Not implemented").into())
            }
        }
        _ => Ok(ContractResult::valid(None)),
    }?;

    Ok(Response::default().set_data(result.encode()?))
}

/// Verifies if the `value` is committed at `path` in the counterparty light client.
pub fn verify_membership(
    _deps: Deps,
    _height: Height,
    _delay_time_period: u64,
    _delay_block_period: u64,
    _proof: Binary,
    _path: MerklePath,
    _value: Binary,
) -> Result<ContractResult, Error> {
    unimplemented!()
}

pub fn update_header(mut deps: DepsMut, header: Header) -> Result<ContractResult, Error> {
    unimplemented!()
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<QueryResponse, Error> {
    let response = match msg {
        QueryMsg::Status {} => query_status(deps, &env)?,
    };

    to_binary(&response).map_err(Into::into)
}

fn query_status(deps: Deps, env: &Env) -> Result<StatusResponse, Error> {
    let client_state = read_client_state(deps)?;

    // TODO(aeryz): make client state optional
    if client_state.data.frozen_height.revision_height == 0 {
        return Ok(Status::Frozen.into());
    }

    let Some(consensus_state) = read_consensus_state(deps, &client_state.latest_height)? else {
        return Ok(Status::Expired.into());
    };

    if is_client_expired(
        consensus_state.timestamp,
        client_state.data.trusting_period.seconds as u64,
        env.block.time.seconds(),
    ) {
        return Ok(Status::Expired.into());
    }

    Ok(Status::Active.into())
}

fn is_client_expired(
    consensus_state_timestamp: u64,
    trusting_period: u64,
    current_block_time: u64,
) -> bool {
    consensus_state_timestamp + trusting_period < current_block_time
}

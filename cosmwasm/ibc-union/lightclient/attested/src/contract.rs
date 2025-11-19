use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, entry_point, to_json_binary,
};
use ibc_union_light_client::{access_managed::state::Authority, default_migrate, default_reply};
use serde::{Deserialize, Serialize};

use crate::{
    client::AttestedLightClient,
    contract::{
        execute::{add_attestor, attest, confirm_attestation, remove_attestor, set_quorum},
        query::{attested_value, attestors, latest_height, quorum, timestamp_at_height},
    },
    errors::Error,
    msg::{ExecuteMsg, QueryMsg, RestrictedExecuteMsg},
};

default_reply!();
defualt_migrate!();

pub mod execute;
pub mod query;

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, Error> {
    match msg {
        ExecuteMsg::Attest {
            attestation,
            attestor,
            signature,
        } => attest(deps, attestation, attestor, signature),
        ExecuteMsg::ConfirmAttestation { attestation } => confirm_attestation(deps, attestation),
        ExecuteMsg::Restricted(msg) => {
            let msg = match msg.ensure_can_call::<Authority>(deps.branch(), &env, &info)? {
                EnsureCanCallResult::Msg(msg) => msg,
                EnsureCanCallResult::Scheduled(sub_msgs) => {
                    return Ok(Response::new().add_submessages(sub_msgs));
                }
            };

            match msg {
                RestrictedExecuteMsg::SetQuorum {
                    chain_id,
                    new_quorum,
                } => set_quorum(deps, chain_id, new_quorum),
                RestrictedExecuteMsg::AddAttestor {
                    chain_id,
                    new_attestor,
                } => add_attestor(deps, chain_id, new_attestor),
                RestrictedExecuteMsg::RemoveAttestor {
                    chain_id,
                    old_attestor,
                } => remove_attestor(deps, chain_id, old_attestor),
            }
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, Error> {
    match msg {
        QueryMsg::Quorum { chain_id } => Ok(to_json_binary(&quorum(deps, chain_id)?)?),
        QueryMsg::Attestors { chain_id } => Ok(to_json_binary(&attestors(deps, chain_id)?)?),
        QueryMsg::AttestedValue {
            chain_id,
            height,
            key,
        } => Ok(to_json_binary(&attested_value(
            deps, chain_id, height, key,
        )?)?),
        QueryMsg::TimestampAtHeight { chain_id, height } => Ok(to_json_binary(
            &timestamp_at_height(deps, chain_id, height)?,
        )?),
        QueryMsg::LatestHeight { chain_id } => Ok(to_json_binary(&latest_height(deps, chain_id)?)?),
        QueryMsg::LightClient(msg) => {
            ibc_union_light_client::query::<AttestedLightClient>(deps, env, msg)
                .map_err(StdError::from)
                .map_err(Into::into)
        }
    }
}

use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, entry_point, to_json_binary,
};
use frissitheto::UpgradeMsg;
use ibc_union_light_client::{IbcClientError, msg::InitMsg};
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

pub mod execute;
pub mod query;

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, Error> {
    match msg {
        ExecuteMsg::Attest {
            attestation,
            attestor,
            signature,
        } => attest(deps, attestation, attestor, signature),
        ExecuteMsg::ConfirmAttestation { attestation } => confirm_attestation(deps, attestation),
        ExecuteMsg::Restricted(msg) => match msg {
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
        },
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrateMsg {}

#[entry_point]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    msg: UpgradeMsg<InitMsg, MigrateMsg>,
) -> Result<Response, IbcClientError<AttestedLightClient>> {
    msg.run(
        deps,
        |deps, init_msg| {
            let res = ibc_union_light_client::init(deps, init_msg)?;

            Ok((res, None))
        },
        |_deps, _migrate_msg, _current_version| Ok((Response::default(), None)),
    )
}

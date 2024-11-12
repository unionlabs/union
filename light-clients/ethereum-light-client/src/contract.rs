use beacon_api_types::{Mainnet, Minimal, PresetBaseKind};
use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use ethereum_light_client_types::{
    ClientState, ConsensusState, Header, Misbehaviour, StorageProof,
};
use union_ibc::{
    lightclient::query::{Status, VerifyClientMessageUpdate},
    state::{CLIENT_CONSENSUS_STATES, CLIENT_STATES},
};
use unionlabs::{
    cosmwasm::wasm::union::custom_query::UnionCustomQuery,
    encoding::{DecodeAs, EncodeAs, Proto},
    hash::H256,
};

use crate::{
    client::{verify_header, verify_membership, verify_misbehaviour, verify_non_membership},
    errors::Error,
    msg::{InstantiateMsg, QueryMsg},
    state::IBC_HOST,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut<UnionCustomQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, Error> {
    IBC_HOST.save(deps.storage, &msg.ibc_host)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<UnionCustomQuery>, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VerifyMembership {
            client_id,
            height,
            proof,
            path,
            value,
        } => {
            let ibc_host = IBC_HOST.load(deps.storage)?;
            let consensus_state = read_consensus_state(deps, &ibc_host, client_id, height)?;
            // This storage root is verified during the header update, so we don't need to verify it again.
            let storage_root = consensus_state.storage_root;

            let storage_proof =
                StorageProof::decode_as::<Proto>(&proof).map_err(Error::StorageProofDecode)?;

            verify_membership(
                H256::try_from(&path.to_vec())
                    .map_err(|_| Error::InvalidCommitmentKeyLength(path.to_vec()))?,
                storage_root,
                storage_proof,
                H256::try_from(&value.to_vec())
                    .map_err(|_| Error::InvalidCommitmentValueLength(path.to_vec()))?,
            )?;

            to_json_binary(&Binary::from(vec![]))
        }
        QueryMsg::VerifyNonMembership {
            client_id,
            height,
            proof,
            path,
        } => {
            let ibc_host = IBC_HOST.load(deps.storage)?;
            let consensus_state = read_consensus_state(deps, &ibc_host, client_id, height)?;
            // This storage root is verified during the header update, so we don't need to verify it again.
            let storage_root = consensus_state.storage_root;

            let storage_proof =
                StorageProof::decode_as::<Proto>(&proof).map_err(Error::StorageProofDecode)?;

            verify_non_membership(
                H256::try_from(&path.to_vec())
                    .map_err(|_| Error::InvalidCommitmentKeyLength(path.to_vec()))?,
                storage_root,
                storage_proof,
            )?;

            to_json_binary(&Binary::from(vec![]))
        }
        QueryMsg::VerifyClientMessage { client_id, message } => {
            let header = Header::decode_as::<Proto>(&message).unwrap();
            let ibc_host = IBC_HOST.load(deps.storage)?;

            let client_state = read_client_state(deps, &ibc_host, client_id)?;
            let consensus_state =
                read_consensus_state(deps, &ibc_host, client_id, header.trusted_height.height())?;

            let (height, client_state, consensus_state) =
                if client_state.chain_spec == PresetBaseKind::Minimal {
                    verify_header::<Minimal>(client_state, consensus_state, deps, env, header)?
                } else {
                    verify_header::<Mainnet>(client_state, consensus_state, deps, env, header)?
                };

            to_json_binary(&VerifyClientMessageUpdate {
                height,
                consensus_state: consensus_state.encode_as::<Proto>().into(),
                client_state: client_state.encode_as::<Proto>().into(),
            })
        }
        QueryMsg::GetLatestHeight { client_id } => {
            let ibc_host = IBC_HOST.load(deps.storage)?;
            let client_state = read_client_state(deps, &ibc_host, client_id)?;
            to_json_binary(&client_state.latest_slot)
        }
        QueryMsg::GetTimestamp { client_id, height } => {
            let ibc_host = IBC_HOST.load(deps.storage)?;
            let consensus_state = read_consensus_state(deps, &ibc_host, client_id, height)?;
            to_json_binary(&consensus_state.timestamp)
        }
        QueryMsg::GetStatus { client_id } => {
            let ibc_host = IBC_HOST.load(deps.storage)?;
            let client_state = read_client_state(deps, &ibc_host, client_id)?;
            let status = if client_state.frozen_height.height() == 0 {
                Status::Active
            } else {
                Status::Frozen
            };
            to_json_binary(&status)
        }
        QueryMsg::VerifyCreation {
            client_state,
            consensus_state,
            ..
        } => {
            let client_state = ClientState::decode_as::<Proto>(&client_state).unwrap();
            // if we are able to parse it then it's fine
            let _ = ConsensusState::decode_as::<Proto>(&consensus_state).unwrap();
            to_json_binary(&client_state.latest_slot)
        }
        QueryMsg::CheckForMisbehavior { client_id, message } => {
            let misbehavior = Misbehaviour::decode_as::<Proto>(&message).unwrap();

            let ibc_host = IBC_HOST.load(deps.storage)?;
            let client_state = read_client_state(deps, &ibc_host, client_id)?;
            let consensus_state = read_consensus_state(
                deps,
                &ibc_host,
                client_id,
                misbehavior.trusted_height.height(),
            )?;

            if client_state.chain_spec == PresetBaseKind::Minimal {
                verify_misbehaviour::<Minimal>(
                    client_state,
                    consensus_state,
                    deps,
                    env,
                    misbehavior,
                )?
            } else {
                verify_misbehaviour::<Mainnet>(
                    client_state,
                    consensus_state,
                    deps,
                    env,
                    misbehavior,
                )?
            };

            to_json_binary(&true)
        }
    }
}

fn read_client_state(
    deps: Deps<UnionCustomQuery>,
    ibc_host: &Addr,
    client_id: u32,
) -> Result<ClientState, Error> {
    let client_state = deps
        .querier
        .query_wasm_raw(ibc_host.to_string(), CLIENT_STATES.key(client_id).to_vec())?
        .unwrap();
    Ok(ClientState::decode_as::<Proto>(&client_state).unwrap())
}

fn read_consensus_state(
    deps: Deps<UnionCustomQuery>,
    ibc_host: &Addr,
    client_id: u32,
    height: u64,
) -> Result<ConsensusState, Error> {
    let consensus_state = deps
        .querier
        .query_wasm_raw(
            ibc_host.to_string(),
            CLIENT_CONSENSUS_STATES.key((client_id, height)).to_vec(),
        )?
        .unwrap();
    Ok(ConsensusState::decode_as::<Proto>(&consensus_state).unwrap())
}

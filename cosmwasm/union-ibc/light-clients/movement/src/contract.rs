use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use movement_light_client_types::{ClientState, ConsensusState, Header};
use protos::ibc::lightclients::wasm::v1::{
    ClientState as ProtoClientState, ConsensusState as ProtoConsensusState,
};
use union_ibc::{
    lightclient::query::VerifyClientMessageUpdate,
    state::{CLIENT_CONSENSUS_STATES, CLIENT_STATES},
};
use unionlabs::{
    aptos::storage_proof::StorageProof,
    cosmwasm::wasm::union::custom_query::UnionCustomQuery,
    encoding::{DecodeAs, EncodeAs, Proto},
    ibc::core::client::height::Height,
};

use crate::{
    client::{verify_header, verify_membership},
    error::Error,
    msg::{InstantiateMsg, QueryMsg},
    state::IBC_HOST,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut<UnionCustomQuery>,
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
            let client_state = read_client_state(deps, &ibc_host, client_id)?;
            let consensus_state = read_consensus_state(deps, &ibc_host, client_id, height)?;
            // This storage root is verified during the header update, so we don't need to verify it again.
            let state_root = consensus_state.state_root;

            let storage_proof =
                StorageProof::decode_as::<Proto>(&proof).map_err(Error::StorageProofDecode)?;

            verify_membership(
                &path,
                state_root,
                client_state.table_handle,
                storage_proof,
                &value,
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
                verify_header(client_state, consensus_state, deps, env, header)?;

            to_json_binary(&VerifyClientMessageUpdate {
                height,
                consensus_state: consensus_state.encode_as::<Proto>().into(),
                client_state: client_state.encode_as::<Proto>().into(),
            })
        }
        _ => todo!(),
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

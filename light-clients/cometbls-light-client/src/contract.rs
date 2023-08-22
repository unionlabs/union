use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response,
    StdError,
};
use prost::Message;
use protos::tendermint::types::{CanonicalBlockId, CanonicalPartSetHeader, SignedMsgType};
use unionlabs::{
    ibc::{
        core::{client::height::Height, commitment::merkle_root::MerkleRoot},
        google::protobuf::{duration::Duration, timestamp::Timestamp},
        lightclients::cometbls::header::Header,
    },
    tendermint::types::commit::Commit,
    TryFromProto,
};
use wasm_light_client_types::msg::{
    ClientMessage, ContractResult, MerklePath, Status, StatusResponse,
};

use crate::{
    errors::Error,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{
        read_client_state, read_consensus_state, save_consensus_state, save_wasm_client_state,
    },
    zkp_verifier::verify_zkp,
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
    env: Env,
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
                let header = Header::try_from_proto_bytes(&header.data).map_err(|err| {
                    Error::decode(format!(
                        "when converting proto header to header in update: {err:#?}"
                    ))
                })?;
                update_header(deps, env, header)
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
    // TODO: #514
    Ok(ContractResult::valid(None))
}

pub fn update_header(mut deps: DepsMut, env: Env, header: Header) -> Result<ContractResult, Error> {
    let mut client_state = read_client_state(deps.as_ref())?;
    let mut consensus_state = read_consensus_state(deps.as_ref(), &header.trusted_height)?.ok_or(
        Error::ConsensusStateNotFound(
            header.trusted_height.revision_number,
            header.trusted_height.revision_height,
        ),
    )?;

    let untrusted_height_number = header.signed_header.commit.height.inner() as u64;
    let trusted_height_number = header.trusted_height.revision_number;

    if untrusted_height_number <= trusted_height_number {
        return Err(Error::InvalidHeader(
            "header height <= consensus state height".into(),
        ));
    }

    let trusted_timestamp = consensus_state.timestamp;
    let untrusted_timestamp = header.signed_header.header.time.seconds;

    if untrusted_timestamp.inner() as u64 <= trusted_timestamp {
        return Err(Error::InvalidHeader(
            "header time <= consensus state time".into(),
        ));
    }

    let current_time: Timestamp = env
        .block
        .time
        .try_into()
        .map_err(|_| Error::InvalidHeader("timestamp conversion failed".into()))?;

    if Duration::from(header.signed_header.header.time)
        < Duration::from(current_time)
            .checked_add(client_state.data.trusting_period)
            .ok_or(Error::DurationAdditionOverflow)?
    {
        return Err(Error::InvalidHeader("header expired".into()));
    }

    let max_clock_drift =
        current_time.seconds.inner() + client_state.data.max_clock_drift.seconds.inner();

    if untrusted_timestamp.inner() >= max_clock_drift {
        return Err(Error::InvalidHeader("header back to the future".into()));
    }

    let trusted_validators_hash = consensus_state.data.next_validators_hash.clone();

    let untrusted_validators_hash = if untrusted_height_number == trusted_height_number + 1 {
        trusted_validators_hash.clone()
    } else {
        header.untrusted_validator_set_root
    };

    let expected_block_hash = header
        .signed_header
        .header
        .calculate_merkle_root()
        .ok_or(Error::UnableToCalculateMerkleRoot)?;

    if header.signed_header.commit.block_id.hash.0.as_slice() != expected_block_hash {
        return Err(Error::InvalidHeader(
            "commit.block_id.hash != header.root()".into(),
        ));
    }

    let signed_vote = canonical_vote(
        &header.signed_header.commit,
        header.signed_header.header.chain_id.clone(),
        expected_block_hash,
    )
    .encode_length_delimited_to_vec();

    if !verify_zkp(
        &trusted_validators_hash.0,
        &untrusted_validators_hash.0,
        &signed_vote,
        &header.zero_knowledge_proof,
    ) {
        return Err(Error::InvalidZKP);
    }

    let untrusted_height = Height::new(
        header.trusted_height.revision_number,
        header.signed_header.commit.height.inner() as u64,
    );

    if untrusted_height > client_state.latest_height {
        client_state.latest_height = untrusted_height;
    }

    consensus_state.data.root = MerkleRoot {
        hash: header.signed_header.header.app_hash,
    };

    consensus_state.data.next_validators_hash = untrusted_validators_hash;
    consensus_state.timestamp = header.signed_header.header.time.seconds.inner() as u64;

    save_wasm_client_state(deps.branch(), client_state);
    save_consensus_state(deps, consensus_state, untrusted_height)?;

    Ok(ContractResult::valid(None))
}

fn canonical_vote(
    commit: &Commit,
    chain_id: String,
    expected_block_hash: [u8; 32],
) -> protos::tendermint::types::CanonicalVote {
    protos::tendermint::types::CanonicalVote {
        r#type: SignedMsgType::Precommit as i32,
        height: commit.height.inner(),
        round: commit.round.inner() as i64,
        // TODO(aeryz): Implement BlockId to proto::CanonicalBlockId
        block_id: Some(CanonicalBlockId {
            hash: expected_block_hash.to_vec(),
            part_set_header: Some(CanonicalPartSetHeader {
                total: commit.block_id.part_set_header.total,
                hash: commit.block_id.part_set_header.hash.0.to_vec(),
            }),
        }),
        chain_id,
    }
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
        client_state.data.trusting_period.seconds.inner() as u64,
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

use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response,
    StdError, Uint256,
};
use prost::Message;
use protos::{
    google::protobuf::StringValue,
    tendermint::types::{CanonicalBlockId, CanonicalPartSetHeader, SignedMsgType},
};
use rs_merkle::{algorithms::Sha256, Hasher, MerkleTree};
use unionlabs::{
    ibc::{
        core::client::height::Height,
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
    state::{read_client_state, read_consensus_state},
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
    unimplemented!()
}

pub fn update_header(mut deps: DepsMut, env: Env, header: Header) -> Result<ContractResult, Error> {
    let client_state = read_client_state(deps.as_ref())?;
    let consensus_state = read_consensus_state(deps.as_ref(), &header.trusted_height)?.ok_or(
        Error::ConsensusStateNotFound(
            header.trusted_height.revision_number,
            header.trusted_height.revision_height,
        ),
    )?;

    let untrusted_height_number = header.signed_header.commit.height as u64;
    let trusted_height_number = header.trusted_height.revision_number;

    if untrusted_height_number <= trusted_height_number {
        return Err(Error::InvalidHeader(
            "header height <= consensus state height".into(),
        ));
    }

    let trusted_timestamp = consensus_state.timestamp;
    let untrusted_timestamp = header.signed_header.header.time.seconds;

    if untrusted_timestamp as u64 <= trusted_timestamp {
        return Err(Error::InvalidHeader(
            "header time <= consensus state time".into(),
        ));
    }

    let current_time: Timestamp = env.block.time.into();

    if Duration::from(header.signed_header.header.time)
        < Duration::from(current_time) + client_state.data.trusting_period
    {
        return Err(Error::InvalidHeader("header expired".into()));
    }

    let max_clock_drift = current_time.seconds + client_state.data.max_clock_drift.seconds;

    if untrusted_timestamp >= max_clock_drift {
        return Err(Error::InvalidHeader("header back to the future".into()));
    }

    let trusted_validators_hash = consensus_state.data.next_validators_hash;

    let untrusted_validators_hash = if untrusted_height_number == trusted_height_number + 1 {
        trusted_validators_hash.clone()
    } else {
        header.untrusted_validator_set_root.0.to_vec()
    };

    let expected_block_hash = calculate_merkle_root(&header.signed_header.header);

    if header.signed_header.commit.block_id.hash.0.as_slice() != expected_block_hash {
        return Err(Error::InvalidHeader(
            "commit.block_id.hash != header.root()".into(),
        ));
    }

    let signed_vote = canonical_vote(
        header.signed_header.commit,
        header.signed_header.header.chain_id,
        expected_block_hash,
    )
    .encode_length_delimited_to_vec();

    if !verify_zkp(
        &trusted_validators_hash,
        &untrusted_validators_hash,
        &signed_vote,
        &header.zero_knowledge_proof,
    ) {
        return Err(Error::InvalidZKP);
    }

    todo!()
}

// TODO(aeryz): Move this to `unionlabs`
fn canonical_vote(
    commit: Commit,
    chain_id: String,
    expected_block_hash: [u8; 32],
) -> protos::tendermint::types::CanonicalVote {
    protos::tendermint::types::CanonicalVote {
        r#type: SignedMsgType::Precommit as i32,
        height: commit.height as i64,
        round: commit.round as i64,
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

// TODO(aeryz): Move to `unionlabs`? Or utils mod in this crate?
trait CdcEncode {
    fn cdc_encode(self) -> Vec<u8>;
}

impl CdcEncode for String {
    fn cdc_encode(self) -> Vec<u8> {
        protos::google::protobuf::StringValue { value: self }.encode_to_vec()
    }
}

impl CdcEncode for i64 {
    fn cdc_encode(self) -> Vec<u8> {
        protos::google::protobuf::Int64Value { value: self }.encode_to_vec()
    }
}

impl CdcEncode for Vec<u8> {
    fn cdc_encode(self) -> Vec<u8> {
        protos::google::protobuf::BytesValue { value: self }.encode_to_vec()
    }
}

// TODO(aeryz): Move to `unionlabs` or `utils`?
fn calculate_merkle_root(header: &unionlabs::tendermint::types::header::Header) -> [u8; 32] {
    let proto_version: protos::tendermint::version::Consensus = header.version.into();
    let proto_time: protos::google::protobuf::Timestamp = header.time.into();
    let proto_block_id: protos::tendermint::types::BlockId = header.last_block_id.clone().into();

    let leaves = [
        Sha256::hash(proto_version.encode_to_vec().as_slice()),
        Sha256::hash(&header.chain_id.clone().cdc_encode()),
        Sha256::hash(&(header.height as i64).cdc_encode()),
        Sha256::hash(&proto_time.encode_to_vec()),
        Sha256::hash(&proto_block_id.encode_to_vec()),
        Sha256::hash(&header.last_commit_hash.0.to_vec().cdc_encode()),
        Sha256::hash(&header.data_hash.0.to_vec().cdc_encode()),
        Sha256::hash(&header.validators_hash.0.to_vec().cdc_encode()),
        Sha256::hash(&header.next_validators_hash.0.to_vec().cdc_encode()),
        Sha256::hash(&header.consensus_hash.0.to_vec().cdc_encode()),
        Sha256::hash(&header.app_hash.0.to_vec().cdc_encode()),
        Sha256::hash(&header.last_results_hash.0.to_vec().cdc_encode()),
        Sha256::hash(&header.evidence_hash.0.to_vec().cdc_encode()),
        Sha256::hash(&header.proposer_address.0.to_vec().cdc_encode()),
    ];

    let merkle_tree: MerkleTree<Sha256> = MerkleTree::from_leaves(&leaves);

    merkle_tree.root().unwrap()
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

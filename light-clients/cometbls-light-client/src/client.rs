use cosmwasm_std::{Binary, Deps, DepsMut, Empty, Env};
use ics008_wasm_client::{
    storage_utils::{
        read_client_state, read_consensus_state, save_client_state, save_consensus_state,
    },
    ContractResult, IbcClient, MerklePath, Status, StorageState,
};
use prost::Message;
use unionlabs::{
    google::protobuf::timestamp::Timestamp,
    ibc::{
        core::{client::height::Height, commitment::merkle_root::MerkleRoot},
        lightclients::cometbls::{
            client_state::ClientState, consensus_state::ConsensusState, header::Header,
        },
    },
    tendermint::types::commit::Commit,
};

use crate::{errors::Error, zkp_verifier::verify_zkp_v2};

type WasmClientState = unionlabs::ibc::lightclients::wasm::client_state::ClientState<ClientState>;
type WasmConsensusState =
    unionlabs::ibc::lightclients::wasm::consensus_state::ConsensusState<ConsensusState>;

pub struct CometblsLightClient;

impl IbcClient for CometblsLightClient {
    type Error = Error;

    type CustomQuery = Empty;

    type Header = Header;

    // TODO(aeryz): Change this to appropriate misbehavior type when it is implemented
    type Misbehaviour = ();

    type ClientState = ClientState;

    type ConsensusState = ConsensusState;

    fn verify_membership(
        _deps: Deps<Self::CustomQuery>,
        _height: Height,
        _delay_time_period: u64,
        _delay_block_period: u64,
        _proof: Binary,
        _path: MerklePath,
        _value: StorageState,
    ) -> Result<ContractResult, Self::Error> {
        Ok(ContractResult::valid(None))
    }

    fn verify_header(
        deps: Deps<Self::CustomQuery>,
        env: Env,
        header: Self::Header,
    ) -> Result<ContractResult, Self::Error> {
        let client_state: WasmClientState = read_client_state(deps)?;
        let consensus_state: WasmConsensusState =
            read_consensus_state(deps, &header.trusted_height)?.ok_or(
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
        let untrusted_timestamp = header.signed_header.header.time.seconds.inner() as u64;

        if untrusted_timestamp <= trusted_timestamp {
            return Err(Error::InvalidHeader(
                "header time <= consensus state time".into(),
            ));
        }

        let current_time: Timestamp = env
            .block
            .time
            .try_into()
            .map_err(|_| Error::InvalidHeader("timestamp conversion failed".into()))?;

        if current_time
            .duration_since(&header.signed_header.header.time)
            .ok_or(Error::DurationAdditionOverflow)?
            .seconds()
            .inner() as u64
            > client_state.data.trusting_period
        {
            return Err(Error::InvalidHeader("header expired".into()));
        }

        let max_clock_drift =
            current_time.seconds.inner() as u64 + client_state.data.max_clock_drift;

        if untrusted_timestamp >= max_clock_drift {
            return Err(Error::InvalidHeader("header back to the future".into()));
        }

        let trusted_validators_hash = consensus_state.data.next_validators_hash;

        let untrusted_validators_hash = if untrusted_height_number == trusted_height_number + 1 {
            trusted_validators_hash.clone()
        } else {
            header.signed_header.header.validators_hash.clone()
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

        if !verify_zkp_v2(
            &trusted_validators_hash.0,
            &untrusted_validators_hash.0,
            &signed_vote,
            &header.zero_knowledge_proof,
        ) {
            return Err(Error::InvalidZKP);
        }

        Ok(ContractResult::valid(None))
    }

    fn verify_misbehaviour(
        _deps: Deps<Self::CustomQuery>,
        _misbehaviour: Self::Misbehaviour,
    ) -> Result<ContractResult, Self::Error> {
        Ok(ContractResult::valid(None))
    }

    fn update_state(
        mut deps: DepsMut<Self::CustomQuery>,
        _env: Env,
        header: Self::Header,
    ) -> Result<ContractResult, Self::Error> {
        let mut client_state: WasmClientState = read_client_state(deps.as_ref())?;
        let mut consensus_state: WasmConsensusState =
            read_consensus_state(deps.as_ref(), &header.trusted_height)?.ok_or(
                Error::ConsensusStateNotFound(
                    header.trusted_height.revision_number,
                    header.trusted_height.revision_height,
                ),
            )?;

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

        let untrusted_height_number = header.signed_header.commit.height.inner() as u64;
        let trusted_height_number = header.trusted_height.revision_number;

        let untrusted_validators_hash = if untrusted_height_number == trusted_height_number + 1 {
            consensus_state.data.next_validators_hash.clone()
        } else {
            header.signed_header.header.validators_hash
        };

        consensus_state.data.next_validators_hash = untrusted_validators_hash;
        consensus_state.timestamp = header.signed_header.header.time.seconds.inner() as u64;

        save_client_state(deps.branch(), client_state);
        save_consensus_state(deps, consensus_state, &untrusted_height);

        Ok(ContractResult::valid(None))
    }

    fn update_state_on_misbehaviour(
        _deps: DepsMut<Self::CustomQuery>,
        _env: Env,
        _client_message: ics008_wasm_client::ClientMessage,
    ) -> Result<ContractResult, Self::Error> {
        Ok(ContractResult::invalid("Not implemented".to_string()))
    }

    fn check_for_misbehaviour_on_header(
        _deps: Deps<Self::CustomQuery>,
        _header: Self::Header,
    ) -> Result<ContractResult, Self::Error> {
        // TODO(aeryz): Leaving this as success for us to be able to update the client. See: #588.
        Ok(ContractResult::valid(None))
    }

    fn check_for_misbehaviour_on_misbehaviour(
        _deps: Deps<Self::CustomQuery>,
        _misbehaviour: Self::Misbehaviour,
    ) -> Result<ContractResult, Self::Error> {
        Ok(ContractResult::invalid("Not implemented".to_string()))
    }

    fn verify_upgrade_and_update_state(
        _deps: DepsMut<Self::CustomQuery>,
        _upgrade_client_state: WasmClientState,
        _upgrade_consensus_state: WasmConsensusState,
        _proof_upgrade_client: Binary,
        _proof_upgrade_consensus_state: Binary,
    ) -> Result<ContractResult, Self::Error> {
        Ok(ContractResult::invalid("Not implemented".to_string()))
    }

    fn check_substitute_and_update_state(
        _deps: Deps<Self::CustomQuery>,
    ) -> Result<ContractResult, Self::Error> {
        Ok(ContractResult::invalid("Not implemented".to_string()))
    }

    fn status(
        deps: Deps<Self::CustomQuery>,
        env: &cosmwasm_std::Env,
    ) -> Result<ics008_wasm_client::QueryResponse, Self::Error> {
        let client_state: WasmClientState = read_client_state(deps)?;

        // TODO(aeryz): make client state optional
        if client_state.data.frozen_height.revision_height == 0 {
            return Ok(Status::Frozen.into());
        }

        let Some(consensus_state) = read_consensus_state::<Self::CustomQuery, ConsensusState>(
            deps,
            &client_state.latest_height,
        )?
        else {
            return Ok(Status::Expired.into());
        };

        if is_client_expired(
            consensus_state.timestamp,
            client_state.data.trusting_period,
            env.block.time.seconds(),
        ) {
            return Ok(Status::Expired.into());
        }

        Ok(Status::Active.into())
    }

    fn export_metadata(
        _deps: Deps<Self::CustomQuery>,
        _env: &cosmwasm_std::Env,
    ) -> Result<ics008_wasm_client::QueryResponse, Self::Error> {
        Ok(ics008_wasm_client::QueryResponse {
            status: String::new(),
            genesis_metadata: vec![],
        })
    }
}

fn is_client_expired(
    consensus_state_timestamp: u64,
    trusting_period: u64,
    current_block_time: u64,
) -> bool {
    consensus_state_timestamp + trusting_period < current_block_time
}

fn canonical_vote(
    commit: &Commit,
    chain_id: String,
    expected_block_hash: [u8; 32],
) -> protos::tendermint::types::CanonicalVote {
    protos::tendermint::types::CanonicalVote {
        r#type: protos::tendermint::types::SignedMsgType::Precommit as i32,
        height: commit.height.inner(),
        round: commit.round.inner() as i64,
        // TODO(aeryz): Implement BlockId to proto::CanonicalBlockId
        block_id: Some(protos::tendermint::types::CanonicalBlockId {
            hash: expected_block_hash.to_vec(),
            part_set_header: Some(protos::tendermint::types::CanonicalPartSetHeader {
                total: commit.block_id.part_set_header.total,
                hash: commit.block_id.part_set_header.hash.0.to_vec(),
            }),
        }),
        chain_id,
    }
}

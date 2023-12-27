use cosmwasm_std::{Binary, Deps, DepsMut, Empty, Env};
use ics008_wasm_client::{
    storage_utils::{
        read_client_state, read_consensus_state, save_client_state, save_consensus_state,
    },
    IbcClient, MerklePath, Status, StorageState,
};
use prost::Message;
use protos::ibc::core::client::v1::GenesisMetadata;
use sha2::Digest;
use unionlabs::{
    cosmos::ics23::{
        batch_proof::BatchProof,
        commitment_proof::CommitmentProof,
        existence_proof::{ExistenceProof, SpecMismatchError},
        hash_op::HashOp,
        inner_op::InnerOp,
        inner_spec::InnerSpec,
        leaf_op::LeafOp,
        length_op::LengthOp,
        proof_spec::ProofSpec,
    },
    ibc::{
        core::{
            client::height::Height,
            commitment::{merkle_proof::MerkleProof, merkle_root::MerkleRoot},
        },
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
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn verify_header(
        deps: Deps<Self::CustomQuery>,
        _env: Env,
        header: Self::Header,
    ) -> Result<(), Self::Error> {
        let client_state: WasmClientState = read_client_state(deps)?;
        let consensus_state: WasmConsensusState =
            read_consensus_state(deps, &header.trusted_height)?
                .ok_or(Error::ConsensusStateNotFound(header.trusted_height))?;

        let untrusted_height_number = header.signed_header.commit.height.inner() as u64;
        let trusted_height_number = header.trusted_height.revision_number;

        if untrusted_height_number <= trusted_height_number {
            return Err(Error::InvalidHeader(
                "header height <= consensus state height".into(),
            ));
        }

        let trusted_timestamp = consensus_state.data.timestamp;
        let untrusted_timestamp = header.signed_header.header.time.seconds.inner() as u64;

        if untrusted_timestamp <= trusted_timestamp {
            return Err(Error::InvalidHeader(
                "header time <= consensus state time".into(),
            ));
        }

        // let current_time: Timestamp = env
        //     .block
        //     .time
        //     .try_into()
        //     .map_err(|_| Error::InvalidHeader("timestamp conversion failed".into()))?;

        // if current_time
        //     .duration_since(&header.signed_header.header.time)
        //     .ok_or(Error::DurationAdditionOverflow)?
        //     .seconds()
        //     .inner() as u64
        //     > client_state.data.trusting_period
        // {
        //     return Err(Error::InvalidHeader("header expired".into()));
        // }

        // let max_clock_drift =
        //     current_time.seconds.inner() as u64 + client_state.data.max_clock_drift;

        // if untrusted_timestamp >= max_clock_drift {
        //     return Err(Error::InvalidHeader("header back to the future".into()));
        // }

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

        if client_state.data.chain_id != header.signed_header.header.chain_id {
            return Err(Error::InvalidHeader(format!(
                "chain ids dont match: {} {}",
                client_state.data.chain_id, header.signed_header.header.chain_id
            )));
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

        Ok(())
    }

    fn verify_misbehaviour(
        _deps: Deps<Self::CustomQuery>,
        _misbehaviour: Self::Misbehaviour,
    ) -> Result<(), Self::Error> {
        panic!("Not implemented")
    }

    fn update_state(
        mut deps: DepsMut<Self::CustomQuery>,
        _env: Env,
        header: Self::Header,
    ) -> Result<ics008_wasm_client::UpdateStateResult, Self::Error> {
        let mut client_state: WasmClientState = read_client_state(deps.as_ref())?;
        let mut consensus_state: WasmConsensusState =
            read_consensus_state(deps.as_ref(), &header.trusted_height)?
                .ok_or(Error::ConsensusStateNotFound(header.trusted_height))?;

        let untrusted_height = Height::new(
            header.trusted_height.revision_number,
            header.signed_header.commit.height.inner() as u64,
        );

        if untrusted_height > client_state.latest_height {
            client_state.latest_height = untrusted_height;
            client_state.data.latest_height = untrusted_height;
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
        consensus_state.data.timestamp = header.signed_header.header.time.seconds.inner() as u64;

        save_client_state(deps.branch(), client_state);
        save_consensus_state(deps, consensus_state, &untrusted_height);

        Ok(ics008_wasm_client::UpdateStateResult {
            heights: vec![untrusted_height],
        })
    }

    fn update_state_on_misbehaviour(
        _deps: DepsMut<Self::CustomQuery>,
        _env: Env,
        _client_message: Vec<u8>,
    ) -> Result<(), Self::Error> {
        panic!("not implemented")
    }

    fn check_for_misbehaviour_on_header(
        _deps: Deps<Self::CustomQuery>,
        _header: Self::Header,
    ) -> Result<ics008_wasm_client::CheckForMisbehaviourResult, Self::Error> {
        // TODO(aeryz): Leaving this as success for us to be able to update the client. See: #588.
        Ok(ics008_wasm_client::CheckForMisbehaviourResult {
            found_misbehaviour: false,
        })
    }

    fn check_for_misbehaviour_on_misbehaviour(
        _deps: Deps<Self::CustomQuery>,
        _misbehaviour: Self::Misbehaviour,
    ) -> Result<ics008_wasm_client::CheckForMisbehaviourResult, Self::Error> {
        unimplemented!()
    }

    fn verify_upgrade_and_update_state(
        _deps: DepsMut<Self::CustomQuery>,
        _upgrade_client_state: WasmClientState,
        _upgrade_consensus_state: WasmConsensusState,
        _proof_upgrade_client: Binary,
        _proof_upgrade_consensus_state: Binary,
    ) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn migrate_client_store(_deps: Deps<Self::CustomQuery>) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn status(
        deps: Deps<Self::CustomQuery>,
        env: &cosmwasm_std::Env,
    ) -> Result<Status, Self::Error> {
        let client_state: WasmClientState = read_client_state(deps)?;

        if client_state.data.frozen_height != Default::default() {
            return Ok(Status::Frozen);
        }

        let Some(consensus_state) = read_consensus_state::<Self::CustomQuery, ConsensusState>(
            deps,
            &client_state.latest_height,
        )?
        else {
            return Ok(Status::Expired);
        };

        if is_client_expired(
            consensus_state.data.timestamp,
            client_state.data.trusting_period,
            env.block.time.seconds(),
        ) {
            return Ok(Status::Expired);
        }

        Ok(Status::Active)
    }

    fn export_metadata(
        _deps: Deps<Self::CustomQuery>,
        _env: &cosmwasm_std::Env,
    ) -> Result<Vec<GenesisMetadata>, Self::Error> {
        Ok(Vec::new())
    }

    fn timestamp_at_height(
        deps: Deps<Self::CustomQuery>,
        height: Height,
    ) -> Result<u64, Self::Error> {
        Ok(
            read_consensus_state::<Self::CustomQuery, ConsensusState>(deps, &height)?
                .ok_or(Error::ConsensusStateNotFound(height))?
                .data
                .timestamp,
        )
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

#[derive(Debug)]
pub enum VerifyMembershipError {
    SpecMismatch(SpecMismatchError),
    KeyAndExistenceProofKeyMismatch {
        key: Vec<u8>,
        existence_proof_key: Vec<u8>,
    },
    ValueAndExistenceProofValueMismatch {
        value: Vec<u8>,
        existence_proof_value: Vec<u8>,
    },
    RootCalculation(unionlabs::cosmos::ics23::existence_proof::CalculateRootError),
    CalculatedAndGivenRootMismatch {
        calculated_root: Vec<u8>,
        given_root: Vec<u8>,
    },
    ProofDoesNotExist,
}

fn verify_membership(
    proof: MerkleProof,
    consensus_root: &MerkleRoot,
    path: MerklePath,
    value: &[u8],
) -> Result<(), VerifyMembershipError> {
    // TODO(aeryz): check if this supposed to be embedded or configurable
    if proof.proofs.len() != 2 {
        panic!("Proof length needs to match the spec");
    }

    // TODO(aeryz): Make this spec a global constant if it's going to be embedded
    if path.key_path.len() != 2 {
        panic!("Path length needs to match the spec");
    }

    verify_chained_membership_proof(consensus_root.hash.as_ref(), &proof.proofs, path, value, 0)
}

fn verify_chained_membership_proof(
    root: &[u8],
    proofs: &[CommitmentProof],
    keys: MerklePath,
    value: &[u8],
    mut index: usize,
) -> Result<(), VerifyMembershipError> {
    let specs = [iavl_spec(), tendermint_proof_spec()];

    // FIXME(aeryz): ugly change
    let mut tmp_value = value.to_vec();

    while index < proofs.len() {
        match &proofs[index] {
            CommitmentProof::Exist(proof) => {
                let subroot = proof
                    .calculate_root(None)
                    .map_err(VerifyMembershipError::RootCalculation)?;

                if let Some(key) = keys.key_path.get(keys.key_path.len() - 1 - index) {
                    do_verify_membership(
                        specs[index].clone(), // TODO(aeryz): I'm about to throw up
                        &subroot,
                        CommitmentProof::Exist(proof.clone()), // TODO(aeryz): disgusting
                        key.as_bytes(), // TODO(aeryz): weird? is this really like this?
                        &tmp_value,
                    )?;
                } else {
                    panic!("could not retrieve key bytes for key");
                };

                tmp_value = subroot;
            }
            CommitmentProof::Nonexist(_) => {
                panic!("chained membership proof contains nonexistence proof at index %d");
            }
            _ => {
                panic!("invalid proof type");
            }
        }

        index += 1;
    }

    Ok(())
}

fn do_verify_membership(
    spec: ProofSpec,
    root: &[u8],
    proof: CommitmentProof,
    key: &[u8],
    value: &[u8],
) -> Result<(), VerifyMembershipError> {
    let proof = proof.decompress();

    if let Some(ep) = get_exist_proof_for_key(proof, key) {
        verify_existence_proof(ep, spec, root, key, value)
    } else {
        Err(VerifyMembershipError::ProofDoesNotExist)
    }
}

fn verify_existence_proof(
    existence_proof: ExistenceProof,
    spec: ProofSpec,
    root: &[u8],
    key: &[u8],
    value: &[u8],
) -> Result<(), VerifyMembershipError> {
    existence_proof
        .check_against_spec(&spec, &iavl_spec())
        .map_err(VerifyMembershipError::SpecMismatch)?;

    if key != existence_proof.key {
        return Err(VerifyMembershipError::KeyAndExistenceProofKeyMismatch {
            key: key.into(),
            existence_proof_key: existence_proof.key,
        });
    }

    if value != existence_proof.value {
        return Err(VerifyMembershipError::ValueAndExistenceProofValueMismatch {
            value: value.into(),
            existence_proof_value: existence_proof.value,
        });
    }

    let calc = existence_proof
        .calculate_root(Some(spec))
        .map_err(VerifyMembershipError::RootCalculation)?;

    if root != calc {
        return Err(VerifyMembershipError::CalculatedAndGivenRootMismatch {
            calculated_root: calc,
            given_root: root.into(),
        });
    }

    Ok(())
}

fn get_exist_proof_for_key(proof: CommitmentProof, key: &[u8]) -> Option<ExistenceProof> {
    match proof {
        CommitmentProof::Exist(exist) => {
            if exist.key.as_slice() == key {
                return Some(exist);
            }

            None
        }
        CommitmentProof::Batch(batch) => {
            for sub in batch.entries {
                match sub {
                    unionlabs::cosmos::ics23::batch_entry::BatchEntry::Exist(exist) => {
                        if exist.key.as_slice() == key {
                            return Some(exist);
                        }
                    }
                    _ => {}
                }
            }

            None
        }
        _ => None,
    }
}

fn iavl_spec() -> ProofSpec {
    ProofSpec {
        leaf_spec: LeafOp {
            hash: HashOp::Sha256,
            prehash_key: HashOp::NoHash,
            prehash_value: HashOp::Sha256,
            length: LengthOp::VarProto,
            prefix: vec![0],
        },
        inner_spec: InnerSpec {
            child_order: vec![0, 1],
            child_size: 33,
            min_prefix_length: 4,
            max_prefix_length: 12,
            empty_child: vec![],
            hash: HashOp::Sha256,
        },
        max_depth: 0,
        min_depth: 0,
    }
}

fn tendermint_proof_spec() -> ProofSpec {
    ProofSpec {
        leaf_spec: LeafOp {
            hash: HashOp::Sha256,
            prehash_key: HashOp::NoHash,
            prehash_value: HashOp::Sha256,
            length: LengthOp::VarProto,
            prefix: [0].into(),
        },
        inner_spec: InnerSpec {
            child_order: [0, 1].into(),
            child_size: 32,
            min_prefix_length: 1,
            max_prefix_length: 1,
            empty_child: [].into(),
            hash: HashOp::Sha256,
        },
        max_depth: 0,
        min_depth: 0,
    }
}

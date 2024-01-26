use cosmwasm_std::{Deps, DepsMut, Empty, Env};
use ics008_wasm_client::{
    storage_utils::{
        read_client_state, read_consensus_state, save_client_state, save_consensus_state,
    },
    IbcClient, Status, StorageState,
};
use ics23::ibc_api::SDK_SPECS;
use prost::Message;
use protos::ibc::core::client::v1::GenesisMetadata;
use unionlabs::{
    encoding::Proto,
    hash::H256,
    ibc::{
        core::{
            client::height::Height,
            commitment::{
                merkle_path::MerklePath, merkle_proof::MerkleProof, merkle_root::MerkleRoot,
            },
        },
        lightclients::cometbls::{
            client_state::ClientState, consensus_state::ConsensusState, header::Header,
        },
    },
    tendermint::types::commit::Commit,
    TryFromProto,
};

use crate::{
    errors::{Error, InvalidHeaderError},
    storage::{
        next_consensus_state_meta, prev_consensus_state_meta, save_consensus_state_metadata,
    },
    zkp_verifier::verify_zkp,
};

type WasmClientState = unionlabs::ibc::lightclients::wasm::client_state::ClientState<ClientState>;
type WasmConsensusState =
    unionlabs::ibc::lightclients::wasm::consensus_state::ConsensusState<ConsensusState>;

pub struct CometblsLightClient;

impl IbcClient for CometblsLightClient {
    type Error = Error;

    type CustomQuery = Empty;

    type Header = Header;

    // TODO(aeryz): Change this to appropriate misbehavior type when it is implemented
    type Misbehaviour = Header;

    type ClientState = ClientState;

    type ConsensusState = ConsensusState;

    type Encoding = Proto;

    fn verify_membership(
        deps: Deps<Self::CustomQuery>,
        height: Height,
        _delay_time_period: u64,
        _delay_block_period: u64,
        proof: Vec<u8>,
        path: MerklePath,
        value: StorageState,
    ) -> Result<(), Self::Error> {
        let consensus_state: WasmConsensusState =
            read_consensus_state(deps, &height)?.ok_or(Error::ConsensusStateNotFound(height))?;

        let merkle_proof = MerkleProof::try_from_proto_bytes(proof.as_ref()).map_err(|e| {
            Error::DecodeFromProto {
                reason: format!("{:?}", e),
            }
        })?;

        match value {
            StorageState::Occupied(value) => ics23::ibc_api::verify_membership(
                &merkle_proof,
                &SDK_SPECS,
                &consensus_state.data.app_hash,
                &path,
                value,
            ),
            StorageState::Empty => ics23::ibc_api::verify_non_membership(
                &merkle_proof,
                &SDK_SPECS,
                &consensus_state.data.app_hash,
                &path,
            ),
        }
        .map_err(Error::VerifyMembership)
    }

    fn verify_header(
        deps: Deps<Self::CustomQuery>,
        env: Env,
        header: Self::Header,
    ) -> Result<(), Self::Error> {
        let client_state: WasmClientState = read_client_state(deps)?;
        let consensus_state: WasmConsensusState =
            read_consensus_state(deps, &header.trusted_height)?
                .ok_or(Error::ConsensusStateNotFound(header.trusted_height))?;

        // SAFETY: height is bound to be 0..i64::MAX which makes it within the bounds of u64
        let untrusted_height_number = header.signed_header.commit.height.inner() as u64;
        let trusted_height_number = header.trusted_height.revision_height;

        if untrusted_height_number <= trusted_height_number {
            return Err(InvalidHeaderError::SignedHeaderHeightMustBeMoreRecent {
                signed_height: untrusted_height_number,
                trusted_height: trusted_height_number,
            }
            .into());
        }

        let trusted_timestamp = consensus_state.data.timestamp;
        let untrusted_timestamp = {
            let header_timestamp = header.signed_header.header.time.seconds.inner();
            header_timestamp
                .try_into()
                .map_err(|_| InvalidHeaderError::NegativeTimestamp(header_timestamp))?
        };

        if untrusted_timestamp <= trusted_timestamp {
            return Err(InvalidHeaderError::SignedHeaderTimestampMustBeMoreRecent {
                signed_timestamp: untrusted_timestamp,
                trusted_timestamp,
            }
            .into());
        }

        if is_client_expired(
            untrusted_timestamp,
            client_state.data.trusting_period,
            env.block.time.seconds(),
        ) {
            return Err(InvalidHeaderError::HeaderExpired(consensus_state.data.timestamp).into());
        }

        let max_clock_drift = env
            .block
            .time
            .seconds()
            .checked_add(client_state.data.max_clock_drift)
            .ok_or(Error::MathOverflow)?;

        if untrusted_timestamp >= max_clock_drift {
            return Err(InvalidHeaderError::SignedHeaderCannotExceedMaxClockDrift {
                signed_timestamp: untrusted_timestamp,
                max_clock_drift,
            }
            .into());
        }

        let trusted_validators_hash = consensus_state.data.next_validators_hash;

        let untrusted_validators_hash = if untrusted_height_number == trusted_height_number + 1 {
            &trusted_validators_hash
        } else {
            &header.signed_header.header.validators_hash
        };

        let expected_block_hash = header
            .signed_header
            .header
            .calculate_merkle_root()
            .ok_or(Error::UnableToCalculateMerkleRoot)?;

        if header.signed_header.commit.block_id.hash != expected_block_hash {
            return Err(InvalidHeaderError::SignedHeaderMismatchWithCommitHash {
                commit_hash: header.signed_header.commit.block_id.hash,
                signed_header_root: expected_block_hash,
            }
            .into());
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
    ) -> Result<Vec<Height>, Self::Error> {
        let mut client_state: WasmClientState = read_client_state(deps.as_ref())?;
        let mut consensus_state: WasmConsensusState =
            read_consensus_state(deps.as_ref(), &header.trusted_height)?
                .ok_or(Error::ConsensusStateNotFound(header.trusted_height))?;

        let untrusted_height = Height::new(
            header.trusted_height.revision_number,
            // SAFETY: height is bound to be 0..i64::MAX which makes it within the bounds of u64
            header.signed_header.commit.height.inner() as u64,
        );

        if untrusted_height > client_state.latest_height {
            client_state.latest_height = untrusted_height;
            client_state.data.latest_height = untrusted_height;
        }

        consensus_state.data.app_hash = MerkleRoot {
            hash: header.signed_header.header.app_hash,
        };

        consensus_state.data.next_validators_hash =
            header.signed_header.header.next_validators_hash;
        consensus_state.data.timestamp = header
            .signed_header
            .header
            .time
            .seconds
            .inner()
            .try_into()
            .map_err(|_| {
                Error::NegativeTimestamp(header.signed_header.header.time.seconds.inner())
            })?;

        save_client_state(deps.branch(), client_state);
        save_consensus_state_metadata(
            deps.branch(),
            consensus_state.data.timestamp,
            untrusted_height,
        );
        save_consensus_state(deps, consensus_state, &untrusted_height);

        Ok(vec![untrusted_height])
    }

    fn update_state_on_misbehaviour(
        _deps: DepsMut<Self::CustomQuery>,
        _env: Env,
        _client_message: Vec<u8>,
    ) -> Result<(), Self::Error> {
        panic!("not implemented")
    }

    fn check_for_misbehaviour_on_header(
        deps: Deps<Self::CustomQuery>,
        header: Self::Header,
    ) -> Result<bool, Self::Error> {
        let height = update_height(&header);

        let expected_timestamp: u64 = header
            .signed_header
            .header
            .time
            .seconds
            .inner()
            .try_into()
            .map_err(|_| {
                Error::NegativeTimestamp(header.signed_header.header.time.seconds.inner())
            })?;

        // If there is already a header at this height, it should be exactly the same as the header that
        // we have saved previously. If this is not the case, either the client is broken or the chain is
        // broken. Because it should not be possible to have two distinct valid headers at a height.
        if let Some(WasmConsensusState {
            data:
                ConsensusState {
                    timestamp,
                    root: MerkleRoot { hash },
                    next_validators_hash,
                },
        }) = read_consensus_state::<_, ConsensusState>(deps, &height)?
        {
            if timestamp != expected_timestamp
                || hash != header.signed_header.header.app_hash
                || next_validators_hash != header.signed_header.header.next_validators_hash
            {
                return Ok(true);
            }
        }

        if let Ok(Some((_, next_consensus_state))) = next_consensus_state_meta(deps, height) {
            if next_consensus_state.timestamp < expected_timestamp {
                return Ok(true);
            }
        }

        if let Ok(Some((_, prev_consensus_state))) = prev_consensus_state_meta(deps, height) {
            if prev_consensus_state.timestamp > expected_timestamp {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn check_for_misbehaviour_on_misbehaviour(
        _deps: Deps<Self::CustomQuery>,
        _misbehaviour: Self::Misbehaviour,
    ) -> Result<bool, Self::Error> {
        unimplemented!()
    }

    fn verify_upgrade_and_update_state(
        _deps: DepsMut<Self::CustomQuery>,
        _upgrade_client_state: Self::ClientState,
        _upgrade_consensus_state: Self::ConsensusState,
        _proof_upgrade_client: Vec<u8>,
        _proof_upgrade_consensus_state: Vec<u8>,
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
    if let Some(sum) = consensus_state_timestamp.checked_add(trusting_period) {
        sum < current_block_time
    } else {
        // TODO(aeryz): This is really an unexpected error and this should return
        // a nice error message.
        true
    }
}

fn canonical_vote(
    commit: &Commit,
    chain_id: String,
    expected_block_hash: H256,
) -> protos::tendermint::types::CanonicalVote {
    protos::tendermint::types::CanonicalVote {
        r#type: protos::tendermint::types::SignedMsgType::Precommit as i32,
        height: commit.height.inner(),
        round: commit.round.inner() as i64,
        // TODO(aeryz): Implement BlockId to proto::CanonicalBlockId
        block_id: Some(protos::tendermint::types::CanonicalBlockId {
            hash: expected_block_hash.into(),
            part_set_header: Some(protos::tendermint::types::CanonicalPartSetHeader {
                total: commit.block_id.part_set_header.total,
                hash: commit.block_id.part_set_header.hash.0.to_vec(),
            }),
        }),
        chain_id,
    }
}

/// Returns the height from the update data
///
/// `header.signed_header.header.height` is `u64` and it does not contain the
/// revision height. This function is a utility to generate a `Height` type out
/// of the update data.
fn update_height(header: &Header) -> Height {
    Height {
        revision_number: header.trusted_height.revision_number,
        // SAFETY: height's bounds are [0..i64::MAX]
        revision_height: header.signed_header.header.height.inner() as u64,
    }
}

#[cfg(test)]
mod tests {

    use std::{cmp::Ordering, fs};

    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage},
        OwnedDeps, Timestamp,
    };
    use unionlabs::{
        google::protobuf::timestamp::TIMESTAMP_SECONDS_MAX, ibc::lightclients::cometbls,
    };

    use super::*;

    const UPDATES_DIR_PATH: &str = "src/test/updates/";

    const INITIAL_CONSENSUS_STATE_HEIGHT: Height = Height {
        revision_number: 1,
        revision_height: 1124,
    };

    lazy_static::lazy_static! {
        static ref UPDATES: Vec<cometbls::header::Header> = {
            let mut update_files = vec![];
            for entry in fs::read_dir(UPDATES_DIR_PATH).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.file_name().is_some() {
                    update_files.push(path);
                }
            }

            update_files.sort_by(|lhs, rhs| {
                let lhs = lhs.file_name().unwrap().to_string_lossy().strip_suffix(".json").unwrap().to_string().parse::<u32>().unwrap();
                let rhs = rhs.file_name().unwrap().to_string_lossy().strip_suffix(".json").unwrap().to_string().parse().unwrap();
                if lhs > rhs {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            });

            let mut updates = vec![];
            let mut prev_height = 0;
            for f in update_files {
                let mut data: cometbls::header::Header= serde_json::from_str(&fs::read_to_string(f).unwrap()).unwrap();
                if prev_height != 0 {
                    data.trusted_height.revision_height = prev_height;
                }
                prev_height = data.signed_header.header.height.inner().try_into().unwrap();
                updates.push(data);
            }

            updates
        };
    }

    fn prepare_test_data() -> (
        OwnedDeps<MockStorage, MockApi, MockQuerier>,
        cometbls::header::Header,
        Env,
    ) {
        let mut deps = mock_dependencies();

        let wasm_client_state: WasmClientState =
            serde_json::from_str(&fs::read_to_string("src/test/client_state.json").unwrap())
                .unwrap();

        let wasm_consensus_state: WasmConsensusState =
            serde_json::from_str(&fs::read_to_string("src/test/consensus_state.json").unwrap())
                .unwrap();

        save_client_state(deps.as_mut(), wasm_client_state);
        save_consensus_state(
            deps.as_mut(),
            wasm_consensus_state.clone(),
            &INITIAL_CONSENSUS_STATE_HEIGHT,
        );

        let update = UPDATES[0].clone();

        let mut env = mock_env();
        env.block.time =
            cosmwasm_std::Timestamp::from_seconds(wasm_consensus_state.data.timestamp + 60 * 5);

        (deps, update, env)
    }

    #[test]
    fn is_client_expired_works() {
        // expires when a + b < c
        assert_eq!(is_client_expired(1, 1, 10), true);
        assert_eq!(is_client_expired(1, 10, 10), false);
        assert_eq!(is_client_expired(10, 1, 10), false);
        // expires when a + b = c
        assert_eq!(is_client_expired(5, 5, 10), false);
        // expires when a + b overflows
        assert_eq!(is_client_expired(u64::MAX, 5, 10), true);
    }

    // TODO: avoid using the zkp verifier for this tests as we already test it
    // separately in cometbls-groth16-verifier. Instead, mock the verifier and
    // predetermine what the result will be.

    #[test]
    fn verify_and_update_header_works_with_good_data() {
        let mut deps = mock_dependencies();

        let wasm_client_state: WasmClientState =
            serde_json::from_str(&fs::read_to_string("src/test/client_state.json").unwrap())
                .unwrap();

        let wasm_consensus_state: WasmConsensusState =
            serde_json::from_str(&fs::read_to_string("src/test/consensus_state.json").unwrap())
                .unwrap();

        let prev_consensus_height = INITIAL_CONSENSUS_STATE_HEIGHT;

        save_client_state(deps.as_mut(), wasm_client_state);
        save_consensus_state(deps.as_mut(), wasm_consensus_state, &prev_consensus_height);

        for update in UPDATES.iter() {
            let mut env = mock_env();
            env.block.time = cosmwasm_std::Timestamp::from_seconds(
                update
                    .signed_header
                    .header
                    .time
                    .seconds
                    .inner()
                    .try_into()
                    .unwrap(),
            );

            assert_eq!(
                CometblsLightClient::check_for_misbehaviour_on_header(
                    deps.as_ref(),
                    update.clone()
                )
                .unwrap(),
                false
            );
            CometblsLightClient::verify_header(deps.as_ref(), env.clone(), update.clone()).unwrap();
            CometblsLightClient::update_state(deps.as_mut(), env, update.clone()).unwrap();

            let consensus_state: WasmConsensusState = read_consensus_state(
                deps.as_ref(),
                &Height {
                    revision_number: 1,
                    revision_height: update
                        .signed_header
                        .header
                        .height
                        .inner()
                        .try_into()
                        .unwrap(),
                },
            )
            .unwrap()
            .unwrap();
            assert_eq!(
                consensus_state.data.timestamp,
                TryInto::<u64>::try_into(update.signed_header.header.time.seconds.inner()).unwrap()
            );
            assert_eq!(
                consensus_state.data.next_validators_hash,
                if TryInto::<u64>::try_into(update.signed_header.commit.height.inner()).unwrap()
                    == update.trusted_height.revision_height + 1
                {
                    let prev_consensus_state: WasmConsensusState =
                        read_consensus_state(deps.as_ref(), &prev_consensus_height)
                            .unwrap()
                            .unwrap();
                    prev_consensus_state.data.next_validators_hash
                } else {
                    update.signed_header.header.validators_hash.clone()
                }
            );
            assert_eq!(
                consensus_state.data.app_hash.hash,
                update.signed_header.header.app_hash
            );
        }
    }

    #[test]
    fn verify_header_fails_with_signed_header_height_from_past() {
        let (deps, mut update, env) = prepare_test_data();

        update.signed_header.commit.height = 0.try_into().unwrap();

        assert!(matches!(
            CometblsLightClient::verify_header(deps.as_ref(), env, update),
            Err(Error::InvalidHeader(
                InvalidHeaderError::SignedHeaderHeightMustBeMoreRecent { .. }
            ))
        ));
    }

    #[test]
    fn verify_header_fails_with_signed_header_timestamp_from_past() {
        let (deps, mut update, env) = prepare_test_data();

        update.signed_header.header.time.seconds = 0.try_into().unwrap();

        assert!(matches!(
            CometblsLightClient::verify_header(deps.as_ref(), env, update),
            Err(Error::InvalidHeader(
                InvalidHeaderError::SignedHeaderTimestampMustBeMoreRecent { .. }
            ))
        ));
    }

    #[test]
    fn verify_header_fails_with_negative_timestamp() {
        let (deps, mut update, env) = prepare_test_data();

        update.signed_header.header.time.seconds = (-10000).try_into().unwrap();

        assert!(matches!(
            CometblsLightClient::verify_header(deps.as_ref(), env, update),
            Err(Error::InvalidHeader(InvalidHeaderError::NegativeTimestamp(
                _
            )))
        ));
    }

    #[test]
    fn verify_header_fails_when_client_expired() {
        let (deps, update, mut env) = prepare_test_data();

        env.block.time = Timestamp::from_nanos(u64::MAX);

        assert!(matches!(
            CometblsLightClient::verify_header(deps.as_ref(), env, update),
            Err(Error::InvalidHeader(InvalidHeaderError::HeaderExpired(_)))
        ));
    }

    #[test]
    fn verify_header_fails_when_signed_header_exceeds_max_clock_drift() {
        let (mut deps, correct_update, mut env) = prepare_test_data();

        let mut update = correct_update.clone();
        update.signed_header.header.time.seconds = TIMESTAMP_SECONDS_MAX.try_into().unwrap();

        assert!(matches!(
            CometblsLightClient::verify_header(deps.as_ref(), env.clone(), update),
            Err(Error::InvalidHeader(
                InvalidHeaderError::SignedHeaderCannotExceedMaxClockDrift { .. }
            ))
        ));

        let mut client_state: WasmClientState = read_client_state(deps.as_ref()).unwrap();
        client_state.data.max_clock_drift = 0;
        save_client_state(deps.as_mut(), client_state);
        env.block.time = Timestamp::from_seconds(
            correct_update
                .signed_header
                .header
                .time
                .seconds
                .inner()
                .try_into()
                .unwrap(),
        );

        assert!(matches!(
            CometblsLightClient::verify_header(deps.as_ref(), env, correct_update),
            Err(Error::InvalidHeader(
                InvalidHeaderError::SignedHeaderCannotExceedMaxClockDrift { .. }
            ))
        ));
    }

    #[test]
    fn verify_header_fails_when_invalid_commit_hash() {
        let (deps, correct_update, env) = prepare_test_data();

        let mut update = correct_update.clone();
        update.signed_header.commit.block_id.hash.0[0] =
            u8::MAX - update.signed_header.commit.block_id.hash.0[0];

        assert!(matches!(
            CometblsLightClient::verify_header(deps.as_ref(), env.clone(), update),
            Err(Error::InvalidHeader(
                InvalidHeaderError::SignedHeaderMismatchWithCommitHash { .. }
            ))
        ));

        let mut update = correct_update.clone();
        update.signed_header.header.chain_id = "foobar".to_string();

        assert!(matches!(
            CometblsLightClient::verify_header(deps.as_ref(), env, update),
            Err(Error::InvalidHeader(
                InvalidHeaderError::SignedHeaderMismatchWithCommitHash { .. }
            ))
        ));
    }

    #[test]
    fn verify_header_fails_when_invalid_zkp() {
        let (deps, mut update, env) = prepare_test_data();

        update.zero_knowledge_proof[0] = u8::MAX - update.zero_knowledge_proof[0];

        assert_eq!(
            CometblsLightClient::verify_header(deps.as_ref(), env, update),
            Err(Error::InvalidZKP)
        );
    }

    #[test]
    fn verify_header_fails_when_invalid_trusted_validators_hash() {
        let (mut deps, update, env) = prepare_test_data();

        let mut consensus_state: WasmConsensusState =
            read_consensus_state(deps.as_ref(), &INITIAL_CONSENSUS_STATE_HEIGHT)
                .unwrap()
                .unwrap();

        consensus_state.data.next_validators_hash.0[0] =
            u8::MAX - consensus_state.data.next_validators_hash.0[0];

        save_consensus_state(
            deps.as_mut(),
            consensus_state,
            &INITIAL_CONSENSUS_STATE_HEIGHT,
        );

        assert_eq!(
            CometblsLightClient::verify_header(deps.as_ref(), env, update),
            Err(Error::InvalidZKP)
        );
    }

    #[test]
    fn verify_header_fails_when_invalid_signed_vote() {
        let (deps, mut update, env) = prepare_test_data();

        update.signed_header.commit.round = (update.signed_header.commit.round.inner() + 1)
            .try_into()
            .unwrap();

        assert_eq!(
            CometblsLightClient::verify_header(deps.as_ref(), env, update),
            Err(Error::InvalidZKP)
        );
    }

    #[test]
    fn query_status_returns_active() {
        let mut deps = mock_dependencies();

        let wasm_client_state: WasmClientState =
            serde_json::from_str(&fs::read_to_string("src/test/client_state.json").unwrap())
                .unwrap();

        let wasm_consensus_state: WasmConsensusState =
            serde_json::from_str(&fs::read_to_string("src/test/consensus_state.json").unwrap())
                .unwrap();

        save_client_state(deps.as_mut(), wasm_client_state);

        save_consensus_state(
            deps.as_mut(),
            wasm_consensus_state,
            &INITIAL_CONSENSUS_STATE_HEIGHT,
        );

        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(0);

        assert_eq!(
            CometblsLightClient::status(deps.as_ref(), &env),
            Ok(Status::Active.into())
        );
    }

    #[test]
    fn query_status_expired_when_consensus_state_missing() {
        let mut deps = mock_dependencies();

        let wasm_client_state: WasmClientState =
            serde_json::from_str(&fs::read_to_string("src/test/client_state.json").unwrap())
                .unwrap();

        save_client_state(deps.as_mut(), wasm_client_state);

        assert_eq!(
            CometblsLightClient::status(deps.as_ref(), &mock_env()),
            Ok(Status::Expired.into())
        );
    }

    #[test]
    fn query_status_expired_when_client_expired() {
        let mut deps = mock_dependencies();

        let wasm_client_state: WasmClientState =
            serde_json::from_str(&fs::read_to_string("src/test/client_state.json").unwrap())
                .unwrap();

        let wasm_consensus_state: WasmConsensusState =
            serde_json::from_str(&fs::read_to_string("src/test/consensus_state.json").unwrap())
                .unwrap();

        save_client_state(deps.as_mut(), wasm_client_state);

        save_consensus_state(
            deps.as_mut(),
            wasm_consensus_state,
            &INITIAL_CONSENSUS_STATE_HEIGHT,
        );

        let mut env = mock_env();
        env.block.time = Timestamp::from_nanos(u64::MAX);

        assert_eq!(
            CometblsLightClient::status(deps.as_ref(), &env),
            Ok(Status::Expired.into())
        );
    }

    #[test]
    fn query_status_frozen() {
        let mut deps = mock_dependencies();

        let mut wasm_client_state: WasmClientState =
            serde_json::from_str(&fs::read_to_string("src/test/client_state.json").unwrap())
                .unwrap();

        wasm_client_state.data.frozen_height.revision_height = 1;

        save_client_state(deps.as_mut(), wasm_client_state);

        assert_eq!(
            CometblsLightClient::status(deps.as_ref(), &mock_env()),
            Ok(Status::Frozen.into()),
        );
    }
}

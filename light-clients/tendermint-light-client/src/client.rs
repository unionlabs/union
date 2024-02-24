use cosmwasm_std::{Deps, DepsMut, Empty, Env};
use ics008_wasm_client::{
    storage_utils::{
        read_client_state, read_consensus_state, save_client_state, save_consensus_state,
    },
    IbcClient, Status, StorageState,
};
use ics23::ibc_api::SDK_SPECS;
use tendermint_verifier::types::SignatureVerifier;
use unionlabs::{
    bounded::BoundedI64,
    encoding::Proto,
    google::protobuf::{duration::Duration, timestamp::Timestamp},
    hash::H256,
    ibc::{
        core::{
            client::{genesis_metadata::GenesisMetadata, height::Height},
            commitment::{
                merkle_path::MerklePath, merkle_proof::MerkleProof, merkle_root::MerkleRoot,
            },
        },
        lightclients::tendermint::{
            client_state::ClientState, consensus_state::ConsensusState, header::Header,
        },
    },
    tendermint::types::{commit::Commit, signed_header::SignedHeader},
    TryFromProto,
};

use crate::{
    errors::{Error, InvalidHeaderError},
    storage::{
        get_or_next_consensus_state_meta, get_or_prev_consensus_state_meta,
        save_consensus_state_metadata,
    },
    verifier::Ed25519Verifier,
};

type WasmClientState = unionlabs::ibc::lightclients::wasm::client_state::ClientState<ClientState>;
type WasmConsensusState =
    unionlabs::ibc::lightclients::wasm::consensus_state::ConsensusState<ConsensusState>;

pub struct TendermintLightClient;

impl IbcClient for TendermintLightClient {
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
                &consensus_state.data.root,
                &path,
                value,
            ),
            StorageState::Empty => ics23::ibc_api::verify_non_membership(
                &merkle_proof,
                &SDK_SPECS,
                &consensus_state.data.root,
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

        check_trusted_header(&header, &consensus_state.data.next_validators_hash)?;

        let revision_number = parse_revision_number(&header.signed_header.header.chain_id).ok_or(
            Error::InvalidChainId(header.signed_header.header.chain_id.clone()),
        )?;

        if revision_number != header.trusted_height.revision_number {
            return Err(Error::RevisionNumberMismatch {
                trusted_rn: revision_number,
                header_rn: header.trusted_height.revision_number,
            });
        }

        let signed_height = header
            .signed_header
            .header
            .height
            .inner()
            .try_into()
            .map_err(|_| Error::InvalidHeight)?;
        if signed_height <= header.trusted_height.revision_height {
            return Err(InvalidHeaderError::SignedHeaderHeightMustBeMoreRecent {
                signed_height,
                trusted_height: header.trusted_height.revision_height,
            }
            .into());
        }

        tendermint_verifier::verify::verify(
            &construct_partial_header(
                client_state.data.chain_id,
                TryInto::<i64>::try_into(header.trusted_height.revision_height)
                    .unwrap()
                    .try_into()
                    .unwrap(),
                consensus_state.data.timestamp,
                consensus_state.data.next_validators_hash,
            ),
            &header.trusted_validators,
            &header.signed_header,
            &header.validator_set,
            client_state.data.trusting_period,
            env.block.time.try_into().unwrap(),
            client_state.data.max_clock_drift,
            client_state.data.trust_level,
            &SignatureVerifier::new(Ed25519Verifier::new(deps)),
        )?;

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
        let update_height = height_from_header(&header);
        if read_consensus_state::<_, ConsensusState>(deps.as_ref(), &update_height)?.is_some() {
            return Ok(vec![update_height]);
        }

        // TODO(aeryz): prune oldest expired consensus state

        let mut client_state: WasmClientState = read_client_state(deps.as_ref())?;

        if update_height > client_state.latest_height {
            client_state.latest_height = update_height;
            client_state.data.latest_height = update_height;
        }

        save_client_state(deps.branch(), client_state);
        save_consensus_state_metadata(
            deps.branch(),
            header.signed_header.header.time,
            update_height,
        );
        save_consensus_state(
            deps,
            WasmConsensusState {
                data: ConsensusState {
                    timestamp: header.signed_header.header.time,
                    root: MerkleRoot {
                        hash: header.signed_header.header.app_hash,
                    },
                    next_validators_hash: header.signed_header.header.next_validators_hash,
                },
            },
            &update_height,
        );

        Ok(vec![update_height])
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
        let height = height_from_header(&header);

        // If there is already a header at this height, it should be exactly the same as the header that
        // we saved previously. If this is not the case, either the client is broken or the chain is
        // broken. Because it should not be possible to have two distinct valid headers at a height.
        if let Some(WasmConsensusState {
            data:
                ConsensusState {
                    timestamp,
                    next_validators_hash,
                    root: MerkleRoot { hash },
                },
        }) = read_consensus_state::<_, ConsensusState>(deps, &height)?
        {
            if timestamp != header.signed_header.header.time
                || hash != header.signed_header.header.app_hash
                || next_validators_hash != header.signed_header.header.next_validators_hash
            {
                return Ok(true);
            }

            // We don't need to check for previous or next consensus state since we know that we already
            // saved this header correctly previously.
            return Ok(false);
        }

        if let Ok(Some((_, next_consensus_state))) = get_or_next_consensus_state_meta(deps, height)
        {
            // next (in terms of height) consensus state must have a larger timestamp
            if next_consensus_state.timestamp <= header.signed_header.header.time {
                return Ok(true);
            }
        }

        if let Ok(Some((_, prev_consensus_state))) = get_or_prev_consensus_state_meta(deps, height)
        {
            // previous (in terms of height) consensus state must have a smaller timestamp
            if prev_consensus_state.timestamp >= header.signed_header.header.time {
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
            &consensus_state.data.timestamp,
            client_state.data.trusting_period,
            env.block
                .time
                .try_into()
                .map_err(|_| Error::InvalidHostTimestamp(env.block.time))?,
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
        let timestamp = read_consensus_state::<Self::CustomQuery, ConsensusState>(deps, &height)?
            .ok_or(Error::ConsensusStateNotFound(height))?
            .data
            .timestamp
            .seconds
            .inner();
        Ok(timestamp
            .try_into()
            .map_err(|_| Error::NegativeTimestamp(timestamp))?)
    }
}

fn construct_partial_header(
    chain_id: String,
    height: BoundedI64<0, { i64::MAX }>,
    time: Timestamp,
    next_validators_hash: H256,
) -> SignedHeader {
    SignedHeader {
        header: unionlabs::tendermint::types::header::Header {
            chain_id,
            time,
            next_validators_hash,
            height,
            version: Default::default(),
            last_block_id: Default::default(),
            last_commit_hash: Default::default(),
            data_hash: Default::default(),
            validators_hash: Default::default(),
            consensus_hash: Default::default(),
            app_hash: Default::default(),
            last_results_hash: Default::default(),
            evidence_hash: Default::default(),
            proposer_address: Default::default(),
        },
        commit: Commit {
            height,
            round: 0.try_into().expect("impossible"),
            block_id: Default::default(),
            signatures: Default::default(),
        },
    }
}

fn is_client_expired(
    consensus_state_timestamp: &Timestamp,
    trusting_period: Duration,
    current_block_time: Timestamp,
) -> bool {
    if let Some(sum) = consensus_state_timestamp.checked_add(trusting_period) {
        sum < current_block_time
    } else {
        true
    }
}

/// Returns the height from the update data
///
/// `header.signed_header.header.height` is `u64` and it does not contain the
/// revision height. This function is a utility to generate a `Height` type out
/// of the update data.
fn height_from_header(header: &Header) -> Height {
    Height {
        revision_number: header.trusted_height.revision_number,
        // SAFETY: height's bounds are [0..i64::MAX]
        revision_height: header.signed_header.header.height.inner() as u64,
    }
}

fn check_trusted_header(header: &Header, next_validators_hash: &H256) -> Result<(), Error> {
    let valhash = tendermint_verifier::utils::validators_hash(&header.trusted_validators);

    if &valhash != next_validators_hash {
        Err(Error::TrustedValidatorsMismatch(
            valhash,
            next_validators_hash.clone(),
        ))
    } else {
        Ok(())
    }
}

fn parse_revision_number(chain_id: &str) -> Option<u64> {
    chain_id
        .split('-')
        .last()
        .map(|height_str| height_str.parse().ok())?
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
    use crate::zkp_verifier::MockZKPVerifier;

    const UPDATES_DIR_PATH: &str = "src/test/updates/";

    const INITIAL_CONSENSUS_STATE_HEIGHT: Height = Height {
        revision_number: 1,
        revision_height: 1124,
    };

    type MockLightClient = CometblsLightClient<MockZKPVerifier>;

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
        assert!(is_client_expired(1, 1, 10));
        assert!(!is_client_expired(1, 10, 10));
        assert!(!is_client_expired(10, 1, 10));
        // expires when a + b = c
        assert!(!is_client_expired(5, 5, 10));
        // expires when a + b overflows
        assert!(is_client_expired(u64::MAX, 5, 10));
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

            assert!(!MockLightClient::check_for_misbehaviour_on_header(
                deps.as_ref(),
                update.clone()
            )
            .unwrap());
            MockLightClient::verify_header(deps.as_ref(), env.clone(), update.clone()).unwrap();
            MockLightClient::update_state(deps.as_mut(), env, update.clone()).unwrap();

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
            MockLightClient::verify_header(deps.as_ref(), env, update),
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
            MockLightClient::verify_header(deps.as_ref(), env, update),
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
            MockLightClient::verify_header(deps.as_ref(), env, update),
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
            MockLightClient::verify_header(deps.as_ref(), env, update),
            Err(Error::InvalidHeader(InvalidHeaderError::HeaderExpired(_)))
        ));
    }

    #[test]
    fn verify_header_fails_when_signed_header_exceeds_max_clock_drift() {
        let (mut deps, correct_update, mut env) = prepare_test_data();

        let mut update = correct_update.clone();
        update.signed_header.header.time.seconds = TIMESTAMP_SECONDS_MAX.try_into().unwrap();

        assert!(matches!(
            MockLightClient::verify_header(deps.as_ref(), env.clone(), update),
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
            MockLightClient::verify_header(deps.as_ref(), env, correct_update),
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
            MockLightClient::verify_header(deps.as_ref(), env.clone(), update),
            Err(Error::InvalidHeader(
                InvalidHeaderError::SignedHeaderMismatchWithCommitHash { .. }
            ))
        ));

        let mut update = correct_update.clone();
        update.signed_header.header.chain_id = "foobar".to_string();

        assert!(matches!(
            MockLightClient::verify_header(deps.as_ref(), env, update),
            Err(Error::InvalidHeader(
                InvalidHeaderError::SignedHeaderMismatchWithCommitHash { .. }
            ))
        ));
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
            MockLightClient::status(deps.as_ref(), &env),
            Ok(Status::Active)
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
            MockLightClient::status(deps.as_ref(), &mock_env()),
            Ok(Status::Expired)
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
            MockLightClient::status(deps.as_ref(), &env),
            Ok(Status::Expired)
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
            MockLightClient::status(deps.as_ref(), &mock_env()),
            Ok(Status::Frozen),
        );
    }
}

use cosmwasm_std::{Deps, DepsMut, Empty, Env};
use ics008_wasm_client::{
    storage_utils::{
        read_client_state, read_consensus_state, read_subject_client_state,
        read_substitute_client_state, read_substitute_consensus_state, save_client_state,
        save_consensus_state, save_subject_client_state, save_subject_consensus_state,
    },
    IbcClient, Status, StorageState, ZERO_HEIGHT,
};
use ics23::ibc_api::SDK_SPECS;
use tendermint_verifier::{
    types::SignatureVerifier,
    verify::{verify_commit_light, verify_commit_light_trusting},
};
use unionlabs::{
    bounded::BoundedI64,
    encoding::Proto,
    ensure,
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
            misbehaviour::Misbehaviour,
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

    type Misbehaviour = Misbehaviour;

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

        // TODO(aeryz): delay period check

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
                i64::try_from(header.trusted_height.revision_height)
                    .map_err(|_| Error::InvalidHeight)? // TODO(aeryz): add context #1333
                    .try_into()
                    .map_err(|_| Error::InvalidHeight)?,
                consensus_state.data.timestamp,
                consensus_state.data.next_validators_hash,
            ),
            &header.trusted_validators,
            &header.signed_header,
            &header.validator_set,
            client_state.data.trusting_period,
            env.block
                .time
                .try_into()
                .map_err(|_| Error::InvalidHostTimestamp(env.block.time))?,
            client_state.data.max_clock_drift,
            client_state.data.trust_level,
            &SignatureVerifier::new(Ed25519Verifier::new(deps)),
        )?;

        Ok(())
    }

    fn verify_misbehaviour(
        deps: Deps<Self::CustomQuery>,
        env: Env,
        misbehaviour: Self::Misbehaviour,
    ) -> Result<(), Self::Error> {
        ensure(
            misbehaviour.header_1.trusted_height.revision_number != 0,
            Error::MisbehaviourZeroHeight,
        )?;

        ensure(
            misbehaviour.header_2.trusted_height.revision_number != 0,
            Error::MisbehaviourZeroHeight,
        )?;

        ensure(
            height_from_header(&misbehaviour.header_1)
                >= height_from_header(&misbehaviour.header_2),
            Error::InvalidHeaderOrdering,
        )?;

        // TODO(aeryz): do we need to do a sanity check on headers or is it done when doing verification?

        validate_commit_light(deps, &misbehaviour.header_1)?;
        validate_commit_light(deps, &misbehaviour.header_2)?;

        let client_state: WasmClientState = read_client_state(deps)?;
        let consensus_state_1: WasmConsensusState =
            read_consensus_state::<_, ConsensusState>(deps, &misbehaviour.header_1.trusted_height)?
                .ok_or(Error::ConsensusStateNotFound(
                    misbehaviour.header_1.trusted_height,
                ))?;

        let consensus_state_2: WasmConsensusState =
            read_consensus_state::<_, ConsensusState>(deps, &misbehaviour.header_1.trusted_height)?
                .ok_or(Error::ConsensusStateNotFound(
                    misbehaviour.header_1.trusted_height,
                ))?;

        let timestamp: Timestamp = env
            .block
            .time
            .try_into()
            .map_err(|_| Error::InvalidHostTimestamp(env.block.time))?;

        check_misbehaviour_header(
            deps,
            &client_state.data,
            &consensus_state_1.data,
            &misbehaviour.header_1,
            timestamp,
        )?;

        check_misbehaviour_header(
            deps,
            &client_state.data,
            &consensus_state_2.data,
            &misbehaviour.header_2,
            timestamp,
        )?;

        Ok(())
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
        Err(Error::Unimplemented)
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
        misbehaviour: Self::Misbehaviour,
    ) -> Result<bool, Self::Error> {
        if height_from_header(&misbehaviour.header_1) == height_from_header(&misbehaviour.header_2)
        {
            if misbehaviour.header_1.signed_header.commit.block_id.hash
                != misbehaviour.header_2.signed_header.commit.block_id.hash
            {
                return Ok(true);
            }
        } else if misbehaviour.header_1.signed_header.header.time
            > misbehaviour.header_2.signed_header.header.time
        {
            return Ok(true);
        }

        Ok(false)
    }

    fn verify_upgrade_and_update_state(
        _deps: DepsMut<Self::CustomQuery>,
        _upgrade_client_state: Self::ClientState,
        _upgrade_consensus_state: Self::ConsensusState,
        _proof_upgrade_client: Vec<u8>,
        _proof_upgrade_consensus_state: Vec<u8>,
    ) -> Result<(), Self::Error> {
        Err(Error::Unimplemented)
    }

    fn migrate_client_store(mut deps: DepsMut<Self::CustomQuery>) -> Result<(), Self::Error> {
        let subject_client_state: WasmClientState = read_subject_client_state(deps.as_ref())?;
        let substitute_client_state: WasmClientState = read_substitute_client_state(deps.as_ref())?;

        ensure(
            substitute_client_state
                .data
                .frozen_height
                .unwrap_or(ZERO_HEIGHT)
                == ZERO_HEIGHT,
            Error::SubstituteClientFrozen,
        )?;

        ensure(
            migrate_check_allowed_fields(&subject_client_state.data, &substitute_client_state.data),
            Error::MigrateFieldsChanged,
        )?;

        let substitute_consensus_state: WasmConsensusState =
            read_substitute_consensus_state(deps.as_ref(), &substitute_client_state.latest_height)?
                .ok_or(Error::ConsensusStateNotFound(
                    substitute_client_state.latest_height,
                ))?;

        save_consensus_state_metadata(
            deps.branch(),
            substitute_consensus_state.data.timestamp,
            substitute_client_state.latest_height,
        );

        save_subject_consensus_state(
            deps.branch(),
            substitute_consensus_state,
            &substitute_client_state.latest_height,
        );

        let scs = substitute_client_state.data;
        save_subject_client_state(
            deps,
            WasmClientState {
                data: ClientState {
                    chain_id: scs.chain_id,
                    trusting_period: scs.trusting_period,
                    latest_height: scs.latest_height,
                    frozen_height: None,
                    ..subject_client_state.data
                },
                checksum: subject_client_state.checksum,
                latest_height: scs.latest_height,
            },
        );

        Ok(())
    }

    fn status(
        deps: Deps<Self::CustomQuery>,
        env: &cosmwasm_std::Env,
    ) -> Result<Status, Self::Error> {
        let client_state: WasmClientState = read_client_state(deps)?;

        // TODO(aeryz): when refactoring the tm client, we should consider making this non-optional
        // because otherwise we always have to check if the inner height is zero.
        if client_state.data.frozen_height.unwrap_or(ZERO_HEIGHT) != ZERO_HEIGHT {
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
        timestamp
            .try_into()
            .map_err(|_| Error::NegativeTimestamp(timestamp))
    }
}

fn validate_commit_light(deps: Deps, header: &Header) -> Result<(), Error> {
    verify_commit_light(
        &header.validator_set,
        &header.signed_header.header.chain_id,
        &header.signed_header.commit.block_id,
        header.signed_header.commit.height.inner(),
        &header.signed_header.commit,
        &SignatureVerifier::new(Ed25519Verifier::new(deps)),
    )?;

    Ok(())
}

fn check_misbehaviour_header(
    deps: Deps,
    client_state: &ClientState,
    consensus_state: &ConsensusState,
    header: &Header,
    current_timestamp: Timestamp,
) -> Result<(), Error> {
    check_trusted_header(header, &consensus_state.next_validators_hash)?;

    if current_timestamp
        >= consensus_state
            .timestamp
            .checked_add(client_state.trusting_period)
            .ok_or(Error::DurationAdditionOverflow)?
    {
        return Err(Error::TrustingPeriodExpired);
    }

    // TODO(aeryz): original implementation checks if the chain id is in the correct format
    // if not, it sets the revision number. Check why this is being done.

    verify_commit_light_trusting(
        &client_state.chain_id,
        &header.trusted_validators,
        &header.signed_header.commit,
        client_state.trust_level,
        &SignatureVerifier::new(Ed25519Verifier::new(deps)),
    )?;

    Ok(())
}

fn migrate_check_allowed_fields(
    subject_client_state: &ClientState,
    substitute_client_state: &ClientState,
) -> bool {
    subject_client_state.trust_level == substitute_client_state.trust_level
        && subject_client_state.unbonding_period == substitute_client_state.unbonding_period
        && subject_client_state.max_clock_drift == substitute_client_state.max_clock_drift
        && subject_client_state.proof_specs == substitute_client_state.proof_specs
        && subject_client_state.upgrade_path == substitute_client_state.upgrade_path
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
    let val_hash = tendermint_verifier::utils::validators_hash(&header.trusted_validators);

    if &val_hash != next_validators_hash {
        Err(Error::TrustedValidatorsMismatch(
            val_hash,
            next_validators_hash.clone(),
        ))
    } else {
        Ok(())
    }
}

fn parse_revision_number(chain_id: &str) -> Option<u64> {
    chain_id
        .rsplit('-')
        .next()
        .map(|height_str| height_str.parse().ok())?
}

#[cfg(test)]
mod tests {
    use std::fs;

    use cosmwasm_std::{
        testing::{mock_dependencies, MockApi, MockQuerier, MockStorage},
        OwnedDeps,
    };
    use ics008_wasm_client::{
        storage_utils::{
            consensus_db_key, read_subject_consensus_state, HOST_CLIENT_STATE_KEY,
            SUBJECT_CLIENT_STORE_PREFIX, SUBSTITUTE_CLIENT_STORE_PREFIX,
        },
        FROZEN_HEIGHT,
    };
    use unionlabs::{google::protobuf::any::Any, IntoProto};

    use super::*;

    const INITIAL_CONSENSUS_STATE_HEIGHT: Height = Height {
        revision_number: 1,
        revision_height: 10,
    };

    const INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT: Height = Height {
        revision_number: 1,
        revision_height: 12,
    };

    fn save_states_to_migrate_store(
        deps: DepsMut,
        subject_client_state: &WasmClientState,
        substitute_client_state: &WasmClientState,
        subject_consensus_state: &WasmConsensusState,
        substitute_consensus_state: &WasmConsensusState,
    ) {
        deps.storage.set(
            format!("{SUBJECT_CLIENT_STORE_PREFIX}{HOST_CLIENT_STATE_KEY}").as_bytes(),
            &Any(subject_client_state.clone()).into_proto_bytes(),
        );
        deps.storage.set(
            format!(
                "{SUBJECT_CLIENT_STORE_PREFIX}{}",
                consensus_db_key(&INITIAL_CONSENSUS_STATE_HEIGHT)
            )
            .as_bytes(),
            &Any(subject_consensus_state.clone()).into_proto_bytes(),
        );
        deps.storage.set(
            format!("{SUBSTITUTE_CLIENT_STORE_PREFIX}{HOST_CLIENT_STATE_KEY}").as_bytes(),
            &Any(substitute_client_state.clone()).into_proto_bytes(),
        );
        deps.storage.set(
            format!(
                "{SUBSTITUTE_CLIENT_STORE_PREFIX}{}",
                consensus_db_key(&INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT)
            )
            .as_bytes(),
            &Any(substitute_consensus_state.clone()).into_proto_bytes(),
        );
    }

    fn prepare_migrate_tests() -> (
        OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>,
        WasmClientState,
        WasmConsensusState,
        WasmClientState,
        WasmConsensusState,
    ) {
        (
            mock_dependencies(),
            serde_json::from_str(&fs::read_to_string("src/test/client_state.json").unwrap())
                .unwrap(),
            serde_json::from_str(&fs::read_to_string("src/test/consensus_state.json").unwrap())
                .unwrap(),
            serde_json::from_str(
                &fs::read_to_string("src/test/substitute_client_state.json").unwrap(),
            )
            .unwrap(),
            serde_json::from_str(
                &fs::read_to_string("src/test/substitute_consensus_state.json").unwrap(),
            )
            .unwrap(),
        )
    }

    #[test]
    fn migrate_client_store_works() {
        let (
            mut deps,
            mut wasm_client_state,
            wasm_consensus_state,
            substitute_wasm_client_state,
            substitute_wasm_consensus_state,
        ) = prepare_migrate_tests();

        wasm_client_state.data.frozen_height = Some(FROZEN_HEIGHT);

        save_states_to_migrate_store(
            deps.as_mut(),
            &wasm_client_state,
            &substitute_wasm_client_state,
            &wasm_consensus_state,
            &substitute_wasm_consensus_state,
        );

        TendermintLightClient::migrate_client_store(deps.as_mut()).unwrap();

        let wasm_client_state: WasmClientState = read_subject_client_state(deps.as_ref()).unwrap();
        // we didn't miss updating any fields
        assert_eq!(wasm_client_state, substitute_wasm_client_state);
        // client is unfrozen
        assert_eq!(wasm_client_state.data.frozen_height, None);

        // the new consensus state is saved under the correct height
        assert_eq!(
            read_subject_consensus_state(deps.as_ref(), &INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT)
                .unwrap()
                .unwrap(),
            substitute_wasm_consensus_state
        );

        // the new consensus state metadata is saved under substitute's latest height
        assert_eq!(
            get_or_next_consensus_state_meta(
                deps.as_ref(),
                INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT
            )
            .unwrap()
            .unwrap()
            .0,
            substitute_wasm_client_state.latest_height
        );
    }

    #[test]
    fn migrate_client_store_fails_when_invalid_change() {
        let (
            mut deps,
            wasm_client_state,
            wasm_consensus_state,
            substitute_wasm_client_state,
            substitute_wasm_consensus_state,
        ) = prepare_migrate_tests();

        macro_rules! modify_fns {
            ($param:ident, $($m:expr), + $(,)?) => ([$(|$param: &mut ClientState| $m),+])
        }

        let modifications = modify_fns! { s,
            s.trust_level.numerator ^= u64::MAX,
            s.unbonding_period = Duration::new(s.unbonding_period.seconds().inner() + 1, 0).unwrap(),
            s.max_clock_drift = Duration::new(s.max_clock_drift.seconds().inner() + 1, 0).unwrap(),
            s.proof_specs.push(s.proof_specs[0].clone()),
            s.upgrade_path.push(String::new()),

        };

        for m in modifications {
            let mut state = substitute_wasm_client_state.clone();
            m(&mut state.data);

            save_states_to_migrate_store(
                deps.as_mut(),
                &wasm_client_state,
                &state,
                &wasm_consensus_state,
                &substitute_wasm_consensus_state,
            );
            assert_eq!(
                TendermintLightClient::migrate_client_store(deps.as_mut()),
                Err(Error::MigrateFieldsChanged)
            );
        }
    }

    #[test]
    fn migrate_client_store_fails_when_substitute_client_frozen() {
        let (
            mut deps,
            wasm_client_state,
            wasm_consensus_state,
            mut substitute_wasm_client_state,
            substitute_wasm_consensus_state,
        ) = prepare_migrate_tests();

        substitute_wasm_client_state.data.frozen_height = Some(FROZEN_HEIGHT);

        save_states_to_migrate_store(
            deps.as_mut(),
            &wasm_client_state,
            &substitute_wasm_client_state,
            &wasm_consensus_state,
            &substitute_wasm_consensus_state,
        );

        assert_eq!(
            TendermintLightClient::migrate_client_store(deps.as_mut()),
            Err(Error::SubstituteClientFrozen)
        );
    }
}

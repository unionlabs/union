use cosmwasm_std::{Deps, DepsMut, Empty, Env};
use ics008_wasm_client::{
    storage_utils::{
        read_client_state, read_consensus_state, read_subject_client_state,
        read_substitute_client_state, read_substitute_consensus_state, save_client_state,
        save_consensus_state, save_subject_client_state, save_subject_consensus_state,
    },
    IbcClient, IbcClientError, Status, StorageState, WasmClientStateOf, WasmConsensusStateOf,
    ZERO_HEIGHT,
};
use ics23::ibc_api::SDK_SPECS;
use tendermint_verifier::types::SignatureVerifier;
use unionlabs::{
    bounded::BoundedI64,
    cometbft::types::{commit::Commit, signed_header::SignedHeader},
    encoding::{DecodeAs, Proto},
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
        },
    },
};

use crate::{
    errors::{
        Error, IbcHeightTooLargeForTendermintHeight, InvalidChainId, InvalidHeaderError,
        InvalidHostTimestamp, MathOverflow, MerkleProofDecode, MigrateClientStoreError,
        NegativeTimestamp, RevisionNumberMismatch, TrustedValidatorsMismatch,
    },
    storage::{
        get_current_or_next_consensus_state_meta, get_current_or_prev_consensus_state_meta,
        save_consensus_state_metadata,
    },
    verifier::Ed25519Verifier,
};

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
    ) -> Result<(), IbcClientError<Self>> {
        let consensus_state = read_consensus_state::<Self>(deps, &height)?
            .ok_or(IbcClientError::ConsensusStateNotFound(height))?;

        let merkle_proof = MerkleProof::decode_as::<Proto>(proof.as_ref())
            .map_err(|e| Error::from(MerkleProofDecode(e)))?;

        // TODO(aeryz): delay period check

        match value {
            StorageState::Occupied(value) => ics23::ibc_api::verify_membership(
                &merkle_proof,
                &SDK_SPECS,
                &consensus_state.data.root,
                &path
                    .key_path
                    .into_iter()
                    .map(|s| s.into_bytes())
                    .collect::<Vec<_>>(),
                value,
            ),
            StorageState::Empty => ics23::ibc_api::verify_non_membership(
                &merkle_proof,
                &SDK_SPECS,
                &consensus_state.data.root,
                &path
                    .key_path
                    .into_iter()
                    .map(|s| s.into_bytes())
                    .collect::<Vec<_>>(),
            ),
        }
        .map_err(Error::VerifyMembership)
        .map_err(Into::into)
    }

    fn verify_header(
        deps: Deps<Self::CustomQuery>,
        env: Env,
        mut header: Self::Header,
    ) -> Result<(), IbcClientError<Self>> {
        set_total_voting_power(&mut header.validator_set).map_err(Error::from)?;
        set_total_voting_power(&mut header.trusted_validators).map_err(Error::from)?;

        let client_state = read_client_state::<Self>(deps)?;
        let consensus_state = read_consensus_state::<Self>(deps, &header.trusted_height)?.ok_or(
            IbcClientError::ConsensusStateNotFound(header.trusted_height),
        )?;

        check_trusted_header(&header, &consensus_state.data.next_validators_hash)
            .map_err(Error::from)?;

        let revision_number = parse_revision_number(&header.signed_header.header.chain_id).ok_or(
            Error::from(InvalidChainId(header.signed_header.header.chain_id.clone())),
        )?;

        if revision_number != header.trusted_height.revision_number {
            return Err(Error::from(RevisionNumberMismatch {
                trusted_revision_number: revision_number,
                header_revision_number: header.trusted_height.revision_number,
            })
            .into());
        }

        let signed_height = header
            .signed_header
            .header
            .height
            .inner()
            .try_into()
            .expect("value is bounded >= 0; qed;");

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
                    .map_err(|_| {
                        Error::from(IbcHeightTooLargeForTendermintHeight(
                            header.trusted_height.revision_height,
                        ))
                    })?
                    .try_into()
                    .expect(
                        "value is converted from u64, which is positive, \
                        and the expected bounded type is >= 0; qed;",
                    ),
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
                .map_err(|_| Error::from(InvalidHostTimestamp(env.block.time)))?,
            client_state.data.max_clock_drift,
            &client_state.data.trust_level,
            &SignatureVerifier::new(Ed25519Verifier::new(deps)),
        )
        .map_err(Error::TendermintVerify)?;

        Ok(())
    }

    fn verify_misbehaviour(
        _deps: Deps<Self::CustomQuery>,
        _env: Env,
        _misbehaviour: Self::Misbehaviour,
    ) -> Result<(), IbcClientError<Self>> {
        Err(Error::Unimplemented.into())
    }

    fn update_state(
        mut deps: DepsMut<Self::CustomQuery>,
        _env: Env,
        header: Self::Header,
    ) -> Result<Vec<Height>, IbcClientError<Self>> {
        let update_height = height_from_header(&header);
        if read_consensus_state::<Self>(deps.as_ref(), &update_height)?.is_some() {
            return Ok(vec![update_height]);
        }

        // TODO(aeryz): prune oldest expired consensus state

        let mut client_state = read_client_state::<Self>(deps.as_ref())?;

        if update_height > client_state.latest_height {
            client_state.latest_height = update_height;
            client_state.data.latest_height = update_height;
        }

        save_client_state::<Self>(deps.branch(), client_state);
        save_consensus_state_metadata(
            deps.branch(),
            header.signed_header.header.time,
            update_height,
        );
        save_consensus_state::<Self>(
            deps,
            WasmConsensusStateOf::<Self> {
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
    ) -> Result<(), IbcClientError<Self>> {
        Err(Error::Unimplemented.into())
    }

    fn check_for_misbehaviour_on_header(
        deps: Deps<Self::CustomQuery>,
        header: Self::Header,
    ) -> Result<bool, IbcClientError<Self>> {
        let height = height_from_header(&header);

        // If there is already a header at this height, it should be exactly the same as the header that
        // we saved previously. If this is not the case, either the client is broken or the chain is
        // broken. Because it should not be possible to have two distinct valid headers at a height.
        if let Some(WasmConsensusStateOf::<Self> {
            data:
                ConsensusState {
                    timestamp,
                    next_validators_hash,
                    root: MerkleRoot { hash },
                },
        }) = read_consensus_state::<Self>(deps, &height)?
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

        if let Ok(Some((_, next_consensus_state))) =
            get_current_or_next_consensus_state_meta(deps, height)
        {
            // next (in terms of height) consensus state must have a larger timestamp
            if next_consensus_state.timestamp <= header.signed_header.header.time {
                return Ok(true);
            }
        }

        if let Ok(Some((_, prev_consensus_state))) =
            get_current_or_prev_consensus_state_meta(deps, height)
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
    ) -> Result<bool, IbcClientError<Self>> {
        Err(Error::Unimplemented.into())
    }

    fn verify_upgrade_and_update_state(
        _deps: DepsMut<Self::CustomQuery>,
        _upgrade_client_state: Self::ClientState,
        _upgrade_consensus_state: Self::ConsensusState,
        _proof_upgrade_client: Vec<u8>,
        _proof_upgrade_consensus_state: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>> {
        Err(Error::Unimplemented.into())
    }

    fn migrate_client_store(
        mut deps: DepsMut<Self::CustomQuery>,
    ) -> Result<(), IbcClientError<Self>> {
        let subject_client_state = read_subject_client_state::<Self>(deps.as_ref())?;
        let substitute_client_state = read_substitute_client_state::<Self>(deps.as_ref())?;

        ensure(
            substitute_client_state
                .data
                .frozen_height
                .unwrap_or(ZERO_HEIGHT)
                == ZERO_HEIGHT,
            MigrateClientStoreError::SubstituteClientFrozen,
        )?;

        ensure(
            migrate_check_allowed_fields(&subject_client_state.data, &substitute_client_state.data),
            MigrateClientStoreError::MigrateFieldsChanged,
        )?;

        let substitute_consensus_state: WasmConsensusStateOf<Self> =
            read_substitute_consensus_state(deps.as_ref(), &substitute_client_state.latest_height)?
                .ok_or(IbcClientError::ConsensusStateNotFound(
                    substitute_client_state.latest_height,
                ))?;

        save_consensus_state_metadata(
            deps.branch(),
            substitute_consensus_state.data.timestamp,
            substitute_client_state.latest_height,
        );

        save_subject_consensus_state::<Self>(
            deps.branch(),
            substitute_consensus_state,
            &substitute_client_state.latest_height,
        );

        let scs = substitute_client_state.data;
        save_subject_client_state::<Self>(
            deps,
            WasmClientStateOf::<Self> {
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
    ) -> Result<Status, IbcClientError<Self>> {
        let client_state = read_client_state::<Self>(deps)?;

        // TODO(aeryz): when refactoring the tm client, we should consider making this non-optional
        // because otherwise we always have to check if the inner height is zero.
        if client_state.data.frozen_height.unwrap_or(ZERO_HEIGHT) != ZERO_HEIGHT {
            return Ok(Status::Frozen);
        }

        let Some(consensus_state) =
            read_consensus_state::<Self>(deps, &client_state.latest_height)?
        else {
            return Ok(Status::Expired);
        };

        if is_client_expired(
            &consensus_state.data.timestamp,
            client_state.data.trusting_period,
            env.block
                .time
                .try_into()
                .map_err(|_| Error::from(InvalidHostTimestamp(env.block.time)))?,
        ) {
            return Ok(Status::Expired);
        }

        Ok(Status::Active)
    }

    fn export_metadata(
        _deps: Deps<Self::CustomQuery>,
        _env: &cosmwasm_std::Env,
    ) -> Result<Vec<GenesisMetadata>, IbcClientError<Self>> {
        Ok(Vec::new())
    }

    fn timestamp_at_height(
        deps: Deps<Self::CustomQuery>,
        height: Height,
    ) -> Result<u64, IbcClientError<Self>> {
        let timestamp = read_consensus_state::<Self>(deps, &height)?
            .ok_or(IbcClientError::ConsensusStateNotFound(height))?
            .data
            .timestamp
            .seconds
            .inner();

        timestamp
            .try_into()
            .map_err(|_| Error::from(NegativeTimestamp(timestamp)))
            .map_err(Into::into)
    }
}

pub fn set_total_voting_power(
    validator_set: &mut unionlabs::cometbft::types::validator_set::ValidatorSet,
) -> Result<(), MathOverflow> {
    validator_set.total_voting_power =
        validator_set
            .validators
            .iter()
            .try_fold(0_i64, |acc, val| {
                acc.checked_add(val.voting_power.inner())
                    .ok_or(MathOverflow)
            })?;
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

pub fn construct_partial_header(
    chain_id: String,
    height: BoundedI64<0, { i64::MAX }>,
    time: Timestamp,
    next_validators_hash: H256,
) -> SignedHeader {
    SignedHeader {
        header: unionlabs::cometbft::types::header::Header {
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

pub fn is_client_expired(
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
pub fn height_from_header(header: &Header) -> Height {
    Height {
        revision_number: header.trusted_height.revision_number,
        // SAFETY: height's bounds are [0..i64::MAX]
        revision_height: header.signed_header.header.height.inner() as u64,
    }
}

pub fn check_trusted_header(
    header: &Header,
    next_validators_hash: &H256,
) -> Result<(), TrustedValidatorsMismatch> {
    let val_hash = tendermint_verifier::utils::validators_hash(&header.trusted_validators);

    if &val_hash != next_validators_hash {
        Err(TrustedValidatorsMismatch(val_hash, *next_validators_hash))
    } else {
        Ok(())
    }
}

pub fn parse_revision_number(chain_id: &str) -> Option<u64> {
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
    use unionlabs::{encoding::EncodeAs, google::protobuf::any::Any};

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
        subject_client_state: &WasmClientStateOf<TendermintLightClient>,
        substitute_client_state: &WasmClientStateOf<TendermintLightClient>,
        subject_consensus_state: &WasmConsensusStateOf<TendermintLightClient>,
        substitute_consensus_state: &WasmConsensusStateOf<TendermintLightClient>,
    ) {
        deps.storage.set(
            format!("{SUBJECT_CLIENT_STORE_PREFIX}{HOST_CLIENT_STATE_KEY}").as_bytes(),
            &Any(subject_client_state.clone()).encode_as::<Proto>(),
        );
        deps.storage.set(
            format!(
                "{SUBJECT_CLIENT_STORE_PREFIX}{}",
                consensus_db_key(&INITIAL_CONSENSUS_STATE_HEIGHT)
            )
            .as_bytes(),
            &Any(subject_consensus_state.clone()).encode_as::<Proto>(),
        );
        deps.storage.set(
            format!("{SUBSTITUTE_CLIENT_STORE_PREFIX}{HOST_CLIENT_STATE_KEY}").as_bytes(),
            &Any(substitute_client_state.clone()).encode_as::<Proto>(),
        );
        deps.storage.set(
            format!(
                "{SUBSTITUTE_CLIENT_STORE_PREFIX}{}",
                consensus_db_key(&INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT)
            )
            .as_bytes(),
            &Any(substitute_consensus_state.clone()).encode_as::<Proto>(),
        );
    }

    #[allow(clippy::type_complexity)] // it's fine bro
    fn prepare_migrate_tests() -> (
        OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>,
        WasmClientStateOf<TendermintLightClient>,
        WasmConsensusStateOf<TendermintLightClient>,
        WasmClientStateOf<TendermintLightClient>,
        WasmConsensusStateOf<TendermintLightClient>,
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

        let wasm_client_state: WasmClientStateOf<TendermintLightClient> =
            read_subject_client_state::<TendermintLightClient>(deps.as_ref()).unwrap();
        // we didn't miss updating any fields
        assert_eq!(wasm_client_state, substitute_wasm_client_state);
        // client is unfrozen
        assert_eq!(wasm_client_state.data.frozen_height, None);

        // the new consensus state is saved under the correct height
        assert_eq!(
            read_subject_consensus_state::<TendermintLightClient>(
                deps.as_ref(),
                &INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT
            )
            .unwrap()
            .unwrap(),
            substitute_wasm_consensus_state
        );

        // the new consensus state metadata is saved under substitute's latest height
        assert_eq!(
            get_current_or_next_consensus_state_meta(
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
                Err(
                    Error::MigrateClientStore(MigrateClientStoreError::MigrateFieldsChanged).into()
                )
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
            Err(Error::MigrateClientStore(MigrateClientStoreError::SubstituteClientFrozen).into())
        );
    }
}

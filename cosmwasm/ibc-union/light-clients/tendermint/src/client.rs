use cometbft_types::{
    crypto::public_key::PublicKey,
    types::{commit::Commit, signed_header::SignedHeader, validator_set::ValidatorSet},
};
use cosmwasm_std::Empty;
use ibc_union_light_client::{IbcClient, IbcClientCtx, IbcClientError};
use ibc_union_msg::lightclient::{Status, VerifyCreationResponseEvent};
use ics23::ibc_api::SDK_SPECS;
use tendermint_light_client_types::{ClientState, ConsensusState, Header};
use tendermint_verifier::types::{HostFns, SignatureVerifier};
use unionlabs::{
    bounded::BoundedI64,
    encoding::Bincode,
    google::protobuf::{duration::Duration, timestamp::Timestamp},
    ibc::core::{
        client::height::Height,
        commitment::{merkle_proof::MerkleProof, merkle_root::MerkleRoot},
    },
    primitives::{encoding::HexUnprefixed, H256},
};

use crate::{
    errors::{
        Error, IbcHeightTooLargeForTendermintHeight, InvalidChainId, InvalidHeaderError,
        MathOverflow, RevisionNumberMismatch, TrustedValidatorsMismatch,
    },
    verifier::{Bls12Verifier, Ed25519Verifier},
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

    type StorageProof = MerkleProof;

    type Encoding = Bincode;

    fn verify_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
        value: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>> {
        let client_state = ctx.read_self_client_state()?;
        let consensus_state = ctx.read_self_consensus_state(height)?;

        verify_membership(
            &client_state.contract_address,
            &consensus_state.root,
            key,
            storage_proof,
            value,
        )?;

        Ok(())
    }

    fn verify_non_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
    ) -> Result<(), IbcClientError<Self>> {
        let client_state = ctx.read_self_client_state()?;
        let consensus_state = ctx.read_self_consensus_state(height)?;

        verify_non_membership(
            &client_state.contract_address,
            &consensus_state.root,
            key,
            storage_proof,
        )?;

        Ok(())
    }

    fn verify_header(
        ctx: IbcClientCtx<Self>,
        header: Self::Header,
        _caller: cosmwasm_std::Addr,
    ) -> Result<
        (u64, Self::ClientState, Self::ConsensusState),
        ibc_union_light_client::IbcClientError<Self>,
    > {
        let client_state = ctx.read_self_client_state()?;
        let consensus_state = ctx.read_self_consensus_state(header.trusted_height.height())?;
        match header.validator_set.validators.first().map(|v| &v.pub_key) {
            Some(PublicKey::Bls12_381(_)) => Ok(verify_header(
                client_state,
                consensus_state,
                header,
                ctx.env.block.time,
                &SignatureVerifier::new(Bls12Verifier::new(ctx.deps)),
            )?),
            Some(PublicKey::Ed25519(_)) => Ok(verify_header(
                client_state,
                consensus_state,
                header,
                ctx.env.block.time,
                &SignatureVerifier::new(Ed25519Verifier::new(ctx.deps)),
            )?),
            _ => Err(Error::InvalidValidatorSet.into()),
        }
    }

    fn misbehaviour(
        _ctx: IbcClientCtx<Self>,
        _misbehaviour: Self::Misbehaviour,
    ) -> Result<Self::ClientState, IbcClientError<Self>> {
        Err(Error::Unimplemented.into())
    }

    fn status(client_state: &Self::ClientState) -> Status {
        // FIXME: read latest consensus to verify if client expired
        // if is_client_expired(
        //     &consensus_state.timestamp,
        //     client_state.trusting_period,
        //     env.block
        //         .time
        //         .try_into()
        //         .map_err(|_| Error::from(InvalidHostTimestamp(env.block.time)))?,
        // ) {
        //     return Ok(Status::Expired);
        // }
        if client_state.frozen_height.unwrap_or_default().height() != 0 {
            Status::Frozen
        } else {
            Status::Active
        }
    }

    fn get_timestamp(consensus_state: &Self::ConsensusState) -> u64 {
        consensus_state.timestamp.as_unix_nanos()
    }

    fn get_latest_height(client_state: &Self::ClientState) -> u64 {
        client_state.latest_height.height()
    }

    fn get_counterparty_chain_id(client_state: &Self::ClientState) -> String {
        client_state.chain_id.clone()
    }

    fn verify_creation(
        _client_state: &Self::ClientState,
        _consensus_state: &Self::ConsensusState,
    ) -> Result<Option<Vec<VerifyCreationResponseEvent>>, IbcClientError<Self>> {
        Ok(None)
    }
}

pub fn verify_header<V: HostFns>(
    mut client_state: ClientState,
    consensus_state: ConsensusState,
    mut header: Header,
    block_timestamp: cosmwasm_std::Timestamp,
    signature_verifier: &SignatureVerifier<V>,
) -> Result<(u64, ClientState, ConsensusState), Error> {
    set_total_voting_power(&mut header.validator_set).map_err(Error::from)?;
    set_total_voting_power(&mut header.trusted_validators).map_err(Error::from)?;

    check_trusted_header(&header, consensus_state.next_validators_hash.as_encoding())
        .map_err(Error::from)?;

    let revision_number = parse_revision_number(&header.signed_header.header.chain_id).ok_or(
        Error::from(InvalidChainId(header.signed_header.header.chain_id.clone())),
    )?;

    if revision_number != header.trusted_height.revision() {
        return Err(Error::from(RevisionNumberMismatch {
            trusted_revision_number: revision_number,
            header_revision_number: header.trusted_height.revision(),
        }));
    }

    let signed_height = header
        .signed_header
        .header
        .height
        .inner()
        .try_into()
        .expect("value is bounded >= 0; qed;");

    if signed_height <= header.trusted_height.height() {
        return Err(InvalidHeaderError::SignedHeaderHeightMustBeMoreRecent {
            signed_height,
            trusted_height: header.trusted_height.height(),
        }
        .into());
    }

    // FIXME: unionlabs is tied to cosmwasm <2, the TryFrom impl can't be used
    let block_timestamp_proto = unionlabs::google::protobuf::timestamp::Timestamp {
        seconds: i64::try_from(block_timestamp.seconds())
            .expect("impossible")
            .try_into()
            .expect("impossible"),
        nanos: i32::try_from(block_timestamp.subsec_nanos())
            .expect("impossible")
            .try_into()
            .expect("impossible"),
    };

    tendermint_verifier::verify::verify(
        &construct_partial_header(
            client_state.chain_id.clone(),
            i64::try_from(header.trusted_height.height())
                .map_err(|_| {
                    Error::from(IbcHeightTooLargeForTendermintHeight(
                        header.trusted_height.height(),
                    ))
                })?
                .try_into()
                .expect(
                    "value is converted from u64, which is positive, \
                        and the expected bounded type is >= 0; qed;",
                ),
            consensus_state.timestamp,
            consensus_state.next_validators_hash,
        ),
        &header.trusted_validators,
        &header.signed_header,
        &header.validator_set,
        client_state.trusting_period,
        block_timestamp_proto,
        client_state.max_clock_drift,
        &client_state.trust_level,
        signature_verifier,
    )
    .map_err(Error::TendermintVerify)?;

    let update_height = header
        .signed_header
        .header
        .height
        .inner()
        .try_into()
        .expect("impossible");

    if client_state.latest_height.height() < update_height {
        *client_state.latest_height.height_mut() = update_height;
    }

    Ok((
        update_height,
        client_state,
        ConsensusState {
            timestamp: header.signed_header.header.time,
            root: MerkleRoot {
                hash: (*header.signed_header.header.app_hash.get()).into(),
            },
            next_validators_hash: header.signed_header.header.next_validators_hash,
        },
    ))
}

pub fn set_total_voting_power(validator_set: &mut ValidatorSet) -> Result<(), MathOverflow> {
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

pub fn construct_partial_header(
    chain_id: String,
    height: BoundedI64<0, { i64::MAX }>,
    time: Timestamp,
    next_validators_hash: H256<HexUnprefixed>,
) -> SignedHeader {
    SignedHeader {
        header: cometbft_types::types::header::Header {
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
    Height::new_with_revision(
        header.trusted_height.revision(),
        // SAFETY: height's bounds are [0..i64::MAX]
        header.signed_header.header.height.inner() as u64,
    )
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

pub fn verify_membership(
    contract_address: &H256,
    root: &MerkleRoot,
    key: Vec<u8>,
    storage_proof: MerkleProof,
    value: Vec<u8>,
) -> Result<(), Error> {
    ics23::ibc_api::verify_membership(
        &storage_proof,
        &SDK_SPECS,
        root,
        &[
            b"wasm".to_vec(),
            0x3u8
                .to_le_bytes()
                .into_iter()
                .chain(*contract_address)
                .chain(key)
                .collect::<Vec<_>>(),
        ],
        value,
    )
    .map_err(Error::VerifyMembership)
}

pub fn verify_non_membership(
    contract_address: &H256,
    root: &MerkleRoot,
    key: Vec<u8>,
    storage_proof: MerkleProof,
) -> Result<(), Error> {
    ics23::ibc_api::verify_non_membership(
        &storage_proof,
        &SDK_SPECS,
        root,
        &[
            b"wasm".to_vec(),
            0x3u8
                .to_le_bytes()
                .into_iter()
                .chain(*contract_address)
                .chain(key)
                .collect::<Vec<_>>(),
        ],
    )
    .map_err(Error::VerifyMembership)
}

// #[cfg(test)]
// mod tests {
//     use std::fs;

//     use cosmwasm_std::{
//         testing::{mock_dependencies, MockApi, MockQuerier, MockStorage},
//         OwnedDeps,
//     };
//     use ics008_wasm_client::{
//         storage_utils::{
//             consensus_db_key, read_subject_consensus_state, HOST_CLIENT_STATE_KEY,
//             SUBJECT_CLIENT_STORE_PREFIX, SUBSTITUTE_CLIENT_STORE_PREFIX,
//         },
//         FROZEN_HEIGHT,
//     };
//     use unionlabs::{encoding::EncodeAs, google::protobuf::any::Any};

//     use super::*;

//     const INITIAL_CONSENSUS_STATE_HEIGHT: Height = Height::new_with_revision(1, 10);

//     const INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT: Height = Height::new_with_revision(1, 12);

//     fn save_states_to_migrate_store(
//         deps: DepsMut,
//         subject_client_state: &WasmClientStateOf<TendermintLightClient>,
//         substitute_client_state: &WasmClientStateOf<TendermintLightClient>,
//         subject_consensus_state: &WasmConsensusStateOf<TendermintLightClient>,
//         substitute_consensus_state: &WasmConsensusStateOf<TendermintLightClient>,
//     ) {
//         deps.storage.set(
//             format!("{SUBJECT_CLIENT_STORE_PREFIX}{HOST_CLIENT_STATE_KEY}").as_bytes(),
//             &Any(subject_client_state.clone()).encode_as::<Proto>(),
//         );
//         deps.storage.set(
//             format!(
//                 "{SUBJECT_CLIENT_STORE_PREFIX}{}",
//                 consensus_db_key(&INITIAL_CONSENSUS_STATE_HEIGHT)
//             )
//             .as_bytes(),
//             &Any(subject_consensus_state.clone()).encode_as::<Proto>(),
//         );
//         deps.storage.set(
//             format!("{SUBSTITUTE_CLIENT_STORE_PREFIX}{HOST_CLIENT_STATE_KEY}").as_bytes(),
//             &Any(substitute_client_state.clone()).encode_as::<Proto>(),
//         );
//         deps.storage.set(
//             format!(
//                 "{SUBSTITUTE_CLIENT_STORE_PREFIX}{}",
//                 consensus_db_key(&INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT)
//             )
//             .as_bytes(),
//             &Any(substitute_consensus_state.clone()).encode_as::<Proto>(),
//         );
//     }

//     #[allow(clippy::type_complexity)] // it's fine bro
//     fn prepare_migrate_tests() -> (
//         OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>,
//         WasmClientStateOf<TendermintLightClient>,
//         WasmConsensusStateOf<TendermintLightClient>,
//         WasmClientStateOf<TendermintLightClient>,
//         WasmConsensusStateOf<TendermintLightClient>,
//     ) {
//         (
//             mock_dependencies(),
//             serde_json::from_str(&fs::read_to_string("src/test/client_state.json").unwrap())
//                 .unwrap(),
//             serde_json::from_str(&fs::read_to_string("src/test/consensus_state.json").unwrap())
//                 .unwrap(),
//             serde_json::from_str(
//                 &fs::read_to_string("src/test/substitute_client_state.json").unwrap(),
//             )
//             .unwrap(),
//             serde_json::from_str(
//                 &fs::read_to_string("src/test/substitute_consensus_state.json").unwrap(),
//             )
//             .unwrap(),
//         )
//     }

//     #[test]
//     fn migrate_client_store_works() {
//         let (
//             mut deps,
//             mut wasm_client_state,
//             wasm_consensus_state,
//             substitute_wasm_client_state,
//             substitute_wasm_consensus_state,
//         ) = prepare_migrate_tests();

//         wasm_client_state.frozen_height = Some(FROZEN_HEIGHT);

//         save_states_to_migrate_store(
//             deps.as_mut(),
//             &wasm_client_state,
//             &substitute_wasm_client_state,
//             &wasm_consensus_state,
//             &substitute_wasm_consensus_state,
//         );

//         TendermintLightClient::migrate_client_store(deps.as_mut()).unwrap();

//         let wasm_client_state: WasmClientStateOf<TendermintLightClient> =
//             read_subject_client_state::<TendermintLightClient>(deps.as_ref()).unwrap();
//         // we didn't miss updating any fields
//         assert_eq!(wasm_client_state, substitute_wasm_client_state);
//         // client is unfrozen
//         assert_eq!(wasm_client_state.frozen_height, None);

//         // the new consensus state is saved under the correct height
//         assert_eq!(
//             read_subject_consensus_state::<TendermintLightClient>(
//                 deps.as_ref(),
//                 &INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT
//             )
//             .unwrap()
//             .unwrap(),
//             substitute_wasm_consensus_state
//         );

//         // the new consensus state metadata is saved under substitute's latest height
//         assert_eq!(
//             get_current_or_next_consensus_state_meta(
//                 deps.as_ref(),
//                 INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT
//             )
//             .unwrap()
//             .unwrap()
//             .0,
//             substitute_wasm_client_state.latest_height
//         );
//     }

//     #[test]
//     fn migrate_client_store_fails_when_invalid_change() {
//         let (
//             mut deps,
//             wasm_client_state,
//             wasm_consensus_state,
//             substitute_wasm_client_state,
//             substitute_wasm_consensus_state,
//         ) = prepare_migrate_tests();

//         macro_rules! modify_fns {
//             ($param:ident, $($m:expr), + $(,)?) => ([$(|$param: &mut ClientState| $m),+])
//         }

//         let modifications = modify_fns! { s,
//             s.trust_level.numerator ^= u64::MAX,
//             s.unbonding_period = Duration::new(s.unbonding_period.seconds().inner() + 1, 0).unwrap(),
//             s.max_clock_drift = Duration::new(s.max_clock_drift.seconds().inner() + 1, 0).unwrap(),
//             s.proof_specs.push(s.proof_specs[0].clone()),
//             s.upgrade_path.push(String::new()),

//         };

//         for m in modifications {
//             let mut state = substitute_wasm_client_state.clone();
//             m(&mut state.data);

//             save_states_to_migrate_store(
//                 deps.as_mut(),
//                 &wasm_client_state,
//                 &state,
//                 &wasm_consensus_state,
//                 &substitute_wasm_consensus_state,
//             );
//             assert_eq!(
//                 TendermintLightClient::migrate_client_store(deps.as_mut()),
//                 Err(
//                     Error::MigrateClientStore(MigrateClientStoreError::MigrateFieldsChanged).into()
//                 )
//             );
//         }
//     }

//     #[test]
//     fn migrate_client_store_fails_when_substitute_client_frozen() {
//         let (
//             mut deps,
//             wasm_client_state,
//             wasm_consensus_state,
//             mut substitute_wasm_client_state,
//             substitute_wasm_consensus_state,
//         ) = prepare_migrate_tests();

//         substitute_wasm_client_state.frozen_height = Some(FROZEN_HEIGHT);

//         save_states_to_migrate_store(
//             deps.as_mut(),
//             &wasm_client_state,
//             &substitute_wasm_client_state,
//             &wasm_consensus_state,
//             &substitute_wasm_consensus_state,
//         );

//         assert_eq!(
//             TendermintLightClient::migrate_client_store(deps.as_mut()),
//             Err(Error::MigrateClientStore(MigrateClientStoreError::SubstituteClientFrozen).into())
//         );
//     }
// }

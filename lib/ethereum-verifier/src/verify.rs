use hash_db::HashDB;
use memory_db::{HashKey, MemoryDB};
use tree_hash::TreeHash;
use trie_db::{Trie, TrieDBBuilder};
use typenum::Unsigned;
use unionlabs::{
    bls::{BlsPublicKey, BlsSignature},
    ensure,
    ethereum::{
        config::{
            consts::{
                floorlog2, get_subtree_index, EXECUTION_PAYLOAD_INDEX, FINALIZED_ROOT_INDEX,
                NEXT_SYNC_COMMITTEE_INDEX,
            },
            ChainSpec, MIN_SYNC_COMMITTEE_PARTICIPANTS,
        },
        DomainType,
    },
    hash::{H160, H256},
    ibc::lightclients::ethereum::{
        execution_payload_header::CapellaExecutionPayloadHeader, fork_parameters::ForkParameters,
        light_client_header::LightClientHeader, light_client_update::LightClientUpdate,
    },
};

use crate::{
    primitives::{Account, GENESIS_SLOT},
    rlp_node_codec::{keccak_256, EthLayout, KeccakHasher},
    utils::{
        compute_domain, compute_epoch_at_slot, compute_fork_version, compute_signing_root,
        compute_sync_committee_period_at_slot, validate_merkle_branch,
    },
    Error, LightClientContext, ValidateLightClientError, VerifyAccountStorageRootError,
    VerifyStorageAbsenceError, VerifyStorageProofError,
};

type MinSyncCommitteeParticipants<Ctx> =
    <<Ctx as LightClientContext>::ChainSpec as MIN_SYNC_COMMITTEE_PARTICIPANTS>::MIN_SYNC_COMMITTEE_PARTICIPANTS;

pub trait BlsVerify {
    fn fast_aggregate_verify<'pk>(
        &self,
        public_keys: impl IntoIterator<Item = &'pk BlsPublicKey>,
        msg: Vec<u8>,
        signature: BlsSignature,
    ) -> Result<(), Error>;
}

/// Verifies if the light client `update` is valid.
///
/// * `update`: The light client update we want to verify.
/// * `current_slot`: The slot number at when the light client update is created.
/// * `genesis_validators_root`: The latest `genesis_validators_root` that is saved by the light client.
/// * `bls_verifier`: BLS verification implementation.
///
/// [See in consensus-spec](https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#validate_light_client_update)
pub fn validate_light_client_update<Ctx: LightClientContext, V: BlsVerify>(
    ctx: &Ctx,
    update: LightClientUpdate<Ctx::ChainSpec>,
    current_slot: u64,
    genesis_validators_root: H256,
    bls_verifier: V,
) -> Result<(), ValidateLightClientError> {
    // Verify sync committee has sufficient participants
    let sync_aggregate = &update.sync_aggregate;
    ensure(
        sync_aggregate.sync_committee_bits.num_set_bits()
            >= MinSyncCommitteeParticipants::<Ctx>::USIZE,
        Error::InsufficientSyncCommitteeParticipants(
            sync_aggregate.sync_committee_bits.num_set_bits(),
        ),
    )?;

    // Verify update does not skip a sync committee period
    is_valid_light_client_header(ctx.fork_parameters(), &update.attested_header)?;
    let update_attested_slot = update.attested_header.beacon.slot;
    let update_finalized_slot = update.finalized_header.beacon.slot;

    ensure(
        update_finalized_slot != GENESIS_SLOT,
        Error::FinalizedSlotIsGenesis,
    )?;

    ensure(
        current_slot >= update.signature_slot
            && update.signature_slot > update_attested_slot
            && update_attested_slot >= update_finalized_slot,
        Error::InvalidSlots,
    )?;

    let stored_period =
        compute_sync_committee_period_at_slot::<Ctx::ChainSpec>(ctx.finalized_slot());
    let signature_period =
        compute_sync_committee_period_at_slot::<Ctx::ChainSpec>(update.signature_slot);

    if ctx.next_sync_committee().is_some() {
        ensure(
            signature_period == stored_period || signature_period == stored_period + 1,
            Error::InvalidSignaturePeriodWhenNextSyncCommitteeExists {
                signature_period,
                stored_period,
            },
        )?;
    } else {
        ensure(
            signature_period == stored_period,
            Error::InvalidSignaturePeriodWhenNextSyncCommitteeDoesNotExist {
                signature_period,
                stored_period,
            },
        )?;
    }

    // Verify update is relevant
    let update_attested_period =
        compute_sync_committee_period_at_slot::<Ctx::ChainSpec>(update_attested_slot);

    ensure(
        update_attested_slot > ctx.finalized_slot()
            || (update_attested_period == stored_period
                && update.next_sync_committee.is_some()
                && ctx.next_sync_committee().is_none()),
        Error::IrrelevantUpdate,
    )?;

    // Verify that the `finality_branch`, if present, confirms `finalized_header`
    // to match the finalized checkpoint root saved in the state of `attested_header`.
    // NOTE(aeryz): We always expect to get `finalized_header` and it's embedded into the type definition.
    is_valid_light_client_header(ctx.fork_parameters(), &update.finalized_header)?;
    let finalized_root = update.finalized_header.beacon.tree_hash_root();

    validate_merkle_branch(
        &finalized_root.into(),
        &update.finality_branch,
        floorlog2(FINALIZED_ROOT_INDEX),
        get_subtree_index(FINALIZED_ROOT_INDEX),
        &update.attested_header.beacon.state_root,
    )?;

    // Verify that the `next_sync_committee`, if present, actually is the next sync committee saved in the
    // state of the `attested_header` if the store period is equal to the attested period
    if let (Some(next_sync_committee), Some(current_next_sync_committee)) =
        (&update.next_sync_committee, ctx.next_sync_committee())
    {
        if update_attested_period == stored_period {
            ensure(
                next_sync_committee == current_next_sync_committee,
                Error::NextSyncCommitteeMismatch {
                    expected: current_next_sync_committee.aggregate_pubkey.clone(),
                    got: next_sync_committee.aggregate_pubkey.clone(),
                },
            )?;
        }
        validate_merkle_branch(
            &next_sync_committee.tree_hash_root().into(),
            &update.next_sync_committee_branch.unwrap_or_default(),
            floorlog2(NEXT_SYNC_COMMITTEE_INDEX),
            get_subtree_index(NEXT_SYNC_COMMITTEE_INDEX),
            &update.attested_header.beacon.state_root,
        )?;
    }

    // Verify sync committee aggregate signature
    let sync_committee = if signature_period == stored_period {
        ctx.current_sync_committee()
            .ok_or(Error::ExpectedCurrentSyncCommittee)?
    } else {
        ctx.next_sync_committee()
            .ok_or(Error::ExpectedNextSyncCommittee)?
    };

    let participant_pubkeys = update
        .sync_aggregate
        .sync_committee_bits
        .iter()
        .zip(sync_committee.pubkeys.iter())
        .filter_map(|(included, pubkey)| included.then_some(pubkey))
        .collect::<Vec<_>>();

    let fork_version_slot = std::cmp::max(update.signature_slot, 1) - 1;
    let fork_version = compute_fork_version(
        ctx.fork_parameters(),
        compute_epoch_at_slot::<Ctx::ChainSpec>(fork_version_slot),
    );

    let domain = compute_domain(
        DomainType::SYNC_COMMITTEE,
        Some(fork_version),
        Some(genesis_validators_root),
        ctx.fork_parameters().genesis_fork_version.clone(),
    );
    let signing_root = compute_signing_root(&update.attested_header.beacon, domain);

    bls_verifier.fast_aggregate_verify(
        participant_pubkeys,
        signing_root.as_ref().to_owned(),
        sync_aggregate.sync_committee_signature.clone(),
    )?;

    Ok(())
}

fn verify_state(
    root: H256,
    key: impl AsRef<[u8]>,
    proof: impl IntoIterator<Item = impl AsRef<[u8]>>,
) -> Result<Option<Vec<u8>>, Error> {
    let mut db = MemoryDB::<KeccakHasher, HashKey<_>, Vec<u8>>::default();
    proof.into_iter().for_each(|n| {
        db.insert(hash_db::EMPTY_PREFIX, n.as_ref());
    });

    let root: primitive_types::H256 = root.into();
    let trie = TrieDBBuilder::<EthLayout>::new(&db, &root).build();
    // REVIEW: Is this arbitrary bytes?
    Ok(trie.get(&keccak_256(key.as_ref()))?)
}

/// Verifies if the `storage_root` of a contract can be verified against the state `root`.
///
/// * `root`: Light client update's (attested/finalized) execution block's state root.
/// * `address`: Address of the contract.
/// * `proof`: Proof of storage.
/// * `storage_root`: Storage root of the contract.
///
/// NOTE: You must not trust the `root` unless you verified it by calling [`validate_light_client_update`].
pub fn verify_account_storage_root(
    root: H256,
    address: &H160,
    proof: impl IntoIterator<Item = impl AsRef<[u8]>>,
    storage_root: &H256,
) -> Result<(), VerifyAccountStorageRootError> {
    match verify_state(root, address.as_ref(), proof)? {
        Some(account) => {
            let account = rlp::decode::<Account>(account.as_ref()).map_err(Error::RlpDecode)?;
            ensure(&account.storage_root == storage_root, Error::ValueMismatch)?;
            Ok(())
        }
        None => Err(Error::ValueMismatch)?,
    }
}

/// Verifies against `root`, if the `expected_value` is stored at `key` by using `proof`.
///
/// * `root`: Storage root of a contract.
/// * `key`: Padded slot number that the `expected_value` should be stored at.
/// * `expected_value`: Expected stored value.
/// * `proof`: Proof that is generated to prove the storage.
///
/// NOTE: You must not trust the `root` unless you verified it by calling [`verify_account_storage_root`].
pub fn verify_storage_proof(
    root: H256,
    key: H256,
    expected_value: &[u8],
    proof: impl IntoIterator<Item = impl AsRef<[u8]>>,
) -> Result<(), VerifyStorageProofError> {
    match verify_state(root, key.as_ref(), proof)? {
        Some(value) if value == expected_value => Ok(()),
        _ => Err(Error::ValueMismatch)?,
    }
}

/// Verifies against `root`, that no value is stored at `key` by using `proof`.
///
/// * `root`: Storage root of a contract.
/// * `key`: Padded slot number that the `expected_value` should be stored at.
/// * `proof`: Proof that is generated to prove the storage.
///
/// NOTE: You must not trust the `root` unless you verified it by calling [`verify_account_storage_root`].
pub fn verify_storage_absence(
    root: H256,
    key: H256,
    proof: impl IntoIterator<Item = impl AsRef<[u8]>>,
) -> Result<bool, VerifyStorageAbsenceError> {
    Ok(verify_state(root, key.as_ref(), proof)?.is_none())
}

/// Computes the execution block root hash.
///
/// [See in consensus-spec](https://github.com/ethereum/consensus-specs/blob/dev/specs/deneb/light-client/sync-protocol.md#modified-get_lc_execution_root)
pub fn get_lc_execution_root<C: ChainSpec>(
    fork_parameters: &ForkParameters,
    header: &LightClientHeader<C>,
) -> H256 {
    let epoch = compute_epoch_at_slot::<C>(header.beacon.slot);
    if epoch >= fork_parameters.deneb.epoch {
        return header.execution.tree_hash_root().into();
    }

    if epoch >= fork_parameters.capella.epoch {
        return CapellaExecutionPayloadHeader::from(header.execution.clone())
            .tree_hash_root()
            .into();
    }

    H256::default()
}

/// Validates a light client header.
///
/// [See in consensus-spec](https://github.com/ethereum/consensus-specs/blob/dev/specs/deneb/light-client/sync-protocol.md#modified-is_valid_light_client_header)
pub fn is_valid_light_client_header<C: ChainSpec>(
    fork_parameters: &ForkParameters,
    header: &LightClientHeader<C>,
) -> Result<(), Error> {
    let epoch = compute_epoch_at_slot::<C>(header.beacon.slot);

    if epoch < fork_parameters.deneb.epoch {
        ensure(
            header.execution.blob_gas_used == 0 && header.execution.excess_blob_gas == 0,
            Error::MustBeDeneb,
        )?;
    }

    ensure(
        epoch >= fork_parameters.capella.epoch,
        Error::InvalidChainVersion,
    )?;

    validate_merkle_branch(
        &get_lc_execution_root(fork_parameters, header),
        &header.execution_branch,
        floorlog2(EXECUTION_PAYLOAD_INDEX),
        get_subtree_index(EXECUTION_PAYLOAD_INDEX),
        &header.beacon.body_root,
    )
}

#[cfg(test)]
mod tests {
    use std::{cmp::Ordering, fs};

    use serde::Deserialize;
    use unionlabs::{
        ethereum::config::{Mainnet, SEPOLIA},
        ibc::lightclients::ethereum::sync_committee::SyncCommittee,
    };

    use super::*;

    #[derive(Debug, Clone)]
    struct Context {
        finalized_slot: u64,
        current_sync_committee: Option<SyncCommittee<Mainnet>>,
        next_sync_committee: Option<SyncCommittee<Mainnet>>,
    }

    #[derive(Deserialize)]
    struct InitialData {
        genesis_validators_root: H256,
        current_sync_committee: SyncCommittee<Mainnet>,
        next_sync_committee: SyncCommittee<Mainnet>,
    }

    #[derive(Deserialize)]
    struct Proof {
        pub storage_root: H256,
        #[serde(with = "::serde_utils::hex_string")]
        pub key: Vec<u8>,
        #[serde(with = "::serde_utils::hex_string")]
        pub value: Vec<u8>,
        #[serde(with = "::serde_utils::hex_string_list")]
        pub proof: Vec<Vec<u8>>,
    }

    lazy_static::lazy_static! {
        static ref VALID_PROOF: Proof = serde_json::from_str(&fs::read_to_string("src/test/state-proofs/valid_proof_1.json").unwrap()).unwrap();

        static ref ABSENT_PROOF: Proof = serde_json::from_str(&fs::read_to_string("src/test/state-proofs/absent_proof_1.json").unwrap()).unwrap();

        static ref INITIAL_DATA: InitialData = serde_json::from_str(&fs::read_to_string("src/test/initial_test_data.json").unwrap()).unwrap();

        static ref UPDATES: Vec<(Context, LightClientUpdate<Mainnet>)> = {
            // Read all the updates, only process files
            let mut updates: Vec<LightClientUpdate<Mainnet>> = fs::read_dir("src/test/updates/").unwrap().filter(|f|
                f.as_ref().unwrap().path().is_file()
            ).map(|f| {
                serde_json::from_str(&fs::read_to_string(f.unwrap().path()).unwrap()).unwrap()
            }).collect();

            // Sort the updates from oldest to most recent for us to do updates by iterating over
            updates.sort_by(|lhs, rhs| {
                if lhs.attested_header.beacon.slot > rhs.attested_header.beacon.slot {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            });

            // Since this verification library is stateless and it does not update any context after verifying an update,
            // we are manually doing it here.
            let mut current_sync_committee = Some(INITIAL_DATA.current_sync_committee.clone());
            let mut next_sync_committee= Some(INITIAL_DATA.next_sync_committee.clone());
            let mut update_data = vec![];
            updates.iter().enumerate().skip(1).for_each(|(i, update)|
                {
                    let current_update = &updates[i - 1];
                    let context = Context {
                        finalized_slot: current_update.attested_header.beacon.slot,
                        current_sync_committee: current_sync_committee.clone(),
                        next_sync_committee: next_sync_committee.clone(),
                    };
                    update_data.push((context, update.clone()));

                    // If the update contains a next sync committee, it means that we are moving to the next sync committee period
                    // and updating the next sync committee.
                    if let Some(ref nsc) = update.next_sync_committee {
                        current_sync_committee = next_sync_committee.take();
                        next_sync_committee = Some(nsc.clone());
                    }
                });

            update_data
        };
    }

    impl LightClientContext for Context {
        type ChainSpec = Mainnet;

        fn finalized_slot(&self) -> u64 {
            self.finalized_slot
        }

        fn current_sync_committee(&self) -> Option<&SyncCommittee<Self::ChainSpec>> {
            self.current_sync_committee.as_ref()
        }

        fn next_sync_committee(&self) -> Option<&SyncCommittee<Self::ChainSpec>> {
            self.next_sync_committee.as_ref()
        }

        fn fork_parameters(&self) -> &ForkParameters {
            &SEPOLIA.fork_parameters
        }
    }

    struct BlsVerifier;

    impl BlsVerify for BlsVerifier {
        fn fast_aggregate_verify<'pk>(
            &self,
            public_keys: impl IntoIterator<Item = &'pk BlsPublicKey>,
            msg: Vec<u8>,
            signature: BlsSignature,
        ) -> Result<(), Error> {
            let res = crate::crypto::fast_aggregate_verify_unchecked(
                public_keys.into_iter().collect::<Vec<_>>().as_slice(),
                msg.as_slice(),
                &signature,
            )
            .unwrap();

            if res {
                Ok(())
            } else {
                Err(Error::Crypto)
            }
        }
    }

    fn do_validate_light_client_update(
        ctx: &Context,
        update: LightClientUpdate<Mainnet>,
    ) -> Result<(), Error> {
        let attested_slot = update.attested_header.beacon.slot;
        validate_light_client_update(
            ctx,
            update,
            attested_slot + 32,
            INITIAL_DATA.genesis_validators_root.clone(),
            BlsVerifier,
        )
        .map_err(|e| e.0)
    }

    #[test]
    fn validate_light_client_update_works() {
        UPDATES.iter().for_each(|(ctx, update)| {
            assert_eq!(do_validate_light_client_update(ctx, update.clone()), Ok(()))
        });
    }

    #[test]
    fn validate_light_client_update_fails_when_insufficient_sync_committee_participants() {
        let (ctx, mut update) = UPDATES[0].clone();

        // Setting the sync committee bits to zero will result in no participants.
        update.sync_aggregate.sync_committee_bits = Default::default();

        assert!(matches!(
            do_validate_light_client_update(&ctx, update),
            Err(Error::InsufficientSyncCommitteeParticipants { .. })
        ));
    }

    #[test]
    fn validate_light_client_update_fails_when_invalid_header() {
        let (ctx, correct_update) = UPDATES[0].clone();

        let mut update = correct_update.clone();
        update.attested_header.execution.timestamp += 1;

        assert!(matches!(
            do_validate_light_client_update(&ctx, update),
            Err(Error::InvalidMerkleBranch(_))
        ));

        let mut update = correct_update;
        update.finalized_header.execution.timestamp += 1;

        assert!(matches!(
            do_validate_light_client_update(&ctx, update),
            Err(Error::InvalidMerkleBranch(_))
        ));
    }

    #[test]
    fn validate_light_client_update_fails_when_incorrect_slot_order() {
        let (ctx, correct_update) = UPDATES[0].clone();

        // signature slot can't be bigger than the current slot
        let mut update = correct_update.clone();
        update.signature_slot = u64::MAX;

        assert_eq!(
            do_validate_light_client_update(&ctx, update),
            Err(Error::InvalidSlots)
        );

        // attested slot can't be bigger than the signature slot
        let mut update = correct_update.clone();

        let before_deneb =
            SEPOLIA.fork_parameters.deneb.epoch * (SEPOLIA.preset.SLOTS_PER_EPOCH as u64) - 1;
        update.finalized_header.beacon.slot = before_deneb - 100;

        assert_eq!(
            do_validate_light_client_update(&ctx, update),
            Err(Error::InvalidSlots)
        );

        // finalized slot can't be bigger than the attested slot
        let mut update = correct_update;
        update.finalized_header.beacon.slot = before_deneb;

        assert_eq!(
            do_validate_light_client_update(&ctx, update),
            Err(Error::InvalidSlots)
        );
    }

    #[test]
    fn validate_light_client_update_fails_when_invalid_signature_period() {
        let (mut ctx, update) = UPDATES[0].clone();

        ctx.finalized_slot = u64::MAX;

        assert!(matches!(
            do_validate_light_client_update(&ctx, update.clone()),
            Err(Error::InvalidSignaturePeriodWhenNextSyncCommitteeExists { .. })
        ));

        // This should fail for both when the next sync committee exist and don't exist
        ctx.next_sync_committee = None;
        assert!(matches!(
            do_validate_light_client_update(&ctx, update),
            Err(Error::InvalidSignaturePeriodWhenNextSyncCommitteeDoesNotExist { .. })
        ));
    }

    #[test]
    fn validate_light_client_update_fails_when_irrelevant_update() {
        let (mut ctx, correct_update) = UPDATES
            .iter()
            .find(|(_, update)| update.next_sync_committee.is_some())
            .cloned()
            .unwrap()
            .clone();

        // Expected next sync committee since attested slot is not bigger than the stored slot.
        let mut update = correct_update.clone();
        update.next_sync_committee = None;
        ctx.finalized_slot = update.attested_header.beacon.slot;

        assert_eq!(
            do_validate_light_client_update(&ctx, update),
            Err(Error::IrrelevantUpdate)
        );

        // Expected stored next sync committee to be None
        assert_eq!(
            do_validate_light_client_update(&ctx, correct_update),
            Err(Error::IrrelevantUpdate)
        );
    }

    #[test]
    fn validate_light_client_update_fails_when_invalid_finality_branch() {
        let (ctx, mut update) = UPDATES[0].clone();

        update.finality_branch[0] = Default::default();

        assert!(matches!(
            do_validate_light_client_update(&ctx, update),
            Err(Error::InvalidMerkleBranch(_))
        ));
    }

    #[test]
    fn validate_light_client_update_fails_when_invalid_next_sync_committee_branch() {
        let (ctx, mut update) = UPDATES
            .iter()
            .find(|(_, update)| update.next_sync_committee.is_some())
            .cloned()
            .unwrap()
            .clone();

        update.next_sync_committee_branch = Some(Default::default());

        assert!(matches!(
            do_validate_light_client_update(&ctx, update),
            Err(Error::InvalidMerkleBranch(_))
        ));
    }

    #[test]
    fn verify_state_works() {
        assert_eq!(
            verify_state(
                VALID_PROOF.storage_root.clone(),
                &VALID_PROOF.key,
                VALID_PROOF.proof.iter()
            )
            .unwrap()
            .as_ref(),
            Some(VALID_PROOF.value.as_ref())
        );
    }

    #[test]
    fn verify_state_fails_when_invalid_root() {
        let storage_root = {
            let mut root = VALID_PROOF.storage_root.clone().into_bytes();
            root[0] = u8::MAX - root[0];
            root.try_into().unwrap()
        };

        assert!(matches!(
            verify_state(storage_root, &VALID_PROOF.key, VALID_PROOF.proof.iter()),
            Err(Error::Trie(_))
        ));
    }

    #[test]
    fn verify_state_returns_none_when_invalid_key() {
        let mut proof_key = VALID_PROOF.key.clone();
        proof_key[0] = u8::MAX - proof_key[0];

        assert_eq!(
            verify_state(
                VALID_PROOF.storage_root.clone(),
                &proof_key,
                VALID_PROOF.proof.iter()
            ),
            Ok(None)
        );
    }

    #[test]
    fn verify_state_fails_when_invalid_proof() {
        let mut proof = VALID_PROOF.proof.clone();
        proof[0][0] = u8::MAX - proof[0][0];

        assert!(matches!(
            verify_state(VALID_PROOF.storage_root.clone(), &VALID_PROOF.key, &proof),
            Err(Error::Trie(_))
        ));
    }

    #[test]
    fn verify_absent_storage_works() {
        assert_eq!(
            verify_storage_absence(
                ABSENT_PROOF.storage_root.clone(),
                ABSENT_PROOF.key.clone().try_into().unwrap(),
                ABSENT_PROOF.proof.iter()
            ),
            Ok(true)
        )
    }

    #[test]
    fn verify_absent_storage_returns_false_when_storage_exists() {
        assert_eq!(
            verify_storage_absence(
                VALID_PROOF.storage_root.clone(),
                VALID_PROOF.key.clone().try_into().unwrap(),
                VALID_PROOF.proof.iter()
            ),
            Ok(false)
        );
    }

    #[test]
    fn verify_storage_proof_works() {
        assert_eq!(
            verify_storage_proof(
                VALID_PROOF.storage_root.clone(),
                VALID_PROOF.key.clone().try_into().unwrap(),
                VALID_PROOF.value.as_slice(),
                VALID_PROOF.proof.iter()
            ),
            Ok(())
        );
    }

    #[test]
    fn verify_storage_proof_fails_when_incorrect_value() {
        let mut proof_value = VALID_PROOF.value.clone();
        proof_value[0] = u8::MAX - proof_value[0];

        assert_eq!(
            verify_storage_proof(
                VALID_PROOF.storage_root.clone(),
                VALID_PROOF.key.clone().try_into().unwrap(),
                proof_value.as_ref(),
                VALID_PROOF.proof.iter()
            ),
            Err(Error::ValueMismatch.into())
        );
    }

    #[test]
    fn is_valid_light_client_header_works() {
        UPDATES.iter().for_each(|(_, update)| {
            // Both finalized and attested headers should be verifiable
            assert_eq!(
                is_valid_light_client_header(&SEPOLIA.fork_parameters, &update.attested_header),
                Ok(()),
                "invalid attested header"
            );

            assert_eq!(
                is_valid_light_client_header(&SEPOLIA.fork_parameters, &update.finalized_header),
                Ok(()),
                "invalid finalized header"
            );
        });
    }
}

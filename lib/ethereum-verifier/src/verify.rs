use hash_db::HashDB;
use memory_db::{HashKey, MemoryDB};
use tree_hash::TreeHash;
use trie_db::{Trie, TrieDBBuilder};
use typenum::Unsigned;
use unionlabs::{
    bls::{BlsPublicKey, BlsSignature},
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
        fork_parameters::ForkParameters, light_client_header::LightClientHeader,
        light_client_update::LightClientUpdate,
    },
};

use crate::{
    primitives::Account,
    rlp_node_codec::{keccak_256, EthLayout, KeccakHasher},
    utils::{
        compute_domain, compute_epoch_at_slot, compute_fork_version, compute_signing_root,
        compute_sync_committee_period_at_slot, validate_merkle_branch,
    },
    Error, LightClientContext, ValidateLightClientError, VerifyAccountStorageRootError,
    VerifyStorageAbsenceError, VerifyStorageProofError,
};

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
    if sync_aggregate.sync_committee_bits.num_set_bits()
        < <Ctx::ChainSpec as MIN_SYNC_COMMITTEE_PARTICIPANTS>::MIN_SYNC_COMMITTEE_PARTICIPANTS::USIZE
    {
        Err(Error::InsufficientSyncCommitteeParticipants {
            min_limit: <Ctx::ChainSpec as MIN_SYNC_COMMITTEE_PARTICIPANTS>::MIN_SYNC_COMMITTEE_PARTICIPANTS::USIZE,
            participants:
            sync_aggregate.sync_committee_bits.num_set_bits()
        })?;
    }

    // Verify update does not skip a sync committee period
    is_valid_light_client_header(ctx.fork_parameters(), &update.attested_header)?;
    let update_attested_slot = update.attested_header.beacon.slot;
    let update_finalized_slot = update.finalized_header.beacon.slot;

    if !(current_slot >= update.signature_slot
        && update.signature_slot > update_attested_slot
        && update_attested_slot >= update_finalized_slot)
    {
        Err(Error::InvalidSlots)?;
    }

    let stored_period =
        compute_sync_committee_period_at_slot::<Ctx::ChainSpec>(ctx.finalized_slot());
    let signature_period =
        compute_sync_committee_period_at_slot::<Ctx::ChainSpec>(update.signature_slot);

    if ctx.next_sync_committee().is_some() {
        if signature_period != stored_period && signature_period != stored_period + 1 {
            Err(Error::InvalidSignaturePeriodWhenNextSyncCommitteeExists {
                signature_period,
                stored_period,
            })?;
        }
    } else if signature_period != stored_period {
        Err(
            Error::InvalidSignaturePeriodWhenNextSyncCommitteeDoesNotExist {
                signature_period,
                stored_period,
            },
        )?;
    }

    // Verify update is relevant
    let update_attested_period =
        compute_sync_committee_period_at_slot::<Ctx::ChainSpec>(update_attested_slot);

    if !(update_attested_slot > ctx.finalized_slot()
        || (update_attested_period == stored_period
            && update.next_sync_committee.is_some()
            && ctx.next_sync_committee().is_none()))
    {
        Err(Error::IrrelevantUpdate)?;
    }

    // Verify that the `finality_branch`, if present, confirms `finalized_header`
    // to match the finalized checkpoint root saved in the state of `attested_header`.
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
    if let Some(next_sync_committee) = &update.next_sync_committee {
        if let Some(current_next_sync_committee) = ctx.next_sync_committee() {
            if update_attested_period == stored_period
                && next_sync_committee != current_next_sync_committee
            {
                Err(Error::NextSyncCommitteeMismatch {
                    expected: current_next_sync_committee.aggregate_pubkey.clone(),
                    got: next_sync_committee.aggregate_pubkey.clone(),
                })?;
            }
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
            if &account.storage_root == storage_root {
                Ok(())
            } else {
                Err(Error::ValueMismatch)?
            }
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

/// Validates a light client header.
///
/// NOTE: This implementation is based on capella.
/// [See in consensus-spec](https://github.com/ethereum/consensus-specs/blob/dev/specs/capella/light-client/sync-protocol.md#modified-is_valid_light_client_header)
pub fn is_valid_light_client_header<C: ChainSpec>(
    fork_parameters: &ForkParameters,
    header: &LightClientHeader<C>,
) -> Result<(), Error> {
    let epoch = compute_epoch_at_slot::<C>(header.beacon.slot);

    if epoch < fork_parameters.capella.epoch {
        return Err(Error::InvalidChainVersion);
    }

    validate_merkle_branch(
        &H256::from(header.execution.tree_hash_root()),
        &header.execution_branch,
        floorlog2(EXECUTION_PAYLOAD_INDEX),
        get_subtree_index(EXECUTION_PAYLOAD_INDEX),
        &header.beacon.body_root,
    )
}

#[cfg(test)]
mod tests {
    use std::{cmp::Ordering, fs};

    use hex_literal::hex;
    use unionlabs::{
        ethereum::config::{Mainnet, Minimal, MINIMAL, SEPOLIA},
        ibc::lightclients::ethereum::{
            header::Header, sync_committee::SyncCommittee,
            trusted_sync_committee::ActiveSyncCommittee,
        },
    };

    use super::*;

    // curl "https://sepolia.cryptware.io/eth/v1/beacon/genesis"
    const GENESIS_VALIDATORS_ROOT: H256 = H256(hex!(
        "d8ea171f3c94aea21ebc42a1ed61052acf3f9209c00e4efbaaddac09ed9b8078"
    ));

    // These are copied from ethereum light client's tests.
    lazy_static::lazy_static! {
        static ref VALID_PROOF: Vec<Vec<u8>> = [
            "f871808080a0b9f6e8d11cf768b8034f04b8b2ab45bb5ca792e1c6e3929cf8222a885631ffac808080808080808080a0f7202a06e8dc011d3123f907597f51546fe03542551af2c9c54d21ba0fbafc7280a0d1797d071b81705da736e39e75f1186c8e529ba339f7a7d12a9b4fafe33e43cc80",
            "f842a03a8c7f353aebdcd6b56a67cd1b5829681a3c6e1695282161ab3faa6c3666d4c3a09f272c7c82ac0f0adbfe4ae30614165bf3b94d49754ce8c1955cc255dcc829b5"
        ].into_iter().map(|x| hex::decode(x).unwrap()).collect();

        static ref ABSENT_PROOF: Vec<Vec<u8>> = [
            hex::decode("f838a120df6966c971051c3d54ec59162606531493a51404a002842f56009d7e5cf4a8c79594be68fc2d8249eb60bfcf0e71d5a0d2f2e292c4ed").unwrap(),
        ].into();


        static ref UPDATES: Vec<(Context, LightClientUpdate<Mainnet>)> = {
            let mut updates = vec![];
            fs::read_dir("src/test/updates/").unwrap().into_iter().for_each(|f| {
                let entry = f.unwrap();
                if entry.path().file_name().is_some() {
                    let update: LightClientUpdate<Mainnet> = serde_json::from_str(&fs::read_to_string(entry.path()).unwrap()).unwrap();
                    updates.push(update);
                }
            });

            updates.sort_by(|lhs, rhs| {
                if lhs.attested_header.beacon.slot > rhs.attested_header.beacon.slot {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            });

            let mut current_sync_committee: ActiveSyncCommittee<Mainnet> = serde_json::from_str(&fs::read_to_string("src/test/initial_current_sync_committee.json").unwrap()).unwrap();
            let mut next_sync_committee: ActiveSyncCommittee<Mainnet> = serde_json::from_str(&fs::read_to_string("src/test/initial_next_sync_committee.json").unwrap()).unwrap();
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

                    if let Some(ref nsc) = update.next_sync_committee {
                        current_sync_committee = ActiveSyncCommittee::Current(next_sync_committee.get().clone());
                        next_sync_committee = ActiveSyncCommittee::Next(nsc.clone());
                    }
                });

            update_data
        };
    }

    const VALID_STORAGE_ROOT: H256 = H256(hex!(
        "5634f342b966b609cdd8d2f7ed43bb94702c9e83d4e974b08a3c2b8205fd85e3"
    ));
    const VALID_PROOF_KEY: H256 = H256(hex!(
        "b35cad2b263a62faaae30d8b3f51201fea5501d2df17d59a3eef2751403e684f"
    ));
    const VALID_RLP_ENCODED_PROOF_VALUE: H256 = H256(hex!(
        "9f272c7c82ac0f0adbfe4ae30614165bf3b94d49754ce8c1955cc255dcc829b5"
    ));

    const ABSENT_STORAGE_ROOT: H256 = H256(hex!(
        "9e352a10c5a38c301ee06c22a90f0971b679985b2ca6dd66aca224bd7a9957c1"
    ));

    const ABSENT_PROOF_KEY: H256 = H256(hex!(
        "aae81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b420"
    ));

    #[derive(Debug, Clone)]
    struct Context {
        finalized_slot: u64,
        current_sync_committee: ActiveSyncCommittee<Mainnet>,
        next_sync_committee: ActiveSyncCommittee<Mainnet>,
    }

    impl LightClientContext for Context {
        type ChainSpec = Mainnet;

        fn finalized_slot(&self) -> u64 {
            self.finalized_slot
        }

        fn current_sync_committee(&self) -> Option<&SyncCommittee<Self::ChainSpec>> {
            if let ActiveSyncCommittee::Current(committee) = &self.current_sync_committee {
                Some(committee)
            } else {
                None
            }
        }

        fn next_sync_committee(&self) -> Option<&SyncCommittee<Self::ChainSpec>> {
            if let ActiveSyncCommittee::Next(committee) = &self.next_sync_committee {
                Some(committee)
            } else {
                None
            }
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
            GENESIS_VALIDATORS_ROOT,
            BlsVerifier,
        )
        .map_err(|e| e.0)
    }

    #[test]
    fn validate_light_client_update_works() {
        UPDATES.iter().for_each(|(ctx, update)| {
            println!("Worked");
            validate_light_client_update(
                ctx,
                update.clone(),
                update.attested_header.beacon.slot + 32,
                GENESIS_VALIDATORS_ROOT,
                BlsVerifier,
            )
            .unwrap();
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
        update.attested_header.beacon.slot = u64::MAX - 100;

        assert_eq!(
            do_validate_light_client_update(&ctx, update),
            Err(Error::InvalidSlots)
        );

        // finalized slot can't be bigger than the attested slot
        let mut update = correct_update;
        update.finalized_header.beacon.slot = u64::MAX;

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
        ctx.next_sync_committee =
            ActiveSyncCommittee::Current(ctx.next_sync_committee.get().clone());
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
            .map(|u| u.clone())
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
            .map(|u| u.clone())
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
                VALID_STORAGE_ROOT.clone(),
                &VALID_PROOF_KEY,
                VALID_PROOF.iter()
            ),
            Ok(Some(VALID_RLP_ENCODED_PROOF_VALUE.into_bytes()))
        );
    }

    #[test]
    fn verify_state_fails_when_invalid_root() {
        let storage_root = {
            let mut root = VALID_STORAGE_ROOT.clone().into_bytes();
            root[0] = u8::MAX - root[0];
            root.try_into().unwrap()
        };

        assert!(matches!(
            verify_state(storage_root, &VALID_PROOF_KEY, VALID_PROOF.iter()),
            Err(Error::Trie(_))
        ));
    }

    #[test]
    fn verify_state_returns_none_when_invalid_key() {
        let mut proof_key = VALID_PROOF_KEY.clone();
        proof_key.0[0] = u8::MAX - proof_key.0[0];

        assert_eq!(
            verify_state(VALID_STORAGE_ROOT.clone(), &proof_key, VALID_PROOF.iter()),
            Ok(None)
        );
    }

    #[test]
    fn verify_state_fails_when_invalid_proof() {
        let mut proof = VALID_PROOF.clone();
        proof[0][0] = u8::MAX - proof[0][0];

        assert!(matches!(
            verify_state(VALID_STORAGE_ROOT.clone(), &VALID_PROOF_KEY, &proof),
            Err(Error::Trie(_))
        ));
    }

    #[test]
    fn verify_absent_storage_works() {
        assert_eq!(
            verify_storage_absence(ABSENT_STORAGE_ROOT, ABSENT_PROOF_KEY, ABSENT_PROOF.iter()),
            Ok(true)
        )
    }

    #[test]
    fn verify_absent_storage_returns_false_when_storage_exists() {
        assert_eq!(
            verify_storage_absence(VALID_STORAGE_ROOT, VALID_PROOF_KEY, VALID_PROOF.iter()),
            Ok(false)
        );
    }

    // #[test]
    // fn verify_account_storage_root_works() {
    //     let valid_header_data = read_valid_header_data();

    //     for header in valid_header_data {
    //         let header =
    //             <Header<Minimal>>::try_from_proto(serde_json::from_str(header).unwrap()).unwrap();

    //         let proof_data = header.account_update.proofs[0].clone();

    //         assert_eq!(
    //             verify_account_storage_root(
    //                 header.consensus_update.attested_header.execution.state_root,
    //                 &proof_data.key.as_slice().try_into().unwrap(),
    //                 &proof_data.proof,
    //                 &proof_data.value.as_slice().try_into().unwrap()
    //             ),
    //             Ok(())
    //         );
    //     }
    // }

    #[test]
    fn verify_storage_proof_works() {
        assert_eq!(
            verify_storage_proof(
                VALID_STORAGE_ROOT,
                VALID_PROOF_KEY,
                VALID_RLP_ENCODED_PROOF_VALUE.as_ref(),
                VALID_PROOF.iter()
            ),
            Ok(())
        );
    }

    #[test]
    fn verify_storage_proof_fails_when_incorrect_value() {
        let mut proof_value = VALID_RLP_ENCODED_PROOF_VALUE.clone();
        proof_value.0[0] = u8::MAX - proof_value.0[0];

        assert_eq!(
            verify_storage_proof(
                VALID_STORAGE_ROOT,
                VALID_PROOF_KEY,
                proof_value.as_ref(),
                VALID_PROOF.iter()
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

    /// https://github.com/unionlabs/union/issues/391
    #[test]
    fn multi_digit_base_fee_per_gas() {
        const JSON: &str = include_str!(
            "../../../light-clients/ethereum-light-client/src/test/multi_digit_base_fee_per_gas_header_update.json"
        );

        let header = serde_json::from_str::<Header<Minimal>>(JSON).unwrap();

        assert_eq!(
            is_valid_light_client_header(
                &MINIMAL.fork_parameters,
                &header.consensus_update.attested_header,
            ),
            Ok(()),
            "invalid attested header"
        );

        assert_eq!(
            is_valid_light_client_header(
                &MINIMAL.fork_parameters,
                &header.consensus_update.finalized_header,
            ),
            Ok(()),
            "invalid finalized header"
        );
    }
}

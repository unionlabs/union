use hash_db::HashDB;
use ibc_types::{
    bls::{BlsPublicKey, BlsSignature},
    ethereum::{DomainType, H256},
    ethereum_consts_traits::{
        consts::{
            floorlog2, get_subtree_index, EXECUTION_PAYLOAD_INDEX, FINALIZED_ROOT_INDEX,
            NEXT_SYNC_COMMITTEE_INDEX,
        },
        ChainSpec, MIN_SYNC_COMMITTEE_PARTICIPANTS,
    },
    ibc::lightclients::ethereum::{
        fork_parameters::ForkParameters, light_client_header::LightClientHeader,
        light_client_update::LightClientUpdate,
    },
};
use memory_db::{HashKey, MemoryDB};
use tree_hash::TreeHash;
use trie_db::{Trie, TrieDBBuilder};
use typenum::Unsigned;

use crate::{
    primitives::{Account, ExecutionAddress, Hash32, Slot},
    rlp_node_codec::{keccak_256, EthLayout, KeccakHasher},
    utils::{
        compute_domain, compute_epoch_at_slot, compute_fork_version, compute_signing_root,
        compute_sync_committee_period_at_slot, validate_merkle_branch,
    },
    Error, LightClientContext,
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
    current_slot: Slot,
    genesis_validators_root: H256,
    bls_verifier: V,
) -> Result<(), Error> {
    // Verify sync committee has sufficient participants
    let sync_aggregate = &update.sync_aggregate;
    if sync_aggregate.sync_committee_bits.num_set_bits()
        < <Ctx::ChainSpec as MIN_SYNC_COMMITTEE_PARTICIPANTS>::MIN_SYNC_COMMITTEE_PARTICIPANTS::USIZE
    {
        return Err(Error::InsufficientSyncCommitteeParticipants);
    }

    // Verify update does not skip a sync committee period
    println!("validating attested header");
    is_valid_light_client_header(ctx.fork_parameters(), &update.attested_header)?;
    let update_attested_slot = update.attested_header.beacon.slot;
    let update_finalized_slot = update.finalized_header.beacon.slot;

    if !(current_slot >= update.signature_slot
        && update.signature_slot > update_attested_slot
        && update_attested_slot >= update_finalized_slot)
    {
        return Err(Error::InvalidSlots);
    }

    let store_period =
        compute_sync_committee_period_at_slot::<Ctx::ChainSpec>(ctx.finalized_slot());
    let update_signature_period =
        compute_sync_committee_period_at_slot::<Ctx::ChainSpec>(update.signature_slot);

    if ctx.next_sync_committee().is_some() {
        if update_signature_period != store_period && update_signature_period != store_period + 1 {
            return Err(Error::InvalidSignaturePeriod);
        }
    } else if update_signature_period != store_period {
        return Err(Error::InvalidSignaturePeriod);
    }

    // Verify update is relevant
    let update_attested_period =
        compute_sync_committee_period_at_slot::<Ctx::ChainSpec>(update_attested_slot);

    if !(update_attested_slot > ctx.finalized_slot()
        || (update_attested_period == store_period
            && update.next_sync_committee.is_some()
            && ctx.next_sync_committee().is_none()))
    {
        return Err(Error::IrrelevantUpdate);
    }

    // Verify that the `finality_branch`, if present, confirms `finalized_header`
    // to match the finalized checkpoint root saved in the state of `attested_header`.
    println!("validating finalized header");
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
            if update_attested_period == store_period
                && next_sync_committee != current_next_sync_committee
            {
                return Err(Error::NextSyncCommitteeMismatch);
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
    let sync_committee = if update_signature_period == store_period {
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

fn verify_state(root: H256, key: &[u8], proof: &[Vec<u8>]) -> Result<Option<Vec<u8>>, Error> {
    let mut db = MemoryDB::<KeccakHasher, HashKey<_>, Vec<u8>>::default();
    proof.iter().for_each(|n| {
        db.insert(hash_db::EMPTY_PREFIX, n);
    });

    let root: primitive_types::H256 = root.into();
    let trie = TrieDBBuilder::<EthLayout>::new(&db, &root).build();
    Ok(trie.get(&keccak_256(key))?)
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
    address: &ExecutionAddress,
    proof: &[Vec<u8>],
    storage_root: &Hash32,
) -> Result<(), Error> {
    match verify_state(root, address.as_ref(), proof)? {
        Some(account) => {
            let account = Account::from_rlp_bytes(account.as_ref())?;
            if &account.storage_root == storage_root {
                Ok(())
            } else {
                Err(Error::ValueMismatch)
            }
        }
        None => Err(Error::ValueMismatch),
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
    key: &[u8],
    expected_value: &[u8],
    proof: &[Vec<u8>],
) -> Result<(), Error> {
    match verify_state(root, key, proof)? {
        Some(value) if value == expected_value => Ok(()),
        _ => Err(Error::ValueMismatch),
    }
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

    // println!("header.execution = {:#?}", header.execution);

    validate_merkle_branch(
        &H256::from(header.execution.tree_hash_root()),
        &header.execution_branch,
        floorlog2(EXECUTION_PAYLOAD_INDEX),
        EXECUTION_PAYLOAD_INDEX,
        &header.beacon.body_root,
    )
}

// TODO(aeryz): Don't forget to add negative cases.
#[cfg(test)]
mod tests {
    use ibc_types::{
        ethereum_consts_traits::{Minimal, MINIMAL},
        ibc::lightclients::ethereum::{header::Header, sync_committee::SyncCommittee},
        TryFromProto,
    };

    use super::*;

    const GENESIS_VALIDATORS_ROOT: &str =
        "270d43e74ce340de4bca2b1936beca0f4f5408d9e78aec4850920baf659d5b69";
    const GENESIS_TIME: u64 = 1686903632;

    // These are copied from ethereum light client's tests.
    lazy_static::lazy_static! {
        static ref VALID_STORAGE_ROOT: H256 =
            hex::decode("5634f342b966b609cdd8d2f7ed43bb94702c9e83d4e974b08a3c2b8205fd85e3").unwrap().try_into().unwrap();
        static ref VALID_PROOF_KEY: Vec<u8> =
            hex::decode("b35cad2b263a62faaae30d8b3f51201fea5501d2df17d59a3eef2751403e684f").unwrap();
        static ref VALID_RLP_ENCODED_PROOF_VALUE: Vec<u8> =
            hex::decode("9f272c7c82ac0f0adbfe4ae30614165bf3b94d49754ce8c1955cc255dcc829b5").unwrap();
        static ref VALID_PROOF: Vec<Vec<u8>> = [
            "f871808080a0b9f6e8d11cf768b8034f04b8b2ab45bb5ca792e1c6e3929cf8222a885631ffac808080808080808080a0f7202a06e8dc011d3123f907597f51546fe03542551af2c9c54d21ba0fbafc7280a0d1797d071b81705da736e39e75f1186c8e529ba339f7a7d12a9b4fafe33e43cc80",
            "f842a03a8c7f353aebdcd6b56a67cd1b5829681a3c6e1695282161ab3faa6c3666d4c3a09f272c7c82ac0f0adbfe4ae30614165bf3b94d49754ce8c1955cc255dcc829b5"
        ].into_iter().map(|x| hex::decode(x).unwrap()).collect();
    }

    struct Context {
        finalized_slot: u64,
        current_sync_committee: Option<SyncCommittee<Minimal>>,
        next_sync_committee: Option<SyncCommittee<Minimal>>,
    }

    impl LightClientContext for Context {
        type ChainSpec = Minimal;

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
            &MINIMAL.fork_parameters
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
            let res = crate::crypto::fast_aggregate_verify(
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

    fn read_valid_header_data() -> Vec<&'static str> {
        // TODO(aeryz): move test data to ibc types
        [
            include_str!(
                "../../../light-clients/ethereum-light-client/src/test/sync_committee_update_1.json"
            ),
            include_str!(
                "../../../light-clients/ethereum-light-client/src/test/finality_update_1.json"
            ),
            include_str!(
                "../../../light-clients/ethereum-light-client/src/test/sync_committee_update_2.json"
            ),
            include_str!(
                "../../../light-clients/ethereum-light-client/src/test/finality_update_2.json"
            ),
            include_str!(
                "../../../light-clients/ethereum-light-client/src/test/finality_update_3.json"
            ),
            include_str!(
                "../../../light-clients/ethereum-light-client/src/test/finality_update_4.json"
            ),
        ]
        .into()
    }

    fn do_validate_light_client_update(header: Header<Minimal>) -> Result<(), Error> {
        let genesis_validators_root: H256 = hex::decode(GENESIS_VALIDATORS_ROOT)
            .unwrap()
            .try_into()
            .unwrap();

        let sync_committee = header.trusted_sync_committee.sync_committee;
        let finalized_slot = header.trusted_sync_committee.trusted_height.revision_height;

        let ctx = if header.trusted_sync_committee.is_next {
            Context {
                finalized_slot,
                current_sync_committee: None,
                next_sync_committee: Some(sync_committee),
            }
        } else {
            Context {
                finalized_slot,
                current_sync_committee: Some(sync_committee),
                next_sync_committee: None,
            }
        };

        validate_light_client_update(
            &ctx,
            header.consensus_update,
            (header.timestamp - GENESIS_TIME) / MINIMAL.preset.SECONDS_PER_SLOT as u64,
            genesis_validators_root,
            BlsVerifier,
        )
    }

    #[test]
    fn validate_light_client_update_works() {
        let valid_header_data = read_valid_header_data();

        for header in valid_header_data {
            let header =
                <Header<Minimal>>::try_from_proto(serde_json::from_str(header).unwrap()).unwrap();

            assert_eq!(do_validate_light_client_update(header), Ok(()));
        }
    }

    #[test]
    fn validate_light_client_update_fails_when_insufficient_sync_committee_participants() {
        let mut header = <Header<Minimal>>::try_from_proto(
            serde_json::from_str(include_str!(
            "../../../light-clients/ethereum-light-client/src/test/sync_committee_update_1.json"
        ))
            .unwrap(),
        )
        .unwrap();

        // Setting the sync committee bits to zero will result in no participants.
        header.consensus_update.sync_aggregate.sync_committee_bits = Default::default();

        assert_eq!(
            do_validate_light_client_update(header),
            Err(Error::InsufficientSyncCommitteeParticipants)
        );
    }

    #[test]
    fn validate_light_client_update_fails_when_invalid_header() {
        let correct_header = <Header<Minimal>>::try_from_proto(
            serde_json::from_str(include_str!(
            "../../../light-clients/ethereum-light-client/src/test/sync_committee_update_1.json"
        ))
            .unwrap(),
        )
        .unwrap();

        let mut header = correct_header.clone();
        header.consensus_update.attested_header.execution.timestamp += 1;

        assert!(matches!(
            do_validate_light_client_update(header),
            Err(Error::InvalidMerkleBranch(_))
        ));

        let mut header = correct_header;
        header.consensus_update.finalized_header.execution.timestamp += 1;

        assert!(matches!(
            do_validate_light_client_update(header),
            Err(Error::InvalidMerkleBranch(_))
        ));
    }

    #[test]
    fn validate_light_client_update_fails_when_incorrect_slot_order() {
        let correct_header = <Header<Minimal>>::try_from_proto(
            serde_json::from_str(include_str!(
            "../../../light-clients/ethereum-light-client/src/test/sync_committee_update_1.json"
        ))
            .unwrap(),
        )
        .unwrap();

        // signature slot can't be bigger than the current slot
        let mut header = correct_header.clone();
        header.consensus_update.signature_slot = u64::MAX;

        assert_eq!(
            do_validate_light_client_update(header),
            Err(Error::InvalidSlots)
        );

        // attested slot can't be bigger than the signature slot
        let mut header = correct_header.clone();
        header.consensus_update.attested_header.beacon.slot = u64::MAX;

        assert_eq!(
            do_validate_light_client_update(header),
            Err(Error::InvalidSlots)
        );

        // finalized slot can't be bigger than the attested slot
        let mut header = correct_header;
        header.consensus_update.finalized_header.beacon.slot = u64::MAX;

        assert_eq!(
            do_validate_light_client_update(header),
            Err(Error::InvalidSlots)
        );
    }

    #[test]
    fn validate_light_client_update_fails_when_invalid_signature_period() {
        let mut header = <Header<Minimal>>::try_from_proto(
            serde_json::from_str(include_str!(
            "../../../light-clients/ethereum-light-client/src/test/sync_committee_update_1.json"
        ))
            .unwrap(),
        )
        .unwrap();

        header.trusted_sync_committee.trusted_height.revision_height = u64::MAX;

        assert_eq!(
            do_validate_light_client_update(header),
            Err(Error::InvalidSignaturePeriod)
        );

        // We do again here because we are expecting it to fail for both `is_next = true` and `is_next = false`
        let mut header = <Header<Minimal>>::try_from_proto(
            serde_json::from_str(include_str!(
                "../../../light-clients/ethereum-light-client/src/test/finality_update_1.json"
            ))
            .unwrap(),
        )
        .unwrap();

        header.trusted_sync_committee.trusted_height.revision_height = u64::MAX;

        assert_eq!(
            do_validate_light_client_update(header),
            Err(Error::InvalidSignaturePeriod)
        );
    }

    #[test]
    fn validate_light_client_update_fails_when_irrelevant_update() {
        let correct_header = <Header<Minimal>>::try_from_proto(
            serde_json::from_str(include_str!(
            "../../../light-clients/ethereum-light-client/src/test/sync_committee_update_1.json"
        ))
            .unwrap(),
        )
        .unwrap();

        // Expected next sync committee since attested slot is not bigger than the stored slot.
        let mut header = correct_header.clone();
        header.consensus_update.next_sync_committee = None;

        assert_eq!(
            do_validate_light_client_update(header),
            Err(Error::IrrelevantUpdate)
        );

        // Expected stored next sync committee to be None.
        let mut header = correct_header;
        header.trusted_sync_committee.is_next = true;

        assert_eq!(
            do_validate_light_client_update(header),
            Err(Error::IrrelevantUpdate)
        );
    }

    #[test]
    fn validate_light_client_update_fails_when_invalid_finality_branch() {
        let mut header = <Header<Minimal>>::try_from_proto(
            serde_json::from_str(include_str!(
            "../../../light-clients/ethereum-light-client/src/test/sync_committee_update_1.json"
        ))
            .unwrap(),
        )
        .unwrap();

        header.consensus_update.finality_branch[0] = Default::default();

        assert!(matches!(
            do_validate_light_client_update(header),
            Err(Error::InvalidMerkleBranch(_))
        ));
    }

    #[test]
    fn validate_light_client_update_fails_when_invalid_next_sync_committee_branch() {
        let mut header = <Header<Minimal>>::try_from_proto(
            serde_json::from_str(include_str!(
            "../../../light-clients/ethereum-light-client/src/test/sync_committee_update_1.json"
        ))
            .unwrap(),
        )
        .unwrap();

        header.consensus_update.next_sync_committee_branch = Some(Default::default());

        assert!(matches!(
            do_validate_light_client_update(header),
            Err(Error::InvalidMerkleBranch(_))
        ));
    }

    #[test]
    fn verify_state_works() {
        assert_eq!(
            verify_state(VALID_STORAGE_ROOT.clone(), &VALID_PROOF_KEY, &VALID_PROOF),
            Ok(Some(VALID_RLP_ENCODED_PROOF_VALUE.clone()))
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
            verify_state(storage_root, &VALID_PROOF_KEY, &VALID_PROOF),
            Err(Error::Trie(_))
        ));
    }

    #[test]
    fn verify_state_returns_none_when_invalid_key() {
        let mut proof_key = VALID_PROOF_KEY.clone();
        proof_key[0] = u8::MAX - proof_key[0];

        assert_eq!(
            verify_state(VALID_STORAGE_ROOT.clone(), &proof_key, &VALID_PROOF),
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
    fn verify_account_storage_root_works() {
        let valid_header_data = read_valid_header_data();

        for header in valid_header_data {
            let header =
                <Header<Minimal>>::try_from_proto(serde_json::from_str(header).unwrap()).unwrap();

            let proof_data = header.account_update.proofs[0].clone();

            assert_eq!(
                verify_account_storage_root(
                    header.consensus_update.attested_header.execution.state_root,
                    &proof_data.key.as_slice().try_into().unwrap(),
                    &proof_data.proof,
                    &proof_data.value.as_slice().try_into().unwrap()
                ),
                Ok(())
            );
        }
    }

    #[test]
    fn verify_storage_proof_works() {
        assert_eq!(
            verify_storage_proof(
                VALID_STORAGE_ROOT.clone(),
                &VALID_PROOF_KEY,
                &VALID_RLP_ENCODED_PROOF_VALUE,
                &VALID_PROOF
            ),
            Ok(())
        );
    }

    #[test]
    fn verify_storage_proof_fails_when_incorrect_value() {
        let mut proof_value = VALID_RLP_ENCODED_PROOF_VALUE.clone();
        proof_value[0] = u8::MAX - proof_value[0];

        assert_eq!(
            verify_storage_proof(
                VALID_STORAGE_ROOT.clone(),
                &VALID_PROOF_KEY,
                &proof_value,
                &VALID_PROOF
            ),
            Err(Error::ValueMismatch)
        );
    }

    #[test]
    fn is_valid_light_client_header_works() {
        let valid_header_data = read_valid_header_data();

        for header in valid_header_data {
            let header =
                <Header<Minimal>>::try_from_proto(serde_json::from_str(header).unwrap()).unwrap();

            // Both finalized and attested headers should be verifiable
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
}

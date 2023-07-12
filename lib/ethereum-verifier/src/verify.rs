use hash_db::HashDB;
use ibc_types::{
    bls::{BlsPublicKey, BlsSignature},
    ethereum::{DomainType, H256},
    ethereum_consts_traits::{
        consts::{
            floorlog2, get_subtree_index, EXECUTION_PAYLOAD_INDEX, FINALIZED_ROOT_INDEX,
            NEXT_SYNC_COMMITTEE_INDEX,
        },
        MIN_SYNC_COMMITTEE_PARTICIPANTS,
    },
    ibc::lightclients::ethereum::{
        light_client_header::LightClientHeader, light_client_update::LightClientUpdate,
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
            let account = Account::from_rlp_bytes(account.as_ref()).unwrap();
            if account.storage_root == *storage_root {
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
pub fn is_valid_light_client_header<CS: ChainSpec>(
    fork_parameters: &ForkParameters,
    header: &LightClientHeader<CS>,
) -> Result<(), Error> {
    let epoch = compute_epoch_at_slot::<CS>(header.beacon.slot);

    if epoch < fork_parameters.capella.epoch {
        return Err(Error::InvalidChainVersion);
    }

    validate_merkle_branch(
        &H256::from(header.execution.tree_hash_root()),
        &header.execution_branch,
        floorlog2(EXECUTION_PAYLOAD_INDEX),
        EXECUTION_PAYLOAD_INDEX,
        &header.beacon.body_root,
    )
}

#[cfg(test)]
mod tests {
    use ibc_types::{
        ethereum_consts_traits::{Minimal, MINIMAL},
        ibc::lightclients::ethereum::{header::Header, sync_committee::SyncCommittee},
        TryFromProto,
    };

    use super::*;

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
                "../../../light-clients/ethereum-light-client/src/test/finality_update_1.json"
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

    #[test]
    fn validate_light_client_update_works() {
        let valid_header_data = read_valid_header_data();

        for header in valid_header_data {
            let header =
                <Header<Minimal>>::try_from_proto(serde_json::from_str(header).unwrap()).unwrap();

            // TODO(aeryz): Dynamic context based on the update
            let ctx = Context {
                finalized_slot: 1,
                current_sync_committee: None,
                next_sync_committee: None,
            };

            assert_eq!(
                validate_light_client_update(
                    &ctx,
                    header.consensus_update,
                    0,
                    Default::default(),
                    BlsVerifier,
                ),
                Ok(())
            );
        }
    }

    #[test]
    fn verify_state_works() {
        // TODO(aeryz): We already define this test in the light client, extract out the helper functions for
        // the test so that we don't have to define everything twice.
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
        // TODO(aeryz): We already define this test in the light client, extract out the helper functions for
        // the test so that we don't have to define everything twice.
    }

    #[test]
    fn is_valid_light_client_header_works() {
        let valid_header_data = read_valid_header_data();

        for header in valid_header_data {
            let header =
                <Header<Minimal>>::try_from_proto(serde_json::from_str(header).unwrap()).unwrap();

            // Both finalized and attested headers should be verifyable
            assert_eq!(
                is_valid_light_client_header(
                    &MINIMAL.fork_parameters,
                    &header.consensus_update.attested_header,
                ),
                Ok(())
            );

            assert_eq!(
                is_valid_light_client_header(
                    &MINIMAL.fork_parameters,
                    &header.consensus_update.finalized_header,
                ),
                Ok(())
            );
        }
    }
}

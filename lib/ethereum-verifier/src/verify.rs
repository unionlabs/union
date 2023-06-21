use crate::{
    capella::{LightClientHeader, LightClientUpdate, NEXT_SYNC_COMMITTEE_INDEX_FLOOR_LOG_2},
    primitives::{Account, DomainType, ExecutionAddress, Hash32, Root, Slot},
    rlp_node_codec::{keccak_256, EthLayout, KeccakHasher},
    utils::*,
    Error, LightClientContext, BYTES_PER_LOGS_BLOOM, EXECUTION_PAYLOAD_DEPTH,
    EXECUTION_PAYLOAD_INDEX, FINALIZED_ROOT_DEPTH, FINALIZED_ROOT_SUBTREE_INDEX,
    GENESIS_FORK_VERSION, MAX_EXTRA_DATA_BYTES, MIN_SYNC_COMMITTEE_PARTICIPANTS,
    NEXT_SYNC_COMMITTEE_SUBTREE_INDEX, SYNC_COMMITTEE_SIZE,
};
use hash_db::HashDB;
use memory_db::{HashKey, MemoryDB};
use ssz_rs::prelude::*;
use trie_db::{Trie, TrieDBBuilder};

pub trait BlsVerify {
    fn fast_aggregate_verify(
        &self,
        public_keys: Vec<Vec<u8>>,
        msg: Vec<u8>,
        signature: Vec<u8>,
    ) -> Result<(), Error>;
}

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#validate_light_client_update
pub fn validate_light_client_update<C: LightClientContext, V: BlsVerify>(
    ctx: &C,
    mut update: LightClientUpdate<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
    current_slot: Slot,
    genesis_validators_root: Root,
    bls_verifier: V,
) -> Result<(), Error> {
    // Verify sync committee has sufficient participants
    let sync_aggregate = &update.sync_aggregate;
    if sync_aggregate.sync_committee_bits.count_ones() < MIN_SYNC_COMMITTEE_PARTICIPANTS {
        return Err(Error::InsufficientSyncCommitteeParticipents);
    }

    // Verify update does not skip a sync committee period
    is_valid_light_client_header(ctx, &update.attested_header)?;
    let update_attested_slot = update.attested_header.beacon.slot;
    let update_finalized_slot = update.finalized_header.beacon.slot;

    if !(current_slot >= update.signature_slot
        && update.signature_slot > update_attested_slot
        && update_attested_slot >= update_finalized_slot)
    {
        return Err(Error::InvalidSlots);
    }

    let store_period = compute_sync_committee_period_at_slot(ctx.finalized_slot());
    let update_signature_period = compute_sync_committee_period_at_slot(update.signature_slot);

    if ctx.next_sync_committee().is_some() {
        if update_signature_period != store_period && update_signature_period != store_period + 1 {
            return Err(Error::InvalidSignaturePeriod);
        }
    } else if update_signature_period != store_period {
        return Err(Error::InvalidSignaturePeriod);
    }

    // Verify update is relevant
    let update_attested_period = compute_sync_committee_period_at_slot(update_attested_slot);

    if !(update_attested_slot > ctx.finalized_slot()
        || (update_attested_period == store_period
            && update.next_sync_committee.is_some()
            && ctx.next_sync_committee().is_none()))
    {
        return Err(Error::IrrelevantUpdate);
    }

    // Verify that the `finality_branch`, if present, confirms `finalized_header`
    // to match the finalized checkpoint root saved in the state of `attested_header`.
    is_valid_light_client_header(ctx, &update.finalized_header)?;
    let finalized_root: Node = hash_tree_root(update.finalized_header.beacon.clone())?;

    validate_merkle_branch(
        &finalized_root,
        &update.finality_branch,
        FINALIZED_ROOT_DEPTH,
        FINALIZED_ROOT_SUBTREE_INDEX,
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
            &hash_tree_root(next_sync_committee.clone())?,
            &update.next_sync_committee_branch,
            NEXT_SYNC_COMMITTEE_INDEX_FLOOR_LOG_2,
            NEXT_SYNC_COMMITTEE_SUBTREE_INDEX,
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

    let participant_pubkeys: Vec<Vec<u8>> = update
        .sync_aggregate
        .sync_committee_bits
        .iter()
        .zip(sync_committee.public_keys.iter())
        .filter(|it| *it.0)
        .map(|(_, pubkey)| pubkey.as_slice().to_owned())
        .collect();
    let fork_version_slot = std::cmp::max(update.signature_slot, 1) - 1;
    let fork_version = compute_fork_version(ctx, compute_epoch_at_slot(fork_version_slot));
    let domain = compute_domain(
        DomainType::SyncCommittee,
        Some(fork_version),
        Some(genesis_validators_root),
        GENESIS_FORK_VERSION,
    )?;
    let signing_root = compute_signing_root(&mut update.attested_header.beacon, domain)?;

    bls_verifier.fast_aggregate_verify(
        participant_pubkeys,
        signing_root.as_ref().to_owned(),
        sync_aggregate
            .sync_committee_signature
            .as_slice()
            .to_owned(),
    )?;

    Ok(())
}

fn verify_state(root: Hash32, key: &[u8], proof: &[Vec<u8>]) -> Result<Option<Vec<u8>>, Error> {
    let mut db = MemoryDB::<KeccakHasher, HashKey<_>, Vec<u8>>::default();
    proof.iter().for_each(|n| {
        db.insert(hash_db::EMPTY_PREFIX, n);
    });

    let root: primitive_types::H256 = root.into();
    let trie = TrieDBBuilder::<EthLayout>::new(&db, &root).build();
    Ok(trie.get(&keccak_256(key))?)
}

pub fn verify_account_storage_root(
    root: Hash32,
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

pub fn verify_storage_proof(
    root: Hash32,
    key: &[u8],
    expected_value: &[u8],
    proof: &[Vec<u8>],
) -> Result<(), Error> {
    match verify_state(root, key, proof)? {
        Some(value) if value == expected_value => Ok(()),
        _ => Err(Error::ValueMismatch),
    }
}

/// https://github.com/ethereum/consensus-specs/blob/82d6267951ad47cffa1b7b4179eab97b25a99b91/specs/capella/light-client/sync-protocol.md#modified-is_valid_light_client_header
pub fn is_valid_light_client_header<C: LightClientContext>(
    ctx: &C,
    header: &LightClientHeader<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
) -> Result<(), Error> {
    let epoch = compute_epoch_at_slot(header.beacon.slot);

    if epoch < ctx.fork_parameters().capella_fork_epoch {
        return Err(Error::InvalidChainVersion);
    }

    validate_merkle_branch(
        &hash_tree_root(header.execution.clone())?,
        &header.execution_branch,
        EXECUTION_PAYLOAD_DEPTH,
        EXECUTION_PAYLOAD_INDEX,
        &header.beacon.body_root,
    )
}

use crate::{
    capella::{LightClientHeader, LightClientUpdate, NEXT_SYNC_COMMITTEE_INDEX_FLOOR_LOG_2},
    crypto::{fast_aggregate_verify, BlsPublicKey},
    primitives::{DomainType, Hash32, Root, Slot},
    rlp_node_codec::{keccak_256, EthLayout, KeccakHasher},
    utils::*,
    ChainConfig, Error, LightClientContext, EXECUTION_PAYLOAD_DEPTH, EXECUTION_PAYLOAD_INDEX,
    FINALIZED_ROOT_DEPTH, FINALIZED_ROOT_SUBTREE_INDEX, NEXT_SYNC_COMMITTEE_SUBTREE_INDEX,
};
use hash_db::HashDB;
use memory_db::{HashKey, MemoryDB};
use ssz_rs::prelude::*;
use trie_db::{Trie, TrieDBBuilder};

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#validate_light_client_update
pub fn validate_light_client_update<C: LightClientContext>(
    ctx: &C,
    mut update: LightClientUpdate<
        { C::Config::SYNC_COMMITTEE_SIZE },
        { C::Config::BYTES_PER_LOGS_BLOOM },
        { C::Config::MAX_EXTRA_DATA_BYTES },
    >,
    current_slot: Slot,
    genesis_validators_root: Root,
) -> Result<(), Error> {
    // Verify sync committee has sufficient participants
    let sync_aggregate = &update.sync_aggregate;
    if sync_aggregate.sync_committee_bits.count_ones() < C::Config::MIN_SYNC_COMMITTEE_PARTICIPANTS
    {
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

    let store_period = compute_sync_committee_period_at_slot::<C>(ctx.finalized_slot());
    let update_signature_period = compute_sync_committee_period_at_slot::<C>(update.signature_slot);

    if update.next_sync_committee.is_some() {
        if update_signature_period != store_period && update_signature_period != store_period + 1 {
            return Err(Error::InvalidSignaturePeriod);
        }
    } else if update_signature_period != store_period {
        return Err(Error::InvalidSignaturePeriod);
    }

    // Verify update is relevant
    let update_attested_period = compute_sync_committee_period_at_slot::<C>(update_attested_slot);

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
    // state of the `attested_header`
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

    let participant_pubkeys: Vec<&BlsPublicKey> = update
        .sync_aggregate
        .sync_committee_bits
        .iter()
        .zip(sync_committee.public_keys.iter())
        .filter(|it| *it.0)
        .map(|(_, pubkey)| pubkey)
        .collect();
    let fork_version_slot = std::cmp::max(update.signature_slot, 1) - 1;
    let fork_version = compute_fork_version(ctx, compute_epoch_at_slot::<C>(fork_version_slot));
    let domain = compute_domain(
        DomainType::SyncCommittee,
        Some(fork_version),
        Some(genesis_validators_root),
        C::Config::GENESIS_FORK_VERSION,
    )?;
    let signing_root = compute_signing_root(&mut update.attested_header.beacon, domain)?;

    fast_aggregate_verify(
        participant_pubkeys.as_slice(),
        signing_root.as_ref(),
        &sync_aggregate.sync_committee_signature,
    )?;

    Ok(())
}

pub fn verify_storage_proof(
    root: Hash32,
    key: &[u8],
    expected_value: &[u8],
    proof: Vec<Vec<u8>>,
) -> Result<(), Error> {
    let mut db = MemoryDB::<KeccakHasher, HashKey<_>, Vec<u8>>::default();
    proof.iter().for_each(|n| {
        db.insert(hash_db::EMPTY_PREFIX, n);
    });

    let root: primitive_types::H256 = root.into();
    let trie = TrieDBBuilder::<EthLayout>::new(&db, &root).build();
    match trie.get(&keccak_256(key))? {
        Some(value) if value == expected_value => Ok(()),
        _ => Err(Error::ValueMismatch),
    }
}

/// https://github.com/ethereum/consensus-specs/blob/82d6267951ad47cffa1b7b4179eab97b25a99b91/specs/capella/light-client/sync-protocol.md#modified-is_valid_light_client_header
pub fn is_valid_light_client_header<C: LightClientContext>(
    ctx: &C,
    header: &LightClientHeader<
        { C::Config::BYTES_PER_LOGS_BLOOM },
        { C::Config::MAX_EXTRA_DATA_BYTES },
    >,
) -> Result<(), Error> {
    let epoch = compute_epoch_at_slot::<C>(header.beacon.slot);

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

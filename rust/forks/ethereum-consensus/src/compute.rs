use crate::{
    beacon::{
        BeaconBlockHeader, Domain, DomainType, Epoch, ForkData, Root, SigningData, Slot, Version,
    },
    context::ChainContext,
    errors::Error,
    sync_protocol::SyncCommitteePeriod,
    types::{H256, U64},
};

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/bellatrix/beacon-chain.md#compute_timestamp_at_slot
pub fn compute_timestamp_at_slot<C: ChainContext>(ctx: &C, slot: Slot) -> U64 {
    let slots_since_genesis = slot - ctx.fork_parameters().genesis_slot;
    ctx.genesis_time() + slots_since_genesis * ctx.seconds_per_slot()
}

pub fn compute_sync_committee_period_at_slot<C: ChainContext>(
    ctx: &C,
    slot: Slot,
) -> SyncCommitteePeriod {
    compute_sync_committee_period(ctx, compute_epoch_at_slot(ctx, slot))
}

/// Return the epoch number at ``slot``.
/// https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#compute_epoch_at_slot
pub fn compute_epoch_at_slot<C: ChainContext>(ctx: &C, slot: Slot) -> Epoch {
    slot / ctx.slots_per_epoch()
}

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/validator.md#sync-committee
pub fn compute_sync_committee_period<C: ChainContext>(
    ctx: &C,
    epoch: Epoch,
) -> SyncCommitteePeriod {
    epoch / ctx.epochs_per_sync_committee_period()
}

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/fork.md#compute_fork_version
pub fn compute_fork_version<C: ChainContext>(ctx: &C, epoch: Epoch) -> Version {
    let fork_parameters = ctx.fork_parameters().clone();
    if epoch >= fork_parameters.eip4844_fork_epoch {
        fork_parameters.eip4844_fork_version
    } else if epoch >= fork_parameters.capella_fork_epoch {
        fork_parameters.capella_fork_version
    } else if epoch >= fork_parameters.bellatrix_fork_epoch {
        fork_parameters.bellatrix_fork_version
    } else if epoch >= fork_parameters.altair_fork_epoch {
        fork_parameters.altair_fork_version
    } else {
        fork_parameters.genesis_fork_version
    }
}

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#compute_fork_data_root
pub fn compute_fork_data_root(
    current_version: Version,
    genesis_validators_root: Root,
) -> Result<Root, Error> {
    hash_tree_root(ForkData {
        current_version,
        genesis_validators_root,
    })
}

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#compute_domain
pub fn compute_domain<C: ChainContext>(
    ctx: &C,
    domain_type: DomainType,
    fork_version: Option<Version>,
    genesis_validators_root: Option<Root>,
) -> Result<Domain, Error> {
    let fork_data_root = compute_fork_data_root(
        fork_version.unwrap_or(ctx.fork_parameters().genesis_fork_version.clone()),
        genesis_validators_root.unwrap_or_default(),
    )?;
    let mut domain: [u8; 32] = Default::default();
    domain[..4].copy_from_slice(&domain_type.0);
    domain[4..].copy_from_slice(&fork_data_root.as_bytes()[..28]);
    Ok(Domain(domain))
}

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#compute_signing_root
pub fn compute_signing_root(header: BeaconBlockHeader, domain: Domain) -> Result<Root, Error> {
    hash_tree_root(SigningData {
        object_root: hash_tree_root(header)?,
        domain,
    })
}

pub fn hash_tree_root<T: ssz_rs::SimpleSerialize>(mut object: T) -> Result<Root, Error> {
    Ok(H256::from_slice(object.hash_tree_root()?.as_bytes()))
}

use crate::{
    Error, LightClientContext, EPOCHS_PER_SYNC_COMMITTEE_PERIOD, SECONDS_PER_SLOT, SLOTS_PER_EPOCH,
};
use ethereum_consensus::primitives::{Bytes32, Epoch, Root, Slot, Version, GENESIS_SLOT};
use ssz_rs::prelude::*;

pub use ethereum_consensus::{altair::compute_domain, signing::compute_signing_root};

pub fn compute_fork_version<C: LightClientContext>(ctx: &C, epoch: Epoch) -> Version {
    let fork_parameters = ctx.fork_parameters();
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

pub fn compute_sync_committee_period_at_slot(slot: Slot) -> u64 {
    compute_sync_committee_period(compute_epoch_at_slot(slot))
}

pub fn compute_epoch_at_slot(slot: Slot) -> Epoch {
    slot / SLOTS_PER_EPOCH
}

pub fn compute_sync_committee_period(epoch: Epoch) -> Slot {
    epoch / EPOCHS_PER_SYNC_COMMITTEE_PERIOD
}

pub fn compute_timestamp_at_slot(genesis_time: u64, slot: Slot) -> u64 {
    let slots_since_genesis = slot - GENESIS_SLOT;
    genesis_time + slots_since_genesis * SECONDS_PER_SLOT
}

pub fn hash_tree_root<T: SimpleSerialize>(mut object: T) -> Result<Root, Error> {
    object.hash_tree_root().map_err(Into::into)
}

// https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#is_valid_merkle_branch
pub fn validate_merkle_branch(
    leaf: &Node,
    branch: &[Bytes32],
    depth: usize,
    index: usize,
    root: &Root,
) -> Result<(), Error> {
    let branch: Vec<_> = branch
        .iter()
        .map(|node| Node::try_from(node.as_ref()).expect("is valid instance"))
        .collect();

    if is_valid_merkle_branch(leaf, branch.iter(), depth, index, root) {
        Ok(())
    } else {
        Err(Error::InvalidMerkleBranch)
    }
}

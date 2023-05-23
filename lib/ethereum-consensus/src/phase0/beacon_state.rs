use crate::phase0::{
    BeaconBlockHeader, Checkpoint, Eth1Data, Fork, PendingAttestation, Validator,
    JUSTIFICATION_BITS_LENGTH,
};
use crate::primitives::{Bytes32, Epoch, Gwei, Root, Slot};
use ssz_rs::prelude::*;

pub(super) const fn get_eth1_data_votes_bound(
    epochs_per_eth1_voting_period: Epoch,
    slots_per_epoch: usize,
) -> usize {
    epochs_per_eth1_voting_period as usize * slots_per_epoch
}

pub(super) const fn get_pending_attestations_bound(
    max_attestations: usize,
    slots_per_epoch: usize,
) -> usize {
    max_attestations * slots_per_epoch
}

#[derive(Default, Debug, SimpleSerialize)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HistoricalBatch<const SLOTS_PER_HISTORICAL_ROOT: usize> {
    pub block_roots: Vector<Root, SLOTS_PER_HISTORICAL_ROOT>,
    pub state_roots: Vector<Root, SLOTS_PER_HISTORICAL_ROOT>,
}

/// `HistoricalBatch` is to be used as a "summary" Merkleized container and is simply a wrapper around
/// `block_roots` & `state_roots` (and their respective Merkle roots). Instead of the `_roots`
/// being of type `Vector<Root, N>`, a single `Root` is pulled out to allow Merkleization of each root to be
/// performed manually (using the `ssz_rs` crate).
///
/// Instead of requiring a full copy of the roots, the container is summarized as `HistoricalSummary`
/// with `block_summary_root` & `state_summary_root`.
///
/// This design decision was chosen for memory optimization purposes, for example, in the
/// `state_transition` crate's `process_historical_roots_update` function. Also note that the
/// `HistoricalBatch` container has no need for serialization, otherwise, this design would pose an issue.
///
/// For more information, see the comment here: <https://github.com/ralexstokes/ethereum-consensus/pull/37#discussion_r775995594>
#[derive(Default, Debug, SimpleSerialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HistoricalSummary {
    pub block_summary_root: Root,
    pub state_summary_root: Root,
}

#[derive(Default, Debug, SimpleSerialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BeaconState<
    const SLOTS_PER_HISTORICAL_ROOT: usize,
    const HISTORICAL_ROOTS_LIMIT: usize,
    const ETH1_DATA_VOTES_BOUND: usize,
    const VALIDATOR_REGISTRY_LIMIT: usize,
    const EPOCHS_PER_HISTORICAL_VECTOR: usize,
    const EPOCHS_PER_SLASHINGS_VECTOR: usize,
    const MAX_VALIDATORS_PER_COMMITTEE: usize,
    const PENDING_ATTESTATIONS_BOUND: usize,
> {
    #[serde(with = "crate::serde::as_string")]
    pub genesis_time: u64,
    pub genesis_validators_root: Root,
    #[serde(with = "crate::serde::as_string")]
    pub slot: Slot,
    pub fork: Fork,
    pub latest_block_header: BeaconBlockHeader,
    pub block_roots: Vector<Root, SLOTS_PER_HISTORICAL_ROOT>,
    pub state_roots: Vector<Root, SLOTS_PER_HISTORICAL_ROOT>,
    pub historical_roots: List<Root, HISTORICAL_ROOTS_LIMIT>,
    pub eth1_data: Eth1Data,
    pub eth1_data_votes: List<Eth1Data, ETH1_DATA_VOTES_BOUND>,
    #[serde(with = "crate::serde::as_string")]
    pub eth1_deposit_index: u64,
    pub validators: List<Validator, VALIDATOR_REGISTRY_LIMIT>,
    #[serde(with = "crate::serde::collection_over_string")]
    pub balances: List<Gwei, VALIDATOR_REGISTRY_LIMIT>,
    pub randao_mixes: Vector<Bytes32, EPOCHS_PER_HISTORICAL_VECTOR>,
    #[serde(with = "crate::serde::collection_over_string")]
    pub slashings: Vector<Gwei, EPOCHS_PER_SLASHINGS_VECTOR>,
    pub previous_epoch_attestations:
        List<PendingAttestation<MAX_VALIDATORS_PER_COMMITTEE>, PENDING_ATTESTATIONS_BOUND>,
    pub current_epoch_attestations:
        List<PendingAttestation<MAX_VALIDATORS_PER_COMMITTEE>, PENDING_ATTESTATIONS_BOUND>,
    pub justification_bits: Bitvector<JUSTIFICATION_BITS_LENGTH>,
    pub previous_justified_checkpoint: Checkpoint,
    pub current_justified_checkpoint: Checkpoint,
    pub finalized_checkpoint: Checkpoint,
}

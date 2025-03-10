/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#get_subtree_index>
#[must_use]
pub const fn get_subtree_index(idx: u64) -> u64 {
    idx % 2_u64.pow(idx.ilog2())
}

/// Convenience function safely to call [`u64::ilog2`] and convert the result into a usize.
#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
#[must_use]
pub const fn floorlog2(n: u64) -> usize {
    // conversion is safe since usize is either 32 or 64 bits as per cfg above
    n.ilog2() as usize
}

// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#constants
// REVIEW: Is it possible to implement get_generalized_index in const rust?

// https://github.com/ethereum/consensus-specs/blob/dev/ssz/merkle-proofs.md
/// `get_generalized_index(BeaconState, "finalized_checkpoint", "root")`
pub const FINALIZED_ROOT_GINDEX: u64 = 105;
/// `get_generalized_index(BeaconState, "current_sync_committee")`
pub const CURRENT_SYNC_COMMITTEE_GINDEX: u64 = 54;
/// `get_generalized_index(BeaconState, "next_sync_committee")`
pub const NEXT_SYNC_COMMITTEE_GINDEX: u64 = 55;
/// `get_generalized_index(BeaconBlockBody, "execution_payload")`
pub const EXECUTION_PAYLOAD_GINDEX: u64 = 25;

// <https://github.com/ethereum/consensus-specs/blob/36d80adb44c21c66379c6207a9578f9b1dcc8a2d/specs/electra/light-client/sync-protocol.md#new-constants>

/// `get_generalized_index(BeaconState, 'finalized_checkpoint', 'root')`
pub const FINALIZED_ROOT_GINDEX_ELECTRA: u64 = 169;
/// `get_generalized_index(BeaconState, 'current_sync_committee')`
pub const CURRENT_SYNC_COMMITTEE_GINDEX_ELECTRA: u64 = 86;
/// `get_generalized_index(BeaconState, 'next_sync_committee')`
pub const NEXT_SYNC_COMMITTEE_GINDEX_ELECTRA: u64 = 87;

pub const fn default_epoch() -> u64 {
    u64::MAX
}

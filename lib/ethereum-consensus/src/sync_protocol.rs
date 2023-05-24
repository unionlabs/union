use crate::{
    bls::{is_equal_pubkeys_and_aggreate_pub_key, PublicKey, Signature},
    errors::Error,
    internal_prelude::*,
    types::U64,
};
use ssz_rs::prelude::{Bitvector, Vector};
use ssz_rs::{Deserialize, Sized};
use ssz_rs_derive::SimpleSerialize;

pub type SyncCommitteePeriod = U64;

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#constants
/// get_generalized_index(BeaconState, 'finalized_checkpoint', 'root')
pub const FINALIZED_ROOT_INDEX: u64 = 105;
pub const FINALIZED_ROOT_SUBTREE_INDEX: u64 = 41;
pub const FINALIZED_ROOT_DEPTH: usize = 6;
/// get_generalized_index(BeaconState, 'current_sync_committee')
pub const CURRENT_SYNC_COMMITTEE_INDEX: u64 = 54;
pub const CURRENT_SYNC_COMMITTEE_SUBTREE_INDEX: u64 = 22;
pub const CURRENT_SYNC_COMMITTEE_DEPTH: usize = 5;
/// get_generalized_index(BeaconState, 'next_sync_committee')
pub const NEXT_SYNC_COMMITTEE_INDEX: u64 = 55;
pub const NEXT_SYNC_COMMITTEE_SUBTREE_INDEX: u64 = 23;
pub const NEXT_SYNC_COMMITTEE_DEPTH: usize = 5;
/// get_generalized_index(BeaconBlockBody, 'execution_payload')
pub const EXECUTION_PAYLOAD_INDEX: u64 = 25;
pub const EXECUTION_PAYLOAD_DEPTH: usize = 4;

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/beacon-chain.md#synccommittee
#[derive(
    Clone, Debug, PartialEq, Eq, Default, SimpleSerialize, serde::Serialize, serde::Deserialize,
)]
pub struct SyncCommittee<const SYNC_COMMITTEE_SIZE: usize> {
    pub pubkeys: Vector<PublicKey, SYNC_COMMITTEE_SIZE>,
    pub aggregate_pubkey: PublicKey,
}

impl<const SYNC_COMMITTEE_SIZE: usize> SyncCommittee<SYNC_COMMITTEE_SIZE> {
    pub fn validate(&self) -> Result<(), Error> {
        is_equal_pubkeys_and_aggreate_pub_key(&self.pubkeys, &self.aggregate_pubkey)
    }
}

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/beacon-chain.md#syncaggregate
/**
 *  sync_committee_bits: Bitvector[SYNC_COMMITTEE_SIZE]
 *  sync_committee_signature: BLSSignature
 */
#[derive(
    Clone, Debug, Default, PartialEq, Eq, SimpleSerialize, serde::Serialize, serde::Deserialize,
)]
pub struct SyncAggregate<const SYNC_COMMITTEE_SIZE: usize> {
    pub sync_committee_bits: Bitvector<SYNC_COMMITTEE_SIZE>,
    pub sync_committee_signature: Signature,
}

impl<const SYNC_COMMITTEE_SIZE: usize> SyncAggregate<SYNC_COMMITTEE_SIZE> {
    pub fn count_participants(&self) -> usize {
        self.sync_committee_bits.count_ones()
    }
}

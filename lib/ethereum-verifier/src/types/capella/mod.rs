pub mod mainnet;
pub mod minimal;

use crate::byte_list::ByteList;
use crate::byte_vector::ByteVector;
use crate::crypto::{BlsPublicKey, BlsSignature};
use crate::primitives::{Bytes32, ExecutionAddress, Hash32, Root, Slot, ValidatorIndex, Version};
use ssz_rs::prelude::*;

pub const NEXT_SYNC_COMMITTEE_INDEX_FLOOR_LOG_2: usize = 5;
pub const FINALIZED_ROOT_INDEX_FLOOR_LOG_2: usize = 6;
pub const EXECUTION_BRANCH_INDEX_FLOOR_LOG_2: usize = 4;

#[derive(Default, Debug, SimpleSerialize, Clone, serde::Serialize, serde::Deserialize)]
pub struct ForkData {
    #[serde(with = "crate::serde::as_hex")]
    pub current_version: Version,
    pub genesis_validators_root: Root,
}

#[derive(
    Default, Debug, Clone, SimpleSerialize, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct SyncAggregate<const SYNC_COMMITTEE_SIZE: usize> {
    pub sync_committee_bits: Bitvector<SYNC_COMMITTEE_SIZE>,
    pub sync_committee_signature: BlsSignature,
}

#[derive(
    Default, Debug, SimpleSerialize, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct SyncCommittee<const SYNC_COMMITTEE_SIZE: usize> {
    #[serde(rename = "pubkeys")]
    pub public_keys: Vector<BlsPublicKey, SYNC_COMMITTEE_SIZE>,
    #[serde(rename = "aggregate_pubkey")]
    pub aggregate_public_key: BlsPublicKey,
}

#[derive(Default, Debug, Clone, SimpleSerialize, serde::Serialize, serde::Deserialize)]
pub struct LightClientUpdate<
    const SYNC_COMMITTEE_SIZE: usize,
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
> {
    pub attested_header: LightClientHeader<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
    pub next_sync_committee: Option<SyncCommittee<SYNC_COMMITTEE_SIZE>>,
    pub next_sync_committee_branch: Vector<Bytes32, NEXT_SYNC_COMMITTEE_INDEX_FLOOR_LOG_2>,
    pub finalized_header: LightClientHeader<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
    pub finality_branch: Vector<Bytes32, FINALIZED_ROOT_INDEX_FLOOR_LOG_2>,
    pub sync_aggregate: SyncAggregate<SYNC_COMMITTEE_SIZE>,
    pub signature_slot: Slot,
}

#[derive(Default, Debug, Clone, SimpleSerialize, serde::Serialize, serde::Deserialize)]
pub struct LightClientHeader<const BYTES_PER_LOGS_BLOOM: usize, const MAX_EXTRA_DATA_BYTES: usize> {
    pub beacon: BeaconBlockHeader,
    pub execution: ExecutionPayloadHeader<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
    pub execution_branch: Vector<Bytes32, EXECUTION_BRANCH_INDEX_FLOOR_LOG_2>,
}

#[derive(
    Default, Debug, SimpleSerialize, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct BeaconBlockHeader {
    #[serde(with = "crate::serde::as_string")]
    pub slot: Slot,
    #[serde(with = "crate::serde::as_string")]
    pub proposer_index: ValidatorIndex,
    pub parent_root: Root,
    pub state_root: Root,
    pub body_root: Root,
}

#[derive(
    Default, Debug, Clone, SimpleSerialize, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub struct ExecutionPayloadHeader<
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
> {
    pub parent_hash: Hash32,
    pub fee_recipient: ExecutionAddress,
    pub state_root: Bytes32,
    pub receipts_root: Bytes32,
    pub logs_bloom: ByteVector<BYTES_PER_LOGS_BLOOM>,
    pub prev_randao: Bytes32,
    #[serde(with = "crate::serde::as_string")]
    pub block_number: u64,
    #[serde(with = "crate::serde::as_string")]
    pub gas_limit: u64,
    #[serde(with = "crate::serde::as_string")]
    pub gas_used: u64,
    #[serde(with = "crate::serde::as_string")]
    pub timestamp: u64,
    pub extra_data: ByteList<MAX_EXTRA_DATA_BYTES>,
    pub base_fee_per_gas: U256,
    pub block_hash: Hash32,
    pub transactions_root: Root,
    pub withdrawals_root: Root,
}

use beacon_api_types::altair::SyncCommittee;
use unionlabs::primitives::H256;

use crate::LightClientUpdateData;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
// boxed for size
pub enum LightClientUpdate {
    SyncCommitteePeriodChange(Box<SyncCommitteePeriodChangeUpdate>),
    WithinSyncCommitteePeriod(Box<WithinSyncCommitteePeriodUpdate>),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct SyncCommitteePeriodChangeUpdate {
    /// The next sync committee of the epoch that the client is being updated to, corresponding to `update_data.attested_header.state_root`.
    ///
    /// If the current epoch is 10, this will be the *next* sync committee for epoch 11 (i.e. the sync committee for epoch 12).
    pub next_sync_committee: SyncCommittee,
    /// The path of the next sync committee in the beacon chain SSZ state root.
    pub next_sync_committee_branch: Vec<H256>,

    pub update_data: LightClientUpdateData,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct WithinSyncCommitteePeriodUpdate {
    pub update_data: LightClientUpdateData,
}

impl LightClientUpdate {
    pub fn update_data(&self) -> &LightClientUpdateData {
        match self {
            LightClientUpdate::SyncCommitteePeriodChange(update) => &update.update_data,
            LightClientUpdate::WithinSyncCommitteePeriod(update) => &update.update_data,
        }
    }

    pub fn into_light_client_update(self) -> ethereum_sync_protocol_types::LightClientUpdate {
        match self {
            LightClientUpdate::SyncCommitteePeriodChange(u) => {
                ethereum_sync_protocol_types::LightClientUpdate {
                    attested_header: u.update_data.attested_header,
                    next_sync_committee: Some(u.next_sync_committee),
                    next_sync_committee_branch: Some(u.next_sync_committee_branch),
                    finalized_header: u.update_data.finalized_header,
                    finality_branch: u.update_data.finality_branch,
                    sync_aggregate: u.update_data.sync_aggregate,
                    signature_slot: u.update_data.signature_slot,
                }
            }
            LightClientUpdate::WithinSyncCommitteePeriod(u) => {
                ethereum_sync_protocol_types::LightClientUpdate {
                    attested_header: u.update_data.attested_header,
                    next_sync_committee: None,
                    next_sync_committee_branch: None,
                    finalized_header: u.update_data.finalized_header,
                    finality_branch: u.update_data.finality_branch,
                    sync_aggregate: u.update_data.sync_aggregate,
                    signature_slot: u.update_data.signature_slot,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use beacon_api_types::{
        altair::SyncAggregate, deneb::ExecutionPayloadHeader, phase0::BeaconBlockHeader, slot::Slot,
    };
    use ethereum_sync_protocol_types::LightClientHeader;
    use unionlabs::{
        encoding::{Bincode, Json},
        primitives::{H160, H256, H384, H768, U256},
        test_utils::assert_codec_iso,
    };

    use super::*;
    use crate::{LightClientUpdateData, SyncCommitteePeriodChangeUpdate};

    fn mk_epoch_change_update() -> SyncCommitteePeriodChangeUpdate {
        SyncCommitteePeriodChangeUpdate {
            next_sync_committee: SyncCommittee {
                pubkeys: vec![H384::new([0xAA; 48])],
                aggregate_pubkey: H384::new([0xAA; 48]),
            },
            next_sync_committee_branch: vec![H256::new([0xAA; 32]); 5],
            update_data: mk_light_client_update_data(),
        }
    }

    fn mk_within_epoch_update() -> WithinSyncCommitteePeriodUpdate {
        WithinSyncCommitteePeriodUpdate {
            update_data: mk_light_client_update_data(),
        }
    }

    fn mk_light_client_update_data() -> LightClientUpdateData {
        LightClientUpdateData {
            attested_header: LightClientHeader {
                beacon: BeaconBlockHeader {
                    slot: Slot::new(123),
                    proposer_index: 456,
                    parent_root: H256::new([0xAA; 32]),
                    state_root: H256::new([0xBB; 32]),
                    body_root: H256::new([0xCC; 32]),
                },
                execution: ExecutionPayloadHeader {
                    parent_hash: H256::new([0xAA; 32]),
                    fee_recipient: H160::new([0xAA; 20]),
                    state_root: H256::new([0xAA; 32]),
                    receipts_root: H256::new([0xAA; 32]),
                    logs_bloom: b"bloom".into(),
                    prev_randao: H256::new([0xAA; 32]),
                    block_number: 69,
                    gas_limit: 1_987_654_321,
                    gas_used: 987_654_321,
                    timestamp: 123_456_789,
                    extra_data: b"extra".into(),
                    base_fee_per_gas: U256::from(1u64),
                    block_hash: H256::new([0xAA; 32]),
                    transactions_root: H256::new([0xAA; 32]),
                    withdrawals_root: H256::new([0xAA; 32]),
                    blob_gas_used: 100,
                    excess_blob_gas: 100,
                },
                execution_branch: vec![H256::new([0xAA; 32]); 4],
            },
            finalized_header: LightClientHeader {
                beacon: BeaconBlockHeader {
                    slot: Slot::new(123),
                    proposer_index: 456,
                    parent_root: H256::new([0xAA; 32]),
                    state_root: H256::new([0xBB; 32]),
                    body_root: H256::new([0xCC; 32]),
                },
                execution: ExecutionPayloadHeader {
                    parent_hash: H256::new([0xAA; 32]),
                    fee_recipient: H160::new([0xAA; 20]),
                    state_root: H256::new([0xAA; 32]),
                    receipts_root: H256::new([0xAA; 32]),
                    logs_bloom: b"bloom".into(),
                    prev_randao: H256::new([0xAA; 32]),
                    block_number: 69,
                    gas_limit: 1_987_654_321,
                    gas_used: 987_654_321,
                    timestamp: 123_456_789,
                    extra_data: b"extra".into(),
                    base_fee_per_gas: U256::from(1u64),
                    block_hash: H256::new([0xAA; 32]),
                    transactions_root: H256::new([0xAA; 32]),
                    withdrawals_root: H256::new([0xAA; 32]),
                    blob_gas_used: 100,
                    excess_blob_gas: 100,
                },
                execution_branch: vec![H256::new([0xAA; 32]); 4],
            },
            finality_branch: vec![H256::new([0xAA; 32]); 6],
            sync_aggregate: SyncAggregate {
                sync_committee_bits: [1, 2, 3].to_vec(),
                sync_committee_signature: H768::new([0xAA; 96]),
            },
            signature_slot: Slot::new(123),
        }
    }

    #[test]
    fn epoch_change_update_bincode_iso() {
        assert_codec_iso::<_, Bincode>(&mk_epoch_change_update());
        assert_codec_iso::<_, Bincode>(&LightClientUpdate::SyncCommitteePeriodChange(Box::new(
            mk_epoch_change_update(),
        )));
    }

    #[test]
    fn epoch_change_update_json_iso() {
        assert_codec_iso::<_, Json>(&mk_epoch_change_update());
        assert_codec_iso::<_, Json>(&LightClientUpdate::SyncCommitteePeriodChange(Box::new(
            mk_epoch_change_update(),
        )));
    }

    #[test]
    fn within_epoch_update_bincode_iso() {
        assert_codec_iso::<_, Bincode>(&mk_within_epoch_update());
        assert_codec_iso::<_, Bincode>(&LightClientUpdate::WithinSyncCommitteePeriod(Box::new(
            mk_within_epoch_update(),
        )));
    }

    #[test]
    fn within_epoch_update_json_iso() {
        assert_codec_iso::<_, Json>(&mk_within_epoch_update());
        assert_codec_iso::<_, Json>(&LightClientUpdate::WithinSyncCommitteePeriod(Box::new(
            mk_within_epoch_update(),
        )));
    }
}

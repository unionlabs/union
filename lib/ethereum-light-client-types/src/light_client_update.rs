use beacon_api_types::{light_client_update::NextSyncCommitteeBranch, SyncCommittee};

use crate::LightClientUpdateData;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
// boxed for size
pub enum LightClientUpdate {
    EpochChange(Box<EpochChangeUpdate>),
    WithinEpoch(Box<WithinEpochUpdate>),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct EpochChangeUpdate {
    /// The trusted sync committee for the *next* epoch that the client is being updated to.
    ///
    /// If the current epoch is 10, this will be the sync committee for epoch 11.
    pub sync_committee: SyncCommittee,

    /// The next sync committee of the epoch that the client is being updated to, corresponding to `update_data.attested_header.state_root`.
    ///
    /// If the current epoch is 10, this will be the *next* sync committee for epoch 11 (i.e. the sync committee for epoch 12).
    pub next_sync_committee: SyncCommittee,
    /// The path of the next sync committee in the beacon chain SSZ state root.
    pub next_sync_committee_branch: NextSyncCommitteeBranch,

    pub update_data: LightClientUpdateData,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct WithinEpochUpdate {
    /// The trusted sync committee for the *current* epoch.
    ///
    /// If the current epoch is 10, this will be the sync committee for epoch 10.
    pub sync_committee: SyncCommittee,

    pub update_data: LightClientUpdateData,
}

impl LightClientUpdate {
    pub fn update_data(&self) -> &LightClientUpdateData {
        match self {
            LightClientUpdate::EpochChange(update) => &update.update_data,
            LightClientUpdate::WithinEpoch(update) => &update.update_data,
        }
    }

    /// `ethereum-sync-protocol` takes both `current_sync_committee` and `next_sync_committee` as a parameter.
    /// Although theoretically it can work when both params to be `Some`, for optimization reasons, the client
    /// will only pass one at a time based on the update type. This function returns the currently trusted sync committee
    /// in tuple format ready to be passed in to the verifier.
    ///
    /// Returns `(current_sync_committee, next_sync_committee)`
    pub fn currently_trusted_sync_committee(
        &self,
    ) -> (Option<&SyncCommittee>, Option<&SyncCommittee>) {
        match self {
            LightClientUpdate::EpochChange(update) => (None, Some(&update.sync_committee)),
            LightClientUpdate::WithinEpoch(update) => (Some(&update.sync_committee), None),
        }
    }
}

impl From<LightClientUpdate> for beacon_api_types::LightClientUpdate {
    fn from(value: LightClientUpdate) -> Self {
        match value {
            LightClientUpdate::EpochChange(update) => {
                update.update_data.into_beacon_light_client_update(
                    Some(update.next_sync_committee),
                    Some(update.next_sync_committee_branch),
                )
            }
            LightClientUpdate::WithinEpoch(update) => update
                .update_data
                .into_beacon_light_client_update(None, None),
        }
    }
}

#[cfg(test)]
mod tests {
    use beacon_api_types::{
        execution_payload_header::ExecutionPayloadHeader, slot::Slot, BeaconBlockHeader,
        LightClientHeader, SyncAggregate, SyncCommittee,
    };
    use unionlabs::{
        encoding::{Bincode, Json},
        primitives::{H160, H256, H384, H768, U256},
        test_utils::assert_codec_iso,
    };

    use super::*;
    use crate::{EpochChangeUpdate, LightClientUpdateData};

    fn mk_epoch_change_update() -> EpochChangeUpdate {
        EpochChangeUpdate {
            sync_committee: SyncCommittee {
                pubkeys: vec![H384::new([0xAA; 48])],
                aggregate_pubkey: H384::new([0xAA; 48]),
            },
            next_sync_committee: SyncCommittee {
                pubkeys: vec![H384::new([0xAA; 48])],
                aggregate_pubkey: H384::new([0xAA; 48]),
            },
            next_sync_committee_branch: [H256::new([0xAA; 32]); 5],
            update_data: LightClientUpdateData {
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
                    execution_branch: [H256::new([0xAA; 32]); 4],
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
                    execution_branch: [H256::new([0xAA; 32]); 4],
                },
                finality_branch: [H256::new([0xAA; 32]); 6],
                sync_aggregate: SyncAggregate {
                    sync_committee_bits: [1, 2, 3].to_vec(),
                    sync_committee_signature: H768::new([0xAA; 96]),
                },
                signature_slot: Slot::new(123),
            },
        }
    }

    fn mk_within_epoch_update() -> WithinEpochUpdate {
        WithinEpochUpdate {
            sync_committee: SyncCommittee {
                pubkeys: vec![H384::new([0xAA; 48])],
                aggregate_pubkey: H384::new([0xAA; 48]),
            },
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
                execution_branch: [H256::new([0xAA; 32]); 4],
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
                execution_branch: [H256::new([0xAA; 32]); 4],
            },
            finality_branch: [H256::new([0xAA; 32]); 6],
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
        assert_codec_iso::<_, Bincode>(&LightClientUpdate::EpochChange(Box::new(
            mk_epoch_change_update(),
        )));
    }

    #[test]
    fn epoch_change_update_json_iso() {
        assert_codec_iso::<_, Json>(&mk_epoch_change_update());
        assert_codec_iso::<_, Json>(&LightClientUpdate::EpochChange(Box::new(
            mk_epoch_change_update(),
        )));
    }

    #[test]
    fn within_epoch_update_bincode_iso() {
        assert_codec_iso::<_, Bincode>(&mk_within_epoch_update());
        assert_codec_iso::<_, Bincode>(&LightClientUpdate::WithinEpoch(Box::new(
            mk_within_epoch_update(),
        )));
    }

    #[test]
    fn within_epoch_update_json_iso() {
        assert_codec_iso::<_, Json>(&mk_within_epoch_update());
        assert_codec_iso::<_, Json>(&LightClientUpdate::WithinEpoch(Box::new(
            mk_within_epoch_update(),
        )));
    }
}

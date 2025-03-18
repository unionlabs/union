use unionlabs::ibc::core::client::height::Height;

use crate::{AccountProof, LightClientUpdate};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Header {
    /// The currently trusted height of the light client to apply this update against.
    pub trusted_height: Height,

    /// The actual update data to be applied.
    pub consensus_update: LightClientUpdate,

    /// Proof of the IBC handler contract against the execution state root provided in `consensus_update`.
    pub ibc_account_proof: AccountProof,
}

#[cfg(test)]
mod tests {
    use beacon_api_types::{
        altair::{SyncAggregate, SyncCommittee},
        custom_types::{Gas, Slot, ValidatorIndex},
        deneb::ExecutionPayloadHeader,
        phase0::BeaconBlockHeader,
    };
    use ethereum_sync_protocol_types::LightClientHeader;
    use unionlabs::{
        encoding::{Bincode, Json},
        primitives::{H160, H256, H384, H768, U256},
        test_utils::assert_codec_iso,
    };

    use super::*;
    use crate::{LightClientUpdateData, SyncCommitteePeriodChangeUpdate};

    fn mk_header() -> Header {
        Header {
            trusted_height: Height::new(123),
            consensus_update: LightClientUpdate::SyncCommitteePeriodChange(Box::new(
                SyncCommitteePeriodChangeUpdate {
                    next_sync_committee: SyncCommittee {
                        pubkeys: vec![H384::new([0xAA; 48])],
                        aggregate_pubkey: H384::new([0xAA; 48]),
                    },
                    next_sync_committee_branch: vec![H256::new([0xAA; 32]); 5],
                    update_data: LightClientUpdateData {
                        attested_header: LightClientHeader {
                            beacon: BeaconBlockHeader {
                                slot: Slot::new(123),
                                proposer_index: ValidatorIndex::new(456),
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
                                gas_limit: Gas::new(1_987_654_321),
                                gas_used: Gas::new(987_654_321),
                                timestamp: 123_456_789,
                                extra_data: b"extra".into(),
                                base_fee_per_gas: U256::from(1u64),
                                block_hash: H256::new([0xAA; 32]),
                                transactions_root: H256::new([0xAA; 32]),
                                withdrawals_root: H256::new([0xAA; 32]),
                                blob_gas_used: Gas::new(100),
                                excess_blob_gas: Gas::new(100),
                            },
                            execution_branch: vec![H256::new([0xAA; 32]); 4],
                        },
                        finalized_header: LightClientHeader {
                            beacon: BeaconBlockHeader {
                                slot: Slot::new(123),
                                proposer_index: ValidatorIndex::new(456),
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
                                gas_limit: Gas::new(1_987_654_321),
                                gas_used: Gas::new(987_654_321),
                                timestamp: 123_456_789,
                                extra_data: b"extra".into(),
                                base_fee_per_gas: U256::from(1u64),
                                block_hash: H256::new([0xAA; 32]),
                                transactions_root: H256::new([0xAA; 32]),
                                withdrawals_root: H256::new([0xAA; 32]),
                                blob_gas_used: Gas::new(100),
                                excess_blob_gas: Gas::new(100),
                            },
                            execution_branch: vec![H256::new([0xAA; 32]); 4],
                        },
                        finality_branch: vec![H256::new([0xAA; 32]); 6],
                        sync_aggregate: SyncAggregate {
                            sync_committee_bits: [1, 2, 3].to_vec(),
                            sync_committee_signature: H768::new([0xAA; 96]),
                        },
                        signature_slot: Slot::new(123),
                    },
                },
            )),
            ibc_account_proof: AccountProof {
                storage_root: H256::new([0xAA; 32]),
                proof: vec![b"ooga".into(), b"booga".into()],
            },
        }
    }

    #[test]
    fn bincode_iso() {
        assert_codec_iso::<_, Bincode>(&mk_header());
    }

    #[test]
    fn json_iso() {
        assert_codec_iso::<_, Json>(&mk_header());
    }
}

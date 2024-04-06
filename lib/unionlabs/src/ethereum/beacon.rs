use macros::model;
use serde::{Deserialize, Serialize};
use ssz::{
    types::{FixedVector, VariableList},
    Decode, Encode,
};
use tree_hash::TreeHash;
use typenum::U;

use super::{config::MAX_BLOB_COMMITMENTS_PER_BLOCK, KZGCommitment};
use crate::{
    bls::{BlsPublicKey, BlsSignature},
    ethereum::{
        config::{
            consts::{floorlog2, CURRENT_SYNC_COMMITTEE_INDEX, FINALIZED_ROOT_INDEX},
            BYTES_PER_LOGS_BLOOM, DEPOSIT_CONTRACT_TREE_DEPTH, MAX_ATTESTATIONS,
            MAX_ATTESTER_SLASHINGS, MAX_BLS_TO_EXECUTION_CHANGES, MAX_BYTES_PER_TRANSACTION,
            MAX_DEPOSITS, MAX_EXTRA_DATA_BYTES, MAX_PROPOSER_SLASHINGS,
            MAX_TRANSACTIONS_PER_PAYLOAD, MAX_VALIDATORS_PER_COMMITTEE, MAX_VOLUNTARY_EXITS,
            MAX_WITHDRAWALS_PER_PAYLOAD, SYNC_COMMITTEE_SIZE,
        },
        Attestation, AttesterSlashing, Deposit, Eth1Data, ProposerSlashing, SignedVoluntaryExit,
        Version,
    },
    hash::{H160, H256},
    ibc::lightclients::ethereum::{
        beacon_block_header::BeaconBlockHeader, execution_payload_header::ExecutionPayloadHeader,
        light_client_header::LightClientHeader, sync_aggregate::SyncAggregate,
        sync_committee::SyncCommittee,
    },
    uint::U256,
};

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#beaconblock>
#[derive(Clone, Debug, PartialEq, Encode, Decode, TreeHash, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub struct BeaconBlock<
    C: MAX_PROPOSER_SLASHINGS
        + MAX_VALIDATORS_PER_COMMITTEE
        + MAX_ATTESTER_SLASHINGS
        + MAX_ATTESTATIONS
        + DEPOSIT_CONTRACT_TREE_DEPTH
        + MAX_DEPOSITS
        + MAX_VOLUNTARY_EXITS
        + BYTES_PER_LOGS_BLOOM
        + MAX_EXTRA_DATA_BYTES
        + MAX_BYTES_PER_TRANSACTION
        + MAX_TRANSACTIONS_PER_PAYLOAD
        + MAX_WITHDRAWALS_PER_PAYLOAD
        + MAX_BLS_TO_EXECUTION_CHANGES
        + MAX_BLOB_COMMITMENTS_PER_BLOCK
        + SYNC_COMMITTEE_SIZE,
> {
    #[serde(with = "::serde_utils::string")]
    pub slot: u64,
    #[serde(with = "::serde_utils::string")]
    pub proposer_index: u64,
    pub parent_root: H256,
    pub state_root: H256,
    pub body: BeaconBlockBody<C>,
}

impl<
        C: MAX_PROPOSER_SLASHINGS
            + MAX_VALIDATORS_PER_COMMITTEE
            + MAX_ATTESTER_SLASHINGS
            + MAX_ATTESTATIONS
            + DEPOSIT_CONTRACT_TREE_DEPTH
            + MAX_DEPOSITS
            + MAX_VOLUNTARY_EXITS
            + BYTES_PER_LOGS_BLOOM
            + MAX_EXTRA_DATA_BYTES
            + MAX_BYTES_PER_TRANSACTION
            + MAX_TRANSACTIONS_PER_PAYLOAD
            + MAX_WITHDRAWALS_PER_PAYLOAD
            + MAX_BLS_TO_EXECUTION_CHANGES
            + MAX_BLOB_COMMITMENTS_PER_BLOCK
            + SYNC_COMMITTEE_SIZE,
    > BeaconBlock<C>
{
    #[must_use]
    pub fn to_header(self) -> BeaconBlockHeader {
        BeaconBlockHeader {
            slot: self.slot,
            proposer_index: self.proposer_index,
            parent_root: self.parent_root,
            state_root: self.state_root,
            body_root: self.body.tree_hash_root().into(),
        }
    }
}

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/bellatrix/beacon-chain.md#beaconblockbody>
#[derive(Encode, Decode, TreeHash)]
#[model]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct BeaconBlockBody<
    C: MAX_PROPOSER_SLASHINGS
        + MAX_VALIDATORS_PER_COMMITTEE
        + MAX_ATTESTER_SLASHINGS
        + MAX_ATTESTATIONS
        + DEPOSIT_CONTRACT_TREE_DEPTH
        + MAX_DEPOSITS
        + MAX_VOLUNTARY_EXITS
        + BYTES_PER_LOGS_BLOOM
        + MAX_EXTRA_DATA_BYTES
        + MAX_BYTES_PER_TRANSACTION
        + MAX_TRANSACTIONS_PER_PAYLOAD
        + MAX_WITHDRAWALS_PER_PAYLOAD
        + MAX_BLS_TO_EXECUTION_CHANGES
        + MAX_BLOB_COMMITMENTS_PER_BLOCK
        + SYNC_COMMITTEE_SIZE,
> {
    pub randao_reveal: BlsSignature,
    pub eth1_data: Eth1Data,
    #[serde(with = "::serde_utils::hex_string")]
    pub graffiti: [u8; 32],
    pub proposer_slashings: VariableList<ProposerSlashing, C::MAX_PROPOSER_SLASHINGS>,
    pub attester_slashings: VariableList<AttesterSlashing<C>, C::MAX_ATTESTER_SLASHINGS>,
    pub attestations: VariableList<Attestation<C>, C::MAX_ATTESTATIONS>,
    pub deposits: VariableList<Deposit<C>, C::MAX_DEPOSITS>,
    pub voluntary_exits: VariableList<SignedVoluntaryExit, C::MAX_VOLUNTARY_EXITS>,
    pub sync_aggregate: SyncAggregate<C>,
    pub execution_payload: ExecutionPayload<C>,
    pub bls_to_execution_changes:
        VariableList<SignedBlsToExecutionChange, C::MAX_BLS_TO_EXECUTION_CHANGES>,
    pub blob_kzg_commitments: VariableList<KZGCommitment, C::MAX_BLOB_COMMITMENTS_PER_BLOCK>,
}

#[derive(Clone, Debug, PartialEq, Encode, Decode, TreeHash, Serialize, Deserialize)]
pub struct BlsToExecutionChange {
    #[serde(with = "::serde_utils::string")]
    pub validator_index: u64,
    pub from_bls_pubkey: BlsPublicKey,
    pub to_execution_address: H160,
}

#[derive(Clone, Debug, PartialEq, Encode, Decode, TreeHash, Serialize, Deserialize)]
pub struct SignedBlsToExecutionChange {
    message: BlsToExecutionChange,
    signature: BlsSignature,
}

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/bellatrix/beacon-chain.md#executionpayload>
#[derive(Clone, Debug, PartialEq, Encode, Decode, TreeHash, Serialize, Deserialize)]
pub struct ExecutionPayload<
    C: BYTES_PER_LOGS_BLOOM
        + MAX_EXTRA_DATA_BYTES
        + MAX_BYTES_PER_TRANSACTION
        + MAX_TRANSACTIONS_PER_PAYLOAD
        + MAX_WITHDRAWALS_PER_PAYLOAD,
> {
    /// Execution block header fields
    pub parent_hash: H256,
    pub fee_recipient: H160,
    pub state_root: H256,
    pub receipts_root: H256,
    #[serde(with = "::serde_utils::hex_string")]
    pub logs_bloom: FixedVector<u8, C::BYTES_PER_LOGS_BLOOM>,
    /// 'difficulty' in the yellow paper
    pub prev_randao: H256,
    /// 'number' in the yellow paper
    #[serde(with = "::serde_utils::string")]
    pub block_number: u64,
    #[serde(with = "::serde_utils::string")]
    pub gas_limit: u64,
    #[serde(with = "::serde_utils::string")]
    pub gas_used: u64,
    #[serde(with = "::serde_utils::string")]
    pub timestamp: u64,
    #[serde(with = "::serde_utils::hex_string")]
    pub extra_data: VariableList<u8, C::MAX_EXTRA_DATA_BYTES>,
    pub base_fee_per_gas: U256,
    /// Extra payload fields
    /// Hash of execution block
    pub block_hash: H256,
    #[serde(with = "::serde_utils::hex_string_list")]
    pub transactions: VariableList<
        VariableList<u8, C::MAX_BYTES_PER_TRANSACTION>,
        C::MAX_TRANSACTIONS_PER_PAYLOAD,
    >,
    pub withdrawals: VariableList<Withdrawal, C::MAX_WITHDRAWALS_PER_PAYLOAD>,
    // blob_gas_used: uint64  # [New in Deneb:EIP4844]
    #[serde(default, with = "::serde_utils::string")]
    pub blob_gas_used: u64,
    // excess_blob_gas: uint64  # [New in Deneb:EIP4844]
    #[serde(default, with = "::serde_utils::string")]
    pub excess_blob_gas: u64,
}

impl<
        C: BYTES_PER_LOGS_BLOOM
            + MAX_EXTRA_DATA_BYTES
            + MAX_BYTES_PER_TRANSACTION
            + MAX_TRANSACTIONS_PER_PAYLOAD
            + MAX_WITHDRAWALS_PER_PAYLOAD,
    > ExecutionPayload<C>
{
    #[must_use]
    pub fn to_header(self) -> ExecutionPayloadHeader<C> {
        ExecutionPayloadHeader {
            parent_hash: self.parent_hash,
            fee_recipient: self.fee_recipient,
            state_root: self.state_root,
            receipts_root: self.receipts_root,
            logs_bloom: self.logs_bloom,
            prev_randao: self.prev_randao,
            block_number: self.block_number,
            gas_limit: self.gas_limit,
            gas_used: self.gas_used,
            timestamp: self.timestamp,
            extra_data: self.extra_data,
            base_fee_per_gas: self.base_fee_per_gas,
            block_hash: self.block_hash,
            transactions_root: self.transactions.tree_hash_root().into(),
            withdrawals_root: self.withdrawals.tree_hash_root().into(),
            blob_gas_used: self.blob_gas_used,
            excess_blob_gas: self.excess_blob_gas,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Encode, Decode, TreeHash, Serialize, Deserialize)]
pub struct Withdrawal {
    #[serde(with = "::serde_utils::string")]
    pub index: u64,
    #[serde(with = "::serde_utils::string")]
    pub validator_index: u64,
    pub address: H160,
    #[serde(with = "::serde_utils::string")]
    pub amount: u64,
}

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#lightclientbootstrap>
#[derive(Clone, Debug, PartialEq, Encode, Decode, TreeHash, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct LightClientBootstrap<
    C: SYNC_COMMITTEE_SIZE + BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES,
> {
    pub header: LightClientHeader<C>,
    /// Current sync committee corresponding to `beacon_header.state_root`
    pub current_sync_committee: SyncCommittee<C>,
    // TODO: Update tree_hash to support const generic arrays
    pub current_sync_committee_branch:
        FixedVector<H256, U<{ floorlog2(CURRENT_SYNC_COMMITTEE_INDEX) }>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct LightClientFinalityUpdate<
    C: SYNC_COMMITTEE_SIZE + BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES,
> {
    /// Header attested to by the sync committee
    pub attested_header: LightClientHeader<C>,
    /// Finalized header corresponding to `attested_header.state_root`
    pub finalized_header: LightClientHeader<C>,
    pub finality_branch: [H256; floorlog2(FINALIZED_ROOT_INDEX)],
    /// Sync committee aggregate signature
    pub sync_aggregate: SyncAggregate<C>,
    /// Slot at which the aggregate signature was created (untrusted)
    #[serde(with = "::serde_utils::string")]
    pub signature_slot: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct GenesisData {
    pub genesis_validators_root: H256,
    #[serde(with = "::serde_utils::string")]
    pub genesis_time: u64,
    pub genesis_fork_version: Version,
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use super::*;
    use crate::{
        ethereum::config::Minimal,
        test_utils::{assert_json_roundtrip, assert_proto_roundtrip},
    };

    #[test]
    fn finality_update_json() {
        const JSON: &str = r#"{
  "attested_header": {
    "beacon": {
      "slot": "280",
      "proposer_index": "7",
      "parent_root": "0x30e57d9c39682aae6f3becb679e39600356fed68d40290a853ee65e1ff8d9ee1",
      "state_root": "0xb073f2525831fd8a9bdb5749c1ec52ec67ab13bf505fdc032ee0e820cc6546f6",
      "body_root": "0x324d12c3994fe88fb55d72d762043cc7c222218448343d88e842e9c4645ff715"
    },
    "execution": {
      "parent_hash": "0x871450e34fc449f7a1bc8b0f8c7139bf076565a23cc452ebf9d86dd07ee22b9a",
      "fee_recipient": "0x0000000000000000000000000000000000000000",
      "state_root": "0x0d4d8098600785265663a9d27fb52ee6f43dca01a0ad4efb0880a6aaf09c457c",
      "receipts_root": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
      "logs_bloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
      "prev_randao": "0xe335e0e5bfcb9f4cd1e01ec5802a6ba57dd68df9d71a7ff914e1e9c5ff85f531",
      "block_number": "280",
      "gas_limit": "30000000",
      "gas_used": "0",
      "timestamp": "1688708475",
      "extra_data": "0xd883010b06846765746888676f312e32302e34856c696e7578",
      "base_fee_per_gas": "77",
      "block_hash": "0x65d0e7fa05f9e3d67c5ac592271261836de85245ba259d445f79d888767652d9",
      "transactions_root": "0x7ffe241ea60187fdb0187bfa22de35d1f9bed7ab061d9401fd47e34a54fbede1",
      "withdrawals_root": "0x28ba1834a3a7b657460ce79fa3a1d909ab8828fd557659d4d0554a9bdbc0ec30"
    },
    "execution_branch": [
      "0x482ae09cbc539f4f49df50168ea76d4ebebbf90690b9ce138103f8a3775398f4",
      "0x336488033fe5f3ef4ccc12af07b9370b92e553e35ecb4a337a1b1c0e4afe1e0e",
      "0xdb56114e00fdd4c1f85c892bf35ac9a89289aaecb1ebd0a96cde606a748b5d71",
      "0xfd142d0fed75a4c9aa054c276334436080c582b69cc6a21a4e8fe436f734be4f"
    ]
  },
  "finalized_header": {
    "beacon": {
      "slot": "264",
      "proposer_index": "7",
      "parent_root": "0x339cd604f81e4a8f7758fe2aef1b8cfcba9a9910bfdc45458a0163780169e85f",
      "state_root": "0x4c0b790c2a58ee8692c50711f8ac146aa9b715f49d131383fe58f103c403a6bc",
      "body_root": "0xd611a2c544fb7b4426c825c500210f88e70dbc09d799a394e3c5bbb9467e634e"
    },
    "execution": {
      "parent_hash": "0x6c7c68f3b77b40a89b43d7fa123465b6e75b1ffc697ad6aed2de4ca05ae3c753",
      "fee_recipient": "0x0000000000000000000000000000000000000000",
      "state_root": "0x4122ceb79224c9d3b34c166ff1a2efdeb2236b4984ac2e8718605ad60b7e2aba",
      "receipts_root": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
      "logs_bloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
      "prev_randao": "0x3420359baf09fc25fa6a6ec5855ed8f0e65212c8c14374f7922eddc320d40714",
      "block_number": "264",
      "gas_limit": "30000000",
      "gas_used": "0",
      "timestamp": "1688708379",
      "extra_data": "0xd883010b06846765746888676f312e32302e34856c696e7578",
      "base_fee_per_gas": "77",
      "block_hash": "0xe05c99fc9df0c133f10456c3d2d7d7e5841ff6aca907508e72d3c02280413065",
      "transactions_root": "0x7ffe241ea60187fdb0187bfa22de35d1f9bed7ab061d9401fd47e34a54fbede1",
      "withdrawals_root": "0x28ba1834a3a7b657460ce79fa3a1d909ab8828fd557659d4d0554a9bdbc0ec30"
    },
    "execution_branch": [
      "0x130aa8119e10aed0862a0550f99d82419af83e4822e7f6c4b799b2928bae551d",
      "0x336488033fe5f3ef4ccc12af07b9370b92e553e35ecb4a337a1b1c0e4afe1e0e",
      "0xdb56114e00fdd4c1f85c892bf35ac9a89289aaecb1ebd0a96cde606a748b5d71",
      "0x9ca42a8d07bb534daa77a5da0d4f2eeba52eaa414cd17807a41a9c18df2aacda"
    ]
  },
  "finality_branch": [
    "0x2100000000000000000000000000000000000000000000000000000000000000",
    "0x10c726fac935bf9657cc7476d3cfa7bedec5983dcfb59e8a7df6d0a619e108d7",
    "0xfd373e9e3590182a6650e8fc1eea2665ee425d02832eb4d1aaa1940f7047abeb",
    "0x40f7f1ddea7f55f22f4ba2c3c188150b20ad892fcd52c1c8487905906e9990f9",
    "0x0e9519008f24522335ee93f022f1c98fe1d5e494094b4e59aec2d6d855b1d628",
    "0x8fa4b6a582dd40416fe63eea0badedb0e2259aed89a6c19dcebfa367476b95fe"
  ],
  "sync_aggregate": {
    "sync_committee_bits": "0xffffffff",
    "sync_committee_signature": "0xb13181bcd13a1e9450452290926985eea1b005a11310c69487f646980f84b3597fbd09d9406093f0cf918e2d6304291406330abb3e90b0d4406e1f902cc49e2faea2df7f6311069d438b4fc23466fe3436bf53caa39461e18fb0b236d446c5a0"
  },
  "signature_slot": "281"
}"#;

        let finality_update =
            serde_json::from_str::<LightClientFinalityUpdate<Minimal>>(JSON).unwrap();

        dbg!(&finality_update);

        assert_json_roundtrip(&finality_update.attested_header);

        assert_proto_roundtrip(&finality_update.attested_header);

        dbg!(U256::from_str("77").unwrap());

        assert_eq!(
            finality_update
                .finalized_header
                .execution
                .base_fee_per_gas
                .0
                .as_u128(),
            77
        );

        serde_json::from_str::<SyncAggregate<Minimal>>(r#"{"sync_committee_bits":"0xffffffff","sync_committee_signature":"0xb13181bcd13a1e9450452290926985eea1b005a11310c69487f646980f84b3597fbd09d9406093f0cf918e2d6304291406330abb3e90b0d4406e1f902cc49e2faea2df7f6311069d438b4fc23466fe3436bf53caa39461e18fb0b236d446c5a0"}"#).unwrap();
    }
}

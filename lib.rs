pub mod attestation;
pub mod attestation_data;
pub mod attester_slashing;
pub mod beacon_block;
pub mod beacon_block_body;
pub mod bls_to_execution_change;
pub mod checkpoint;
pub mod deposit;
pub mod deposit_data;
pub mod eth1_data;
pub mod execution_payload;
pub mod fork_data;
pub mod genesis_data;
pub mod indexed_attestation;
pub mod kzg_commitment;
pub mod light_client_bootstrap;
pub mod light_client_finality_update;
pub mod proposer_slashing;
pub mod signed_beacon_block;
pub mod signed_beacon_block_header;
pub mod signed_bls_to_execution_change;
pub mod signed_voluntary_exit;
pub mod signing_data;
pub mod voluntary_exit;
pub mod withdrawal;

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use super::*;
    use crate::{
        ethereum::config::Minimal,
        ibc::lightclients::ethereum::sync_aggregate::SyncAggregate,
        test_utils::{assert_json_roundtrip, assert_proto_roundtrip},
        uint::U256,
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

        let finality_update = serde_json::from_str::<
            light_client_finality_update::LightClientFinalityUpdate<Minimal>,
        >(JSON)
        .unwrap();

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

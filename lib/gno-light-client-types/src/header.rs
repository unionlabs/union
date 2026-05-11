use gno_types::{SignedHeader, ValidatorSet};
use unionlabs::ibc::core::client::height::Height;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Header {
    pub signed_header: SignedHeader,
    pub validator_set: ValidatorSet,
    pub trusted_height: Height,
    pub trusted_validators: ValidatorSet,
}

#[cfg(test)]
mod tests {
    use gno_types::{BlockId, Commit, PartSetHeader, PublicKey, SignedMsgType, Validator, Vote};
    use hex_literal::hex;
    use unionlabs::{
        encoding::{Bincode, DecodeAs, Json},
        google::protobuf::timestamp::Timestamp,
        primitives::{Bech32, H160, H256},
        test_utils::assert_codec_iso,
    };

    use super::*;

    fn mk_header() -> Header {
        Header {
            signed_header: SignedHeader {
                header: gno_types::Header {
                    version: "version".to_owned(),
                    chain_id: "oogabooga".to_owned(),
                    height: 321.try_into().unwrap(),
                    time: Timestamp {
                        seconds: 654.try_into().unwrap(),
                        nanos: 321.try_into().unwrap(),
                    },
                    last_block_id: BlockId {
                        hash: Some(H256::new([0xAA; 32])),
                        parts_header: PartSetHeader {
                            total: 1,
                            hash: Some(H256::new([0xAA; 32])),
                        },
                    },
                    last_commit_hash: Some(H256::new([0xAA; 32])),
                    data_hash: Some(H256::new([0xAA; 32])),
                    validators_hash: H256::new([0xAA; 32]),
                    next_validators_hash: H256::new([0xAA; 32]),
                    consensus_hash: H256::new([0xAA; 32]),
                    app_hash: Some(H256::new([0xAA; 32])),
                    last_results_hash: Some(H256::new([0xAA; 32])),
                    proposer_address: Bech32::new("g".to_owned(), H160::new([0xAA; 20])),
                    num_txs: 10_i64.try_into().unwrap(),
                    total_txs: 100_i64.try_into().unwrap(),
                    app_version: "".to_owned(),
                },
                commit: Commit {
                    block_id: BlockId {
                        hash: Some(H256::new([0xAA; 32])),
                        parts_header: PartSetHeader {
                            total: 1,
                            hash: Some(H256::new([0xAA; 32])),
                        },
                    },
                    precommits: [
                        None,
                        Some(Vote {
                            ty: SignedMsgType::Precommit,
                            height: 0_i64.try_into().unwrap(),
                            round: 0_i32.try_into().unwrap(),
                            block_id: BlockId {
                                hash: Some(H256::new([0xAA; 32])),
                                parts_header: PartSetHeader {
                                    total: 1,
                                    hash: Some(H256::new([0xAA; 32])),
                                },
                            },
                            timestamp: Timestamp::default(),
                            validator_address: Bech32::new("g".to_owned(), H160::new([0xBB; 20])),
                            validator_index: 1,
                            signature: [1, 2, 3].into(),
                        }),
                    ]
                    .to_vec(),
                },
            },
            validator_set: ValidatorSet {
                validators: [
                    Validator {
                        address: Bech32::new("g".to_owned(), H160::new([0xAA; 20])),
                        pub_key: PublicKey::Ed25519([1, 2, 3].into()),
                        voting_power: 1.try_into().unwrap(),
                        proposer_priority: -1,
                    },
                    Validator {
                        address: Bech32::new("g".to_owned(), H160::new([0xAA; 20])),
                        pub_key: PublicKey::Secp256k1([1, 2, 3].into()),
                        voting_power: 1.try_into().unwrap(),
                        proposer_priority: -1,
                    },
                ]
                .to_vec(),
                proposer: Validator {
                    address: Bech32::new("g".to_owned(), H160::new([0xAA; 20])),
                    pub_key: PublicKey::Ed25519([1, 2, 3].into()),
                    voting_power: 1.try_into().unwrap(),
                    proposer_priority: -1,
                },
            },
            trusted_height: Height::new_with_revision(69, 420),
            trusted_validators: ValidatorSet {
                validators: [
                    Validator {
                        address: Bech32::new("g".to_owned(), H160::new([0xAA; 20])),
                        pub_key: PublicKey::Secp256k1([1, 2, 3].into()),
                        voting_power: 1.try_into().unwrap(),
                        proposer_priority: -1,
                    },
                    Validator {
                        address: Bech32::new("g".to_owned(), H160::new([0xAA; 20])),
                        pub_key: PublicKey::Ed25519([1, 2, 3].into()),
                        voting_power: 1.try_into().unwrap(),
                        proposer_priority: -1,
                    },
                ]
                .to_vec(),
                proposer: Validator {
                    address: Bech32::new("g".to_owned(), H160::new([0xAA; 20])),
                    pub_key: PublicKey::Ed25519([1, 2, 3].into()),
                    voting_power: 1.try_into().unwrap(),
                    proposer_priority: -1,
                },
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

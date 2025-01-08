use cometbft_types::types::{signed_header::SignedHeader, validator_set::ValidatorSet};
use unionlabs::ibc::core::client::height::Height;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Header {
    pub signed_header: SignedHeader,
    pub validator_set: ValidatorSet,
    pub trusted_height: Height,
    pub trusted_validators: ValidatorSet,
}

#[cfg(feature = "proto")]
pub mod proto {
    use cometbft_types::types::{signed_header, validator_set};
    use unionlabs::{errors::MissingField, impl_proto_via_try_from_into, required};

    use crate::Header;

    impl_proto_via_try_from_into!(Header => protos::ibc::lightclients::tendermint::v1::Header);

    impl From<Header> for protos::ibc::lightclients::tendermint::v1::Header {
        fn from(value: Header) -> Self {
            Self {
                signed_header: Some(value.signed_header.into()),
                validator_set: Some(value.validator_set.into()),
                trusted_height: Some(value.trusted_height.into()),
                trusted_validators: Some(value.trusted_validators.into()),
            }
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid signed header")]
        SignedHeader(#[from] signed_header::proto::Error),
        #[error("invalid validator set")]
        ValidatorSet(#[source] validator_set::proto::Error),
        #[error("invalid trusted validators")]
        TrustedValidators(#[source] validator_set::proto::Error),
    }

    impl TryFrom<protos::ibc::lightclients::tendermint::v1::Header> for Header {
        type Error = Error;

        fn try_from(
            value: protos::ibc::lightclients::tendermint::v1::Header,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                signed_header: required!(value.signed_header)?.try_into()?,
                validator_set: required!(value.validator_set)?
                    .try_into()
                    .map_err(Error::ValidatorSet)?,
                trusted_height: required!(value.trusted_height)?.into(),
                trusted_validators: required!(value.trusted_validators)?
                    .try_into()
                    .map_err(Error::TrustedValidators)?,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use cometbft_types::{
        crypto::public_key::PublicKey,
        types::{
            block_id::BlockId, commit::Commit, commit_sig::CommitSig,
            part_set_header::PartSetHeader, validator::Validator,
        },
        version::consensus::Consensus,
    };
    use unionlabs::{
        encoding::{Bincode, Json, Proto},
        google::protobuf::timestamp::Timestamp,
        primitives::{H160, H256},
        test_utils::assert_codec_iso,
    };

    use super::*;

    fn mk_header() -> Header {
        Header {
            signed_header: SignedHeader {
                header: cometbft_types::types::header::Header {
                    version: Consensus {
                        block: 123,
                        app: 456,
                    },
                    chain_id: "oogabooga".to_owned(),
                    height: 321.try_into().unwrap(),
                    time: Timestamp {
                        seconds: 654.try_into().unwrap(),
                        nanos: 321.try_into().unwrap(),
                    },
                    last_block_id: BlockId {
                        hash: Some(H256::new([0xAA; 32])),
                        part_set_header: PartSetHeader {
                            total: 1,
                            hash: Some(H256::new([0xAA; 32])),
                        },
                    },
                    last_commit_hash: H256::new([0xAA; 32]),
                    data_hash: H256::new([0xAA; 32]),
                    validators_hash: H256::new([0xAA; 32]),
                    next_validators_hash: H256::new([0xAA; 32]),
                    consensus_hash: H256::new([0xAA; 32]),
                    app_hash: H256::new([0xAA; 32]),
                    last_results_hash: H256::new([0xAA; 32]),
                    evidence_hash: H256::new([0xAA; 32]),
                    proposer_address: H160::new([0xAA; 20]),
                },
                commit: Commit {
                    height: 321.try_into().unwrap(),
                    round: 321.try_into().unwrap(),
                    block_id: BlockId {
                        hash: Some(H256::new([0xAA; 32])),
                        part_set_header: PartSetHeader {
                            total: 1,
                            hash: Some(H256::new([0xAA; 32])),
                        },
                    },
                    signatures: [
                        CommitSig::Absent,
                        CommitSig::Commit {
                            validator_address: H160::new([0xAA; 20]),
                            timestamp: Timestamp {
                                seconds: 102030.try_into().unwrap(),
                                nanos: 405060.try_into().unwrap(),
                            },
                            signature: [1, 2, 3].into(),
                        },
                        CommitSig::Nil {
                            validator_address: H160::new([0xAA; 20]),
                            timestamp: Timestamp {
                                seconds: 102030.try_into().unwrap(),
                                nanos: 405060.try_into().unwrap(),
                            },
                            signature: [1, 2, 3].into(),
                        },
                    ]
                    .to_vec(),
                },
            },
            validator_set: ValidatorSet {
                validators: [
                    Validator {
                        address: H160::new([0xAA; 20]),
                        pub_key: PublicKey::Ed25519([1, 2, 3].into()),
                        voting_power: 1.try_into().unwrap(),
                        proposer_priority: -1,
                    },
                    Validator {
                        address: H160::new([0xAA; 20]),
                        pub_key: PublicKey::Secp256k1([1, 2, 3].into()),
                        voting_power: 1.try_into().unwrap(),
                        proposer_priority: -1,
                    },
                ]
                .to_vec(),
                proposer: Validator {
                    address: H160::new([0xAA; 20]),
                    pub_key: PublicKey::Ed25519([1, 2, 3].into()),
                    voting_power: 1.try_into().unwrap(),
                    proposer_priority: -1,
                },
                total_voting_power: 123456789,
            },
            trusted_height: Height::new_with_revision(69, 420),
            trusted_validators: ValidatorSet {
                validators: [
                    Validator {
                        address: H160::new([0xAA; 20]),
                        pub_key: PublicKey::Bls12_381([1, 2, 3].into()),
                        voting_power: 1.try_into().unwrap(),
                        proposer_priority: -1,
                    },
                    Validator {
                        address: H160::new([0xAA; 20]),
                        pub_key: PublicKey::Bn254([1, 2, 3].into()),
                        voting_power: 1.try_into().unwrap(),
                        proposer_priority: -1,
                    },
                ]
                .to_vec(),
                proposer: Validator {
                    address: H160::new([0xAA; 20]),
                    pub_key: PublicKey::Ed25519([1, 2, 3].into()),
                    voting_power: 1.try_into().unwrap(),
                    proposer_priority: -1,
                },
                total_voting_power: 123456789,
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

    #[test]
    fn proto_iso() {
        let mut header = mk_header();

        // bls12_381 and bn254 aren't supported by the old tendermint types, which the proto encoding uses
        header.trusted_validators.validators = vec![];

        assert_codec_iso::<_, Proto>(&header);
    }
}

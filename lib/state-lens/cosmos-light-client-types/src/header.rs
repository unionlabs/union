use unionlabs::{
    bytes::Bytes,
    ibc::core::{client::height::Height, commitment::merkle_proof::MerkleProof},
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Header {
    pub l1_height: Height,
    pub l2_height: Height,
    pub l2_consensus_state_proof: MerkleProof,
    pub l2_consensus_state: Bytes,
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use alloy::sol_types::SolValue;
    use unionlabs::{
        encoding::{Encode, EthAbi},
        union::ics23,
    };

    use crate::Header;

    impl Encode<EthAbi> for Header {
        fn encode(self) -> Vec<u8> {
            Into::<SolHeader>::into(self).abi_encode_params()
        }
    }

    alloy::sol! {
        struct SolHeader {
            uint64 l1Height;
            uint64 l2Height;
            bytes l2InclusionProof;
            bytes l2ConsensusState;
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {}

    impl From<Header> for SolHeader {
        fn from(value: Header) -> Self {
            Self {
                l1Height: value.l1_height.height(),
                l2Height: value.l2_height.height(),
                l2InclusionProof: encode_merkle_proof_for_evm(value.l2_consensus_state_proof)
                    .into(),
                l2ConsensusState: value.l2_consensus_state.into(),
            }
        }
    }

    // FIXME: deduplicate with voyager/module/client/cometbls, in unionlabs?
    fn encode_merkle_proof_for_evm(
        proof: unionlabs::ibc::core::commitment::merkle_proof::MerkleProof,
    ) -> Vec<u8> {
        alloy::sol! {
            struct ExistenceProof {
                bytes key;
                bytes value;
                bytes leafPrefix;
                InnerOp[] path;
            }

            struct NonExistenceProof {
                bytes key;
                ExistenceProof left;
                ExistenceProof right;
            }

            struct InnerOp {
                bytes prefix;
                bytes suffix;
            }

            struct ProofSpec {
                uint256 childSize;
                uint256 minPrefixLength;
                uint256 maxPrefixLength;
            }
        }

        let merkle_proof = ics23::merkle_proof::MerkleProof::try_from(
            protos::ibc::core::commitment::v1::MerkleProof::from(proof),
        )
        .unwrap();

        let convert_inner_op = |i: unionlabs::union::ics23::inner_op::InnerOp| InnerOp {
            prefix: i.prefix.into(),
            suffix: i.suffix.into(),
        };

        let convert_existence_proof =
            |e: unionlabs::union::ics23::existence_proof::ExistenceProof| ExistenceProof {
                key: e.key.into(),
                value: e.value.into(),
                leafPrefix: e.leaf_prefix.into(),
                path: e.path.into_iter().map(convert_inner_op).collect(),
            };

        let exist_default = || ics23::existence_proof::ExistenceProof {
            key: vec![].into(),
            value: vec![].into(),
            leaf_prefix: vec![].into(),
            path: vec![],
        };

        match merkle_proof {
            ics23::merkle_proof::MerkleProof::Membership(a, b) => {
                (convert_existence_proof(a), convert_existence_proof(b)).abi_encode_params()
            }
            ics23::merkle_proof::MerkleProof::NonMembership(a, b) => (
                NonExistenceProof {
                    key: a.key.into(),
                    left: convert_existence_proof(a.left.unwrap_or_else(exist_default)),
                    right: convert_existence_proof(a.right.unwrap_or_else(exist_default)),
                },
                convert_existence_proof(b),
            )
                .abi_encode_params(),
        }
    }
}

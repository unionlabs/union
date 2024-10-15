use macros::model;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct MerklePath {
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    pub value: Vec<u8>,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string_list"))]
    pub proof_related_nodes: Vec<Vec<u8>>,
}

#[model(
    proto(
        raw(protos::union::ibc::lightclients::linea::v1::InclusionProof),
        into,
        from
    ),
    no_serde
)]
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct InclusionProof {
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    pub key: Vec<u8>,
    pub leaf_index: u64,
    pub proof: MerklePath,
}

#[model(
    proto(
        raw(protos::union::ibc::lightclients::linea::v1::NonInclusionProof),
        into,
        from
    ),
    no_serde
)]
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct NonInclusionProof {
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    pub key: Vec<u8>,
    pub left_leaf_index: u64,
    pub left_proof: MerklePath,
    pub right_leaf_index: u64,
    pub right_proof: MerklePath,
}

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(untagged)
)]
pub enum MerkleProof {
    Inclusion(InclusionProof),
    NonInclusion(NonInclusionProof),
}

#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct GetProof {
    pub account_proof: MerkleProof,
    pub storage_proofs: Vec<MerkleProof>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        errors::{required, MissingField},
        linea::proof::{InclusionProof, MerklePath, MerkleProof, NonInclusionProof},
    };

    impl From<MerkleProof> for protos::union::ibc::lightclients::linea::v1::MerkleProof {
        fn from(value: MerkleProof) -> Self {
            Self {
                proof: Some(match value {
                    MerkleProof::Inclusion(inclusion) => {
                        protos::union::ibc::lightclients::linea::v1::merkle_proof::Proof::Inclusion(
                            inclusion.into(),
                        )
                    }
                    MerkleProof::NonInclusion(noninclusion) => {
                        protos::union::ibc::lightclients::linea::v1::merkle_proof::Proof::Noninclusion(
                            noninclusion.into(),
                        )
                    }
                }),
            }
        }
    }

    impl TryFrom<protos::union::ibc::lightclients::linea::v1::MerkleProof> for MerkleProof {
        type Error = TryFromMerkleProofError;

        fn try_from(
            value: protos::union::ibc::lightclients::linea::v1::MerkleProof,
        ) -> Result<Self, Self::Error> {
            Ok(match required!(value.proof)? {
                protos::union::ibc::lightclients::linea::v1::merkle_proof::Proof::Inclusion(
                    inclusion,
                ) => MerkleProof::Inclusion(inclusion.try_into()?),
                protos::union::ibc::lightclients::linea::v1::merkle_proof::Proof::Noninclusion(
                    noninclusion,
                ) => MerkleProof::NonInclusion(noninclusion.try_into()?),
            })
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum TryFromMerkleProofError {
        #[error(transparent)]
        MissingField(#[from] MissingField),
    }

    impl From<NonInclusionProof> for protos::union::ibc::lightclients::linea::v1::NonInclusionProof {
        fn from(value: NonInclusionProof) -> Self {
            Self {
                key: value.key,
                left_leaf_index: value.left_leaf_index,
                left_proof: Some(value.left_proof.into()),
                right_leaf_index: value.right_leaf_index,
                right_proof: Some(value.right_proof.into()),
            }
        }
    }

    impl TryFrom<protos::union::ibc::lightclients::linea::v1::NonInclusionProof> for NonInclusionProof {
        type Error = TryFromMerkleProofError;

        fn try_from(
            value: protos::union::ibc::lightclients::linea::v1::NonInclusionProof,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                key: value.key,
                left_leaf_index: value.left_leaf_index,
                left_proof: required!(value.left_proof)?.into(),
                right_leaf_index: value.right_leaf_index,
                right_proof: required!(value.right_proof)?.into(),
            })
        }
    }

    impl From<InclusionProof> for protos::union::ibc::lightclients::linea::v1::InclusionProof {
        fn from(value: InclusionProof) -> Self {
            Self {
                key: value.key,
                leaf_index: value.leaf_index,
                merkle_path: Some(value.proof.into()),
            }
        }
    }

    impl TryFrom<protos::union::ibc::lightclients::linea::v1::InclusionProof> for InclusionProof {
        type Error = TryFromMerkleProofError;

        fn try_from(
            value: protos::union::ibc::lightclients::linea::v1::InclusionProof,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                key: value.key,
                leaf_index: value.leaf_index,
                proof: required!(value.merkle_path)?.into(),
            })
        }
    }

    impl From<MerklePath> for protos::union::ibc::lightclients::linea::v1::MerklePath {
        fn from(value: MerklePath) -> Self {
            Self {
                value: value.value,
                proof_related_nodes: value.proof_related_nodes,
            }
        }
    }

    impl From<protos::union::ibc::lightclients::linea::v1::MerklePath> for MerklePath {
        fn from(value: protos::union::ibc::lightclients::linea::v1::MerklePath) -> Self {
            Self {
                value: value.value,
                proof_related_nodes: value.proof_related_nodes,
            }
        }
    }
}

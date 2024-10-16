use macros::model;

use crate::aptos::sparse_merkle_proof::SparseMerkleProof;

#[model(proto(
    raw(protos::union::ibc::lightclients::movement::v1::StorageProof),
    into,
    from
))]
pub struct StorageProof {
    pub state_value: Option<StateValue>,
    pub proof: SparseMerkleProof,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StateValue {
    V0(Vec<u8>),
    WithMetadata {
        data: Vec<u8>,
        metadata: StateValueMetadata,
    },
}

impl StateValue {
    #[must_use]
    pub fn data(&self) -> &[u8] {
        match self {
            StateValue::V0(data) | StateValue::WithMetadata { data, .. } => data,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StateValueMetadata {
    V0 {
        deposit: u64,
        creation_time_usecs: u64,
    },
    V1 {
        slot_deposit: u64,
        bytes_deposit: u64,
        creation_time_usecs: u64,
    },
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        aptos::{
            sparse_merkle_proof::proto::TryFromSparseMerkleProofError,
            storage_proof::{StateValue, StateValueMetadata, StorageProof},
        },
        errors::{required, MissingField},
    };

    impl From<StorageProof> for protos::union::ibc::lightclients::movement::v1::StorageProof {
        fn from(value: StorageProof) -> Self {
            Self {
                proof: Some(value.proof.into()),
                state_value: value.state_value.map(Into::into),
            }
        }
    }

    impl From<StateValue>
        for protos::union::ibc::lightclients::movement::v1::storage_proof::StateValue
    {
        fn from(value: StateValue) -> Self {
            match value {
                StateValue::V0(data) => Self::V0(data),
                StateValue::WithMetadata { data, metadata } => Self::WithMetadata(
                    protos::union::ibc::lightclients::movement::v1::StateValueWithMetadata {
                        data,
                        metadata: Some(metadata.into()),
                    },
                ),
            }
        }
    }

    impl From<StateValueMetadata>
        for protos::union::ibc::lightclients::movement::v1::state_value_with_metadata::Metadata
    {
        fn from(value: StateValueMetadata) -> Self {
            match value {
                StateValueMetadata::V0 {
                    deposit,
                    creation_time_usecs,
                } => Self::V0(
                    protos::union::ibc::lightclients::movement::v1::StateValueMetadataV0 {
                        deposit,
                        creation_time_usecs,
                    },
                ),
                StateValueMetadata::V1 {
                    slot_deposit,
                    bytes_deposit,
                    creation_time_usecs,
                } => Self::V1(
                    protos::union::ibc::lightclients::movement::v1::StateValueMetadataV1 {
                        slot_deposit,
                        bytes_deposit,
                        creation_time_usecs,
                    },
                ),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum TryFromStorageProofError {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid state value")]
        StateValue(#[from] TryFromStateValueError),
        #[error("invalid proof")]
        Proof(#[from] TryFromSparseMerkleProofError),
    }

    impl TryFrom<protos::union::ibc::lightclients::movement::v1::StorageProof> for StorageProof {
        type Error = TryFromStorageProofError;

        fn try_from(
            value: protos::union::ibc::lightclients::movement::v1::StorageProof,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                state_value: value.state_value.map(TryInto::try_into).transpose()?,
                proof: required!(value.proof)?.try_into()?,
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum TryFromStateValueError {
        #[error(transparent)]
        MissingField(#[from] MissingField),
    }

    impl TryFrom<protos::union::ibc::lightclients::movement::v1::storage_proof::StateValue>
        for StateValue
    {
        type Error = TryFromStateValueError;

        fn try_from(
            value: protos::union::ibc::lightclients::movement::v1::storage_proof::StateValue,
        ) -> Result<Self, Self::Error> {
            Ok(match value {
                protos::union::ibc::lightclients::movement::v1::storage_proof::StateValue::V0(data) => Self::V0(data),
                protos::union::ibc::lightclients::movement::v1::storage_proof::StateValue::WithMetadata(
                    protos::union::ibc::lightclients::movement::v1::StateValueWithMetadata { data, metadata },
                ) => Self::WithMetadata {
                    data,
                    metadata: required!(metadata)?.into(),
                },
            })
        }
    }

    impl From<protos::union::ibc::lightclients::movement::v1::state_value_with_metadata::Metadata>
        for StateValueMetadata
    {
        fn from(
            value: protos::union::ibc::lightclients::movement::v1::state_value_with_metadata::Metadata,
        ) -> Self {
            match value {
                protos::union::ibc::lightclients::movement::v1::state_value_with_metadata::Metadata::V0(
                    protos::union::ibc::lightclients::movement::v1::StateValueMetadataV0 {
                        deposit,
                        creation_time_usecs,
                    },
                ) => StateValueMetadata::V0 {
                    deposit,
                    creation_time_usecs,
                },
                protos::union::ibc::lightclients::movement::v1::state_value_with_metadata::Metadata::V1(
                    protos::union::ibc::lightclients::movement::v1::StateValueMetadataV1 {
                        slot_deposit,
                        bytes_deposit,
                        creation_time_usecs,
                    },
                ) => StateValueMetadata::V1 {
                    slot_deposit,
                    bytes_deposit,
                    creation_time_usecs,
                },
            }
        }
    }
}

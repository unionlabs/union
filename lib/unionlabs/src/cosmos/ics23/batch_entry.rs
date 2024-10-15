use macros::model;

use crate::cosmos::ics23::{
    existence_proof::ExistenceProof, non_existence_proof::NonExistenceProof,
};

#[model(proto(raw(protos::cosmos::ics23::v1::BatchEntry), into, from))]
pub enum BatchEntry {
    Exist(ExistenceProof),
    Nonexist(NonExistenceProof),
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        cosmos::ics23::{
            batch_entry::BatchEntry, existence_proof::proto::TryFromExistenceProofError,
            non_existence_proof::proto::TryFromNonExistenceProofError,
        },
        errors::{required, MissingField},
    };

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum TryFromBatchEntryError {
        #[error(transparent)]
        MissingField(MissingField),
        #[error("invalid existence proof")]
        Exist(#[from] TryFromExistenceProofError),
        #[error("invalid non existence proof")]
        Nonexist(#[from] TryFromNonExistenceProofError),
    }

    impl TryFrom<protos::cosmos::ics23::v1::BatchEntry> for BatchEntry {
        type Error = TryFromBatchEntryError;

        fn try_from(value: protos::cosmos::ics23::v1::BatchEntry) -> Result<Self, Self::Error> {
            match required!(value.proof)? {
                protos::cosmos::ics23::v1::batch_entry::Proof::Exist(exist) => Ok(Self::Exist(
                    exist.try_into().map_err(TryFromBatchEntryError::Exist)?,
                )),
                protos::cosmos::ics23::v1::batch_entry::Proof::Nonexist(nonexist) => {
                    Ok(Self::Nonexist(
                        nonexist
                            .try_into()
                            .map_err(TryFromBatchEntryError::Nonexist)?,
                    ))
                }
            }
        }
    }

    impl From<BatchEntry> for protos::cosmos::ics23::v1::BatchEntry {
        fn from(value: BatchEntry) -> Self {
            Self {
                proof: Some(match value {
                    BatchEntry::Exist(exist) => {
                        protos::cosmos::ics23::v1::batch_entry::Proof::Exist(exist.into())
                    }
                    BatchEntry::Nonexist(nonexist) => {
                        protos::cosmos::ics23::v1::batch_entry::Proof::Nonexist(nonexist.into())
                    }
                }),
            }
        }
    }
}

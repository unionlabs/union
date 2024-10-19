use macros::model;

use crate::{
    cosmos::ics23::{
        existence_proof::{ExistenceProof, TryFromExistenceProofError},
        non_existence_proof::{NonExistenceProof, TryFromNonExistenceProofError},
    },
    errors::{required, MissingField},
};

#[model(proto(raw(protos::cosmos::ics23::v1::BatchEntry), into, from))]
pub enum BatchEntry {
    Exist(ExistenceProof),
    Nonexist(NonExistenceProof),
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromBatchEntryError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
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

#[cfg(feature = "ethabi")]
impl From<BatchEntry> for contracts::glue::CosmosIcs23V1BatchEntryData {
    fn from(value: BatchEntry) -> Self {
        match value {
            BatchEntry::Exist(exist) => contracts::glue::CosmosIcs23V1BatchEntryData {
                exist: exist.into(),
                ..Default::default()
            },
            BatchEntry::Nonexist(nonexist) => contracts::glue::CosmosIcs23V1BatchEntryData {
                nonexist: nonexist.into(),
                ..Default::default()
            },
        }
    }
}

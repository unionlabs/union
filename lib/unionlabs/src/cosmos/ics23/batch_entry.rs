use serde::{Deserialize, Serialize};

use crate::{
    cosmos::ics23::{existence_proof::ExistenceProof, non_existence_proof::NonExistenceProof},
    errors::{required, MissingField},
    TryFromProtoErrorOf,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BatchEntry {
    Exist(ExistenceProof),
    Nonexist(NonExistenceProof),
}

impl crate::Proto for BatchEntry {
    type Proto = protos::cosmos::ics23::v1::BatchEntry;
}

#[derive(Debug)]
pub enum TryFromBatchEntryError {
    MissingField(MissingField),
    Exist(TryFromProtoErrorOf<ExistenceProof>),
    Nonexist(TryFromProtoErrorOf<NonExistenceProof>),
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

use macros::model;
use serde::{Deserialize, Serialize};

use crate::{
    cosmos::ics23::{
        compressed_existence_proof::{
            CompressedExistenceProof, TryFromCompressedExistenceProofError,
        },
        compressed_non_existence_proof::{
            CompressedNonExistenceProof, TryFromCompressedNonExistenceProofError,
        },
    },
    errors::{required, MissingField},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "@type",
    content = "@value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[model(proto(raw(protos::cosmos::ics23::v1::CompressedBatchEntry), into, from))]
pub enum CompressedBatchEntry {
    Exist(CompressedExistenceProof),
    Nonexist(CompressedNonExistenceProof),
}

impl From<CompressedBatchEntry> for protos::cosmos::ics23::v1::CompressedBatchEntry {
    fn from(value: CompressedBatchEntry) -> Self {
        Self {
            proof: Some(match value {
                CompressedBatchEntry::Exist(exist) => {
                    protos::cosmos::ics23::v1::compressed_batch_entry::Proof::Exist(exist.into())
                }
                CompressedBatchEntry::Nonexist(nonexist) => {
                    protos::cosmos::ics23::v1::compressed_batch_entry::Proof::Nonexist(
                        nonexist.into(),
                    )
                }
            }),
        }
    }
}

#[derive(Debug)]
pub enum TryFromCompressedBatchEntryProofError {
    MissingField(MissingField),
    Exist(TryFromCompressedExistenceProofError),
    Nonexist(TryFromCompressedNonExistenceProofError),
}

impl TryFrom<protos::cosmos::ics23::v1::CompressedBatchEntry> for CompressedBatchEntry {
    type Error = TryFromCompressedBatchEntryProofError;

    fn try_from(
        value: protos::cosmos::ics23::v1::CompressedBatchEntry,
    ) -> Result<Self, Self::Error> {
        Ok(match required!(value.proof)? {
            protos::cosmos::ics23::v1::compressed_batch_entry::Proof::Exist(exist) => Self::Exist(
                exist
                    .try_into()
                    .map_err(TryFromCompressedBatchEntryProofError::Exist)?,
            ),
            protos::cosmos::ics23::v1::compressed_batch_entry::Proof::Nonexist(nonexist) => {
                Self::Nonexist(
                    nonexist
                        .try_into()
                        .map_err(TryFromCompressedBatchEntryProofError::Nonexist)?,
                )
            }
        })
    }
}

#[cfg(feature = "ethabi")]
impl From<CompressedBatchEntry> for contracts::glue::CosmosIcs23V1CompressedBatchEntryData {
    fn from(value: CompressedBatchEntry) -> Self {
        match value {
            CompressedBatchEntry::Exist(exist) => {
                contracts::glue::CosmosIcs23V1CompressedBatchEntryData {
                    exist: exist.into(),
                    ..Default::default()
                }
            }
            CompressedBatchEntry::Nonexist(nonexist) => {
                contracts::glue::CosmosIcs23V1CompressedBatchEntryData {
                    nonexist: nonexist.into(),
                    ..Default::default()
                }
            }
        }
    }
}

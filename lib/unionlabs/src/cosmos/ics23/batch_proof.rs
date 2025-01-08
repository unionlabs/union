use macros::model;

use crate::cosmos::ics23::batch_entry::{BatchEntry, TryFromBatchEntryError};

#[model(proto(raw(protos::cosmos::ics23::v1::BatchProof), into, from))]
pub struct BatchProof {
    pub entries: Vec<BatchEntry>,
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromBatchProofError {
    #[error("invalid entries")]
    Entries(#[from] TryFromBatchEntryError),
}

impl TryFrom<protos::cosmos::ics23::v1::BatchProof> for BatchProof {
    type Error = TryFromBatchProofError;

    fn try_from(value: protos::cosmos::ics23::v1::BatchProof) -> Result<Self, Self::Error> {
        Ok(Self {
            entries: value
                .entries
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()
                .map_err(TryFromBatchProofError::Entries)?,
        })
    }
}

impl From<BatchProof> for protos::cosmos::ics23::v1::BatchProof {
    fn from(value: BatchProof) -> Self {
        Self {
            entries: value
                .entries
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>(),
        }
    }
}

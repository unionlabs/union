use serde::{Deserialize, Serialize};

use crate::{cosmos::ics23::batch_entry::BatchEntry, TryFromProtoErrorOf};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct BatchProof {
    pub entries: Vec<BatchEntry>,
}

impl crate::Proto for BatchProof {
    type Proto = protos::cosmos::ics23::v1::BatchProof;
}

#[derive(Debug)]
pub enum TryFromBatchProofError {
    Entries(TryFromProtoErrorOf<BatchEntry>),
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

#[cfg(feature = "ethabi")]
impl From<BatchProof> for contracts::glue::CosmosIcs23V1BatchProofData {
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

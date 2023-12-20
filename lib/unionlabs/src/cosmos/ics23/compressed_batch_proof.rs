use serde::{Deserialize, Serialize};

use crate::{
    cosmos::ics23::{compressed_batch_entry::CompressedBatchEntry, inner_op::InnerOp},
    TryFromProtoErrorOf,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompressedBatchProof {
    pub entries: Vec<CompressedBatchEntry>,
    pub lookup_inners: Vec<InnerOp>,
}

impl crate::Proto for CompressedBatchProof {
    type Proto = protos::cosmos::ics23::v1::CompressedBatchProof;
}

#[derive(Debug)]
pub enum TryFromCompressedBatchProofProofError {
    Entries(TryFromProtoErrorOf<CompressedBatchEntry>),
    LookupInners(TryFromProtoErrorOf<InnerOp>),
}

impl TryFrom<protos::cosmos::ics23::v1::CompressedBatchProof> for CompressedBatchProof {
    type Error = TryFromCompressedBatchProofProofError;

    fn try_from(
        value: protos::cosmos::ics23::v1::CompressedBatchProof,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            entries: value
                .entries
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()
                .map_err(TryFromCompressedBatchProofProofError::Entries)?,
            lookup_inners: value
                .lookup_inners
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()
                .map_err(TryFromCompressedBatchProofProofError::LookupInners)?,
        })
    }
}

#[cfg(feature = "ethabi")]
impl From<CompressedBatchProof> for contracts::glue::CosmosIcs23V1CompressedBatchProofData {
    fn from(value: CompressedBatchProof) -> Self {
        Self {
            entries: value
                .entries
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>(),
            lookup_inners: value
                .lookup_inners
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>(),
        }
    }
}

impl From<CompressedBatchProof> for protos::cosmos::ics23::v1::CompressedBatchProof {
    fn from(value: CompressedBatchProof) -> Self {
        Self {
            entries: value
                .entries
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>(),
            lookup_inners: value
                .lookup_inners
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>(),
        }
    }
}

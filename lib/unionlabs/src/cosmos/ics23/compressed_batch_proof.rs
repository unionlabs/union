use macros::model;

use crate::cosmos::ics23::{
    compressed_batch_entry::{CompressedBatchEntry, TryFromCompressedBatchEntryProofError},
    inner_op::{InnerOp, TryFromInnerOpError},
};

#[model(proto(raw(protos::cosmos::ics23::v1::CompressedBatchProof), into, from))]
pub struct CompressedBatchProof {
    pub entries: Vec<CompressedBatchEntry>,
    pub lookup_inners: Vec<InnerOp>,
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromCompressedBatchProofProofError {
    #[error("invalid entries")]
    Entries(#[from] TryFromCompressedBatchEntryProofError),
    #[error("invalid lookup inners")]
    LookupInners(#[from] TryFromInnerOpError),
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

use macros::model;

use crate::{
    cosmos::ics23::{
        batch_proof::{BatchProof, TryFromBatchProofError},
        compressed_batch_proof::{CompressedBatchProof, TryFromCompressedBatchProofProofError},
        existence_proof::{ExistenceProof, TryFromExistenceProofError},
        non_existence_proof::{NonExistenceProof, TryFromNonExistenceProofError},
    },
    errors::{required, MissingField},
};

#[model(proto(raw(protos::cosmos::ics23::v1::CommitmentProof), into, from))]
pub enum CommitmentProof {
    Exist(ExistenceProof),
    Nonexist(NonExistenceProof),
    Batch(BatchProof),
    CompressedBatch(CompressedBatchProof),
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromCommitmentProofError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid existence proof")]
    Exist(#[from] TryFromExistenceProofError),
    #[error("invalid non existence proof")]
    Nonexist(#[from] TryFromNonExistenceProofError),
    #[error("invalid batch proof")]
    Batch(#[from] TryFromBatchProofError),
    #[error("invalid compressed batch proof")]
    CompressedBatch(#[from] TryFromCompressedBatchProofProofError),
}

impl TryFrom<protos::cosmos::ics23::v1::CommitmentProof> for CommitmentProof {
    type Error = TryFromCommitmentProofError;

    fn try_from(value: protos::cosmos::ics23::v1::CommitmentProof) -> Result<Self, Self::Error> {
        match required!(value.proof)? {
            protos::cosmos::ics23::v1::commitment_proof::Proof::Exist(exist) => Ok(Self::Exist(
                exist
                    .try_into()
                    .map_err(TryFromCommitmentProofError::Exist)?,
            )),
            protos::cosmos::ics23::v1::commitment_proof::Proof::Nonexist(nonexist) => {
                Ok(Self::Nonexist(
                    nonexist
                        .try_into()
                        .map_err(TryFromCommitmentProofError::Nonexist)?,
                ))
            }
            protos::cosmos::ics23::v1::commitment_proof::Proof::Batch(batch) => Ok(Self::Batch(
                batch
                    .try_into()
                    .map_err(TryFromCommitmentProofError::Batch)?,
            )),
            protos::cosmos::ics23::v1::commitment_proof::Proof::Compressed(compressed) => {
                Ok(Self::CompressedBatch(
                    compressed
                        .try_into()
                        .map_err(TryFromCommitmentProofError::CompressedBatch)?,
                ))
            }
        }
    }
}

impl From<CommitmentProof> for protos::cosmos::ics23::v1::CommitmentProof {
    fn from(value: CommitmentProof) -> Self {
        Self {
            proof: Some(match value {
                CommitmentProof::Exist(exist) => {
                    protos::cosmos::ics23::v1::commitment_proof::Proof::Exist(exist.into())
                }
                CommitmentProof::Nonexist(nonexist) => {
                    protos::cosmos::ics23::v1::commitment_proof::Proof::Nonexist(nonexist.into())
                }
                CommitmentProof::Batch(batch) => {
                    protos::cosmos::ics23::v1::commitment_proof::Proof::Batch(batch.into())
                }
                CommitmentProof::CompressedBatch(compressed_batch) => {
                    protos::cosmos::ics23::v1::commitment_proof::Proof::Compressed(
                        compressed_batch.into(),
                    )
                }
            }),
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<CommitmentProof> for contracts::glue::CosmosIcs23V1CommitmentProofData {
    fn from(value: CommitmentProof) -> Self {
        match value {
            CommitmentProof::Exist(exist) => contracts::glue::CosmosIcs23V1CommitmentProofData {
                exist: exist.into(),
                ..Default::default()
            },
            CommitmentProof::Nonexist(nonexist) => {
                contracts::glue::CosmosIcs23V1CommitmentProofData {
                    nonexist: nonexist.into(),
                    ..Default::default()
                }
            }
            CommitmentProof::Batch(batch) => contracts::glue::CosmosIcs23V1CommitmentProofData {
                batch: batch.into(),
                ..Default::default()
            },
            CommitmentProof::CompressedBatch(compressed_batch) => {
                contracts::glue::CosmosIcs23V1CommitmentProofData {
                    compressed: compressed_batch.into(),
                    ..Default::default()
                }
            }
        }
    }
}

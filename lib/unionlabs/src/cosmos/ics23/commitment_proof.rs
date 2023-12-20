use serde::{Deserialize, Serialize};

use crate::{
    cosmos::ics23::{
        batch_proof::BatchProof, compressed_batch_proof::CompressedBatchProof,
        existence_proof::ExistenceProof, non_existence_proof::NonExistenceProof,
    },
    errors::{required, MissingField},
    TryFromProtoErrorOf,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommitmentProof {
    Exist(ExistenceProof),
    Nonexist(NonExistenceProof),
    Batch(BatchProof),
    CompressedBatch(CompressedBatchProof),
}

impl crate::Proto for CommitmentProof {
    type Proto = protos::cosmos::ics23::v1::CommitmentProof;
}

#[derive(Debug)]
pub enum TryFromCommitmentProofError {
    MissingField(MissingField),
    Exist(TryFromProtoErrorOf<ExistenceProof>),
    Nonexist(TryFromProtoErrorOf<NonExistenceProof>),
    Batch(TryFromProtoErrorOf<BatchProof>),
    CompressedBatch(TryFromProtoErrorOf<CompressedBatchProof>),
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

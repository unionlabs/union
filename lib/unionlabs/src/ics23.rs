use serde::{Deserialize, Serialize};

use crate::{
    errors::{required, MissingField},
    InlineFields,
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InnerOp {
    pub hash: u8,
    pub prefix: Vec<u8>,
    pub suffix: Vec<u8>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeafOp {
    pub hash: u8,
    pub prehash_key: u8,
    pub prehash_value: u8,
    pub length: u8,
    pub prefix: Vec<u8>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExistenceProof {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub leaf: LeafOp,
    pub path: Vec<InnerOp>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NonExistenceProof {
    pub key: Vec<u8>,
    pub left: ExistenceProof,
    pub right: ExistenceProof,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BatchEntryProof {
    Exist(ExistenceProof),
    Nonexist(NonExistenceProof),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatchEntry {
    pub proof: BatchEntryProof,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatchProof {
    pub entries: Vec<BatchEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompressedBatchProof {
    pub entries: Vec<CompressedBatchEntry>,
    pub lookup_inners: Vec<InnerOp>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompressedBatchEntry {
    pub proof: CompressedProof,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CompressedProof {
    Exist(CompressedExistenceProof),
    Nonexist(CompressedNonExistenceProof),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompressedExistenceProof {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub leaf: LeafOp,
    pub path: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompressedNonExistenceProof {
    pub key: Vec<u8>,
    pub left: CompressedExistenceProof,
    pub right: CompressedExistenceProof,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommitmentProof {
    Exist(ExistenceProof),
    Nonexist(NonExistenceProof),
    Batch(BatchProof),
    CompressedBatch(CompressedBatchProof),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MerkleProof {
    pub proofs: Vec<CommitmentProof>,
}

#[derive(Debug)]
pub enum TryFromMerkleProofError {
    MissingField(MissingField),
    InvalidEnumLength,
}

impl TryFrom<protos::cosmos::ics23::v1::InnerOp> for InnerOp {
    type Error = TryFromMerkleProofError;

    fn try_from(value: protos::cosmos::ics23::v1::InnerOp) -> Result<Self, Self::Error> {
        Ok(Self {
            hash: value
                .hash
                .try_into()
                .map_err(|_| TryFromMerkleProofError::InvalidEnumLength)?,
            prefix: value.prefix,
            suffix: value.suffix,
        })
    }
}

impl TryFrom<protos::cosmos::ics23::v1::LeafOp> for LeafOp {
    type Error = TryFromMerkleProofError;

    fn try_from(value: protos::cosmos::ics23::v1::LeafOp) -> Result<Self, Self::Error> {
        Ok(Self {
            hash: value
                .hash
                .try_into()
                .map_err(|_| TryFromMerkleProofError::InvalidEnumLength)?,
            prehash_key: value
                .prehash_key
                .try_into()
                .map_err(|_| TryFromMerkleProofError::InvalidEnumLength)?,
            prehash_value: value
                .prehash_value
                .try_into()
                .map_err(|_| TryFromMerkleProofError::InvalidEnumLength)?,
            length: value
                .length
                .try_into()
                .map_err(|_| TryFromMerkleProofError::InvalidEnumLength)?,
            prefix: value.prefix,
        })
    }
}

impl TryFrom<protos::cosmos::ics23::v1::ExistenceProof> for ExistenceProof {
    type Error = TryFromMerkleProofError;

    fn try_from(value: protos::cosmos::ics23::v1::ExistenceProof) -> Result<Self, Self::Error> {
        Ok(Self {
            key: value.key,
            value: value.value,
            leaf: required!(value.leaf)?.try_into()?,
            path: value
                .path
                .into_iter()
                .map(TryFrom::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl TryFrom<protos::cosmos::ics23::v1::NonExistenceProof> for NonExistenceProof {
    type Error = TryFromMerkleProofError;

    fn try_from(value: protos::cosmos::ics23::v1::NonExistenceProof) -> Result<Self, Self::Error> {
        Ok(Self {
            key: value.key,
            left: required!(value.left)?.try_into()?,
            right: required!(value.right)?.try_into()?,
        })
    }
}

impl TryFrom<protos::cosmos::ics23::v1::batch_entry::Proof> for BatchEntryProof {
    type Error = TryFromMerkleProofError;

    fn try_from(value: protos::cosmos::ics23::v1::batch_entry::Proof) -> Result<Self, Self::Error> {
        match value {
            protos::cosmos::ics23::v1::batch_entry::Proof::Exist(exist) => {
                Ok(Self::Exist(exist.try_into()?))
            }
            protos::cosmos::ics23::v1::batch_entry::Proof::Nonexist(nonexist) => {
                Ok(Self::Nonexist(nonexist.try_into()?))
            }
        }
    }
}

impl TryFrom<protos::cosmos::ics23::v1::BatchEntry> for BatchEntry {
    type Error = TryFromMerkleProofError;

    fn try_from(value: protos::cosmos::ics23::v1::BatchEntry) -> Result<Self, Self::Error> {
        Ok(Self {
            proof: required!(value.proof)?.try_into()?,
        })
    }
}

impl TryFrom<protos::cosmos::ics23::v1::BatchProof> for BatchProof {
    type Error = TryFromMerkleProofError;

    fn try_from(value: protos::cosmos::ics23::v1::BatchProof) -> Result<Self, Self::Error> {
        Ok(Self {
            entries: value
                .entries
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl TryFrom<protos::cosmos::ics23::v1::CompressedExistenceProof> for CompressedExistenceProof {
    type Error = TryFromMerkleProofError;

    fn try_from(
        value: protos::cosmos::ics23::v1::CompressedExistenceProof,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            key: value.key,
            value: value.value,
            leaf: required!(value.leaf)?.try_into()?,
            path: value.path,
        })
    }
}

impl TryFrom<protos::cosmos::ics23::v1::CompressedNonExistenceProof>
    for CompressedNonExistenceProof
{
    type Error = TryFromMerkleProofError;

    fn try_from(
        value: protos::cosmos::ics23::v1::CompressedNonExistenceProof,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            key: value.key,
            left: required!(value.left)?.try_into()?,
            right: required!(value.right)?.try_into()?,
        })
    }
}

impl TryFrom<protos::cosmos::ics23::v1::compressed_batch_entry::Proof> for CompressedProof {
    type Error = TryFromMerkleProofError;

    fn try_from(
        value: protos::cosmos::ics23::v1::compressed_batch_entry::Proof,
    ) -> Result<Self, Self::Error> {
        match value {
            protos::cosmos::ics23::v1::compressed_batch_entry::Proof::Exist(exist) => {
                Ok(Self::Exist(exist.try_into()?))
            }
            protos::cosmos::ics23::v1::compressed_batch_entry::Proof::Nonexist(nonexist) => {
                Ok(Self::Nonexist(nonexist.try_into()?))
            }
        }
    }
}

impl TryFrom<protos::cosmos::ics23::v1::CompressedBatchEntry> for CompressedBatchEntry {
    type Error = TryFromMerkleProofError;

    fn try_from(
        value: protos::cosmos::ics23::v1::CompressedBatchEntry,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            proof: required!(value.proof)?.try_into()?,
        })
    }
}

impl TryFrom<protos::cosmos::ics23::v1::CompressedBatchProof> for CompressedBatchProof {
    type Error = TryFromMerkleProofError;

    fn try_from(
        value: protos::cosmos::ics23::v1::CompressedBatchProof,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            entries: value
                .entries
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            lookup_inners: value
                .lookup_inners
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl TryFrom<protos::cosmos::ics23::v1::CommitmentProof> for CommitmentProof {
    type Error = TryFromMerkleProofError;

    fn try_from(value: protos::cosmos::ics23::v1::CommitmentProof) -> Result<Self, Self::Error> {
        match required!(value.proof)? {
            protos::cosmos::ics23::v1::commitment_proof::Proof::Exist(exist) => {
                Ok(Self::Exist(exist.try_into()?))
            }
            protos::cosmos::ics23::v1::commitment_proof::Proof::Nonexist(nonexist) => {
                Ok(Self::Nonexist(nonexist.try_into()?))
            }
            protos::cosmos::ics23::v1::commitment_proof::Proof::Batch(batch) => {
                Ok(Self::Batch(batch.try_into()?))
            }
            protos::cosmos::ics23::v1::commitment_proof::Proof::Compressed(compressed) => {
                Ok(Self::CompressedBatch(compressed.try_into()?))
            }
        }
    }
}

impl TryFrom<protos::ibc::core::commitment::v1::MerkleProof> for MerkleProof {
    type Error = TryFromMerkleProofError;

    fn try_from(
        value: protos::ibc::core::commitment::v1::MerkleProof,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            proofs: value
                .proofs
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

#[cfg(feature = "ethabi")]
impl From<InnerOp> for contracts::glue::CosmosIcs23V1InnerOpData {
    fn from(value: InnerOp) -> Self {
        Self {
            hash: value.hash,
            prefix: value.prefix.into(),
            suffix: value.suffix.into(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<LeafOp> for contracts::glue::CosmosIcs23V1LeafOpData {
    fn from(value: LeafOp) -> Self {
        Self {
            hash: value.hash,
            prehash_key: value.prehash_key,
            prehash_value: value.prehash_value,
            length: value.length,
            prefix: value.prefix.into(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<ExistenceProof> for contracts::glue::CosmosIcs23V1ExistenceProofData {
    fn from(value: ExistenceProof) -> Self {
        Self {
            key: value.key.into(),
            value: value.value.into(),
            leaf: value.leaf.into(),
            path: value.path.into_iter().map(Into::into).collect::<Vec<_>>(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<NonExistenceProof> for contracts::glue::CosmosIcs23V1NonExistenceProofData {
    fn from(value: NonExistenceProof) -> Self {
        Self {
            key: value.key.into(),
            left: value.left.into(),
            right: value.right.into(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<BatchEntryProof> for contracts::glue::CosmosIcs23V1BatchEntryData {
    fn from(value: BatchEntryProof) -> Self {
        match value {
            BatchEntryProof::Exist(exist) => contracts::glue::CosmosIcs23V1BatchEntryData {
                exist: exist.into(),
                ..Default::default()
            },
            BatchEntryProof::Nonexist(nonexist) => contracts::glue::CosmosIcs23V1BatchEntryData {
                nonexist: nonexist.into(),
                ..Default::default()
            },
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<BatchProof> for contracts::glue::CosmosIcs23V1BatchProofData {
    fn from(value: BatchProof) -> Self {
        Self {
            entries: value
                .entries
                .into_iter()
                .map(|x| x.proof)
                .map(Into::into)
                .collect::<Vec<_>>(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<CompressedExistenceProof> for contracts::glue::CosmosIcs23V1CompressedExistenceProofData {
    fn from(value: CompressedExistenceProof) -> Self {
        Self {
            key: value.key.into(),
            value: value.value.into(),
            leaf: value.leaf.into(),
            path: value.path,
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<CompressedNonExistenceProof>
    for contracts::glue::CosmosIcs23V1CompressedNonExistenceProofData
{
    fn from(value: CompressedNonExistenceProof) -> Self {
        Self {
            key: value.key.into(),
            left: value.left.into(),
            right: value.right.into(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<CompressedProof> for contracts::glue::CosmosIcs23V1CompressedBatchEntryData {
    fn from(value: CompressedProof) -> Self {
        match value {
            CompressedProof::Exist(exist) => {
                contracts::glue::CosmosIcs23V1CompressedBatchEntryData {
                    exist: exist.into(),
                    ..Default::default()
                }
            }
            CompressedProof::Nonexist(nonexist) => {
                contracts::glue::CosmosIcs23V1CompressedBatchEntryData {
                    nonexist: nonexist.into(),
                    ..Default::default()
                }
            }
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<CompressedBatchProof> for contracts::glue::CosmosIcs23V1CompressedBatchProofData {
    fn from(value: CompressedBatchProof) -> Self {
        Self {
            entries: value
                .entries
                .into_iter()
                .map(|x| x.proof)
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
            CommitmentProof::CompressedBatch(compressed) => {
                contracts::glue::CosmosIcs23V1CommitmentProofData {
                    compressed: compressed.into(),
                    ..Default::default()
                }
            }
        }
    }
}

#[cfg(feature = "ethabi")]
impl crate::EthAbi for MerkleProof {
    type EthAbi = InlineFields<contracts::glue::IbcCoreCommitmentV1MerkleProofData>;
}

#[cfg(feature = "ethabi")]
impl From<MerkleProof> for InlineFields<contracts::glue::IbcCoreCommitmentV1MerkleProofData> {
    fn from(value: MerkleProof) -> Self {
        contracts::glue::IbcCoreCommitmentV1MerkleProofData {
            proofs: value.proofs.into_iter().map(Into::into).collect::<Vec<_>>(),
        }
        .into()
    }
}

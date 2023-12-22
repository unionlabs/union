use serde::{Deserialize, Serialize};

use crate::{cosmos::ics23::commitment_proof::CommitmentProof, TryFromProtoErrorOf};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MerkleProof {
    pub proofs: Vec<CommitmentProof>,
}

impl crate::Proto for MerkleProof {
    type Proto = protos::ibc::core::commitment::v1::MerkleProof;
}

#[derive(Debug)]
pub enum TryFromMerkleProofError {
    Proofs(TryFromProtoErrorOf<CommitmentProof>),
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
                .collect::<Result<Vec<_>, _>>()
                .map_err(TryFromMerkleProofError::Proofs)?,
        })
    }
}

impl From<MerkleProof> for protos::ibc::core::commitment::v1::MerkleProof {
    fn from(value: MerkleProof) -> Self {
        Self {
            proofs: value.proofs.into_iter().map(Into::into).collect(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl crate::EthAbi for MerkleProof {
    type EthAbi = crate::InlineFields<contracts::glue::IbcCoreCommitmentV1MerkleProofData>;
}

#[cfg(feature = "ethabi")]
impl From<MerkleProof>
    for crate::InlineFields<contracts::glue::IbcCoreCommitmentV1MerkleProofData>
{
    fn from(value: MerkleProof) -> Self {
        crate::InlineFields(contracts::glue::IbcCoreCommitmentV1MerkleProofData {
            proofs: value.proofs.into_iter().map(Into::into).collect::<Vec<_>>(),
        })
    }
}

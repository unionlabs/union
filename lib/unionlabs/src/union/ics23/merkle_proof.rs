use macros::model;

use crate::{
    errors::{ExpectedLength, InvalidLength, MissingField},
    union::ics23::{existence_proof::ExistenceProof, non_existence_proof::NonExistenceProof},
};

#[model(proto(raw(protos::ibc::core::commitment::v1::MerkleProof), into, from))]
// TODO: Rename to optimized merkle proof
pub enum MerkleProof {
    Membership(ExistenceProof, ExistenceProof),
    NonMembership(NonExistenceProof, ExistenceProof),
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromMerkleProofError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("unable to decode existence proof")]
    Existence(#[from] crate::union::ics23::existence_proof::TryFromExistenceProofError),
    #[error("unable to decode non existence proof")]
    NonExistence(#[from] crate::union::ics23::non_existence_proof::TryFromNonExistenceProofError),
    #[error("invalid commitment proof type")]
    InvalidCommitmentProofType,
    #[error("invalid proofs length")]
    InvalidProofsLength(#[from] InvalidLength),
}

impl TryFrom<protos::ibc::core::commitment::v1::MerkleProof> for MerkleProof {
    type Error = TryFromMerkleProofError;

    fn try_from(
        value: protos::ibc::core::commitment::v1::MerkleProof,
    ) -> Result<Self, Self::Error> {
        use protos::cosmos::ics23::v1::{
            commitment_proof::Proof as RawProof, CommitmentProof as RawCommitmentProof,
        };

        let proofs: [_; 2] = value.proofs.try_into().map_err(|invalid: Vec<_>| {
            TryFromMerkleProofError::InvalidProofsLength(InvalidLength {
                expected: ExpectedLength::Exact(2),
                found: invalid.len(),
            })
        })?;

        match proofs {
            [RawCommitmentProof {
                proof: Some(RawProof::Exist(exist_1)),
            }, RawCommitmentProof {
                proof: Some(RawProof::Exist(exist_2)),
            }] => Ok(Self::Membership(exist_1.try_into()?, exist_2.try_into()?)),
            [RawCommitmentProof {
                proof: Some(RawProof::Nonexist(non_exist)),
            }, RawCommitmentProof {
                proof: Some(RawProof::Exist(exist)),
            }] => Ok(Self::NonMembership(
                non_exist.try_into()?,
                exist.try_into()?,
            )),
            [_, _] => Err(TryFromMerkleProofError::InvalidCommitmentProofType),
        }
    }
}

impl From<MerkleProof> for protos::ibc::core::commitment::v1::MerkleProof {
    fn from(value: MerkleProof) -> Self {
        use protos::cosmos::ics23::v1::{
            commitment_proof::Proof as RawProof, CommitmentProof as RawCommitmentProof,
        };

        match value {
            MerkleProof::Membership(a, b) => Self {
                proofs: vec![
                    RawCommitmentProof {
                        proof: Some(RawProof::Exist(a.into())),
                    },
                    RawCommitmentProof {
                        proof: Some(RawProof::Exist(b.into())),
                    },
                ],
            },
            MerkleProof::NonMembership(a, b) => Self {
                proofs: vec![
                    RawCommitmentProof {
                        proof: Some(RawProof::Nonexist(a.into())),
                    },
                    RawCommitmentProof {
                        proof: Some(RawProof::Exist(b.into())),
                    },
                ],
            },
        }
    }
}

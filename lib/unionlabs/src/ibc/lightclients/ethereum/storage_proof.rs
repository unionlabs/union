use macros::model;

use crate::ibc::lightclients::ethereum::proof::{Proof, TryFromProofError};

#[model(proto(
    raw(protos::union::ibc::lightclients::ethereum::v1::StorageProof),
    into,
    from
))]
pub struct StorageProof {
    pub proofs: Vec<Proof>,
}

impl From<StorageProof> for protos::union::ibc::lightclients::ethereum::v1::StorageProof {
    fn from(value: StorageProof) -> Self {
        Self {
            proofs: value.proofs.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromStorageProofError {
    #[error("unable to decode proofs")]
    Proofs(#[from] TryFromProofError),
}

impl TryFrom<protos::union::ibc::lightclients::ethereum::v1::StorageProof> for StorageProof {
    type Error = TryFromStorageProofError;

    fn try_from(
        value: protos::union::ibc::lightclients::ethereum::v1::StorageProof,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            proofs: value
                .proofs
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

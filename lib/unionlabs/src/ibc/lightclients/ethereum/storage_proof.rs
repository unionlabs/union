use serde::{Deserialize, Serialize};

use crate::{
    ibc::lightclients::ethereum::proof::{Proof, TryFromProofError},
    Proto, TypeUrl,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
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

#[derive(Debug)]
pub enum TryFromStorageProofError {
    Proofs(TryFromProofError),
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
                .map(|proof| proof.try_into().map_err(TryFromStorageProofError::Proofs))
                .collect::<Result<_, _>>()?,
        })
    }
}

impl Proto for StorageProof {
    type Proto = protos::union::ibc::lightclients::ethereum::v1::StorageProof;
}

impl TypeUrl for protos::union::ibc::lightclients::ethereum::v1::StorageProof {
    const TYPE_URL: &'static str = "/union.ibc.lightclients.ethereum.v1.StorageProof";
}

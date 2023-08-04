use serde::{Deserialize, Serialize};

use crate::{ibc::lightclients::ethereum::proof::Proof, Proto, TypeUrl};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

impl From<protos::union::ibc::lightclients::ethereum::v1::StorageProof> for StorageProof {
    fn from(value: protos::union::ibc::lightclients::ethereum::v1::StorageProof) -> Self {
        Self {
            proofs: value.proofs.into_iter().map(Into::into).collect(),
        }
    }
}

impl Proto for StorageProof {
    type Proto = protos::union::ibc::lightclients::ethereum::v1::StorageProof;
}

impl TypeUrl for protos::union::ibc::lightclients::ethereum::v1::StorageProof {
    const TYPE_URL: &'static str = "/union.ibc.lightclients.ethereum.v1.StorageProof";
}

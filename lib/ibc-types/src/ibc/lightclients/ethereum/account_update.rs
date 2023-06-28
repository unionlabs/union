use serde::{Deserialize, Serialize};

use crate::{FromProto, IntoProto, TypeUrl};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AccountUpdate {
    pub proofs: Vec<AccountProof>,
}

impl From<AccountUpdate> for protos::union::ibc::lightclients::ethereum::v1::AccountUpdate {
    fn from(value: AccountUpdate) -> Self {
        Self {
            proofs: value
                .proofs
                .into_iter()
                .map(
                    |proof| protos::union::ibc::lightclients::ethereum::v1::Proof {
                        key: proof.address,
                        value: proof.storage_hash,
                        proof: proof.proof,
                    },
                )
                .collect(),
        }
    }
}

impl From<protos::union::ibc::lightclients::ethereum::v1::AccountUpdate> for AccountUpdate {
    fn from(value: protos::union::ibc::lightclients::ethereum::v1::AccountUpdate) -> Self {
        Self {
            proofs: value
                .proofs
                .into_iter()
                .map(|proof| AccountProof {
                    address: proof.key,
                    storage_hash: proof.value,
                    proof: proof.proof,
                })
                .collect(),
        }
    }
}

impl IntoProto for AccountUpdate {
    type Proto = protos::union::ibc::lightclients::ethereum::v1::AccountUpdate;
}

impl FromProto for AccountUpdate {
    type Proto = protos::union::ibc::lightclients::ethereum::v1::AccountUpdate;
}

impl TypeUrl for protos::union::ibc::lightclients::ethereum::v1::AccountUpdate {
    const TYPE_URL: &'static str = "/union.ibc.lightclients.ethereum.v1.AccountUpdate";
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AccountProof {
    pub address: Vec<u8>,
    pub storage_hash: Vec<u8>,
    pub proof: Vec<Vec<u8>>,
}

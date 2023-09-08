use serde::{Deserialize, Serialize};

use crate::{ibc::lightclients::ethereum::proof::Proof, Proto, TypeUrl};

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
                .map(|proof| Into::<Proof>::into(proof).into())
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
                .map(|proof| Into::<Proof>::into(proof).into())
                .collect(),
        }
    }
}

impl Proto for AccountUpdate {
    type Proto = protos::union::ibc::lightclients::ethereum::v1::AccountUpdate;
}

impl TypeUrl for protos::union::ibc::lightclients::ethereum::v1::AccountUpdate {
    const TYPE_URL: &'static str = "/union.ibc.lightclients.ethereum.v1.AccountUpdate";
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AccountProof {
    #[serde(with = "::serde_utils::base64")]
    pub address: Vec<u8>,
    #[serde(with = "::serde_utils::base64")]
    pub storage_hash: Vec<u8>,
    #[serde(with = "::serde_utils::inner_base64")]
    pub proof: Vec<Vec<u8>>,
}

impl From<Proof> for AccountProof {
    fn from(value: Proof) -> Self {
        Self {
            address: value.key,
            storage_hash: value.value,
            proof: value.proof,
        }
    }
}

impl From<AccountProof> for Proof {
    fn from(value: AccountProof) -> Self {
        Self {
            key: value.address,
            value: value.storage_hash,
            proof: value.proof,
        }
    }
}

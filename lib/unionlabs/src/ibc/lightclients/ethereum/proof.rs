use std::fmt::Debug;

use serde::{Deserialize, Serialize};

// REVIEW: H256 or actual arbitrary bytes?
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Proof {
    #[serde(with = "::serde_utils::base64")]
    pub key: Vec<u8>,
    #[serde(with = "::serde_utils::base64")]
    pub value: Vec<u8>,
    #[serde(with = "::serde_utils::inner_base64")]
    pub proof: Vec<Vec<u8>>,
}

impl Debug for Proof {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccountUpdate")
            .field("key", &serde_utils::to_hex(&self.key))
            .field("value", &serde_utils::to_hex(&self.value))
            .field(
                "proof",
                &self
                    .proof
                    .iter()
                    .map(serde_utils::to_hex)
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl From<Proof> for protos::union::ibc::lightclients::ethereum::v1::Proof {
    fn from(value: Proof) -> Self {
        Self {
            key: value.key,
            value: value.value,
            proof: value.proof,
        }
    }
}

impl From<protos::union::ibc::lightclients::ethereum::v1::Proof> for Proof {
    fn from(value: protos::union::ibc::lightclients::ethereum::v1::Proof) -> Self {
        Self {
            key: value.key,
            value: value.value,
            proof: value.proof,
        }
    }
}

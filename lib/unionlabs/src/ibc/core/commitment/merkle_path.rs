use serde::{Deserialize, Serialize};

use crate::{Proto, TypeUrl};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MerklePath {
    pub key_path: Vec<String>,
}

impl From<MerklePath> for protos::ibc::core::commitment::v1::MerklePath {
    fn from(value: MerklePath) -> Self {
        Self {
            key_path: value.key_path,
        }
    }
}

impl From<protos::ibc::core::commitment::v1::MerklePath> for MerklePath {
    fn from(value: protos::ibc::core::commitment::v1::MerklePath) -> Self {
        Self {
            key_path: value.key_path,
        }
    }
}

impl Proto for MerklePath {
    type Proto = protos::ibc::core::commitment::v1::MerklePath;
}

impl TypeUrl for protos::ibc::core::commitment::v1::MerklePath {
    const TYPE_URL: &'static str = "/ibc.core.commitment.v1.MerklePath";
}

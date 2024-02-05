use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GenesisMetadata {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

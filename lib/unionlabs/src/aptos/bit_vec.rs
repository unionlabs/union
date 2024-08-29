use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct BitVec {
    #[serde(with = "serde_bytes")]
    pub inner: Vec<u8>,
}

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::Binary;

/// Simplified version of the PageRequest type for pagination from the cosmos-sdk
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq, JsonSchema)]
pub struct PageRequest {
    pub key: Option<Binary>,
    pub limit: u32,
    pub reverse: bool,
}

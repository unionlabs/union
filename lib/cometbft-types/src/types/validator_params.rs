use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidatorParams {
    pub pub_key_types: Vec<String>,
}

// TODO: Implement proto
// #[cfg(feature = "proto")]
// pub mod proto {}

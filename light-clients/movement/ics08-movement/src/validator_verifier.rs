use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize};

use crate::types::{AccountAddress, PublicKey};

/// Supports validation of signatures for known authors with individual voting powers. This struct
/// can be used for all signature verification operations including block and network signature
/// verification, respectively.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ValidatorVerifier {
    /// A vector of each validator's on-chain account address to its pubkeys and voting power.
    validator_infos: Vec<ValidatorConsensusInfo>,
}

/// Helper struct to manage validator information for validation
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ValidatorConsensusInfo {
    pub address: AccountAddress,
    pub public_key: PublicKey,
    pub voting_power: u64,
}

use macros::model;

use crate::aptos::{account::AccountAddress, public_key::PublicKey};

/// Supports validation of signatures for known authors with individual voting powers. This struct
/// can be used for all signature verification operations including block and network signature
/// verification, respectively.
#[model]
pub struct ValidatorVerifier {
    /// A vector of each validator's on-chain account address to its pubkeys and voting power.
    pub validator_infos: Vec<ValidatorConsensusInfo>,
}

/// Helper struct to manage validator information for validation
#[model]
pub struct ValidatorConsensusInfo {
    pub address: AccountAddress,
    pub public_key: PublicKey,
    pub voting_power: u64,
}

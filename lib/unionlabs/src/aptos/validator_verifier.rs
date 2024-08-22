use serde::{Deserialize, Serialize};

use super::public_key::PublicKey;
use crate::aptos::account::AccountAddress;

/// Supports validation of signatures for known authors with individual voting powers. This struct
/// can be used for all signature verification operations including block and network signature
/// verification, respectively.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ValidatorVerifier {
    /// A vector of each validator's on-chain account address to its pubkeys and voting power.
    pub validator_infos: Vec<ValidatorConsensusInfo>,
}

/// Helper struct to manage validator information for validation
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ValidatorConsensusInfo {
    pub address: AccountAddress,
    pub public_key: PublicKey,
    pub voting_power: u64,
}

impl From<ValidatorVerifier> for protos::union::ibc::lightclients::movement::v1::ValidatorVerifier {
    fn from(value: ValidatorVerifier) -> Self {
        Self {
            validator_infos: value.validator_infos.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<ValidatorConsensusInfo>
    for protos::union::ibc::lightclients::movement::v1::ValidatorConsensusInfo
{
    fn from(value: ValidatorConsensusInfo) -> Self {
        Self {
            address: value.address.0.to_vec(),
            public_key: Some(value.public_key.into()),
            voting_power: value.voting_power,
        }
    }
}

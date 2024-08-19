use serde::{Deserialize, Deserializer, Serialize};

use crate::types::AccountAddress;

/// Supports validation of signatures for known authors with individual voting powers. This struct
/// can be used for all signature verification operations including block and network signature
/// verification, respectively.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ValidatorVerifier {
    /// A vector of each validator's on-chain account address to its pubkeys and voting power.
    validator_infos: Vec<ValidatorConsensusInfo>,
    /// The minimum voting power required to achieve a quorum
    #[serde(skip)]
    quorum_voting_power: u128,
    /// Total voting power of all validators (cached from address_to_validator_info)
    #[serde(skip)]
    total_voting_power: u128,
    /// In-memory index of account address to its index in the vector, does not go through serde.
    #[serde(skip)]
    address_to_validator_index: HashMap<AccountAddress, usize>,
}

impl ValidatorVerifier {
    /// Private constructor to calculate the in-memory index
    fn build_index(
        validator_infos: Vec<ValidatorConsensusInfo>,
        quorum_voting_power: u128,
        total_voting_power: u128,
    ) -> Self {
        let address_to_validator_index = validator_infos
            .iter()
            .enumerate()
            .map(|(index, info)| (info.address, index))
            .collect();
        Self {
            validator_infos,
            quorum_voting_power,
            total_voting_power,
            address_to_validator_index,
        }
    }

    /// Initialize with a map of account address to validator info and set quorum size to
    /// default (`2f + 1`) or zero if `address_to_validator_info` is empty.
    pub fn new(validator_infos: Vec<ValidatorConsensusInfo>) -> Self {
        let total_voting_power = sum_voting_power(&validator_infos);
        let quorum_voting_power = if validator_infos.is_empty() {
            0
        } else {
            total_voting_power * 2 / 3 + 1
        };
        Self::build_index(validator_infos, quorum_voting_power, total_voting_power)
    }
}

/// Returns sum of voting power from Map of validator account addresses, validator consensus info
fn sum_voting_power(address_to_validator_info: &[ValidatorConsensusInfo]) -> u128 {
    address_to_validator_info.iter().fold(0, |sum, x| {
        sum.checked_add(x.voting_power as u128)
            .expect("sum of all voting power is greater than u64::max")
    })
}

/// Helper struct to manage validator information for validation
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ValidatorConsensusInfo {
    pub address: AccountAddress,
    pub public_key: PublicKey,
    pub voting_power: u64,
}

impl<'de> Deserialize<'de> for ValidatorVerifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename = "ValidatorVerifier")]
        struct RawValidatorVerifier {
            validator_infos: Vec<ValidatorConsensusInfo>,
        }

        let RawValidatorVerifier { validator_infos } =
            RawValidatorVerifier::deserialize(deserializer)?;

        Ok(ValidatorVerifier::new(validator_infos))
    }
}

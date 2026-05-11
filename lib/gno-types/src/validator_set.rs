use serde::{Deserialize, Serialize};
use unionlabs::primitives::{Bech32, H160};

use crate::Validator;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[serde(deny_unknown_fields)]
pub struct ValidatorSet {
    pub validators: Vec<Validator>,
    pub proposer: Validator,
}

pub const MAX_TOTAL_VOTING_POWER: i64 = i64::MAX / 8;

impl ValidatorSet {
    /// GetByAddress returns an index of the validator with address and validator
    /// itself if found. Otherwise, -1 and nil are returned.
    ///
    /// Source: <https://github.com/gnolang/gno/blob/db1e3ec26c613fd5d119c4466b32c2c0806b2e5c/tm2/pkg/bft/types/validator_set.go#L233>
    pub fn get_by_address(&self, validator_address: &Bech32<H160>) -> Option<(usize, &Validator)> {
        self.validators
            .iter()
            .enumerate()
            .find(|(_, val)| &val.address == validator_address)
    }

    // GetByIndex returns the validator's address and validator itself by index.
    // It returns nil values if index is less than 0 or greater or equal to
    // len(ValidatorSet.Validators).
    pub fn get_by_index(&self, index: usize) -> Option<(&Bech32<H160>, &Validator)> {
        self.validators.get(index).map(|val| (&val.address, val))
    }

    pub fn total_voting_power(&self) -> i64 {
        let sum = self
            .validators
            .iter()
            .map(|v| v.voting_power.inner())
            .fold(0_i64, |acc, curr| acc.saturating_add(curr));

        if sum > MAX_TOTAL_VOTING_POWER {
            // TODO: Don't panic here, even though the original go code does, but instead return an error
            panic!(
                "Total voting power should be guarded to not exceed {}; got: {}",
                MAX_TOTAL_VOTING_POWER, sum
            )
        }

        sum
        // for _, val := range vals.Validators {
        // 	// mind overflow
        // 	sum = safeAddClip(sum, val.VotingPower)
        // 	if sum > MaxTotalVotingPower {
        // 		panic(fmt.Sprintf(
        // 			"Total voting power should be guarded to not exceed %v; got: %v",
        // 			MaxTotalVotingPower,
        // 			sum))
        // 	}
        // }
    }
}

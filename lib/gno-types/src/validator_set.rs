use serde::{Deserialize, Serialize};
use unionlabs::primitives::{Bech32, H160};

use crate::Validator;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidatorSet {
    pub validators: Vec<Validator>,
    pub proposer: Validator,
}

impl ValidatorSet {
    /// GetByAddress returns an index of the validator with address and validator
    /// itself if found. Otherwise, -1 and nil are returned.
    ///
    /// Source: <https://github.com/gnolang/gno/blob/db1e3ec26c613fd5d119c4466b32c2c0806b2e5c/tm2/pkg/bft/types/validator_set.go#L233>
    pub fn get_by_address(&self, validator_address: Bech32<H160>) -> Option<(usize, &Validator)> {
        self.validators
            .iter()
            .enumerate()
            .find(|(_, val)| val.address == validator_address)
    }
}

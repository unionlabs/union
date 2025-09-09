use serde::{Deserialize, Serialize};

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/utils/types/Time.sol#L61>
///
/// ```solidity
/// type Delay is uint112;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct Delay {
    pub effect_date: u64,
    pub value_before: u32,
    pub value_after: u32,
}

impl Delay {
    pub fn get_full_at(&self, timepoint: u64) -> Self {
        let Delay {
            effect_date,
            value_before,
            value_after,
        } = self;
    }
}

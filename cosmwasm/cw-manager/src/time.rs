use std::cmp;

use cosmwasm_std::Env;
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

pub struct UnpackedDelay {
    pub effect_date: u64,
    pub value_before: u32,
    pub value_after: u32,
}

impl Delay {
    /// Wrap a duration into a Delay to add the one-step "update in the future" feature
    ///
    /// ```solidity
    /// function toDelay(uint32 duration) internal pure returns (Delay)
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/utils/types/Time.sol#L66>
    pub fn new(delay: u32) -> Self {
        Self {
            effect_date: 0,
            value_before: delay,
            value_after: 0,
        }
    }

    /// Get the value at a given timepoint plus the pending value and effect timepoint if there is a scheduled
    /// change after this timepoint. If the effect timepoint is 0, then the pending value should not be considered.
    ///
    /// ```solidity
    /// function _getFullAt(
    ///     Delay self,
    ///     uint48 timepoint
    /// ) private pure returns (uint32 valueBefore, uint32 valueAfter, uint48 effect)
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/utils/types/Time.sol#L74-L77>
    pub fn get_full_at(&self, timepoint: u64) -> UnpackedDelay {
        let &Delay {
            effect_date,
            value_before,
            value_after,
        } = self;

        if effect_date <= timepoint {
            UnpackedDelay {
                effect_date: 0,
                value_before: value_after,
                value_after: 0,
            }
        } else {
            UnpackedDelay {
                effect_date,
                value_before,
                value_after,
            }
        }
    }

    /// Get the current value plus the pending value and effect timepoint if there is a scheduled change. If the
    /// effect timepoint is 0, then the pending value should not be considered.
    ///
    /// ```solidity
    /// function getFull(Delay self) internal view returns (uint32 valueBefore, uint32 valueAfter, uint48 effect)
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/utils/types/Time.sol#L86>
    pub fn get_full(&self, env: &Env) -> UnpackedDelay {
        self.get_full_at(env.block.time.seconds())
    }

    /// Get the current value.
    ///
    /// ```solidity
    /// function get(Delay self) internal view returns (uint32)
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/utils/types/Time.sol#L93>
    pub fn get(&self, env: &Env) -> u32 {
        self.get_full_at(env.block.time.seconds()).value_before
    }

    /// Update a Delay object so that it takes a new duration after a timepoint that is automatically computed to enforce the old delay at the moment of the update. Returns the updated Delay object and the timestamp when the new delay becomes effective.
    ///
    /// ```solidity
    /// function withUpdate(
    ///     Delay self,
    ///     uint32 newValue,
    ///     uint32 minSetback
    /// ) internal view returns (Delay updatedDelay, uint48 effect)
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/utils/types/Time.sol#L103-L107>
    pub fn with_update(&self, env: &Env, new_value: u32, min_setback: u32) -> (Delay, u64) {
        let value = self.get(env);
        let setback = cmp::max(
            min_setback,
            if value > new_value {
                value - new_value
            } else {
                0
            },
        );
        let effect = env.block.time.seconds() + setback as u64;
        (
            Delay {
                effect_date: effect,
                value_before: value,
                value_after: new_value,
            },
            effect,
        )
    }
}

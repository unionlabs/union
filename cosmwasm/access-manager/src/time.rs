// TODO: Add a trait for "timestamp provider" and use that instead of threading a timestamp value
// everywhere

use std::cmp;

use serde::{Deserialize, Serialize};

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/utils/types/Time.sol#L61>
///
/// ```solidity
/// type Delay is uint112;
/// ```
#[derive(
    Debug, Clone, PartialEq, Default, Serialize, Deserialize, bincode::Encode, bincode::Decode,
)]
pub struct Delay {
    effect_date: u64,
    value_before: u32,
    value_after: u32,
}

#[derive(Debug, PartialEq)]
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
    #[must_use]
    pub fn new(delay: u32) -> Self {
        Self {
            effect_date: 0,
            value_before: 0,
            value_after: delay,
        }
    }

    /// Get the value at a given timepoint plus the pending value and effect timepoint if there is a
    /// scheduled change after this timepoint. If the effect timepoint is 0, then the pending value
    /// should not be considered.
    ///
    /// ```solidity
    /// function _getFullAt(
    ///     Delay self,
    ///     uint48 timepoint
    /// ) private pure returns (uint32 valueBefore, uint32 valueAfter, uint48 effect)
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/utils/types/Time.sol#L74-L77>
    fn _get_full_at(&self, timepoint: u64) -> UnpackedDelay {
        let &Delay {
            effect_date,
            value_before,
            value_after,
        } = self;

        if effect_date <= timepoint {
            UnpackedDelay {
                value_before: value_after,
                value_after: 0,
                effect_date: 0,
            }
        } else {
            UnpackedDelay {
                effect_date,
                value_before,
                value_after,
            }
        }
    }

    /// Get the current value plus the pending value and effect timepoint if there is a scheduled
    /// change. If the effect timepoint is 0, then the pending value should not be considered.
    ///
    /// ```solidity
    /// function getFull(Delay self) internal view returns (uint32 valueBefore, uint32 valueAfter, uint48 effect)
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/utils/types/Time.sol#L86>
    #[must_use]
    pub fn get_full(&self, timestamp: u64) -> UnpackedDelay {
        self._get_full_at(timestamp)
    }

    /// Get the current value.
    ///
    /// ```solidity
    /// function get(Delay self) internal view returns (uint32)
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/utils/types/Time.sol#L93>
    #[must_use]
    pub fn get(&self, timestamp: u64) -> u32 {
        self._get_full_at(timestamp).value_before
    }

    /// Update a Delay object so that it takes a new duration after a timepoint that is
    /// automatically computed to enforce the old delay at the moment of the update. Returns the
    /// updated Delay object and the timestamp when the new delay becomes effective.
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
    #[must_use]
    pub fn with_update(&self, timestamp: u64, new_value: u32, min_setback: u32) -> (Delay, u64) {
        let value = self.get(timestamp);
        let setback = cmp::max(min_setback, value.saturating_sub(new_value));
        let effect = timestamp + u64::from(setback);
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

/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/test/utils/types/Time.test.js>
#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use hex_literal::hex;
    use itertools::Itertools;
    use unionlabs_encoding::{Bincode, DecodeAs, EncodeAs};

    use super::*;

    const MAX_UINT48: u64 = 2_u64.pow(48);
    const SOME_VALUES: [u32; 7] = [0, 1, 2, 15, 16, 17, 42];

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn effect_samples_for_timepoint(timepoint: u64) -> impl Iterator<Item = u64> {
        [0, timepoint]
            .into_iter()
            .chain(
                [true, false]
                    .into_iter()
                    .cartesian_product([1, 2, 17, 42])
                    .map(move |(sign, shift)| {
                        if sign {
                            timepoint.saturating_sub(shift)
                        } else {
                            timepoint + shift
                        }
                    })
                    .filter(|&effect| effect > 0_u64 && effect <= MAX_UINT48),
            )
            .chain([MAX_UINT48])
    }

    #[test]
    fn to_delay() {
        let timepoint = now();

        for delay in SOME_VALUES.into_iter().chain([u32::MAX]) {
            assert_eq!(
                Delay::new(delay),
                Delay {
                    effect_date: 0,
                    value_before: 0,
                    value_after: delay,
                }
            );
            assert_eq!(Delay::new(delay).get(timepoint), delay);
        }
    }

    #[test]
    fn get_and_get_full() {
        let timepoint = now();

        let value_before = 24_194;
        let value_after = 4_214_143;

        for effect in effect_samples_for_timepoint(timepoint) {
            let is_past = effect <= timepoint;

            let delay = Delay {
                effect_date: effect,
                value_before,
                value_after,
            };

            if is_past {
                assert_eq!(delay.get(timepoint), value_after);
                assert_eq!(
                    delay.get_full(timepoint),
                    UnpackedDelay {
                        value_before: value_after,
                        value_after: 0,
                        effect_date: 0,
                    }
                );
            } else {
                assert_eq!(delay.get(timepoint), value_before);
                assert_eq!(
                    delay.get_full(timepoint),
                    UnpackedDelay {
                        value_before,
                        value_after,
                        effect_date: effect,
                    }
                );
            }
        }
    }

    #[test]
    fn with_update() {
        let timepoint = now();

        let value_before = 24_194_u32;
        let value_after = 4_214_143_u32;
        let new_value_after = 94_716_u32;

        for effect in effect_samples_for_timepoint(timepoint) {
            for min_setback in SOME_VALUES.into_iter().chain([u32::MAX]) {
                let is_past = effect <= timepoint;
                let expected_value_before = if is_past { value_after } else { value_before };
                let expected_setback = [
                    min_setback,
                    expected_value_before.saturating_sub(new_value_after),
                    0,
                ]
                .into_iter()
                .max()
                .unwrap();

                let delay = Delay {
                    effect_date: effect,
                    value_before,
                    value_after,
                };

                assert_eq!(
                    delay.with_update(timepoint, new_value_after, min_setback),
                    (
                        Delay {
                            effect_date: timepoint + u64::from(expected_setback),
                            value_before: expected_value_before,
                            value_after: new_value_after,
                        },
                        timepoint + u64::from(expected_setback)
                    ),
                );
            }
        }
    }

    #[test]
    fn bincode() {
        let bz = hex!("0100000000000000" "02000000" "03000000");

        let decoded = Delay::decode_as::<Bincode>(&bz).unwrap();
        assert_eq!(
            decoded,
            Delay {
                effect_date: 1,
                value_before: 2,
                value_after: 3
            },
        );

        assert_eq!(decoded.encode_as::<Bincode>(), bz);
    }
}

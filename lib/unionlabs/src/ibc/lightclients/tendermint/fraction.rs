use core::num::NonZeroU64;

use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct Fraction {
    pub numerator: u64,
    pub denominator: NonZeroU64,
}

impl From<Fraction> for protos::ibc::lightclients::tendermint::v1::Fraction {
    fn from(value: Fraction) -> Self {
        Self {
            numerator: value.numerator,
            denominator: value.denominator.get(),
        }
    }
}

#[derive(Debug)]
pub enum TryFromFractionError {
    ZeroDenominator,
}

impl TryFrom<protos::ibc::lightclients::tendermint::v1::Fraction> for Fraction {
    type Error = TryFromFractionError;

    fn try_from(
        value: protos::ibc::lightclients::tendermint::v1::Fraction,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            numerator: value.numerator,
            denominator: value
                .denominator
                .try_into()
                .map_err(|_| TryFromFractionError::ZeroDenominator)?,
        })
    }
}

// TODO(benluelo): This will be replaced with tendermint once the solidity contract types are regenerated
#[cfg(feature = "ethabi")]
impl From<Fraction> for contracts::glue::IbcLightclientsTendermintV1FractionData {
    fn from(value: Fraction) -> Self {
        Self {
            numerator: value.numerator,
            denominator: value.denominator.get(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<contracts::glue::IbcLightclientsTendermintV1FractionData> for Fraction {
    fn from(value: contracts::glue::IbcLightclientsTendermintV1FractionData) -> Self {
        Self {
            numerator: value.numerator,
            denominator: NonZeroU64::new(value.denominator).expect("non-zero denominator"),
        }
    }
}

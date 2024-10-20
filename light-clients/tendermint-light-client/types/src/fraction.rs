use core::num::NonZeroU64;

use macros::model;

#[model(proto(raw(protos::ibc::lightclients::tendermint::v1::Fraction), into, from))]
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

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromFractionError {
    #[error("zero denominator")]
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
            // TODO: Don't panic here lol
            denominator: NonZeroU64::new(value.denominator).expect("non-zero denominator"),
        }
    }
}

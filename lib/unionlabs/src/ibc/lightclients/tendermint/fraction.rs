use core::num::NonZeroU64;

use macros::model;

#[model(proto(raw(protos::ibc::lightclients::tendermint::v1::Fraction), into, from))]
pub struct Fraction {
    pub numerator: u64,
    pub denominator: NonZeroU64,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::ibc::lightclients::tendermint::fraction::Fraction;

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
}

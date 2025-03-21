use core::num::NonZeroU64;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Fraction {
    pub numerator: u64,
    pub denominator: NonZeroU64,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::impl_proto_via_try_from_into;

    use crate::Fraction;

    impl_proto_via_try_from_into!(Fraction => protos::ibc::lightclients::tendermint::v1::Fraction);

    impl From<Fraction> for protos::ibc::lightclients::tendermint::v1::Fraction {
        fn from(value: Fraction) -> Self {
            Self {
                numerator: value.numerator,
                denominator: value.denominator.get(),
            }
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum Error {
        #[error("zero denominator")]
        ZeroDenominator,
    }

    impl TryFrom<protos::ibc::lightclients::tendermint::v1::Fraction> for Fraction {
        type Error = Error;

        fn try_from(
            value: protos::ibc::lightclients::tendermint::v1::Fraction,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                numerator: value.numerator,
                denominator: value
                    .denominator
                    .try_into()
                    .map_err(|_| Error::ZeroDenominator)?,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use unionlabs::{
        encoding::{Bincode, Json, Proto},
        test_utils::assert_codec_iso,
    };

    use super::*;

    fn mk_fraction() -> Fraction {
        Fraction {
            numerator: 1,
            denominator: 3.try_into().unwrap(),
        }
    }

    #[test]
    fn bincode_iso() {
        assert_codec_iso::<_, Bincode>(&mk_fraction());
    }

    #[test]
    fn json_iso() {
        assert_codec_iso::<_, Json>(&mk_fraction());
    }

    #[test]
    fn proto_iso() {
        assert_codec_iso::<_, Proto>(&mk_fraction());
    }
}

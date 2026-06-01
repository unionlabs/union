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

#[cfg(test)]
mod tests {
    use unionlabs::{
        encoding::{Bincode, Json},
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
}

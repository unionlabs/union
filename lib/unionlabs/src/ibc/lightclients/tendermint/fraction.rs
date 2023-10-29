use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fraction {
    pub numerator: u64,
    pub denominator: u64,
}

impl From<Fraction> for protos::ibc::lightclients::tendermint::v1::Fraction {
    fn from(value: Fraction) -> Self {
        Self {
            numerator: value.numerator,
            denominator: value.denominator,
        }
    }
}

impl From<protos::ibc::lightclients::tendermint::v1::Fraction> for Fraction {
    fn from(value: protos::ibc::lightclients::tendermint::v1::Fraction) -> Self {
        Self {
            numerator: value.numerator,
            denominator: value.denominator,
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<Fraction> for contracts::glue::IbcLightclientsTendermintV1FractionData {
    fn from(value: Fraction) -> Self {
        Self {
            numerator: value.numerator,
            denominator: value.denominator,
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<contracts::glue::IbcLightclientsTendermintV1FractionData> for Fraction {
    fn from(value: contracts::glue::IbcLightclientsTendermintV1FractionData) -> Self {
        Self {
            numerator: value.numerator,
            denominator: value.denominator,
        }
    }
}

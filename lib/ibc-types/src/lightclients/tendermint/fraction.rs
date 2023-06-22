use contracts::glue::UnionIbcLightclientsCometblsV1FractionData;

#[derive(Debug, Clone)]
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

// TODO(benluelo): This will be replaced with tendermint once the solidity contract types are regenerated
impl From<Fraction> for UnionIbcLightclientsCometblsV1FractionData {
    fn from(value: Fraction) -> Self {
        Self {
            numerator: value.numerator,
            denominator: value.denominator,
        }
    }
}

impl From<UnionIbcLightclientsCometblsV1FractionData> for Fraction {
    fn from(value: UnionIbcLightclientsCometblsV1FractionData) -> Self {
        Self {
            numerator: value.numerator,
            denominator: value.denominator,
        }
    }
}

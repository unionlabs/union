use macros::model;

use crate::{bls::BlsSignature, errors::InvalidLength};

#[model(proto(
    raw(protos::union::ibc::lightclients::movement::v1::AggregateSignature),
    into,
    from
))]
pub struct AggregateSignature {
    validator_bitmask: Vec<u8>,
    sig: BlsSignature,
}

impl From<AggregateSignature>
    for protos::union::ibc::lightclients::movement::v1::AggregateSignature
{
    fn from(value: AggregateSignature) -> Self {
        Self {
            validator_bitmask: value.validator_bitmask,
            sig: value.sig.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromAggregateSignatureError {
    #[error("invalid sig")]
    Sig(#[from] InvalidLength),
}

impl TryFrom<protos::union::ibc::lightclients::movement::v1::AggregateSignature>
    for AggregateSignature
{
    type Error = TryFromAggregateSignatureError;

    fn try_from(
        value: protos::union::ibc::lightclients::movement::v1::AggregateSignature,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            validator_bitmask: value.validator_bitmask,
            sig: value.sig.try_into()?,
        })
    }
}

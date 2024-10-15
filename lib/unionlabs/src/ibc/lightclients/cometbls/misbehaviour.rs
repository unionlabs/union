use macros::model;

use crate::{
    errors::{required, MissingField},
    ibc::lightclients::cometbls::header::{Header, TryFromHeaderError},
};

#[model(proto(
    raw(protos::union::ibc::lightclients::cometbls::v1::Misbehaviour),
    into,
    from
))]
pub struct Misbehaviour {
    pub header_a: Header,
    pub header_b: Header,
}

impl From<Misbehaviour> for protos::union::ibc::lightclients::cometbls::v1::Misbehaviour {
    fn from(value: Misbehaviour) -> Self {
        Self {
            header_a: Some(value.header_a.into()),
            header_b: Some(value.header_b.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromMisbehaviourError {
    #[error(transparent)]
    MissingField(MissingField),
    #[error("invalid signed header")]
    Header(#[from] TryFromHeaderError),
}

impl TryFrom<protos::union::ibc::lightclients::cometbls::v1::Misbehaviour> for Misbehaviour {
    type Error = TryFromMisbehaviourError;

    fn try_from(
        value: protos::union::ibc::lightclients::cometbls::v1::Misbehaviour,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            header_a: required!(value.header_a)?.try_into()?,
            header_b: required!(value.header_b)?.try_into()?,
        })
    }
}

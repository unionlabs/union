use super::header::Header;
use crate::{
    errors::{required, MissingField},
    Proto, TryFromProtoErrorOf, TypeUrl,
};

#[derive(Debug)]
pub struct Misbehaviour {
    pub header_1: Header,
    pub header_2: Header,
}

impl From<Misbehaviour> for protos::ibc::lightclients::tendermint::v1::Misbehaviour {
    fn from(value: Misbehaviour) -> Self {
        #[allow(deprecated)]
        Self {
            header_1: Some(value.header_1.into()),
            header_2: Some(value.header_2.into()),
            client_id: Default::default(),
        }
    }
}

#[derive(Debug)]
pub enum TryFromMisbehaviourError {
    MissingField(MissingField),
    Header1(TryFromProtoErrorOf<Header>),
    Header2(TryFromProtoErrorOf<Header>),
}

impl TryFrom<protos::ibc::lightclients::tendermint::v1::Misbehaviour> for Misbehaviour {
    type Error = TryFromMisbehaviourError;

    fn try_from(
        value: protos::ibc::lightclients::tendermint::v1::Misbehaviour,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            header_1: required!(value.header_1)?
                .try_into()
                .map_err(TryFromMisbehaviourError::Header1)?,
            header_2: required!(value.header_2)?
                .try_into()
                .map_err(TryFromMisbehaviourError::Header2)?,
        })
    }
}

impl Proto for Misbehaviour {
    type Proto = protos::ibc::lightclients::tendermint::v1::Misbehaviour;
}

impl TypeUrl for protos::ibc::lightclients::tendermint::v1::Misbehaviour {
    const TYPE_URL: &'static str = "/ibc.lightclients.tendermint.v1.Misbehaviour";
}

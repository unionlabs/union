use macros::model;

use crate::{
    aptos::state_proof::{StateProof, TryFromStateProofError},
    errors::{required, MissingField},
    ibc::core::client::height::Height,
};

#[model(proto(
    raw(protos::union::ibc::lightclients::movement::v1::Header),
    into,
    from
))]
pub struct Header {
    pub l1_height: Height,
    pub trusted_height: Height,
    pub state_proof: StateProof,
}

impl From<Header> for protos::union::ibc::lightclients::movement::v1::Header {
    fn from(value: Header) -> Self {
        Self {
            l1_height: Some(value.l1_height.into()),
            trusted_height: Some(value.trusted_height.into()),
            state_proof: Some(value.state_proof.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromHeaderError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid state proof")]
    StateProof(#[from] TryFromStateProofError),
}

impl TryFrom<protos::union::ibc::lightclients::movement::v1::Header> for Header {
    type Error = TryFromHeaderError;

    fn try_from(
        value: protos::union::ibc::lightclients::movement::v1::Header,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            l1_height: required!(value.l1_height)?.into(),
            trusted_height: required!(value.trusted_height)?.into(),
            state_proof: required!(value.state_proof)?.try_into()?,
        })
    }
}

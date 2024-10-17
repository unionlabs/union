use macros::model;

use crate::{
    cometbft::types::{
        signed_header::{SignedHeader, TryFromSignedHeaderError},
        validator_set::{TryFromValidatorSetError, ValidatorSet},
    },
    errors::{required, MissingField},
    ibc::core::client::height::Height,
};

#[model(proto(raw(protos::ibc::lightclients::tendermint::v1::Header), into, from))]
pub struct Header {
    pub signed_header: SignedHeader,
    pub validator_set: ValidatorSet,
    pub trusted_height: Height,
    pub trusted_validators: ValidatorSet,
}

impl From<Header> for protos::ibc::lightclients::tendermint::v1::Header {
    fn from(value: Header) -> Self {
        Self {
            signed_header: Some(value.signed_header.into()),
            validator_set: Some(value.validator_set.into()),
            trusted_height: Some(value.trusted_height.into()),
            trusted_validators: Some(value.trusted_validators.into()),
        }
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromHeaderError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid signed header")]
    SignedHeader(#[source] TryFromSignedHeaderError),
    #[error("invalid validator set")]
    ValidatorSet(#[source] TryFromValidatorSetError),
    #[error("invalid trusted validators")]
    TrustedValidators(#[source] TryFromValidatorSetError),
}

impl TryFrom<protos::ibc::lightclients::tendermint::v1::Header> for Header {
    type Error = TryFromHeaderError;

    fn try_from(
        value: protos::ibc::lightclients::tendermint::v1::Header,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            signed_header: required!(value.signed_header)?
                .try_into()
                .map_err(TryFromHeaderError::SignedHeader)?,
            validator_set: required!(value.validator_set)?
                .try_into()
                .map_err(TryFromHeaderError::ValidatorSet)?,
            trusted_height: required!(value.trusted_height)?.into(),
            trusted_validators: required!(value.trusted_validators)?
                .try_into()
                .map_err(TryFromHeaderError::TrustedValidators)?,
        })
    }
}

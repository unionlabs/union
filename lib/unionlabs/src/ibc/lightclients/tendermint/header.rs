use serde::{Deserialize, Serialize};

use crate::{
    errors::{required, MissingField},
    ibc::core::client::height::Height,
    tendermint::types::{signed_header::SignedHeader, validator_set::ValidatorSet},
    Proto, TryFromProtoErrorOf, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
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

#[derive(Debug)]
pub enum TryFromHeaderError {
    MissingField(MissingField),
    SignedHeader(TryFromProtoErrorOf<SignedHeader>),
    ValidatorSet(TryFromProtoErrorOf<ValidatorSet>),
    TrustedValidators(TryFromProtoErrorOf<ValidatorSet>),
}

impl TryFrom<protos::ibc::lightclients::tendermint::v1::Header> for Header {
    type Error = TryFromHeaderError;

    fn try_from(
        value: protos::ibc::lightclients::tendermint::v1::Header,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            signed_header: value
                .signed_header
                .ok_or(TryFromHeaderError::MissingField(MissingField(
                    "signed header",
                )))?
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

impl Proto for Header {
    type Proto = protos::ibc::lightclients::tendermint::v1::Header;
}

impl TypeUrl for protos::ibc::lightclients::tendermint::v1::Header {
    const TYPE_URL: &'static str = "/ibc.lightclients.tendermint.v1.Header";
}

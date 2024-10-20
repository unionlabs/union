use serde::{Deserialize, Serialize};
use unionlabs::{
    ibc::core::client::height::Height,
    tendermint::types::{signed_header::SignedHeader, validator_set::ValidatorSet},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Header {
    pub signed_header: SignedHeader,
    pub validator_set: ValidatorSet,
    pub trusted_height: Height,
    pub trusted_validators: ValidatorSet,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{
        errors::MissingField,
        impl_proto_via_try_from_into, required,
        tendermint::types::{
            signed_header::TryFromSignedHeaderError, validator_set::TryFromValidatorSetError,
        },
    };

    use crate::Header;

    impl_proto_via_try_from_into!(Header => protos::ibc::lightclients::tendermint::v1::Header);

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
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid signed header")]
        SignedHeader(#[from] TryFromSignedHeaderError),
        #[error("invalid validator set")]
        ValidatorSet(#[source] TryFromValidatorSetError),
        #[error("invalid trusted validators")]
        TrustedValidators(#[source] TryFromValidatorSetError),
    }

    impl TryFrom<protos::ibc::lightclients::tendermint::v1::Header> for Header {
        type Error = Error;

        fn try_from(
            value: protos::ibc::lightclients::tendermint::v1::Header,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                signed_header: required!(value.signed_header)?.try_into()?,
                validator_set: required!(value.validator_set)?
                    .try_into()
                    .map_err(Error::ValidatorSet)?,
                trusted_height: required!(value.trusted_height)?.into(),
                trusted_validators: required!(value.trusted_validators)?
                    .try_into()
                    .map_err(Error::TrustedValidators)?,
            })
        }
    }
}

use macros::model;

use crate::{
    cosmos::tx::{
        fee::{Fee, TryFromFeeError},
        signer_info::{SignerInfo, TryFromSignerInfoError},
    },
    errors::{required, MissingField},
};

#[model(proto(raw(protos::cosmos::tx::v1beta1::AuthInfo), into, from))]
pub struct AuthInfo {
    pub signer_infos: Vec<SignerInfo>,
    pub fee: Fee,
}

impl From<AuthInfo> for protos::cosmos::tx::v1beta1::AuthInfo {
    fn from(value: AuthInfo) -> Self {
        Self {
            signer_infos: value.signer_infos.into_iter().map(Into::into).collect(),
            fee: Some(value.fee.into()),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromAuthInfoError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid signer info")]
    SignerInfo(#[from] TryFromSignerInfoError),
    #[error("invalid fee")]
    Fee(#[from] TryFromFeeError),
}

impl TryFrom<protos::cosmos::tx::v1beta1::AuthInfo> for AuthInfo {
    type Error = TryFromAuthInfoError;

    fn try_from(value: protos::cosmos::tx::v1beta1::AuthInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            signer_infos: value
                .signer_infos
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            fee: required!(value.fee)?.try_into()?,
        })
    }
}

use serde::{Deserialize, Serialize};

use crate::cosmos::tx::{fee::Fee, signer_info::SignerInfo};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthInfo {
    pub signer_infos: Vec<SignerInfo>,
    pub fee: Fee,
}

#[cfg(feature = "proto")]
pub mod proto {
    use super::AuthInfo;
    use crate::{
        cosmos::tx::{fee, signer_info},
        errors::MissingField,
        impl_proto_via_try_from_into, required,
    };

    impl_proto_via_try_from_into!(AuthInfo => protos::cosmos::tx::v1beta1::AuthInfo);

    impl From<AuthInfo> for protos::cosmos::tx::v1beta1::AuthInfo {
        fn from(value: AuthInfo) -> Self {
            Self {
                signer_infos: value.signer_infos.into_iter().map(Into::into).collect(),
                fee: Some(value.fee.into()),
                ..Default::default()
            }
        }
    }

    impl TryFrom<protos::cosmos::tx::v1beta1::AuthInfo> for AuthInfo {
        type Error = Error;

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

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid signer info")]
        SignerInfo(#[from] signer_info::proto::Error),
        #[error("invalid fee")]
        Fee(#[from] fee::proto::Error),
    }
}

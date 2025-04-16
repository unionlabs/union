use serde::{Deserialize, Serialize};

use crate::cosmos::{crypto::AnyPubKey, tx::mode_info::ModeInfo};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignerInfo {
    pub public_key: Option<AnyPubKey>,
    pub mode_info: ModeInfo,
    pub sequence: u64,
}

#[cfg(feature = "proto")]
pub mod proto {
    use super::SignerInfo;
    use crate::{
        cosmos::{
            crypto::proto::TryFromAnyPubKeyError, tx::mode_info::proto::TryFromModeInfoError,
        },
        errors::MissingField,
        impl_proto_via_try_from_into, required,
    };

    impl_proto_via_try_from_into!(SignerInfo => protos::cosmos::tx::v1beta1::SignerInfo);

    impl From<SignerInfo> for protos::cosmos::tx::v1beta1::SignerInfo {
        fn from(value: SignerInfo) -> Self {
            Self {
                public_key: value.public_key.map(Into::into),
                mode_info: Some(value.mode_info.into()),
                sequence: value.sequence,
            }
        }
    }

    impl TryFrom<protos::cosmos::tx::v1beta1::SignerInfo> for SignerInfo {
        type Error = Error;

        fn try_from(value: protos::cosmos::tx::v1beta1::SignerInfo) -> Result<Self, Self::Error> {
            Ok(Self {
                public_key: value.public_key.map(TryInto::try_into).transpose()?,
                mode_info: required!(value.mode_info)?.try_into()?,
                sequence: value.sequence,
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid public key")]
        PublicKey(#[from] TryFromAnyPubKeyError),
        #[error("invalid mode info")]
        ModeInfo(#[from] TryFromModeInfoError),
    }
}

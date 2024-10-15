use macros::model;

use crate::cosmos::{crypto::AnyPubKey, tx::mode_info::ModeInfo};

#[model(proto(raw(protos::cosmos::tx::v1beta1::SignerInfo), into, from))]
pub struct SignerInfo {
    pub public_key: Option<AnyPubKey>,
    pub mode_info: ModeInfo,
    pub sequence: u64,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        cosmos::{
            crypto::proto::TryFromAnyPubKeyError,
            tx::{mode_info::proto::TryFromModeInfoError, signer_info::SignerInfo},
        },
        errors::{required, MissingField},
    };

    impl From<SignerInfo> for protos::cosmos::tx::v1beta1::SignerInfo {
        fn from(value: SignerInfo) -> Self {
            Self {
                public_key: value.public_key.map(Into::into),
                mode_info: Some(value.mode_info.into()),
                sequence: value.sequence,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum TryFromSignerInfoError {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid public key")]
        PublicKey(#[from] TryFromAnyPubKeyError),
        #[error("invalid mode info")]
        ModeInfo(#[from] TryFromModeInfoError),
    }

    impl TryFrom<protos::cosmos::tx::v1beta1::SignerInfo> for SignerInfo {
        type Error = TryFromSignerInfoError;

        fn try_from(value: protos::cosmos::tx::v1beta1::SignerInfo) -> Result<Self, Self::Error> {
            Ok(Self {
                public_key: value.public_key.map(TryInto::try_into).transpose()?,
                mode_info: required!(value.mode_info)?.try_into()?,
                sequence: value.sequence,
            })
        }
    }
}

use macros::model;

use crate::cosmos::tx::{auth_info::AuthInfo, tx_body::TxBody};

#[model(proto(raw(protos::cosmos::tx::v1beta1::Tx), into, from))]
pub struct Tx {
    pub body: TxBody,
    pub auth_info: AuthInfo,
    pub signatures: Vec<Vec<u8>>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        cosmos::tx::{auth_info::proto::TryFromAuthInfoError, tx::Tx},
        errors::{required, MissingField},
    };

    impl From<Tx> for protos::cosmos::tx::v1beta1::Tx {
        fn from(value: Tx) -> Self {
            Self {
                body: Some(value.body.into()),
                auth_info: Some(value.auth_info.into()),
                signatures: value.signatures,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum TryFromTxError {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid auth info")]
        AuthInfo(#[from] TryFromAuthInfoError),
    }

    impl TryFrom<protos::cosmos::tx::v1beta1::Tx> for Tx {
        type Error = TryFromTxError;

        fn try_from(value: protos::cosmos::tx::v1beta1::Tx) -> Result<Self, Self::Error> {
            Ok(Self {
                body: required!(value.body)?.into(),
                auth_info: required!(value.auth_info)?.try_into()?,
                signatures: value.signatures,
            })
        }
    }
}

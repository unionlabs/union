use serde::{Deserialize, Serialize};

use crate::{
    cosmos::tx::{auth_info::AuthInfo, tx_body::TxBody},
    google::protobuf::any::RawAny,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tx<M = RawAny> {
    pub body: TxBody<M>,
    pub auth_info: AuthInfo,
    pub signatures: Vec<Vec<u8>>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use super::Tx;
    use crate::{
        cosmos::tx::{auth_info, tx_body},
        errors::MissingField,
        google::protobuf::any::RawAny,
        impl_proto_via_try_from_into, required,
    };

    impl_proto_via_try_from_into!(
        {
            for(M) where
            encode(M: Into<RawAny>)
            decode(RawAny: TryInto<M, Error: core::error::Error>)
        }
        Tx<M> => protos::cosmos::tx::v1beta1::Tx
    );

    impl<M: Into<RawAny>> From<Tx<M>> for protos::cosmos::tx::v1beta1::Tx {
        fn from(value: Tx<M>) -> Self {
            Self {
                body: Some(value.body.into()),
                auth_info: Some(value.auth_info.into()),
                signatures: value.signatures,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid auth_info")]
        AuthInfo(#[from] auth_info::proto::Error),
        #[error("invalid tx_body")]
        TxBody(#[from] tx_body::proto::Error),
    }

    impl<M> TryFrom<protos::cosmos::tx::v1beta1::Tx> for Tx<M>
    where
        RawAny: TryInto<M, Error: core::error::Error>,
    {
        type Error = Error;

        fn try_from(value: protos::cosmos::tx::v1beta1::Tx) -> Result<Self, Self::Error> {
            Ok(Self {
                body: required!(value.body)?.try_into()?,
                auth_info: required!(value.auth_info)?.try_into()?,
                signatures: value.signatures,
            })
        }
    }
}

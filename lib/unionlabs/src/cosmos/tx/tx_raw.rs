use macros::model;

#[model(proto(raw(protos::cosmos::tx::v1beta1::TxRaw), into, from))]
pub struct TxRaw {
    /// `body_bytes` is a protobuf serialization of a [`TxBody`](crate::cosmos::tx::tx_body::TxBody) that matches the
    /// representation in [`SignDoc`](crate::cosmos::tx::sign_doc::SignDoc).
    pub body_bytes: Vec<u8>,
    /// `auth_info_bytes` is a protobuf serialization of an [`AuthInfo`](crate::cosmos::tx::auth_info::AuthInfo) that matches the
    /// representation in [`SignDoc`](crate::cosmos::tx::sign_doc::SignDoc).
    pub auth_info_bytes: Vec<u8>,
    /// signatures is a list of signatures that matches the length and order of
    /// [`AuthInfo`](crate::cosmos::tx::auth_info::AuthInfo)'s `signer_infos` to allow connecting signature meta information like
    /// public key and signing mode by position.
    pub signatures: Vec<Vec<u8>>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::cosmos::tx::tx_raw::TxRaw;

    impl From<TxRaw> for protos::cosmos::tx::v1beta1::TxRaw {
        fn from(value: TxRaw) -> Self {
            Self {
                body_bytes: value.body_bytes,
                auth_info_bytes: value.auth_info_bytes,
                signatures: value.signatures,
            }
        }
    }

    impl From<protos::cosmos::tx::v1beta1::TxRaw> for TxRaw {
        fn from(value: protos::cosmos::tx::v1beta1::TxRaw) -> Self {
            Self {
                body_bytes: value.body_bytes,
                auth_info_bytes: value.auth_info_bytes,
                signatures: value.signatures,
            }
        }
    }
}

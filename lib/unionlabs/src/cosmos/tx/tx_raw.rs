use macros::model;

#[model(proto(raw(protos::cosmos::tx::v1beta1::TxRaw), into, from))]
pub struct TxRaw {
    /// `body_bytes` is a protobuf serialization of a [`TxBody`] that matches the
    /// representation in [`SignDoc`].
    pub body_bytes: Vec<u8>,
    /// `auth_info_bytes` is a protobuf serialization of an [`AuthInfo`] that matches the
    /// representation in [`SignDoc`].
    pub auth_info_bytes: Vec<u8>,
    /// signatures is a list of signatures that matches the length and order of
    /// [`AuthInfo`]'s `signer_infos` to allow connecting signature meta information like
    /// public key and signing mode by position.
    pub signatures: Vec<Vec<u8>>,
}

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

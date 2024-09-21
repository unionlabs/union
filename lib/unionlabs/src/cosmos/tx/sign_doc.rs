use macros::model;

#[model(proto(raw(protos::cosmos::tx::v1beta1::SignDoc), into, from))]
pub struct SignDoc {
    /// `body_bytes` is protobuf serialization of a [`TxBody`](crate::cosmos::tx::tx_body::TxBody) that matches the
    /// representation in [`TxRaw`](crate::cosmos::tx::tx_raw::TxRaw).
    pub body_bytes: Vec<u8>,
    /// `auth_info_bytes` is a protobuf serialization of an [`AuthInfo`](crate::cosmos::tx::auth_info::AuthInfo) that matches the
    /// representation in [`TxRaw`](crate::cosmos::tx::tx_raw::TxRaw).
    pub auth_info_bytes: Vec<u8>,
    /// `chain_id` is the unique identifier of the chain this transaction targets.
    /// It prevents signed transactions from being used on another chain by an
    /// attacker
    pub chain_id: String,
    /// `account_number` is the account number of the account in state
    pub account_number: u64,
}

impl From<SignDoc> for protos::cosmos::tx::v1beta1::SignDoc {
    fn from(value: SignDoc) -> Self {
        Self {
            body_bytes: value.body_bytes,
            auth_info_bytes: value.auth_info_bytes,
            chain_id: value.chain_id,
            account_number: value.account_number,
        }
    }
}

impl From<protos::cosmos::tx::v1beta1::SignDoc> for SignDoc {
    fn from(value: protos::cosmos::tx::v1beta1::SignDoc) -> Self {
        Self {
            body_bytes: value.body_bytes,
            auth_info_bytes: value.auth_info_bytes,
            chain_id: value.chain_id,
            account_number: value.account_number,
        }
    }
}

use macros::model;

use crate::{
    cosmos::crypto::{AnyPubKey, TryFromAnyPubKeyError},
    errors::MissingField,
};

#[model(proto(raw(protos::cosmos::auth::v1beta1::BaseAccount), into, from))]
pub struct BaseAccount {
    // REVIEW: is this a bech32 address?
    pub address: String,
    // accounts which haven't sent any transactions yet won't have a pubkey
    // TODO: `sequence` will also be 0, find a way to validate this?
    pub pub_key: Option<AnyPubKey>,
    pub account_number: u64,
    pub sequence: u64,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromBaseAccountError {
    #[error(transparent)]
    MissingField(MissingField),
    #[error("unable to decode pub key")]
    PubKey(#[from] TryFromAnyPubKeyError),
}

impl TryFrom<protos::cosmos::auth::v1beta1::BaseAccount> for BaseAccount {
    type Error = TryFromBaseAccountError;

    fn try_from(value: protos::cosmos::auth::v1beta1::BaseAccount) -> Result<Self, Self::Error> {
        Ok(Self {
            address: value.address,
            pub_key: value.pub_key.map(TryInto::try_into).transpose()?,
            account_number: value.account_number,
            sequence: value.sequence,
        })
    }
}

impl From<BaseAccount> for protos::cosmos::auth::v1beta1::BaseAccount {
    fn from(value: BaseAccount) -> Self {
        Self {
            address: value.address,
            pub_key: value.pub_key.map(Into::into),
            account_number: value.account_number,
            sequence: value.sequence,
        }
    }
}

use serde::{Deserialize, Serialize};

use crate::{cosmos::crypto::AnyPubKey, errors::MissingField, Proto, TryFromProtoErrorOf, TypeUrl};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BaseAccount {
    // REVIEW: is this a bech32 address?
    pub address: String,
    // accounts which haven't sent any transactions yet won't have a pubkey
    // TODO: `sequence` will also be 0, find a way to validate this?
    pub pub_key: Option<AnyPubKey>,
    pub account_number: u64,
    pub sequence: u64,
}

impl Proto for BaseAccount {
    type Proto = protos::cosmos::auth::v1beta1::BaseAccount;
}

impl TypeUrl for protos::cosmos::auth::v1beta1::BaseAccount {
    const TYPE_URL: &'static str = "/cosmos.auth.v1beta1.BaseAccount";
}

#[derive(Debug)]
pub enum TryFromBaseAccountError {
    MissingField(MissingField),
    PubKey(TryFromProtoErrorOf<AnyPubKey>),
}

impl TryFrom<protos::cosmos::auth::v1beta1::BaseAccount> for BaseAccount {
    type Error = TryFromBaseAccountError;

    fn try_from(value: protos::cosmos::auth::v1beta1::BaseAccount) -> Result<Self, Self::Error> {
        dbg!(&value);
        Ok(Self {
            address: value.address,
            pub_key: value
                .pub_key
                .map(|pk| pk.try_into().map_err(TryFromBaseAccountError::PubKey))
                .transpose()?,
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

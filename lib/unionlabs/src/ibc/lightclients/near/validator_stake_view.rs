use core::str::FromStr;

use borsh::{BorshDeserialize, BorshSerialize};
use macros::model;
use near_account_id::AccountId;
use near_primitives_core::{account::id::ParseAccountError, types::Balance};

use crate::{
    errors::{required, MissingField},
    near::types::{PublicKey, TryFromPublicKeyError},
};

#[model(proto(
    raw(protos::union::ibc::lightclients::near::v1::ValidatorStakeView),
    into,
    from
))]
#[derive(BorshSerialize, BorshDeserialize, Eq)]
pub enum ValidatorStakeView {
    V1(ValidatorStakeViewV1),
}

#[derive(
    BorshSerialize,
    BorshDeserialize,
    Debug,
    Clone,
    Eq,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct ValidatorStakeViewV1 {
    pub account_id: AccountId,
    pub public_key: PublicKey,
    // TODO(aeryz): #[serde(with = "dec_format")]
    pub stake: Balance,
}

impl From<ValidatorStakeView> for protos::union::ibc::lightclients::near::v1::ValidatorStakeView {
    fn from(value: ValidatorStakeView) -> Self {
        let ValidatorStakeView::V1(value) = value;
        Self {
            account_id: value.account_id.to_string(),
            public_key: Some(value.public_key.into()),
            balance: value.stake.to_le_bytes().into(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromValidatorStakeView {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error(transparent)]
    AccountId(#[from] ParseAccountError),
    #[error(transparent)]
    PublicKey(#[from] TryFromPublicKeyError),
    #[error("invalid balance size {0:?}")]
    Balance(Vec<u8>),
}

impl TryFrom<protos::union::ibc::lightclients::near::v1::ValidatorStakeView>
    for ValidatorStakeView
{
    type Error = TryFromValidatorStakeView;

    fn try_from(
        value: protos::union::ibc::lightclients::near::v1::ValidatorStakeView,
    ) -> Result<Self, Self::Error> {
        Ok(Self::V1(ValidatorStakeViewV1 {
            account_id: AccountId::from_str(&value.account_id)
                .map_err(TryFromValidatorStakeView::AccountId)?,
            public_key: required!(value.public_key)?
                .try_into()
                .map_err(TryFromValidatorStakeView::PublicKey)?,
            stake: u128::from_le_bytes(
                value
                    .balance
                    .try_into()
                    .map_err(TryFromValidatorStakeView::Balance)?,
            ),
        }))
    }
}

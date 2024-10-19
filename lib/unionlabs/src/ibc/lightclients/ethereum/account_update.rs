use macros::model;

use crate::{
    errors::{required, MissingField},
    ibc::lightclients::ethereum::account_proof::{AccountProof, TryFromAccountProofError},
};

#[model(proto(
    raw(protos::union::ibc::lightclients::ethereum::v1::AccountUpdate),
    into,
    from
))]
pub struct AccountUpdate {
    pub account_proof: AccountProof,
}

impl From<AccountUpdate> for protos::union::ibc::lightclients::ethereum::v1::AccountUpdate {
    fn from(value: AccountUpdate) -> Self {
        Self {
            account_proof: Some(value.account_proof.into()),
        }
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromAccountUpdateError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid `account_proof`")]
    AccountProof(#[from] TryFromAccountProofError),
}

impl TryFrom<protos::union::ibc::lightclients::ethereum::v1::AccountUpdate> for AccountUpdate {
    type Error = TryFromAccountUpdateError;

    fn try_from(
        value: protos::union::ibc::lightclients::ethereum::v1::AccountUpdate,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            account_proof: required!(value.account_proof)?.try_into()?,
        })
    }
}

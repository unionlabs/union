use serde::{Deserialize, Serialize};

use crate::{
    errors::{required, MissingField},
    ibc::lightclients::ethereum::account_proof::{AccountProof, TryFromAccountProofError},
    Proto, TypeUrl,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

#[derive(Debug)]
pub enum TryFromAccountUpdateError {
    MissingField(MissingField),
    AccountProof(TryFromAccountProofError),
}

impl TryFrom<protos::union::ibc::lightclients::ethereum::v1::AccountUpdate> for AccountUpdate {
    type Error = TryFromAccountUpdateError;

    fn try_from(
        value: protos::union::ibc::lightclients::ethereum::v1::AccountUpdate,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            account_proof: required!(value.account_proof)?
                .try_into()
                .map_err(TryFromAccountUpdateError::AccountProof)?,
        })
    }
}

impl Proto for AccountUpdate {
    type Proto = protos::union::ibc::lightclients::ethereum::v1::AccountUpdate;
}

impl TypeUrl for protos::union::ibc::lightclients::ethereum::v1::AccountUpdate {
    const TYPE_URL: &'static str = "/union.ibc.lightclients.ethereum.v1.AccountUpdate";
}

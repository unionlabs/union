use macros::model;

use crate::ibc::lightclients::ethereum::account_proof::AccountProof;

#[model(proto(
    raw(protos::union::ibc::lightclients::ethereum::v1::AccountUpdate),
    into,
    from
))]
pub struct AccountUpdate {
    pub account_proof: AccountProof,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        errors::{required, MissingField},
        ibc::lightclients::ethereum::{
            account_proof::proto::TryFromAccountProofError, account_update::AccountUpdate,
        },
    };

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
        MissingField(MissingField),
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
}

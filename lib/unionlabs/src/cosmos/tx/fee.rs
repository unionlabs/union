use macros::model;

use crate::cosmos::base::coin::Coin;

#[model(proto(raw(protos::cosmos::tx::v1beta1::Fee), into, from))]
pub struct Fee {
    /// amount is the amount of coins to be paid as a fee
    pub amount: Vec<Coin>,
    /// `gas_limit` is the maximum gas that can be used in transaction processing
    /// before an out of gas error occurs
    pub gas_limit: u64,
    /// if unset, the first signer is responsible for paying the fees. If set, the specified account must pay the fees.
    /// the payer must be a tx signer (and thus have signed this field in [`AuthInfo`](crate::cosmos::tx::auth_info::AuthInfo)).
    /// setting this field does *not* change the ordering of required signers for the transaction.
    pub payer: String,
    /// if set, the fee payer (either the first signer or the value of the payer field) requests that a fee grant be used
    /// to pay fees instead of the fee payer's own balance. If an appropriate fee grant does not exist or the chain does
    /// not support fee grants, this will fail
    pub granter: String,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::cosmos::{base::coin::proto::TryFromCoinError, tx::fee::Fee};

    impl From<Fee> for protos::cosmos::tx::v1beta1::Fee {
        fn from(value: Fee) -> Self {
            Self {
                amount: value.amount.into_iter().map(Into::into).collect(),
                gas_limit: value.gas_limit,
                payer: value.payer,
                granter: value.granter,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum TryFromFeeError {
        #[error("invalid amount")]
        Amount(#[from] TryFromCoinError),
    }

    impl TryFrom<protos::cosmos::tx::v1beta1::Fee> for Fee {
        type Error = TryFromFeeError;

        fn try_from(value: protos::cosmos::tx::v1beta1::Fee) -> Result<Self, Self::Error> {
            Ok(Self {
                amount: value
                    .amount
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?,
                gas_limit: value.gas_limit,
                payer: value.payer,
                granter: value.granter,
            })
        }
    }
}

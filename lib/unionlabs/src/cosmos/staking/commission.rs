use macros::model;

use crate::{
    cosmos::staking::commission_rates::CommissionRates, google::protobuf::timestamp::Timestamp,
};

#[model(proto(raw(protos::cosmos::staking::v1beta1::Commission), into, from))]
pub struct Commission {
    /// `commission_rates` defines the initial commission rates to be used for creating a validator.
    pub commission_rates: CommissionRates,
    /// `update_time` is the last time the commission rate was changed.
    pub update_time: Timestamp,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        cosmos::staking::commission::Commission,
        errors::{required, MissingField},
        google::protobuf::timestamp::proto::TryFromTimestampError,
    };

    #[derive(Debug)]
    pub enum TryFromCommissionError {
        MissingField(MissingField),
        Timestamp(TryFromTimestampError),
    }

    impl TryFrom<protos::cosmos::staking::v1beta1::Commission> for Commission {
        type Error = TryFromCommissionError;

        fn try_from(
            value: protos::cosmos::staking::v1beta1::Commission,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                commission_rates: required!(value.commission_rates)?.into(),
                update_time: required!(value.update_time)?
                    .try_into()
                    .map_err(TryFromCommissionError::Timestamp)?,
            })
        }
    }

    impl From<Commission> for protos::cosmos::staking::v1beta1::Commission {
        fn from(value: Commission) -> Self {
            Self {
                commission_rates: Some(value.commission_rates.into()),
                update_time: Some(value.update_time.into()),
            }
        }
    }
}

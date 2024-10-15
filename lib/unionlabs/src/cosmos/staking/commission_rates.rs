use macros::model;

#[model(proto(raw(protos::cosmos::staking::v1beta1::CommissionRates), into, from))]
pub struct CommissionRates {
    /// rate is the commission rate charged to delegators, as a fraction.
    pub rate: String,
    /// `max_rate` defines the maximum commission rate which validator can ever charge, as a fraction.
    pub max_rate: String,
    /// `max_change_rate` defines the maximum daily increase of the validator commission, as a fraction.
    pub max_change_rate: String,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::cosmos::staking::commission_rates::CommissionRates;

    impl From<protos::cosmos::staking::v1beta1::CommissionRates> for CommissionRates {
        fn from(value: protos::cosmos::staking::v1beta1::CommissionRates) -> Self {
            Self {
                rate: value.rate,
                max_rate: value.max_rate,
                max_change_rate: value.max_change_rate,
            }
        }
    }

    impl From<CommissionRates> for protos::cosmos::staking::v1beta1::CommissionRates {
        fn from(value: CommissionRates) -> Self {
            Self {
                rate: value.rate,
                max_rate: value.max_rate,
                max_change_rate: value.max_change_rate,
            }
        }
    }
}

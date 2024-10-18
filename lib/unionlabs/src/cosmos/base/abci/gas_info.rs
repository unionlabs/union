use macros::model;

#[model(proto(raw(protos::cosmos::base::abci::v1beta1::GasInfo), from, into))]
pub struct GasInfo {
    pub gas_wanted: u64,
    pub gas_used: u64,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::cosmos::base::abci::gas_info::GasInfo;

    impl From<protos::cosmos::base::abci::v1beta1::GasInfo> for GasInfo {
        fn from(value: protos::cosmos::base::abci::v1beta1::GasInfo) -> Self {
            Self {
                gas_wanted: value.gas_wanted,
                gas_used: value.gas_used,
            }
        }
    }

    impl From<GasInfo> for protos::cosmos::base::abci::v1beta1::GasInfo {
        fn from(value: GasInfo) -> Self {
            Self {
                gas_wanted: value.gas_wanted,
                gas_used: value.gas_used,
            }
        }
    }
}

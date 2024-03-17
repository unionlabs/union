use crate::macros::wrapper_enum;

wrapper_enum! {
    #[model(proto(protos::cosmos::staking::v1beta1::BondStatus))]
    pub enum BondStatus {
        Unspecified = 0,
        Unbonded = 1,
        Unbonding = 2,
        Bonded = 3,
    }
}

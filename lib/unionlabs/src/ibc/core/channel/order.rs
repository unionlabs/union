use crate::macros::wrapper_enum;

wrapper_enum! {
    #[model(proto(protos::ibc::core::channel::v1::Order))]
    #[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
    pub enum Order {
        NoneUnspecified = 0,
        Unordered = 1,
        Ordered = 2,
    }
}

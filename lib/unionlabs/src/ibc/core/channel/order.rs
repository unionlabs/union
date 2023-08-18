use crate::macros::wrapper_enum;

wrapper_enum! {
    #[proto(protos::ibc::core::channel::v1::Order)]
    pub enum Order {
        NoneUnspecified = 0,
        Unordered = 1,
        Ordered = 2,
    }
}

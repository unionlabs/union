use crate::macros::wrapper_enum;

wrapper_enum! {
    #[model(proto(protos::ibc::core::channel::v1::State))]
    pub enum State {
        UninitializedUnspecified = 0,
        Init = 1,
        Tryopen = 2,
        Open = 3,
        Closed = 4,
    }
}

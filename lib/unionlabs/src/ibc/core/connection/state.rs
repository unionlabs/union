use crate::macros::wrapper_enum;

wrapper_enum! {
    #[model(proto(protos::ibc::core::connection::v1::State))]
    pub enum State {
        /// Default State
        UninitializedUnspecified = 0,
        /// A connection end has just started the opening handshake.
        Init = 1,
        /// A connection end has acknowledged the handshake step on the counterparty
        /// chain.
        Tryopen = 2,
        /// A connection end has completed the handshake.
        Open = 3,
    }
}

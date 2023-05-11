/// ClientState defines the 09-localhost client state
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientState {
    /// the latest block height
    #[prost(message, optional, tag = "1")]
    pub latest_height: ::core::option::Option<
        super::super::super::core::client::v1::Height,
    >,
}

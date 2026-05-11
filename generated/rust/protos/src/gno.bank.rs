/// MsgSend is the fund transfer tx message
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgSend {
    /// the bech32 address of the fund sender
    #[prost(string, tag = "1")]
    pub from_address: ::prost::alloc::string::String,
    /// the bech32 address of the fund receiver
    #[prost(string, tag = "2")]
    pub to_address: ::prost::alloc::string::String,
    /// the denomination and amount of fund sent ("<amount><denomination>")
    #[prost(string, tag = "3")]
    pub amount: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgSend {
    const NAME: &'static str = "MsgSend";
    const PACKAGE: &'static str = "gno.bank";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("gno.bank.{}", Self::NAME)
    }
}

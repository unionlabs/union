#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct PubKeySecp256k1 {
    #[prost(bytes = "vec", tag = "1")]
    pub key: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct Tx {
    /// specific message types
    #[prost(message, repeated, tag = "1")]
    pub messages: ::prost::alloc::vec::Vec<super::super::google::protobuf::Any>,
    /// transaction costs (fee)
    #[prost(message, optional, tag = "2")]
    pub fee: ::core::option::Option<TxFee>,
    /// the signatures for the transaction
    #[prost(message, repeated, tag = "3")]
    pub signatures: ::prost::alloc::vec::Vec<TxSignature>,
    /// memo attached to the transaction
    #[prost(string, tag = "4")]
    pub memo: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct TxFee {
    /// gas limit
    #[prost(sint64, tag = "1")]
    pub gas_wanted: i64,
    /// gas fee details (<value><denomination>)
    #[prost(string, tag = "2")]
    pub gas_fee: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct TxSignature {
    /// public key associated with the signature
    #[prost(message, optional, tag = "1")]
    pub pub_key: ::core::option::Option<super::super::google::protobuf::Any>,
    /// the signature
    #[prost(bytes = "vec", tag = "2")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for PubKeySecp256k1 {
    const NAME: &'static str = "PubKeySecp256k1";
    const PACKAGE: &'static str = "tm2.tx";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tm2.tx.{}", Self::NAME)
    }
}
impl ::prost::Name for Tx {
    const NAME: &'static str = "Tx";
    const PACKAGE: &'static str = "tm2.tx";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tm2.tx.{}", Self::NAME)
    }
}
impl ::prost::Name for TxFee {
    const NAME: &'static str = "TxFee";
    const PACKAGE: &'static str = "tm2.tx";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tm2.tx.{}", Self::NAME)
    }
}
impl ::prost::Name for TxSignature {
    const NAME: &'static str = "TxSignature";
    const PACKAGE: &'static str = "tm2.tx";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tm2.tx.{}", Self::NAME)
    }
}

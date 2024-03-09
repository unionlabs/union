// @generated
/// Coin defines a token with a denomination and an amount.
///
/// NOTE: The amount field is an Int which implements the custom method
/// signatures required by gogoproto.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Coin {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub amount: ::prost::alloc::string::String,
}
impl ::prost::Name for Coin {
    const NAME: &'static str = "Coin";
    const PACKAGE: &'static str = "cosmos.base.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.v1beta1.{}", Self::NAME)
    }
}
/// DecCoin defines a token with a denomination and a decimal amount.
///
/// NOTE: The amount field is an Dec which implements the custom method
/// signatures required by gogoproto.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DecCoin {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub amount: ::prost::alloc::string::String,
}
impl ::prost::Name for DecCoin {
    const NAME: &'static str = "DecCoin";
    const PACKAGE: &'static str = "cosmos.base.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.v1beta1.{}", Self::NAME)
    }
}
/// IntProto defines a Protobuf wrapper around an Int object.
/// Deprecated: Prefer to use math.Int directly. It supports binary Marshal and Unmarshal.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IntProto {
    #[prost(string, tag = "1")]
    pub int: ::prost::alloc::string::String,
}
impl ::prost::Name for IntProto {
    const NAME: &'static str = "IntProto";
    const PACKAGE: &'static str = "cosmos.base.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.v1beta1.{}", Self::NAME)
    }
}
/// DecProto defines a Protobuf wrapper around a Dec object.
/// Deprecated: Prefer to use math.LegacyDec directly. It supports binary Marshal and Unmarshal.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DecProto {
    #[prost(string, tag = "1")]
    pub dec: ::prost::alloc::string::String,
}
impl ::prost::Name for DecProto {
    const NAME: &'static str = "DecProto";
    const PACKAGE: &'static str = "cosmos.base.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.v1beta1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)

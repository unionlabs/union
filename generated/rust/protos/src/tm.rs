#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct CompactBitArray {
    /// The number of extra bits in elems.
    #[prost(uint32, tag = "1")]
    pub extra_bits_stored: u32,
    #[prost(bytes = "vec", tag = "2")]
    pub elems: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct Multisignature {
    #[prost(message, optional, tag = "1")]
    pub bit_array: ::core::option::Option<CompactBitArray>,
    #[prost(bytes = "vec", repeated, tag = "2")]
    pub sigs: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
/// messages
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct PubKeyMultisig {
    #[prost(uint64, tag = "1")]
    pub k: u64,
    #[prost(message, repeated, tag = "2")]
    pub pub_keys: ::prost::alloc::vec::Vec<super::google::protobuf::Any>,
}
impl ::prost::Name for CompactBitArray {
    const NAME: &'static str = "CompactBitArray";
    const PACKAGE: &'static str = "tm";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tm.{}", Self::NAME)
    }
}
impl ::prost::Name for Multisignature {
    const NAME: &'static str = "Multisignature";
    const PACKAGE: &'static str = "tm";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tm.{}", Self::NAME)
    }
}
impl ::prost::Name for PubKeyMultisig {
    const NAME: &'static str = "PubKeyMultisig";
    const PACKAGE: &'static str = "tm";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tm.{}", Self::NAME)
    }
}

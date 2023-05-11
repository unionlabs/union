/// MultiSignature wraps the signatures from a multisig.LegacyAminoPubKey.
/// See cosmos.tx.v1betata1.ModeInfo.Multi for how to specify which signers
/// signed and with which modes.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MultiSignature {
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub signatures: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
/// CompactBitArray is an implementation of a space efficient bit array.
/// This is used to ensure that the encoded data takes up a minimal amount of
/// space after proto encoding.
/// This is not thread safe, and is not intended for concurrent usage.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CompactBitArray {
    #[prost(uint32, tag = "1")]
    pub extra_bits_stored: u32,
    #[prost(bytes = "vec", tag = "2")]
    pub elems: ::prost::alloc::vec::Vec<u8>,
}

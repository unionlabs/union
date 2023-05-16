// @generated
/// PubKey defines a secp256r1 ECDSA public key.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PubKey {
    /// Point on secp256r1 curve in a compressed representation as specified in section
    /// 4.3.6 of ANSI X9.62: <https://webstore.ansi.org/standards/ascx9/ansix9621998>
    #[prost(bytes = "vec", tag = "1")]
    pub key: ::prost::alloc::vec::Vec<u8>,
}
/// PrivKey defines a secp256r1 ECDSA private key.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PrivKey {
    /// secret number serialized using big-endian encoding
    #[prost(bytes = "vec", tag = "1")]
    pub secret: ::prost::alloc::vec::Vec<u8>,
}
// @@protoc_insertion_point(module)

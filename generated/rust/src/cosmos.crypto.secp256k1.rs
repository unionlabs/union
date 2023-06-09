// @generated
/// PubKey defines a secp256k1 public key
/// Key is the compressed form of the pubkey. The first byte depends is a 0x02 byte
/// if the y-coordinate is the lexicographically largest of the two associated with
/// the x-coordinate. Otherwise the first byte is a 0x03.
/// This prefix is followed with the x-coordinate.
#[cfg_attr(
    feature = "ethers",
    derive(::ethers::contract::EthAbiType, ::ethers::contract::EthAbiCodec)
)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PubKey {
    #[prost(bytes = "vec", tag = "1")]
    pub key: ::prost::alloc::vec::Vec<u8>,
}
/// PrivKey defines a secp256k1 private key.
#[cfg_attr(
    feature = "ethers",
    derive(::ethers::contract::EthAbiType, ::ethers::contract::EthAbiCodec)
)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PrivKey {
    #[prost(bytes = "vec", tag = "1")]
    pub key: ::prost::alloc::vec::Vec<u8>,
}
// @@protoc_insertion_point(module)

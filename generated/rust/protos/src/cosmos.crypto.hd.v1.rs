// @generated
/// BIP44Params is used as path field in ledger item in Record.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Bip44Params {
    /// purpose is a constant set to 44' (or 0x8000002C) following the BIP43 recommendation
    #[prost(uint32, tag = "1")]
    pub purpose: u32,
    /// coin_type is a constant that improves privacy
    #[prost(uint32, tag = "2")]
    pub coin_type: u32,
    /// account splits the key space into independent user identities
    #[prost(uint32, tag = "3")]
    pub account: u32,
    /// change is a constant used for public derivation. Constant 0 is used for external chain and constant 1 for internal
    /// chain.
    #[prost(bool, tag = "4")]
    pub change: bool,
    /// address_index is used as child index in BIP32 derivation
    #[prost(uint32, tag = "5")]
    pub address_index: u32,
}
// @@protoc_insertion_point(module)

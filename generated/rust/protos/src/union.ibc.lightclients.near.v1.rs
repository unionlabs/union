// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Header {
    #[prost(message, optional, tag = "1")]
    pub new_state: ::core::option::Option<LightClientBlockView>,
    #[prost(int64, tag = "2")]
    pub trusted_height: i64,
    #[prost(message, repeated, tag = "3")]
    pub prev_state_root_proof: ::prost::alloc::vec::Vec<MerklePathItem>,
    #[prost(bytes = "vec", tag = "4")]
    pub prev_state_root: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for Header {
    const NAME: &'static str = "Header";
    const PACKAGE: &'static str = "union.ibc.lightclients.near.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.near.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MerklePathItem {
    #[prost(bytes = "vec", tag = "1")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag = "2")]
    pub direction: i64,
}
impl ::prost::Name for MerklePathItem {
    const NAME: &'static str = "MerklePathItem";
    const PACKAGE: &'static str = "union.ibc.lightclients.near.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.near.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LightClientBlockView {
    #[prost(bytes = "vec", tag = "1")]
    pub prev_block_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub next_block_inner_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub inner_lite: ::core::option::Option<BlockHeaderInnerLiteView>,
    #[prost(bytes = "vec", tag = "4")]
    pub inner_rest_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, repeated, tag = "5")]
    pub next_bps: ::prost::alloc::vec::Vec<ValidatorStakeView>,
    #[prost(message, repeated, tag = "6")]
    pub approvals_after_next: ::prost::alloc::vec::Vec<Signature>,
}
impl ::prost::Name for LightClientBlockView {
    const NAME: &'static str = "LightClientBlockView";
    const PACKAGE: &'static str = "union.ibc.lightclients.near.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.near.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Signature {
    #[prost(bytes = "vec", tag = "1")]
    pub ed25519: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub secp256k1: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for Signature {
    const NAME: &'static str = "Signature";
    const PACKAGE: &'static str = "union.ibc.lightclients.near.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.near.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorStakeView {
    #[prost(string, tag = "1")]
    pub account_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub public_key: ::core::option::Option<PublicKey>,
    /// TODO(aeryz): u128
    #[prost(int64, tag = "3")]
    pub balance: i64,
}
impl ::prost::Name for ValidatorStakeView {
    const NAME: &'static str = "ValidatorStakeView";
    const PACKAGE: &'static str = "union.ibc.lightclients.near.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.near.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PublicKey {
    #[prost(bytes = "vec", tag = "1")]
    pub ed25519: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub secp256k1: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for PublicKey {
    const NAME: &'static str = "PublicKey";
    const PACKAGE: &'static str = "union.ibc.lightclients.near.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.near.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockHeaderInnerLiteView {
    #[prost(bytes = "vec", tag = "1")]
    pub height: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub epoch_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    pub next_epoch_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "4")]
    pub prev_state_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "5")]
    pub outcome_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag = "6")]
    pub timestamp: i64,
    #[prost(int64, tag = "7")]
    pub timestamp_nanosec: i64,
    #[prost(bytes = "vec", tag = "8")]
    pub next_bp_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "9")]
    pub block_merkle_root: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for BlockHeaderInnerLiteView {
    const NAME: &'static str = "BlockHeaderInnerLiteView";
    const PACKAGE: &'static str = "union.ibc.lightclients.near.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("union.ibc.lightclients.near.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)

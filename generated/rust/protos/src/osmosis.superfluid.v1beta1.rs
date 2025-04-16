// @generated
/// SetSuperfluidAssetsProposal is a gov Content type to update the superfluid
/// assets
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetSuperfluidAssetsProposal {
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "3")]
    pub assets: ::prost::alloc::vec::Vec<super::SuperfluidAsset>,
}
impl ::prost::Name for SetSuperfluidAssetsProposal {
    const NAME: &'static str = "SetSuperfluidAssetsProposal";
    const PACKAGE: &'static str = "osmosis.superfluid.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.v1beta1.{}", Self::NAME)
    }
}
/// RemoveSuperfluidAssetsProposal is a gov Content type to remove the superfluid
/// assets by denom
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RemoveSuperfluidAssetsProposal {
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "3")]
    pub superfluid_asset_denoms: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for RemoveSuperfluidAssetsProposal {
    const NAME: &'static str = "RemoveSuperfluidAssetsProposal";
    const PACKAGE: &'static str = "osmosis.superfluid.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.v1beta1.{}", Self::NAME)
    }
}
/// UpdateUnpoolWhiteListProposal is a gov Content type to update the
/// allowed list of pool ids.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateUnpoolWhiteListProposal {
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    #[prost(uint64, repeated, tag = "3")]
    pub ids: ::prost::alloc::vec::Vec<u64>,
    #[prost(bool, tag = "4")]
    pub is_overwrite: bool,
}
impl ::prost::Name for UpdateUnpoolWhiteListProposal {
    const NAME: &'static str = "UpdateUnpoolWhiteListProposal";
    const PACKAGE: &'static str = "osmosis.superfluid.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("osmosis.superfluid.v1beta1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)

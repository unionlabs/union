// @generated
/// Capability defines an implementation of an object capability. The index
/// provided to a Capability must be globally unique.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Capability {
    #[prost(uint64, tag = "1")]
    pub index: u64,
}
impl ::prost::Name for Capability {
    const NAME: &'static str = "Capability";
    const PACKAGE: &'static str = "capability.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("capability.v1.{}", Self::NAME)
    }
}
/// Owner defines a single capability owner. An owner is defined by the name of
/// capability and the module name.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Owner {
    #[prost(string, tag = "1")]
    pub module: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
}
impl ::prost::Name for Owner {
    const NAME: &'static str = "Owner";
    const PACKAGE: &'static str = "capability.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("capability.v1.{}", Self::NAME)
    }
}
/// CapabilityOwners defines a set of owners of a single Capability. The set of
/// owners must be unique.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CapabilityOwners {
    #[prost(message, repeated, tag = "1")]
    pub owners: ::prost::alloc::vec::Vec<Owner>,
}
impl ::prost::Name for CapabilityOwners {
    const NAME: &'static str = "CapabilityOwners";
    const PACKAGE: &'static str = "capability.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("capability.v1.{}", Self::NAME)
    }
}
/// GenesisOwners defines the capability owners with their corresponding index.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisOwners {
    /// index is the index of the capability owner.
    #[prost(uint64, tag = "1")]
    pub index: u64,
    /// index_owners are the owners at the given index.
    #[prost(message, optional, tag = "2")]
    pub index_owners: ::core::option::Option<CapabilityOwners>,
}
impl ::prost::Name for GenesisOwners {
    const NAME: &'static str = "GenesisOwners";
    const PACKAGE: &'static str = "capability.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("capability.v1.{}", Self::NAME)
    }
}
/// GenesisState defines the capability module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// index is the capability global index.
    #[prost(uint64, tag = "1")]
    pub index: u64,
    /// owners represents a map from index to owners of the capability index
    /// index key is string to allow amino marshalling.
    #[prost(message, repeated, tag = "2")]
    pub owners: ::prost::alloc::vec::Vec<GenesisOwners>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "capability.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("capability.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)

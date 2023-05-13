// @generated
/// Record is used for representing a key in the keyring.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Record {
    /// name represents a name of Record
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// pub_key represents a public key in any format
    #[prost(message, optional, tag = "2")]
    pub pub_key: ::core::option::Option<::pbjson_types::Any>,
    /// Record contains one of the following items
    #[prost(oneof = "record::Item", tags = "3, 4, 5, 6")]
    pub item: ::core::option::Option<record::Item>,
}
/// Nested message and enum types in `Record`.
pub mod record {
    /// Item is a keyring item stored in a keyring backend.
    /// Local item
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Local {
        #[prost(message, optional, tag = "1")]
        pub priv_key: ::core::option::Option<::pbjson_types::Any>,
    }
    /// Ledger item
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Ledger {
        #[prost(message, optional, tag = "1")]
        pub path: ::core::option::Option<super::super::super::hd::v1::Bip44Params>,
    }
    /// Multi item
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Multi {}
    /// Offline item
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Offline {}
    /// Record contains one of the following items
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Item {
        /// local stores the private key locally.
        #[prost(message, tag = "3")]
        Local(Local),
        /// ledger stores the information about a Ledger key.
        #[prost(message, tag = "4")]
        Ledger(Ledger),
        /// Multi does not store any other information.
        #[prost(message, tag = "5")]
        Multi(Multi),
        /// Offline does not store any other information.
        #[prost(message, tag = "6")]
        Offline(Offline),
    }
}
// @@protoc_insertion_point(module)

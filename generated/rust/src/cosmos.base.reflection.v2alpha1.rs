// @generated
/// AppDescriptor describes a cosmos-sdk based application
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AppDescriptor {
    /// AuthnDescriptor provides information on how to authenticate transactions on the application
    /// NOTE: experimental and subject to change in future releases.
    #[prost(message, optional, tag = "1")]
    pub authn: ::core::option::Option<AuthnDescriptor>,
    /// chain provides the chain descriptor
    #[prost(message, optional, tag = "2")]
    pub chain: ::core::option::Option<ChainDescriptor>,
    /// codec provides metadata information regarding codec related types
    #[prost(message, optional, tag = "3")]
    pub codec: ::core::option::Option<CodecDescriptor>,
    /// configuration provides metadata information regarding the sdk.Config type
    #[prost(message, optional, tag = "4")]
    pub configuration: ::core::option::Option<ConfigurationDescriptor>,
    /// query_services provides metadata information regarding the available queriable endpoints
    #[prost(message, optional, tag = "5")]
    pub query_services: ::core::option::Option<QueryServicesDescriptor>,
    /// tx provides metadata information regarding how to send transactions to the given application
    #[prost(message, optional, tag = "6")]
    pub tx: ::core::option::Option<TxDescriptor>,
}
/// TxDescriptor describes the accepted transaction type
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxDescriptor {
    /// fullname is the protobuf fullname of the raw transaction type (for instance the tx.Tx type)
    /// it is not meant to support polymorphism of transaction types, it is supposed to be used by
    /// reflection clients to understand if they can handle a specific transaction type in an application.
    #[prost(string, tag = "1")]
    pub fullname: ::prost::alloc::string::String,
    /// msgs lists the accepted application messages (sdk.Msg)
    #[prost(message, repeated, tag = "2")]
    pub msgs: ::prost::alloc::vec::Vec<MsgDescriptor>,
}
/// AuthnDescriptor provides information on how to sign transactions without relying
/// on the online RPCs GetTxMetadata and CombineUnsignedTxAndSignatures
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AuthnDescriptor {
    /// sign_modes defines the supported signature algorithm
    #[prost(message, repeated, tag = "1")]
    pub sign_modes: ::prost::alloc::vec::Vec<SigningModeDescriptor>,
}
/// SigningModeDescriptor provides information on a signing flow of the application
/// NOTE(fdymylja): here we could go as far as providing an entire flow on how
/// to sign a message given a SigningModeDescriptor, but it's better to think about
/// this another time
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SigningModeDescriptor {
    /// name defines the unique name of the signing mode
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// number is the unique int32 identifier for the sign_mode enum
    #[prost(int32, tag = "2")]
    pub number: i32,
    /// authn_info_provider_method_fullname defines the fullname of the method to call to get
    /// the metadata required to authenticate using the provided sign_modes
    #[prost(string, tag = "3")]
    pub authn_info_provider_method_fullname: ::prost::alloc::string::String,
}
/// ChainDescriptor describes chain information of the application
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChainDescriptor {
    /// id is the chain id
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
}
/// CodecDescriptor describes the registered interfaces and provides metadata information on the types
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CodecDescriptor {
    /// interfaces is a list of the registerted interfaces descriptors
    #[prost(message, repeated, tag = "1")]
    pub interfaces: ::prost::alloc::vec::Vec<InterfaceDescriptor>,
}
/// InterfaceDescriptor describes the implementation of an interface
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InterfaceDescriptor {
    /// fullname is the name of the interface
    #[prost(string, tag = "1")]
    pub fullname: ::prost::alloc::string::String,
    /// interface_accepting_messages contains information regarding the proto messages which contain the interface as
    /// google.protobuf.Any field
    #[prost(message, repeated, tag = "2")]
    pub interface_accepting_messages: ::prost::alloc::vec::Vec<InterfaceAcceptingMessageDescriptor>,
    /// interface_implementers is a list of the descriptors of the interface implementers
    #[prost(message, repeated, tag = "3")]
    pub interface_implementers: ::prost::alloc::vec::Vec<InterfaceImplementerDescriptor>,
}
/// InterfaceImplementerDescriptor describes an interface implementer
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InterfaceImplementerDescriptor {
    /// fullname is the protobuf queryable name of the interface implementer
    #[prost(string, tag = "1")]
    pub fullname: ::prost::alloc::string::String,
    /// type_url defines the type URL used when marshalling the type as any
    /// this is required so we can provide type safe google.protobuf.Any marshalling and
    /// unmarshalling, making sure that we don't accept just 'any' type
    /// in our interface fields
    #[prost(string, tag = "2")]
    pub type_url: ::prost::alloc::string::String,
}
/// InterfaceAcceptingMessageDescriptor describes a protobuf message which contains
/// an interface represented as a google.protobuf.Any
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InterfaceAcceptingMessageDescriptor {
    /// fullname is the protobuf fullname of the type containing the interface
    #[prost(string, tag = "1")]
    pub fullname: ::prost::alloc::string::String,
    /// field_descriptor_names is a list of the protobuf name (not fullname) of the field
    /// which contains the interface as google.protobuf.Any (the interface is the same, but
    /// it can be in multiple fields of the same proto message)
    #[prost(string, repeated, tag = "2")]
    pub field_descriptor_names: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// ConfigurationDescriptor contains metadata information on the sdk.Config
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConfigurationDescriptor {
    /// bech32_account_address_prefix is the account address prefix
    #[prost(string, tag = "1")]
    pub bech32_account_address_prefix: ::prost::alloc::string::String,
}
/// MsgDescriptor describes a cosmos-sdk message that can be delivered with a transaction
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgDescriptor {
    /// msg_type_url contains the TypeURL of a sdk.Msg.
    #[prost(string, tag = "1")]
    pub msg_type_url: ::prost::alloc::string::String,
}
/// GetAuthnDescriptorRequest is the request used for the GetAuthnDescriptor RPC
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetAuthnDescriptorRequest {}
/// GetAuthnDescriptorResponse is the response returned by the GetAuthnDescriptor RPC
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetAuthnDescriptorResponse {
    /// authn describes how to authenticate to the application when sending transactions
    #[prost(message, optional, tag = "1")]
    pub authn: ::core::option::Option<AuthnDescriptor>,
}
/// GetChainDescriptorRequest is the request used for the GetChainDescriptor RPC
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetChainDescriptorRequest {}
/// GetChainDescriptorResponse is the response returned by the GetChainDescriptor RPC
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetChainDescriptorResponse {
    /// chain describes application chain information
    #[prost(message, optional, tag = "1")]
    pub chain: ::core::option::Option<ChainDescriptor>,
}
/// GetCodecDescriptorRequest is the request used for the GetCodecDescriptor RPC
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetCodecDescriptorRequest {}
/// GetCodecDescriptorResponse is the response returned by the GetCodecDescriptor RPC
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetCodecDescriptorResponse {
    /// codec describes the application codec such as registered interfaces and implementations
    #[prost(message, optional, tag = "1")]
    pub codec: ::core::option::Option<CodecDescriptor>,
}
/// GetConfigurationDescriptorRequest is the request used for the GetConfigurationDescriptor RPC
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetConfigurationDescriptorRequest {}
/// GetConfigurationDescriptorResponse is the response returned by the GetConfigurationDescriptor RPC
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetConfigurationDescriptorResponse {
    /// config describes the application's sdk.Config
    #[prost(message, optional, tag = "1")]
    pub config: ::core::option::Option<ConfigurationDescriptor>,
}
/// GetQueryServicesDescriptorRequest is the request used for the GetQueryServicesDescriptor RPC
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetQueryServicesDescriptorRequest {}
/// GetQueryServicesDescriptorResponse is the response returned by the GetQueryServicesDescriptor RPC
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetQueryServicesDescriptorResponse {
    /// queries provides information on the available queryable services
    #[prost(message, optional, tag = "1")]
    pub queries: ::core::option::Option<QueryServicesDescriptor>,
}
/// GetTxDescriptorRequest is the request used for the GetTxDescriptor RPC
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetTxDescriptorRequest {}
/// GetTxDescriptorResponse is the response returned by the GetTxDescriptor RPC
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetTxDescriptorResponse {
    /// tx provides information on msgs that can be forwarded to the application
    /// alongside the accepted transaction protobuf type
    #[prost(message, optional, tag = "1")]
    pub tx: ::core::option::Option<TxDescriptor>,
}
/// QueryServicesDescriptor contains the list of cosmos-sdk queriable services
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryServicesDescriptor {
    /// query_services is a list of cosmos-sdk QueryServiceDescriptor
    #[prost(message, repeated, tag = "1")]
    pub query_services: ::prost::alloc::vec::Vec<QueryServiceDescriptor>,
}
/// QueryServiceDescriptor describes a cosmos-sdk queryable service
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryServiceDescriptor {
    /// fullname is the protobuf fullname of the service descriptor
    #[prost(string, tag = "1")]
    pub fullname: ::prost::alloc::string::String,
    /// is_module describes if this service is actually exposed by an application's module
    #[prost(bool, tag = "2")]
    pub is_module: bool,
    /// methods provides a list of query service methods
    #[prost(message, repeated, tag = "3")]
    pub methods: ::prost::alloc::vec::Vec<QueryMethodDescriptor>,
}
/// QueryMethodDescriptor describes a queryable method of a query service
/// no other info is provided beside method name and tendermint queryable path
/// because it would be redundant with the grpc reflection service
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryMethodDescriptor {
    /// name is the protobuf name (not fullname) of the method
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// full_query_path is the path that can be used to query
    /// this method via tendermint abci.Query
    #[prost(string, tag = "2")]
    pub full_query_path: ::prost::alloc::string::String,
}
include!("cosmos.base.reflection.v2alpha1.tonic.rs");
// @@protoc_insertion_point(module)

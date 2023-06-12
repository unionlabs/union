// @generated
/// FileDescriptorsRequest is the Query/FileDescriptors request type.
#[cfg_attr(
    feature = "ethers",
    derive(::ethers::contract::EthAbiType, ::ethers::contract::EthAbiCodec)
)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FileDescriptorsRequest {}
/// FileDescriptorsResponse is the Query/FileDescriptors response type.
#[cfg_attr(
    feature = "ethers",
    derive(::ethers::contract::EthAbiType, ::ethers::contract::EthAbiCodec)
)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FileDescriptorsResponse {
    /// files is the file descriptors.
    #[prost(message, repeated, tag = "1")]
    pub files: ::prost::alloc::vec::Vec<super::super::super::google::protobuf::FileDescriptorProto>,
}
include!("cosmos.reflection.v1.tonic.rs");
// @@protoc_insertion_point(module)

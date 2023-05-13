// @generated
/// FileDescriptorsRequest is the Query/FileDescriptors request type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FileDescriptorsRequest {}
/// FileDescriptorsResponse is the Query/FileDescriptors response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FileDescriptorsResponse {
    /// files is the file descriptors.
    #[prost(message, repeated, tag = "1")]
    pub files: ::prost::alloc::vec::Vec<::pbjson_types::FileDescriptorProto>,
}
include!("cosmos.reflection.v1.tonic.rs");
// @@protoc_insertion_point(module)

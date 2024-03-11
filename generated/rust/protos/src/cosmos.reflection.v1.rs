// @generated
/// FileDescriptorsRequest is the Query/FileDescriptors request type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FileDescriptorsRequest {}
impl ::prost::Name for FileDescriptorsRequest {
    const NAME: &'static str = "FileDescriptorsRequest";
    const PACKAGE: &'static str = "cosmos.reflection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.reflection.v1.{}", Self::NAME)
    }
}
/// FileDescriptorsResponse is the Query/FileDescriptors response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FileDescriptorsResponse {
    /// files is the file descriptors.
    #[prost(message, repeated, tag = "1")]
    pub files: ::prost::alloc::vec::Vec<super::super::super::google::protobuf::FileDescriptorProto>,
}
impl ::prost::Name for FileDescriptorsResponse {
    const NAME: &'static str = "FileDescriptorsResponse";
    const PACKAGE: &'static str = "cosmos.reflection.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.reflection.v1.{}", Self::NAME)
    }
}
include!("cosmos.reflection.v1.tonic.rs");
// @@protoc_insertion_point(module)

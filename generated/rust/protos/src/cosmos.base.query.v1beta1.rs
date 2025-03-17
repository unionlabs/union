// @generated
/// PageRequest is to be embedded in gRPC request messages for efficient
/// pagination. Ex:
///
///   message SomeRequest {
///           Foo some_parameter = 1;
///           PageRequest pagination = 2;
///   }
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PageRequest {
    /// key is a value returned in PageResponse.next_key to begin
    /// querying the next page most efficiently. Only one of offset or key
    /// should be set.
    #[prost(bytes = "vec", tag = "1")]
    pub key: ::prost::alloc::vec::Vec<u8>,
    /// offset is a numeric offset that can be used when key is unavailable.
    /// It is less efficient than using key. Only one of offset or key should
    /// be set.
    #[prost(uint64, tag = "2")]
    pub offset: u64,
    /// limit is the total number of results to be returned in the result page.
    /// If left empty it will default to a value to be set by each app.
    #[prost(uint64, tag = "3")]
    pub limit: u64,
    /// count_total is set to true  to indicate that the result set should include
    /// a count of the total number of items available for pagination in UIs.
    /// count_total is only respected when offset is used. It is ignored when key
    /// is set.
    #[prost(bool, tag = "4")]
    pub count_total: bool,
    /// reverse is set to true if results are to be returned in the descending order.
    ///
    /// Since: cosmos-sdk 0.43
    #[prost(bool, tag = "5")]
    pub reverse: bool,
}
impl ::prost::Name for PageRequest {
    const NAME: &'static str = "PageRequest";
    const PACKAGE: &'static str = "cosmos.base.query.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.query.v1beta1.{}", Self::NAME)
    }
}
/// PageResponse is to be embedded in gRPC response messages where the
/// corresponding request message has used PageRequest.
///
///   message SomeResponse {
///           repeated Bar results = 1;
///           PageResponse page = 2;
///   }
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PageResponse {
    /// next_key is the key to be passed to PageRequest.key to
    /// query the next page most efficiently. It will be empty if
    /// there are no more results.
    #[prost(bytes = "vec", tag = "1")]
    pub next_key: ::prost::alloc::vec::Vec<u8>,
    /// total is total number of results available if PageRequest.count_total
    /// was set, its value is undefined otherwise
    #[prost(uint64, tag = "2")]
    pub total: u64,
}
impl ::prost::Name for PageResponse {
    const NAME: &'static str = "PageResponse";
    const PACKAGE: &'static str = "cosmos.base.query.v1beta1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.base.query.v1beta1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)

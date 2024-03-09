// @generated
/// GetRequest is the Query/Get request type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetRequest {
    /// message_name is the fully-qualified message name of the ORM table being queried.
    #[prost(string, tag = "1")]
    pub message_name: ::prost::alloc::string::String,
    /// index is the index fields expression used in orm definitions. If it
    /// is empty, the table's primary key is assumed. If it is non-empty, it must
    /// refer to an unique index.
    #[prost(string, tag = "2")]
    pub index: ::prost::alloc::string::String,
    /// values are the values of the fields corresponding to the requested index.
    /// There must be as many values provided as there are fields in the index and
    /// these values must correspond to the index field types.
    #[prost(message, repeated, tag = "3")]
    pub values: ::prost::alloc::vec::Vec<IndexValue>,
}
impl ::prost::Name for GetRequest {
    const NAME: &'static str = "GetRequest";
    const PACKAGE: &'static str = "cosmos.orm.query.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.orm.query.v1alpha1.{}", Self::NAME)
    }
}
/// GetResponse is the Query/Get response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetResponse {
    /// result is the result of the get query. If no value is found, the gRPC
    /// status code NOT_FOUND will be returned.
    #[prost(message, optional, tag = "1")]
    pub result: ::core::option::Option<super::super::super::super::google::protobuf::Any>,
}
impl ::prost::Name for GetResponse {
    const NAME: &'static str = "GetResponse";
    const PACKAGE: &'static str = "cosmos.orm.query.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.orm.query.v1alpha1.{}", Self::NAME)
    }
}
/// ListRequest is the Query/List request type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListRequest {
    /// message_name is the fully-qualified message name of the ORM table being queried.
    #[prost(string, tag = "1")]
    pub message_name: ::prost::alloc::string::String,
    /// index is the index fields expression used in orm definitions. If it
    /// is empty, the table's primary key is assumed.
    #[prost(string, tag = "2")]
    pub index: ::prost::alloc::string::String,
    /// pagination is the pagination request.
    #[prost(message, optional, tag = "5")]
    pub pagination: ::core::option::Option<super::super::super::base::query::v1beta1::PageRequest>,
    /// query is the query expression corresponding to the provided index. If
    /// neither prefix nor range is specified, the query will list all the fields
    /// in the index.
    #[prost(oneof = "list_request::Query", tags = "3, 4")]
    pub query: ::core::option::Option<list_request::Query>,
}
/// Nested message and enum types in `ListRequest`.
pub mod list_request {
    /// Prefix specifies the arguments to a prefix query.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Prefix {
        /// values specifies the index values for the prefix query.
        /// It is valid to special a partial prefix with fewer values than
        /// the number of fields in the index.
        #[prost(message, repeated, tag = "1")]
        pub values: ::prost::alloc::vec::Vec<super::IndexValue>,
    }
    impl ::prost::Name for Prefix {
        const NAME: &'static str = "Prefix";
        const PACKAGE: &'static str = "cosmos.orm.query.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!("cosmos.orm.query.v1alpha1.ListRequest.{}", Self::NAME)
        }
    }
    /// Range specifies the arguments to a range query.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Range {
        /// start specifies the starting index values for the range query.
        /// It is valid to provide fewer values than the number of fields in the
        /// index.
        #[prost(message, repeated, tag = "1")]
        pub start: ::prost::alloc::vec::Vec<super::IndexValue>,
        /// end specifies the inclusive ending index values for the range query.
        /// It is valid to provide fewer values than the number of fields in the
        /// index.
        #[prost(message, repeated, tag = "2")]
        pub end: ::prost::alloc::vec::Vec<super::IndexValue>,
    }
    impl ::prost::Name for Range {
        const NAME: &'static str = "Range";
        const PACKAGE: &'static str = "cosmos.orm.query.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!("cosmos.orm.query.v1alpha1.ListRequest.{}", Self::NAME)
        }
    }
    /// query is the query expression corresponding to the provided index. If
    /// neither prefix nor range is specified, the query will list all the fields
    /// in the index.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Query {
        /// prefix defines a prefix query.
        #[prost(message, tag = "3")]
        Prefix(Prefix),
        /// range defines a range query.
        #[prost(message, tag = "4")]
        Range(Range),
    }
}
impl ::prost::Name for ListRequest {
    const NAME: &'static str = "ListRequest";
    const PACKAGE: &'static str = "cosmos.orm.query.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.orm.query.v1alpha1.{}", Self::NAME)
    }
}
/// ListResponse is the Query/List response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListResponse {
    /// results are the results of the query.
    #[prost(message, repeated, tag = "1")]
    pub results: ::prost::alloc::vec::Vec<super::super::super::super::google::protobuf::Any>,
    /// pagination is the pagination response.
    #[prost(message, optional, tag = "5")]
    pub pagination: ::core::option::Option<super::super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for ListResponse {
    const NAME: &'static str = "ListResponse";
    const PACKAGE: &'static str = "cosmos.orm.query.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.orm.query.v1alpha1.{}", Self::NAME)
    }
}
/// IndexValue represents the value of a field in an ORM index expression.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IndexValue {
    /// value specifies the index value
    #[prost(oneof = "index_value::Value", tags = "1, 2, 3, 4, 5, 6, 7, 8")]
    pub value: ::core::option::Option<index_value::Value>,
}
/// Nested message and enum types in `IndexValue`.
pub mod index_value {
    /// value specifies the index value
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        /// uint specifies a value for an uint32, fixed32, uint64, or fixed64
        /// index field.
        #[prost(uint64, tag = "1")]
        Uint(u64),
        /// int64 specifies a value for an int32, sfixed32, int64, or sfixed64
        /// index field.
        #[prost(int64, tag = "2")]
        Int(i64),
        /// str specifies a value for a string index field.
        #[prost(string, tag = "3")]
        Str(::prost::alloc::string::String),
        /// bytes specifies a value for a bytes index field.
        #[prost(bytes, tag = "4")]
        Bytes(::prost::alloc::vec::Vec<u8>),
        /// enum specifies a value for an enum index field.
        #[prost(string, tag = "5")]
        Enum(::prost::alloc::string::String),
        /// bool specifies a value for a bool index field.
        #[prost(bool, tag = "6")]
        Bool(bool),
        /// timestamp specifies a value for a timestamp index field.
        #[prost(message, tag = "7")]
        Timestamp(super::super::super::super::super::google::protobuf::Timestamp),
        /// duration specifies a value for a duration index field.
        #[prost(message, tag = "8")]
        Duration(super::super::super::super::super::google::protobuf::Duration),
    }
}
impl ::prost::Name for IndexValue {
    const NAME: &'static str = "IndexValue";
    const PACKAGE: &'static str = "cosmos.orm.query.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.orm.query.v1alpha1.{}", Self::NAME)
    }
}
include!("cosmos.orm.query.v1alpha1.tonic.rs");
// @@protoc_insertion_point(module)

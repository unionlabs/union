/// InterfaceDescriptor describes an interface type to be used with
/// accepts_interface and implements_interface and declared by declare_interface.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InterfaceDescriptor {
    /// name is the name of the interface. It should be a short-name (without
    /// a period) such that the fully qualified name of the interface will be
    /// package.name, ex. for the package a.b and interface named C, the
    /// fully-qualified name will be a.b.C.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// description is a human-readable description of the interface and its
    /// purpose.
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
}
/// ScalarDescriptor describes an scalar type to be used with
/// the scalar field option and declared by declare_scalar.
/// Scalars extend simple protobuf built-in types with additional
/// syntax and semantics, for instance to represent big integers.
/// Scalars should ideally define an encoding such that there is only one
/// valid syntactical representation for a given semantic meaning,
/// i.e. the encoding should be deterministic.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ScalarDescriptor {
    /// name is the name of the scalar. It should be a short-name (without
    /// a period) such that the fully qualified name of the scalar will be
    /// package.name, ex. for the package a.b and scalar named C, the
    /// fully-qualified name will be a.b.C.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// description is a human-readable description of the scalar and its
    /// encoding format. For instance a big integer or decimal scalar should
    /// specify precisely the expected encoding format.
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// field_type is the type of field with which this scalar can be used.
    /// Scalars can be used with one and only one type of field so that
    /// encoding standards and simple and clear. Currently only string and
    /// bytes fields are supported for scalars.
    #[prost(enumeration = "ScalarType", repeated, tag = "3")]
    pub field_type: ::prost::alloc::vec::Vec<i32>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ScalarType {
    Unspecified = 0,
    String = 1,
    Bytes = 2,
}
impl ScalarType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ScalarType::Unspecified => "SCALAR_TYPE_UNSPECIFIED",
            ScalarType::String => "SCALAR_TYPE_STRING",
            ScalarType::Bytes => "SCALAR_TYPE_BYTES",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "SCALAR_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
            "SCALAR_TYPE_STRING" => Some(Self::String),
            "SCALAR_TYPE_BYTES" => Some(Self::Bytes),
            _ => None,
        }
    }
}

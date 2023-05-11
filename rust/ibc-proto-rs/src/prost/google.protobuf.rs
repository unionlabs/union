/// The protocol compiler can output a FileDescriptorSet containing the .proto
/// files it parses.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FileDescriptorSet {
    #[prost(message, repeated, tag = "1")]
    pub file: ::prost::alloc::vec::Vec<FileDescriptorProto>,
}
/// Describes a complete .proto file.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FileDescriptorProto {
    /// file name, relative to root of source tree
    #[prost(string, optional, tag = "1")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    /// e.g. "foo", "foo.bar", etc.
    #[prost(string, optional, tag = "2")]
    pub package: ::core::option::Option<::prost::alloc::string::String>,
    /// Names of files imported by this file.
    #[prost(string, repeated, tag = "3")]
    pub dependency: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Indexes of the public imported files in the dependency list above.
    #[prost(int32, repeated, packed = "false", tag = "10")]
    pub public_dependency: ::prost::alloc::vec::Vec<i32>,
    /// Indexes of the weak imported files in the dependency list.
    /// For Google-internal migration only. Do not use.
    #[prost(int32, repeated, packed = "false", tag = "11")]
    pub weak_dependency: ::prost::alloc::vec::Vec<i32>,
    /// All top-level definitions in this file.
    #[prost(message, repeated, tag = "4")]
    pub message_type: ::prost::alloc::vec::Vec<DescriptorProto>,
    #[prost(message, repeated, tag = "5")]
    pub enum_type: ::prost::alloc::vec::Vec<EnumDescriptorProto>,
    #[prost(message, repeated, tag = "6")]
    pub service: ::prost::alloc::vec::Vec<ServiceDescriptorProto>,
    #[prost(message, repeated, tag = "7")]
    pub extension: ::prost::alloc::vec::Vec<FieldDescriptorProto>,
    #[prost(message, optional, tag = "8")]
    pub options: ::core::option::Option<FileOptions>,
    /// This field contains optional information about the original source code.
    /// You may safely remove this entire field without harming runtime
    /// functionality of the descriptors -- the information is needed only by
    /// development tools.
    #[prost(message, optional, tag = "9")]
    pub source_code_info: ::core::option::Option<SourceCodeInfo>,
    /// The syntax of the proto file.
    /// The supported values are "proto2" and "proto3".
    #[prost(string, optional, tag = "12")]
    pub syntax: ::core::option::Option<::prost::alloc::string::String>,
}
/// Describes a message type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DescriptorProto {
    #[prost(string, optional, tag = "1")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "2")]
    pub field: ::prost::alloc::vec::Vec<FieldDescriptorProto>,
    #[prost(message, repeated, tag = "6")]
    pub extension: ::prost::alloc::vec::Vec<FieldDescriptorProto>,
    #[prost(message, repeated, tag = "3")]
    pub nested_type: ::prost::alloc::vec::Vec<DescriptorProto>,
    #[prost(message, repeated, tag = "4")]
    pub enum_type: ::prost::alloc::vec::Vec<EnumDescriptorProto>,
    #[prost(message, repeated, tag = "5")]
    pub extension_range: ::prost::alloc::vec::Vec<descriptor_proto::ExtensionRange>,
    #[prost(message, repeated, tag = "8")]
    pub oneof_decl: ::prost::alloc::vec::Vec<OneofDescriptorProto>,
    #[prost(message, optional, tag = "7")]
    pub options: ::core::option::Option<MessageOptions>,
    #[prost(message, repeated, tag = "9")]
    pub reserved_range: ::prost::alloc::vec::Vec<descriptor_proto::ReservedRange>,
    /// Reserved field names, which may not be used by fields in the same message.
    /// A given name may only be reserved once.
    #[prost(string, repeated, tag = "10")]
    pub reserved_name: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// Nested message and enum types in `DescriptorProto`.
pub mod descriptor_proto {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ExtensionRange {
        /// Inclusive.
        #[prost(int32, optional, tag = "1")]
        pub start: ::core::option::Option<i32>,
        /// Exclusive.
        #[prost(int32, optional, tag = "2")]
        pub end: ::core::option::Option<i32>,
        #[prost(message, optional, tag = "3")]
        pub options: ::core::option::Option<super::ExtensionRangeOptions>,
    }
    /// Range of reserved tag numbers. Reserved tag numbers may not be used by
    /// fields or extension ranges in the same message. Reserved ranges may
    /// not overlap.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ReservedRange {
        /// Inclusive.
        #[prost(int32, optional, tag = "1")]
        pub start: ::core::option::Option<i32>,
        /// Exclusive.
        #[prost(int32, optional, tag = "2")]
        pub end: ::core::option::Option<i32>,
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExtensionRangeOptions {
    /// The parser stores options it doesn't recognize here. See above.
    #[prost(message, repeated, tag = "999")]
    pub uninterpreted_option: ::prost::alloc::vec::Vec<UninterpretedOption>,
}
/// Describes a field within a message.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FieldDescriptorProto {
    #[prost(string, optional, tag = "1")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(int32, optional, tag = "3")]
    pub number: ::core::option::Option<i32>,
    #[prost(enumeration = "field_descriptor_proto::Label", optional, tag = "4")]
    pub label: ::core::option::Option<i32>,
    /// If type_name is set, this need not be set.  If both this and type_name
    /// are set, this must be one of TYPE_ENUM, TYPE_MESSAGE or TYPE_GROUP.
    #[prost(enumeration = "field_descriptor_proto::Type", optional, tag = "5")]
    pub r#type: ::core::option::Option<i32>,
    /// For message and enum types, this is the name of the type.  If the name
    /// starts with a '.', it is fully-qualified.  Otherwise, C++-like scoping
    /// rules are used to find the type (i.e. first the nested types within this
    /// message are searched, then within the parent, on up to the root
    /// namespace).
    #[prost(string, optional, tag = "6")]
    pub type_name: ::core::option::Option<::prost::alloc::string::String>,
    /// For extensions, this is the name of the type being extended.  It is
    /// resolved in the same manner as type_name.
    #[prost(string, optional, tag = "2")]
    pub extendee: ::core::option::Option<::prost::alloc::string::String>,
    /// For numeric types, contains the original text representation of the value.
    /// For booleans, "true" or "false".
    /// For strings, contains the default text contents (not escaped in any way).
    /// For bytes, contains the C escaped value.  All bytes >= 128 are escaped.
    #[prost(string, optional, tag = "7")]
    pub default_value: ::core::option::Option<::prost::alloc::string::String>,
    /// If set, gives the index of a oneof in the containing type's oneof_decl
    /// list.  This field is a member of that oneof.
    #[prost(int32, optional, tag = "9")]
    pub oneof_index: ::core::option::Option<i32>,
    /// JSON name of this field. The value is set by protocol compiler. If the
    /// user has set a "json_name" option on this field, that option's value
    /// will be used. Otherwise, it's deduced from the field's name by converting
    /// it to camelCase.
    #[prost(string, optional, tag = "10")]
    pub json_name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, optional, tag = "8")]
    pub options: ::core::option::Option<FieldOptions>,
    /// If true, this is a proto3 "optional". When a proto3 field is optional, it
    /// tracks presence regardless of field type.
    ///
    /// When proto3_optional is true, this field must be belong to a oneof to
    /// signal to old proto3 clients that presence is tracked for this field. This
    /// oneof is known as a "synthetic" oneof, and this field must be its sole
    /// member (each proto3 optional field gets its own synthetic oneof). Synthetic
    /// oneofs exist in the descriptor only, and do not generate any API. Synthetic
    /// oneofs must be ordered after all "real" oneofs.
    ///
    /// For message fields, proto3_optional doesn't create any semantic change,
    /// since non-repeated message fields always track presence. However it still
    /// indicates the semantic detail of whether the user wrote "optional" or not.
    /// This can be useful for round-tripping the .proto file. For consistency we
    /// give message fields a synthetic oneof also, even though it is not required
    /// to track presence. This is especially important because the parser can't
    /// tell if a field is a message or an enum, so it must always create a
    /// synthetic oneof.
    ///
    /// Proto2 optional fields do not set this flag, because they already indicate
    /// optional with `LABEL_OPTIONAL`.
    #[prost(bool, optional, tag = "17")]
    pub proto3_optional: ::core::option::Option<bool>,
}
/// Nested message and enum types in `FieldDescriptorProto`.
pub mod field_descriptor_proto {
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum Type {
        /// 0 is reserved for errors.
        /// Order is weird for historical reasons.
        Double = 1,
        Float = 2,
        /// Not ZigZag encoded.  Negative numbers take 10 bytes.  Use TYPE_SINT64 if
        /// negative values are likely.
        Int64 = 3,
        Uint64 = 4,
        /// Not ZigZag encoded.  Negative numbers take 10 bytes.  Use TYPE_SINT32 if
        /// negative values are likely.
        Int32 = 5,
        Fixed64 = 6,
        Fixed32 = 7,
        Bool = 8,
        String = 9,
        /// Tag-delimited aggregate.
        /// Group type is deprecated and not supported in proto3. However, Proto3
        /// implementations should still be able to parse the group wire format and
        /// treat group fields as unknown fields.
        Group = 10,
        /// Length-delimited aggregate.
        Message = 11,
        /// New in version 2.
        Bytes = 12,
        Uint32 = 13,
        Enum = 14,
        Sfixed32 = 15,
        Sfixed64 = 16,
        /// Uses ZigZag encoding.
        Sint32 = 17,
        /// Uses ZigZag encoding.
        Sint64 = 18,
    }
    impl Type {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Type::Double => "TYPE_DOUBLE",
                Type::Float => "TYPE_FLOAT",
                Type::Int64 => "TYPE_INT64",
                Type::Uint64 => "TYPE_UINT64",
                Type::Int32 => "TYPE_INT32",
                Type::Fixed64 => "TYPE_FIXED64",
                Type::Fixed32 => "TYPE_FIXED32",
                Type::Bool => "TYPE_BOOL",
                Type::String => "TYPE_STRING",
                Type::Group => "TYPE_GROUP",
                Type::Message => "TYPE_MESSAGE",
                Type::Bytes => "TYPE_BYTES",
                Type::Uint32 => "TYPE_UINT32",
                Type::Enum => "TYPE_ENUM",
                Type::Sfixed32 => "TYPE_SFIXED32",
                Type::Sfixed64 => "TYPE_SFIXED64",
                Type::Sint32 => "TYPE_SINT32",
                Type::Sint64 => "TYPE_SINT64",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "TYPE_DOUBLE" => Some(Self::Double),
                "TYPE_FLOAT" => Some(Self::Float),
                "TYPE_INT64" => Some(Self::Int64),
                "TYPE_UINT64" => Some(Self::Uint64),
                "TYPE_INT32" => Some(Self::Int32),
                "TYPE_FIXED64" => Some(Self::Fixed64),
                "TYPE_FIXED32" => Some(Self::Fixed32),
                "TYPE_BOOL" => Some(Self::Bool),
                "TYPE_STRING" => Some(Self::String),
                "TYPE_GROUP" => Some(Self::Group),
                "TYPE_MESSAGE" => Some(Self::Message),
                "TYPE_BYTES" => Some(Self::Bytes),
                "TYPE_UINT32" => Some(Self::Uint32),
                "TYPE_ENUM" => Some(Self::Enum),
                "TYPE_SFIXED32" => Some(Self::Sfixed32),
                "TYPE_SFIXED64" => Some(Self::Sfixed64),
                "TYPE_SINT32" => Some(Self::Sint32),
                "TYPE_SINT64" => Some(Self::Sint64),
                _ => None,
            }
        }
    }
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum Label {
        /// 0 is reserved for errors
        Optional = 1,
        Required = 2,
        Repeated = 3,
    }
    impl Label {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Label::Optional => "LABEL_OPTIONAL",
                Label::Required => "LABEL_REQUIRED",
                Label::Repeated => "LABEL_REPEATED",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "LABEL_OPTIONAL" => Some(Self::Optional),
                "LABEL_REQUIRED" => Some(Self::Required),
                "LABEL_REPEATED" => Some(Self::Repeated),
                _ => None,
            }
        }
    }
}
/// Describes a oneof.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OneofDescriptorProto {
    #[prost(string, optional, tag = "1")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, optional, tag = "2")]
    pub options: ::core::option::Option<OneofOptions>,
}
/// Describes an enum type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EnumDescriptorProto {
    #[prost(string, optional, tag = "1")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "2")]
    pub value: ::prost::alloc::vec::Vec<EnumValueDescriptorProto>,
    #[prost(message, optional, tag = "3")]
    pub options: ::core::option::Option<EnumOptions>,
    /// Range of reserved numeric values. Reserved numeric values may not be used
    /// by enum values in the same enum declaration. Reserved ranges may not
    /// overlap.
    #[prost(message, repeated, tag = "4")]
    pub reserved_range: ::prost::alloc::vec::Vec<
        enum_descriptor_proto::EnumReservedRange,
    >,
    /// Reserved enum value names, which may not be reused. A given name may only
    /// be reserved once.
    #[prost(string, repeated, tag = "5")]
    pub reserved_name: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// Nested message and enum types in `EnumDescriptorProto`.
pub mod enum_descriptor_proto {
    /// Range of reserved numeric values. Reserved values may not be used by
    /// entries in the same enum. Reserved ranges may not overlap.
    ///
    /// Note that this is distinct from DescriptorProto.ReservedRange in that it
    /// is inclusive such that it can appropriately represent the entire int32
    /// domain.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct EnumReservedRange {
        /// Inclusive.
        #[prost(int32, optional, tag = "1")]
        pub start: ::core::option::Option<i32>,
        /// Inclusive.
        #[prost(int32, optional, tag = "2")]
        pub end: ::core::option::Option<i32>,
    }
}
/// Describes a value within an enum.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EnumValueDescriptorProto {
    #[prost(string, optional, tag = "1")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(int32, optional, tag = "2")]
    pub number: ::core::option::Option<i32>,
    #[prost(message, optional, tag = "3")]
    pub options: ::core::option::Option<EnumValueOptions>,
}
/// Describes a service.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ServiceDescriptorProto {
    #[prost(string, optional, tag = "1")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "2")]
    pub method: ::prost::alloc::vec::Vec<MethodDescriptorProto>,
    #[prost(message, optional, tag = "3")]
    pub options: ::core::option::Option<ServiceOptions>,
}
/// Describes a method of a service.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MethodDescriptorProto {
    #[prost(string, optional, tag = "1")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    /// Input and output type names.  These are resolved in the same way as
    /// FieldDescriptorProto.type_name, but must refer to a message type.
    #[prost(string, optional, tag = "2")]
    pub input_type: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub output_type: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, optional, tag = "4")]
    pub options: ::core::option::Option<MethodOptions>,
    /// Identifies if client streams multiple client messages
    #[prost(bool, optional, tag = "5", default = "false")]
    pub client_streaming: ::core::option::Option<bool>,
    /// Identifies if server streams multiple server messages
    #[prost(bool, optional, tag = "6", default = "false")]
    pub server_streaming: ::core::option::Option<bool>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FileOptions {
    /// Sets the Java package where classes generated from this .proto will be
    /// placed.  By default, the proto package is used, but this is often
    /// inappropriate because proto packages do not normally start with backwards
    /// domain names.
    #[prost(string, optional, tag = "1")]
    pub java_package: ::core::option::Option<::prost::alloc::string::String>,
    /// Controls the name of the wrapper Java class generated for the .proto file.
    /// That class will always contain the .proto file's getDescriptor() method as
    /// well as any top-level extensions defined in the .proto file.
    /// If java_multiple_files is disabled, then all the other classes from the
    /// .proto file will be nested inside the single wrapper outer class.
    #[prost(string, optional, tag = "8")]
    pub java_outer_classname: ::core::option::Option<::prost::alloc::string::String>,
    /// If enabled, then the Java code generator will generate a separate .java
    /// file for each top-level message, enum, and service defined in the .proto
    /// file.  Thus, these types will *not* be nested inside the wrapper class
    /// named by java_outer_classname.  However, the wrapper class will still be
    /// generated to contain the file's getDescriptor() method as well as any
    /// top-level extensions defined in the file.
    #[prost(bool, optional, tag = "10", default = "false")]
    pub java_multiple_files: ::core::option::Option<bool>,
    /// This option does nothing.
    #[deprecated]
    #[prost(bool, optional, tag = "20")]
    pub java_generate_equals_and_hash: ::core::option::Option<bool>,
    /// If set true, then the Java2 code generator will generate code that
    /// throws an exception whenever an attempt is made to assign a non-UTF-8
    /// byte sequence to a string field.
    /// Message reflection will do the same.
    /// However, an extension field still accepts non-UTF-8 byte sequences.
    /// This option has no effect on when used with the lite runtime.
    #[prost(bool, optional, tag = "27", default = "false")]
    pub java_string_check_utf8: ::core::option::Option<bool>,
    #[prost(
        enumeration = "file_options::OptimizeMode",
        optional,
        tag = "9",
        default = "Speed"
    )]
    pub optimize_for: ::core::option::Option<i32>,
    /// Sets the Go package where structs generated from this .proto will be
    /// placed. If omitted, the Go package will be derived from the following:
    ///    - The basename of the package import path, if provided.
    ///    - Otherwise, the package statement in the .proto file, if present.
    ///    - Otherwise, the basename of the .proto file, without extension.
    #[prost(string, optional, tag = "11")]
    pub go_package: ::core::option::Option<::prost::alloc::string::String>,
    /// Should generic services be generated in each language?  "Generic" services
    /// are not specific to any particular RPC system.  They are generated by the
    /// main code generators in each language (without additional plugins).
    /// Generic services were the only kind of service generation supported by
    /// early versions of google.protobuf.
    ///
    /// Generic services are now considered deprecated in favor of using plugins
    /// that generate code specific to your particular RPC system.  Therefore,
    /// these default to false.  Old code which depends on generic services should
    /// explicitly set them to true.
    #[prost(bool, optional, tag = "16", default = "false")]
    pub cc_generic_services: ::core::option::Option<bool>,
    #[prost(bool, optional, tag = "17", default = "false")]
    pub java_generic_services: ::core::option::Option<bool>,
    #[prost(bool, optional, tag = "18", default = "false")]
    pub py_generic_services: ::core::option::Option<bool>,
    #[prost(bool, optional, tag = "42", default = "false")]
    pub php_generic_services: ::core::option::Option<bool>,
    /// Is this file deprecated?
    /// Depending on the target platform, this can emit Deprecated annotations
    /// for everything in the file, or it will be completely ignored; in the very
    /// least, this is a formalization for deprecating files.
    #[prost(bool, optional, tag = "23", default = "false")]
    pub deprecated: ::core::option::Option<bool>,
    /// Enables the use of arenas for the proto messages in this file. This applies
    /// only to generated classes for C++.
    #[prost(bool, optional, tag = "31", default = "true")]
    pub cc_enable_arenas: ::core::option::Option<bool>,
    /// Sets the objective c class prefix which is prepended to all objective c
    /// generated classes from this .proto. There is no default.
    #[prost(string, optional, tag = "36")]
    pub objc_class_prefix: ::core::option::Option<::prost::alloc::string::String>,
    /// Namespace for generated classes; defaults to the package.
    #[prost(string, optional, tag = "37")]
    pub csharp_namespace: ::core::option::Option<::prost::alloc::string::String>,
    /// By default Swift generators will take the proto package and CamelCase it
    /// replacing '.' with underscore and use that to prefix the types/symbols
    /// defined. When this options is provided, they will use this value instead
    /// to prefix the types/symbols defined.
    #[prost(string, optional, tag = "39")]
    pub swift_prefix: ::core::option::Option<::prost::alloc::string::String>,
    /// Sets the php class prefix which is prepended to all php generated classes
    /// from this .proto. Default is empty.
    #[prost(string, optional, tag = "40")]
    pub php_class_prefix: ::core::option::Option<::prost::alloc::string::String>,
    /// Use this option to change the namespace of php generated classes. Default
    /// is empty. When this option is empty, the package name will be used for
    /// determining the namespace.
    #[prost(string, optional, tag = "41")]
    pub php_namespace: ::core::option::Option<::prost::alloc::string::String>,
    /// Use this option to change the namespace of php generated metadata classes.
    /// Default is empty. When this option is empty, the proto file name will be
    /// used for determining the namespace.
    #[prost(string, optional, tag = "44")]
    pub php_metadata_namespace: ::core::option::Option<::prost::alloc::string::String>,
    /// Use this option to change the package of ruby generated classes. Default
    /// is empty. When this option is not set, the package name will be used for
    /// determining the ruby package.
    #[prost(string, optional, tag = "45")]
    pub ruby_package: ::core::option::Option<::prost::alloc::string::String>,
    /// The parser stores options it doesn't recognize here.
    /// See the documentation for the "Options" section above.
    #[prost(message, repeated, tag = "999")]
    pub uninterpreted_option: ::prost::alloc::vec::Vec<UninterpretedOption>,
}
/// Nested message and enum types in `FileOptions`.
pub mod file_options {
    /// Generated classes can be optimized for speed or code size.
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum OptimizeMode {
        /// Generate complete code for parsing, serialization,
        Speed = 1,
        /// etc.
        ///
        /// Use ReflectionOps to implement these methods.
        CodeSize = 2,
        /// Generate code using MessageLite and the lite runtime.
        LiteRuntime = 3,
    }
    impl OptimizeMode {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                OptimizeMode::Speed => "SPEED",
                OptimizeMode::CodeSize => "CODE_SIZE",
                OptimizeMode::LiteRuntime => "LITE_RUNTIME",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "SPEED" => Some(Self::Speed),
                "CODE_SIZE" => Some(Self::CodeSize),
                "LITE_RUNTIME" => Some(Self::LiteRuntime),
                _ => None,
            }
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MessageOptions {
    /// Set true to use the old proto1 MessageSet wire format for extensions.
    /// This is provided for backwards-compatibility with the MessageSet wire
    /// format.  You should not use this for any other reason:  It's less
    /// efficient, has fewer features, and is more complicated.
    ///
    /// The message must be defined exactly as follows:
    ///    message Foo {
    ///      option message_set_wire_format = true;
    ///      extensions 4 to max;
    ///    }
    /// Note that the message cannot have any defined fields; MessageSets only
    /// have extensions.
    ///
    /// All extensions of your type must be singular messages; e.g. they cannot
    /// be int32s, enums, or repeated messages.
    ///
    /// Because this is an option, the above two restrictions are not enforced by
    /// the protocol compiler.
    #[prost(bool, optional, tag = "1", default = "false")]
    pub message_set_wire_format: ::core::option::Option<bool>,
    /// Disables the generation of the standard "descriptor()" accessor, which can
    /// conflict with a field of the same name.  This is meant to make migration
    /// from proto1 easier; new code should avoid fields named "descriptor".
    #[prost(bool, optional, tag = "2", default = "false")]
    pub no_standard_descriptor_accessor: ::core::option::Option<bool>,
    /// Is this message deprecated?
    /// Depending on the target platform, this can emit Deprecated annotations
    /// for the message, or it will be completely ignored; in the very least,
    /// this is a formalization for deprecating messages.
    #[prost(bool, optional, tag = "3", default = "false")]
    pub deprecated: ::core::option::Option<bool>,
    /// Whether the message is an automatically generated map entry type for the
    /// maps field.
    ///
    /// For maps fields:
    ///      map<KeyType, ValueType> map_field = 1;
    /// The parsed descriptor looks like:
    ///      message MapFieldEntry {
    ///          option map_entry = true;
    ///          optional KeyType key = 1;
    ///          optional ValueType value = 2;
    ///      }
    ///      repeated MapFieldEntry map_field = 1;
    ///
    /// Implementations may choose not to generate the map_entry=true message, but
    /// use a native map in the target language to hold the keys and values.
    /// The reflection APIs in such implementations still need to work as
    /// if the field is a repeated message field.
    ///
    /// NOTE: Do not set the option in .proto files. Always use the maps syntax
    /// instead. The option should only be implicitly set by the proto compiler
    /// parser.
    #[prost(bool, optional, tag = "7")]
    pub map_entry: ::core::option::Option<bool>,
    /// The parser stores options it doesn't recognize here. See above.
    #[prost(message, repeated, tag = "999")]
    pub uninterpreted_option: ::prost::alloc::vec::Vec<UninterpretedOption>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FieldOptions {
    /// The ctype option instructs the C++ code generator to use a different
    /// representation of the field than it normally would.  See the specific
    /// options below.  This option is not yet implemented in the open source
    /// release -- sorry, we'll try to include it in a future version!
    #[prost(
        enumeration = "field_options::CType",
        optional,
        tag = "1",
        default = "String"
    )]
    pub ctype: ::core::option::Option<i32>,
    /// The packed option can be enabled for repeated primitive fields to enable
    /// a more efficient representation on the wire. Rather than repeatedly
    /// writing the tag and type for each element, the entire array is encoded as
    /// a single length-delimited blob. In proto3, only explicit setting it to
    /// false will avoid using packed encoding.
    #[prost(bool, optional, tag = "2")]
    pub packed: ::core::option::Option<bool>,
    /// The jstype option determines the JavaScript type used for values of the
    /// field.  The option is permitted only for 64 bit integral and fixed types
    /// (int64, uint64, sint64, fixed64, sfixed64).  A field with jstype JS_STRING
    /// is represented as JavaScript string, which avoids loss of precision that
    /// can happen when a large value is converted to a floating point JavaScript.
    /// Specifying JS_NUMBER for the jstype causes the generated JavaScript code to
    /// use the JavaScript "number" type.  The behavior of the default option
    /// JS_NORMAL is implementation dependent.
    ///
    /// This option is an enum to permit additional types to be added, e.g.
    /// goog.math.Integer.
    #[prost(
        enumeration = "field_options::JsType",
        optional,
        tag = "6",
        default = "JsNormal"
    )]
    pub jstype: ::core::option::Option<i32>,
    /// Should this field be parsed lazily?  Lazy applies only to message-type
    /// fields.  It means that when the outer message is initially parsed, the
    /// inner message's contents will not be parsed but instead stored in encoded
    /// form.  The inner message will actually be parsed when it is first accessed.
    ///
    /// This is only a hint.  Implementations are free to choose whether to use
    /// eager or lazy parsing regardless of the value of this option.  However,
    /// setting this option true suggests that the protocol author believes that
    /// using lazy parsing on this field is worth the additional bookkeeping
    /// overhead typically needed to implement it.
    ///
    /// This option does not affect the public interface of any generated code;
    /// all method signatures remain the same.  Furthermore, thread-safety of the
    /// interface is not affected by this option; const methods remain safe to
    /// call from multiple threads concurrently, while non-const methods continue
    /// to require exclusive access.
    ///
    ///
    /// Note that implementations may choose not to check required fields within
    /// a lazy sub-message.  That is, calling IsInitialized() on the outer message
    /// may return true even if the inner message has missing required fields.
    /// This is necessary because otherwise the inner message would have to be
    /// parsed in order to perform the check, defeating the purpose of lazy
    /// parsing.  An implementation which chooses not to check required fields
    /// must be consistent about it.  That is, for any particular sub-message, the
    /// implementation must either *always* check its required fields, or *never*
    /// check its required fields, regardless of whether or not the message has
    /// been parsed.
    ///
    /// As of 2021, lazy does no correctness checks on the byte stream during
    /// parsing.  This may lead to crashes if and when an invalid byte stream is
    /// finally parsed upon access.
    ///
    /// TODO(b/211906113):  Enable validation on lazy fields.
    #[prost(bool, optional, tag = "5", default = "false")]
    pub lazy: ::core::option::Option<bool>,
    /// unverified_lazy does no correctness checks on the byte stream. This should
    /// only be used where lazy with verification is prohibitive for performance
    /// reasons.
    #[prost(bool, optional, tag = "15", default = "false")]
    pub unverified_lazy: ::core::option::Option<bool>,
    /// Is this field deprecated?
    /// Depending on the target platform, this can emit Deprecated annotations
    /// for accessors, or it will be completely ignored; in the very least, this
    /// is a formalization for deprecating fields.
    #[prost(bool, optional, tag = "3", default = "false")]
    pub deprecated: ::core::option::Option<bool>,
    /// For Google-internal migration only. Do not use.
    #[prost(bool, optional, tag = "10", default = "false")]
    pub weak: ::core::option::Option<bool>,
    /// The parser stores options it doesn't recognize here. See above.
    #[prost(message, repeated, tag = "999")]
    pub uninterpreted_option: ::prost::alloc::vec::Vec<UninterpretedOption>,
}
/// Nested message and enum types in `FieldOptions`.
pub mod field_options {
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum CType {
        /// Default mode.
        String = 0,
        Cord = 1,
        StringPiece = 2,
    }
    impl CType {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                CType::String => "STRING",
                CType::Cord => "CORD",
                CType::StringPiece => "STRING_PIECE",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "STRING" => Some(Self::String),
                "CORD" => Some(Self::Cord),
                "STRING_PIECE" => Some(Self::StringPiece),
                _ => None,
            }
        }
    }
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum JsType {
        /// Use the default type.
        JsNormal = 0,
        /// Use JavaScript strings.
        JsString = 1,
        /// Use JavaScript numbers.
        JsNumber = 2,
    }
    impl JsType {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                JsType::JsNormal => "JS_NORMAL",
                JsType::JsString => "JS_STRING",
                JsType::JsNumber => "JS_NUMBER",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "JS_NORMAL" => Some(Self::JsNormal),
                "JS_STRING" => Some(Self::JsString),
                "JS_NUMBER" => Some(Self::JsNumber),
                _ => None,
            }
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OneofOptions {
    /// The parser stores options it doesn't recognize here. See above.
    #[prost(message, repeated, tag = "999")]
    pub uninterpreted_option: ::prost::alloc::vec::Vec<UninterpretedOption>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EnumOptions {
    /// Set this option to true to allow mapping different tag names to the same
    /// value.
    #[prost(bool, optional, tag = "2")]
    pub allow_alias: ::core::option::Option<bool>,
    /// Is this enum deprecated?
    /// Depending on the target platform, this can emit Deprecated annotations
    /// for the enum, or it will be completely ignored; in the very least, this
    /// is a formalization for deprecating enums.
    #[prost(bool, optional, tag = "3", default = "false")]
    pub deprecated: ::core::option::Option<bool>,
    /// The parser stores options it doesn't recognize here. See above.
    #[prost(message, repeated, tag = "999")]
    pub uninterpreted_option: ::prost::alloc::vec::Vec<UninterpretedOption>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EnumValueOptions {
    /// Is this enum value deprecated?
    /// Depending on the target platform, this can emit Deprecated annotations
    /// for the enum value, or it will be completely ignored; in the very least,
    /// this is a formalization for deprecating enum values.
    #[prost(bool, optional, tag = "1", default = "false")]
    pub deprecated: ::core::option::Option<bool>,
    /// The parser stores options it doesn't recognize here. See above.
    #[prost(message, repeated, tag = "999")]
    pub uninterpreted_option: ::prost::alloc::vec::Vec<UninterpretedOption>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ServiceOptions {
    /// Is this service deprecated?
    /// Depending on the target platform, this can emit Deprecated annotations
    /// for the service, or it will be completely ignored; in the very least,
    /// this is a formalization for deprecating services.
    #[prost(bool, optional, tag = "33", default = "false")]
    pub deprecated: ::core::option::Option<bool>,
    /// The parser stores options it doesn't recognize here. See above.
    #[prost(message, repeated, tag = "999")]
    pub uninterpreted_option: ::prost::alloc::vec::Vec<UninterpretedOption>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MethodOptions {
    /// Is this method deprecated?
    /// Depending on the target platform, this can emit Deprecated annotations
    /// for the method, or it will be completely ignored; in the very least,
    /// this is a formalization for deprecating methods.
    #[prost(bool, optional, tag = "33", default = "false")]
    pub deprecated: ::core::option::Option<bool>,
    #[prost(
        enumeration = "method_options::IdempotencyLevel",
        optional,
        tag = "34",
        default = "IdempotencyUnknown"
    )]
    pub idempotency_level: ::core::option::Option<i32>,
    /// The parser stores options it doesn't recognize here. See above.
    #[prost(message, repeated, tag = "999")]
    pub uninterpreted_option: ::prost::alloc::vec::Vec<UninterpretedOption>,
}
/// Nested message and enum types in `MethodOptions`.
pub mod method_options {
    /// Is this method side-effect-free (or safe in HTTP parlance), or idempotent,
    /// or neither? HTTP based RPC implementation may choose GET verb for safe
    /// methods, and PUT verb for idempotent methods instead of the default POST.
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum IdempotencyLevel {
        IdempotencyUnknown = 0,
        /// implies idempotent
        NoSideEffects = 1,
        /// idempotent, but may have side effects
        Idempotent = 2,
    }
    impl IdempotencyLevel {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                IdempotencyLevel::IdempotencyUnknown => "IDEMPOTENCY_UNKNOWN",
                IdempotencyLevel::NoSideEffects => "NO_SIDE_EFFECTS",
                IdempotencyLevel::Idempotent => "IDEMPOTENT",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "IDEMPOTENCY_UNKNOWN" => Some(Self::IdempotencyUnknown),
                "NO_SIDE_EFFECTS" => Some(Self::NoSideEffects),
                "IDEMPOTENT" => Some(Self::Idempotent),
                _ => None,
            }
        }
    }
}
/// A message representing a option the parser does not recognize. This only
/// appears in options protos created by the compiler::Parser class.
/// DescriptorPool resolves these when building Descriptor objects. Therefore,
/// options protos in descriptor objects (e.g. returned by Descriptor::options(),
/// or produced by Descriptor::CopyTo()) will never have UninterpretedOptions
/// in them.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UninterpretedOption {
    #[prost(message, repeated, tag = "2")]
    pub name: ::prost::alloc::vec::Vec<uninterpreted_option::NamePart>,
    /// The value of the uninterpreted option, in whatever type the tokenizer
    /// identified it as during parsing. Exactly one of these should be set.
    #[prost(string, optional, tag = "3")]
    pub identifier_value: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(uint64, optional, tag = "4")]
    pub positive_int_value: ::core::option::Option<u64>,
    #[prost(int64, optional, tag = "5")]
    pub negative_int_value: ::core::option::Option<i64>,
    #[prost(double, optional, tag = "6")]
    pub double_value: ::core::option::Option<f64>,
    #[prost(bytes = "vec", optional, tag = "7")]
    pub string_value: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
    #[prost(string, optional, tag = "8")]
    pub aggregate_value: ::core::option::Option<::prost::alloc::string::String>,
}
/// Nested message and enum types in `UninterpretedOption`.
pub mod uninterpreted_option {
    /// The name of the uninterpreted option.  Each string represents a segment in
    /// a dot-separated name.  is_extension is true iff a segment represents an
    /// extension (denoted with parentheses in options specs in .proto files).
    /// E.g.,{ ["foo", false], ["bar.baz", true], ["moo", false] } represents
    /// "foo.(bar.baz).moo".
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct NamePart {
        #[prost(string, required, tag = "1")]
        pub name_part: ::prost::alloc::string::String,
        #[prost(bool, required, tag = "2")]
        pub is_extension: bool,
    }
}
/// Encapsulates information about the original source file from which a
/// FileDescriptorProto was generated.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SourceCodeInfo {
    /// A Location identifies a piece of source code in a .proto file which
    /// corresponds to a particular definition.  This information is intended
    /// to be useful to IDEs, code indexers, documentation generators, and similar
    /// tools.
    ///
    /// For example, say we have a file like:
    ///    message Foo {
    ///      optional string foo = 1;
    ///    }
    /// Let's look at just the field definition:
    ///    optional string foo = 1;
    ///    ^       ^^     ^^  ^  ^^^
    ///    a       bc     de  f  ghi
    /// We have the following locations:
    ///    span   path               represents
    ///    [a,i)  [ 4, 0, 2, 0 ]     The whole field definition.
    ///    [a,b)  [ 4, 0, 2, 0, 4 ]  The label (optional).
    ///    [c,d)  [ 4, 0, 2, 0, 5 ]  The type (string).
    ///    [e,f)  [ 4, 0, 2, 0, 1 ]  The name (foo).
    ///    [g,h)  [ 4, 0, 2, 0, 3 ]  The number (1).
    ///
    /// Notes:
    /// - A location may refer to a repeated field itself (i.e. not to any
    ///    particular index within it).  This is used whenever a set of elements are
    ///    logically enclosed in a single code segment.  For example, an entire
    ///    extend block (possibly containing multiple extension definitions) will
    ///    have an outer location whose path refers to the "extensions" repeated
    ///    field without an index.
    /// - Multiple locations may have the same path.  This happens when a single
    ///    logical declaration is spread out across multiple places.  The most
    ///    obvious example is the "extend" block again -- there may be multiple
    ///    extend blocks in the same scope, each of which will have the same path.
    /// - A location's span is not always a subset of its parent's span.  For
    ///    example, the "extendee" of an extension declaration appears at the
    ///    beginning of the "extend" block and is shared by all extensions within
    ///    the block.
    /// - Just because a location's span is a subset of some other location's span
    ///    does not mean that it is a descendant.  For example, a "group" defines
    ///    both a type and a field in a single declaration.  Thus, the locations
    ///    corresponding to the type and field and their components will overlap.
    /// - Code which tries to interpret locations should probably be designed to
    ///    ignore those that it doesn't understand, as more types of locations could
    ///    be recorded in the future.
    #[prost(message, repeated, tag = "1")]
    pub location: ::prost::alloc::vec::Vec<source_code_info::Location>,
}
/// Nested message and enum types in `SourceCodeInfo`.
pub mod source_code_info {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Location {
        /// Identifies which part of the FileDescriptorProto was defined at this
        /// location.
        ///
        /// Each element is a field number or an index.  They form a path from
        /// the root FileDescriptorProto to the place where the definition occurs.
        /// For example, this path:
        ///    [ 4, 3, 2, 7, 1 ]
        /// refers to:
        ///    file.message_type(3)  // 4, 3
        ///        .field(7)         // 2, 7
        ///        .name()           // 1
        /// This is because FileDescriptorProto.message_type has field number 4:
        ///    repeated DescriptorProto message_type = 4;
        /// and DescriptorProto.field has field number 2:
        ///    repeated FieldDescriptorProto field = 2;
        /// and FieldDescriptorProto.name has field number 1:
        ///    optional string name = 1;
        ///
        /// Thus, the above path gives the location of a field name.  If we removed
        /// the last element:
        ///    [ 4, 3, 2, 7 ]
        /// this path refers to the whole field declaration (from the beginning
        /// of the label to the terminating semicolon).
        #[prost(int32, repeated, tag = "1")]
        pub path: ::prost::alloc::vec::Vec<i32>,
        /// Always has exactly three or four elements: start line, start column,
        /// end line (optional, otherwise assumed same as start line), end column.
        /// These are packed into a single field for efficiency.  Note that line
        /// and column numbers are zero-based -- typically you will want to add
        /// 1 to each before displaying to a user.
        #[prost(int32, repeated, tag = "2")]
        pub span: ::prost::alloc::vec::Vec<i32>,
        /// If this SourceCodeInfo represents a complete declaration, these are any
        /// comments appearing before and after the declaration which appear to be
        /// attached to the declaration.
        ///
        /// A series of line comments appearing on consecutive lines, with no other
        /// tokens appearing on those lines, will be treated as a single comment.
        ///
        /// leading_detached_comments will keep paragraphs of comments that appear
        /// before (but not connected to) the current element. Each paragraph,
        /// separated by empty lines, will be one comment element in the repeated
        /// field.
        ///
        /// Only the comment content is provided; comment markers (e.g. //) are
        /// stripped out.  For block comments, leading whitespace and an asterisk
        /// will be stripped from the beginning of each line other than the first.
        /// Newlines are included in the output.
        ///
        /// Examples:
        ///
        ///    optional int32 foo = 1;  // Comment attached to foo.
        ///    // Comment attached to bar.
        ///    optional int32 bar = 2;
        ///
        ///    optional string baz = 3;
        ///    // Comment attached to baz.
        ///    // Another line attached to baz.
        ///
        ///    // Comment attached to moo.
        ///    //
        ///    // Another line attached to moo.
        ///    optional double moo = 4;
        ///
        ///    // Detached comment for corge. This is not leading or trailing comments
        ///    // to moo or corge because there are blank lines separating it from
        ///    // both.
        ///
        ///    // Detached comment for corge paragraph 2.
        ///
        ///    optional string corge = 5;
        ///    /* Block comment attached
        ///     * to corge.  Leading asterisks
        ///     * will be removed. */
        ///    /* Block comment attached to
        ///     * grault. */
        ///    optional int32 grault = 6;
        ///
        ///    // ignored detached comments.
        #[prost(string, optional, tag = "3")]
        pub leading_comments: ::core::option::Option<::prost::alloc::string::String>,
        #[prost(string, optional, tag = "4")]
        pub trailing_comments: ::core::option::Option<::prost::alloc::string::String>,
        #[prost(string, repeated, tag = "6")]
        pub leading_detached_comments: ::prost::alloc::vec::Vec<
            ::prost::alloc::string::String,
        >,
    }
}
/// Describes the relationship between generated code and its original source
/// file. A GeneratedCodeInfo message is associated with only one generated
/// source file, but may contain references to different source .proto files.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GeneratedCodeInfo {
    /// An Annotation connects some span of text in generated code to an element
    /// of its generating .proto file.
    #[prost(message, repeated, tag = "1")]
    pub annotation: ::prost::alloc::vec::Vec<generated_code_info::Annotation>,
}
/// Nested message and enum types in `GeneratedCodeInfo`.
pub mod generated_code_info {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Annotation {
        /// Identifies the element in the original source .proto file. This field
        /// is formatted the same as SourceCodeInfo.Location.path.
        #[prost(int32, repeated, tag = "1")]
        pub path: ::prost::alloc::vec::Vec<i32>,
        /// Identifies the filesystem path to the original source .proto.
        #[prost(string, optional, tag = "2")]
        pub source_file: ::core::option::Option<::prost::alloc::string::String>,
        /// Identifies the starting offset in bytes in the generated code
        /// that relates to the identified object.
        #[prost(int32, optional, tag = "3")]
        pub begin: ::core::option::Option<i32>,
        /// Identifies the ending offset in bytes in the generated code that
        /// relates to the identified offset. The end offset should be one past
        /// the last relevant byte (so the length of the text = end - begin).
        #[prost(int32, optional, tag = "4")]
        pub end: ::core::option::Option<i32>,
    }
}
/// `Any` contains an arbitrary serialized protocol buffer message along with a
/// URL that describes the type of the serialized message.
///
/// Protobuf library provides support to pack/unpack Any values in the form
/// of utility functions or additional generated methods of the Any type.
///
/// Example 1: Pack and unpack a message in C++.
///
///      Foo foo = ...;
///      Any any;
///      any.PackFrom(foo);
///      ...
///      if (any.UnpackTo(&foo)) {
///        ...
///      }
///
/// Example 2: Pack and unpack a message in Java.
///
///      Foo foo = ...;
///      Any any = Any.pack(foo);
///      ...
///      if (any.is(Foo.class)) {
///        foo = any.unpack(Foo.class);
///      }
///
/// Example 3: Pack and unpack a message in Python.
///
///      foo = Foo(...)
///      any = Any()
///      any.Pack(foo)
///      ...
///      if any.Is(Foo.DESCRIPTOR):
///        any.Unpack(foo)
///        ...
///
/// Example 4: Pack and unpack a message in Go
///
///       foo := &pb.Foo{...}
///       any, err := anypb.New(foo)
///       if err != nil {
///         ...
///       }
///       ...
///       foo := &pb.Foo{}
///       if err := any.UnmarshalTo(foo); err != nil {
///         ...
///       }
///
/// The pack methods provided by protobuf library will by default use
/// 'type.googleapis.com/full.type.name' as the type URL and the unpack
/// methods only use the fully qualified type name after the last '/'
/// in the type URL, for example "foo.bar.com/x/y.z" will yield type
/// name "y.z".
///
///
/// JSON
///
/// The JSON representation of an `Any` value uses the regular
/// representation of the deserialized, embedded message, with an
/// additional field `@type` which contains the type URL. Example:
///
///      package google.profile;
///      message Person {
///        string first_name = 1;
///        string last_name = 2;
///      }
///
///      {
///        "@type": "type.googleapis.com/google.profile.Person",
///        "firstName": <string>,
///        "lastName": <string>
///      }
///
/// If the embedded message type is well-known and has a custom JSON
/// representation, that representation will be embedded adding a field
/// `value` which holds the custom JSON in addition to the `@type`
/// field. Example (for message \[google.protobuf.Duration][\]):
///
///      {
///        "@type": "type.googleapis.com/google.protobuf.Duration",
///        "value": "1.212s"
///      }
///
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Eq)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Any {
    /// A URL/resource name that uniquely identifies the type of the serialized
    /// protocol buffer message. This string must contain at least
    /// one "/" character. The last segment of the URL's path must represent
    /// the fully qualified name of the type (as in
    /// `path/google.protobuf.Duration`). The name should be in a canonical form
    /// (e.g., leading "." is not accepted).
    ///
    /// In practice, teams usually precompile into the binary all types that they
    /// expect it to use in the context of Any. However, for URLs which use the
    /// scheme `http`, `https`, or no scheme, one can optionally set up a type
    /// server that maps type URLs to message definitions as follows:
    ///
    /// * If no scheme is provided, `https` is assumed.
    /// * An HTTP GET on the URL must yield a \[google.protobuf.Type][\]
    ///    value in binary format, or produce an error.
    /// * Applications are allowed to cache lookup results based on the
    ///    URL, or have them precompiled into a binary to avoid any
    ///    lookup. Therefore, binary compatibility needs to be preserved
    ///    on changes to types. (Use versioned type names to manage
    ///    breaking changes.)
    ///
    /// Note: this functionality is not currently available in the official
    /// protobuf release, and it is not used for type URLs beginning with
    /// type.googleapis.com.
    ///
    /// Schemes other than `http`, `https` (or the empty scheme) might be
    /// used with implementation specific semantics.
    ///
    #[prost(string, tag = "1")]
    pub type_url: ::prost::alloc::string::String,
    /// Must be a valid serialized protocol buffer of the above specified type.
    #[prost(bytes = "vec", tag = "2")]
    pub value: ::prost::alloc::vec::Vec<u8>,
}
/// A Timestamp represents a point in time independent of any time zone or local
/// calendar, encoded as a count of seconds and fractions of seconds at
/// nanosecond resolution. The count is relative to an epoch at UTC midnight on
/// January 1, 1970, in the proleptic Gregorian calendar which extends the
/// Gregorian calendar backwards to year one.
///
/// All minutes are 60 seconds long. Leap seconds are "smeared" so that no leap
/// second table is needed for interpretation, using a [24-hour linear
/// smear](<https://developers.google.com/time/smear>).
///
/// The range is from 0001-01-01T00:00:00Z to 9999-12-31T23:59:59.999999999Z. By
/// restricting to that range, we ensure that we can convert to and from [RFC
/// 3339](<https://www.ietf.org/rfc/rfc3339.txt>) date strings.
///
/// # Examples
///
/// Example 1: Compute Timestamp from POSIX `time()`.
///
///      Timestamp timestamp;
///      timestamp.set_seconds(time(NULL));
///      timestamp.set_nanos(0);
///
/// Example 2: Compute Timestamp from POSIX `gettimeofday()`.
///
///      struct timeval tv;
///      gettimeofday(&tv, NULL);
///
///      Timestamp timestamp;
///      timestamp.set_seconds(tv.tv_sec);
///      timestamp.set_nanos(tv.tv_usec * 1000);
///
/// Example 3: Compute Timestamp from Win32 `GetSystemTimeAsFileTime()`.
///
///      FILETIME ft;
///      GetSystemTimeAsFileTime(&ft);
///      UINT64 ticks = (((UINT64)ft.dwHighDateTime) << 32) | ft.dwLowDateTime;
///
///      // A Windows tick is 100 nanoseconds. Windows epoch 1601-01-01T00:00:00Z
///      // is 11644473600 seconds before Unix epoch 1970-01-01T00:00:00Z.
///      Timestamp timestamp;
///      timestamp.set_seconds((INT64) ((ticks / 10000000) - 11644473600LL));
///      timestamp.set_nanos((INT32) ((ticks % 10000000) * 100));
///
/// Example 4: Compute Timestamp from Java `System.currentTimeMillis()`.
///
///      long millis = System.currentTimeMillis();
///
///      Timestamp timestamp = Timestamp.newBuilder().setSeconds(millis / 1000)
///          .setNanos((int) ((millis % 1000) * 1000000)).build();
///
///
/// Example 5: Compute Timestamp from Java `Instant.now()`.
///
///      Instant now = Instant.now();
///
///      Timestamp timestamp =
///          Timestamp.newBuilder().setSeconds(now.getEpochSecond())
///              .setNanos(now.getNano()).build();
///
///
/// Example 6: Compute Timestamp from current time in Python.
///
///      timestamp = Timestamp()
///      timestamp.GetCurrentTime()
///
/// # JSON Mapping
///
/// In JSON format, the Timestamp type is encoded as a string in the
/// [RFC 3339](<https://www.ietf.org/rfc/rfc3339.txt>) format. That is, the
/// format is "{year}-{month}-{day}T{hour}:{min}:{sec}\[.{frac_sec}\]Z"
/// where {year} is always expressed using four digits while {month}, {day},
/// {hour}, {min}, and {sec} are zero-padded to two digits each. The fractional
/// seconds, which can go up to 9 digits (i.e. up to 1 nanosecond resolution),
/// are optional. The "Z" suffix indicates the timezone ("UTC"); the timezone
/// is required. A proto3 JSON serializer should always use UTC (as indicated by
/// "Z") when printing the Timestamp type and a proto3 JSON parser should be
/// able to accept both UTC and other timezones (as indicated by an offset).
///
/// For example, "2017-01-15T01:30:15.01Z" encodes 15.01 seconds past
/// 01:30 UTC on January 15, 2017.
///
/// In JavaScript, one can convert a Date object to this format using the
/// standard
/// \[toISOString()\](<https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toISOString>)
/// method. In Python, a standard `datetime.datetime` object can be converted
/// to this format using
/// \[`strftime`\](<https://docs.python.org/2/library/time.html#time.strftime>) with
/// the time format spec '%Y-%m-%dT%H:%M:%S.%fZ'. Likewise, in Java, one can use
/// the Joda Time's \[`ISODateTimeFormat.dateTime()`\](
/// <http://www.joda.org/joda-time/apidocs/org/joda/time/format/ISODateTimeFormat.html#dateTime%2D%2D>
/// ) to obtain a formatter capable of generating timestamps in this format.
///
///
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Timestamp {
    /// Represents seconds of UTC time since Unix epoch
    /// 1970-01-01T00:00:00Z. Must be from 0001-01-01T00:00:00Z to
    /// 9999-12-31T23:59:59Z inclusive.
    #[prost(int64, tag = "1")]
    pub seconds: i64,
    /// Non-negative fractions of a second at nanosecond resolution. Negative
    /// second values with fractions must still have non-negative nanos values
    /// that count forward in time. Must be from 0 to 999,999,999
    /// inclusive.
    #[prost(int32, tag = "2")]
    pub nanos: i32,
}
/// A Duration represents a signed, fixed-length span of time represented
/// as a count of seconds and fractions of seconds at nanosecond
/// resolution. It is independent of any calendar and concepts like "day"
/// or "month". It is related to Timestamp in that the difference between
/// two Timestamp values is a Duration and it can be added or subtracted
/// from a Timestamp. Range is approximately +-10,000 years.
///
/// # Examples
///
/// Example 1: Compute Duration from two Timestamps in pseudo code.
///
///      Timestamp start = ...;
///      Timestamp end = ...;
///      Duration duration = ...;
///
///      duration.seconds = end.seconds - start.seconds;
///      duration.nanos = end.nanos - start.nanos;
///
///      if (duration.seconds < 0 && duration.nanos > 0) {
///        duration.seconds += 1;
///        duration.nanos -= 1000000000;
///      } else if (duration.seconds > 0 && duration.nanos < 0) {
///        duration.seconds -= 1;
///        duration.nanos += 1000000000;
///      }
///
/// Example 2: Compute Timestamp from Timestamp + Duration in pseudo code.
///
///      Timestamp start = ...;
///      Duration duration = ...;
///      Timestamp end = ...;
///
///      end.seconds = start.seconds + duration.seconds;
///      end.nanos = start.nanos + duration.nanos;
///
///      if (end.nanos < 0) {
///        end.seconds -= 1;
///        end.nanos += 1000000000;
///      } else if (end.nanos >= 1000000000) {
///        end.seconds += 1;
///        end.nanos -= 1000000000;
///      }
///
/// Example 3: Compute Duration from datetime.timedelta in Python.
///
///      td = datetime.timedelta(days=3, minutes=10)
///      duration = Duration()
///      duration.FromTimedelta(td)
///
/// # JSON Mapping
///
/// In JSON format, the Duration type is encoded as a string rather than an
/// object, where the string ends in the suffix "s" (indicating seconds) and
/// is preceded by the number of seconds, with nanoseconds expressed as
/// fractional seconds. For example, 3 seconds with 0 nanoseconds should be
/// encoded in JSON format as "3s", while 3 seconds and 1 nanosecond should
/// be expressed in JSON format as "3.000000001s", and 3 seconds and 1
/// microsecond should be expressed in JSON format as "3.000001s".
///
///
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Eq)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Duration {
    /// Signed seconds of the span of time. Must be from -315,576,000,000
    /// to +315,576,000,000 inclusive. Note: these bounds are computed from:
    /// 60 sec/min * 60 min/hr * 24 hr/day * 365.25 days/year * 10000 years
    #[prost(int64, tag = "1")]
    pub seconds: i64,
    /// Signed fractions of a second at nanosecond resolution of the span
    /// of time. Durations less than one second are represented with a 0
    /// `seconds` field and a positive or negative `nanos` field. For durations
    /// of one second or more, a non-zero value for the `nanos` field must be
    /// of the same sign as the `seconds` field. Must be from -999,999,999
    /// to +999,999,999 inclusive.
    #[prost(int32, tag = "2")]
    pub nanos: i32,
}

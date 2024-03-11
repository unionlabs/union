// @generated
/// ModuleSchemaDescriptor describe's a module's ORM schema.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ModuleSchemaDescriptor {
    #[prost(message, repeated, tag = "1")]
    pub schema_file: ::prost::alloc::vec::Vec<module_schema_descriptor::FileEntry>,
    /// prefix is an optional prefix that precedes all keys in this module's
    /// store.
    #[prost(bytes = "vec", tag = "2")]
    pub prefix: ::prost::alloc::vec::Vec<u8>,
}
/// Nested message and enum types in `ModuleSchemaDescriptor`.
pub mod module_schema_descriptor {
    /// FileEntry describes an ORM file used in a module.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct FileEntry {
        /// id is a prefix that will be varint encoded and prepended to all the
        /// table keys specified in the file's tables.
        #[prost(uint32, tag = "1")]
        pub id: u32,
        /// proto_file_name is the name of a file .proto in that contains
        /// table definitions. The .proto file must be in a package that the
        /// module has referenced using cosmos.app.v1.ModuleDescriptor.use_package.
        #[prost(string, tag = "2")]
        pub proto_file_name: ::prost::alloc::string::String,
        /// storage_type optionally indicates the type of storage this file's
        /// tables should used. If it is left unspecified, the default KV-storage
        /// of the app will be used.
        #[prost(enumeration = "super::StorageType", tag = "3")]
        pub storage_type: i32,
    }
    impl ::prost::Name for FileEntry {
        const NAME: &'static str = "FileEntry";
        const PACKAGE: &'static str = "cosmos.orm.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!("cosmos.orm.v1alpha1.ModuleSchemaDescriptor.{}", Self::NAME)
        }
    }
}
impl ::prost::Name for ModuleSchemaDescriptor {
    const NAME: &'static str = "ModuleSchemaDescriptor";
    const PACKAGE: &'static str = "cosmos.orm.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.orm.v1alpha1.{}", Self::NAME)
    }
}
/// StorageType
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum StorageType {
    /// STORAGE_TYPE_DEFAULT_UNSPECIFIED indicates the persistent storage where all
    /// data is stored in the regular Merkle-tree backed KV-store.
    DefaultUnspecified = 0,
    /// STORAGE_TYPE_MEMORY indicates in-memory storage that will be
    /// reloaded every time an app restarts. Tables with this type of storage
    /// will by default be ignored when importing and exporting a module's
    /// state from JSON.
    Memory = 1,
    /// STORAGE_TYPE_TRANSIENT indicates transient storage that is reset
    /// at the end of every block. Tables with this type of storage
    /// will by default be ignored when importing and exporting a module's
    /// state from JSON.
    Transient = 2,
}
impl StorageType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            StorageType::DefaultUnspecified => "STORAGE_TYPE_DEFAULT_UNSPECIFIED",
            StorageType::Memory => "STORAGE_TYPE_MEMORY",
            StorageType::Transient => "STORAGE_TYPE_TRANSIENT",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "STORAGE_TYPE_DEFAULT_UNSPECIFIED" => Some(Self::DefaultUnspecified),
            "STORAGE_TYPE_MEMORY" => Some(Self::Memory),
            "STORAGE_TYPE_TRANSIENT" => Some(Self::Transient),
            _ => None,
        }
    }
}
// @@protoc_insertion_point(module)

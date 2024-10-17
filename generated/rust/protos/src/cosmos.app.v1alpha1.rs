// @generated
/// ModuleDescriptor describes an app module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ModuleDescriptor {
    /// go_import names the package that should be imported by an app to load the
    /// module in the runtime module registry. It is required to make debugging
    /// of configuration errors easier for users.
    #[prost(string, tag = "1")]
    pub go_import: ::prost::alloc::string::String,
    /// use_package refers to a protobuf package that this module
    /// uses and exposes to the world. In an app, only one module should "use"
    /// or own a single protobuf package. It is assumed that the module uses
    /// all of the .proto files in a single package.
    #[prost(message, repeated, tag = "2")]
    pub use_package: ::prost::alloc::vec::Vec<PackageReference>,
    /// can_migrate_from defines which module versions this module can migrate
    /// state from. The framework will check that one module version is able to
    /// migrate from a previous module version before attempting to update its
    /// config. It is assumed that modules can transitively migrate from earlier
    /// versions. For instance if v3 declares it can migrate from v2, and v2
    /// declares it can migrate from v1, the framework knows how to migrate
    /// from v1 to v3, assuming all 3 module versions are registered at runtime.
    #[prost(message, repeated, tag = "3")]
    pub can_migrate_from: ::prost::alloc::vec::Vec<MigrateFromInfo>,
}
impl ::prost::Name for ModuleDescriptor {
    const NAME: &'static str = "ModuleDescriptor";
    const PACKAGE: &'static str = "cosmos.app.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.app.v1alpha1.{}", Self::NAME)
    }
}
/// PackageReference is a reference to a protobuf package used by a module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PackageReference {
    /// name is the fully-qualified name of the package.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// revision is the optional revision of the package that is being used.
    /// Protobuf packages used in Cosmos should generally have a major version
    /// as the last part of the package name, ex. foo.bar.baz.v1.
    /// The revision of a package can be thought of as the minor version of a
    /// package which has additional backwards compatible definitions that weren't
    /// present in a previous version.
    ///
    /// A package should indicate its revision with a source code comment
    /// above the package declaration in one of its files containing the
    /// text "Revision N" where N is an integer revision. All packages start
    /// at revision 0 the first time they are released in a module.
    ///
    /// When a new version of a module is released and items are added to existing
    /// .proto files, these definitions should contain comments of the form
    /// "Since: Revision N" where N is an integer revision.
    ///
    /// When the module runtime starts up, it will check the pinned proto
    /// image and panic if there are runtime protobuf definitions that are not
    /// in the pinned descriptor which do not have
    /// a "Since Revision N" comment or have a "Since Revision N" comment where
    /// N is <= to the revision specified here. This indicates that the protobuf
    /// files have been updated, but the pinned file descriptor hasn't.
    ///
    /// If there are items in the pinned file descriptor with a revision
    /// greater than the value indicated here, this will also cause a panic
    /// as it may mean that the pinned descriptor for a legacy module has been
    /// improperly updated or that there is some other versioning discrepancy.
    /// Runtime protobuf definitions will also be checked for compatibility
    /// with pinned file descriptors to make sure there are no incompatible changes.
    ///
    /// This behavior ensures that:
    /// * pinned proto images are up-to-date
    /// * protobuf files are carefully annotated with revision comments which
    ///    are important good client UX
    /// * protobuf files are changed in backwards and forwards compatible ways
    #[prost(uint32, tag = "2")]
    pub revision: u32,
}
impl ::prost::Name for PackageReference {
    const NAME: &'static str = "PackageReference";
    const PACKAGE: &'static str = "cosmos.app.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.app.v1alpha1.{}", Self::NAME)
    }
}
/// MigrateFromInfo is information on a module version that a newer module
/// can migrate from.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MigrateFromInfo {
    /// module is the fully-qualified protobuf name of the module config object
    /// for the previous module version, ex: "cosmos.group.module.v1.Module".
    #[prost(string, tag = "1")]
    pub module: ::prost::alloc::string::String,
}
impl ::prost::Name for MigrateFromInfo {
    const NAME: &'static str = "MigrateFromInfo";
    const PACKAGE: &'static str = "cosmos.app.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.app.v1alpha1.{}", Self::NAME)
    }
}
/// Config represents the configuration for a Cosmos SDK ABCI app.
/// It is intended that all state machine logic including the version of
/// baseapp and tx handlers (and possibly even Tendermint) that an app needs
/// can be described in a config object. For compatibility, the framework should
/// allow a mixture of declarative and imperative app wiring, however, apps
/// that strive for the maximum ease of maintainability should be able to describe
/// their state machine with a config object alone.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Config {
    /// modules are the module configurations for the app.
    #[prost(message, repeated, tag = "1")]
    pub modules: ::prost::alloc::vec::Vec<ModuleConfig>,
    /// golang_bindings specifies explicit interface to implementation type bindings which
    /// depinject uses to resolve interface inputs to provider functions.  The scope of this
    /// field's configuration is global (not module specific).
    #[prost(message, repeated, tag = "2")]
    pub golang_bindings: ::prost::alloc::vec::Vec<GolangBinding>,
}
impl ::prost::Name for Config {
    const NAME: &'static str = "Config";
    const PACKAGE: &'static str = "cosmos.app.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.app.v1alpha1.{}", Self::NAME)
    }
}
/// ModuleConfig is a module configuration for an app.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ModuleConfig {
    /// name is the unique name of the module within the app. It should be a name
    /// that persists between different versions of a module so that modules
    /// can be smoothly upgraded to new versions.
    ///
    /// For example, for the module cosmos.bank.module.v1.Module, we may chose
    /// to simply name the module "bank" in the app. When we upgrade to
    /// cosmos.bank.module.v2.Module, the app-specific name "bank" stays the same
    /// and the framework knows that the v2 module should receive all the same state
    /// that the v1 module had. Note: modules should provide info on which versions
    /// they can migrate from in the ModuleDescriptor.can_migration_from field.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// config is the config object for the module. Module config messages should
    /// define a ModuleDescriptor using the cosmos.app.v1alpha1.is_module extension.
    #[prost(message, optional, tag = "2")]
    pub config: ::core::option::Option<::pbjson_types::Any>,
    /// golang_bindings specifies explicit interface to implementation type bindings which
    /// depinject uses to resolve interface inputs to provider functions.  The scope of this
    /// field's configuration is module specific.
    #[prost(message, repeated, tag = "3")]
    pub golang_bindings: ::prost::alloc::vec::Vec<GolangBinding>,
}
impl ::prost::Name for ModuleConfig {
    const NAME: &'static str = "ModuleConfig";
    const PACKAGE: &'static str = "cosmos.app.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.app.v1alpha1.{}", Self::NAME)
    }
}
/// GolangBinding is an explicit interface type to implementing type binding for dependency injection.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GolangBinding {
    /// interface_type is the interface type which will be bound to a specific implementation type
    #[prost(string, tag = "1")]
    pub interface_type: ::prost::alloc::string::String,
    /// implementation is the implementing type which will be supplied when an input of type interface is requested
    #[prost(string, tag = "2")]
    pub implementation: ::prost::alloc::string::String,
}
impl ::prost::Name for GolangBinding {
    const NAME: &'static str = "GolangBinding";
    const PACKAGE: &'static str = "cosmos.app.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.app.v1alpha1.{}", Self::NAME)
    }
}
/// QueryConfigRequest is the Query/Config request type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConfigRequest {}
impl ::prost::Name for QueryConfigRequest {
    const NAME: &'static str = "QueryConfigRequest";
    const PACKAGE: &'static str = "cosmos.app.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.app.v1alpha1.{}", Self::NAME)
    }
}
/// QueryConfigResponse is the Query/Config response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConfigResponse {
    /// config is the current app config.
    #[prost(message, optional, tag = "1")]
    pub config: ::core::option::Option<Config>,
}
impl ::prost::Name for QueryConfigResponse {
    const NAME: &'static str = "QueryConfigResponse";
    const PACKAGE: &'static str = "cosmos.app.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.app.v1alpha1.{}", Self::NAME)
    }
}
include!("cosmos.app.v1alpha1.tonic.rs");
// @@protoc_insertion_point(module)

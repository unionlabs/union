/// MemFile is the metadata information tied to
/// a single gno package / realm file
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MemFile {
    /// the name of the source gno file
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// the content of the source gno file
    #[prost(string, tag = "2")]
    pub body: ::prost::alloc::string::String,
}
/// MemPackage is the metadata information tied to
/// package / realm deployment
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MemPackage {
    /// the name of the package
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// the gno path of the package
    #[prost(string, tag = "2")]
    pub path: ::prost::alloc::string::String,
    /// the associated package gno source
    #[prost(message, repeated, tag = "3")]
    pub files: ::prost::alloc::vec::Vec<MemFile>,
    /// the (user defined) package type
    #[prost(message, optional, tag = "4")]
    pub r#type: ::core::option::Option<super::super::google::protobuf::Any>,
    /// the (user defined) extra information
    #[prost(message, optional, tag = "5")]
    pub info: ::core::option::Option<super::super::google::protobuf::Any>,
}
/// MsgAddPackage is the package deployment tx message,
/// denoted as "m_addpkg"
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgAddPackage {
    /// the package deployer
    #[prost(string, tag = "1")]
    pub creator: ::prost::alloc::string::String,
    /// the package being deployed
    #[prost(message, optional, tag = "2")]
    pub package: ::core::option::Option<MemPackage>,
    /// the amount of funds to be deposited at deployment, if any ("<amount><denomination>")
    #[prost(string, tag = "3")]
    pub send: ::prost::alloc::string::String,
    /// the amount of funds to put down for the storage fee, if any ("<amount><denomination>")
    #[prost(string, tag = "4")]
    pub max_deposit: ::prost::alloc::string::String,
}
/// MsgCall is the method invocation tx message,
/// denoted as "m_call"
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgCall {
    /// the bech32 address of the caller
    #[prost(string, tag = "1")]
    pub caller: ::prost::alloc::string::String,
    /// the amount of funds to be deposited to the package, if any ("<amount><denomination>")
    #[prost(string, tag = "2")]
    pub send: ::prost::alloc::string::String,
    /// the amount of funds to lock for the storage, if any ("<amount><denomination>")
    #[prost(string, tag = "3")]
    pub max_deposit: ::prost::alloc::string::String,
    /// the gno package path
    #[prost(string, tag = "4")]
    pub pkg_path: ::prost::alloc::string::String,
    /// the function name being invoked
    #[prost(string, tag = "5")]
    pub func: ::prost::alloc::string::String,
    /// the function arguments
    ///
    /// null | string\[\]
    #[prost(string, repeated, tag = "6")]
    pub args: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// MsgRun is the execute arbitrary Gno code tx message,
/// denoted as "m_run"
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgRun {
    /// the bech32 address of the caller
    #[prost(string, tag = "1")]
    pub caller: ::prost::alloc::string::String,
    /// the amount of funds to be deposited to the package, if any ("<amount><denomination>")
    #[prost(string, tag = "2")]
    pub send: ::prost::alloc::string::String,
    /// the amount of funds to put down for the storage fee, if any ("<amount><denomination>")
    #[prost(string, tag = "3")]
    pub max_deposit: ::prost::alloc::string::String,
    /// the package being executed
    #[prost(message, optional, tag = "4")]
    pub package: ::core::option::Option<MemPackage>,
}
impl ::prost::Name for MemFile {
    const NAME: &'static str = "MemFile";
    const PACKAGE: &'static str = "gno.vm";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("gno.vm.{}", Self::NAME)
    }
}
impl ::prost::Name for MemPackage {
    const NAME: &'static str = "MemPackage";
    const PACKAGE: &'static str = "gno.vm";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("gno.vm.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgAddPackage {
    const NAME: &'static str = "MsgAddPackage";
    const PACKAGE: &'static str = "gno.vm";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("gno.vm.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgCall {
    const NAME: &'static str = "MsgCall";
    const PACKAGE: &'static str = "gno.vm";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("gno.vm.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgRun {
    const NAME: &'static str = "MsgRun";
    const PACKAGE: &'static str = "gno.vm";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("gno.vm.{}", Self::NAME)
    }
}

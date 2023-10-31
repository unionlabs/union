#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, clashing_extern_declarations, clippy::all)]
#[cfg(feature = "Security_Authentication_Identity_Core")]
pub mod Core;
#[cfg(feature = "Security_Authentication_Identity_Provider")]
pub mod Provider;
pub type EnterpriseKeyCredentialRegistrationInfo = *mut ::core::ffi::c_void;
pub type EnterpriseKeyCredentialRegistrationManager = *mut ::core::ffi::c_void;

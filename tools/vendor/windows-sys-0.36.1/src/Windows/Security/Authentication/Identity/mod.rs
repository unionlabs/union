#[cfg(feature = "Security_Authentication_Identity_Core")]
pub mod Core;
#[cfg(feature = "Security_Authentication_Identity_Provider")]
pub mod Provider;
pub type EnterpriseKeyCredentialRegistrationInfo = *mut ::core::ffi::c_void;
pub type EnterpriseKeyCredentialRegistrationManager = *mut ::core::ffi::c_void;

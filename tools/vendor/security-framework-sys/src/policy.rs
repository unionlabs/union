use core_foundation_sys::base::{Boolean, CFTypeID};
#[cfg(any(feature = "OSX_10_9", target_os = "ios"))]
use core_foundation_sys::base::CFOptionFlags;
use core_foundation_sys::string::CFStringRef;

use crate::base::SecPolicyRef;

#[cfg(any(feature = "OSX_10_9", target_os = "ios"))]
mod revocation_flags {
    use super::CFOptionFlags;

    pub const kSecRevocationOCSPMethod: CFOptionFlags = 1 << 0;
    pub const kSecRevocationCRLMethod: CFOptionFlags = 1 << 1;
    pub const kSecRevocationPreferCRL: CFOptionFlags = 1 << 2;
    pub const kSecRevocationRequirePositiveResponse: CFOptionFlags = 1 << 3;
    pub const kSecRevocationNetworkAccessDisabled: CFOptionFlags = 1 << 4;
    pub const kSecRevocationUseAnyAvailableMethod: CFOptionFlags = kSecRevocationOCSPMethod | kSecRevocationCRLMethod; 
}

#[cfg(any(feature = "OSX_10_9", target_os = "ios"))]
pub use revocation_flags::*;

extern "C" {
    pub fn SecPolicyCreateSSL(server: Boolean, hostname: CFStringRef) -> SecPolicyRef;
    #[cfg(any(feature = "OSX_10_9", target_os = "ios"))]
    pub fn SecPolicyCreateRevocation(revocationFlags: CFOptionFlags) -> SecPolicyRef;
    pub fn SecPolicyGetTypeID() -> CFTypeID;
    pub fn SecPolicyCreateBasicX509() -> SecPolicyRef;
}

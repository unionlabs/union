//! Encryption key support

use core_foundation::base::TCFType;
#[cfg(any(feature = "OSX_10_12", target_os = "ios"))]
use core_foundation::base::ToVoid;
#[cfg(any(feature = "OSX_10_12", target_os = "ios"))]
use core_foundation::data::CFData;
#[cfg(any(feature = "OSX_10_12", target_os = "ios"))]
use core_foundation::dictionary::CFDictionary;
#[cfg(any(feature = "OSX_10_12", target_os = "ios"))]
use core_foundation::error::{CFError, CFErrorRef};
use security_framework_sys::base::SecKeyRef;
#[cfg(any(feature = "OSX_10_12", target_os = "ios"))]
pub use security_framework_sys::key::Algorithm;
use security_framework_sys::key::SecKeyGetTypeID;
#[cfg(any(feature = "OSX_10_12", target_os = "ios"))]
use security_framework_sys::key::{
    SecKeyCopyAttributes, SecKeyCopyExternalRepresentation, SecKeyCreateSignature,
};
use std::fmt;

declare_TCFType! {
    /// A type representing an encryption key.
    SecKey, SecKeyRef
}
impl_TCFType!(SecKey, SecKeyRef, SecKeyGetTypeID);

unsafe impl Sync for SecKey {}
unsafe impl Send for SecKey {}

impl SecKey {
    #[cfg(any(feature = "OSX_10_12", target_os = "ios"))]
    /// Translates to SecKeyCopyAttributes
    pub fn attributes(&self) -> CFDictionary {
        let pka = unsafe { SecKeyCopyAttributes(self.to_void() as _) };
        unsafe { CFDictionary::wrap_under_create_rule(pka) }
    }

    #[cfg(any(feature = "OSX_10_12", target_os = "ios"))]
    /// Translates to SecKeyCopyExternalRepresentation
    pub fn external_representation(&self) -> Option<CFData> {
        let mut error: CFErrorRef = ::std::ptr::null_mut();
        let data = unsafe { SecKeyCopyExternalRepresentation(self.to_void() as _, &mut error) };
        if data.is_null() {
            return None;
        }
        Some(unsafe { CFData::wrap_under_create_rule(data) })
    }

    #[cfg(any(feature = "OSX_10_12", target_os = "ios"))]
    /// Creates the cryptographic signature for a block of data using a private
    /// key and specified algorithm.
    pub fn create_signature(
        &self,
        algorithm: Algorithm,
        input: impl AsRef<[u8]>,
    ) -> Result<Vec<u8>, CFError> {
        let mut error: CFErrorRef = std::ptr::null_mut();

        let output = unsafe {
            SecKeyCreateSignature(
                self.as_concrete_TypeRef(),
                algorithm.into(),
                CFData::from_buffer(input.as_ref()).as_concrete_TypeRef(),
                &mut error,
            )
        };

        if !error.is_null() {
            Err(unsafe { CFError::wrap_under_create_rule(error) })
        } else {
            let output = unsafe { CFData::wrap_under_create_rule(output) };
            Ok(output.to_vec())
        }
    }
}

// FIXME
impl fmt::Debug for SecKey {
    #[cold]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "SecKey")
    }
}

//! OIDs associated with certificate properties.
use core_foundation::base::TCFType;
use core_foundation::string::CFString;
use core_foundation_sys::string::CFStringRef;
use security_framework_sys::certificate_oids::*;

/// An identifier of a property of a certificate.
#[derive(Copy, Clone)]
pub struct CertificateOid(CFStringRef);

#[allow(missing_docs)]
impl CertificateOid {
    #[inline(always)]
    pub fn x509_v1_signature_algorithm() -> Self {
        unsafe { Self(kSecOIDX509V1SignatureAlgorithm) }
    }

    /// Returns the underlying raw pointer corresponding to this OID.
    #[inline(always)]
    pub fn as_ptr(&self) -> CFStringRef {
        self.0
    }

    /// Returns the string representation of the OID.
    #[inline]
    pub fn to_str(&self) -> CFString {
        unsafe { CFString::wrap_under_get_rule(self.0) }
    }
}

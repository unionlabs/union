pub type Certificate = *mut ::core::ffi::c_void;
pub type CertificateChain = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Security_Cryptography_Certificates\"`*"]
#[repr(transparent)]
pub struct CertificateChainPolicy(pub i32);
impl CertificateChainPolicy {
    pub const Base: Self = Self(0i32);
    pub const Ssl: Self = Self(1i32);
    pub const NTAuthentication: Self = Self(2i32);
    pub const MicrosoftRoot: Self = Self(3i32);
}
impl ::core::marker::Copy for CertificateChainPolicy {}
impl ::core::clone::Clone for CertificateChainPolicy {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CertificateExtension = *mut ::core::ffi::c_void;
pub type CertificateKeyUsages = *mut ::core::ffi::c_void;
pub type CertificateQuery = *mut ::core::ffi::c_void;
pub type CertificateRequestProperties = *mut ::core::ffi::c_void;
pub type CertificateStore = *mut ::core::ffi::c_void;
pub type ChainBuildingParameters = *mut ::core::ffi::c_void;
pub type ChainValidationParameters = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Security_Cryptography_Certificates\"`*"]
#[repr(transparent)]
pub struct ChainValidationResult(pub i32);
impl ChainValidationResult {
    pub const Success: Self = Self(0i32);
    pub const Untrusted: Self = Self(1i32);
    pub const Revoked: Self = Self(2i32);
    pub const Expired: Self = Self(3i32);
    pub const IncompleteChain: Self = Self(4i32);
    pub const InvalidSignature: Self = Self(5i32);
    pub const WrongUsage: Self = Self(6i32);
    pub const InvalidName: Self = Self(7i32);
    pub const InvalidCertificateAuthorityPolicy: Self = Self(8i32);
    pub const BasicConstraintsError: Self = Self(9i32);
    pub const UnknownCriticalExtension: Self = Self(10i32);
    pub const RevocationInformationMissing: Self = Self(11i32);
    pub const RevocationFailure: Self = Self(12i32);
    pub const OtherErrors: Self = Self(13i32);
}
impl ::core::marker::Copy for ChainValidationResult {}
impl ::core::clone::Clone for ChainValidationResult {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CmsAttachedSignature = *mut ::core::ffi::c_void;
pub type CmsDetachedSignature = *mut ::core::ffi::c_void;
pub type CmsSignerInfo = *mut ::core::ffi::c_void;
pub type CmsTimestampInfo = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Security_Cryptography_Certificates\"`*"]
#[repr(transparent)]
pub struct EnrollKeyUsages(pub u32);
impl EnrollKeyUsages {
    pub const None: Self = Self(0u32);
    pub const Decryption: Self = Self(1u32);
    pub const Signing: Self = Self(2u32);
    pub const KeyAgreement: Self = Self(4u32);
    pub const All: Self = Self(16777215u32);
}
impl ::core::marker::Copy for EnrollKeyUsages {}
impl ::core::clone::Clone for EnrollKeyUsages {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Security_Cryptography_Certificates\"`*"]
#[repr(transparent)]
pub struct ExportOption(pub i32);
impl ExportOption {
    pub const NotExportable: Self = Self(0i32);
    pub const Exportable: Self = Self(1i32);
}
impl ::core::marker::Copy for ExportOption {}
impl ::core::clone::Clone for ExportOption {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Security_Cryptography_Certificates\"`*"]
#[repr(transparent)]
pub struct InstallOptions(pub u32);
impl InstallOptions {
    pub const None: Self = Self(0u32);
    pub const DeleteExpired: Self = Self(1u32);
}
impl ::core::marker::Copy for InstallOptions {}
impl ::core::clone::Clone for InstallOptions {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Security_Cryptography_Certificates\"`*"]
#[repr(transparent)]
pub struct KeyProtectionLevel(pub i32);
impl KeyProtectionLevel {
    pub const NoConsent: Self = Self(0i32);
    pub const ConsentOnly: Self = Self(1i32);
    pub const ConsentWithPassword: Self = Self(2i32);
    pub const ConsentWithFingerprint: Self = Self(3i32);
}
impl ::core::marker::Copy for KeyProtectionLevel {}
impl ::core::clone::Clone for KeyProtectionLevel {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Security_Cryptography_Certificates\"`*"]
#[repr(transparent)]
pub struct KeySize(pub i32);
impl KeySize {
    pub const Invalid: Self = Self(0i32);
    pub const Rsa2048: Self = Self(2048i32);
    pub const Rsa4096: Self = Self(4096i32);
}
impl ::core::marker::Copy for KeySize {}
impl ::core::clone::Clone for KeySize {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PfxImportParameters = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Security_Cryptography_Certificates\"`*"]
#[repr(transparent)]
pub struct SignatureValidationResult(pub i32);
impl SignatureValidationResult {
    pub const Success: Self = Self(0i32);
    pub const InvalidParameter: Self = Self(1i32);
    pub const BadMessage: Self = Self(2i32);
    pub const InvalidSignature: Self = Self(3i32);
    pub const OtherErrors: Self = Self(4i32);
}
impl ::core::marker::Copy for SignatureValidationResult {}
impl ::core::clone::Clone for SignatureValidationResult {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SubjectAlternativeNameInfo = *mut ::core::ffi::c_void;
pub type UserCertificateEnrollmentManager = *mut ::core::ffi::c_void;
pub type UserCertificateStore = *mut ::core::ffi::c_void;

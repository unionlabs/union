#[doc = "*Required features: `\"ApplicationModel_Store_LicenseManagement\"`*"]
#[repr(transparent)]
pub struct LicenseRefreshOption(pub i32);
impl LicenseRefreshOption {
    pub const RunningLicenses: Self = Self(0i32);
    pub const AllLicenses: Self = Self(1i32);
}
impl ::core::marker::Copy for LicenseRefreshOption {}
impl ::core::clone::Clone for LicenseRefreshOption {
    fn clone(&self) -> Self {
        *self
    }
}
pub type LicenseSatisfactionInfo = *mut ::core::ffi::c_void;
pub type LicenseSatisfactionResult = *mut ::core::ffi::c_void;

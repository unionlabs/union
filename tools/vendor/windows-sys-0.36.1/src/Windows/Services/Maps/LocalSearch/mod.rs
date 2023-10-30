pub type LocalLocation = *mut ::core::ffi::c_void;
pub type LocalLocationFinderResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Services_Maps_LocalSearch\"`*"]
#[repr(transparent)]
pub struct LocalLocationFinderStatus(pub i32);
impl LocalLocationFinderStatus {
    pub const Success: Self = Self(0i32);
    pub const UnknownError: Self = Self(1i32);
    pub const InvalidCredentials: Self = Self(2i32);
    pub const InvalidCategory: Self = Self(3i32);
    pub const InvalidSearchTerm: Self = Self(4i32);
    pub const InvalidSearchArea: Self = Self(5i32);
    pub const NetworkFailure: Self = Self(6i32);
    pub const NotSupported: Self = Self(7i32);
}
impl ::core::marker::Copy for LocalLocationFinderStatus {}
impl ::core::clone::Clone for LocalLocationFinderStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type LocalLocationHoursOfOperationItem = *mut ::core::ffi::c_void;
pub type LocalLocationRatingInfo = *mut ::core::ffi::c_void;

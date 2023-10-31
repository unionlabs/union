#[cfg(feature = "ApplicationModel_UserActivities_Core")]
pub mod Core;
pub type IUserActivityContentInfo = *mut ::core::ffi::c_void;
pub type UserActivity = *mut ::core::ffi::c_void;
pub type UserActivityAttribution = *mut ::core::ffi::c_void;
pub type UserActivityChannel = *mut ::core::ffi::c_void;
pub type UserActivityContentInfo = *mut ::core::ffi::c_void;
pub type UserActivityRequest = *mut ::core::ffi::c_void;
pub type UserActivityRequestManager = *mut ::core::ffi::c_void;
pub type UserActivityRequestedEventArgs = *mut ::core::ffi::c_void;
pub type UserActivitySession = *mut ::core::ffi::c_void;
pub type UserActivitySessionHistoryItem = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_UserActivities\"`*"]
#[repr(transparent)]
pub struct UserActivityState(pub i32);
impl UserActivityState {
    pub const New: Self = Self(0i32);
    pub const Published: Self = Self(1i32);
}
impl ::core::marker::Copy for UserActivityState {}
impl ::core::clone::Clone for UserActivityState {
    fn clone(&self) -> Self {
        *self
    }
}
pub type UserActivityVisualElements = *mut ::core::ffi::c_void;

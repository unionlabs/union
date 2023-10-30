pub type DevicePortalConnection = *mut ::core::ffi::c_void;
pub type DevicePortalConnectionClosedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"System_Diagnostics_DevicePortal\"`*"]
#[repr(transparent)]
pub struct DevicePortalConnectionClosedReason(pub i32);
impl DevicePortalConnectionClosedReason {
    pub const Unknown: Self = Self(0i32);
    pub const ResourceLimitsExceeded: Self = Self(1i32);
    pub const ProtocolError: Self = Self(2i32);
    pub const NotAuthorized: Self = Self(3i32);
    pub const UserNotPresent: Self = Self(4i32);
    pub const ServiceTerminated: Self = Self(5i32);
}
impl ::core::marker::Copy for DevicePortalConnectionClosedReason {}
impl ::core::clone::Clone for DevicePortalConnectionClosedReason {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DevicePortalConnectionRequestReceivedEventArgs = *mut ::core::ffi::c_void;

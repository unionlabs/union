pub type CortanaActionableInsights = *mut ::core::ffi::c_void;
pub type CortanaActionableInsightsOptions = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Services_Cortana\"`, `\"deprecated\"`*"]
#[cfg(feature = "deprecated")]
#[repr(transparent)]
pub struct CortanaPermission(pub i32);
#[cfg(feature = "deprecated")]
impl CortanaPermission {
    pub const BrowsingHistory: Self = Self(0i32);
    pub const Calendar: Self = Self(1i32);
    pub const CallHistory: Self = Self(2i32);
    pub const Contacts: Self = Self(3i32);
    pub const Email: Self = Self(4i32);
    pub const InputPersonalization: Self = Self(5i32);
    pub const Location: Self = Self(6i32);
    pub const Messaging: Self = Self(7i32);
    pub const Microphone: Self = Self(8i32);
    pub const Personalization: Self = Self(9i32);
    pub const PhoneCall: Self = Self(10i32);
}
#[cfg(feature = "deprecated")]
impl ::core::marker::Copy for CortanaPermission {}
#[cfg(feature = "deprecated")]
impl ::core::clone::Clone for CortanaPermission {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Services_Cortana\"`, `\"deprecated\"`*"]
#[cfg(feature = "deprecated")]
#[repr(transparent)]
pub struct CortanaPermissionsChangeResult(pub i32);
#[cfg(feature = "deprecated")]
impl CortanaPermissionsChangeResult {
    pub const Success: Self = Self(0i32);
    pub const Unavailable: Self = Self(1i32);
    pub const DisabledByPolicy: Self = Self(2i32);
}
#[cfg(feature = "deprecated")]
impl ::core::marker::Copy for CortanaPermissionsChangeResult {}
#[cfg(feature = "deprecated")]
impl ::core::clone::Clone for CortanaPermissionsChangeResult {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CortanaPermissionsManager = *mut ::core::ffi::c_void;
pub type CortanaSettings = *mut ::core::ffi::c_void;

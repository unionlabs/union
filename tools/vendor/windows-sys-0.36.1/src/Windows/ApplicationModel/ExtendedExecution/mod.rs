#[cfg(feature = "ApplicationModel_ExtendedExecution_Foreground")]
pub mod Foreground;
#[doc = "*Required features: `\"ApplicationModel_ExtendedExecution\"`*"]
#[repr(transparent)]
pub struct ExtendedExecutionReason(pub i32);
impl ExtendedExecutionReason {
    pub const Unspecified: Self = Self(0i32);
    pub const LocationTracking: Self = Self(1i32);
    pub const SavingData: Self = Self(2i32);
}
impl ::core::marker::Copy for ExtendedExecutionReason {}
impl ::core::clone::Clone for ExtendedExecutionReason {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"ApplicationModel_ExtendedExecution\"`*"]
#[repr(transparent)]
pub struct ExtendedExecutionResult(pub i32);
impl ExtendedExecutionResult {
    pub const Allowed: Self = Self(0i32);
    pub const Denied: Self = Self(1i32);
}
impl ::core::marker::Copy for ExtendedExecutionResult {}
impl ::core::clone::Clone for ExtendedExecutionResult {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ExtendedExecutionRevokedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_ExtendedExecution\"`*"]
#[repr(transparent)]
pub struct ExtendedExecutionRevokedReason(pub i32);
impl ExtendedExecutionRevokedReason {
    pub const Resumed: Self = Self(0i32);
    pub const SystemPolicy: Self = Self(1i32);
}
impl ::core::marker::Copy for ExtendedExecutionRevokedReason {}
impl ::core::clone::Clone for ExtendedExecutionRevokedReason {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ExtendedExecutionSession = *mut ::core::ffi::c_void;

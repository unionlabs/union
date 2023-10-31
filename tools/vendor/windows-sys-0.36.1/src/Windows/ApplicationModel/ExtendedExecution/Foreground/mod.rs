#[doc = "*Required features: `\"ApplicationModel_ExtendedExecution_Foreground\"`*"]
#[repr(transparent)]
pub struct ExtendedExecutionForegroundReason(pub i32);
impl ExtendedExecutionForegroundReason {
    pub const Unspecified: Self = Self(0i32);
    pub const SavingData: Self = Self(1i32);
    pub const BackgroundAudio: Self = Self(2i32);
    pub const Unconstrained: Self = Self(3i32);
}
impl ::core::marker::Copy for ExtendedExecutionForegroundReason {}
impl ::core::clone::Clone for ExtendedExecutionForegroundReason {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"ApplicationModel_ExtendedExecution_Foreground\"`*"]
#[repr(transparent)]
pub struct ExtendedExecutionForegroundResult(pub i32);
impl ExtendedExecutionForegroundResult {
    pub const Allowed: Self = Self(0i32);
    pub const Denied: Self = Self(1i32);
}
impl ::core::marker::Copy for ExtendedExecutionForegroundResult {}
impl ::core::clone::Clone for ExtendedExecutionForegroundResult {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ExtendedExecutionForegroundRevokedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_ExtendedExecution_Foreground\"`*"]
#[repr(transparent)]
pub struct ExtendedExecutionForegroundRevokedReason(pub i32);
impl ExtendedExecutionForegroundRevokedReason {
    pub const Resumed: Self = Self(0i32);
    pub const SystemPolicy: Self = Self(1i32);
}
impl ::core::marker::Copy for ExtendedExecutionForegroundRevokedReason {}
impl ::core::clone::Clone for ExtendedExecutionForegroundRevokedReason {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ExtendedExecutionForegroundSession = *mut ::core::ffi::c_void;

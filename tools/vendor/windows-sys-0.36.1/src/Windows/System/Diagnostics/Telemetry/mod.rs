pub type PlatformTelemetryRegistrationResult = *mut ::core::ffi::c_void;
pub type PlatformTelemetryRegistrationSettings = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"System_Diagnostics_Telemetry\"`*"]
#[repr(transparent)]
pub struct PlatformTelemetryRegistrationStatus(pub i32);
impl PlatformTelemetryRegistrationStatus {
    pub const Success: Self = Self(0i32);
    pub const SettingsOutOfRange: Self = Self(1i32);
    pub const UnknownFailure: Self = Self(2i32);
}
impl ::core::marker::Copy for PlatformTelemetryRegistrationStatus {}
impl ::core::clone::Clone for PlatformTelemetryRegistrationStatus {
    fn clone(&self) -> Self {
        *self
    }
}

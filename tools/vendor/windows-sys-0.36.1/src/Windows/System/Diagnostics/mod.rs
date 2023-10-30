#[cfg(feature = "System_Diagnostics_DevicePortal")]
pub mod DevicePortal;
#[cfg(feature = "System_Diagnostics_Telemetry")]
pub mod Telemetry;
#[cfg(feature = "System_Diagnostics_TraceReporting")]
pub mod TraceReporting;
pub type DiagnosticActionResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"System_Diagnostics\"`*"]
#[repr(transparent)]
pub struct DiagnosticActionState(pub i32);
impl DiagnosticActionState {
    pub const Initializing: Self = Self(0i32);
    pub const Downloading: Self = Self(1i32);
    pub const VerifyingTrust: Self = Self(2i32);
    pub const Detecting: Self = Self(3i32);
    pub const Resolving: Self = Self(4i32);
    pub const VerifyingResolution: Self = Self(5i32);
    pub const Executing: Self = Self(6i32);
}
impl ::core::marker::Copy for DiagnosticActionState {}
impl ::core::clone::Clone for DiagnosticActionState {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DiagnosticInvoker = *mut ::core::ffi::c_void;
pub type ProcessCpuUsage = *mut ::core::ffi::c_void;
pub type ProcessCpuUsageReport = *mut ::core::ffi::c_void;
pub type ProcessDiagnosticInfo = *mut ::core::ffi::c_void;
pub type ProcessDiskUsage = *mut ::core::ffi::c_void;
pub type ProcessDiskUsageReport = *mut ::core::ffi::c_void;
pub type ProcessMemoryUsage = *mut ::core::ffi::c_void;
pub type ProcessMemoryUsageReport = *mut ::core::ffi::c_void;
pub type SystemCpuUsage = *mut ::core::ffi::c_void;
pub type SystemCpuUsageReport = *mut ::core::ffi::c_void;
pub type SystemDiagnosticInfo = *mut ::core::ffi::c_void;
pub type SystemMemoryUsage = *mut ::core::ffi::c_void;
pub type SystemMemoryUsageReport = *mut ::core::ffi::c_void;

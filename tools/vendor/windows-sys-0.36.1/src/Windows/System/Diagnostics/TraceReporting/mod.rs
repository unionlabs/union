#[doc = "*Required features: `\"System_Diagnostics_TraceReporting\"`*"]
#[repr(transparent)]
pub struct PlatformDiagnosticActionState(pub i32);
impl PlatformDiagnosticActionState {
    pub const Success: Self = Self(0i32);
    pub const FreeNetworkNotAvailable: Self = Self(1i32);
    pub const ACPowerNotAvailable: Self = Self(2i32);
}
impl ::core::marker::Copy for PlatformDiagnosticActionState {}
impl ::core::clone::Clone for PlatformDiagnosticActionState {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"System_Diagnostics_TraceReporting\"`*"]
#[repr(transparent)]
pub struct PlatformDiagnosticEscalationType(pub i32);
impl PlatformDiagnosticEscalationType {
    pub const OnCompletion: Self = Self(0i32);
    pub const OnFailure: Self = Self(1i32);
}
impl ::core::marker::Copy for PlatformDiagnosticEscalationType {}
impl ::core::clone::Clone for PlatformDiagnosticEscalationType {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"System_Diagnostics_TraceReporting\"`*"]
#[repr(transparent)]
pub struct PlatformDiagnosticEventBufferLatencies(pub u32);
impl PlatformDiagnosticEventBufferLatencies {
    pub const Normal: Self = Self(1u32);
    pub const CostDeferred: Self = Self(2u32);
    pub const Realtime: Self = Self(4u32);
}
impl ::core::marker::Copy for PlatformDiagnosticEventBufferLatencies {}
impl ::core::clone::Clone for PlatformDiagnosticEventBufferLatencies {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PlatformDiagnosticTraceInfo = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"System_Diagnostics_TraceReporting\"`*"]
#[repr(transparent)]
pub struct PlatformDiagnosticTracePriority(pub i32);
impl PlatformDiagnosticTracePriority {
    pub const Normal: Self = Self(0i32);
    pub const UserElevated: Self = Self(1i32);
}
impl ::core::marker::Copy for PlatformDiagnosticTracePriority {}
impl ::core::clone::Clone for PlatformDiagnosticTracePriority {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PlatformDiagnosticTraceRuntimeInfo = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"System_Diagnostics_TraceReporting\"`*"]
#[repr(transparent)]
pub struct PlatformDiagnosticTraceSlotState(pub i32);
impl PlatformDiagnosticTraceSlotState {
    pub const NotRunning: Self = Self(0i32);
    pub const Running: Self = Self(1i32);
    pub const Throttled: Self = Self(2i32);
}
impl ::core::marker::Copy for PlatformDiagnosticTraceSlotState {}
impl ::core::clone::Clone for PlatformDiagnosticTraceSlotState {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"System_Diagnostics_TraceReporting\"`*"]
#[repr(transparent)]
pub struct PlatformDiagnosticTraceSlotType(pub i32);
impl PlatformDiagnosticTraceSlotType {
    pub const Alternative: Self = Self(0i32);
    pub const AlwaysOn: Self = Self(1i32);
    pub const Mini: Self = Self(2i32);
}
impl ::core::marker::Copy for PlatformDiagnosticTraceSlotType {}
impl ::core::clone::Clone for PlatformDiagnosticTraceSlotType {
    fn clone(&self) -> Self {
        *self
    }
}

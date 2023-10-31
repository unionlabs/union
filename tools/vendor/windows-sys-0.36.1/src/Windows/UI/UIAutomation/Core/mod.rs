#[repr(C)]
#[doc = "*Required features: `\"UI_UIAutomation_Core\"`*"]
pub struct AutomationAnnotationTypeRegistration {
    pub LocalId: i32,
}
impl ::core::marker::Copy for AutomationAnnotationTypeRegistration {}
impl ::core::clone::Clone for AutomationAnnotationTypeRegistration {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"UI_UIAutomation_Core\"`*"]
pub struct AutomationRemoteOperationOperandId {
    pub Value: i32,
}
impl ::core::marker::Copy for AutomationRemoteOperationOperandId {}
impl ::core::clone::Clone for AutomationRemoteOperationOperandId {
    fn clone(&self) -> Self {
        *self
    }
}
pub type AutomationRemoteOperationResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_UIAutomation_Core\"`*"]
#[repr(transparent)]
pub struct AutomationRemoteOperationStatus(pub i32);
impl AutomationRemoteOperationStatus {
    pub const Success: Self = Self(0i32);
    pub const MalformedBytecode: Self = Self(1i32);
    pub const InstructionLimitExceeded: Self = Self(2i32);
    pub const UnhandledException: Self = Self(3i32);
    pub const ExecutionFailure: Self = Self(4i32);
}
impl ::core::marker::Copy for AutomationRemoteOperationStatus {}
impl ::core::clone::Clone for AutomationRemoteOperationStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CoreAutomationRemoteOperation = *mut ::core::ffi::c_void;
pub type CoreAutomationRemoteOperationContext = *mut ::core::ffi::c_void;
pub type ICoreAutomationConnectionBoundObjectProvider = *mut ::core::ffi::c_void;
pub type ICoreAutomationRemoteOperationExtensionProvider = *mut ::core::ffi::c_void;
pub type RemoteAutomationClientSession = *mut ::core::ffi::c_void;
pub type RemoteAutomationConnectionRequestedEventArgs = *mut ::core::ffi::c_void;
pub type RemoteAutomationDisconnectedEventArgs = *mut ::core::ffi::c_void;
pub type RemoteAutomationWindow = *mut ::core::ffi::c_void;

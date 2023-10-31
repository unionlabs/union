pub type PrintWorkflowBackgroundSession = *mut ::core::ffi::c_void;
pub type PrintWorkflowBackgroundSetupRequestedEventArgs = *mut ::core::ffi::c_void;
pub type PrintWorkflowConfiguration = *mut ::core::ffi::c_void;
pub type PrintWorkflowForegroundSession = *mut ::core::ffi::c_void;
pub type PrintWorkflowForegroundSetupRequestedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Graphics_Printing_Workflow\"`*"]
#[repr(transparent)]
pub struct PrintWorkflowJobAbortReason(pub i32);
impl PrintWorkflowJobAbortReason {
    pub const JobFailed: Self = Self(0i32);
    pub const UserCanceled: Self = Self(1i32);
}
impl ::core::marker::Copy for PrintWorkflowJobAbortReason {}
impl ::core::clone::Clone for PrintWorkflowJobAbortReason {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PrintWorkflowJobActivatedEventArgs = *mut ::core::ffi::c_void;
pub type PrintWorkflowJobBackgroundSession = *mut ::core::ffi::c_void;
pub type PrintWorkflowJobNotificationEventArgs = *mut ::core::ffi::c_void;
pub type PrintWorkflowJobStartingEventArgs = *mut ::core::ffi::c_void;
pub type PrintWorkflowJobTriggerDetails = *mut ::core::ffi::c_void;
pub type PrintWorkflowJobUISession = *mut ::core::ffi::c_void;
pub type PrintWorkflowObjectModelSourceFileContent = *mut ::core::ffi::c_void;
pub type PrintWorkflowObjectModelTargetPackage = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Graphics_Printing_Workflow\"`*"]
#[repr(transparent)]
pub struct PrintWorkflowPdlConversionType(pub i32);
impl PrintWorkflowPdlConversionType {
    pub const XpsToPdf: Self = Self(0i32);
    pub const XpsToPwgr: Self = Self(1i32);
    pub const XpsToPclm: Self = Self(2i32);
}
impl ::core::marker::Copy for PrintWorkflowPdlConversionType {}
impl ::core::clone::Clone for PrintWorkflowPdlConversionType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PrintWorkflowPdlConverter = *mut ::core::ffi::c_void;
pub type PrintWorkflowPdlDataAvailableEventArgs = *mut ::core::ffi::c_void;
pub type PrintWorkflowPdlModificationRequestedEventArgs = *mut ::core::ffi::c_void;
pub type PrintWorkflowPdlSourceContent = *mut ::core::ffi::c_void;
pub type PrintWorkflowPdlTargetStream = *mut ::core::ffi::c_void;
pub type PrintWorkflowPrinterJob = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Graphics_Printing_Workflow\"`*"]
#[repr(transparent)]
pub struct PrintWorkflowPrinterJobStatus(pub i32);
impl PrintWorkflowPrinterJobStatus {
    pub const Error: Self = Self(0i32);
    pub const Aborted: Self = Self(1i32);
    pub const InProgress: Self = Self(2i32);
    pub const Completed: Self = Self(3i32);
}
impl ::core::marker::Copy for PrintWorkflowPrinterJobStatus {}
impl ::core::clone::Clone for PrintWorkflowPrinterJobStatus {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Graphics_Printing_Workflow\"`*"]
#[repr(transparent)]
pub struct PrintWorkflowSessionStatus(pub i32);
impl PrintWorkflowSessionStatus {
    pub const Started: Self = Self(0i32);
    pub const Completed: Self = Self(1i32);
    pub const Aborted: Self = Self(2i32);
    pub const Closed: Self = Self(3i32);
    pub const PdlDataAvailableForModification: Self = Self(4i32);
}
impl ::core::marker::Copy for PrintWorkflowSessionStatus {}
impl ::core::clone::Clone for PrintWorkflowSessionStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PrintWorkflowSourceContent = *mut ::core::ffi::c_void;
pub type PrintWorkflowSpoolStreamContent = *mut ::core::ffi::c_void;
pub type PrintWorkflowStreamTarget = *mut ::core::ffi::c_void;
pub type PrintWorkflowSubmittedEventArgs = *mut ::core::ffi::c_void;
pub type PrintWorkflowSubmittedOperation = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Graphics_Printing_Workflow\"`*"]
#[repr(transparent)]
pub struct PrintWorkflowSubmittedStatus(pub i32);
impl PrintWorkflowSubmittedStatus {
    pub const Succeeded: Self = Self(0i32);
    pub const Canceled: Self = Self(1i32);
    pub const Failed: Self = Self(2i32);
}
impl ::core::marker::Copy for PrintWorkflowSubmittedStatus {}
impl ::core::clone::Clone for PrintWorkflowSubmittedStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PrintWorkflowTarget = *mut ::core::ffi::c_void;
pub type PrintWorkflowTriggerDetails = *mut ::core::ffi::c_void;
pub type PrintWorkflowUIActivatedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Graphics_Printing_Workflow\"`*"]
#[repr(transparent)]
pub struct PrintWorkflowUICompletionStatus(pub i32);
impl PrintWorkflowUICompletionStatus {
    pub const Completed: Self = Self(0i32);
    pub const LaunchFailed: Self = Self(1i32);
    pub const JobFailed: Self = Self(2i32);
    pub const UserCanceled: Self = Self(3i32);
}
impl ::core::marker::Copy for PrintWorkflowUICompletionStatus {}
impl ::core::clone::Clone for PrintWorkflowUICompletionStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PrintWorkflowUILauncher = *mut ::core::ffi::c_void;
pub type PrintWorkflowXpsDataAvailableEventArgs = *mut ::core::ffi::c_void;

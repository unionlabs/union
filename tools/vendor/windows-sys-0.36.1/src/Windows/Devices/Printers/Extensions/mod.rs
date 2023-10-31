pub type Print3DWorkflow = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Printers_Extensions\"`*"]
#[repr(transparent)]
pub struct Print3DWorkflowDetail(pub i32);
impl Print3DWorkflowDetail {
    pub const Unknown: Self = Self(0i32);
    pub const ModelExceedsPrintBed: Self = Self(1i32);
    pub const UploadFailed: Self = Self(2i32);
    pub const InvalidMaterialSelection: Self = Self(3i32);
    pub const InvalidModel: Self = Self(4i32);
    pub const ModelNotManifold: Self = Self(5i32);
    pub const InvalidPrintTicket: Self = Self(6i32);
}
impl ::core::marker::Copy for Print3DWorkflowDetail {}
impl ::core::clone::Clone for Print3DWorkflowDetail {
    fn clone(&self) -> Self {
        *self
    }
}
pub type Print3DWorkflowPrintRequestedEventArgs = *mut ::core::ffi::c_void;
pub type Print3DWorkflowPrinterChangedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Printers_Extensions\"`*"]
#[repr(transparent)]
pub struct Print3DWorkflowStatus(pub i32);
impl Print3DWorkflowStatus {
    pub const Abandoned: Self = Self(0i32);
    pub const Canceled: Self = Self(1i32);
    pub const Failed: Self = Self(2i32);
    pub const Slicing: Self = Self(3i32);
    pub const Submitted: Self = Self(4i32);
}
impl ::core::marker::Copy for Print3DWorkflowStatus {}
impl ::core::clone::Clone for Print3DWorkflowStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PrintNotificationEventDetails = *mut ::core::ffi::c_void;
pub type PrintTaskConfiguration = *mut ::core::ffi::c_void;
pub type PrintTaskConfigurationSaveRequest = *mut ::core::ffi::c_void;
pub type PrintTaskConfigurationSaveRequestedDeferral = *mut ::core::ffi::c_void;
pub type PrintTaskConfigurationSaveRequestedEventArgs = *mut ::core::ffi::c_void;

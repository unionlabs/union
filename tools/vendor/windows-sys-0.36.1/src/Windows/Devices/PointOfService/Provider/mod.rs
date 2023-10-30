pub type BarcodeScannerDisableScannerRequest = *mut ::core::ffi::c_void;
pub type BarcodeScannerDisableScannerRequestEventArgs = *mut ::core::ffi::c_void;
pub type BarcodeScannerEnableScannerRequest = *mut ::core::ffi::c_void;
pub type BarcodeScannerEnableScannerRequestEventArgs = *mut ::core::ffi::c_void;
pub type BarcodeScannerFrameReader = *mut ::core::ffi::c_void;
pub type BarcodeScannerFrameReaderFrameArrivedEventArgs = *mut ::core::ffi::c_void;
pub type BarcodeScannerGetSymbologyAttributesRequest = *mut ::core::ffi::c_void;
pub type BarcodeScannerGetSymbologyAttributesRequestEventArgs = *mut ::core::ffi::c_void;
pub type BarcodeScannerHideVideoPreviewRequest = *mut ::core::ffi::c_void;
pub type BarcodeScannerHideVideoPreviewRequestEventArgs = *mut ::core::ffi::c_void;
pub type BarcodeScannerProviderConnection = *mut ::core::ffi::c_void;
pub type BarcodeScannerProviderTriggerDetails = *mut ::core::ffi::c_void;
pub type BarcodeScannerSetActiveSymbologiesRequest = *mut ::core::ffi::c_void;
pub type BarcodeScannerSetActiveSymbologiesRequestEventArgs = *mut ::core::ffi::c_void;
pub type BarcodeScannerSetSymbologyAttributesRequest = *mut ::core::ffi::c_void;
pub type BarcodeScannerSetSymbologyAttributesRequestEventArgs = *mut ::core::ffi::c_void;
pub type BarcodeScannerStartSoftwareTriggerRequest = *mut ::core::ffi::c_void;
pub type BarcodeScannerStartSoftwareTriggerRequestEventArgs = *mut ::core::ffi::c_void;
pub type BarcodeScannerStopSoftwareTriggerRequest = *mut ::core::ffi::c_void;
pub type BarcodeScannerStopSoftwareTriggerRequestEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_PointOfService_Provider\"`*"]
#[repr(transparent)]
pub struct BarcodeScannerTriggerState(pub i32);
impl BarcodeScannerTriggerState {
    pub const Released: Self = Self(0i32);
    pub const Pressed: Self = Self(1i32);
}
impl ::core::marker::Copy for BarcodeScannerTriggerState {}
impl ::core::clone::Clone for BarcodeScannerTriggerState {
    fn clone(&self) -> Self {
        *self
    }
}
pub type BarcodeScannerVideoFrame = *mut ::core::ffi::c_void;
pub type BarcodeSymbologyAttributesBuilder = *mut ::core::ffi::c_void;

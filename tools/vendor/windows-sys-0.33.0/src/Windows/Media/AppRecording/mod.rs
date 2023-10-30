#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, clashing_extern_declarations, clippy::all)]
pub type AppRecordingManager = *mut ::core::ffi::c_void;
pub type AppRecordingResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: 'Media_AppRecording'*"]
#[repr(transparent)]
pub struct AppRecordingSaveScreenshotOption(pub i32);
impl AppRecordingSaveScreenshotOption {
    pub const None: Self = Self(0i32);
    pub const HdrContentVisible: Self = Self(1i32);
}
impl ::core::marker::Copy for AppRecordingSaveScreenshotOption {}
impl ::core::clone::Clone for AppRecordingSaveScreenshotOption {
    fn clone(&self) -> Self {
        *self
    }
}
pub type AppRecordingSaveScreenshotResult = *mut ::core::ffi::c_void;
pub type AppRecordingSavedScreenshotInfo = *mut ::core::ffi::c_void;
pub type AppRecordingStatus = *mut ::core::ffi::c_void;
pub type AppRecordingStatusDetails = *mut ::core::ffi::c_void;

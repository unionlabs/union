#[cfg(feature = "UI_WebUI_Core")]
pub mod Core;
pub type ActivatedDeferral = *mut ::core::ffi::c_void;
pub type ActivatedEventHandler = *mut ::core::ffi::c_void;
pub type ActivatedOperation = *mut ::core::ffi::c_void;
pub type BackgroundActivatedEventArgs = *mut ::core::ffi::c_void;
pub type BackgroundActivatedEventHandler = *mut ::core::ffi::c_void;
pub type EnteredBackgroundEventArgs = *mut ::core::ffi::c_void;
pub type EnteredBackgroundEventHandler = *mut ::core::ffi::c_void;
pub type HtmlPrintDocumentSource = *mut ::core::ffi::c_void;
pub type IActivatedEventArgsDeferral = *mut ::core::ffi::c_void;
pub type IWebUIBackgroundTaskInstance = *mut ::core::ffi::c_void;
pub type IWebUINavigatedEventArgs = *mut ::core::ffi::c_void;
pub type LeavingBackgroundEventArgs = *mut ::core::ffi::c_void;
pub type LeavingBackgroundEventHandler = *mut ::core::ffi::c_void;
pub type NavigatedEventHandler = *mut ::core::ffi::c_void;
pub type NewWebUIViewCreatedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_WebUI\"`*"]
#[repr(transparent)]
pub struct PrintContent(pub i32);
impl PrintContent {
    pub const AllPages: Self = Self(0i32);
    pub const CurrentPage: Self = Self(1i32);
    pub const CustomPageRange: Self = Self(2i32);
    pub const CurrentSelection: Self = Self(3i32);
}
impl ::core::marker::Copy for PrintContent {}
impl ::core::clone::Clone for PrintContent {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ResumingEventHandler = *mut ::core::ffi::c_void;
pub type SuspendingDeferral = *mut ::core::ffi::c_void;
pub type SuspendingEventArgs = *mut ::core::ffi::c_void;
pub type SuspendingEventHandler = *mut ::core::ffi::c_void;
pub type SuspendingOperation = *mut ::core::ffi::c_void;
pub type WebUIAppointmentsProviderAddAppointmentActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIAppointmentsProviderRemoveAppointmentActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIAppointmentsProviderReplaceAppointmentActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIAppointmentsProviderShowAppointmentDetailsActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIAppointmentsProviderShowTimeFrameActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIBackgroundTaskInstanceRuntimeClass = *mut ::core::ffi::c_void;
pub type WebUIBarcodeScannerPreviewActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUICachedFileUpdaterActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUICameraSettingsActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUICommandLineActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIContactCallActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIContactMapActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIContactMessageActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIContactPanelActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIContactPickerActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIContactPostActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIContactVideoCallActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIDeviceActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIDevicePairingActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIDialReceiverActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIFileActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIFileOpenPickerActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIFileOpenPickerContinuationEventArgs = *mut ::core::ffi::c_void;
pub type WebUIFileSavePickerActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIFileSavePickerContinuationEventArgs = *mut ::core::ffi::c_void;
pub type WebUIFolderPickerContinuationEventArgs = *mut ::core::ffi::c_void;
pub type WebUILaunchActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUILockScreenActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUILockScreenCallActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUILockScreenComponentActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUINavigatedDeferral = *mut ::core::ffi::c_void;
pub type WebUINavigatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUINavigatedOperation = *mut ::core::ffi::c_void;
pub type WebUIPhoneCallActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIPrint3DWorkflowActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIPrintTaskSettingsActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIPrintWorkflowForegroundTaskActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIProtocolActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIProtocolForResultsActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIRestrictedLaunchActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUISearchActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIShareTargetActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIStartupTaskActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIToastNotificationActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIUserDataAccountProviderActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIView = *mut ::core::ffi::c_void;
pub type WebUIVoiceCommandActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIWalletActionActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIWebAccountProviderActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebUIWebAuthenticationBrokerContinuationEventArgs = *mut ::core::ffi::c_void;

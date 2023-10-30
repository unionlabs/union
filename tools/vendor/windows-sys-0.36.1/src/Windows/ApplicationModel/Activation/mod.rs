#[doc = "*Required features: `\"ApplicationModel_Activation\"`*"]
#[repr(transparent)]
pub struct ActivationKind(pub i32);
impl ActivationKind {
    pub const Launch: Self = Self(0i32);
    pub const Search: Self = Self(1i32);
    pub const ShareTarget: Self = Self(2i32);
    pub const File: Self = Self(3i32);
    pub const Protocol: Self = Self(4i32);
    pub const FileOpenPicker: Self = Self(5i32);
    pub const FileSavePicker: Self = Self(6i32);
    pub const CachedFileUpdater: Self = Self(7i32);
    pub const ContactPicker: Self = Self(8i32);
    pub const Device: Self = Self(9i32);
    pub const PrintTaskSettings: Self = Self(10i32);
    pub const CameraSettings: Self = Self(11i32);
    pub const RestrictedLaunch: Self = Self(12i32);
    pub const AppointmentsProvider: Self = Self(13i32);
    pub const Contact: Self = Self(14i32);
    pub const LockScreenCall: Self = Self(15i32);
    pub const VoiceCommand: Self = Self(16i32);
    pub const LockScreen: Self = Self(17i32);
    pub const PickerReturned: Self = Self(1000i32);
    pub const WalletAction: Self = Self(1001i32);
    pub const PickFileContinuation: Self = Self(1002i32);
    pub const PickSaveFileContinuation: Self = Self(1003i32);
    pub const PickFolderContinuation: Self = Self(1004i32);
    pub const WebAuthenticationBrokerContinuation: Self = Self(1005i32);
    pub const WebAccountProvider: Self = Self(1006i32);
    pub const ComponentUI: Self = Self(1007i32);
    pub const ProtocolForResults: Self = Self(1009i32);
    pub const ToastNotification: Self = Self(1010i32);
    pub const Print3DWorkflow: Self = Self(1011i32);
    pub const DialReceiver: Self = Self(1012i32);
    pub const DevicePairing: Self = Self(1013i32);
    pub const UserDataAccountsProvider: Self = Self(1014i32);
    pub const FilePickerExperience: Self = Self(1015i32);
    pub const LockScreenComponent: Self = Self(1016i32);
    pub const ContactPanel: Self = Self(1017i32);
    pub const PrintWorkflowForegroundTask: Self = Self(1018i32);
    pub const GameUIProvider: Self = Self(1019i32);
    pub const StartupTask: Self = Self(1020i32);
    pub const CommandLineLaunch: Self = Self(1021i32);
    pub const BarcodeScannerProvider: Self = Self(1022i32);
    pub const PrintSupportJobUI: Self = Self(1023i32);
    pub const PrintSupportSettingsUI: Self = Self(1024i32);
    pub const PhoneCallActivation: Self = Self(1025i32);
    pub const VpnForeground: Self = Self(1026i32);
}
impl ::core::marker::Copy for ActivationKind {}
impl ::core::clone::Clone for ActivationKind {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"ApplicationModel_Activation\"`*"]
#[repr(transparent)]
pub struct ApplicationExecutionState(pub i32);
impl ApplicationExecutionState {
    pub const NotRunning: Self = Self(0i32);
    pub const Running: Self = Self(1i32);
    pub const Suspended: Self = Self(2i32);
    pub const Terminated: Self = Self(3i32);
    pub const ClosedByUser: Self = Self(4i32);
}
impl ::core::marker::Copy for ApplicationExecutionState {}
impl ::core::clone::Clone for ApplicationExecutionState {
    fn clone(&self) -> Self {
        *self
    }
}
pub type AppointmentsProviderAddAppointmentActivatedEventArgs = *mut ::core::ffi::c_void;
pub type AppointmentsProviderRemoveAppointmentActivatedEventArgs = *mut ::core::ffi::c_void;
pub type AppointmentsProviderReplaceAppointmentActivatedEventArgs = *mut ::core::ffi::c_void;
pub type AppointmentsProviderShowAppointmentDetailsActivatedEventArgs = *mut ::core::ffi::c_void;
pub type AppointmentsProviderShowTimeFrameActivatedEventArgs = *mut ::core::ffi::c_void;
pub type BackgroundActivatedEventArgs = *mut ::core::ffi::c_void;
pub type BarcodeScannerPreviewActivatedEventArgs = *mut ::core::ffi::c_void;
pub type CachedFileUpdaterActivatedEventArgs = *mut ::core::ffi::c_void;
pub type CameraSettingsActivatedEventArgs = *mut ::core::ffi::c_void;
pub type CommandLineActivatedEventArgs = *mut ::core::ffi::c_void;
pub type CommandLineActivationOperation = *mut ::core::ffi::c_void;
pub type ContactCallActivatedEventArgs = *mut ::core::ffi::c_void;
pub type ContactMapActivatedEventArgs = *mut ::core::ffi::c_void;
pub type ContactMessageActivatedEventArgs = *mut ::core::ffi::c_void;
pub type ContactPanelActivatedEventArgs = *mut ::core::ffi::c_void;
pub type ContactPickerActivatedEventArgs = *mut ::core::ffi::c_void;
pub type ContactPostActivatedEventArgs = *mut ::core::ffi::c_void;
pub type ContactVideoCallActivatedEventArgs = *mut ::core::ffi::c_void;
pub type DeviceActivatedEventArgs = *mut ::core::ffi::c_void;
pub type DevicePairingActivatedEventArgs = *mut ::core::ffi::c_void;
pub type DialReceiverActivatedEventArgs = *mut ::core::ffi::c_void;
pub type FileActivatedEventArgs = *mut ::core::ffi::c_void;
pub type FileOpenPickerActivatedEventArgs = *mut ::core::ffi::c_void;
pub type FileOpenPickerContinuationEventArgs = *mut ::core::ffi::c_void;
pub type FileSavePickerActivatedEventArgs = *mut ::core::ffi::c_void;
pub type FileSavePickerContinuationEventArgs = *mut ::core::ffi::c_void;
pub type FolderPickerContinuationEventArgs = *mut ::core::ffi::c_void;
pub type IActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IActivatedEventArgsWithUser = *mut ::core::ffi::c_void;
pub type IApplicationViewActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IAppointmentsProviderActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IAppointmentsProviderAddAppointmentActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IAppointmentsProviderRemoveAppointmentActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IAppointmentsProviderReplaceAppointmentActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IAppointmentsProviderShowAppointmentDetailsActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IAppointmentsProviderShowTimeFrameActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IBackgroundActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IBarcodeScannerPreviewActivatedEventArgs = *mut ::core::ffi::c_void;
pub type ICachedFileUpdaterActivatedEventArgs = *mut ::core::ffi::c_void;
pub type ICameraSettingsActivatedEventArgs = *mut ::core::ffi::c_void;
pub type ICommandLineActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IContactActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IContactCallActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IContactMapActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IContactMessageActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IContactPanelActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IContactPickerActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IContactPostActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IContactVideoCallActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IContactsProviderActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IContinuationActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IDeviceActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IDevicePairingActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IDialReceiverActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IFileActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IFileActivatedEventArgsWithCallerPackageFamilyName = *mut ::core::ffi::c_void;
pub type IFileActivatedEventArgsWithNeighboringFiles = *mut ::core::ffi::c_void;
pub type IFileOpenPickerActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IFileOpenPickerActivatedEventArgs2 = *mut ::core::ffi::c_void;
pub type IFileOpenPickerContinuationEventArgs = *mut ::core::ffi::c_void;
pub type IFileSavePickerActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IFileSavePickerActivatedEventArgs2 = *mut ::core::ffi::c_void;
pub type IFileSavePickerContinuationEventArgs = *mut ::core::ffi::c_void;
pub type IFolderPickerContinuationEventArgs = *mut ::core::ffi::c_void;
pub type ILaunchActivatedEventArgs = *mut ::core::ffi::c_void;
pub type ILaunchActivatedEventArgs2 = *mut ::core::ffi::c_void;
pub type ILockScreenActivatedEventArgs = *mut ::core::ffi::c_void;
pub type ILockScreenCallActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IPhoneCallActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IPickerReturnedActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IPrelaunchActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IPrint3DWorkflowActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IPrintTaskSettingsActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IProtocolActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IProtocolActivatedEventArgsWithCallerPackageFamilyNameAndData = *mut ::core::ffi::c_void;
pub type IProtocolForResultsActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IRestrictedLaunchActivatedEventArgs = *mut ::core::ffi::c_void;
pub type ISearchActivatedEventArgs = *mut ::core::ffi::c_void;
pub type ISearchActivatedEventArgsWithLinguisticDetails = *mut ::core::ffi::c_void;
pub type IShareTargetActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IStartupTaskActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IToastNotificationActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IUserDataAccountProviderActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IViewSwitcherProvider = *mut ::core::ffi::c_void;
pub type IVoiceCommandActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IWalletActionActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IWebAccountProviderActivatedEventArgs = *mut ::core::ffi::c_void;
pub type IWebAuthenticationBrokerContinuationEventArgs = *mut ::core::ffi::c_void;
pub type LaunchActivatedEventArgs = *mut ::core::ffi::c_void;
pub type LockScreenActivatedEventArgs = *mut ::core::ffi::c_void;
pub type LockScreenCallActivatedEventArgs = *mut ::core::ffi::c_void;
pub type LockScreenComponentActivatedEventArgs = *mut ::core::ffi::c_void;
pub type PhoneCallActivatedEventArgs = *mut ::core::ffi::c_void;
pub type PickerReturnedActivatedEventArgs = *mut ::core::ffi::c_void;
pub type Print3DWorkflowActivatedEventArgs = *mut ::core::ffi::c_void;
pub type PrintTaskSettingsActivatedEventArgs = *mut ::core::ffi::c_void;
pub type ProtocolActivatedEventArgs = *mut ::core::ffi::c_void;
pub type ProtocolForResultsActivatedEventArgs = *mut ::core::ffi::c_void;
pub type RestrictedLaunchActivatedEventArgs = *mut ::core::ffi::c_void;
pub type SearchActivatedEventArgs = *mut ::core::ffi::c_void;
pub type ShareTargetActivatedEventArgs = *mut ::core::ffi::c_void;
pub type SplashScreen = *mut ::core::ffi::c_void;
pub type StartupTaskActivatedEventArgs = *mut ::core::ffi::c_void;
pub type TileActivatedInfo = *mut ::core::ffi::c_void;
pub type ToastNotificationActivatedEventArgs = *mut ::core::ffi::c_void;
pub type UserDataAccountProviderActivatedEventArgs = *mut ::core::ffi::c_void;
pub type VoiceCommandActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WalletActionActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebAccountProviderActivatedEventArgs = *mut ::core::ffi::c_void;
pub type WebAuthenticationBrokerContinuationEventArgs = *mut ::core::ffi::c_void;

pub type AccountsSettingsPane = *mut ::core::ffi::c_void;
pub type AccountsSettingsPaneCommandsRequestedEventArgs = *mut ::core::ffi::c_void;
pub type AccountsSettingsPaneEventDeferral = *mut ::core::ffi::c_void;
pub type CredentialCommand = *mut ::core::ffi::c_void;
pub type CredentialCommandCredentialDeletedHandler = *mut ::core::ffi::c_void;
pub type SettingsCommand = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_ApplicationSettings\"`, `\"deprecated\"`*"]
#[cfg(feature = "deprecated")]
#[repr(transparent)]
pub struct SettingsEdgeLocation(pub i32);
#[cfg(feature = "deprecated")]
impl SettingsEdgeLocation {
    pub const Right: Self = Self(0i32);
    pub const Left: Self = Self(1i32);
}
#[cfg(feature = "deprecated")]
impl ::core::marker::Copy for SettingsEdgeLocation {}
#[cfg(feature = "deprecated")]
impl ::core::clone::Clone for SettingsEdgeLocation {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SettingsPane = *mut ::core::ffi::c_void;
pub type SettingsPaneCommandsRequest = *mut ::core::ffi::c_void;
pub type SettingsPaneCommandsRequestedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_ApplicationSettings\"`*"]
#[repr(transparent)]
pub struct SupportedWebAccountActions(pub u32);
impl SupportedWebAccountActions {
    pub const None: Self = Self(0u32);
    pub const Reconnect: Self = Self(1u32);
    pub const Remove: Self = Self(2u32);
    pub const ViewDetails: Self = Self(4u32);
    pub const Manage: Self = Self(8u32);
    pub const More: Self = Self(16u32);
}
impl ::core::marker::Copy for SupportedWebAccountActions {}
impl ::core::clone::Clone for SupportedWebAccountActions {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_ApplicationSettings\"`*"]
#[repr(transparent)]
pub struct WebAccountAction(pub i32);
impl WebAccountAction {
    pub const Reconnect: Self = Self(0i32);
    pub const Remove: Self = Self(1i32);
    pub const ViewDetails: Self = Self(2i32);
    pub const Manage: Self = Self(3i32);
    pub const More: Self = Self(4i32);
}
impl ::core::marker::Copy for WebAccountAction {}
impl ::core::clone::Clone for WebAccountAction {
    fn clone(&self) -> Self {
        *self
    }
}
pub type WebAccountCommand = *mut ::core::ffi::c_void;
pub type WebAccountCommandInvokedHandler = *mut ::core::ffi::c_void;
pub type WebAccountInvokedArgs = *mut ::core::ffi::c_void;
pub type WebAccountProviderCommand = *mut ::core::ffi::c_void;
pub type WebAccountProviderCommandInvokedHandler = *mut ::core::ffi::c_void;

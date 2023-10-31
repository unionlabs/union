#[link(name = "windows")]
extern "system" {
    #[doc = "*Required features: `\"Win32_Security_Isolation\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn CreateAppContainerProfile(pszappcontainername: ::windows_sys::core::PCWSTR, pszdisplayname: ::windows_sys::core::PCWSTR, pszdescription: ::windows_sys::core::PCWSTR, pcapabilities: *const super::SID_AND_ATTRIBUTES, dwcapabilitycount: u32, ppsidappcontainersid: *mut super::super::Foundation::PSID) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Security_Isolation\"`*"]
    pub fn DeleteAppContainerProfile(pszappcontainername: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Security_Isolation\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DeriveAppContainerSidFromAppContainerName(pszappcontainername: ::windows_sys::core::PCWSTR, ppsidappcontainersid: *mut super::super::Foundation::PSID) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Security_Isolation\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DeriveRestrictedAppContainerSidFromAppContainerSidAndRestrictedName(psidappcontainersid: super::super::Foundation::PSID, pszrestrictedappcontainername: ::windows_sys::core::PCWSTR, ppsidrestrictedappcontainersid: *mut super::super::Foundation::PSID) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Security_Isolation\"`*"]
    pub fn GetAppContainerFolderPath(pszappcontainersid: ::windows_sys::core::PCWSTR, ppszpath: *mut ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Security_Isolation\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetAppContainerNamedObjectPath(token: super::super::Foundation::HANDLE, appcontainersid: super::super::Foundation::PSID, objectpathlength: u32, objectpath: ::windows_sys::core::PWSTR, returnlength: *mut u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Security_Isolation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn GetAppContainerRegistryLocation(desiredaccess: u32, phappcontainerkey: *mut super::super::System::Registry::HKEY) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Security_Isolation\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IsProcessInIsolatedContainer(isprocessinisolatedcontainer: *mut super::super::Foundation::BOOL) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Security_Isolation\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IsProcessInIsolatedWindowsEnvironment(isprocessinisolatedwindowsenvironment: *mut super::super::Foundation::BOOL) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Security_Isolation\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IsProcessInWDAGContainer(reserved: *const ::core::ffi::c_void, isprocessinwdagcontainer: *mut super::super::Foundation::BOOL) -> ::windows_sys::core::HRESULT;
}
pub type IIsolatedAppLauncher = *mut ::core::ffi::c_void;
pub const IsolatedAppLauncher: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3162580016, data2: 59230, data3: 20433, data4: [150, 65, 31, 159, 30, 45, 154, 31] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_Security_Isolation\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct IsolatedAppLauncherTelemetryParameters {
    pub EnableForLaunch: super::super::Foundation::BOOL,
    pub CorrelationGUID: ::windows_sys::core::GUID,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for IsolatedAppLauncherTelemetryParameters {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for IsolatedAppLauncherTelemetryParameters {
    fn clone(&self) -> Self {
        *self
    }
}

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, clashing_extern_declarations, clippy::all)]
#[link(name = "windows")]
extern "system" {
    #[doc = "*Required features: 'Win32_System_DeveloperLicensing', 'Win32_Foundation'*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn AcquireDeveloperLicense(hwndparent: super::super::Foundation::HWND, pexpiration: *mut super::super::Foundation::FILETIME) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: 'Win32_System_DeveloperLicensing', 'Win32_Foundation'*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn CheckDeveloperLicense(pexpiration: *mut super::super::Foundation::FILETIME) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: 'Win32_System_DeveloperLicensing', 'Win32_Foundation'*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn RemoveDeveloperLicense(hwndparent: super::super::Foundation::HWND) -> ::windows_sys::core::HRESULT;
}

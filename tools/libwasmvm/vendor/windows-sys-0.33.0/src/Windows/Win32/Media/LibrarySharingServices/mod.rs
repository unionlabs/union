#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, clashing_extern_declarations, clippy::all)]
pub type IWindowsMediaLibrarySharingDevice = *mut ::core::ffi::c_void;
pub type IWindowsMediaLibrarySharingDeviceProperties = *mut ::core::ffi::c_void;
pub type IWindowsMediaLibrarySharingDeviceProperty = *mut ::core::ffi::c_void;
pub type IWindowsMediaLibrarySharingDevices = *mut ::core::ffi::c_void;
pub type IWindowsMediaLibrarySharingServices = *mut ::core::ffi::c_void;
#[doc = "*Required features: 'Win32_Media_LibrarySharingServices'*"]
pub type WindowsMediaLibrarySharingDeviceAuthorizationStatus = i32;
#[doc = "*Required features: 'Win32_Media_LibrarySharingServices'*"]
pub const DEVICE_AUTHORIZATION_UNKNOWN: WindowsMediaLibrarySharingDeviceAuthorizationStatus = 0i32;
#[doc = "*Required features: 'Win32_Media_LibrarySharingServices'*"]
pub const DEVICE_AUTHORIZATION_ALLOWED: WindowsMediaLibrarySharingDeviceAuthorizationStatus = 1i32;
#[doc = "*Required features: 'Win32_Media_LibrarySharingServices'*"]
pub const DEVICE_AUTHORIZATION_DENIED: WindowsMediaLibrarySharingDeviceAuthorizationStatus = 2i32;
pub const WindowsMediaLibrarySharingServices: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2908232448, data2: 31588, data3: 20057, data4: [163, 141, 210, 197, 191, 81, 221, 179] };

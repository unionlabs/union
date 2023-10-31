#[cfg_attr(windows, link(name = "windows"))]
extern "system" {
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn BroadcastSystemMessageA(flags: u32, lpinfo: *mut u32, msg: u32, wparam: super::super::Foundation::WPARAM, lparam: super::super::Foundation::LPARAM) -> i32;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn BroadcastSystemMessageExA(flags: BROADCAST_SYSTEM_MESSAGE_FLAGS, lpinfo: *mut BROADCAST_SYSTEM_MESSAGE_INFO, msg: u32, wparam: super::super::Foundation::WPARAM, lparam: super::super::Foundation::LPARAM, pbsminfo: *mut BSMINFO) -> i32;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn BroadcastSystemMessageExW(flags: BROADCAST_SYSTEM_MESSAGE_FLAGS, lpinfo: *mut BROADCAST_SYSTEM_MESSAGE_INFO, msg: u32, wparam: super::super::Foundation::WPARAM, lparam: super::super::Foundation::LPARAM, pbsminfo: *mut BSMINFO) -> i32;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn BroadcastSystemMessageW(flags: BROADCAST_SYSTEM_MESSAGE_FLAGS, lpinfo: *mut BROADCAST_SYSTEM_MESSAGE_INFO, msg: u32, wparam: super::super::Foundation::WPARAM, lparam: super::super::Foundation::LPARAM) -> i32;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn CloseDesktop(hdesktop: HDESK) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn CloseWindowStation(hwinsta: HWINSTA) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_Security"))]
    pub fn CreateDesktopA(lpszdesktop: ::windows_sys::core::PCSTR, lpszdevice: ::windows_sys::core::PCSTR, pdevmode: *mut super::super::Graphics::Gdi::DEVMODEA, dwflags: DESKTOP_CONTROL_FLAGS, dwdesiredaccess: u32, lpsa: *const super::super::Security::SECURITY_ATTRIBUTES) -> HDESK;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_Security"))]
    pub fn CreateDesktopExA(lpszdesktop: ::windows_sys::core::PCSTR, lpszdevice: ::windows_sys::core::PCSTR, pdevmode: *mut super::super::Graphics::Gdi::DEVMODEA, dwflags: DESKTOP_CONTROL_FLAGS, dwdesiredaccess: u32, lpsa: *const super::super::Security::SECURITY_ATTRIBUTES, ulheapsize: u32, pvoid: *mut ::core::ffi::c_void) -> HDESK;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_Security"))]
    pub fn CreateDesktopExW(lpszdesktop: ::windows_sys::core::PCWSTR, lpszdevice: ::windows_sys::core::PCWSTR, pdevmode: *mut super::super::Graphics::Gdi::DEVMODEW, dwflags: DESKTOP_CONTROL_FLAGS, dwdesiredaccess: u32, lpsa: *const super::super::Security::SECURITY_ATTRIBUTES, ulheapsize: u32, pvoid: *mut ::core::ffi::c_void) -> HDESK;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_Security"))]
    pub fn CreateDesktopW(lpszdesktop: ::windows_sys::core::PCWSTR, lpszdevice: ::windows_sys::core::PCWSTR, pdevmode: *mut super::super::Graphics::Gdi::DEVMODEW, dwflags: DESKTOP_CONTROL_FLAGS, dwdesiredaccess: u32, lpsa: *const super::super::Security::SECURITY_ATTRIBUTES) -> HDESK;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn CreateWindowStationA(lpwinsta: ::windows_sys::core::PCSTR, dwflags: u32, dwdesiredaccess: u32, lpsa: *const super::super::Security::SECURITY_ATTRIBUTES) -> HWINSTA;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn CreateWindowStationW(lpwinsta: ::windows_sys::core::PCWSTR, dwflags: u32, dwdesiredaccess: u32, lpsa: *const super::super::Security::SECURITY_ATTRIBUTES) -> HWINSTA;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn EnumDesktopWindows(hdesktop: HDESK, lpfn: super::super::UI::WindowsAndMessaging::WNDENUMPROC, lparam: super::super::Foundation::LPARAM) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn EnumDesktopsA(hwinsta: HWINSTA, lpenumfunc: DESKTOPENUMPROCA, lparam: super::super::Foundation::LPARAM) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn EnumDesktopsW(hwinsta: HWINSTA, lpenumfunc: DESKTOPENUMPROCW, lparam: super::super::Foundation::LPARAM) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn EnumWindowStationsA(lpenumfunc: WINSTAENUMPROCA, lparam: super::super::Foundation::LPARAM) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn EnumWindowStationsW(lpenumfunc: WINSTAENUMPROCW, lparam: super::super::Foundation::LPARAM) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`*"]
    pub fn GetProcessWindowStation() -> HWINSTA;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`*"]
    pub fn GetThreadDesktop(dwthreadid: u32) -> HDESK;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetUserObjectInformationA(hobj: super::super::Foundation::HANDLE, nindex: USER_OBJECT_INFORMATION_INDEX, pvinfo: *mut ::core::ffi::c_void, nlength: u32, lpnlengthneeded: *mut u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetUserObjectInformationW(hobj: super::super::Foundation::HANDLE, nindex: USER_OBJECT_INFORMATION_INDEX, pvinfo: *mut ::core::ffi::c_void, nlength: u32, lpnlengthneeded: *mut u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn OpenDesktopA(lpszdesktop: ::windows_sys::core::PCSTR, dwflags: DESKTOP_CONTROL_FLAGS, finherit: super::super::Foundation::BOOL, dwdesiredaccess: u32) -> HDESK;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn OpenDesktopW(lpszdesktop: ::windows_sys::core::PCWSTR, dwflags: DESKTOP_CONTROL_FLAGS, finherit: super::super::Foundation::BOOL, dwdesiredaccess: u32) -> HDESK;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn OpenInputDesktop(dwflags: DESKTOP_CONTROL_FLAGS, finherit: super::super::Foundation::BOOL, dwdesiredaccess: u32) -> HDESK;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn OpenWindowStationA(lpszwinsta: ::windows_sys::core::PCSTR, finherit: super::super::Foundation::BOOL, dwdesiredaccess: u32) -> HWINSTA;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn OpenWindowStationW(lpszwinsta: ::windows_sys::core::PCWSTR, finherit: super::super::Foundation::BOOL, dwdesiredaccess: u32) -> HWINSTA;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SetProcessWindowStation(hwinsta: HWINSTA) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SetThreadDesktop(hdesktop: HDESK) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SetUserObjectInformationA(hobj: super::super::Foundation::HANDLE, nindex: i32, pvinfo: *const ::core::ffi::c_void, nlength: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SetUserObjectInformationW(hobj: super::super::Foundation::HANDLE, nindex: i32, pvinfo: *const ::core::ffi::c_void, nlength: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SwitchDesktop(hdesktop: HDESK) -> super::super::Foundation::BOOL;
}
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`*"]
pub type BROADCAST_SYSTEM_MESSAGE_FLAGS = u32;
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`*"]
pub const BSF_ALLOWSFW: BROADCAST_SYSTEM_MESSAGE_FLAGS = 128u32;
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`*"]
pub const BSF_FLUSHDISK: BROADCAST_SYSTEM_MESSAGE_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`*"]
pub const BSF_FORCEIFHUNG: BROADCAST_SYSTEM_MESSAGE_FLAGS = 32u32;
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`*"]
pub const BSF_IGNORECURRENTTASK: BROADCAST_SYSTEM_MESSAGE_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`*"]
pub const BSF_NOHANG: BROADCAST_SYSTEM_MESSAGE_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`*"]
pub const BSF_NOTIMEOUTIFNOTHUNG: BROADCAST_SYSTEM_MESSAGE_FLAGS = 64u32;
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`*"]
pub const BSF_POSTMESSAGE: BROADCAST_SYSTEM_MESSAGE_FLAGS = 16u32;
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`*"]
pub const BSF_QUERY: BROADCAST_SYSTEM_MESSAGE_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`*"]
pub const BSF_SENDNOTIFYMESSAGE: BROADCAST_SYSTEM_MESSAGE_FLAGS = 256u32;
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`*"]
pub const BSF_LUID: BROADCAST_SYSTEM_MESSAGE_FLAGS = 1024u32;
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`*"]
pub const BSF_RETURNHDESK: BROADCAST_SYSTEM_MESSAGE_FLAGS = 512u32;
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`*"]
pub type BROADCAST_SYSTEM_MESSAGE_INFO = u32;
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`*"]
pub const BSM_ALLCOMPONENTS: BROADCAST_SYSTEM_MESSAGE_INFO = 0u32;
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`*"]
pub const BSM_ALLDESKTOPS: BROADCAST_SYSTEM_MESSAGE_INFO = 16u32;
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`*"]
pub const BSM_APPLICATIONS: BROADCAST_SYSTEM_MESSAGE_INFO = 8u32;
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`*"]
pub type DESKTOP_CONTROL_FLAGS = u32;
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`*"]
pub const DF_ALLOWOTHERACCOUNTHOOK: DESKTOP_CONTROL_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`*"]
pub type USER_OBJECT_INFORMATION_INDEX = u32;
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`*"]
pub const UOI_FLAGS: USER_OBJECT_INFORMATION_INDEX = 1u32;
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`*"]
pub const UOI_HEAPSIZE: USER_OBJECT_INFORMATION_INDEX = 5u32;
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`*"]
pub const UOI_IO: USER_OBJECT_INFORMATION_INDEX = 6u32;
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`*"]
pub const UOI_NAME: USER_OBJECT_INFORMATION_INDEX = 2u32;
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`*"]
pub const UOI_TYPE: USER_OBJECT_INFORMATION_INDEX = 3u32;
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`*"]
pub const UOI_USER_SID: USER_OBJECT_INFORMATION_INDEX = 4u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct BSMINFO {
    pub cbSize: u32,
    pub hdesk: HDESK,
    pub hwnd: super::super::Foundation::HWND,
    pub luid: super::super::Foundation::LUID,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for BSMINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for BSMINFO {
    fn clone(&self) -> Self {
        *self
    }
}
pub type HDESK = isize;
pub type HWINSTA = isize;
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct USEROBJECTFLAGS {
    pub fInherit: super::super::Foundation::BOOL,
    pub fReserved: super::super::Foundation::BOOL,
    pub dwFlags: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for USEROBJECTFLAGS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for USEROBJECTFLAGS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type DESKTOPENUMPROCA = ::core::option::Option<unsafe extern "system" fn(param0: ::windows_sys::core::PCSTR, param1: super::super::Foundation::LPARAM) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type DESKTOPENUMPROCW = ::core::option::Option<unsafe extern "system" fn(param0: ::windows_sys::core::PCWSTR, param1: super::super::Foundation::LPARAM) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type WINSTAENUMPROCA = ::core::option::Option<unsafe extern "system" fn(param0: ::windows_sys::core::PCSTR, param1: super::super::Foundation::LPARAM) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_System_StationsAndDesktops\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type WINSTAENUMPROCW = ::core::option::Option<unsafe extern "system" fn(param0: ::windows_sys::core::PCWSTR, param1: super::super::Foundation::LPARAM) -> super::super::Foundation::BOOL>;

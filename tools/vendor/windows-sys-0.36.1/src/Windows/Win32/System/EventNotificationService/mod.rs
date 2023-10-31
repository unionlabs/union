#[link(name = "windows")]
extern "system" {
    #[doc = "*Required features: `\"Win32_System_EventNotificationService\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IsDestinationReachableA(lpszdestination: ::windows_sys::core::PCSTR, lpqocinfo: *mut QOCINFO) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_EventNotificationService\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IsDestinationReachableW(lpszdestination: ::windows_sys::core::PCWSTR, lpqocinfo: *mut QOCINFO) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_EventNotificationService\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IsNetworkAlive(lpdwflags: *mut u32) -> super::super::Foundation::BOOL;
}
#[doc = "*Required features: `\"Win32_System_EventNotificationService\"`*"]
pub const CONNECTION_AOL: u32 = 4u32;
pub type ISensLogon = *mut ::core::ffi::c_void;
pub type ISensLogon2 = *mut ::core::ffi::c_void;
pub type ISensNetwork = *mut ::core::ffi::c_void;
pub type ISensOnNow = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Win32_System_EventNotificationService\"`*"]
pub const NETWORK_ALIVE_AOL: u32 = 4u32;
#[doc = "*Required features: `\"Win32_System_EventNotificationService\"`*"]
pub const NETWORK_ALIVE_INTERNET: u32 = 8u32;
#[doc = "*Required features: `\"Win32_System_EventNotificationService\"`*"]
pub const NETWORK_ALIVE_LAN: u32 = 1u32;
#[doc = "*Required features: `\"Win32_System_EventNotificationService\"`*"]
pub const NETWORK_ALIVE_WAN: u32 = 2u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_EventNotificationService\"`*"]
pub struct QOCINFO {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub dwInSpeed: u32,
    pub dwOutSpeed: u32,
}
impl ::core::marker::Copy for QOCINFO {}
impl ::core::clone::Clone for QOCINFO {
    fn clone(&self) -> Self {
        *self
    }
}
pub const SENS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3583494910, data2: 23455, data3: 4561, data4: [141, 210, 0, 170, 0, 74, 189, 94] };
pub const SENSGUID_EVENTCLASS_LOGON: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3583477296, data2: 23455, data3: 4561, data4: [141, 210, 0, 170, 0, 74, 189, 94] };
pub const SENSGUID_EVENTCLASS_LOGON2: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3583477328, data2: 23455, data3: 4561, data4: [141, 210, 0, 170, 0, 74, 189, 94] };
pub const SENSGUID_EVENTCLASS_NETWORK: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3583477280, data2: 23455, data3: 4561, data4: [141, 210, 0, 170, 0, 74, 189, 94] };
pub const SENSGUID_EVENTCLASS_ONNOW: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3583477312, data2: 23455, data3: 4561, data4: [141, 210, 0, 170, 0, 74, 189, 94] };
pub const SENSGUID_PUBLISHER: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1609440214, data2: 23451, data3: 4561, data4: [141, 210, 0, 170, 0, 74, 189, 94] };
pub const SENSGUID_SUBSCRIBER_LCE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3549661872, data2: 23453, data3: 4561, data4: [141, 210, 0, 170, 0, 74, 189, 94] };
pub const SENSGUID_SUBSCRIBER_WININET: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3549661877, data2: 23453, data3: 4561, data4: [141, 210, 0, 170, 0, 74, 189, 94] };
#[doc = "*Required features: `\"Win32_System_EventNotificationService\"`*"]
pub type SENS_CONNECTION_TYPE = u32;
#[doc = "*Required features: `\"Win32_System_EventNotificationService\"`*"]
pub const CONNECTION_LAN: SENS_CONNECTION_TYPE = 0u32;
#[doc = "*Required features: `\"Win32_System_EventNotificationService\"`*"]
pub const CONNECTION_WAN: SENS_CONNECTION_TYPE = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_EventNotificationService\"`*"]
pub struct SENS_QOCINFO {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub dwOutSpeed: u32,
    pub dwInSpeed: u32,
}
impl ::core::marker::Copy for SENS_QOCINFO {}
impl ::core::clone::Clone for SENS_QOCINFO {
    fn clone(&self) -> Self {
        *self
    }
}

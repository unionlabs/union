#[link(name = "windows")]
extern "system" {
    #[doc = "*Required features: `\"Win32_System_Time\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn EnumDynamicTimeZoneInformation(dwindex: u32, lptimezoneinformation: *mut DYNAMIC_TIME_ZONE_INFORMATION) -> u32;
    #[doc = "*Required features: `\"Win32_System_Time\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FileTimeToSystemTime(lpfiletime: *const super::super::Foundation::FILETIME, lpsystemtime: *mut super::super::Foundation::SYSTEMTIME) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_Time\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetDynamicTimeZoneInformation(ptimezoneinformation: *mut DYNAMIC_TIME_ZONE_INFORMATION) -> u32;
    #[doc = "*Required features: `\"Win32_System_Time\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetDynamicTimeZoneInformationEffectiveYears(lptimezoneinformation: *const DYNAMIC_TIME_ZONE_INFORMATION, firstyear: *mut u32, lastyear: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Time\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetTimeZoneInformation(lptimezoneinformation: *mut TIME_ZONE_INFORMATION) -> u32;
    #[doc = "*Required features: `\"Win32_System_Time\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetTimeZoneInformationForYear(wyear: u16, pdtzi: *const DYNAMIC_TIME_ZONE_INFORMATION, ptzi: *mut TIME_ZONE_INFORMATION) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_Time\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn LocalFileTimeToLocalSystemTime(timezoneinformation: *const TIME_ZONE_INFORMATION, localfiletime: *const super::super::Foundation::FILETIME, localsystemtime: *mut super::super::Foundation::SYSTEMTIME) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_Time\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn LocalSystemTimeToLocalFileTime(timezoneinformation: *const TIME_ZONE_INFORMATION, localsystemtime: *const super::super::Foundation::SYSTEMTIME, localfiletime: *mut super::super::Foundation::FILETIME) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_Time\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SetDynamicTimeZoneInformation(lptimezoneinformation: *const DYNAMIC_TIME_ZONE_INFORMATION) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_Time\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SetTimeZoneInformation(lptimezoneinformation: *const TIME_ZONE_INFORMATION) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_Time\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SystemTimeToFileTime(lpsystemtime: *const super::super::Foundation::SYSTEMTIME, lpfiletime: *mut super::super::Foundation::FILETIME) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_Time\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SystemTimeToTzSpecificLocalTime(lptimezoneinformation: *const TIME_ZONE_INFORMATION, lpuniversaltime: *const super::super::Foundation::SYSTEMTIME, lplocaltime: *mut super::super::Foundation::SYSTEMTIME) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_Time\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SystemTimeToTzSpecificLocalTimeEx(lptimezoneinformation: *const DYNAMIC_TIME_ZONE_INFORMATION, lpuniversaltime: *const super::super::Foundation::SYSTEMTIME, lplocaltime: *mut super::super::Foundation::SYSTEMTIME) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_Time\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn TzSpecificLocalTimeToSystemTime(lptimezoneinformation: *const TIME_ZONE_INFORMATION, lplocaltime: *const super::super::Foundation::SYSTEMTIME, lpuniversaltime: *mut super::super::Foundation::SYSTEMTIME) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_Time\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn TzSpecificLocalTimeToSystemTimeEx(lptimezoneinformation: *const DYNAMIC_TIME_ZONE_INFORMATION, lplocaltime: *const super::super::Foundation::SYSTEMTIME, lpuniversaltime: *mut super::super::Foundation::SYSTEMTIME) -> super::super::Foundation::BOOL;
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Time\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DYNAMIC_TIME_ZONE_INFORMATION {
    pub Bias: i32,
    pub StandardName: [u16; 32],
    pub StandardDate: super::super::Foundation::SYSTEMTIME,
    pub StandardBias: i32,
    pub DaylightName: [u16; 32],
    pub DaylightDate: super::super::Foundation::SYSTEMTIME,
    pub DaylightBias: i32,
    pub TimeZoneKeyName: [u16; 128],
    pub DynamicDaylightTimeDisabled: super::super::Foundation::BOOLEAN,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DYNAMIC_TIME_ZONE_INFORMATION {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DYNAMIC_TIME_ZONE_INFORMATION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Time\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct TIME_ZONE_INFORMATION {
    pub Bias: i32,
    pub StandardName: [u16; 32],
    pub StandardDate: super::super::Foundation::SYSTEMTIME,
    pub StandardBias: i32,
    pub DaylightName: [u16; 32],
    pub DaylightDate: super::super::Foundation::SYSTEMTIME,
    pub DaylightBias: i32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TIME_ZONE_INFORMATION {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TIME_ZONE_INFORMATION {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_System_Time\"`*"]
pub const TSF_Authenticated: u32 = 2u32;
#[doc = "*Required features: `\"Win32_System_Time\"`*"]
pub const TSF_Hardware: u32 = 1u32;
#[doc = "*Required features: `\"Win32_System_Time\"`*"]
pub const TSF_IPv6: u32 = 4u32;
#[doc = "*Required features: `\"Win32_System_Time\"`*"]
pub const TSF_SignatureAuthenticated: u32 = 8u32;
#[doc = "*Required features: `\"Win32_System_Time\"`*"]
pub const wszW32TimeRegKeyPolicyTimeProviders: &str = "Software\\Policies\\Microsoft\\W32Time\\TimeProviders";
#[doc = "*Required features: `\"Win32_System_Time\"`*"]
pub const wszW32TimeRegKeyTimeProviders: &str = "System\\CurrentControlSet\\Services\\W32Time\\TimeProviders";
#[doc = "*Required features: `\"Win32_System_Time\"`*"]
pub const wszW32TimeRegValueDllName: &str = "DllName";
#[doc = "*Required features: `\"Win32_System_Time\"`*"]
pub const wszW32TimeRegValueEnabled: &str = "Enabled";
#[doc = "*Required features: `\"Win32_System_Time\"`*"]
pub const wszW32TimeRegValueInputProvider: &str = "InputProvider";
#[doc = "*Required features: `\"Win32_System_Time\"`*"]
pub const wszW32TimeRegValueMetaDataProvider: &str = "MetaDataProvider";

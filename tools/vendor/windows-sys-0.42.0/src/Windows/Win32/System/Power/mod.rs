#[cfg_attr(windows, link(name = "windows"))]
extern "system" {
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn CallNtPowerInformation(informationlevel: POWER_INFORMATION_LEVEL, inputbuffer: *const ::core::ffi::c_void, inputbufferlength: u32, outputbuffer: *mut ::core::ffi::c_void, outputbufferlength: u32) -> super::super::Foundation::NTSTATUS;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn CanUserWritePwrScheme() -> super::super::Foundation::BOOLEAN;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DeletePwrScheme(uiid: u32) -> super::super::Foundation::BOOLEAN;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DevicePowerClose() -> super::super::Foundation::BOOLEAN;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DevicePowerEnumDevices(queryindex: u32, queryinterpretationflags: u32, queryflags: u32, preturnbuffer: *mut u8, pbuffersize: *mut u32) -> super::super::Foundation::BOOLEAN;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DevicePowerOpen(debugmask: u32) -> super::super::Foundation::BOOLEAN;
    #[doc = "*Required features: `\"Win32_System_Power\"`*"]
    pub fn DevicePowerSetDeviceState(devicedescription: ::windows_sys::core::PCWSTR, setflags: u32, setdata: *const ::core::ffi::c_void) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn EnumPwrSchemes(lpfn: PWRSCHEMESENUMPROC, lparam: super::super::Foundation::LPARAM) -> super::super::Foundation::BOOLEAN;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetActivePwrScheme(puiid: *mut u32) -> super::super::Foundation::BOOLEAN;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetCurrentPowerPolicies(pglobalpowerpolicy: *mut GLOBAL_POWER_POLICY, ppowerpolicy: *mut POWER_POLICY) -> super::super::Foundation::BOOLEAN;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetDevicePowerState(hdevice: super::super::Foundation::HANDLE, pfon: *mut super::super::Foundation::BOOL) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetPwrCapabilities(lpspc: *mut SYSTEM_POWER_CAPABILITIES) -> super::super::Foundation::BOOLEAN;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetPwrDiskSpindownRange(puimax: *mut u32, puimin: *mut u32) -> super::super::Foundation::BOOLEAN;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetSystemPowerStatus(lpsystempowerstatus: *mut SYSTEM_POWER_STATUS) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IsAdminOverrideActive(papp: *const ADMINISTRATOR_POWER_POLICY) -> super::super::Foundation::BOOLEAN;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IsPwrHibernateAllowed() -> super::super::Foundation::BOOLEAN;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IsPwrShutdownAllowed() -> super::super::Foundation::BOOLEAN;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IsPwrSuspendAllowed() -> super::super::Foundation::BOOLEAN;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IsSystemResumeAutomatic() -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_Power\"`*"]
    pub fn PowerCanRestoreIndividualDefaultPowerScheme(schemeguid: *const ::windows_sys::core::GUID) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PowerClearRequest(powerrequest: super::super::Foundation::HANDLE, requesttype: POWER_REQUEST_TYPE) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerCreatePossibleSetting(rootsystempowerkey: super::Registry::HKEY, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, possiblesettingindex: u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`, `\"Win32_System_Threading\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Threading"))]
    pub fn PowerCreateRequest(context: *const super::Threading::REASON_CONTEXT) -> super::super::Foundation::HANDLE;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerCreateSetting(rootsystempowerkey: super::Registry::HKEY, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerDeleteScheme(rootpowerkey: super::Registry::HKEY, schemeguid: *const ::windows_sys::core::GUID) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`*"]
    pub fn PowerDeterminePlatformRole() -> POWER_PLATFORM_ROLE;
    #[doc = "*Required features: `\"Win32_System_Power\"`*"]
    pub fn PowerDeterminePlatformRoleEx(version: POWER_PLATFORM_ROLE_VERSION) -> POWER_PLATFORM_ROLE;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerDuplicateScheme(rootpowerkey: super::Registry::HKEY, sourceschemeguid: *const ::windows_sys::core::GUID, destinationschemeguid: *mut *mut ::windows_sys::core::GUID) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerEnumerate(rootpowerkey: super::Registry::HKEY, schemeguid: *const ::windows_sys::core::GUID, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, accessflags: POWER_DATA_ACCESSOR, index: u32, buffer: *mut u8, buffersize: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerGetActiveScheme(userrootpowerkey: super::Registry::HKEY, activepolicyguid: *mut *mut ::windows_sys::core::GUID) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerImportPowerScheme(rootpowerkey: super::Registry::HKEY, importfilenamepath: ::windows_sys::core::PCWSTR, destinationschemeguid: *mut *mut ::windows_sys::core::GUID) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PowerIsSettingRangeDefined(subkeyguid: *const ::windows_sys::core::GUID, settingguid: *const ::windows_sys::core::GUID) -> super::super::Foundation::BOOLEAN;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn PowerOpenSystemPowerKey(phsystempowerkey: *mut super::Registry::HKEY, access: u32, openexisting: super::super::Foundation::BOOL) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn PowerOpenUserPowerKey(phuserpowerkey: *mut super::Registry::HKEY, access: u32, openexisting: super::super::Foundation::BOOL) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerReadACDefaultIndex(rootpowerkey: super::Registry::HKEY, schemepersonalityguid: *const ::windows_sys::core::GUID, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, acdefaultindex: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerReadACValue(rootpowerkey: super::Registry::HKEY, schemeguid: *const ::windows_sys::core::GUID, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, r#type: *mut u32, buffer: *mut u8, buffersize: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerReadACValueIndex(rootpowerkey: super::Registry::HKEY, schemeguid: *const ::windows_sys::core::GUID, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, acvalueindex: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerReadDCDefaultIndex(rootpowerkey: super::Registry::HKEY, schemepersonalityguid: *const ::windows_sys::core::GUID, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, dcdefaultindex: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerReadDCValue(rootpowerkey: super::Registry::HKEY, schemeguid: *const ::windows_sys::core::GUID, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, r#type: *mut u32, buffer: *mut u8, buffersize: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerReadDCValueIndex(rootpowerkey: super::Registry::HKEY, schemeguid: *const ::windows_sys::core::GUID, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, dcvalueindex: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerReadDescription(rootpowerkey: super::Registry::HKEY, schemeguid: *const ::windows_sys::core::GUID, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, buffer: *mut u8, buffersize: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerReadFriendlyName(rootpowerkey: super::Registry::HKEY, schemeguid: *const ::windows_sys::core::GUID, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, buffer: *mut u8, buffersize: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerReadIconResourceSpecifier(rootpowerkey: super::Registry::HKEY, schemeguid: *const ::windows_sys::core::GUID, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, buffer: *mut u8, buffersize: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerReadPossibleDescription(rootpowerkey: super::Registry::HKEY, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, possiblesettingindex: u32, buffer: *mut u8, buffersize: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerReadPossibleFriendlyName(rootpowerkey: super::Registry::HKEY, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, possiblesettingindex: u32, buffer: *mut u8, buffersize: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerReadPossibleValue(rootpowerkey: super::Registry::HKEY, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, r#type: *mut u32, possiblesettingindex: u32, buffer: *mut u8, buffersize: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`*"]
    pub fn PowerReadSettingAttributes(subgroupguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerReadValueIncrement(rootpowerkey: super::Registry::HKEY, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, valueincrement: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerReadValueMax(rootpowerkey: super::Registry::HKEY, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, valuemaximum: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerReadValueMin(rootpowerkey: super::Registry::HKEY, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, valueminimum: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerReadValueUnitsSpecifier(rootpowerkey: super::Registry::HKEY, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, buffer: *mut u8, buffersize: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`*"]
    pub fn PowerRegisterForEffectivePowerModeNotifications(version: u32, callback: EFFECTIVE_POWER_MODE_CALLBACK, context: *const ::core::ffi::c_void, registrationhandle: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PowerRegisterSuspendResumeNotification(flags: u32, recipient: super::super::Foundation::HANDLE, registrationhandle: *mut *mut ::core::ffi::c_void) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`*"]
    pub fn PowerRemovePowerSetting(powersettingsubkeyguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`*"]
    pub fn PowerReplaceDefaultPowerSchemes() -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`*"]
    pub fn PowerReportThermalEvent(event: *const THERMAL_EVENT) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`*"]
    pub fn PowerRestoreDefaultPowerSchemes() -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`*"]
    pub fn PowerRestoreIndividualDefaultPowerScheme(schemeguid: *const ::windows_sys::core::GUID) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerSetActiveScheme(userrootpowerkey: super::Registry::HKEY, schemeguid: *const ::windows_sys::core::GUID) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PowerSetRequest(powerrequest: super::super::Foundation::HANDLE, requesttype: POWER_REQUEST_TYPE) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_Power\"`*"]
    pub fn PowerSettingAccessCheck(accessflags: POWER_DATA_ACCESSOR, powerguid: *const ::windows_sys::core::GUID) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerSettingAccessCheckEx(accessflags: POWER_DATA_ACCESSOR, powerguid: *const ::windows_sys::core::GUID, accesstype: super::Registry::REG_SAM_FLAGS) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PowerSettingRegisterNotification(settingguid: *const ::windows_sys::core::GUID, flags: POWER_SETTING_REGISTER_NOTIFICATION_FLAGS, recipient: super::super::Foundation::HANDLE, registrationhandle: *mut *mut ::core::ffi::c_void) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`*"]
    pub fn PowerSettingUnregisterNotification(registrationhandle: HPOWERNOTIFY) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`*"]
    pub fn PowerUnregisterFromEffectivePowerModeNotifications(registrationhandle: *const ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_Power\"`*"]
    pub fn PowerUnregisterSuspendResumeNotification(registrationhandle: HPOWERNOTIFY) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerWriteACDefaultIndex(rootsystempowerkey: super::Registry::HKEY, schemepersonalityguid: *const ::windows_sys::core::GUID, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, defaultacindex: u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerWriteACValueIndex(rootpowerkey: super::Registry::HKEY, schemeguid: *const ::windows_sys::core::GUID, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, acvalueindex: u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerWriteDCDefaultIndex(rootsystempowerkey: super::Registry::HKEY, schemepersonalityguid: *const ::windows_sys::core::GUID, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, defaultdcindex: u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerWriteDCValueIndex(rootpowerkey: super::Registry::HKEY, schemeguid: *const ::windows_sys::core::GUID, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, dcvalueindex: u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerWriteDescription(rootpowerkey: super::Registry::HKEY, schemeguid: *const ::windows_sys::core::GUID, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, buffer: *const u8, buffersize: u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerWriteFriendlyName(rootpowerkey: super::Registry::HKEY, schemeguid: *const ::windows_sys::core::GUID, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, buffer: *const u8, buffersize: u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerWriteIconResourceSpecifier(rootpowerkey: super::Registry::HKEY, schemeguid: *const ::windows_sys::core::GUID, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, buffer: *const u8, buffersize: u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerWritePossibleDescription(rootpowerkey: super::Registry::HKEY, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, possiblesettingindex: u32, buffer: *const u8, buffersize: u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerWritePossibleFriendlyName(rootpowerkey: super::Registry::HKEY, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, possiblesettingindex: u32, buffer: *const u8, buffersize: u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerWritePossibleValue(rootpowerkey: super::Registry::HKEY, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, r#type: u32, possiblesettingindex: u32, buffer: *const u8, buffersize: u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`*"]
    pub fn PowerWriteSettingAttributes(subgroupguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, attributes: u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerWriteValueIncrement(rootpowerkey: super::Registry::HKEY, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, valueincrement: u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerWriteValueMax(rootpowerkey: super::Registry::HKEY, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, valuemaximum: u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerWriteValueMin(rootpowerkey: super::Registry::HKEY, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, valueminimum: u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn PowerWriteValueUnitsSpecifier(rootpowerkey: super::Registry::HKEY, subgroupofpowersettingsguid: *const ::windows_sys::core::GUID, powersettingguid: *const ::windows_sys::core::GUID, buffer: *const u8, buffersize: u32) -> u32;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ReadGlobalPwrPolicy(pglobalpowerpolicy: *const GLOBAL_POWER_POLICY) -> super::super::Foundation::BOOLEAN;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ReadProcessorPwrScheme(uiid: u32, pmachineprocessorpowerpolicy: *mut MACHINE_PROCESSOR_POWER_POLICY) -> super::super::Foundation::BOOLEAN;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ReadPwrScheme(uiid: u32, ppowerpolicy: *mut POWER_POLICY) -> super::super::Foundation::BOOLEAN;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn RegisterPowerSettingNotification(hrecipient: super::super::Foundation::HANDLE, powersettingguid: *const ::windows_sys::core::GUID, flags: u32) -> HPOWERNOTIFY;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn RegisterSuspendResumeNotification(hrecipient: super::super::Foundation::HANDLE, flags: u32) -> HPOWERNOTIFY;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn RequestWakeupLatency(latency: LATENCY_TIME) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SetActivePwrScheme(uiid: u32, pglobalpowerpolicy: *const GLOBAL_POWER_POLICY, ppowerpolicy: *const POWER_POLICY) -> super::super::Foundation::BOOLEAN;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SetSuspendState(bhibernate: super::super::Foundation::BOOLEAN, bforce: super::super::Foundation::BOOLEAN, bwakeupeventsdisabled: super::super::Foundation::BOOLEAN) -> super::super::Foundation::BOOLEAN;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SetSystemPowerState(fsuspend: super::super::Foundation::BOOL, fforce: super::super::Foundation::BOOL) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_Power\"`*"]
    pub fn SetThreadExecutionState(esflags: EXECUTION_STATE) -> EXECUTION_STATE;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn UnregisterPowerSettingNotification(handle: HPOWERNOTIFY) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn UnregisterSuspendResumeNotification(handle: HPOWERNOTIFY) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ValidatePowerPolicies(pglobalpowerpolicy: *mut GLOBAL_POWER_POLICY, ppowerpolicy: *mut POWER_POLICY) -> super::super::Foundation::BOOLEAN;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn WriteGlobalPwrPolicy(pglobalpowerpolicy: *const GLOBAL_POWER_POLICY) -> super::super::Foundation::BOOLEAN;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn WriteProcessorPwrScheme(uiid: u32, pmachineprocessorpowerpolicy: *const MACHINE_PROCESSOR_POWER_POLICY) -> super::super::Foundation::BOOLEAN;
    #[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn WritePwrScheme(puiid: *const u32, lpszschemename: ::windows_sys::core::PCWSTR, lpszdescription: ::windows_sys::core::PCWSTR, lpscheme: *const POWER_POLICY) -> super::super::Foundation::BOOLEAN;
}
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACPI_TIME_ADJUST_DAYLIGHT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACPI_TIME_IN_DAYLIGHT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACPI_TIME_ZONE_UNKNOWN: u32 = 2047u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACTIVE_COOLING: u32 = 0u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BATTERY_CAPACITY_RELATIVE: u32 = 1073741824u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BATTERY_CHARGING: u32 = 4u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BATTERY_CLASS_MAJOR_VERSION: u32 = 1u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BATTERY_CLASS_MINOR_VERSION: u32 = 0u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BATTERY_CLASS_MINOR_VERSION_1: u32 = 1u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BATTERY_CRITICAL: u32 = 8u32;
pub const BATTERY_CYCLE_COUNT_WMI_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4019772196, data2: 20, data3: 19493, data4: [165, 11, 199, 36, 174, 92, 211, 113] };
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BATTERY_DISCHARGING: u32 = 2u32;
pub const BATTERY_FULL_CHARGED_CAPACITY_WMI_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1085539685, data2: 38647, data3: 17461, data4: [134, 148, 151, 224, 228, 57, 89, 5] };
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BATTERY_IS_SHORT_TERM: u32 = 536870912u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BATTERY_MINIPORT_UPDATE_DATA_VER_1: u32 = 1u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BATTERY_MINIPORT_UPDATE_DATA_VER_2: u32 = 2u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BATTERY_POWER_ON_LINE: u32 = 1u32;
pub const BATTERY_RUNTIME_WMI_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1398421351, data2: 6850, data3: 18876, data4: [160, 119, 63, 122, 2, 228, 10, 236] };
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BATTERY_SEALED: u32 = 268435456u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BATTERY_SET_CHARGER_ID_SUPPORTED: u32 = 8u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BATTERY_SET_CHARGE_SUPPORTED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BATTERY_SET_CHARGINGSOURCE_SUPPORTED: u32 = 4u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BATTERY_SET_DISCHARGE_SUPPORTED: u32 = 2u32;
pub const BATTERY_STATIC_DATA_WMI_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 98690147, data2: 58594, data3: 20137, data4: [128, 203, 155, 212, 179, 202, 6, 85] };
pub const BATTERY_STATUS_CHANGE_WMI_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3453984963, data2: 31835, data3: 20035, data4: [160, 52, 5, 159, 165, 184, 67, 100] };
pub const BATTERY_STATUS_WMI_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4232474833, data2: 60351, data3: 16750, data4: [135, 206, 55, 74, 78, 188, 17, 26] };
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BATTERY_SYSTEM_BATTERY: u32 = 2147483648u32;
pub const BATTERY_TAG_CHANGE_WMI_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1579118105, data2: 34694, data3: 19747, data4: [148, 252, 158, 116, 107, 213, 216, 136] };
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BATTERY_TAG_INVALID: u32 = 0u32;
pub const BATTERY_TEMPERATURE_WMI_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 441622861, data2: 44494, data3: 19012, data4: [154, 62, 200, 216, 241, 95, 242, 194] };
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BATTERY_UNKNOWN_CAPACITY: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BATTERY_UNKNOWN_CURRENT: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BATTERY_UNKNOWN_RATE: u32 = 2147483648u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BATTERY_UNKNOWN_TIME: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BATTERY_UNKNOWN_VOLTAGE: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BATTERY_USB_CHARGER_STATUS_FN_DEFAULT_USB: u32 = 1u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BATTERY_USB_CHARGER_STATUS_UCM_PD: u32 = 2u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const DEVICEPOWER_AND_OPERATION: u32 = 1073741824u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const DEVICEPOWER_CLEAR_WAKEENABLED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const DEVICEPOWER_FILTER_DEVICES_PRESENT: u32 = 536870912u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const DEVICEPOWER_FILTER_HARDWARE: u32 = 268435456u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const DEVICEPOWER_FILTER_ON_NAME: u32 = 33554432u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const DEVICEPOWER_FILTER_WAKEENABLED: u32 = 134217728u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const DEVICEPOWER_FILTER_WAKEPROGRAMMABLE: u32 = 67108864u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const DEVICEPOWER_HARDWAREID: u32 = 2147483648u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const DEVICEPOWER_SET_WAKEENABLED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const EFFECTIVE_POWER_MODE_V1: u32 = 1u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const EFFECTIVE_POWER_MODE_V2: u32 = 2u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const EMI_NAME_MAX: u32 = 16u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const EMI_VERSION_V1: u32 = 1u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const EMI_VERSION_V2: u32 = 2u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const EnableMultiBatteryDisplay: u32 = 2u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const EnablePasswordLogon: u32 = 4u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const EnableSysTrayBatteryMeter: u32 = 1u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const EnableVideoDimDisplay: u32 = 16u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const EnableWakeOnRing: u32 = 8u32;
pub const GUID_CLASS_INPUT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1293833650, data2: 61807, data3: 4559, data4: [136, 203, 0, 17, 17, 0, 0, 48] };
pub const GUID_DEVICE_ACPI_TIME: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2549718006, data2: 17559, data3: 20248, data4: [187, 34, 75, 159, 178, 251, 239, 156] };
pub const GUID_DEVICE_APPLICATIONLAUNCH_BUTTON: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1654085870, data2: 39022, data3: 19870, data4: [142, 71, 222, 39, 248, 171, 5, 77] };
pub const GUID_DEVICE_BATTERY: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1919098452, data2: 30884, data3: 4560, data4: [188, 247, 0, 170, 0, 183, 179, 42] };
pub const GUID_DEVICE_ENERGY_METER: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1170047812, data2: 32470, data3: 18895, data4: [164, 64, 194, 118, 201, 51, 176, 83] };
pub const GUID_DEVICE_FAN: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 99406141, data2: 33242, data3: 18986, data4: [138, 76, 82, 79, 35, 221, 77, 201] };
pub const GUID_DEVICE_LID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1257913682, data2: 29863, data3: 4560, data4: [190, 94, 0, 160, 201, 6, 40, 87] };
pub const GUID_DEVICE_MEMORY: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1070657597, data2: 37600, data3: 17915, data4: [183, 92, 94, 216, 255, 176, 16, 33] };
pub const GUID_DEVICE_MESSAGE_INDICATOR: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3444089701, data2: 64148, data3: 19682, data4: [162, 50, 161, 183, 100, 229, 216, 180] };
pub const GUID_DEVICE_PROCESSOR: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2549799696, data2: 20019, data3: 16558, data4: [53, 156, 139, 239, 2, 157, 189, 208] };
pub const GUID_DEVICE_SYS_BUTTON: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1257913683, data2: 29863, data3: 4560, data4: [190, 94, 0, 160, 201, 6, 40, 87] };
pub const GUID_DEVICE_THERMAL_ZONE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1257913681, data2: 29863, data3: 4560, data4: [190, 94, 0, 160, 201, 6, 40, 87] };
pub const GUID_DEVINTERFACE_THERMAL_COOLING: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3689166653, data2: 15489, data3: 16587, data4: [172, 228, 224, 229, 208, 95, 12, 159] };
pub const GUID_DEVINTERFACE_THERMAL_MANAGER: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2457780371, data2: 27044, data3: 19392, data4: [189, 2, 113, 22, 100, 113, 68, 99] };
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const IOCTL_ACPI_GET_REAL_TIME: u32 = 2703888u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const IOCTL_ACPI_SET_REAL_TIME: u32 = 2720276u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const IOCTL_BATTERY_CHARGING_SOURCE_CHANGE: u32 = 2703440u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const IOCTL_BATTERY_QUERY_INFORMATION: u32 = 2703428u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const IOCTL_BATTERY_QUERY_STATUS: u32 = 2703436u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const IOCTL_BATTERY_QUERY_TAG: u32 = 2703424u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const IOCTL_BATTERY_SET_INFORMATION: u32 = 2719816u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const IOCTL_EMI_GET_MEASUREMENT: u32 = 2244620u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const IOCTL_EMI_GET_METADATA: u32 = 2244616u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const IOCTL_EMI_GET_METADATA_SIZE: u32 = 2244612u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const IOCTL_EMI_GET_VERSION: u32 = 2244608u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const IOCTL_GET_PROCESSOR_OBJ_INFO: u32 = 2703744u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const IOCTL_GET_SYS_BUTTON_CAPS: u32 = 2703680u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const IOCTL_GET_SYS_BUTTON_EVENT: u32 = 2703684u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const IOCTL_GET_WAKE_ALARM_POLICY: u32 = 2736652u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const IOCTL_GET_WAKE_ALARM_SYSTEM_POWERSTATE: u32 = 2703896u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const IOCTL_GET_WAKE_ALARM_VALUE: u32 = 2736648u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const IOCTL_NOTIFY_SWITCH_EVENT: u32 = 2703616u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const IOCTL_QUERY_LID: u32 = 2703552u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const IOCTL_RUN_ACTIVE_COOLING_METHOD: u32 = 2719880u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const IOCTL_SET_SYS_MESSAGE_INDICATOR: u32 = 2720192u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const IOCTL_SET_WAKE_ALARM_POLICY: u32 = 2720260u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const IOCTL_SET_WAKE_ALARM_VALUE: u32 = 2720256u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const IOCTL_THERMAL_QUERY_INFORMATION: u32 = 2703488u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const IOCTL_THERMAL_READ_POLICY: u32 = 2703508u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const IOCTL_THERMAL_READ_TEMPERATURE: u32 = 2703504u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const IOCTL_THERMAL_SET_COOLING_POLICY: u32 = 2719876u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const IOCTL_THERMAL_SET_PASSIVE_LIMIT: u32 = 2719884u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const MAX_ACTIVE_COOLING_LEVELS: u32 = 10u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const MAX_BATTERY_STRING_SIZE: u32 = 128u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PASSIVE_COOLING: u32 = 1u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PDCAP_S0_SUPPORTED: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PDCAP_S1_SUPPORTED: u32 = 131072u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PDCAP_S2_SUPPORTED: u32 = 262144u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PDCAP_S3_SUPPORTED: u32 = 524288u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PDCAP_S4_SUPPORTED: u32 = 16777216u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PDCAP_S5_SUPPORTED: u32 = 33554432u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PDCAP_WAKE_FROM_S0_SUPPORTED: u32 = 1048576u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PDCAP_WAKE_FROM_S1_SUPPORTED: u32 = 2097152u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PDCAP_WAKE_FROM_S2_SUPPORTED: u32 = 4194304u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PDCAP_WAKE_FROM_S3_SUPPORTED: u32 = 8388608u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const POWER_ATTRIBUTE_HIDE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const POWER_ATTRIBUTE_SHOW_AOAC: u32 = 2u32;
#[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Devices_Properties\"`*"]
#[cfg(feature = "Win32_Devices_Properties")]
pub const PROCESSOR_NUMBER_PKEY: super::super::Devices::Properties::DEVPROPKEY = super::super::Devices::Properties::DEVPROPKEY { fmtid: ::windows_sys::core::GUID { data1: 1462028317, data2: 54703, data3: 19487, data4: [161, 3, 160, 110, 40, 242, 4, 198] }, pid: 1u32 };
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SYS_BUTTON_LID: u32 = 4u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SYS_BUTTON_LID_CHANGED: u32 = 524288u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SYS_BUTTON_LID_CLOSED: u32 = 131072u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SYS_BUTTON_LID_INITIAL: u32 = 262144u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SYS_BUTTON_LID_OPEN: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SYS_BUTTON_LID_STATE_MASK: u32 = 196608u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SYS_BUTTON_POWER: u32 = 1u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SYS_BUTTON_SLEEP: u32 = 2u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SYS_BUTTON_WAKE: u32 = 2147483648u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const THERMAL_COOLING_INTERFACE_VERSION: u32 = 1u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const THERMAL_DEVICE_INTERFACE_VERSION: u32 = 1u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const THERMAL_EVENT_VERSION: u32 = 1u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const THERMAL_POLICY_VERSION_1: u32 = 1u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const THERMAL_POLICY_VERSION_2: u32 = 2u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const TZ_ACTIVATION_REASON_CURRENT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const TZ_ACTIVATION_REASON_THERMAL: u32 = 1u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const UNKNOWN_CAPACITY: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const UNKNOWN_CURRENT: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const UNKNOWN_RATE: u32 = 2147483648u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const UNKNOWN_VOLTAGE: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub type BATTERY_CHARGING_SOURCE_TYPE = i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BatteryChargingSourceType_AC: BATTERY_CHARGING_SOURCE_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BatteryChargingSourceType_USB: BATTERY_CHARGING_SOURCE_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BatteryChargingSourceType_Wireless: BATTERY_CHARGING_SOURCE_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BatteryChargingSourceType_Max: BATTERY_CHARGING_SOURCE_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub type BATTERY_QUERY_INFORMATION_LEVEL = i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BatteryInformation: BATTERY_QUERY_INFORMATION_LEVEL = 0i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BatteryGranularityInformation: BATTERY_QUERY_INFORMATION_LEVEL = 1i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BatteryTemperature: BATTERY_QUERY_INFORMATION_LEVEL = 2i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BatteryEstimatedTime: BATTERY_QUERY_INFORMATION_LEVEL = 3i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BatteryDeviceName: BATTERY_QUERY_INFORMATION_LEVEL = 4i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BatteryManufactureDate: BATTERY_QUERY_INFORMATION_LEVEL = 5i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BatteryManufactureName: BATTERY_QUERY_INFORMATION_LEVEL = 6i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BatteryUniqueID: BATTERY_QUERY_INFORMATION_LEVEL = 7i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BatterySerialNumber: BATTERY_QUERY_INFORMATION_LEVEL = 8i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub type BATTERY_SET_INFORMATION_LEVEL = i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BatteryCriticalBias: BATTERY_SET_INFORMATION_LEVEL = 0i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BatteryCharge: BATTERY_SET_INFORMATION_LEVEL = 1i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BatteryDischarge: BATTERY_SET_INFORMATION_LEVEL = 2i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BatteryChargingSource: BATTERY_SET_INFORMATION_LEVEL = 3i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BatteryChargerId: BATTERY_SET_INFORMATION_LEVEL = 4i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BatteryChargerStatus: BATTERY_SET_INFORMATION_LEVEL = 5i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub type DEVICE_POWER_STATE = i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerDeviceUnspecified: DEVICE_POWER_STATE = 0i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerDeviceD0: DEVICE_POWER_STATE = 1i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerDeviceD1: DEVICE_POWER_STATE = 2i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerDeviceD2: DEVICE_POWER_STATE = 3i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerDeviceD3: DEVICE_POWER_STATE = 4i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerDeviceMaximum: DEVICE_POWER_STATE = 5i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub type EFFECTIVE_POWER_MODE = i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const EffectivePowerModeBatterySaver: EFFECTIVE_POWER_MODE = 0i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const EffectivePowerModeBetterBattery: EFFECTIVE_POWER_MODE = 1i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const EffectivePowerModeBalanced: EFFECTIVE_POWER_MODE = 2i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const EffectivePowerModeHighPerformance: EFFECTIVE_POWER_MODE = 3i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const EffectivePowerModeMaxPerformance: EFFECTIVE_POWER_MODE = 4i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const EffectivePowerModeGameMode: EFFECTIVE_POWER_MODE = 5i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const EffectivePowerModeMixedReality: EFFECTIVE_POWER_MODE = 6i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub type EMI_MEASUREMENT_UNIT = i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const EmiMeasurementUnitPicowattHours: EMI_MEASUREMENT_UNIT = 0i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub type EXECUTION_STATE = u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ES_AWAYMODE_REQUIRED: EXECUTION_STATE = 64u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ES_CONTINUOUS: EXECUTION_STATE = 2147483648u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ES_DISPLAY_REQUIRED: EXECUTION_STATE = 2u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ES_SYSTEM_REQUIRED: EXECUTION_STATE = 1u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ES_USER_PRESENT: EXECUTION_STATE = 4u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub type LATENCY_TIME = i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const LT_DONT_CARE: LATENCY_TIME = 0i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const LT_LOWEST_LATENCY: LATENCY_TIME = 1i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub type POWER_ACTION = i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerActionNone: POWER_ACTION = 0i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerActionReserved: POWER_ACTION = 1i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerActionSleep: POWER_ACTION = 2i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerActionHibernate: POWER_ACTION = 3i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerActionShutdown: POWER_ACTION = 4i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerActionShutdownReset: POWER_ACTION = 5i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerActionShutdownOff: POWER_ACTION = 6i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerActionWarmEject: POWER_ACTION = 7i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerActionDisplayOff: POWER_ACTION = 8i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub type POWER_ACTION_POLICY_EVENT_CODE = u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const POWER_FORCE_TRIGGER_RESET: POWER_ACTION_POLICY_EVENT_CODE = 2147483648u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const POWER_LEVEL_USER_NOTIFY_EXEC: POWER_ACTION_POLICY_EVENT_CODE = 4u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const POWER_LEVEL_USER_NOTIFY_SOUND: POWER_ACTION_POLICY_EVENT_CODE = 2u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const POWER_LEVEL_USER_NOTIFY_TEXT: POWER_ACTION_POLICY_EVENT_CODE = 1u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const POWER_USER_NOTIFY_BUTTON: POWER_ACTION_POLICY_EVENT_CODE = 8u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const POWER_USER_NOTIFY_SHUTDOWN: POWER_ACTION_POLICY_EVENT_CODE = 16u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub type POWER_COOLING_MODE = u16;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PO_TZ_ACTIVE: POWER_COOLING_MODE = 0u16;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PO_TZ_PASSIVE: POWER_COOLING_MODE = 1u16;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PO_TZ_INVALID_MODE: POWER_COOLING_MODE = 2u16;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub type POWER_DATA_ACCESSOR = i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACCESS_AC_POWER_SETTING_INDEX: POWER_DATA_ACCESSOR = 0i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACCESS_DC_POWER_SETTING_INDEX: POWER_DATA_ACCESSOR = 1i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACCESS_FRIENDLY_NAME: POWER_DATA_ACCESSOR = 2i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACCESS_DESCRIPTION: POWER_DATA_ACCESSOR = 3i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACCESS_POSSIBLE_POWER_SETTING: POWER_DATA_ACCESSOR = 4i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACCESS_POSSIBLE_POWER_SETTING_FRIENDLY_NAME: POWER_DATA_ACCESSOR = 5i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACCESS_POSSIBLE_POWER_SETTING_DESCRIPTION: POWER_DATA_ACCESSOR = 6i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACCESS_DEFAULT_AC_POWER_SETTING: POWER_DATA_ACCESSOR = 7i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACCESS_DEFAULT_DC_POWER_SETTING: POWER_DATA_ACCESSOR = 8i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACCESS_POSSIBLE_VALUE_MIN: POWER_DATA_ACCESSOR = 9i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACCESS_POSSIBLE_VALUE_MAX: POWER_DATA_ACCESSOR = 10i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACCESS_POSSIBLE_VALUE_INCREMENT: POWER_DATA_ACCESSOR = 11i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACCESS_POSSIBLE_VALUE_UNITS: POWER_DATA_ACCESSOR = 12i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACCESS_ICON_RESOURCE: POWER_DATA_ACCESSOR = 13i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACCESS_DEFAULT_SECURITY_DESCRIPTOR: POWER_DATA_ACCESSOR = 14i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACCESS_ATTRIBUTES: POWER_DATA_ACCESSOR = 15i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACCESS_SCHEME: POWER_DATA_ACCESSOR = 16i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACCESS_SUBGROUP: POWER_DATA_ACCESSOR = 17i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACCESS_INDIVIDUAL_SETTING: POWER_DATA_ACCESSOR = 18i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACCESS_ACTIVE_SCHEME: POWER_DATA_ACCESSOR = 19i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACCESS_CREATE_SCHEME: POWER_DATA_ACCESSOR = 20i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACCESS_AC_POWER_SETTING_MAX: POWER_DATA_ACCESSOR = 21i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACCESS_DC_POWER_SETTING_MAX: POWER_DATA_ACCESSOR = 22i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACCESS_AC_POWER_SETTING_MIN: POWER_DATA_ACCESSOR = 23i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACCESS_DC_POWER_SETTING_MIN: POWER_DATA_ACCESSOR = 24i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACCESS_PROFILE: POWER_DATA_ACCESSOR = 25i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACCESS_OVERLAY_SCHEME: POWER_DATA_ACCESSOR = 26i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ACCESS_ACTIVE_OVERLAY_SCHEME: POWER_DATA_ACCESSOR = 27i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub type POWER_INFORMATION_LEVEL = i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SystemPowerPolicyAc: POWER_INFORMATION_LEVEL = 0i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SystemPowerPolicyDc: POWER_INFORMATION_LEVEL = 1i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const VerifySystemPolicyAc: POWER_INFORMATION_LEVEL = 2i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const VerifySystemPolicyDc: POWER_INFORMATION_LEVEL = 3i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SystemPowerCapabilities: POWER_INFORMATION_LEVEL = 4i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SystemBatteryState: POWER_INFORMATION_LEVEL = 5i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SystemPowerStateHandler: POWER_INFORMATION_LEVEL = 6i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ProcessorStateHandler: POWER_INFORMATION_LEVEL = 7i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SystemPowerPolicyCurrent: POWER_INFORMATION_LEVEL = 8i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const AdministratorPowerPolicy: POWER_INFORMATION_LEVEL = 9i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SystemReserveHiberFile: POWER_INFORMATION_LEVEL = 10i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ProcessorInformation: POWER_INFORMATION_LEVEL = 11i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SystemPowerInformation: POWER_INFORMATION_LEVEL = 12i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ProcessorStateHandler2: POWER_INFORMATION_LEVEL = 13i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const LastWakeTime: POWER_INFORMATION_LEVEL = 14i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const LastSleepTime: POWER_INFORMATION_LEVEL = 15i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SystemExecutionState: POWER_INFORMATION_LEVEL = 16i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SystemPowerStateNotifyHandler: POWER_INFORMATION_LEVEL = 17i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ProcessorPowerPolicyAc: POWER_INFORMATION_LEVEL = 18i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ProcessorPowerPolicyDc: POWER_INFORMATION_LEVEL = 19i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const VerifyProcessorPowerPolicyAc: POWER_INFORMATION_LEVEL = 20i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const VerifyProcessorPowerPolicyDc: POWER_INFORMATION_LEVEL = 21i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ProcessorPowerPolicyCurrent: POWER_INFORMATION_LEVEL = 22i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SystemPowerStateLogging: POWER_INFORMATION_LEVEL = 23i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SystemPowerLoggingEntry: POWER_INFORMATION_LEVEL = 24i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SetPowerSettingValue: POWER_INFORMATION_LEVEL = 25i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const NotifyUserPowerSetting: POWER_INFORMATION_LEVEL = 26i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerInformationLevelUnused0: POWER_INFORMATION_LEVEL = 27i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SystemMonitorHiberBootPowerOff: POWER_INFORMATION_LEVEL = 28i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SystemVideoState: POWER_INFORMATION_LEVEL = 29i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const TraceApplicationPowerMessage: POWER_INFORMATION_LEVEL = 30i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const TraceApplicationPowerMessageEnd: POWER_INFORMATION_LEVEL = 31i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ProcessorPerfStates: POWER_INFORMATION_LEVEL = 32i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ProcessorIdleStates: POWER_INFORMATION_LEVEL = 33i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ProcessorCap: POWER_INFORMATION_LEVEL = 34i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SystemWakeSource: POWER_INFORMATION_LEVEL = 35i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SystemHiberFileInformation: POWER_INFORMATION_LEVEL = 36i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const TraceServicePowerMessage: POWER_INFORMATION_LEVEL = 37i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ProcessorLoad: POWER_INFORMATION_LEVEL = 38i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerShutdownNotification: POWER_INFORMATION_LEVEL = 39i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const MonitorCapabilities: POWER_INFORMATION_LEVEL = 40i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SessionPowerInit: POWER_INFORMATION_LEVEL = 41i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SessionDisplayState: POWER_INFORMATION_LEVEL = 42i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerRequestCreate: POWER_INFORMATION_LEVEL = 43i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerRequestAction: POWER_INFORMATION_LEVEL = 44i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const GetPowerRequestList: POWER_INFORMATION_LEVEL = 45i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ProcessorInformationEx: POWER_INFORMATION_LEVEL = 46i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const NotifyUserModeLegacyPowerEvent: POWER_INFORMATION_LEVEL = 47i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const GroupPark: POWER_INFORMATION_LEVEL = 48i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ProcessorIdleDomains: POWER_INFORMATION_LEVEL = 49i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const WakeTimerList: POWER_INFORMATION_LEVEL = 50i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SystemHiberFileSize: POWER_INFORMATION_LEVEL = 51i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ProcessorIdleStatesHv: POWER_INFORMATION_LEVEL = 52i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ProcessorPerfStatesHv: POWER_INFORMATION_LEVEL = 53i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ProcessorPerfCapHv: POWER_INFORMATION_LEVEL = 54i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ProcessorSetIdle: POWER_INFORMATION_LEVEL = 55i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const LogicalProcessorIdling: POWER_INFORMATION_LEVEL = 56i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const UserPresence: POWER_INFORMATION_LEVEL = 57i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerSettingNotificationName: POWER_INFORMATION_LEVEL = 58i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const GetPowerSettingValue: POWER_INFORMATION_LEVEL = 59i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const IdleResiliency: POWER_INFORMATION_LEVEL = 60i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SessionRITState: POWER_INFORMATION_LEVEL = 61i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SessionConnectNotification: POWER_INFORMATION_LEVEL = 62i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SessionPowerCleanup: POWER_INFORMATION_LEVEL = 63i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SessionLockState: POWER_INFORMATION_LEVEL = 64i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SystemHiberbootState: POWER_INFORMATION_LEVEL = 65i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PlatformInformation: POWER_INFORMATION_LEVEL = 66i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PdcInvocation: POWER_INFORMATION_LEVEL = 67i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const MonitorInvocation: POWER_INFORMATION_LEVEL = 68i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const FirmwareTableInformationRegistered: POWER_INFORMATION_LEVEL = 69i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SetShutdownSelectedTime: POWER_INFORMATION_LEVEL = 70i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SuspendResumeInvocation: POWER_INFORMATION_LEVEL = 71i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PlmPowerRequestCreate: POWER_INFORMATION_LEVEL = 72i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ScreenOff: POWER_INFORMATION_LEVEL = 73i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const CsDeviceNotification: POWER_INFORMATION_LEVEL = 74i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PlatformRole: POWER_INFORMATION_LEVEL = 75i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const LastResumePerformance: POWER_INFORMATION_LEVEL = 76i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const DisplayBurst: POWER_INFORMATION_LEVEL = 77i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ExitLatencySamplingPercentage: POWER_INFORMATION_LEVEL = 78i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const RegisterSpmPowerSettings: POWER_INFORMATION_LEVEL = 79i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PlatformIdleStates: POWER_INFORMATION_LEVEL = 80i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ProcessorIdleVeto: POWER_INFORMATION_LEVEL = 81i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PlatformIdleVeto: POWER_INFORMATION_LEVEL = 82i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SystemBatteryStatePrecise: POWER_INFORMATION_LEVEL = 83i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ThermalEvent: POWER_INFORMATION_LEVEL = 84i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerRequestActionInternal: POWER_INFORMATION_LEVEL = 85i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const BatteryDeviceState: POWER_INFORMATION_LEVEL = 86i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerInformationInternal: POWER_INFORMATION_LEVEL = 87i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const ThermalStandby: POWER_INFORMATION_LEVEL = 88i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SystemHiberFileType: POWER_INFORMATION_LEVEL = 89i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PhysicalPowerButtonPress: POWER_INFORMATION_LEVEL = 90i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const QueryPotentialDripsConstraint: POWER_INFORMATION_LEVEL = 91i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const EnergyTrackerCreate: POWER_INFORMATION_LEVEL = 92i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const EnergyTrackerQuery: POWER_INFORMATION_LEVEL = 93i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const UpdateBlackBoxRecorder: POWER_INFORMATION_LEVEL = 94i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SessionAllowExternalDmaDevices: POWER_INFORMATION_LEVEL = 95i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const SendSuspendResumeNotification: POWER_INFORMATION_LEVEL = 96i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerInformationLevelMaximum: POWER_INFORMATION_LEVEL = 97i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub type POWER_PLATFORM_ROLE = i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PlatformRoleUnspecified: POWER_PLATFORM_ROLE = 0i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PlatformRoleDesktop: POWER_PLATFORM_ROLE = 1i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PlatformRoleMobile: POWER_PLATFORM_ROLE = 2i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PlatformRoleWorkstation: POWER_PLATFORM_ROLE = 3i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PlatformRoleEnterpriseServer: POWER_PLATFORM_ROLE = 4i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PlatformRoleSOHOServer: POWER_PLATFORM_ROLE = 5i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PlatformRoleAppliancePC: POWER_PLATFORM_ROLE = 6i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PlatformRolePerformanceServer: POWER_PLATFORM_ROLE = 7i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PlatformRoleSlate: POWER_PLATFORM_ROLE = 8i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PlatformRoleMaximum: POWER_PLATFORM_ROLE = 9i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub type POWER_PLATFORM_ROLE_VERSION = u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const POWER_PLATFORM_ROLE_V1: POWER_PLATFORM_ROLE_VERSION = 1u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const POWER_PLATFORM_ROLE_V2: POWER_PLATFORM_ROLE_VERSION = 2u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub type POWER_REQUEST_TYPE = i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerRequestDisplayRequired: POWER_REQUEST_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerRequestSystemRequired: POWER_REQUEST_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerRequestAwayModeRequired: POWER_REQUEST_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerRequestExecutionRequired: POWER_REQUEST_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub type POWER_SETTING_REGISTER_NOTIFICATION_FLAGS = u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const DEVICE_NOTIFY_SERVICE_HANDLE: POWER_SETTING_REGISTER_NOTIFICATION_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const DEVICE_NOTIFY_CALLBACK: POWER_SETTING_REGISTER_NOTIFICATION_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const DEVICE_NOTIFY_WINDOW_HANDLE: POWER_SETTING_REGISTER_NOTIFICATION_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub type SYSTEM_POWER_CONDITION = i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PoAc: SYSTEM_POWER_CONDITION = 0i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PoDc: SYSTEM_POWER_CONDITION = 1i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PoHot: SYSTEM_POWER_CONDITION = 2i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PoConditionMaximum: SYSTEM_POWER_CONDITION = 3i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub type SYSTEM_POWER_STATE = i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerSystemUnspecified: SYSTEM_POWER_STATE = 0i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerSystemWorking: SYSTEM_POWER_STATE = 1i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerSystemSleeping1: SYSTEM_POWER_STATE = 2i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerSystemSleeping2: SYSTEM_POWER_STATE = 3i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerSystemSleeping3: SYSTEM_POWER_STATE = 4i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerSystemHibernate: SYSTEM_POWER_STATE = 5i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerSystemShutdown: SYSTEM_POWER_STATE = 6i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const PowerSystemMaximum: SYSTEM_POWER_STATE = 7i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub type USB_CHARGER_PORT = i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const UsbChargerPort_Legacy: USB_CHARGER_PORT = 0i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const UsbChargerPort_TypeC: USB_CHARGER_PORT = 1i32;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub const UsbChargerPort_Max: USB_CHARGER_PORT = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct ACPI_REAL_TIME {
    pub Year: u16,
    pub Month: u8,
    pub Day: u8,
    pub Hour: u8,
    pub Minute: u8,
    pub Second: u8,
    pub Valid: u8,
    pub Milliseconds: u16,
    pub TimeZone: i16,
    pub DayLight: u8,
    pub Reserved1: [u8; 3],
}
impl ::core::marker::Copy for ACPI_REAL_TIME {}
impl ::core::clone::Clone for ACPI_REAL_TIME {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct ADMINISTRATOR_POWER_POLICY {
    pub MinSleep: SYSTEM_POWER_STATE,
    pub MaxSleep: SYSTEM_POWER_STATE,
    pub MinVideoTimeout: u32,
    pub MaxVideoTimeout: u32,
    pub MinSpindownTimeout: u32,
    pub MaxSpindownTimeout: u32,
}
impl ::core::marker::Copy for ADMINISTRATOR_POWER_POLICY {}
impl ::core::clone::Clone for ADMINISTRATOR_POWER_POLICY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct BATTERY_CHARGER_STATUS {
    pub Type: BATTERY_CHARGING_SOURCE_TYPE,
    pub VaData: [u32; 1],
}
impl ::core::marker::Copy for BATTERY_CHARGER_STATUS {}
impl ::core::clone::Clone for BATTERY_CHARGER_STATUS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct BATTERY_CHARGING_SOURCE {
    pub Type: BATTERY_CHARGING_SOURCE_TYPE,
    pub MaxCurrent: u32,
}
impl ::core::marker::Copy for BATTERY_CHARGING_SOURCE {}
impl ::core::clone::Clone for BATTERY_CHARGING_SOURCE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct BATTERY_CHARGING_SOURCE_INFORMATION {
    pub Type: BATTERY_CHARGING_SOURCE_TYPE,
    pub SourceOnline: super::super::Foundation::BOOLEAN,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for BATTERY_CHARGING_SOURCE_INFORMATION {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for BATTERY_CHARGING_SOURCE_INFORMATION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct BATTERY_INFORMATION {
    pub Capabilities: u32,
    pub Technology: u8,
    pub Reserved: [u8; 3],
    pub Chemistry: [u8; 4],
    pub DesignedCapacity: u32,
    pub FullChargedCapacity: u32,
    pub DefaultAlert1: u32,
    pub DefaultAlert2: u32,
    pub CriticalBias: u32,
    pub CycleCount: u32,
}
impl ::core::marker::Copy for BATTERY_INFORMATION {}
impl ::core::clone::Clone for BATTERY_INFORMATION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct BATTERY_MANUFACTURE_DATE {
    pub Day: u8,
    pub Month: u8,
    pub Year: u16,
}
impl ::core::marker::Copy for BATTERY_MANUFACTURE_DATE {}
impl ::core::clone::Clone for BATTERY_MANUFACTURE_DATE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct BATTERY_QUERY_INFORMATION {
    pub BatteryTag: u32,
    pub InformationLevel: BATTERY_QUERY_INFORMATION_LEVEL,
    pub AtRate: u32,
}
impl ::core::marker::Copy for BATTERY_QUERY_INFORMATION {}
impl ::core::clone::Clone for BATTERY_QUERY_INFORMATION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct BATTERY_REPORTING_SCALE {
    pub Granularity: u32,
    pub Capacity: u32,
}
impl ::core::marker::Copy for BATTERY_REPORTING_SCALE {}
impl ::core::clone::Clone for BATTERY_REPORTING_SCALE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct BATTERY_SET_INFORMATION {
    pub BatteryTag: u32,
    pub InformationLevel: BATTERY_SET_INFORMATION_LEVEL,
    pub Buffer: [u8; 1],
}
impl ::core::marker::Copy for BATTERY_SET_INFORMATION {}
impl ::core::clone::Clone for BATTERY_SET_INFORMATION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct BATTERY_STATUS {
    pub PowerState: u32,
    pub Capacity: u32,
    pub Voltage: u32,
    pub Rate: i32,
}
impl ::core::marker::Copy for BATTERY_STATUS {}
impl ::core::clone::Clone for BATTERY_STATUS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct BATTERY_USB_CHARGER_STATUS {
    pub Type: BATTERY_CHARGING_SOURCE_TYPE,
    pub Reserved: u32,
    pub Flags: u32,
    pub MaxCurrent: u32,
    pub Voltage: u32,
    pub PortType: USB_CHARGER_PORT,
    pub PortId: u64,
    pub PowerSourceInformation: *mut ::core::ffi::c_void,
    pub OemCharger: ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for BATTERY_USB_CHARGER_STATUS {}
impl ::core::clone::Clone for BATTERY_USB_CHARGER_STATUS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct BATTERY_WAIT_STATUS {
    pub BatteryTag: u32,
    pub Timeout: u32,
    pub PowerState: u32,
    pub LowCapacity: u32,
    pub HighCapacity: u32,
}
impl ::core::marker::Copy for BATTERY_WAIT_STATUS {}
impl ::core::clone::Clone for BATTERY_WAIT_STATUS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct CM_POWER_DATA {
    pub PD_Size: u32,
    pub PD_MostRecentPowerState: DEVICE_POWER_STATE,
    pub PD_Capabilities: u32,
    pub PD_D1Latency: u32,
    pub PD_D2Latency: u32,
    pub PD_D3Latency: u32,
    pub PD_PowerStateMapping: [DEVICE_POWER_STATE; 7],
    pub PD_DeepestSystemWake: SYSTEM_POWER_STATE,
}
impl ::core::marker::Copy for CM_POWER_DATA {}
impl ::core::clone::Clone for CM_POWER_DATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS {
    pub Callback: PDEVICE_NOTIFY_CALLBACK_ROUTINE,
    pub Context: *mut ::core::ffi::c_void,
}
impl ::core::marker::Copy for DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS {}
impl ::core::clone::Clone for DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct EMI_CHANNEL_MEASUREMENT_DATA {
    pub AbsoluteEnergy: u64,
    pub AbsoluteTime: u64,
}
impl ::core::marker::Copy for EMI_CHANNEL_MEASUREMENT_DATA {}
impl ::core::clone::Clone for EMI_CHANNEL_MEASUREMENT_DATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct EMI_CHANNEL_V2 {
    pub MeasurementUnit: EMI_MEASUREMENT_UNIT,
    pub ChannelNameSize: u16,
    pub ChannelName: [u16; 1],
}
impl ::core::marker::Copy for EMI_CHANNEL_V2 {}
impl ::core::clone::Clone for EMI_CHANNEL_V2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct EMI_MEASUREMENT_DATA_V2 {
    pub ChannelData: [EMI_CHANNEL_MEASUREMENT_DATA; 1],
}
impl ::core::marker::Copy for EMI_MEASUREMENT_DATA_V2 {}
impl ::core::clone::Clone for EMI_MEASUREMENT_DATA_V2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct EMI_METADATA_SIZE {
    pub MetadataSize: u32,
}
impl ::core::marker::Copy for EMI_METADATA_SIZE {}
impl ::core::clone::Clone for EMI_METADATA_SIZE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct EMI_METADATA_V1 {
    pub MeasurementUnit: EMI_MEASUREMENT_UNIT,
    pub HardwareOEM: [u16; 16],
    pub HardwareModel: [u16; 16],
    pub HardwareRevision: u16,
    pub MeteredHardwareNameSize: u16,
    pub MeteredHardwareName: [u16; 1],
}
impl ::core::marker::Copy for EMI_METADATA_V1 {}
impl ::core::clone::Clone for EMI_METADATA_V1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct EMI_METADATA_V2 {
    pub HardwareOEM: [u16; 16],
    pub HardwareModel: [u16; 16],
    pub HardwareRevision: u16,
    pub ChannelCount: u16,
    pub Channels: [EMI_CHANNEL_V2; 1],
}
impl ::core::marker::Copy for EMI_METADATA_V2 {}
impl ::core::clone::Clone for EMI_METADATA_V2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct EMI_VERSION {
    pub EmiVersion: u16,
}
impl ::core::marker::Copy for EMI_VERSION {}
impl ::core::clone::Clone for EMI_VERSION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct GLOBAL_MACHINE_POWER_POLICY {
    pub Revision: u32,
    pub LidOpenWakeAc: SYSTEM_POWER_STATE,
    pub LidOpenWakeDc: SYSTEM_POWER_STATE,
    pub BroadcastCapacityResolution: u32,
}
impl ::core::marker::Copy for GLOBAL_MACHINE_POWER_POLICY {}
impl ::core::clone::Clone for GLOBAL_MACHINE_POWER_POLICY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct GLOBAL_POWER_POLICY {
    pub user: GLOBAL_USER_POWER_POLICY,
    pub mach: GLOBAL_MACHINE_POWER_POLICY,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for GLOBAL_POWER_POLICY {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for GLOBAL_POWER_POLICY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct GLOBAL_USER_POWER_POLICY {
    pub Revision: u32,
    pub PowerButtonAc: POWER_ACTION_POLICY,
    pub PowerButtonDc: POWER_ACTION_POLICY,
    pub SleepButtonAc: POWER_ACTION_POLICY,
    pub SleepButtonDc: POWER_ACTION_POLICY,
    pub LidCloseAc: POWER_ACTION_POLICY,
    pub LidCloseDc: POWER_ACTION_POLICY,
    pub DischargePolicy: [SYSTEM_POWER_LEVEL; 4],
    pub GlobalFlags: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for GLOBAL_USER_POWER_POLICY {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for GLOBAL_USER_POWER_POLICY {
    fn clone(&self) -> Self {
        *self
    }
}
pub type HPOWERNOTIFY = isize;
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct MACHINE_POWER_POLICY {
    pub Revision: u32,
    pub MinSleepAc: SYSTEM_POWER_STATE,
    pub MinSleepDc: SYSTEM_POWER_STATE,
    pub ReducedLatencySleepAc: SYSTEM_POWER_STATE,
    pub ReducedLatencySleepDc: SYSTEM_POWER_STATE,
    pub DozeTimeoutAc: u32,
    pub DozeTimeoutDc: u32,
    pub DozeS4TimeoutAc: u32,
    pub DozeS4TimeoutDc: u32,
    pub MinThrottleAc: u8,
    pub MinThrottleDc: u8,
    pub pad1: [u8; 2],
    pub OverThrottledAc: POWER_ACTION_POLICY,
    pub OverThrottledDc: POWER_ACTION_POLICY,
}
impl ::core::marker::Copy for MACHINE_POWER_POLICY {}
impl ::core::clone::Clone for MACHINE_POWER_POLICY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct MACHINE_PROCESSOR_POWER_POLICY {
    pub Revision: u32,
    pub ProcessorPolicyAc: PROCESSOR_POWER_POLICY,
    pub ProcessorPolicyDc: PROCESSOR_POWER_POLICY,
}
impl ::core::marker::Copy for MACHINE_PROCESSOR_POWER_POLICY {}
impl ::core::clone::Clone for MACHINE_PROCESSOR_POWER_POLICY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct POWERBROADCAST_SETTING {
    pub PowerSetting: ::windows_sys::core::GUID,
    pub DataLength: u32,
    pub Data: [u8; 1],
}
impl ::core::marker::Copy for POWERBROADCAST_SETTING {}
impl ::core::clone::Clone for POWERBROADCAST_SETTING {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct POWER_ACTION_POLICY {
    pub Action: POWER_ACTION,
    pub Flags: u32,
    pub EventCode: POWER_ACTION_POLICY_EVENT_CODE,
}
impl ::core::marker::Copy for POWER_ACTION_POLICY {}
impl ::core::clone::Clone for POWER_ACTION_POLICY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct POWER_POLICY {
    pub user: USER_POWER_POLICY,
    pub mach: MACHINE_POWER_POLICY,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for POWER_POLICY {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for POWER_POLICY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct PROCESSOR_OBJECT_INFO {
    pub PhysicalID: u32,
    pub PBlkAddress: u32,
    pub PBlkLength: u8,
}
impl ::core::marker::Copy for PROCESSOR_OBJECT_INFO {}
impl ::core::clone::Clone for PROCESSOR_OBJECT_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct PROCESSOR_OBJECT_INFO_EX {
    pub PhysicalID: u32,
    pub PBlkAddress: u32,
    pub PBlkLength: u8,
    pub InitialApicId: u32,
}
impl ::core::marker::Copy for PROCESSOR_OBJECT_INFO_EX {}
impl ::core::clone::Clone for PROCESSOR_OBJECT_INFO_EX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct PROCESSOR_POWER_INFORMATION {
    pub Number: u64,
    pub MaxMhz: u64,
    pub CurrentMhz: u64,
    pub MhzLimit: u64,
    pub MaxIdleState: u64,
    pub CurrentIdleState: u64,
}
impl ::core::marker::Copy for PROCESSOR_POWER_INFORMATION {}
impl ::core::clone::Clone for PROCESSOR_POWER_INFORMATION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct PROCESSOR_POWER_POLICY {
    pub Revision: u32,
    pub DynamicThrottle: u8,
    pub Spare: [u8; 3],
    pub _bitfield: u32,
    pub PolicyCount: u32,
    pub Policy: [PROCESSOR_POWER_POLICY_INFO; 3],
}
impl ::core::marker::Copy for PROCESSOR_POWER_POLICY {}
impl ::core::clone::Clone for PROCESSOR_POWER_POLICY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct PROCESSOR_POWER_POLICY_INFO {
    pub TimeCheck: u32,
    pub DemoteLimit: u32,
    pub PromoteLimit: u32,
    pub DemotePercent: u8,
    pub PromotePercent: u8,
    pub Spare: [u8; 2],
    pub _bitfield: u32,
}
impl ::core::marker::Copy for PROCESSOR_POWER_POLICY_INFO {}
impl ::core::clone::Clone for PROCESSOR_POWER_POLICY_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct SET_POWER_SETTING_VALUE {
    pub Version: u32,
    pub Guid: ::windows_sys::core::GUID,
    pub PowerCondition: SYSTEM_POWER_CONDITION,
    pub DataLength: u32,
    pub Data: [u8; 1],
}
impl ::core::marker::Copy for SET_POWER_SETTING_VALUE {}
impl ::core::clone::Clone for SET_POWER_SETTING_VALUE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct SYSTEM_BATTERY_STATE {
    pub AcOnLine: super::super::Foundation::BOOLEAN,
    pub BatteryPresent: super::super::Foundation::BOOLEAN,
    pub Charging: super::super::Foundation::BOOLEAN,
    pub Discharging: super::super::Foundation::BOOLEAN,
    pub Spare1: [super::super::Foundation::BOOLEAN; 3],
    pub Tag: u8,
    pub MaxCapacity: u32,
    pub RemainingCapacity: u32,
    pub Rate: u32,
    pub EstimatedTime: u32,
    pub DefaultAlert1: u32,
    pub DefaultAlert2: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for SYSTEM_BATTERY_STATE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for SYSTEM_BATTERY_STATE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct SYSTEM_POWER_CAPABILITIES {
    pub PowerButtonPresent: super::super::Foundation::BOOLEAN,
    pub SleepButtonPresent: super::super::Foundation::BOOLEAN,
    pub LidPresent: super::super::Foundation::BOOLEAN,
    pub SystemS1: super::super::Foundation::BOOLEAN,
    pub SystemS2: super::super::Foundation::BOOLEAN,
    pub SystemS3: super::super::Foundation::BOOLEAN,
    pub SystemS4: super::super::Foundation::BOOLEAN,
    pub SystemS5: super::super::Foundation::BOOLEAN,
    pub HiberFilePresent: super::super::Foundation::BOOLEAN,
    pub FullWake: super::super::Foundation::BOOLEAN,
    pub VideoDimPresent: super::super::Foundation::BOOLEAN,
    pub ApmPresent: super::super::Foundation::BOOLEAN,
    pub UpsPresent: super::super::Foundation::BOOLEAN,
    pub ThermalControl: super::super::Foundation::BOOLEAN,
    pub ProcessorThrottle: super::super::Foundation::BOOLEAN,
    pub ProcessorMinThrottle: u8,
    pub ProcessorMaxThrottle: u8,
    pub FastSystemS4: super::super::Foundation::BOOLEAN,
    pub Hiberboot: super::super::Foundation::BOOLEAN,
    pub WakeAlarmPresent: super::super::Foundation::BOOLEAN,
    pub AoAc: super::super::Foundation::BOOLEAN,
    pub DiskSpinDown: super::super::Foundation::BOOLEAN,
    pub HiberFileType: u8,
    pub AoAcConnectivitySupported: super::super::Foundation::BOOLEAN,
    pub spare3: [u8; 6],
    pub SystemBatteriesPresent: super::super::Foundation::BOOLEAN,
    pub BatteriesAreShortTerm: super::super::Foundation::BOOLEAN,
    pub BatteryScale: [BATTERY_REPORTING_SCALE; 3],
    pub AcOnLineWake: SYSTEM_POWER_STATE,
    pub SoftLidWake: SYSTEM_POWER_STATE,
    pub RtcWake: SYSTEM_POWER_STATE,
    pub MinDeviceWakeState: SYSTEM_POWER_STATE,
    pub DefaultLowLatencyWake: SYSTEM_POWER_STATE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for SYSTEM_POWER_CAPABILITIES {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for SYSTEM_POWER_CAPABILITIES {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct SYSTEM_POWER_INFORMATION {
    pub MaxIdlenessAllowed: u64,
    pub Idleness: u64,
    pub TimeRemaining: u64,
    pub CoolingMode: POWER_COOLING_MODE,
}
impl ::core::marker::Copy for SYSTEM_POWER_INFORMATION {}
impl ::core::clone::Clone for SYSTEM_POWER_INFORMATION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct SYSTEM_POWER_LEVEL {
    pub Enable: super::super::Foundation::BOOLEAN,
    pub Spare: [u8; 3],
    pub BatteryLevel: u32,
    pub PowerPolicy: POWER_ACTION_POLICY,
    pub MinSystemState: SYSTEM_POWER_STATE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for SYSTEM_POWER_LEVEL {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for SYSTEM_POWER_LEVEL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct SYSTEM_POWER_POLICY {
    pub Revision: u32,
    pub PowerButton: POWER_ACTION_POLICY,
    pub SleepButton: POWER_ACTION_POLICY,
    pub LidClose: POWER_ACTION_POLICY,
    pub LidOpenWake: SYSTEM_POWER_STATE,
    pub Reserved: u32,
    pub Idle: POWER_ACTION_POLICY,
    pub IdleTimeout: u32,
    pub IdleSensitivity: u8,
    pub DynamicThrottle: u8,
    pub Spare2: [u8; 2],
    pub MinSleep: SYSTEM_POWER_STATE,
    pub MaxSleep: SYSTEM_POWER_STATE,
    pub ReducedLatencySleep: SYSTEM_POWER_STATE,
    pub WinLogonFlags: u32,
    pub Spare3: u32,
    pub DozeS4Timeout: u32,
    pub BroadcastCapacityResolution: u32,
    pub DischargePolicy: [SYSTEM_POWER_LEVEL; 4],
    pub VideoTimeout: u32,
    pub VideoDimDisplay: super::super::Foundation::BOOLEAN,
    pub VideoReserved: [u32; 3],
    pub SpindownTimeout: u32,
    pub OptimizeForPower: super::super::Foundation::BOOLEAN,
    pub FanThrottleTolerance: u8,
    pub ForcedThrottle: u8,
    pub MinThrottle: u8,
    pub OverThrottled: POWER_ACTION_POLICY,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for SYSTEM_POWER_POLICY {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for SYSTEM_POWER_POLICY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct SYSTEM_POWER_STATUS {
    pub ACLineStatus: u8,
    pub BatteryFlag: u8,
    pub BatteryLifePercent: u8,
    pub SystemStatusFlag: u8,
    pub BatteryLifeTime: u32,
    pub BatteryFullLifeTime: u32,
}
impl ::core::marker::Copy for SYSTEM_POWER_STATUS {}
impl ::core::clone::Clone for SYSTEM_POWER_STATUS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct THERMAL_EVENT {
    pub Version: u32,
    pub Size: u32,
    pub Type: u32,
    pub Temperature: u32,
    pub TripPointTemperature: u32,
    pub Initiator: ::windows_sys::core::PWSTR,
}
impl ::core::marker::Copy for THERMAL_EVENT {}
impl ::core::clone::Clone for THERMAL_EVENT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct THERMAL_INFORMATION {
    pub ThermalStamp: u32,
    pub ThermalConstant1: u32,
    pub ThermalConstant2: u32,
    pub Processors: usize,
    pub SamplingPeriod: u32,
    pub CurrentTemperature: u32,
    pub PassiveTripPoint: u32,
    pub CriticalTripPoint: u32,
    pub ActiveTripPointCount: u8,
    pub ActiveTripPoint: [u32; 10],
}
impl ::core::marker::Copy for THERMAL_INFORMATION {}
impl ::core::clone::Clone for THERMAL_INFORMATION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct THERMAL_POLICY {
    pub Version: u32,
    pub WaitForUpdate: super::super::Foundation::BOOLEAN,
    pub Hibernate: super::super::Foundation::BOOLEAN,
    pub Critical: super::super::Foundation::BOOLEAN,
    pub ThermalStandby: super::super::Foundation::BOOLEAN,
    pub ActivationReasons: u32,
    pub PassiveLimit: u32,
    pub ActiveLevel: u32,
    pub OverThrottled: super::super::Foundation::BOOLEAN,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for THERMAL_POLICY {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for THERMAL_POLICY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct THERMAL_WAIT_READ {
    pub Timeout: u32,
    pub LowTemperature: u32,
    pub HighTemperature: u32,
}
impl ::core::marker::Copy for THERMAL_WAIT_READ {}
impl ::core::clone::Clone for THERMAL_WAIT_READ {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct USER_POWER_POLICY {
    pub Revision: u32,
    pub IdleAc: POWER_ACTION_POLICY,
    pub IdleDc: POWER_ACTION_POLICY,
    pub IdleTimeoutAc: u32,
    pub IdleTimeoutDc: u32,
    pub IdleSensitivityAc: u8,
    pub IdleSensitivityDc: u8,
    pub ThrottlePolicyAc: u8,
    pub ThrottlePolicyDc: u8,
    pub MaxSleepAc: SYSTEM_POWER_STATE,
    pub MaxSleepDc: SYSTEM_POWER_STATE,
    pub Reserved: [u32; 2],
    pub VideoTimeoutAc: u32,
    pub VideoTimeoutDc: u32,
    pub SpindownTimeoutAc: u32,
    pub SpindownTimeoutDc: u32,
    pub OptimizeForPowerAc: super::super::Foundation::BOOLEAN,
    pub OptimizeForPowerDc: super::super::Foundation::BOOLEAN,
    pub FanThrottleToleranceAc: u8,
    pub FanThrottleToleranceDc: u8,
    pub ForcedThrottleAc: u8,
    pub ForcedThrottleDc: u8,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for USER_POWER_POLICY {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for USER_POWER_POLICY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub struct WAKE_ALARM_INFORMATION {
    pub TimerIdentifier: u32,
    pub Timeout: u32,
}
impl ::core::marker::Copy for WAKE_ALARM_INFORMATION {}
impl ::core::clone::Clone for WAKE_ALARM_INFORMATION {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub type EFFECTIVE_POWER_MODE_CALLBACK = ::core::option::Option<unsafe extern "system" fn(mode: EFFECTIVE_POWER_MODE, context: *const ::core::ffi::c_void)>;
#[doc = "*Required features: `\"Win32_System_Power\"`*"]
pub type PDEVICE_NOTIFY_CALLBACK_ROUTINE = ::core::option::Option<unsafe extern "system" fn(context: *const ::core::ffi::c_void, r#type: u32, setting: *const ::core::ffi::c_void) -> u32>;
#[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PWRSCHEMESENUMPROC = ::core::option::Option<unsafe extern "system" fn(index: u32, namesize: u32, name: ::windows_sys::core::PCWSTR, descriptionsize: u32, description: ::windows_sys::core::PCWSTR, policy: *const POWER_POLICY, context: super::super::Foundation::LPARAM) -> super::super::Foundation::BOOLEAN>;
#[doc = "*Required features: `\"Win32_System_Power\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PWRSCHEMESENUMPROC_V1 = ::core::option::Option<unsafe extern "system" fn(index: u32, namesize: u32, name: *const i8, descriptionsize: u32, description: *const i8, policy: *const POWER_POLICY, context: super::super::Foundation::LPARAM) -> super::super::Foundation::BOOLEAN>;

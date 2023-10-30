#[link(name = "windows")]
extern "system" {
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn CanSendToFaxRecipient() -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxAbort(faxhandle: super::super::Foundation::HANDLE, jobid: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxAccessCheck(faxhandle: super::super::Foundation::HANDLE, accessmask: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxClose(faxhandle: super::super::Foundation::HANDLE) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxCompleteJobParamsA(jobparams: *mut *mut FAX_JOB_PARAMA, coverpageinfo: *mut *mut FAX_COVERPAGE_INFOA) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxCompleteJobParamsW(jobparams: *mut *mut FAX_JOB_PARAMW, coverpageinfo: *mut *mut FAX_COVERPAGE_INFOW) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxConnectFaxServerA(machinename: ::windows_sys::core::PCSTR, faxhandle: *mut super::super::Foundation::HANDLE) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxConnectFaxServerW(machinename: ::windows_sys::core::PCWSTR, faxhandle: *mut super::super::Foundation::HANDLE) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxEnableRoutingMethodA(faxporthandle: super::super::Foundation::HANDLE, routingguid: ::windows_sys::core::PCSTR, enabled: super::super::Foundation::BOOL) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxEnableRoutingMethodW(faxporthandle: super::super::Foundation::HANDLE, routingguid: ::windows_sys::core::PCWSTR, enabled: super::super::Foundation::BOOL) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxEnumGlobalRoutingInfoA(faxhandle: super::super::Foundation::HANDLE, routinginfo: *mut *mut FAX_GLOBAL_ROUTING_INFOA, methodsreturned: *mut u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxEnumGlobalRoutingInfoW(faxhandle: super::super::Foundation::HANDLE, routinginfo: *mut *mut FAX_GLOBAL_ROUTING_INFOW, methodsreturned: *mut u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxEnumJobsA(faxhandle: super::super::Foundation::HANDLE, jobentry: *mut *mut FAX_JOB_ENTRYA, jobsreturned: *mut u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxEnumJobsW(faxhandle: super::super::Foundation::HANDLE, jobentry: *mut *mut FAX_JOB_ENTRYW, jobsreturned: *mut u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxEnumPortsA(faxhandle: super::super::Foundation::HANDLE, portinfo: *mut *mut FAX_PORT_INFOA, portsreturned: *mut u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxEnumPortsW(faxhandle: super::super::Foundation::HANDLE, portinfo: *mut *mut FAX_PORT_INFOW, portsreturned: *mut u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxEnumRoutingMethodsA(faxporthandle: super::super::Foundation::HANDLE, routingmethod: *mut *mut FAX_ROUTING_METHODA, methodsreturned: *mut u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxEnumRoutingMethodsW(faxporthandle: super::super::Foundation::HANDLE, routingmethod: *mut *mut FAX_ROUTING_METHODW, methodsreturned: *mut u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
    pub fn FaxFreeBuffer(buffer: *mut ::core::ffi::c_void);
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxGetConfigurationA(faxhandle: super::super::Foundation::HANDLE, faxconfig: *mut *mut FAX_CONFIGURATIONA) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxGetConfigurationW(faxhandle: super::super::Foundation::HANDLE, faxconfig: *mut *mut FAX_CONFIGURATIONW) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxGetDeviceStatusA(faxporthandle: super::super::Foundation::HANDLE, devicestatus: *mut *mut FAX_DEVICE_STATUSA) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxGetDeviceStatusW(faxporthandle: super::super::Foundation::HANDLE, devicestatus: *mut *mut FAX_DEVICE_STATUSW) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxGetJobA(faxhandle: super::super::Foundation::HANDLE, jobid: u32, jobentry: *mut *mut FAX_JOB_ENTRYA) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxGetJobW(faxhandle: super::super::Foundation::HANDLE, jobid: u32, jobentry: *mut *mut FAX_JOB_ENTRYW) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxGetLoggingCategoriesA(faxhandle: super::super::Foundation::HANDLE, categories: *mut *mut FAX_LOG_CATEGORYA, numbercategories: *mut u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxGetLoggingCategoriesW(faxhandle: super::super::Foundation::HANDLE, categories: *mut *mut FAX_LOG_CATEGORYW, numbercategories: *mut u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxGetPageData(faxhandle: super::super::Foundation::HANDLE, jobid: u32, buffer: *mut *mut u8, buffersize: *mut u32, imagewidth: *mut u32, imageheight: *mut u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxGetPortA(faxporthandle: super::super::Foundation::HANDLE, portinfo: *mut *mut FAX_PORT_INFOA) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxGetPortW(faxporthandle: super::super::Foundation::HANDLE, portinfo: *mut *mut FAX_PORT_INFOW) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxGetRoutingInfoA(faxporthandle: super::super::Foundation::HANDLE, routingguid: ::windows_sys::core::PCSTR, routinginfobuffer: *mut *mut u8, routinginfobuffersize: *mut u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxGetRoutingInfoW(faxporthandle: super::super::Foundation::HANDLE, routingguid: ::windows_sys::core::PCWSTR, routinginfobuffer: *mut *mut u8, routinginfobuffersize: *mut u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxInitializeEventQueue(faxhandle: super::super::Foundation::HANDLE, completionport: super::super::Foundation::HANDLE, completionkey: usize, hwnd: super::super::Foundation::HWND, messagestart: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxOpenPort(faxhandle: super::super::Foundation::HANDLE, deviceid: u32, flags: u32, faxporthandle: *mut super::super::Foundation::HANDLE) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn FaxPrintCoverPageA(faxcontextinfo: *const FAX_CONTEXT_INFOA, coverpageinfo: *const FAX_COVERPAGE_INFOA) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn FaxPrintCoverPageW(faxcontextinfo: *const FAX_CONTEXT_INFOW, coverpageinfo: *const FAX_COVERPAGE_INFOW) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxRegisterRoutingExtensionW(faxhandle: super::super::Foundation::HANDLE, extensionname: ::windows_sys::core::PCWSTR, friendlyname: ::windows_sys::core::PCWSTR, imagename: ::windows_sys::core::PCWSTR, callback: PFAX_ROUTING_INSTALLATION_CALLBACKW, context: *mut ::core::ffi::c_void) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxRegisterServiceProviderW(deviceprovider: ::windows_sys::core::PCWSTR, friendlyname: ::windows_sys::core::PCWSTR, imagename: ::windows_sys::core::PCWSTR, tspname: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxSendDocumentA(faxhandle: super::super::Foundation::HANDLE, filename: ::windows_sys::core::PCSTR, jobparams: *mut FAX_JOB_PARAMA, coverpageinfo: *const FAX_COVERPAGE_INFOA, faxjobid: *mut u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxSendDocumentForBroadcastA(faxhandle: super::super::Foundation::HANDLE, filename: ::windows_sys::core::PCSTR, faxjobid: *mut u32, faxrecipientcallback: PFAX_RECIPIENT_CALLBACKA, context: *mut ::core::ffi::c_void) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxSendDocumentForBroadcastW(faxhandle: super::super::Foundation::HANDLE, filename: ::windows_sys::core::PCWSTR, faxjobid: *mut u32, faxrecipientcallback: PFAX_RECIPIENT_CALLBACKW, context: *mut ::core::ffi::c_void) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxSendDocumentW(faxhandle: super::super::Foundation::HANDLE, filename: ::windows_sys::core::PCWSTR, jobparams: *mut FAX_JOB_PARAMW, coverpageinfo: *const FAX_COVERPAGE_INFOW, faxjobid: *mut u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxSetConfigurationA(faxhandle: super::super::Foundation::HANDLE, faxconfig: *const FAX_CONFIGURATIONA) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxSetConfigurationW(faxhandle: super::super::Foundation::HANDLE, faxconfig: *const FAX_CONFIGURATIONW) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxSetGlobalRoutingInfoA(faxhandle: super::super::Foundation::HANDLE, routinginfo: *const FAX_GLOBAL_ROUTING_INFOA) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxSetGlobalRoutingInfoW(faxhandle: super::super::Foundation::HANDLE, routinginfo: *const FAX_GLOBAL_ROUTING_INFOW) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxSetJobA(faxhandle: super::super::Foundation::HANDLE, jobid: u32, command: u32, jobentry: *const FAX_JOB_ENTRYA) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxSetJobW(faxhandle: super::super::Foundation::HANDLE, jobid: u32, command: u32, jobentry: *const FAX_JOB_ENTRYW) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxSetLoggingCategoriesA(faxhandle: super::super::Foundation::HANDLE, categories: *const FAX_LOG_CATEGORYA, numbercategories: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxSetLoggingCategoriesW(faxhandle: super::super::Foundation::HANDLE, categories: *const FAX_LOG_CATEGORYW, numbercategories: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxSetPortA(faxporthandle: super::super::Foundation::HANDLE, portinfo: *const FAX_PORT_INFOA) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxSetPortW(faxporthandle: super::super::Foundation::HANDLE, portinfo: *const FAX_PORT_INFOW) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxSetRoutingInfoA(faxporthandle: super::super::Foundation::HANDLE, routingguid: ::windows_sys::core::PCSTR, routinginfobuffer: *const u8, routinginfobuffersize: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxSetRoutingInfoW(faxporthandle: super::super::Foundation::HANDLE, routingguid: ::windows_sys::core::PCWSTR, routinginfobuffer: *const u8, routinginfobuffersize: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn FaxStartPrintJobA(printername: ::windows_sys::core::PCSTR, printinfo: *const FAX_PRINT_INFOA, faxjobid: *mut u32, faxcontextinfo: *mut FAX_CONTEXT_INFOA) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn FaxStartPrintJobW(printername: ::windows_sys::core::PCWSTR, printinfo: *const FAX_PRINT_INFOW, faxjobid: *mut u32, faxcontextinfo: *mut FAX_CONTEXT_INFOW) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FaxUnregisterServiceProviderW(deviceprovider: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
    pub fn SendToFaxRecipient(sndmode: SendToMode, lpfilename: ::windows_sys::core::PCWSTR) -> u32;
    #[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn StiCreateInstanceW(hinst: super::super::Foundation::HINSTANCE, dwver: u32, ppsti: *mut IStillImageW, punkouter: ::windows_sys::core::IUnknown) -> ::windows_sys::core::HRESULT;
}
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const CF_MSFAXSRV_DEVICE_ID: &str = "FAXSRV_DeviceID";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const CF_MSFAXSRV_FSP_GUID: &str = "FAXSRV_FSPGuid";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const CF_MSFAXSRV_ROUTEEXT_NAME: &str = "FAXSRV_RoutingExtName";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const CF_MSFAXSRV_ROUTING_METHOD_GUID: &str = "FAXSRV_RoutingMethodGuid";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const CF_MSFAXSRV_SERVER_NAME: &str = "FAXSRV_ServerName";
pub const CLSID_Sti: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3005479136, data2: 11880, data3: 4560, data4: [144, 234, 0, 170, 0, 96, 248, 108] };
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Devices_Properties\"`*"]
#[cfg(feature = "Win32_Devices_Properties")]
pub const DEVPKEY_WIA_DeviceType: super::Properties::DEVPROPKEY = super::Properties::DEVPROPKEY { fmtid: ::windows_sys::core::GUID { data1: 1809653702, data2: 33039, data3: 4560, data4: [190, 199, 8, 0, 43, 226, 9, 47] }, pid: 2u32 };
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Devices_Properties\"`*"]
#[cfg(feature = "Win32_Devices_Properties")]
pub const DEVPKEY_WIA_USDClassId: super::Properties::DEVPROPKEY = super::Properties::DEVPROPKEY { fmtid: ::windows_sys::core::GUID { data1: 1809653702, data2: 33039, data3: 4560, data4: [190, 199, 8, 0, 43, 226, 9, 47] }, pid: 3u32 };
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAXDEVRECEIVE_SIZE: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAXDEVREPORTSTATUS_SIZE: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type FAXROUTE_ENABLE = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const QUERY_STATUS: FAXROUTE_ENABLE = -1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STATUS_DISABLE: FAXROUTE_ENABLE = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STATUS_ENABLE: FAXROUTE_ENABLE = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type FAX_ACCESS_RIGHTS_ENUM = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const farSUBMIT_LOW: FAX_ACCESS_RIGHTS_ENUM = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const farSUBMIT_NORMAL: FAX_ACCESS_RIGHTS_ENUM = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const farSUBMIT_HIGH: FAX_ACCESS_RIGHTS_ENUM = 4i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const farQUERY_JOBS: FAX_ACCESS_RIGHTS_ENUM = 8i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const farMANAGE_JOBS: FAX_ACCESS_RIGHTS_ENUM = 16i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const farQUERY_CONFIG: FAX_ACCESS_RIGHTS_ENUM = 32i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const farMANAGE_CONFIG: FAX_ACCESS_RIGHTS_ENUM = 64i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const farQUERY_IN_ARCHIVE: FAX_ACCESS_RIGHTS_ENUM = 128i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const farMANAGE_IN_ARCHIVE: FAX_ACCESS_RIGHTS_ENUM = 256i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const farQUERY_OUT_ARCHIVE: FAX_ACCESS_RIGHTS_ENUM = 512i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const farMANAGE_OUT_ARCHIVE: FAX_ACCESS_RIGHTS_ENUM = 1024i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type FAX_ACCESS_RIGHTS_ENUM_2 = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const far2SUBMIT_LOW: FAX_ACCESS_RIGHTS_ENUM_2 = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const far2SUBMIT_NORMAL: FAX_ACCESS_RIGHTS_ENUM_2 = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const far2SUBMIT_HIGH: FAX_ACCESS_RIGHTS_ENUM_2 = 4i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const far2QUERY_OUT_JOBS: FAX_ACCESS_RIGHTS_ENUM_2 = 8i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const far2MANAGE_OUT_JOBS: FAX_ACCESS_RIGHTS_ENUM_2 = 16i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const far2QUERY_CONFIG: FAX_ACCESS_RIGHTS_ENUM_2 = 32i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const far2MANAGE_CONFIG: FAX_ACCESS_RIGHTS_ENUM_2 = 64i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const far2QUERY_ARCHIVES: FAX_ACCESS_RIGHTS_ENUM_2 = 128i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const far2MANAGE_ARCHIVES: FAX_ACCESS_RIGHTS_ENUM_2 = 256i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const far2MANAGE_RECEIVE_FOLDER: FAX_ACCESS_RIGHTS_ENUM_2 = 512i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type FAX_ACCOUNT_EVENTS_TYPE_ENUM = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const faetNONE: FAX_ACCOUNT_EVENTS_TYPE_ENUM = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const faetIN_QUEUE: FAX_ACCOUNT_EVENTS_TYPE_ENUM = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const faetOUT_QUEUE: FAX_ACCOUNT_EVENTS_TYPE_ENUM = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const faetIN_ARCHIVE: FAX_ACCOUNT_EVENTS_TYPE_ENUM = 4i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const faetOUT_ARCHIVE: FAX_ACCOUNT_EVENTS_TYPE_ENUM = 8i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const faetFXSSVC_ENDED: FAX_ACCOUNT_EVENTS_TYPE_ENUM = 16i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FAX_CONFIGURATIONA {
    pub SizeOfStruct: u32,
    pub Retries: u32,
    pub RetryDelay: u32,
    pub DirtyDays: u32,
    pub Branding: super::super::Foundation::BOOL,
    pub UseDeviceTsid: super::super::Foundation::BOOL,
    pub ServerCp: super::super::Foundation::BOOL,
    pub PauseServerQueue: super::super::Foundation::BOOL,
    pub StartCheapTime: FAX_TIME,
    pub StopCheapTime: FAX_TIME,
    pub ArchiveOutgoingFaxes: super::super::Foundation::BOOL,
    pub ArchiveDirectory: ::windows_sys::core::PCSTR,
    pub Reserved: ::windows_sys::core::PCSTR,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FAX_CONFIGURATIONA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FAX_CONFIGURATIONA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FAX_CONFIGURATIONW {
    pub SizeOfStruct: u32,
    pub Retries: u32,
    pub RetryDelay: u32,
    pub DirtyDays: u32,
    pub Branding: super::super::Foundation::BOOL,
    pub UseDeviceTsid: super::super::Foundation::BOOL,
    pub ServerCp: super::super::Foundation::BOOL,
    pub PauseServerQueue: super::super::Foundation::BOOL,
    pub StartCheapTime: FAX_TIME,
    pub StopCheapTime: FAX_TIME,
    pub ArchiveOutgoingFaxes: super::super::Foundation::BOOL,
    pub ArchiveDirectory: ::windows_sys::core::PCWSTR,
    pub Reserved: ::windows_sys::core::PCWSTR,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FAX_CONFIGURATIONW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FAX_CONFIGURATIONW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_CONFIG_QUERY: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_CONFIG_SET: u32 = 8u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct FAX_CONTEXT_INFOA {
    pub SizeOfStruct: u32,
    pub hDC: super::super::Graphics::Gdi::HDC,
    pub ServerName: [super::super::Foundation::CHAR; 16],
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for FAX_CONTEXT_INFOA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for FAX_CONTEXT_INFOA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(feature = "Win32_Graphics_Gdi")]
pub struct FAX_CONTEXT_INFOW {
    pub SizeOfStruct: u32,
    pub hDC: super::super::Graphics::Gdi::HDC,
    pub ServerName: [u16; 16],
}
#[cfg(feature = "Win32_Graphics_Gdi")]
impl ::core::marker::Copy for FAX_CONTEXT_INFOW {}
#[cfg(feature = "Win32_Graphics_Gdi")]
impl ::core::clone::Clone for FAX_CONTEXT_INFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FAX_COVERPAGE_INFOA {
    pub SizeOfStruct: u32,
    pub CoverPageName: ::windows_sys::core::PCSTR,
    pub UseServerCoverPage: super::super::Foundation::BOOL,
    pub RecName: ::windows_sys::core::PCSTR,
    pub RecFaxNumber: ::windows_sys::core::PCSTR,
    pub RecCompany: ::windows_sys::core::PCSTR,
    pub RecStreetAddress: ::windows_sys::core::PCSTR,
    pub RecCity: ::windows_sys::core::PCSTR,
    pub RecState: ::windows_sys::core::PCSTR,
    pub RecZip: ::windows_sys::core::PCSTR,
    pub RecCountry: ::windows_sys::core::PCSTR,
    pub RecTitle: ::windows_sys::core::PCSTR,
    pub RecDepartment: ::windows_sys::core::PCSTR,
    pub RecOfficeLocation: ::windows_sys::core::PCSTR,
    pub RecHomePhone: ::windows_sys::core::PCSTR,
    pub RecOfficePhone: ::windows_sys::core::PCSTR,
    pub SdrName: ::windows_sys::core::PCSTR,
    pub SdrFaxNumber: ::windows_sys::core::PCSTR,
    pub SdrCompany: ::windows_sys::core::PCSTR,
    pub SdrAddress: ::windows_sys::core::PCSTR,
    pub SdrTitle: ::windows_sys::core::PCSTR,
    pub SdrDepartment: ::windows_sys::core::PCSTR,
    pub SdrOfficeLocation: ::windows_sys::core::PCSTR,
    pub SdrHomePhone: ::windows_sys::core::PCSTR,
    pub SdrOfficePhone: ::windows_sys::core::PCSTR,
    pub Note: ::windows_sys::core::PCSTR,
    pub Subject: ::windows_sys::core::PCSTR,
    pub TimeSent: super::super::Foundation::SYSTEMTIME,
    pub PageCount: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FAX_COVERPAGE_INFOA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FAX_COVERPAGE_INFOA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FAX_COVERPAGE_INFOW {
    pub SizeOfStruct: u32,
    pub CoverPageName: ::windows_sys::core::PCWSTR,
    pub UseServerCoverPage: super::super::Foundation::BOOL,
    pub RecName: ::windows_sys::core::PCWSTR,
    pub RecFaxNumber: ::windows_sys::core::PCWSTR,
    pub RecCompany: ::windows_sys::core::PCWSTR,
    pub RecStreetAddress: ::windows_sys::core::PCWSTR,
    pub RecCity: ::windows_sys::core::PCWSTR,
    pub RecState: ::windows_sys::core::PCWSTR,
    pub RecZip: ::windows_sys::core::PCWSTR,
    pub RecCountry: ::windows_sys::core::PCWSTR,
    pub RecTitle: ::windows_sys::core::PCWSTR,
    pub RecDepartment: ::windows_sys::core::PCWSTR,
    pub RecOfficeLocation: ::windows_sys::core::PCWSTR,
    pub RecHomePhone: ::windows_sys::core::PCWSTR,
    pub RecOfficePhone: ::windows_sys::core::PCWSTR,
    pub SdrName: ::windows_sys::core::PCWSTR,
    pub SdrFaxNumber: ::windows_sys::core::PCWSTR,
    pub SdrCompany: ::windows_sys::core::PCWSTR,
    pub SdrAddress: ::windows_sys::core::PCWSTR,
    pub SdrTitle: ::windows_sys::core::PCWSTR,
    pub SdrDepartment: ::windows_sys::core::PCWSTR,
    pub SdrOfficeLocation: ::windows_sys::core::PCWSTR,
    pub SdrHomePhone: ::windows_sys::core::PCWSTR,
    pub SdrOfficePhone: ::windows_sys::core::PCWSTR,
    pub Note: ::windows_sys::core::PCWSTR,
    pub Subject: ::windows_sys::core::PCWSTR,
    pub TimeSent: super::super::Foundation::SYSTEMTIME,
    pub PageCount: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FAX_COVERPAGE_INFOW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FAX_COVERPAGE_INFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type FAX_COVERPAGE_TYPE_ENUM = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fcptNONE: FAX_COVERPAGE_TYPE_ENUM = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fcptLOCAL: FAX_COVERPAGE_TYPE_ENUM = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fcptSERVER: FAX_COVERPAGE_TYPE_ENUM = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type FAX_DEVICE_RECEIVE_MODE_ENUM = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fdrmNO_ANSWER: FAX_DEVICE_RECEIVE_MODE_ENUM = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fdrmAUTO_ANSWER: FAX_DEVICE_RECEIVE_MODE_ENUM = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fdrmMANUAL_ANSWER: FAX_DEVICE_RECEIVE_MODE_ENUM = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FAX_DEVICE_STATUSA {
    pub SizeOfStruct: u32,
    pub CallerId: ::windows_sys::core::PCSTR,
    pub Csid: ::windows_sys::core::PCSTR,
    pub CurrentPage: u32,
    pub DeviceId: u32,
    pub DeviceName: ::windows_sys::core::PCSTR,
    pub DocumentName: ::windows_sys::core::PCSTR,
    pub JobType: u32,
    pub PhoneNumber: ::windows_sys::core::PCSTR,
    pub RoutingString: ::windows_sys::core::PCSTR,
    pub SenderName: ::windows_sys::core::PCSTR,
    pub RecipientName: ::windows_sys::core::PCSTR,
    pub Size: u32,
    pub StartTime: super::super::Foundation::FILETIME,
    pub Status: u32,
    pub StatusString: ::windows_sys::core::PCSTR,
    pub SubmittedTime: super::super::Foundation::FILETIME,
    pub TotalPages: u32,
    pub Tsid: ::windows_sys::core::PCSTR,
    pub UserName: ::windows_sys::core::PCSTR,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FAX_DEVICE_STATUSA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FAX_DEVICE_STATUSA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FAX_DEVICE_STATUSW {
    pub SizeOfStruct: u32,
    pub CallerId: ::windows_sys::core::PCWSTR,
    pub Csid: ::windows_sys::core::PCWSTR,
    pub CurrentPage: u32,
    pub DeviceId: u32,
    pub DeviceName: ::windows_sys::core::PCWSTR,
    pub DocumentName: ::windows_sys::core::PCWSTR,
    pub JobType: u32,
    pub PhoneNumber: ::windows_sys::core::PCWSTR,
    pub RoutingString: ::windows_sys::core::PCWSTR,
    pub SenderName: ::windows_sys::core::PCWSTR,
    pub RecipientName: ::windows_sys::core::PCWSTR,
    pub Size: u32,
    pub StartTime: super::super::Foundation::FILETIME,
    pub Status: u32,
    pub StatusString: ::windows_sys::core::PCWSTR,
    pub SubmittedTime: super::super::Foundation::FILETIME,
    pub TotalPages: u32,
    pub Tsid: ::windows_sys::core::PCWSTR,
    pub UserName: ::windows_sys::core::PCWSTR,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FAX_DEVICE_STATUSW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FAX_DEVICE_STATUSW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub struct FAX_DEV_STATUS {
    pub SizeOfStruct: u32,
    pub StatusId: u32,
    pub StringId: u32,
    pub PageCount: u32,
    pub CSI: ::windows_sys::core::PWSTR,
    pub CallerId: ::windows_sys::core::PWSTR,
    pub RoutingInfo: ::windows_sys::core::PWSTR,
    pub ErrorCode: u32,
    pub Reserved: [u32; 3],
}
impl ::core::marker::Copy for FAX_DEV_STATUS {}
impl ::core::clone::Clone for FAX_DEV_STATUS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type FAX_ENUM_DELIVERY_REPORT_TYPES = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const DRT_NONE: FAX_ENUM_DELIVERY_REPORT_TYPES = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const DRT_EMAIL: FAX_ENUM_DELIVERY_REPORT_TYPES = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const DRT_INBOX: FAX_ENUM_DELIVERY_REPORT_TYPES = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type FAX_ENUM_DEVICE_ID_SOURCE = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const DEV_ID_SRC_FAX: FAX_ENUM_DEVICE_ID_SOURCE = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const DEV_ID_SRC_TAPI: FAX_ENUM_DEVICE_ID_SOURCE = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type FAX_ENUM_JOB_COMMANDS = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const JC_UNKNOWN: FAX_ENUM_JOB_COMMANDS = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const JC_DELETE: FAX_ENUM_JOB_COMMANDS = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const JC_PAUSE: FAX_ENUM_JOB_COMMANDS = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const JC_RESUME: FAX_ENUM_JOB_COMMANDS = 3i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type FAX_ENUM_JOB_SEND_ATTRIBUTES = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const JSA_NOW: FAX_ENUM_JOB_SEND_ATTRIBUTES = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const JSA_SPECIFIC_TIME: FAX_ENUM_JOB_SEND_ATTRIBUTES = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const JSA_DISCOUNT_PERIOD: FAX_ENUM_JOB_SEND_ATTRIBUTES = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type FAX_ENUM_LOG_CATEGORIES = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAXLOG_CATEGORY_INIT: FAX_ENUM_LOG_CATEGORIES = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAXLOG_CATEGORY_OUTBOUND: FAX_ENUM_LOG_CATEGORIES = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAXLOG_CATEGORY_INBOUND: FAX_ENUM_LOG_CATEGORIES = 3i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAXLOG_CATEGORY_UNKNOWN: FAX_ENUM_LOG_CATEGORIES = 4i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type FAX_ENUM_LOG_LEVELS = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAXLOG_LEVEL_NONE: FAX_ENUM_LOG_LEVELS = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAXLOG_LEVEL_MIN: FAX_ENUM_LOG_LEVELS = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAXLOG_LEVEL_MED: FAX_ENUM_LOG_LEVELS = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAXLOG_LEVEL_MAX: FAX_ENUM_LOG_LEVELS = 3i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type FAX_ENUM_PORT_OPEN_TYPE = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const PORT_OPEN_QUERY: FAX_ENUM_PORT_OPEN_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const PORT_OPEN_MODIFY: FAX_ENUM_PORT_OPEN_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_ERR_BAD_GROUP_CONFIGURATION: i32 = 7003i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_ERR_DEVICE_NUM_LIMIT_EXCEEDED: i32 = 7010i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_ERR_DIRECTORY_IN_USE: i32 = 7007i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_ERR_END: i32 = 7013i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_ERR_FILE_ACCESS_DENIED: i32 = 7008i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_ERR_GROUP_IN_USE: i32 = 7004i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_ERR_GROUP_NOT_FOUND: i32 = 7002i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_ERR_MESSAGE_NOT_FOUND: i32 = 7009i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_ERR_NOT_NTFS: i32 = 7006i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_ERR_NOT_SUPPORTED_ON_THIS_SKU: i32 = 7011i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_ERR_RECIPIENTS_LIMIT: i32 = 7013i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_ERR_RULE_NOT_FOUND: i32 = 7005i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_ERR_SRV_OUTOFMEMORY: i32 = 7001i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_ERR_START: i32 = 7001i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_ERR_VERSION_MISMATCH: i32 = 7012i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FAX_EVENTA {
    pub SizeOfStruct: u32,
    pub TimeStamp: super::super::Foundation::FILETIME,
    pub DeviceId: u32,
    pub EventId: u32,
    pub JobId: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FAX_EVENTA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FAX_EVENTA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FAX_EVENTW {
    pub SizeOfStruct: u32,
    pub TimeStamp: super::super::Foundation::FILETIME,
    pub DeviceId: u32,
    pub EventId: u32,
    pub JobId: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FAX_EVENTW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FAX_EVENTW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_E_BAD_GROUP_CONFIGURATION: ::windows_sys::core::HRESULT = -2147214501i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_E_DEVICE_NUM_LIMIT_EXCEEDED: ::windows_sys::core::HRESULT = -2147214494i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_E_DIRECTORY_IN_USE: ::windows_sys::core::HRESULT = -2147214497i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_E_FILE_ACCESS_DENIED: ::windows_sys::core::HRESULT = -2147214496i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_E_GROUP_IN_USE: ::windows_sys::core::HRESULT = -2147214500i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_E_GROUP_NOT_FOUND: ::windows_sys::core::HRESULT = -2147214502i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_E_MESSAGE_NOT_FOUND: ::windows_sys::core::HRESULT = -2147214495i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_E_NOT_NTFS: ::windows_sys::core::HRESULT = -2147214498i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_E_NOT_SUPPORTED_ON_THIS_SKU: ::windows_sys::core::HRESULT = -2147214493i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_E_RECIPIENTS_LIMIT: ::windows_sys::core::HRESULT = -2147214491i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_E_RULE_NOT_FOUND: ::windows_sys::core::HRESULT = -2147214499i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_E_SRV_OUTOFMEMORY: ::windows_sys::core::HRESULT = -2147214503i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_E_VERSION_MISMATCH: ::windows_sys::core::HRESULT = -2147214492i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub struct FAX_GLOBAL_ROUTING_INFOA {
    pub SizeOfStruct: u32,
    pub Priority: u32,
    pub Guid: ::windows_sys::core::PCSTR,
    pub FriendlyName: ::windows_sys::core::PCSTR,
    pub FunctionName: ::windows_sys::core::PCSTR,
    pub ExtensionImageName: ::windows_sys::core::PCSTR,
    pub ExtensionFriendlyName: ::windows_sys::core::PCSTR,
}
impl ::core::marker::Copy for FAX_GLOBAL_ROUTING_INFOA {}
impl ::core::clone::Clone for FAX_GLOBAL_ROUTING_INFOA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub struct FAX_GLOBAL_ROUTING_INFOW {
    pub SizeOfStruct: u32,
    pub Priority: u32,
    pub Guid: ::windows_sys::core::PCWSTR,
    pub FriendlyName: ::windows_sys::core::PCWSTR,
    pub FunctionName: ::windows_sys::core::PCWSTR,
    pub ExtensionImageName: ::windows_sys::core::PCWSTR,
    pub ExtensionFriendlyName: ::windows_sys::core::PCWSTR,
}
impl ::core::marker::Copy for FAX_GLOBAL_ROUTING_INFOW {}
impl ::core::clone::Clone for FAX_GLOBAL_ROUTING_INFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type FAX_GROUP_STATUS_ENUM = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fgsALL_DEV_VALID: FAX_GROUP_STATUS_ENUM = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fgsEMPTY: FAX_GROUP_STATUS_ENUM = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fgsALL_DEV_NOT_VALID: FAX_GROUP_STATUS_ENUM = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fgsSOME_DEV_NOT_VALID: FAX_GROUP_STATUS_ENUM = 3i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FAX_JOB_ENTRYA {
    pub SizeOfStruct: u32,
    pub JobId: u32,
    pub UserName: ::windows_sys::core::PCSTR,
    pub JobType: u32,
    pub QueueStatus: u32,
    pub Status: u32,
    pub Size: u32,
    pub PageCount: u32,
    pub RecipientNumber: ::windows_sys::core::PCSTR,
    pub RecipientName: ::windows_sys::core::PCSTR,
    pub Tsid: ::windows_sys::core::PCSTR,
    pub SenderName: ::windows_sys::core::PCSTR,
    pub SenderCompany: ::windows_sys::core::PCSTR,
    pub SenderDept: ::windows_sys::core::PCSTR,
    pub BillingCode: ::windows_sys::core::PCSTR,
    pub ScheduleAction: u32,
    pub ScheduleTime: super::super::Foundation::SYSTEMTIME,
    pub DeliveryReportType: u32,
    pub DeliveryReportAddress: ::windows_sys::core::PCSTR,
    pub DocumentName: ::windows_sys::core::PCSTR,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FAX_JOB_ENTRYA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FAX_JOB_ENTRYA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FAX_JOB_ENTRYW {
    pub SizeOfStruct: u32,
    pub JobId: u32,
    pub UserName: ::windows_sys::core::PCWSTR,
    pub JobType: u32,
    pub QueueStatus: u32,
    pub Status: u32,
    pub Size: u32,
    pub PageCount: u32,
    pub RecipientNumber: ::windows_sys::core::PCWSTR,
    pub RecipientName: ::windows_sys::core::PCWSTR,
    pub Tsid: ::windows_sys::core::PCWSTR,
    pub SenderName: ::windows_sys::core::PCWSTR,
    pub SenderCompany: ::windows_sys::core::PCWSTR,
    pub SenderDept: ::windows_sys::core::PCWSTR,
    pub BillingCode: ::windows_sys::core::PCWSTR,
    pub ScheduleAction: u32,
    pub ScheduleTime: super::super::Foundation::SYSTEMTIME,
    pub DeliveryReportType: u32,
    pub DeliveryReportAddress: ::windows_sys::core::PCWSTR,
    pub DocumentName: ::windows_sys::core::PCWSTR,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FAX_JOB_ENTRYW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FAX_JOB_ENTRYW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type FAX_JOB_EXTENDED_STATUS_ENUM = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjesNONE: FAX_JOB_EXTENDED_STATUS_ENUM = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjesDISCONNECTED: FAX_JOB_EXTENDED_STATUS_ENUM = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjesINITIALIZING: FAX_JOB_EXTENDED_STATUS_ENUM = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjesDIALING: FAX_JOB_EXTENDED_STATUS_ENUM = 3i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjesTRANSMITTING: FAX_JOB_EXTENDED_STATUS_ENUM = 4i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjesANSWERED: FAX_JOB_EXTENDED_STATUS_ENUM = 5i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjesRECEIVING: FAX_JOB_EXTENDED_STATUS_ENUM = 6i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjesLINE_UNAVAILABLE: FAX_JOB_EXTENDED_STATUS_ENUM = 7i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjesBUSY: FAX_JOB_EXTENDED_STATUS_ENUM = 8i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjesNO_ANSWER: FAX_JOB_EXTENDED_STATUS_ENUM = 9i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjesBAD_ADDRESS: FAX_JOB_EXTENDED_STATUS_ENUM = 10i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjesNO_DIAL_TONE: FAX_JOB_EXTENDED_STATUS_ENUM = 11i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjesFATAL_ERROR: FAX_JOB_EXTENDED_STATUS_ENUM = 12i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjesCALL_DELAYED: FAX_JOB_EXTENDED_STATUS_ENUM = 13i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjesCALL_BLACKLISTED: FAX_JOB_EXTENDED_STATUS_ENUM = 14i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjesNOT_FAX_CALL: FAX_JOB_EXTENDED_STATUS_ENUM = 15i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjesPARTIALLY_RECEIVED: FAX_JOB_EXTENDED_STATUS_ENUM = 16i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjesHANDLED: FAX_JOB_EXTENDED_STATUS_ENUM = 17i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjesCALL_COMPLETED: FAX_JOB_EXTENDED_STATUS_ENUM = 18i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjesCALL_ABORTED: FAX_JOB_EXTENDED_STATUS_ENUM = 19i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjesPROPRIETARY: FAX_JOB_EXTENDED_STATUS_ENUM = 16777216i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_JOB_MANAGE: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type FAX_JOB_OPERATIONS_ENUM = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjoVIEW: FAX_JOB_OPERATIONS_ENUM = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjoPAUSE: FAX_JOB_OPERATIONS_ENUM = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjoRESUME: FAX_JOB_OPERATIONS_ENUM = 4i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjoRESTART: FAX_JOB_OPERATIONS_ENUM = 8i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjoDELETE: FAX_JOB_OPERATIONS_ENUM = 16i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjoRECIPIENT_INFO: FAX_JOB_OPERATIONS_ENUM = 32i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjoSENDER_INFO: FAX_JOB_OPERATIONS_ENUM = 64i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FAX_JOB_PARAMA {
    pub SizeOfStruct: u32,
    pub RecipientNumber: ::windows_sys::core::PCSTR,
    pub RecipientName: ::windows_sys::core::PCSTR,
    pub Tsid: ::windows_sys::core::PCSTR,
    pub SenderName: ::windows_sys::core::PCSTR,
    pub SenderCompany: ::windows_sys::core::PCSTR,
    pub SenderDept: ::windows_sys::core::PCSTR,
    pub BillingCode: ::windows_sys::core::PCSTR,
    pub ScheduleAction: u32,
    pub ScheduleTime: super::super::Foundation::SYSTEMTIME,
    pub DeliveryReportType: u32,
    pub DeliveryReportAddress: ::windows_sys::core::PCSTR,
    pub DocumentName: ::windows_sys::core::PCSTR,
    pub CallHandle: u32,
    pub Reserved: [usize; 3],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FAX_JOB_PARAMA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FAX_JOB_PARAMA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FAX_JOB_PARAMW {
    pub SizeOfStruct: u32,
    pub RecipientNumber: ::windows_sys::core::PCWSTR,
    pub RecipientName: ::windows_sys::core::PCWSTR,
    pub Tsid: ::windows_sys::core::PCWSTR,
    pub SenderName: ::windows_sys::core::PCWSTR,
    pub SenderCompany: ::windows_sys::core::PCWSTR,
    pub SenderDept: ::windows_sys::core::PCWSTR,
    pub BillingCode: ::windows_sys::core::PCWSTR,
    pub ScheduleAction: u32,
    pub ScheduleTime: super::super::Foundation::SYSTEMTIME,
    pub DeliveryReportType: u32,
    pub DeliveryReportAddress: ::windows_sys::core::PCWSTR,
    pub DocumentName: ::windows_sys::core::PCWSTR,
    pub CallHandle: u32,
    pub Reserved: [usize; 3],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FAX_JOB_PARAMW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FAX_JOB_PARAMW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_JOB_QUERY: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type FAX_JOB_STATUS_ENUM = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjsPENDING: FAX_JOB_STATUS_ENUM = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjsINPROGRESS: FAX_JOB_STATUS_ENUM = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjsFAILED: FAX_JOB_STATUS_ENUM = 8i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjsPAUSED: FAX_JOB_STATUS_ENUM = 16i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjsNOLINE: FAX_JOB_STATUS_ENUM = 32i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjsRETRYING: FAX_JOB_STATUS_ENUM = 64i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjsRETRIES_EXCEEDED: FAX_JOB_STATUS_ENUM = 128i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjsCOMPLETED: FAX_JOB_STATUS_ENUM = 256i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjsCANCELED: FAX_JOB_STATUS_ENUM = 512i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjsCANCELING: FAX_JOB_STATUS_ENUM = 1024i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjsROUTING: FAX_JOB_STATUS_ENUM = 2048i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_JOB_SUBMIT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type FAX_JOB_TYPE_ENUM = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjtSEND: FAX_JOB_TYPE_ENUM = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjtRECEIVE: FAX_JOB_TYPE_ENUM = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fjtROUTING: FAX_JOB_TYPE_ENUM = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub struct FAX_LOG_CATEGORYA {
    pub Name: ::windows_sys::core::PCSTR,
    pub Category: u32,
    pub Level: u32,
}
impl ::core::marker::Copy for FAX_LOG_CATEGORYA {}
impl ::core::clone::Clone for FAX_LOG_CATEGORYA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub struct FAX_LOG_CATEGORYW {
    pub Name: ::windows_sys::core::PCWSTR,
    pub Category: u32,
    pub Level: u32,
}
impl ::core::marker::Copy for FAX_LOG_CATEGORYW {}
impl ::core::clone::Clone for FAX_LOG_CATEGORYW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type FAX_LOG_LEVEL_ENUM = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fllNONE: FAX_LOG_LEVEL_ENUM = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fllMIN: FAX_LOG_LEVEL_ENUM = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fllMED: FAX_LOG_LEVEL_ENUM = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fllMAX: FAX_LOG_LEVEL_ENUM = 3i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub struct FAX_PORT_INFOA {
    pub SizeOfStruct: u32,
    pub DeviceId: u32,
    pub State: u32,
    pub Flags: u32,
    pub Rings: u32,
    pub Priority: u32,
    pub DeviceName: ::windows_sys::core::PCSTR,
    pub Tsid: ::windows_sys::core::PCSTR,
    pub Csid: ::windows_sys::core::PCSTR,
}
impl ::core::marker::Copy for FAX_PORT_INFOA {}
impl ::core::clone::Clone for FAX_PORT_INFOA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub struct FAX_PORT_INFOW {
    pub SizeOfStruct: u32,
    pub DeviceId: u32,
    pub State: u32,
    pub Flags: u32,
    pub Rings: u32,
    pub Priority: u32,
    pub DeviceName: ::windows_sys::core::PCWSTR,
    pub Tsid: ::windows_sys::core::PCWSTR,
    pub Csid: ::windows_sys::core::PCWSTR,
}
impl ::core::marker::Copy for FAX_PORT_INFOW {}
impl ::core::clone::Clone for FAX_PORT_INFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_PORT_QUERY: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FAX_PORT_SET: u32 = 32u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub struct FAX_PRINT_INFOA {
    pub SizeOfStruct: u32,
    pub DocName: ::windows_sys::core::PCSTR,
    pub RecipientName: ::windows_sys::core::PCSTR,
    pub RecipientNumber: ::windows_sys::core::PCSTR,
    pub SenderName: ::windows_sys::core::PCSTR,
    pub SenderCompany: ::windows_sys::core::PCSTR,
    pub SenderDept: ::windows_sys::core::PCSTR,
    pub SenderBillingCode: ::windows_sys::core::PCSTR,
    pub Reserved: ::windows_sys::core::PCSTR,
    pub DrEmailAddress: ::windows_sys::core::PCSTR,
    pub OutputFileName: ::windows_sys::core::PCSTR,
}
impl ::core::marker::Copy for FAX_PRINT_INFOA {}
impl ::core::clone::Clone for FAX_PRINT_INFOA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub struct FAX_PRINT_INFOW {
    pub SizeOfStruct: u32,
    pub DocName: ::windows_sys::core::PCWSTR,
    pub RecipientName: ::windows_sys::core::PCWSTR,
    pub RecipientNumber: ::windows_sys::core::PCWSTR,
    pub SenderName: ::windows_sys::core::PCWSTR,
    pub SenderCompany: ::windows_sys::core::PCWSTR,
    pub SenderDept: ::windows_sys::core::PCWSTR,
    pub SenderBillingCode: ::windows_sys::core::PCWSTR,
    pub Reserved: ::windows_sys::core::PCWSTR,
    pub DrEmailAddress: ::windows_sys::core::PCWSTR,
    pub OutputFileName: ::windows_sys::core::PCWSTR,
}
impl ::core::marker::Copy for FAX_PRINT_INFOW {}
impl ::core::clone::Clone for FAX_PRINT_INFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type FAX_PRIORITY_TYPE_ENUM = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fptLOW: FAX_PRIORITY_TYPE_ENUM = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fptNORMAL: FAX_PRIORITY_TYPE_ENUM = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fptHIGH: FAX_PRIORITY_TYPE_ENUM = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type FAX_PROVIDER_STATUS_ENUM = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fpsSUCCESS: FAX_PROVIDER_STATUS_ENUM = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fpsSERVER_ERROR: FAX_PROVIDER_STATUS_ENUM = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fpsBAD_GUID: FAX_PROVIDER_STATUS_ENUM = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fpsBAD_VERSION: FAX_PROVIDER_STATUS_ENUM = 3i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fpsCANT_LOAD: FAX_PROVIDER_STATUS_ENUM = 4i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fpsCANT_LINK: FAX_PROVIDER_STATUS_ENUM = 5i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fpsCANT_INIT: FAX_PROVIDER_STATUS_ENUM = 6i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type FAX_RECEIPT_TYPE_ENUM = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const frtNONE: FAX_RECEIPT_TYPE_ENUM = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const frtMAIL: FAX_RECEIPT_TYPE_ENUM = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const frtMSGBOX: FAX_RECEIPT_TYPE_ENUM = 4i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub struct FAX_RECEIVE {
    pub SizeOfStruct: u32,
    pub FileName: ::windows_sys::core::PWSTR,
    pub ReceiverName: ::windows_sys::core::PWSTR,
    pub ReceiverNumber: ::windows_sys::core::PWSTR,
    pub Reserved: [u32; 4],
}
impl ::core::marker::Copy for FAX_RECEIVE {}
impl ::core::clone::Clone for FAX_RECEIVE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub struct FAX_ROUTE {
    pub SizeOfStruct: u32,
    pub JobId: u32,
    pub ElapsedTime: u64,
    pub ReceiveTime: u64,
    pub PageCount: u32,
    pub Csid: ::windows_sys::core::PCWSTR,
    pub Tsid: ::windows_sys::core::PCWSTR,
    pub CallerId: ::windows_sys::core::PCWSTR,
    pub RoutingInfo: ::windows_sys::core::PCWSTR,
    pub ReceiverName: ::windows_sys::core::PCWSTR,
    pub ReceiverNumber: ::windows_sys::core::PCWSTR,
    pub DeviceName: ::windows_sys::core::PCWSTR,
    pub DeviceId: u32,
    pub RoutingInfoData: *mut u8,
    pub RoutingInfoDataSize: u32,
}
impl ::core::marker::Copy for FAX_ROUTE {}
impl ::core::clone::Clone for FAX_ROUTE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FAX_ROUTE_CALLBACKROUTINES {
    pub SizeOfStruct: u32,
    pub FaxRouteAddFile: PFAXROUTEADDFILE,
    pub FaxRouteDeleteFile: PFAXROUTEDELETEFILE,
    pub FaxRouteGetFile: PFAXROUTEGETFILE,
    pub FaxRouteEnumFiles: PFAXROUTEENUMFILES,
    pub FaxRouteModifyRoutingData: PFAXROUTEMODIFYROUTINGDATA,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FAX_ROUTE_CALLBACKROUTINES {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FAX_ROUTE_CALLBACKROUTINES {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FAX_ROUTING_METHODA {
    pub SizeOfStruct: u32,
    pub DeviceId: u32,
    pub Enabled: super::super::Foundation::BOOL,
    pub DeviceName: ::windows_sys::core::PCSTR,
    pub Guid: ::windows_sys::core::PCSTR,
    pub FriendlyName: ::windows_sys::core::PCSTR,
    pub FunctionName: ::windows_sys::core::PCSTR,
    pub ExtensionImageName: ::windows_sys::core::PCSTR,
    pub ExtensionFriendlyName: ::windows_sys::core::PCSTR,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FAX_ROUTING_METHODA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FAX_ROUTING_METHODA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FAX_ROUTING_METHODW {
    pub SizeOfStruct: u32,
    pub DeviceId: u32,
    pub Enabled: super::super::Foundation::BOOL,
    pub DeviceName: ::windows_sys::core::PCWSTR,
    pub Guid: ::windows_sys::core::PCWSTR,
    pub FriendlyName: ::windows_sys::core::PCWSTR,
    pub FunctionName: ::windows_sys::core::PCWSTR,
    pub ExtensionImageName: ::windows_sys::core::PCWSTR,
    pub ExtensionFriendlyName: ::windows_sys::core::PCWSTR,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FAX_ROUTING_METHODW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FAX_ROUTING_METHODW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type FAX_ROUTING_RULE_CODE_ENUM = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const frrcANY_CODE: FAX_ROUTING_RULE_CODE_ENUM = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type FAX_RULE_STATUS_ENUM = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const frsVALID: FAX_RULE_STATUS_ENUM = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const frsEMPTY_GROUP: FAX_RULE_STATUS_ENUM = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const frsALL_GROUP_DEV_NOT_VALID: FAX_RULE_STATUS_ENUM = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const frsSOME_GROUP_DEV_NOT_VALID: FAX_RULE_STATUS_ENUM = 3i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const frsBAD_DEVICE: FAX_RULE_STATUS_ENUM = 4i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type FAX_SCHEDULE_TYPE_ENUM = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fstNOW: FAX_SCHEDULE_TYPE_ENUM = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fstSPECIFIC_TIME: FAX_SCHEDULE_TYPE_ENUM = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fstDISCOUNT_PERIOD: FAX_SCHEDULE_TYPE_ENUM = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FAX_SEND {
    pub SizeOfStruct: u32,
    pub FileName: ::windows_sys::core::PWSTR,
    pub CallerName: ::windows_sys::core::PWSTR,
    pub CallerNumber: ::windows_sys::core::PWSTR,
    pub ReceiverName: ::windows_sys::core::PWSTR,
    pub ReceiverNumber: ::windows_sys::core::PWSTR,
    pub Branding: super::super::Foundation::BOOL,
    pub CallHandle: u32,
    pub Reserved: [u32; 3],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FAX_SEND {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FAX_SEND {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type FAX_SERVER_APIVERSION_ENUM = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fsAPI_VERSION_0: FAX_SERVER_APIVERSION_ENUM = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fsAPI_VERSION_1: FAX_SERVER_APIVERSION_ENUM = 65536i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fsAPI_VERSION_2: FAX_SERVER_APIVERSION_ENUM = 131072i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fsAPI_VERSION_3: FAX_SERVER_APIVERSION_ENUM = 196608i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type FAX_SERVER_EVENTS_TYPE_ENUM = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fsetNONE: FAX_SERVER_EVENTS_TYPE_ENUM = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fsetIN_QUEUE: FAX_SERVER_EVENTS_TYPE_ENUM = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fsetOUT_QUEUE: FAX_SERVER_EVENTS_TYPE_ENUM = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fsetCONFIG: FAX_SERVER_EVENTS_TYPE_ENUM = 4i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fsetACTIVITY: FAX_SERVER_EVENTS_TYPE_ENUM = 8i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fsetQUEUE_STATE: FAX_SERVER_EVENTS_TYPE_ENUM = 16i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fsetIN_ARCHIVE: FAX_SERVER_EVENTS_TYPE_ENUM = 32i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fsetOUT_ARCHIVE: FAX_SERVER_EVENTS_TYPE_ENUM = 64i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fsetFXSSVC_ENDED: FAX_SERVER_EVENTS_TYPE_ENUM = 128i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fsetDEVICE_STATUS: FAX_SERVER_EVENTS_TYPE_ENUM = 256i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fsetINCOMING_CALL: FAX_SERVER_EVENTS_TYPE_ENUM = 512i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type FAX_SMTP_AUTHENTICATION_TYPE_ENUM = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fsatANONYMOUS: FAX_SMTP_AUTHENTICATION_TYPE_ENUM = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fsatBASIC: FAX_SMTP_AUTHENTICATION_TYPE_ENUM = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const fsatNTLM: FAX_SMTP_AUTHENTICATION_TYPE_ENUM = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub struct FAX_TIME {
    pub Hour: u16,
    pub Minute: u16,
}
impl ::core::marker::Copy for FAX_TIME {}
impl ::core::clone::Clone for FAX_TIME {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FEI_ABORTING: u32 = 15u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FEI_ANSWERED: u32 = 21u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FEI_BAD_ADDRESS: u32 = 7u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FEI_BUSY: u32 = 5u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FEI_CALL_BLACKLISTED: u32 = 13u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FEI_CALL_DELAYED: u32 = 12u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FEI_COMPLETED: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FEI_DELETED: u32 = 23u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FEI_DIALING: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FEI_DISCONNECTED: u32 = 9u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FEI_FATAL_ERROR: u32 = 10u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FEI_FAXSVC_ENDED: u32 = 20u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FEI_FAXSVC_STARTED: u32 = 27u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FEI_HANDLED: u32 = 26u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FEI_IDLE: u32 = 19u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FEI_INITIALIZING: u32 = 24u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FEI_JOB_QUEUED: u32 = 22u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FEI_LINE_UNAVAILABLE: u32 = 25u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FEI_MODEM_POWERED_OFF: u32 = 18u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FEI_MODEM_POWERED_ON: u32 = 17u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FEI_NEVENTS: u32 = 27u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FEI_NOT_FAX_CALL: u32 = 11u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FEI_NO_ANSWER: u32 = 6u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FEI_NO_DIAL_TONE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FEI_RECEIVING: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FEI_RINGING: u32 = 14u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FEI_ROUTING: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FEI_SENDING: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FPF_RECEIVE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FPF_SEND: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FPF_VIRTUAL: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FPS_ABORTING: u32 = 538968064u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FPS_ANSWERED: u32 = 545259520u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FPS_AVAILABLE: u32 = 537919488u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FPS_BAD_ADDRESS: u32 = 536871168u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FPS_BUSY: u32 = 536870976u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FPS_CALL_BLACKLISTED: u32 = 536887296u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FPS_CALL_DELAYED: u32 = 536879104u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FPS_COMPLETED: u32 = 536870920u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FPS_DIALING: u32 = 536870913u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FPS_DISCONNECTED: u32 = 536871936u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FPS_FATAL_ERROR: u32 = 536872960u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FPS_HANDLED: u32 = 536870928u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FPS_INITIALIZING: u32 = 536903680u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FPS_NOT_FAX_CALL: u32 = 536875008u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FPS_NO_ANSWER: u32 = 536871040u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FPS_NO_DIAL_TONE: u32 = 536871424u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FPS_OFFLINE: u32 = 536936448u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FPS_RECEIVING: u32 = 536870916u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FPS_RINGING: u32 = 537001984u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FPS_ROUTING: u32 = 541065216u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FPS_SENDING: u32 = 536870914u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FPS_UNAVAILABLE: u32 = 536870944u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FS_ANSWERED: u32 = 545259520u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FS_BAD_ADDRESS: u32 = 536871168u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FS_BUSY: u32 = 536870976u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FS_CALL_BLACKLISTED: u32 = 536887296u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FS_CALL_DELAYED: u32 = 536879104u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FS_COMPLETED: u32 = 536870920u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FS_DIALING: u32 = 536870913u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FS_DISCONNECTED: u32 = 536871936u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FS_FATAL_ERROR: u32 = 536872960u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FS_HANDLED: u32 = 536870928u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FS_INITIALIZING: u32 = 536870912u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FS_LINE_UNAVAILABLE: u32 = 536870944u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FS_NOT_FAX_CALL: u32 = 536875008u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FS_NO_ANSWER: u32 = 536871040u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FS_NO_DIAL_TONE: u32 = 536871424u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FS_RECEIVING: u32 = 536870916u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FS_TRANSMITTING: u32 = 536870914u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const FS_USER_ABORT: u32 = 538968064u32;
pub const FaxAccount: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2816500863, data2: 17700, data3: 17508, data4: [165, 109, 185, 254, 102, 111, 113, 94] };
pub const FaxAccountFolders: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2235141961, data2: 49204, data3: 19007, data4: [130, 28, 219, 125, 104, 94, 129, 41] };
pub const FaxAccountIncomingArchive: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 347291061, data2: 19520, data3: 20175, data4: [158, 248, 163, 96, 203, 232, 9, 237] };
pub const FaxAccountIncomingQueue: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2614059156, data2: 46298, data3: 17908, data4: [184, 214, 221, 235, 33, 134, 101, 44] };
pub const FaxAccountOutgoingArchive: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2233367285, data2: 17210, data3: 18233, data4: [162, 223, 173, 36, 92, 44, 185, 142] };
pub const FaxAccountOutgoingQueue: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4276940539, data2: 49481, data3: 18618, data4: [186, 184, 183, 145, 225, 1, 246, 47] };
pub const FaxAccountSet: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4223810635, data2: 31200, data3: 17041, data4: [188, 86, 193, 46, 37, 59, 191, 58] };
pub const FaxAccounts: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3659502762, data2: 60972, data3: 18368, data4: [143, 79, 42, 33, 112, 117, 183, 110] };
pub const FaxActivity: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3488570638, data2: 59469, data3: 17966, data4: [170, 187, 135, 211, 30, 176, 79, 239] };
pub const FaxActivityLogging: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4037028174, data2: 15293, data3: 18616, data4: [143, 19, 140, 89, 26, 85, 189, 188] };
pub const FaxConfiguration: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1482109551, data2: 59315, data3: 16807, data4: [156, 25, 169, 27, 70, 62, 45, 86] };
pub const FaxDevice: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1508091314, data2: 54902, data3: 18507, data4: [166, 222, 114, 11, 250, 137, 181, 175] };
pub const FaxDeviceIds: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3452254698, data2: 29303, data3: 17934, data4: [141, 224, 72, 160, 165, 118, 13, 31] };
pub const FaxDeviceProvider: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 399448739, data2: 62955, data3: 18506, data4: [156, 154, 68, 64, 165, 186, 171, 252] };
pub const FaxDeviceProviders: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3952076648, data2: 34650, data3: 20319, data4: [130, 197, 3, 242, 58, 172, 27, 215] };
pub const FaxDevices: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1435099790, data2: 9163, data3: 18713, data4: [136, 8, 230, 16, 24, 70, 232, 13] };
pub const FaxDocument: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 255827857, data2: 51256, data3: 16734, data4: [164, 243, 62, 130, 140, 164, 69, 224] };
pub const FaxEventLogging: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2793736496, data2: 41206, data3: 19055, data4: [149, 183, 219, 46, 191, 61, 2, 227] };
pub const FaxFolders: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3276935639, data2: 22390, data3: 18635, data4: [175, 68, 195, 27, 227, 178, 207, 229] };
pub const FaxInboundRouting: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3892463853, data2: 44389, data3: 16920, data4: [129, 8, 153, 25, 36, 212, 231, 237] };
pub const FaxInboundRoutingExtension: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 494795601, data2: 29191, data3: 17462, data4: [160, 217, 36, 227, 46, 229, 105, 136] };
pub const FaxInboundRoutingExtensions: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 412764397, data2: 25148, data3: 19469, data4: [128, 242, 214, 108, 123, 158, 254, 194] };
pub const FaxInboundRoutingMethod: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1268766556, data2: 404, data3: 19314, data4: [156, 229, 2, 168, 32, 90, 199, 212] };
pub const FaxInboundRoutingMethods: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 637319018, data2: 46928, data3: 19330, data4: [146, 102, 251, 187, 174, 137, 34, 186] };
pub const FaxIncomingArchive: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2217133418, data2: 13729, data3: 19567, data4: [175, 147, 252, 149, 36, 34, 226, 194] };
pub const FaxIncomingJob: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3295875564, data2: 44594, data3: 16824, data4: [174, 75, 62, 174, 6, 41, 208, 201] };
pub const FaxIncomingJobs: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2713422403, data2: 34918, data3: 20407, data4: [161, 93, 98, 102, 200, 117, 165, 204] };
pub const FaxIncomingMessage: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 422771959, data2: 40259, data3: 19802, data4: [137, 255, 3, 134, 27, 50, 23, 54] };
pub const FaxIncomingMessageIterator: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1619583448, data2: 16328, data3: 17858, data4: [135, 177, 144, 154, 41, 96, 126, 169] };
pub const FaxIncomingQueue: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1762858775, data2: 62449, data3: 16611, data4: [128, 157, 166, 203, 247, 189, 133, 229] };
pub const FaxJobStatus: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2079466228, data2: 48781, data3: 17455, data4: [132, 29, 97, 50, 116, 36, 35, 187] };
pub const FaxLoggingOptions: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 469364390, data2: 60640, data3: 18309, data4: [161, 139, 222, 86, 233, 238, 249, 106] };
pub const FaxOutboundRouting: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3357227102, data2: 47209, data3: 19197, data4: [134, 192, 97, 100, 152, 237, 155, 226] };
pub const FaxOutboundRoutingGroup: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 34862048, data2: 26513, data3: 19831, data4: [162, 113, 4, 210, 53, 124, 80, 214] };
pub const FaxOutboundRoutingGroups: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3435045285, data2: 58036, data3: 19287, data4: [148, 33, 176, 75, 98, 137, 70, 75] };
pub const FaxOutboundRoutingRule: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1699344063, data2: 2257, data3: 18266, data4: [130, 139, 59, 241, 5, 149, 47, 160] };
pub const FaxOutboundRoutingRules: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3548757706, data2: 58916, data3: 17523, data4: [191, 170, 159, 64, 0, 131, 31, 84] };
pub const FaxOutgoingArchive: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1136821251, data2: 57423, data3: 18253, data4: [153, 12, 185, 70, 105, 20, 143, 89] };
pub const FaxOutgoingJob: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1908097692, data2: 3833, data3: 18709, data4: [190, 197, 165, 216, 151, 163, 233, 36] };
pub const FaxOutgoingJobs: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2462001772, data2: 14270, data3: 17402, data4: [163, 125, 203, 14, 95, 117, 59, 53] };
pub const FaxOutgoingMessage: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2444534648, data2: 19160, data3: 19183, data4: [164, 220, 151, 217, 110, 147, 154, 58] };
pub const FaxOutgoingMessageIterator: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2318542032, data2: 54027, data3: 18910, data4: [152, 19, 203, 56, 87, 144, 251, 187] };
pub const FaxOutgoingQueue: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1948325534, data2: 35907, data3: 19213, data4: [187, 22, 100, 92, 143, 164, 3, 87] };
pub const FaxReceiptOptions: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1770145915, data2: 8827, data3: 19606, data4: [166, 28, 36, 131, 72, 176, 90, 182] };
pub const FaxRecipient: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1623143169, data2: 32248, data3: 19416, data4: [145, 72, 123, 88, 1, 249, 239, 223] };
pub const FaxRecipients: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3936083795, data2: 4265, data3: 19791, data4: [160, 103, 99, 200, 248, 79, 1, 176] };
pub const FaxSecurity: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 281337310, data2: 44016, data3: 17375, data4: [150, 79, 127, 58, 194, 26, 76, 123] };
pub const FaxSecurity2: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1935413832, data2: 60553, data3: 19504, data4: [161, 39, 101, 110, 146, 227, 196, 234] };
pub const FaxSender: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 643663056, data2: 6224, data3: 17248, data4: [183, 200, 117, 139, 187, 95, 11, 150] };
pub const FaxServer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3450383536, data2: 36085, data3: 20332, data4: [155, 162, 89, 49, 212, 12, 140, 174] };
pub const GUID_DeviceArrivedLaunch: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1947049702, data2: 28913, data3: 4561, data4: [173, 16, 0, 160, 36, 56, 173, 72] };
pub const GUID_STIUserDefined1: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3222189973, data2: 35950, data3: 4562, data4: [151, 122, 0, 0, 248, 122, 146, 111] };
pub const GUID_STIUserDefined2: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3346721221, data2: 35950, data3: 4562, data4: [151, 122, 0, 0, 248, 122, 146, 111] };
pub const GUID_STIUserDefined3: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3346721222, data2: 35950, data3: 4562, data4: [151, 122, 0, 0, 248, 122, 146, 111] };
pub const GUID_ScanFaxImage: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3222189971, data2: 35950, data3: 4562, data4: [151, 122, 0, 0, 248, 122, 146, 111] };
pub const GUID_ScanImage: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2797971221, data2: 35950, data3: 4562, data4: [151, 122, 0, 0, 248, 122, 146, 111] };
pub const GUID_ScanPrintImage: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3024221221, data2: 35950, data3: 4562, data4: [151, 122, 0, 0, 248, 122, 146, 111] };
pub type IFaxAccount = *mut ::core::ffi::c_void;
pub type IFaxAccountFolders = *mut ::core::ffi::c_void;
pub type IFaxAccountIncomingArchive = *mut ::core::ffi::c_void;
pub type IFaxAccountIncomingQueue = *mut ::core::ffi::c_void;
pub type IFaxAccountNotify = *mut ::core::ffi::c_void;
pub type IFaxAccountOutgoingArchive = *mut ::core::ffi::c_void;
pub type IFaxAccountOutgoingQueue = *mut ::core::ffi::c_void;
pub type IFaxAccountSet = *mut ::core::ffi::c_void;
pub type IFaxAccounts = *mut ::core::ffi::c_void;
pub type IFaxActivity = *mut ::core::ffi::c_void;
pub type IFaxActivityLogging = *mut ::core::ffi::c_void;
pub type IFaxConfiguration = *mut ::core::ffi::c_void;
pub type IFaxDevice = *mut ::core::ffi::c_void;
pub type IFaxDeviceIds = *mut ::core::ffi::c_void;
pub type IFaxDeviceProvider = *mut ::core::ffi::c_void;
pub type IFaxDeviceProviders = *mut ::core::ffi::c_void;
pub type IFaxDevices = *mut ::core::ffi::c_void;
pub type IFaxDocument = *mut ::core::ffi::c_void;
pub type IFaxDocument2 = *mut ::core::ffi::c_void;
pub type IFaxEventLogging = *mut ::core::ffi::c_void;
pub type IFaxFolders = *mut ::core::ffi::c_void;
pub type IFaxInboundRouting = *mut ::core::ffi::c_void;
pub type IFaxInboundRoutingExtension = *mut ::core::ffi::c_void;
pub type IFaxInboundRoutingExtensions = *mut ::core::ffi::c_void;
pub type IFaxInboundRoutingMethod = *mut ::core::ffi::c_void;
pub type IFaxInboundRoutingMethods = *mut ::core::ffi::c_void;
pub type IFaxIncomingArchive = *mut ::core::ffi::c_void;
pub type IFaxIncomingJob = *mut ::core::ffi::c_void;
pub type IFaxIncomingJobs = *mut ::core::ffi::c_void;
pub type IFaxIncomingMessage = *mut ::core::ffi::c_void;
pub type IFaxIncomingMessage2 = *mut ::core::ffi::c_void;
pub type IFaxIncomingMessageIterator = *mut ::core::ffi::c_void;
pub type IFaxIncomingQueue = *mut ::core::ffi::c_void;
pub type IFaxJobStatus = *mut ::core::ffi::c_void;
pub type IFaxLoggingOptions = *mut ::core::ffi::c_void;
pub type IFaxOutboundRouting = *mut ::core::ffi::c_void;
pub type IFaxOutboundRoutingGroup = *mut ::core::ffi::c_void;
pub type IFaxOutboundRoutingGroups = *mut ::core::ffi::c_void;
pub type IFaxOutboundRoutingRule = *mut ::core::ffi::c_void;
pub type IFaxOutboundRoutingRules = *mut ::core::ffi::c_void;
pub type IFaxOutgoingArchive = *mut ::core::ffi::c_void;
pub type IFaxOutgoingJob = *mut ::core::ffi::c_void;
pub type IFaxOutgoingJob2 = *mut ::core::ffi::c_void;
pub type IFaxOutgoingJobs = *mut ::core::ffi::c_void;
pub type IFaxOutgoingMessage = *mut ::core::ffi::c_void;
pub type IFaxOutgoingMessage2 = *mut ::core::ffi::c_void;
pub type IFaxOutgoingMessageIterator = *mut ::core::ffi::c_void;
pub type IFaxOutgoingQueue = *mut ::core::ffi::c_void;
pub type IFaxReceiptOptions = *mut ::core::ffi::c_void;
pub type IFaxRecipient = *mut ::core::ffi::c_void;
pub type IFaxRecipients = *mut ::core::ffi::c_void;
pub type IFaxSecurity = *mut ::core::ffi::c_void;
pub type IFaxSecurity2 = *mut ::core::ffi::c_void;
pub type IFaxSender = *mut ::core::ffi::c_void;
pub type IFaxServer = *mut ::core::ffi::c_void;
pub type IFaxServer2 = *mut ::core::ffi::c_void;
pub type IFaxServerNotify = *mut ::core::ffi::c_void;
pub type IFaxServerNotify2 = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const IS_DIGITAL_CAMERA_STR: &str = "IsDigitalCamera";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const IS_DIGITAL_CAMERA_VAL: u32 = 1u32;
pub type IStiDevice = *mut ::core::ffi::c_void;
pub type IStiDeviceControl = *mut ::core::ffi::c_void;
#[repr(C)]
pub struct IStiDeviceW(pub u8);
pub type IStiUSD = *mut ::core::ffi::c_void;
pub type IStillImageW = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const JS_DELETING: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const JS_FAILED: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const JS_INPROGRESS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const JS_NOLINE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const JS_PAUSED: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const JS_PENDING: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const JS_RETRIES_EXCEEDED: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const JS_RETRYING: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const JT_FAIL_RECEIVE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const JT_RECEIVE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const JT_ROUTING: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const JT_SEND: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const JT_UNKNOWN: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const MAX_NOTIFICATION_DATA: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const MS_FAXROUTE_EMAIL_GUID: &str = "{6bbf7bfe-9af2-11d0-abf7-00c04fd91a4e}";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const MS_FAXROUTE_FOLDER_GUID: &str = "{92041a90-9af2-11d0-abf7-00c04fd91a4e}";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const MS_FAXROUTE_PRINTING_GUID: &str = "{aec1b37c-9af2-11d0-abf7-00c04fd91a4e}";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXABORT = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, jobid: u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXACCESSCHECK = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, accessmask: u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXCLOSE = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXCOMPLETEJOBPARAMSA = ::core::option::Option<unsafe extern "system" fn(jobparams: *mut *mut FAX_JOB_PARAMA, coverpageinfo: *mut *mut FAX_COVERPAGE_INFOA) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXCOMPLETEJOBPARAMSW = ::core::option::Option<unsafe extern "system" fn(jobparams: *mut *mut FAX_JOB_PARAMW, coverpageinfo: *mut *mut FAX_COVERPAGE_INFOW) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXCONNECTFAXSERVERA = ::core::option::Option<unsafe extern "system" fn(machinename: ::windows_sys::core::PCSTR, faxhandle: *mut super::super::Foundation::HANDLE) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXCONNECTFAXSERVERW = ::core::option::Option<unsafe extern "system" fn(machinename: ::windows_sys::core::PCWSTR, faxhandle: *mut super::super::Foundation::HANDLE) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXDEVABORTOPERATION = ::core::option::Option<unsafe extern "system" fn(param0: super::super::Foundation::HANDLE) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Controls\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Controls"))]
pub type PFAXDEVCONFIGURE = ::core::option::Option<unsafe extern "system" fn(param0: *mut super::super::UI::Controls::HPROPSHEETPAGE) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXDEVENDJOB = ::core::option::Option<unsafe extern "system" fn(param0: super::super::Foundation::HANDLE) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXDEVINITIALIZE = ::core::option::Option<unsafe extern "system" fn(param0: u32, param1: super::super::Foundation::HANDLE, param2: *mut PFAX_LINECALLBACK, param3: PFAX_SERVICE_CALLBACK) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXDEVRECEIVE = ::core::option::Option<unsafe extern "system" fn(param0: super::super::Foundation::HANDLE, param1: u32, param2: *mut FAX_RECEIVE) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXDEVREPORTSTATUS = ::core::option::Option<unsafe extern "system" fn(param0: super::super::Foundation::HANDLE, param1: *mut FAX_DEV_STATUS, param2: u32, param3: *mut u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXDEVSEND = ::core::option::Option<unsafe extern "system" fn(param0: super::super::Foundation::HANDLE, param1: *mut FAX_SEND, param2: PFAX_SEND_CALLBACK) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type PFAXDEVSHUTDOWN = ::core::option::Option<unsafe extern "system" fn() -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXDEVSTARTJOB = ::core::option::Option<unsafe extern "system" fn(param0: u32, param1: u32, param2: *mut super::super::Foundation::HANDLE, param3: super::super::Foundation::HANDLE, param4: usize) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXDEVVIRTUALDEVICECREATION = ::core::option::Option<unsafe extern "system" fn(devicecount: *mut u32, devicenameprefix: ::windows_sys::core::PWSTR, deviceidprefix: *mut u32, completionport: super::super::Foundation::HANDLE, completionkey: usize) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXENABLEROUTINGMETHODA = ::core::option::Option<unsafe extern "system" fn(faxporthandle: super::super::Foundation::HANDLE, routingguid: ::windows_sys::core::PCSTR, enabled: super::super::Foundation::BOOL) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXENABLEROUTINGMETHODW = ::core::option::Option<unsafe extern "system" fn(faxporthandle: super::super::Foundation::HANDLE, routingguid: ::windows_sys::core::PCWSTR, enabled: super::super::Foundation::BOOL) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXENUMGLOBALROUTINGINFOA = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, routinginfo: *mut *mut FAX_GLOBAL_ROUTING_INFOA, methodsreturned: *mut u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXENUMGLOBALROUTINGINFOW = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, routinginfo: *mut *mut FAX_GLOBAL_ROUTING_INFOW, methodsreturned: *mut u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXENUMJOBSA = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, jobentry: *mut *mut FAX_JOB_ENTRYA, jobsreturned: *mut u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXENUMJOBSW = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, jobentry: *mut *mut FAX_JOB_ENTRYW, jobsreturned: *mut u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXENUMPORTSA = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, portinfo: *mut *mut FAX_PORT_INFOA, portsreturned: *mut u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXENUMPORTSW = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, portinfo: *mut *mut FAX_PORT_INFOW, portsreturned: *mut u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXENUMROUTINGMETHODSA = ::core::option::Option<unsafe extern "system" fn(faxporthandle: super::super::Foundation::HANDLE, routingmethod: *mut *mut FAX_ROUTING_METHODA, methodsreturned: *mut u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXENUMROUTINGMETHODSW = ::core::option::Option<unsafe extern "system" fn(faxporthandle: super::super::Foundation::HANDLE, routingmethod: *mut *mut FAX_ROUTING_METHODW, methodsreturned: *mut u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type PFAXFREEBUFFER = ::core::option::Option<unsafe extern "system" fn(buffer: *mut ::core::ffi::c_void)>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXGETCONFIGURATIONA = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, faxconfig: *mut *mut FAX_CONFIGURATIONA) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXGETCONFIGURATIONW = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, faxconfig: *mut *mut FAX_CONFIGURATIONW) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXGETDEVICESTATUSA = ::core::option::Option<unsafe extern "system" fn(faxporthandle: super::super::Foundation::HANDLE, devicestatus: *mut *mut FAX_DEVICE_STATUSA) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXGETDEVICESTATUSW = ::core::option::Option<unsafe extern "system" fn(faxporthandle: super::super::Foundation::HANDLE, devicestatus: *mut *mut FAX_DEVICE_STATUSW) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXGETJOBA = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, jobid: u32, jobentry: *mut *mut FAX_JOB_ENTRYA) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXGETJOBW = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, jobid: u32, jobentry: *mut *mut FAX_JOB_ENTRYW) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXGETLOGGINGCATEGORIESA = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, categories: *mut *mut FAX_LOG_CATEGORYA, numbercategories: *mut u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXGETLOGGINGCATEGORIESW = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, categories: *mut *mut FAX_LOG_CATEGORYW, numbercategories: *mut u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXGETPAGEDATA = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, jobid: u32, buffer: *mut *mut u8, buffersize: *mut u32, imagewidth: *mut u32, imageheight: *mut u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXGETPORTA = ::core::option::Option<unsafe extern "system" fn(faxporthandle: super::super::Foundation::HANDLE, portinfo: *mut *mut FAX_PORT_INFOA) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXGETPORTW = ::core::option::Option<unsafe extern "system" fn(faxporthandle: super::super::Foundation::HANDLE, portinfo: *mut *mut FAX_PORT_INFOW) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXGETROUTINGINFOA = ::core::option::Option<unsafe extern "system" fn(faxporthandle: super::super::Foundation::HANDLE, routingguid: ::windows_sys::core::PCSTR, routinginfobuffer: *mut *mut u8, routinginfobuffersize: *mut u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXGETROUTINGINFOW = ::core::option::Option<unsafe extern "system" fn(faxporthandle: super::super::Foundation::HANDLE, routingguid: ::windows_sys::core::PCWSTR, routinginfobuffer: *mut *mut u8, routinginfobuffersize: *mut u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXINITIALIZEEVENTQUEUE = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, completionport: super::super::Foundation::HANDLE, completionkey: usize, hwnd: super::super::Foundation::HWND, messagestart: u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXOPENPORT = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, deviceid: u32, flags: u32, faxporthandle: *mut super::super::Foundation::HANDLE) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type PFAXPRINTCOVERPAGEA = ::core::option::Option<unsafe extern "system" fn(faxcontextinfo: *const FAX_CONTEXT_INFOA, coverpageinfo: *const FAX_COVERPAGE_INFOA) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type PFAXPRINTCOVERPAGEW = ::core::option::Option<unsafe extern "system" fn(faxcontextinfo: *const FAX_CONTEXT_INFOW, coverpageinfo: *const FAX_COVERPAGE_INFOW) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXREGISTERROUTINGEXTENSIONW = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, extensionname: ::windows_sys::core::PCWSTR, friendlyname: ::windows_sys::core::PCWSTR, imagename: ::windows_sys::core::PCWSTR, callback: PFAX_ROUTING_INSTALLATION_CALLBACKW, context: *mut ::core::ffi::c_void) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXREGISTERSERVICEPROVIDERW = ::core::option::Option<unsafe extern "system" fn(deviceprovider: ::windows_sys::core::PCWSTR, friendlyname: ::windows_sys::core::PCWSTR, imagename: ::windows_sys::core::PCWSTR, tspname: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type PFAXROUTEADDFILE = ::core::option::Option<unsafe extern "system" fn(jobid: u32, filename: ::windows_sys::core::PCWSTR, guid: *mut ::windows_sys::core::GUID) -> i32>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type PFAXROUTEDELETEFILE = ::core::option::Option<unsafe extern "system" fn(jobid: u32, filename: ::windows_sys::core::PCWSTR) -> i32>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXROUTEDEVICECHANGENOTIFICATION = ::core::option::Option<unsafe extern "system" fn(param0: u32, param1: super::super::Foundation::BOOL) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXROUTEDEVICEENABLE = ::core::option::Option<unsafe extern "system" fn(param0: ::windows_sys::core::PCWSTR, param1: u32, param2: i32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXROUTEENUMFILE = ::core::option::Option<unsafe extern "system" fn(jobid: u32, guidowner: *mut ::windows_sys::core::GUID, guidcaller: *mut ::windows_sys::core::GUID, filename: ::windows_sys::core::PCWSTR, context: *mut ::core::ffi::c_void) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXROUTEENUMFILES = ::core::option::Option<unsafe extern "system" fn(jobid: u32, guid: *mut ::windows_sys::core::GUID, fileenumerator: PFAXROUTEENUMFILE, context: *mut ::core::ffi::c_void) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXROUTEGETFILE = ::core::option::Option<unsafe extern "system" fn(jobid: u32, index: u32, filenamebuffer: ::windows_sys::core::PWSTR, requiredsize: *mut u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXROUTEGETROUTINGINFO = ::core::option::Option<unsafe extern "system" fn(param0: ::windows_sys::core::PCWSTR, param1: u32, param2: *mut u8, param3: *mut u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXROUTEINITIALIZE = ::core::option::Option<unsafe extern "system" fn(param0: super::super::Foundation::HANDLE, param1: *mut FAX_ROUTE_CALLBACKROUTINES) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXROUTEMETHOD = ::core::option::Option<unsafe extern "system" fn(param0: *const FAX_ROUTE, param1: *mut *mut ::core::ffi::c_void, param2: *mut u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXROUTEMODIFYROUTINGDATA = ::core::option::Option<unsafe extern "system" fn(jobid: u32, routingguid: ::windows_sys::core::PCWSTR, routingdata: *mut u8, routingdatasize: u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXROUTESETROUTINGINFO = ::core::option::Option<unsafe extern "system" fn(param0: ::windows_sys::core::PCWSTR, param1: u32, param2: *const u8, param3: u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXSENDDOCUMENTA = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, filename: ::windows_sys::core::PCSTR, jobparams: *mut FAX_JOB_PARAMA, coverpageinfo: *const FAX_COVERPAGE_INFOA, faxjobid: *mut u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXSENDDOCUMENTFORBROADCASTA = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, filename: ::windows_sys::core::PCSTR, faxjobid: *mut u32, faxrecipientcallback: PFAX_RECIPIENT_CALLBACKA, context: *mut ::core::ffi::c_void) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXSENDDOCUMENTFORBROADCASTW = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, filename: ::windows_sys::core::PCWSTR, faxjobid: *mut u32, faxrecipientcallback: PFAX_RECIPIENT_CALLBACKW, context: *mut ::core::ffi::c_void) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXSENDDOCUMENTW = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, filename: ::windows_sys::core::PCWSTR, jobparams: *mut FAX_JOB_PARAMW, coverpageinfo: *const FAX_COVERPAGE_INFOW, faxjobid: *mut u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXSETCONFIGURATIONA = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, faxconfig: *const FAX_CONFIGURATIONA) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXSETCONFIGURATIONW = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, faxconfig: *const FAX_CONFIGURATIONW) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXSETGLOBALROUTINGINFOA = ::core::option::Option<unsafe extern "system" fn(faxporthandle: super::super::Foundation::HANDLE, routinginfo: *const FAX_GLOBAL_ROUTING_INFOA) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXSETGLOBALROUTINGINFOW = ::core::option::Option<unsafe extern "system" fn(faxporthandle: super::super::Foundation::HANDLE, routinginfo: *const FAX_GLOBAL_ROUTING_INFOW) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXSETJOBA = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, jobid: u32, command: u32, jobentry: *const FAX_JOB_ENTRYA) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXSETJOBW = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, jobid: u32, command: u32, jobentry: *const FAX_JOB_ENTRYW) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXSETLOGGINGCATEGORIESA = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, categories: *const FAX_LOG_CATEGORYA, numbercategories: u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXSETLOGGINGCATEGORIESW = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, categories: *const FAX_LOG_CATEGORYW, numbercategories: u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXSETPORTA = ::core::option::Option<unsafe extern "system" fn(faxporthandle: super::super::Foundation::HANDLE, portinfo: *const FAX_PORT_INFOA) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXSETPORTW = ::core::option::Option<unsafe extern "system" fn(faxporthandle: super::super::Foundation::HANDLE, portinfo: *const FAX_PORT_INFOW) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXSETROUTINGINFOA = ::core::option::Option<unsafe extern "system" fn(faxporthandle: super::super::Foundation::HANDLE, routingguid: ::windows_sys::core::PCSTR, routinginfobuffer: *const u8, routinginfobuffersize: u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXSETROUTINGINFOW = ::core::option::Option<unsafe extern "system" fn(faxporthandle: super::super::Foundation::HANDLE, routingguid: ::windows_sys::core::PCWSTR, routinginfobuffer: *const u8, routinginfobuffersize: u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type PFAXSTARTPRINTJOBA = ::core::option::Option<unsafe extern "system" fn(printername: ::windows_sys::core::PCSTR, printinfo: *const FAX_PRINT_INFOA, faxjobid: *mut u32, faxcontextinfo: *mut FAX_CONTEXT_INFOA) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type PFAXSTARTPRINTJOBW = ::core::option::Option<unsafe extern "system" fn(printername: ::windows_sys::core::PCWSTR, printinfo: *const FAX_PRINT_INFOW, faxjobid: *mut u32, faxcontextinfo: *mut FAX_CONTEXT_INFOW) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAXUNREGISTERSERVICEPROVIDERW = ::core::option::Option<unsafe extern "system" fn(deviceprovider: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type PFAX_EXT_CONFIG_CHANGE = ::core::option::Option<unsafe extern "system" fn(param0: u32, param1: ::windows_sys::core::PCWSTR, param2: *mut u8, param3: u32) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type PFAX_EXT_FREE_BUFFER = ::core::option::Option<unsafe extern "system" fn(param0: *mut ::core::ffi::c_void)>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type PFAX_EXT_GET_DATA = ::core::option::Option<unsafe extern "system" fn(param0: u32, param1: FAX_ENUM_DEVICE_ID_SOURCE, param2: ::windows_sys::core::PCWSTR, param3: *mut *mut u8, param4: *mut u32) -> u32>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAX_EXT_INITIALIZE_CONFIG = ::core::option::Option<unsafe extern "system" fn(param0: PFAX_EXT_GET_DATA, param1: PFAX_EXT_SET_DATA, param2: PFAX_EXT_REGISTER_FOR_EVENTS, param3: PFAX_EXT_UNREGISTER_FOR_EVENTS, param4: PFAX_EXT_FREE_BUFFER) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAX_EXT_REGISTER_FOR_EVENTS = ::core::option::Option<unsafe extern "system" fn(param0: super::super::Foundation::HINSTANCE, param1: u32, param2: FAX_ENUM_DEVICE_ID_SOURCE, param3: ::windows_sys::core::PCWSTR, param4: PFAX_EXT_CONFIG_CHANGE) -> super::super::Foundation::HANDLE>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAX_EXT_SET_DATA = ::core::option::Option<unsafe extern "system" fn(param0: super::super::Foundation::HINSTANCE, param1: u32, param2: FAX_ENUM_DEVICE_ID_SOURCE, param3: ::windows_sys::core::PCWSTR, param4: *mut u8, param5: u32) -> u32>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAX_EXT_UNREGISTER_FOR_EVENTS = ::core::option::Option<unsafe extern "system" fn(param0: super::super::Foundation::HANDLE) -> u32>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAX_LINECALLBACK = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, hdevice: u32, dwmessage: u32, dwinstance: usize, dwparam1: usize, dwparam2: usize, dwparam3: usize)>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAX_RECIPIENT_CALLBACKA = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, recipientnumber: u32, context: *mut ::core::ffi::c_void, jobparams: *mut FAX_JOB_PARAMA, coverpageinfo: *mut FAX_COVERPAGE_INFOA) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAX_RECIPIENT_CALLBACKW = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, recipientnumber: u32, context: *mut ::core::ffi::c_void, jobparams: *mut FAX_JOB_PARAMW, coverpageinfo: *mut FAX_COVERPAGE_INFOW) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAX_ROUTING_INSTALLATION_CALLBACKW = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, context: *mut ::core::ffi::c_void, methodname: ::windows_sys::core::PCWSTR, friendlyname: ::windows_sys::core::PCWSTR, functionname: ::windows_sys::core::PCWSTR, guid: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAX_SEND_CALLBACK = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, callhandle: u32, reserved1: u32, reserved2: u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFAX_SERVICE_CALLBACK = ::core::option::Option<unsafe extern "system" fn(faxhandle: super::super::Foundation::HANDLE, deviceid: u32, param1: usize, param2: usize, param3: usize) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const REGSTR_VAL_BAUDRATE: &str = "BaudRate";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const REGSTR_VAL_BAUDRATE_A: &str = "BaudRate";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const REGSTR_VAL_DATA_W: &str = "DeviceData";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const REGSTR_VAL_DEVICESUBTYPE_W: &str = "DeviceSubType";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const REGSTR_VAL_DEVICETYPE_W: &str = "DeviceType";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const REGSTR_VAL_DEVICE_NAME_W: &str = "DriverDesc";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const REGSTR_VAL_DEV_NAME_W: &str = "DeviceName";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const REGSTR_VAL_DRIVER_DESC_W: &str = "DriverDesc";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const REGSTR_VAL_FRIENDLY_NAME_W: &str = "FriendlyName";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const REGSTR_VAL_GENERIC_CAPS_W: &str = "Capabilities";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const REGSTR_VAL_GUID: &str = "GUID";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const REGSTR_VAL_GUID_W: &str = "GUID";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const REGSTR_VAL_HARDWARE: &str = "HardwareConfig";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const REGSTR_VAL_HARDWARE_W: &str = "HardwareConfig";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const REGSTR_VAL_LAUNCHABLE: &str = "Launchable";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const REGSTR_VAL_LAUNCHABLE_W: &str = "Launchable";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const REGSTR_VAL_LAUNCH_APPS: &str = "LaunchApplications";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const REGSTR_VAL_LAUNCH_APPS_W: &str = "LaunchApplications";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const REGSTR_VAL_SHUTDOWNDELAY: &str = "ShutdownIfUnusedDelay";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const REGSTR_VAL_SHUTDOWNDELAY_W: &str = "ShutdownIfUnusedDelay";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const REGSTR_VAL_TYPE_W: &str = "Type";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const REGSTR_VAL_VENDOR_NAME_W: &str = "Vendor";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STIEDFL_ALLDEVICES: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STIEDFL_ATTACHEDONLY: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STIERR_ALREADY_INITIALIZED: ::windows_sys::core::HRESULT = -2147023649i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STIERR_BADDRIVER: ::windows_sys::core::HRESULT = -2147024777i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STIERR_BETA_VERSION: ::windows_sys::core::HRESULT = -2147023743i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STIERR_DEVICENOTREG: i32 = -2147221164i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STIERR_DEVICE_LOCKED: ::windows_sys::core::HRESULT = -2147024863i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STIERR_DEVICE_NOTREADY: ::windows_sys::core::HRESULT = -2147024875i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STIERR_GENERIC: i32 = -2147467259i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STIERR_HANDLEEXISTS: ::windows_sys::core::HRESULT = -2147024713i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STIERR_INVALID_DEVICE_NAME: ::windows_sys::core::HRESULT = -2147024773i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STIERR_INVALID_HW_TYPE: ::windows_sys::core::HRESULT = -2147024883i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STIERR_INVALID_PARAM: i32 = -2147024809i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STIERR_NEEDS_LOCK: ::windows_sys::core::HRESULT = -2147024738i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STIERR_NOEVENTS: ::windows_sys::core::HRESULT = -2147024637i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STIERR_NOINTERFACE: i32 = -2147467262i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STIERR_NOTINITIALIZED: i32 = -2147024891i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STIERR_NOT_INITIALIZED: ::windows_sys::core::HRESULT = -2147024875i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STIERR_OBJECTNOTFOUND: ::windows_sys::core::HRESULT = -2147024894i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STIERR_OLD_VERSION: ::windows_sys::core::HRESULT = -2147023746i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STIERR_OUTOFMEMORY: i32 = -2147024882i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STIERR_READONLY: i32 = -2147024891i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STIERR_SHARING_VIOLATION: ::windows_sys::core::HRESULT = -2147024864i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STIERR_UNSUPPORTED: i32 = -2147467263i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub struct STINOTIFY {
    pub dwSize: u32,
    pub guidNotificationCode: ::windows_sys::core::GUID,
    pub abNotificationData: [u8; 64],
}
impl ::core::marker::Copy for STINOTIFY {}
impl ::core::clone::Clone for STINOTIFY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct STISUBSCRIBE {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub dwFilter: u32,
    pub hWndNotify: super::super::Foundation::HWND,
    pub hEvent: super::super::Foundation::HANDLE,
    pub uiNotificationMessage: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for STISUBSCRIBE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for STISUBSCRIBE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_ADD_DEVICE_BROADCAST_ACTION: &str = "Arrival";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_ADD_DEVICE_BROADCAST_STRING: &str = "STI\\";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_CHANGENOEFFECT: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_DEVICE_CREATE_BOTH: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_DEVICE_CREATE_DATA: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_DEVICE_CREATE_FOR_MONITOR: u32 = 16777216u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_DEVICE_CREATE_MASK: u32 = 65535u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_DEVICE_CREATE_STATUS: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub struct STI_DEVICE_INFORMATIONW {
    pub dwSize: u32,
    pub DeviceType: u32,
    pub szDeviceInternalName: [u16; 128],
    pub DeviceCapabilitiesA: STI_DEV_CAPS,
    pub dwHardwareConfiguration: u32,
    pub pszVendorDescription: ::windows_sys::core::PWSTR,
    pub pszDeviceDescription: ::windows_sys::core::PWSTR,
    pub pszPortName: ::windows_sys::core::PWSTR,
    pub pszPropProvider: ::windows_sys::core::PWSTR,
    pub pszLocalName: ::windows_sys::core::PWSTR,
}
impl ::core::marker::Copy for STI_DEVICE_INFORMATIONW {}
impl ::core::clone::Clone for STI_DEVICE_INFORMATIONW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type STI_DEVICE_MJ_TYPE = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const StiDeviceTypeDefault: STI_DEVICE_MJ_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const StiDeviceTypeScanner: STI_DEVICE_MJ_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const StiDeviceTypeDigitalCamera: STI_DEVICE_MJ_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const StiDeviceTypeStreamingVideo: STI_DEVICE_MJ_TYPE = 3i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub struct STI_DEVICE_STATUS {
    pub dwSize: u32,
    pub StatusMask: u32,
    pub dwOnlineState: u32,
    pub dwHardwareStatusCode: u32,
    pub dwEventHandlingState: u32,
    pub dwPollingInterval: u32,
}
impl ::core::marker::Copy for STI_DEVICE_STATUS {}
impl ::core::clone::Clone for STI_DEVICE_STATUS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_DEVICE_VALUE_DEFAULT_LAUNCHAPP: &str = "DefaultLaunchApp";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_DEVICE_VALUE_DEFAULT_LAUNCHAPP_A: &str = "DefaultLaunchApp";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_DEVICE_VALUE_DISABLE_NOTIFICATIONS: &str = "DisableNotifications";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_DEVICE_VALUE_DISABLE_NOTIFICATIONS_A: &str = "DisableNotifications";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_DEVICE_VALUE_ICM_PROFILE: &str = "ICMProfile";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_DEVICE_VALUE_ICM_PROFILE_A: &str = "ICMProfile";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_DEVICE_VALUE_ISIS_NAME: &str = "ISISDriverName";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_DEVICE_VALUE_ISIS_NAME_A: &str = "ISISDriverName";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_DEVICE_VALUE_TIMEOUT: &str = "PollTimeout";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_DEVICE_VALUE_TIMEOUT_A: &str = "PollTimeout";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_DEVICE_VALUE_TWAIN_NAME: &str = "TwainDS";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_DEVICE_VALUE_TWAIN_NAME_A: &str = "TwainDS";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_DEVSTATUS_EVENTS_STATE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_DEVSTATUS_ONLINE_STATE: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub struct STI_DEV_CAPS {
    pub dwGeneric: u32,
}
impl ::core::marker::Copy for STI_DEV_CAPS {}
impl ::core::clone::Clone for STI_DEV_CAPS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub struct STI_DIAG {
    pub dwSize: u32,
    pub dwBasicDiagCode: u32,
    pub dwVendorDiagCode: u32,
    pub dwStatusMask: u32,
    pub sErrorInfo: _ERROR_INFOW,
}
impl ::core::marker::Copy for STI_DIAG {}
impl ::core::clone::Clone for STI_DIAG {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_DIAGCODE_HWPRESENCE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_ERROR_NO_ERROR: i32 = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_EVENTHANDLING_ENABLED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_EVENTHANDLING_PENDING: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_EVENTHANDLING_POLLING: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_GENCAP_AUTO_PORTSELECT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_GENCAP_GENERATE_ARRIVALEVENT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_GENCAP_NOTIFICATIONS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_GENCAP_POLLING_NEEDED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_GENCAP_SUBSET: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_GENCAP_WIA: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_HW_CONFIG_PARALLEL: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_HW_CONFIG_SCSI: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_HW_CONFIG_SERIAL: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_HW_CONFIG_UNKNOWN: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_HW_CONFIG_USB: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_MAX_INTERNAL_NAME_LENGTH: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_NOTCONNECTED: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_OK: i32 = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_ONLINESTATE_BUSY: u32 = 256u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_ONLINESTATE_ERROR: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_ONLINESTATE_INITIALIZING: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_ONLINESTATE_IO_ACTIVE: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_ONLINESTATE_OFFLINE: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_ONLINESTATE_OPERATIONAL: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_ONLINESTATE_PAPER_JAM: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_ONLINESTATE_PAPER_PROBLEM: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_ONLINESTATE_PAUSED: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_ONLINESTATE_PENDING: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_ONLINESTATE_POWER_SAVE: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_ONLINESTATE_TRANSFERRING: u32 = 512u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_ONLINESTATE_USER_INTERVENTION: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_ONLINESTATE_WARMING_UP: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_RAW_RESERVED: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_REMOVE_DEVICE_BROADCAST_ACTION: &str = "Removal";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_REMOVE_DEVICE_BROADCAST_STRING: &str = "STI\\";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_SUBSCRIBE_FLAG_EVENT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_SUBSCRIBE_FLAG_WINDOW: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_TRACE_ERROR: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_TRACE_INFORMATION: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_TRACE_WARNING: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_UNICODE: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub struct STI_USD_CAPS {
    pub dwVersion: u32,
    pub dwGenericCaps: u32,
}
impl ::core::marker::Copy for STI_USD_CAPS {}
impl ::core::clone::Clone for STI_USD_CAPS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_USD_GENCAP_NATIVE_PUSHSUPPORT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_VERSION: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_VERSION_FLAG_MASK: u32 = 4278190080u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_VERSION_FLAG_UNICODE: u32 = 16777216u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_VERSION_MIN_ALLOWED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const STI_VERSION_REAL: u32 = 2u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub struct STI_WIA_DEVICE_INFORMATIONW {
    pub dwSize: u32,
    pub DeviceType: u32,
    pub szDeviceInternalName: [u16; 128],
    pub DeviceCapabilitiesA: STI_DEV_CAPS,
    pub dwHardwareConfiguration: u32,
    pub pszVendorDescription: ::windows_sys::core::PWSTR,
    pub pszDeviceDescription: ::windows_sys::core::PWSTR,
    pub pszPortName: ::windows_sys::core::PWSTR,
    pub pszPropProvider: ::windows_sys::core::PWSTR,
    pub pszLocalName: ::windows_sys::core::PWSTR,
    pub pszUiDll: ::windows_sys::core::PWSTR,
    pub pszServer: ::windows_sys::core::PWSTR,
}
impl ::core::marker::Copy for STI_WIA_DEVICE_INFORMATIONW {}
impl ::core::clone::Clone for STI_WIA_DEVICE_INFORMATIONW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const SUPPORTS_MSCPLUS_STR: &str = "SupportsMSCPlus";
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const SUPPORTS_MSCPLUS_VAL: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub type SendToMode = i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const SEND_TO_FAX_RECIPIENT_ATTACHMENT: SendToMode = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const WIA_INCOMPAT_XP: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub struct _ERROR_INFOW {
    pub dwSize: u32,
    pub dwGenericError: u32,
    pub dwVendorError: u32,
    pub szExtendedErrorText: [u16; 255],
}
impl ::core::marker::Copy for _ERROR_INFOW {}
impl ::core::clone::Clone for _ERROR_INFOW {
    fn clone(&self) -> Self {
        *self
    }
}
pub type _IFaxAccountNotify = *mut ::core::ffi::c_void;
pub type _IFaxServerNotify2 = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const lDEFAULT_PREFETCH_SIZE: i32 = 100i32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const prv_DEFAULT_PREFETCH_SIZE: u32 = 100u32;
#[doc = "*Required features: `\"Win32_Devices_Fax\"`*"]
pub const wcharREASSIGN_RECIPIENTS_DELIMITER: u16 = 59u16;

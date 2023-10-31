#[cfg_attr(windows, link(name = "windows"))]
extern "system" {
    #[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
    pub fn SwDeviceClose(hswdevice: HSWDEVICE);
    #[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`, `\"Win32_Devices_Properties\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Devices_Properties", feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn SwDeviceCreate(pszenumeratorname: ::windows_sys::core::PCWSTR, pszparentdeviceinstance: ::windows_sys::core::PCWSTR, pcreateinfo: *const SW_DEVICE_CREATE_INFO, cpropertycount: u32, pproperties: *const super::super::Properties::DEVPROPERTY, pcallback: SW_DEVICE_CREATE_CALLBACK, pcontext: *const ::core::ffi::c_void, phswdevice: *mut isize) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
    pub fn SwDeviceGetLifetime(hswdevice: HSWDEVICE, plifetime: *mut SW_DEVICE_LIFETIME) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`, `\"Win32_Devices_Properties\"`*"]
    #[cfg(feature = "Win32_Devices_Properties")]
    pub fn SwDeviceInterfacePropertySet(hswdevice: HSWDEVICE, pszdeviceinterfaceid: ::windows_sys::core::PCWSTR, cpropertycount: u32, pproperties: *const super::super::Properties::DEVPROPERTY) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`, `\"Win32_Devices_Properties\"`, `\"Win32_Foundation\"`*"]
    #[cfg(all(feature = "Win32_Devices_Properties", feature = "Win32_Foundation"))]
    pub fn SwDeviceInterfaceRegister(hswdevice: HSWDEVICE, pinterfaceclassguid: *const ::windows_sys::core::GUID, pszreferencestring: ::windows_sys::core::PCWSTR, cpropertycount: u32, pproperties: *const super::super::Properties::DEVPROPERTY, fenabled: super::super::super::Foundation::BOOL, ppszdeviceinterfaceid: *mut ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SwDeviceInterfaceSetState(hswdevice: HSWDEVICE, pszdeviceinterfaceid: ::windows_sys::core::PCWSTR, fenabled: super::super::super::Foundation::BOOL) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`, `\"Win32_Devices_Properties\"`*"]
    #[cfg(feature = "Win32_Devices_Properties")]
    pub fn SwDevicePropertySet(hswdevice: HSWDEVICE, cpropertycount: u32, pproperties: *const super::super::Properties::DEVPROPERTY) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
    pub fn SwDeviceSetLifetime(hswdevice: HSWDEVICE, lifetime: SW_DEVICE_LIFETIME) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
    pub fn SwMemFree(pmem: *const ::core::ffi::c_void);
}
pub type IUPnPAddressFamilyControl = *mut ::core::ffi::c_void;
pub type IUPnPAsyncResult = *mut ::core::ffi::c_void;
pub type IUPnPDescriptionDocument = *mut ::core::ffi::c_void;
pub type IUPnPDescriptionDocumentCallback = *mut ::core::ffi::c_void;
pub type IUPnPDevice = *mut ::core::ffi::c_void;
pub type IUPnPDeviceControl = *mut ::core::ffi::c_void;
pub type IUPnPDeviceControlHttpHeaders = *mut ::core::ffi::c_void;
pub type IUPnPDeviceDocumentAccess = *mut ::core::ffi::c_void;
pub type IUPnPDeviceDocumentAccessEx = *mut ::core::ffi::c_void;
pub type IUPnPDeviceFinder = *mut ::core::ffi::c_void;
pub type IUPnPDeviceFinderAddCallbackWithInterface = *mut ::core::ffi::c_void;
pub type IUPnPDeviceFinderCallback = *mut ::core::ffi::c_void;
pub type IUPnPDeviceProvider = *mut ::core::ffi::c_void;
pub type IUPnPDevices = *mut ::core::ffi::c_void;
pub type IUPnPEventSink = *mut ::core::ffi::c_void;
pub type IUPnPEventSource = *mut ::core::ffi::c_void;
pub type IUPnPHttpHeaderControl = *mut ::core::ffi::c_void;
pub type IUPnPRegistrar = *mut ::core::ffi::c_void;
pub type IUPnPRemoteEndpointInfo = *mut ::core::ffi::c_void;
pub type IUPnPReregistrar = *mut ::core::ffi::c_void;
pub type IUPnPService = *mut ::core::ffi::c_void;
pub type IUPnPServiceAsync = *mut ::core::ffi::c_void;
pub type IUPnPServiceCallback = *mut ::core::ffi::c_void;
pub type IUPnPServiceDocumentAccess = *mut ::core::ffi::c_void;
pub type IUPnPServiceEnumProperty = *mut ::core::ffi::c_void;
pub type IUPnPServices = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const ADDRESS_FAMILY_VALUE_NAME: &str = "AddressFamily";
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const FAULT_ACTION_SPECIFIC_BASE: u32 = 600u32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const FAULT_ACTION_SPECIFIC_MAX: u32 = 899u32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const FAULT_DEVICE_INTERNAL_ERROR: u32 = 501u32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const FAULT_INVALID_ACTION: u32 = 401u32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const FAULT_INVALID_ARG: u32 = 402u32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const FAULT_INVALID_SEQUENCE_NUMBER: u32 = 403u32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const FAULT_INVALID_VARIABLE: u32 = 404u32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const REMOTE_ADDRESS_VALUE_NAME: &str = "RemoteAddress";
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_ADDRESSFAMILY_BOTH: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_ADDRESSFAMILY_IPv4: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_ADDRESSFAMILY_IPv6: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_ACTION_REQUEST_FAILED: ::windows_sys::core::HRESULT = -2147220976i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_ACTION_SPECIFIC_BASE: ::windows_sys::core::HRESULT = -2147220736i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_DEVICE_ELEMENT_EXPECTED: ::windows_sys::core::HRESULT = -2147220991i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_DEVICE_ERROR: ::windows_sys::core::HRESULT = -2147220972i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_DEVICE_NODE_INCOMPLETE: ::windows_sys::core::HRESULT = -2147220988i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_DEVICE_NOTREGISTERED: ::windows_sys::core::HRESULT = -2147180494i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_DEVICE_RUNNING: ::windows_sys::core::HRESULT = -2147180495i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_DEVICE_TIMEOUT: ::windows_sys::core::HRESULT = -2147220969i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_DUPLICATE_NOT_ALLOWED: ::windows_sys::core::HRESULT = -2147180511i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_DUPLICATE_SERVICE_ID: ::windows_sys::core::HRESULT = -2147180510i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_ERROR_PROCESSING_RESPONSE: ::windows_sys::core::HRESULT = -2147220970i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_EVENT_SUBSCRIPTION_FAILED: ::windows_sys::core::HRESULT = -2147220223i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_ICON_ELEMENT_EXPECTED: ::windows_sys::core::HRESULT = -2147220987i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_ICON_NODE_INCOMPLETE: ::windows_sys::core::HRESULT = -2147220986i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_INVALID_ACTION: ::windows_sys::core::HRESULT = -2147220985i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_INVALID_ARGUMENTS: ::windows_sys::core::HRESULT = -2147220984i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_INVALID_DESCRIPTION: ::windows_sys::core::HRESULT = -2147180509i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_INVALID_DOCUMENT: ::windows_sys::core::HRESULT = -2147220224i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_INVALID_ICON: ::windows_sys::core::HRESULT = -2147180507i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_INVALID_ROOT_NAMESPACE: ::windows_sys::core::HRESULT = -2147180505i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_INVALID_SERVICE: ::windows_sys::core::HRESULT = -2147180508i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_INVALID_VARIABLE: ::windows_sys::core::HRESULT = -2147220973i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_INVALID_XML: ::windows_sys::core::HRESULT = -2147180506i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_OUT_OF_SYNC: ::windows_sys::core::HRESULT = -2147220983i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_PROTOCOL_ERROR: ::windows_sys::core::HRESULT = -2147220971i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_REQUIRED_ELEMENT_ERROR: ::windows_sys::core::HRESULT = -2147180512i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_ROOT_ELEMENT_EXPECTED: ::windows_sys::core::HRESULT = -2147220992i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_SERVICE_ELEMENT_EXPECTED: ::windows_sys::core::HRESULT = -2147220990i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_SERVICE_NODE_INCOMPLETE: ::windows_sys::core::HRESULT = -2147220989i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_SUFFIX_TOO_LONG: ::windows_sys::core::HRESULT = -2147180504i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_TRANSPORT_ERROR: ::windows_sys::core::HRESULT = -2147220975i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_URLBASE_PRESENT: ::windows_sys::core::HRESULT = -2147180503i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_VALUE_TOO_LONG: ::windows_sys::core::HRESULT = -2147180496i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_E_VARIABLE_VALUE_UNKNOWN: ::windows_sys::core::HRESULT = -2147220974i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const UPNP_SERVICE_DELAY_SCPD_AND_SUBSCRIPTION: u32 = 1u32;
pub const UPnPDescriptionDocument: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 495622983, data2: 14888, data3: 19682, data4: [138, 75, 189, 52, 228, 91, 206, 235] };
pub const UPnPDescriptionDocumentEx: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 872220003, data2: 55322, data3: 17299, data4: [131, 204, 1, 149, 177, 218, 47, 145] };
pub const UPnPDevice: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2737132229, data2: 47713, data3: 17786, data4: [181, 154, 162, 86, 30, 18, 94, 51] };
pub const UPnPDeviceFinder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3792199464, data2: 65207, data3: 16458, data4: [184, 231, 230, 89, 189, 234, 170, 2] };
pub const UPnPDeviceFinderEx: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 404444412, data2: 14347, data3: 19061, data4: [179, 241, 74, 196, 94, 150, 5, 176] };
pub const UPnPDevices: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3119009789, data2: 44348, data3: 16548, data4: [184, 53, 8, 130, 235, 203, 170, 168] };
pub const UPnPRegistrar: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 541593785, data2: 29618, data3: 4564, data4: [191, 66, 0, 176, 208, 17, 139, 86] };
pub const UPnPRemoteEndpointInfo: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 777946345, data2: 16457, data3: 16964, data4: [183, 40, 45, 36, 34, 113, 87, 199] };
pub const UPnPService: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3324295829, data2: 64459, data3: 17417, data4: [140, 3, 140, 206, 236, 83, 62, 241] };
pub const UPnPServices: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3233565514, data2: 41990, data3: 20220, data4: [147, 47, 184, 84, 107, 129, 0, 204] };
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub type SW_DEVICE_CAPABILITIES = i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const SWDeviceCapabilitiesNone: SW_DEVICE_CAPABILITIES = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const SWDeviceCapabilitiesRemovable: SW_DEVICE_CAPABILITIES = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const SWDeviceCapabilitiesSilentInstall: SW_DEVICE_CAPABILITIES = 2i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const SWDeviceCapabilitiesNoDisplayInUI: SW_DEVICE_CAPABILITIES = 4i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const SWDeviceCapabilitiesDriverRequired: SW_DEVICE_CAPABILITIES = 8i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub type SW_DEVICE_LIFETIME = i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const SWDeviceLifetimeHandle: SW_DEVICE_LIFETIME = 0i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const SWDeviceLifetimeParentPresent: SW_DEVICE_LIFETIME = 1i32;
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub const SWDeviceLifetimeMax: SW_DEVICE_LIFETIME = 2i32;
pub type HSWDEVICE = isize;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
pub struct SW_DEVICE_CREATE_INFO {
    pub cbSize: u32,
    pub pszInstanceId: ::windows_sys::core::PCWSTR,
    pub pszzHardwareIds: ::windows_sys::core::PCWSTR,
    pub pszzCompatibleIds: ::windows_sys::core::PCWSTR,
    pub pContainerId: *const ::windows_sys::core::GUID,
    pub CapabilityFlags: u32,
    pub pszDeviceDescription: ::windows_sys::core::PCWSTR,
    pub pszDeviceLocation: ::windows_sys::core::PCWSTR,
    pub pSecurityDescriptor: *const super::super::super::Security::SECURITY_DESCRIPTOR,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::marker::Copy for SW_DEVICE_CREATE_INFO {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
impl ::core::clone::Clone for SW_DEVICE_CREATE_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Devices_Enumeration_Pnp\"`*"]
pub type SW_DEVICE_CREATE_CALLBACK = ::core::option::Option<unsafe extern "system" fn(hswdevice: HSWDEVICE, createresult: ::windows_sys::core::HRESULT, pcontext: *const ::core::ffi::c_void, pszdeviceinstanceid: ::windows_sys::core::PCWSTR)>;

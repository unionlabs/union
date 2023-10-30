#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, clashing_extern_declarations, clippy::all)]
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const ALLOW_OUTOFBAND_NOTIFICATION: u32 = 2u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const DO_NOT_VIRTUALIZE_STORAGES_AS_DEVICES: u32 = 1u32;
pub const EVENT_WMDM_CONTENT_TRANSFER: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 865901556, data2: 48382, data3: 20184, data4: [148, 223, 234, 248, 194, 106, 182, 27] };
pub type IComponentAuthenticate = *mut ::core::ffi::c_void;
pub type IMDSPDevice = *mut ::core::ffi::c_void;
pub type IMDSPDevice2 = *mut ::core::ffi::c_void;
pub type IMDSPDevice3 = *mut ::core::ffi::c_void;
pub type IMDSPDeviceControl = *mut ::core::ffi::c_void;
pub type IMDSPDirectTransfer = *mut ::core::ffi::c_void;
pub type IMDSPEnumDevice = *mut ::core::ffi::c_void;
pub type IMDSPEnumStorage = *mut ::core::ffi::c_void;
pub type IMDSPObject = *mut ::core::ffi::c_void;
pub type IMDSPObject2 = *mut ::core::ffi::c_void;
pub type IMDSPObjectInfo = *mut ::core::ffi::c_void;
pub type IMDSPRevoked = *mut ::core::ffi::c_void;
pub type IMDSPStorage = *mut ::core::ffi::c_void;
pub type IMDSPStorage2 = *mut ::core::ffi::c_void;
pub type IMDSPStorage3 = *mut ::core::ffi::c_void;
pub type IMDSPStorage4 = *mut ::core::ffi::c_void;
pub type IMDSPStorageGlobals = *mut ::core::ffi::c_void;
pub type IMDServiceProvider = *mut ::core::ffi::c_void;
pub type IMDServiceProvider2 = *mut ::core::ffi::c_void;
pub type IMDServiceProvider3 = *mut ::core::ffi::c_void;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const IOCTL_MTP_CUSTOM_COMMAND: u32 = 827348045u32;
pub type ISCPSecureAuthenticate = *mut ::core::ffi::c_void;
pub type ISCPSecureAuthenticate2 = *mut ::core::ffi::c_void;
pub type ISCPSecureExchange = *mut ::core::ffi::c_void;
pub type ISCPSecureExchange2 = *mut ::core::ffi::c_void;
pub type ISCPSecureExchange3 = *mut ::core::ffi::c_void;
pub type ISCPSecureQuery = *mut ::core::ffi::c_void;
pub type ISCPSecureQuery2 = *mut ::core::ffi::c_void;
pub type ISCPSecureQuery3 = *mut ::core::ffi::c_void;
pub type ISCPSession = *mut ::core::ffi::c_void;
pub type IWMDMDevice = *mut ::core::ffi::c_void;
pub type IWMDMDevice2 = *mut ::core::ffi::c_void;
pub type IWMDMDevice3 = *mut ::core::ffi::c_void;
pub type IWMDMDeviceControl = *mut ::core::ffi::c_void;
pub type IWMDMDeviceSession = *mut ::core::ffi::c_void;
pub type IWMDMEnumDevice = *mut ::core::ffi::c_void;
pub type IWMDMEnumStorage = *mut ::core::ffi::c_void;
pub type IWMDMLogger = *mut ::core::ffi::c_void;
pub type IWMDMMetaData = *mut ::core::ffi::c_void;
pub type IWMDMNotification = *mut ::core::ffi::c_void;
pub type IWMDMObjectInfo = *mut ::core::ffi::c_void;
pub type IWMDMOperation = *mut ::core::ffi::c_void;
pub type IWMDMOperation2 = *mut ::core::ffi::c_void;
pub type IWMDMOperation3 = *mut ::core::ffi::c_void;
pub type IWMDMProgress = *mut ::core::ffi::c_void;
pub type IWMDMProgress2 = *mut ::core::ffi::c_void;
pub type IWMDMProgress3 = *mut ::core::ffi::c_void;
pub type IWMDMRevoked = *mut ::core::ffi::c_void;
pub type IWMDMStorage = *mut ::core::ffi::c_void;
pub type IWMDMStorage2 = *mut ::core::ffi::c_void;
pub type IWMDMStorage3 = *mut ::core::ffi::c_void;
pub type IWMDMStorage4 = *mut ::core::ffi::c_void;
pub type IWMDMStorageControl = *mut ::core::ffi::c_void;
pub type IWMDMStorageControl2 = *mut ::core::ffi::c_void;
pub type IWMDMStorageControl3 = *mut ::core::ffi::c_void;
pub type IWMDMStorageGlobals = *mut ::core::ffi::c_void;
pub type IWMDeviceManager = *mut ::core::ffi::c_void;
pub type IWMDeviceManager2 = *mut ::core::ffi::c_void;
pub type IWMDeviceManager3 = *mut ::core::ffi::c_void;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const MDSP_READ: u32 = 1u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const MDSP_SEEK_BOF: u32 = 1u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const MDSP_SEEK_CUR: u32 = 2u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const MDSP_SEEK_EOF: u32 = 4u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const MDSP_WRITE: u32 = 2u32;
#[repr(C, packed(1))]
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub struct MTP_COMMAND_DATA_IN {
    pub OpCode: u16,
    pub NumParams: u32,
    pub Params: [u32; 5],
    pub NextPhase: u32,
    pub CommandWriteDataSize: u32,
    pub CommandWriteData: [u8; 1],
}
impl ::core::marker::Copy for MTP_COMMAND_DATA_IN {}
impl ::core::clone::Clone for MTP_COMMAND_DATA_IN {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub struct MTP_COMMAND_DATA_OUT {
    pub ResponseCode: u16,
    pub NumParams: u32,
    pub Params: [u32; 5],
    pub CommandReadDataSize: u32,
    pub CommandReadData: [u8; 1],
}
impl ::core::marker::Copy for MTP_COMMAND_DATA_OUT {}
impl ::core::clone::Clone for MTP_COMMAND_DATA_OUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const MTP_COMMAND_MAX_PARAMS: u32 = 5u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const MTP_NEXTPHASE_NO_DATA: u32 = 3u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const MTP_NEXTPHASE_READ_DATA: u32 = 1u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const MTP_NEXTPHASE_WRITE_DATA: u32 = 2u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const MTP_RESPONSE_MAX_PARAMS: u32 = 5u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const MTP_RESPONSE_OK: u16 = 8193u16;
pub const MediaDevMgr: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 632991105, data2: 13664, data3: 4563, data4: [132, 113, 0, 192, 79, 121, 219, 192] };
pub const MediaDevMgrClassFactory: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1342442525, data2: 48575, data3: 18724, data4: [184, 115, 241, 77, 108, 91, 253, 102] };
#[repr(C)]
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub struct OPAQUECOMMAND {
    pub guidCommand: ::windows_sys::core::GUID,
    pub dwDataLen: u32,
    pub pData: *mut u8,
    pub abMAC: [u8; 20],
}
impl ::core::marker::Copy for OPAQUECOMMAND {}
impl ::core::clone::Clone for OPAQUECOMMAND {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const RSA_KEY_LEN: u32 = 64u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const SAC_CERT_V1: u32 = 2u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const SAC_CERT_X509: u32 = 1u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const SAC_MAC_LEN: u32 = 8u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const SAC_PROTOCOL_V1: u32 = 2u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const SAC_PROTOCOL_WMDM: u32 = 1u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const SAC_SESSION_KEYLEN: u32 = 8u32;
pub const SCP_EVENTID_ACQSECURECLOCK: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2250542281, data2: 19033, data3: 17378, data4: [145, 70, 72, 167, 243, 244, 20, 12] };
pub const SCP_EVENTID_DRMINFO: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 557699719, data2: 16850, data3: 17195, data4: [158, 63, 59, 79, 123, 53, 129, 221] };
pub const SCP_EVENTID_NEEDTOINDIV: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2275739591, data2: 46185, data3: 17286, data4: [185, 118, 213, 209, 206, 83, 138, 111] };
pub const SCP_PARAMID_DRMVERSION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1104155997, data2: 31943, data3: 16919, data4: [173, 169, 0, 80, 116, 98, 77, 164] };
#[repr(C)]
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub struct WMDMDATETIME {
    pub wYear: u16,
    pub wMonth: u16,
    pub wDay: u16,
    pub wHour: u16,
    pub wMinute: u16,
    pub wSecond: u16,
}
impl ::core::marker::Copy for WMDMDATETIME {}
impl ::core::clone::Clone for WMDMDATETIME {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub union WMDMDetermineMaxPropStringLen {
    pub sz001: [u16; 27],
    pub sz002: [u16; 31],
    pub sz003: [u16; 14],
    pub sz004: [u16; 16],
    pub sz005: [u16; 22],
    pub sz006: [u16; 14],
    pub sz007: [u16; 20],
    pub sz008: [u16; 20],
    pub sz009: [u16; 22],
    pub sz010: [u16; 11],
    pub sz011: [u16; 12],
    pub sz012: [u16; 17],
    pub sz013: [u16; 17],
    pub sz014: [u16; 16],
    pub sz015: [u16; 17],
    pub sz016: [u16; 11],
    pub sz017: [u16; 11],
    pub sz018: [u16; 15],
    pub sz019: [u16; 22],
    pub sz020: [u16; 20],
    pub sz021: [u16; 22],
    pub sz022: [u16; 21],
    pub sz023: [u16; 24],
    pub sz024: [u16; 20],
    pub sz025: [u16; 10],
    pub sz026: [u16; 14],
    pub sz027: [u16; 11],
    pub sz028: [u16; 11],
    pub sz029: [u16; 13],
    pub sz030: [u16; 17],
    pub sz031: [u16; 16],
    pub sz032: [u16; 17],
    pub sz033: [u16; 20],
    pub sz034: [u16; 19],
    pub sz035: [u16; 18],
    pub sz036: [u16; 18],
    pub sz037: [u16; 15],
    pub sz041: [u16; 14],
    pub sz043: [u16; 22],
    pub sz044: [u16; 16],
    pub sz045: [u16; 20],
    pub sz046: [u16; 14],
    pub sz047: [u16; 14],
    pub sz048: [u16; 12],
    pub sz049: [u16; 25],
    pub sz050: [u16; 26],
    pub sz051: [u16; 25],
    pub sz052: [u16; 16],
    pub sz053: [u16; 24],
    pub sz054: [u16; 15],
    pub sz055: [u16; 21],
    pub sz056: [u16; 16],
    pub sz057: [u16; 22],
    pub sz058: [u16; 14],
    pub sz059: [u16; 25],
    pub sz060: [u16; 18],
    pub sz061: [u16; 22],
    pub sz062: [u16; 26],
    pub sz063: [u16; 36],
    pub sz064: [u16; 23],
    pub sz065: [u16; 12],
    pub sz066: [u16; 24],
    pub sz067: [u16; 11],
    pub sz068: [u16; 12],
    pub sz069: [u16; 14],
    pub sz070: [u16; 20],
    pub sz071: [u16; 15],
    pub sz072: [u16; 14],
    pub sz073: [u16; 31],
    pub sz074: [u16; 24],
    pub sz075: [u16; 22],
    pub sz076: [u16; 24],
    pub sz077: [u16; 21],
    pub sz078: [u16; 27],
    pub sz079: [u16; 27],
    pub sz080: [u16; 20],
    pub sz081: [u16; 33],
    pub sz082: [u16; 21],
    pub sz083: [u16; 32],
    pub sz084: [u16; 26],
    pub sz085: [u16; 18],
    pub sz086: [u16; 30],
}
impl ::core::marker::Copy for WMDMDetermineMaxPropStringLen {}
impl ::core::clone::Clone for WMDMDetermineMaxPropStringLen {
    fn clone(&self) -> Self {
        *self
    }
}
pub const WMDMDevice: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2155560159, data2: 13690, data3: 4563, data4: [132, 113, 0, 192, 79, 121, 219, 192] };
pub const WMDMDeviceEnum: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1125004719, data2: 14705, data3: 4563, data4: [132, 116, 0, 192, 79, 121, 219, 192] };
#[repr(C)]
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub struct WMDMID {
    pub cbSize: u32,
    pub dwVendorID: u32,
    pub pID: [u8; 128],
    pub SerialNumberLength: u32,
}
impl ::core::marker::Copy for WMDMID {}
impl ::core::clone::Clone for WMDMID {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDMID_LENGTH: u32 = 128u32;
pub const WMDMLogger: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 285880834, data2: 23161, data3: 4563, data4: [141, 120, 68, 69, 83, 84, 0, 0] };
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub type WMDMMessage = i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_MSG_DEVICE_ARRIVAL: WMDMMessage = 0i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_MSG_DEVICE_REMOVAL: WMDMMessage = 1i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_MSG_MEDIA_ARRIVAL: WMDMMessage = 2i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_MSG_MEDIA_REMOVAL: WMDMMessage = 3i32;
#[repr(C)]
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub struct WMDMMetadataView {
    pub pwszViewName: ::windows_sys::core::PWSTR,
    pub nDepth: u32,
    pub ppwszTags: *mut *mut u16,
}
impl ::core::marker::Copy for WMDMMetadataView {}
impl ::core::clone::Clone for WMDMMetadataView {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub struct WMDMRIGHTS {
    pub cbSize: u32,
    pub dwContentType: u32,
    pub fuFlags: u32,
    pub fuRights: u32,
    pub dwAppSec: u32,
    pub dwPlaybackCount: u32,
    pub ExpirationDate: WMDMDATETIME,
}
impl ::core::marker::Copy for WMDMRIGHTS {}
impl ::core::clone::Clone for WMDMRIGHTS {
    fn clone(&self) -> Self {
        *self
    }
}
pub const WMDMStorage: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2155560160, data2: 13690, data3: 4563, data4: [132, 113, 0, 192, 79, 121, 219, 192] };
pub const WMDMStorageEnum: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3946846779, data2: 15095, data3: 4563, data4: [132, 116, 0, 192, 79, 121, 219, 192] };
pub const WMDMStorageGlobal: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2155560161, data2: 13690, data3: 4563, data4: [132, 113, 0, 192, 79, 121, 219, 192] };
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_APP_REVOKED: u32 = 2u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_CONTENT_FILE: u32 = 4u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_CONTENT_FOLDER: u32 = 8u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_CONTENT_OPERATIONINTERFACE: u32 = 16u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_DEVICECAP_CANPAUSE: u32 = 16u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_DEVICECAP_CANPLAY: u32 = 1u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_DEVICECAP_CANRECORD: u32 = 4u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_DEVICECAP_CANRESUME: u32 = 32u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_DEVICECAP_CANSEEK: u32 = 128u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_DEVICECAP_CANSTOP: u32 = 64u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_DEVICECAP_CANSTREAMPLAY: u32 = 2u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_DEVICECAP_CANSTREAMRECORD: u32 = 8u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_DEVICECAP_HASSECURECLOCK: u32 = 256u32;
pub const WMDM_DEVICE_PROTOCOL_MSC: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2765275756, data2: 43137, data3: 17595, data4: [189, 93, 31, 112, 60, 113, 247, 169] };
pub const WMDM_DEVICE_PROTOCOL_MTP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2543736037, data2: 2812, data3: 17924, data4: [141, 147, 220, 121, 138, 75, 207, 69] };
pub const WMDM_DEVICE_PROTOCOL_RAPI: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 705818001, data2: 35983, data3: 16868, data4: [130, 209, 131, 134, 224, 3, 86, 28] };
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_DEVICE_TYPE_DECODE: u32 = 4u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_DEVICE_TYPE_ENCODE: u32 = 8u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_DEVICE_TYPE_FILELISTRESYNC: u32 = 512u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_DEVICE_TYPE_NONREENTRANT: u32 = 256u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_DEVICE_TYPE_NONSDMI: u32 = 128u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_DEVICE_TYPE_PLAYBACK: u32 = 1u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_DEVICE_TYPE_RECORD: u32 = 2u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_DEVICE_TYPE_SDMI: u32 = 64u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_DEVICE_TYPE_STORAGE: u32 = 16u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_DEVICE_TYPE_VIEW_PREF_METADATAVIEW: u32 = 1024u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_DEVICE_TYPE_VIRTUAL: u32 = 32u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub type WMDM_ENUM_PROP_VALID_VALUES_FORM = i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_ENUM_PROP_VALID_VALUES_ANY: WMDM_ENUM_PROP_VALID_VALUES_FORM = 0i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_ENUM_PROP_VALID_VALUES_RANGE: WMDM_ENUM_PROP_VALID_VALUES_FORM = 1i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_ENUM_PROP_VALID_VALUES_ENUM: WMDM_ENUM_PROP_VALID_VALUES_FORM = 2i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_E_BUFFERTOOSMALL: i32 = -2147201016i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_E_BUSY: i32 = -2147201024i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_E_CALL_OUT_OF_SEQUENCE: i32 = -2147201017i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_E_CANTOPEN_PMSN_SERVICE_PIPE: i32 = -2147201005i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_E_INCORRECT_APPSEC: i32 = -2147201008i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_E_INCORRECT_RIGHTS: i32 = -2147201007i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_E_INTERFACEDEAD: i32 = -2147201023i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_E_INVALIDTYPE: i32 = -2147201022i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_E_LICENSE_EXPIRED: i32 = -2147201006i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_E_LICENSE_NOTEXIST: i32 = -2147201009i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_E_MAC_CHECK_FAILED: i32 = -2147201014i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_E_MOREDATA: i32 = -2147201015i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_E_NORIGHTS: i32 = -2147201018i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_E_NOTCERTIFIED: i32 = -2147201019i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_E_NOTSUPPORTED: i32 = -2147201020i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_E_PROCESSFAILED: i32 = -2147201021i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_E_REVOKED: i32 = -2147201010i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_E_SDMI_NOMORECOPIES: i32 = -2147201011i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_E_SDMI_TRIGGER: i32 = -2147201012i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_E_TOO_MANY_SESSIONS: i32 = -2147201005i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_E_USER_CANCELLED: i32 = -2147201013i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FILE_ATTR_AUDIO: u32 = 4096u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FILE_ATTR_AUDIOBOOK: u32 = 2097152u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FILE_ATTR_CANDELETE: u32 = 32768u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FILE_ATTR_CANMOVE: u32 = 65536u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FILE_ATTR_CANPLAY: u32 = 16384u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FILE_ATTR_CANREAD: u32 = 262144u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FILE_ATTR_CANRENAME: u32 = 131072u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FILE_ATTR_DATA: u32 = 8192u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FILE_ATTR_FILE: u32 = 32u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FILE_ATTR_FOLDER: u32 = 8u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FILE_ATTR_HIDDEN: u32 = 4194304u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FILE_ATTR_LINK: u32 = 16u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FILE_ATTR_MUSIC: u32 = 524288u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FILE_ATTR_READONLY: u32 = 16777216u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FILE_ATTR_SYSTEM: u32 = 8388608u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FILE_ATTR_VIDEO: u32 = 64u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FILE_CREATE_OVERWRITE: u32 = 1048576u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub type WMDM_FIND_SCOPE = i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FIND_SCOPE_GLOBAL: WMDM_FIND_SCOPE = 0i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FIND_SCOPE_IMMEDIATE_CHILDREN: WMDM_FIND_SCOPE = 1i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub type WMDM_FORMATCODE = i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_NOTUSED: WMDM_FORMATCODE = 0i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_ALLIMAGES: WMDM_FORMATCODE = -1i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_UNDEFINED: WMDM_FORMATCODE = 12288i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_ASSOCIATION: WMDM_FORMATCODE = 12289i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_SCRIPT: WMDM_FORMATCODE = 12290i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_EXECUTABLE: WMDM_FORMATCODE = 12291i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_TEXT: WMDM_FORMATCODE = 12292i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_HTML: WMDM_FORMATCODE = 12293i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_DPOF: WMDM_FORMATCODE = 12294i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_AIFF: WMDM_FORMATCODE = 12295i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_WAVE: WMDM_FORMATCODE = 12296i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_MP3: WMDM_FORMATCODE = 12297i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_AVI: WMDM_FORMATCODE = 12298i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_MPEG: WMDM_FORMATCODE = 12299i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_ASF: WMDM_FORMATCODE = 12300i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_RESERVED_FIRST: WMDM_FORMATCODE = 12301i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_RESERVED_LAST: WMDM_FORMATCODE = 14335i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_IMAGE_UNDEFINED: WMDM_FORMATCODE = 14336i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_IMAGE_EXIF: WMDM_FORMATCODE = 14337i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_IMAGE_TIFFEP: WMDM_FORMATCODE = 14338i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_IMAGE_FLASHPIX: WMDM_FORMATCODE = 14339i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_IMAGE_BMP: WMDM_FORMATCODE = 14340i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_IMAGE_CIFF: WMDM_FORMATCODE = 14341i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_IMAGE_GIF: WMDM_FORMATCODE = 14343i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_IMAGE_JFIF: WMDM_FORMATCODE = 14344i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_IMAGE_PCD: WMDM_FORMATCODE = 14345i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_IMAGE_PICT: WMDM_FORMATCODE = 14346i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_IMAGE_PNG: WMDM_FORMATCODE = 14347i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_IMAGE_TIFF: WMDM_FORMATCODE = 14349i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_IMAGE_TIFFIT: WMDM_FORMATCODE = 14350i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_IMAGE_JP2: WMDM_FORMATCODE = 14351i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_IMAGE_JPX: WMDM_FORMATCODE = 14352i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_IMAGE_RESERVED_FIRST: WMDM_FORMATCODE = 14353i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_IMAGE_RESERVED_LAST: WMDM_FORMATCODE = 16383i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_UNDEFINEDFIRMWARE: WMDM_FORMATCODE = 47106i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_WBMP: WMDM_FORMATCODE = 47107i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_JPEGXR: WMDM_FORMATCODE = 47108i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_WINDOWSIMAGEFORMAT: WMDM_FORMATCODE = 47233i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_UNDEFINEDAUDIO: WMDM_FORMATCODE = 47360i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_WMA: WMDM_FORMATCODE = 47361i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_OGG: WMDM_FORMATCODE = 47362i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_AAC: WMDM_FORMATCODE = 47363i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_AUDIBLE: WMDM_FORMATCODE = 47364i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_FLAC: WMDM_FORMATCODE = 47366i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_QCELP: WMDM_FORMATCODE = 47367i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_AMR: WMDM_FORMATCODE = 47368i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_UNDEFINEDVIDEO: WMDM_FORMATCODE = 47488i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_WMV: WMDM_FORMATCODE = 47489i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_MP4: WMDM_FORMATCODE = 47490i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_MP2: WMDM_FORMATCODE = 47491i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_3GP: WMDM_FORMATCODE = 47492i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_3G2: WMDM_FORMATCODE = 47493i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_AVCHD: WMDM_FORMATCODE = 47494i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_ATSCTS: WMDM_FORMATCODE = 47495i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_DVBTS: WMDM_FORMATCODE = 47496i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_MKV: WMDM_FORMATCODE = 47497i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_MKA: WMDM_FORMATCODE = 47498i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_MK3D: WMDM_FORMATCODE = 47499i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_UNDEFINEDCOLLECTION: WMDM_FORMATCODE = 47616i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_ABSTRACTMULTIMEDIAALBUM: WMDM_FORMATCODE = 47617i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_ABSTRACTIMAGEALBUM: WMDM_FORMATCODE = 47618i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_ABSTRACTAUDIOALBUM: WMDM_FORMATCODE = 47619i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_ABSTRACTVIDEOALBUM: WMDM_FORMATCODE = 47620i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_ABSTRACTAUDIOVIDEOPLAYLIST: WMDM_FORMATCODE = 47621i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_ABSTRACTCONTACTGROUP: WMDM_FORMATCODE = 47622i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_ABSTRACTMESSAGEFOLDER: WMDM_FORMATCODE = 47623i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_ABSTRACTCHAPTEREDPRODUCTION: WMDM_FORMATCODE = 47624i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_MEDIA_CAST: WMDM_FORMATCODE = 47627i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_WPLPLAYLIST: WMDM_FORMATCODE = 47632i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_M3UPLAYLIST: WMDM_FORMATCODE = 47633i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_MPLPLAYLIST: WMDM_FORMATCODE = 47634i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_ASXPLAYLIST: WMDM_FORMATCODE = 47635i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_PLSPLAYLIST: WMDM_FORMATCODE = 47636i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_UNDEFINEDDOCUMENT: WMDM_FORMATCODE = 47744i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_ABSTRACTDOCUMENT: WMDM_FORMATCODE = 47745i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_XMLDOCUMENT: WMDM_FORMATCODE = 47746i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_MICROSOFTWORDDOCUMENT: WMDM_FORMATCODE = 47747i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_MHTCOMPILEDHTMLDOCUMENT: WMDM_FORMATCODE = 47748i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_MICROSOFTEXCELSPREADSHEET: WMDM_FORMATCODE = 47749i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_MICROSOFTPOWERPOINTDOCUMENT: WMDM_FORMATCODE = 47750i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_UNDEFINEDMESSAGE: WMDM_FORMATCODE = 47872i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_ABSTRACTMESSAGE: WMDM_FORMATCODE = 47873i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_UNDEFINEDCONTACT: WMDM_FORMATCODE = 48000i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_ABSTRACTCONTACT: WMDM_FORMATCODE = 48001i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_VCARD2: WMDM_FORMATCODE = 48002i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_VCARD3: WMDM_FORMATCODE = 48003i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_UNDEFINEDCALENDARITEM: WMDM_FORMATCODE = 48640i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_ABSTRACTCALENDARITEM: WMDM_FORMATCODE = 48641i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_VCALENDAR1: WMDM_FORMATCODE = 48642i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_VCALENDAR2: WMDM_FORMATCODE = 48643i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_UNDEFINEDWINDOWSEXECUTABLE: WMDM_FORMATCODE = 48768i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_M4A: WMDM_FORMATCODE = 1297101889i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_3GPA: WMDM_FORMATCODE = 860311617i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_3G2A: WMDM_FORMATCODE = 860303937i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_FORMATCODE_SECTION: WMDM_FORMATCODE = 48770i32;
#[repr(C)]
#[doc = "*Required features: 'Win32_Media_DeviceManager', 'Win32_Foundation', 'Win32_System_Com_StructuredStorage'*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
pub struct WMDM_FORMAT_CAPABILITY {
    pub nPropConfig: u32,
    pub pConfigs: *mut WMDM_PROP_CONFIG,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
impl ::core::marker::Copy for WMDM_FORMAT_CAPABILITY {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
impl ::core::clone::Clone for WMDM_FORMAT_CAPABILITY {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_GET_FORMAT_SUPPORT_AUDIO: u32 = 1u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_GET_FORMAT_SUPPORT_FILE: u32 = 4u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_GET_FORMAT_SUPPORT_VIDEO: u32 = 2u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_LOG_NOTIMESTAMP: u32 = 16u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_LOG_SEV_ERROR: u32 = 4u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_LOG_SEV_INFO: u32 = 1u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_LOG_SEV_WARN: u32 = 2u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_MAC_LENGTH: u32 = 8u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_MODE_BLOCK: u32 = 1u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_MODE_PROGRESS: u32 = 64u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_MODE_QUERY: u32 = 32u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_MODE_RECURSIVE: u32 = 4096u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_MODE_THREAD: u32 = 2u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_MODE_TRANSFER_PROTECTED: u32 = 128u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_MODE_TRANSFER_UNPROTECTED: u32 = 256u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_POWER_CAP_BATTERY: u32 = 1u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_POWER_CAP_EXTERNAL: u32 = 2u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_POWER_IS_BATTERY: u32 = 4u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_POWER_IS_EXTERNAL: u32 = 8u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_POWER_PERCENT_AVAILABLE: u32 = 16u32;
#[repr(C)]
#[doc = "*Required features: 'Win32_Media_DeviceManager', 'Win32_Foundation', 'Win32_System_Com_StructuredStorage'*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
pub struct WMDM_PROP_CONFIG {
    pub nPreference: u32,
    pub nPropDesc: u32,
    pub pPropDesc: *mut WMDM_PROP_DESC,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
impl ::core::marker::Copy for WMDM_PROP_CONFIG {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
impl ::core::clone::Clone for WMDM_PROP_CONFIG {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: 'Win32_Media_DeviceManager', 'Win32_Foundation', 'Win32_System_Com_StructuredStorage'*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
pub struct WMDM_PROP_DESC {
    pub pwszPropName: ::windows_sys::core::PWSTR,
    pub ValidValuesForm: WMDM_ENUM_PROP_VALID_VALUES_FORM,
    pub ValidValues: WMDM_PROP_DESC_0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
impl ::core::marker::Copy for WMDM_PROP_DESC {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
impl ::core::clone::Clone for WMDM_PROP_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: 'Win32_Media_DeviceManager', 'Win32_Foundation', 'Win32_System_Com_StructuredStorage'*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
pub union WMDM_PROP_DESC_0 {
    pub ValidValuesRange: WMDM_PROP_VALUES_RANGE,
    pub EnumeratedValidValues: WMDM_PROP_VALUES_ENUM,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
impl ::core::marker::Copy for WMDM_PROP_DESC_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
impl ::core::clone::Clone for WMDM_PROP_DESC_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: 'Win32_Media_DeviceManager', 'Win32_Foundation', 'Win32_System_Com_StructuredStorage'*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
pub struct WMDM_PROP_VALUES_ENUM {
    pub cEnumValues: u32,
    pub pValues: *mut super::super::System::Com::StructuredStorage::PROPVARIANT,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
impl ::core::marker::Copy for WMDM_PROP_VALUES_ENUM {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
impl ::core::clone::Clone for WMDM_PROP_VALUES_ENUM {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: 'Win32_Media_DeviceManager', 'Win32_Foundation', 'Win32_System_Com_StructuredStorage'*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
pub struct WMDM_PROP_VALUES_RANGE {
    pub rangeMin: super::super::System::Com::StructuredStorage::PROPVARIANT,
    pub rangeMax: super::super::System::Com::StructuredStorage::PROPVARIANT,
    pub rangeStep: super::super::System::Com::StructuredStorage::PROPVARIANT,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
impl ::core::marker::Copy for WMDM_PROP_VALUES_RANGE {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
impl ::core::clone::Clone for WMDM_PROP_VALUES_RANGE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_RIGHTS_COPY_TO_CD: u32 = 8u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_RIGHTS_COPY_TO_NON_SDMI_DEVICE: u32 = 2u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_RIGHTS_COPY_TO_SDMI_DEVICE: u32 = 16u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_RIGHTS_EXPIRATIONDATE: u32 = 2u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_RIGHTS_FREESERIALIDS: u32 = 8u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_RIGHTS_GROUPID: u32 = 4u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_RIGHTS_NAMEDSERIALIDS: u32 = 16u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_RIGHTS_PLAYBACKCOUNT: u32 = 1u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_RIGHTS_PLAY_ON_PC: u32 = 1u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_SCP_DECIDE_DATA: i32 = 8i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_SCP_DRMINFO_NOT_DRMPROTECTED: i32 = 0i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_SCP_DRMINFO_V1HEADER: i32 = 1i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_SCP_DRMINFO_V2HEADER: i32 = 2i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_SCP_EXAMINE_DATA: i32 = 2i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_SCP_EXAMINE_EXTENSION: i32 = 1i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_SCP_NO_MORE_CHANGES: i32 = 64i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_SCP_PROTECTED_OUTPUT: i32 = 16i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_SCP_REVOKED: u32 = 8u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_SCP_RIGHTS_DATA: i32 = 64i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_SCP_TRANSFER_OBJECTDATA: i32 = 32i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_SCP_UNPROTECTED_OUTPUT: i32 = 32i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_SEEK_BEGIN: u32 = 1u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_SEEK_CURRENT: u32 = 2u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_SEEK_END: u32 = 8u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_SEEK_REMOTECONTROL: u32 = 1u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_SEEK_STREAMINGAUDIO: u32 = 2u32;
pub const WMDM_SERVICE_PROVIDER_VENDOR_MICROSOFT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2112383085, data2: 30958, data3: 17386, data4: [164, 150, 198, 37, 172, 145, 204, 93] };
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub type WMDM_SESSION_TYPE = i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_SESSION_NONE: WMDM_SESSION_TYPE = 0i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_SESSION_TRANSFER_TO_DEVICE: WMDM_SESSION_TYPE = 1i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_SESSION_TRANSFER_FROM_DEVICE: WMDM_SESSION_TYPE = 16i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_SESSION_DELETE: WMDM_SESSION_TYPE = 256i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_SESSION_CUSTOM: WMDM_SESSION_TYPE = 4096i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_SP_REVOKED: u32 = 4u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STATUS_BUSY: u32 = 2u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STATUS_DEVICECONTROL_PAUSED: u32 = 32u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STATUS_DEVICECONTROL_PLAYING: u32 = 8u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STATUS_DEVICECONTROL_RECORDING: u32 = 16u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STATUS_DEVICECONTROL_REMOTE: u32 = 64u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STATUS_DEVICECONTROL_STREAM: u32 = 128u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STATUS_DEVICE_NOTPRESENT: u32 = 4u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STATUS_READY: u32 = 1u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STATUS_STORAGECONTROL_APPENDING: u32 = 32768u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STATUS_STORAGECONTROL_DELETING: u32 = 16384u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STATUS_STORAGECONTROL_INSERTING: u32 = 8192u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STATUS_STORAGECONTROL_MOVING: u32 = 65536u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STATUS_STORAGECONTROL_READING: u32 = 131072u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STATUS_STORAGE_BROKEN: u32 = 1024u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STATUS_STORAGE_INITIALIZING: u32 = 512u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STATUS_STORAGE_NOTPRESENT: u32 = 256u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STATUS_STORAGE_NOTSUPPORTED: u32 = 2048u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STATUS_STORAGE_UNFORMATTED: u32 = 4096u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STORAGECAP_FILELIMITEXISTS: u32 = 32u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STORAGECAP_FILESINFOLDERS: u32 = 8u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STORAGECAP_FILESINROOT: u32 = 2u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STORAGECAP_FOLDERLIMITEXISTS: u32 = 16u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STORAGECAP_FOLDERSINFOLDERS: u32 = 4u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STORAGECAP_FOLDERSINROOT: u32 = 1u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STORAGECAP_NOT_INITIALIZABLE: u32 = 64u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STORAGECONTROL_INSERTAFTER: u32 = 1024u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STORAGECONTROL_INSERTBEFORE: u32 = 512u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STORAGECONTROL_INSERTINTO: u32 = 2048u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STORAGE_ATTR_CANEDITMETADATA: u32 = 128u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STORAGE_ATTR_FILESYSTEM: u32 = 1u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STORAGE_ATTR_FOLDERS: u32 = 256u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STORAGE_ATTR_HAS_FILES: u32 = 67108864u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STORAGE_ATTR_HAS_FOLDERS: u32 = 33554432u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STORAGE_ATTR_NONREMOVABLE: u32 = 4u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STORAGE_ATTR_REMOVABLE: u32 = 2u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STORAGE_ATTR_VIRTUAL: u32 = 536870912u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STORAGE_CONTAINS_DEFAULT: u32 = 268435456u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub type WMDM_STORAGE_ENUM_MODE = i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const ENUM_MODE_RAW: WMDM_STORAGE_ENUM_MODE = 0i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const ENUM_MODE_USE_DEVICE_PREF: WMDM_STORAGE_ENUM_MODE = 1i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const ENUM_MODE_METADATA_VIEWS: WMDM_STORAGE_ENUM_MODE = 2i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_STORAGE_IS_DEFAULT: u32 = 134217728u32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_S_NOT_ALL_PROPERTIES_APPLIED: i32 = 282625i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_S_NOT_ALL_PROPERTIES_RETRIEVED: i32 = 282626i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub type WMDM_TAG_DATATYPE = i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_TYPE_DWORD: WMDM_TAG_DATATYPE = 0i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_TYPE_STRING: WMDM_TAG_DATATYPE = 1i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_TYPE_BINARY: WMDM_TAG_DATATYPE = 2i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_TYPE_BOOL: WMDM_TAG_DATATYPE = 3i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_TYPE_QWORD: WMDM_TAG_DATATYPE = 4i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_TYPE_WORD: WMDM_TAG_DATATYPE = 5i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_TYPE_GUID: WMDM_TAG_DATATYPE = 6i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_TYPE_DATE: WMDM_TAG_DATATYPE = 7i32;
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const WMDM_WMDM_REVOKED: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub struct WMFILECAPABILITIES {
    pub pwszMimeType: ::windows_sys::core::PWSTR,
    pub dwReserved: u32,
}
impl ::core::marker::Copy for WMFILECAPABILITIES {}
impl ::core::clone::Clone for WMFILECAPABILITIES {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub struct _BITMAPINFOHEADER {
    pub biSize: u32,
    pub biWidth: i32,
    pub biHeight: i32,
    pub biPlanes: u16,
    pub biBitCount: u16,
    pub biCompression: u32,
    pub biSizeImage: u32,
    pub biXPelsPerMeter: i32,
    pub biYPelsPerMeter: i32,
    pub biClrUsed: u32,
    pub biClrImportant: u32,
}
impl ::core::marker::Copy for _BITMAPINFOHEADER {}
impl ::core::clone::Clone for _BITMAPINFOHEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: 'Win32_Media_DeviceManager', 'Win32_Foundation'*"]
#[cfg(feature = "Win32_Foundation")]
pub struct _VIDEOINFOHEADER {
    pub rcSource: super::super::Foundation::RECT,
    pub rcTarget: super::super::Foundation::RECT,
    pub dwBitRate: u32,
    pub dwBitErrorRate: u32,
    pub AvgTimePerFrame: i64,
    pub bmiHeader: _BITMAPINFOHEADER,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for _VIDEOINFOHEADER {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for _VIDEOINFOHEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub struct _WAVEFORMATEX {
    pub wFormatTag: u16,
    pub nChannels: u16,
    pub nSamplesPerSec: u32,
    pub nAvgBytesPerSec: u32,
    pub nBlockAlign: u16,
    pub wBitsPerSample: u16,
    pub cbSize: u16,
}
impl ::core::marker::Copy for _WAVEFORMATEX {}
impl ::core::clone::Clone for _WAVEFORMATEX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: 'Win32_Media_DeviceManager', 'Win32_Foundation'*"]
#[cfg(feature = "Win32_Foundation")]
pub struct __MACINFO {
    pub fUsed: super::super::Foundation::BOOL,
    pub abMacState: [u8; 36],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for __MACINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for __MACINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszAudioWAVECodec: &'static str = "WMDM/AudioWAVECodec";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszVideoFourCCCodec: &'static str = "WMDM/VideoFourCCCodec";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMAlbumArt: &'static str = "WMDM/AlbumArt";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMAlbumArtist: &'static str = "WMDM/AlbumArtist";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMAlbumCoverData: &'static str = "WMDM/AlbumCoverData";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMAlbumCoverDuration: &'static str = "WMDM/AlbumCoverDuration";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMAlbumCoverFormat: &'static str = "WMDM/AlbumCoverFormat";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMAlbumCoverHeight: &'static str = "WMDM/AlbumCoverHeight";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMAlbumCoverSize: &'static str = "WMDM/AlbumCoverSize";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMAlbumCoverWidth: &'static str = "WMDM/AlbumCoverWidth";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMAlbumTitle: &'static str = "WMDM/AlbumTitle";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMAudioBitDepth: &'static str = "WMDM/AudioBitDepth";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMAuthor: &'static str = "WMDM/Author";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMAuthorDate: &'static str = "WMDM/AuthorDate";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMBitRateType: &'static str = "WMDM/BitRateType";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMBitrate: &'static str = "WMDM/Bitrate";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMBlockAlignment: &'static str = "WMDM/BlockAlignment";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMBufferSize: &'static str = "WMDM/BufferSize";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMBuyNow: &'static str = "WMDM/BuyNow";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMByteBookmark: &'static str = "WMDM/ByteBookmark";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMCategory: &'static str = "WMDM/Category";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMCodec: &'static str = "WMDM/Codec";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMCollectionID: &'static str = "WMDM/CollectionID";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMComposer: &'static str = "WMDM/Composer";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMDRMId: &'static str = "WMDM/DRMId";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMDataLength: &'static str = "WMDM/DataLength";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMDataOffset: &'static str = "WMDM/DataOffset";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMDataUnits: &'static str = "WMDM/DataUnits";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMDescription: &'static str = "WMDM/Description";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMDestinationURL: &'static str = "WMDM/DestinationURL";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMDeviceFirmwareVersion: &'static str = "WMDM/DeviceFirmwareVersion";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMDeviceFriendlyName: &'static str = "WMDM/DeviceFriendlyName";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMDeviceModelName: &'static str = "WMDM/DeviceModelName";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMDevicePlayCount: &'static str = "WMDM/DevicePlayCount";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMDeviceProtocol: &'static str = "WMDM/DeviceProtocol";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMDeviceRevocationInfo: &'static str = "WMDM/DeviceRevocationInfo";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMDeviceServiceProviderVendor: &'static str = "WMDM/DeviceServiceProviderVendor";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMDeviceVendorExtension: &'static str = "WMDM/DeviceVendorExtension";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMDuration: &'static str = "WMDM/Duration";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMEditor: &'static str = "WMDM/Editor";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMEncodingProfile: &'static str = "WMDM/EncodingProfile";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMFileAttributes: &'static str = "WMDM/FileAttributes";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMFileCreationDate: &'static str = "WMDM/FileCreationDate";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMFileName: &'static str = "WMDM/FileName";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMFileSize: &'static str = "WMDM/FileSize";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMFormatCode: &'static str = "WMDM/FormatCode";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMFormatsSupported: &'static str = "WMDM/FormatsSupported";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMFormatsSupportedAreOrdered: &'static str = "WMDM/FormatsSupportedAreOrdered";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMFrameRate: &'static str = "WMDM/FrameRate";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMGenre: &'static str = "WMDM/Genre";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMHeight: &'static str = "WMDM/Height";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMIsProtected: &'static str = "WMDM/IsProtected";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMIsRepeat: &'static str = "WMDM/IsRepeat";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMKeyFrameDistance: &'static str = "WMDM/KeyFrameDistance";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMLastModifiedDate: &'static str = "WMDM/LastModifiedDate";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMMediaClassSecondaryID: &'static str = "WMDM/MediaClassSecondaryID";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMMediaCredits: &'static str = "WMDM/MediaCredits";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMMediaGuid: &'static str = "WMDM/MediaGuid";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMMediaOriginalBroadcastDateTime: &'static str = "WMDM/MediaOriginalBroadcastDateTime";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMMediaOriginalChannel: &'static str = "WMDM/MediaOriginalChannel";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMMediaStationName: &'static str = "WMDM/MediaStationName";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMMetaGenre: &'static str = "WMDM/MetaGenre";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMNonConsumable: &'static str = "WMDM/NonConsumable";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMNumChannels: &'static str = "WMDM/NumChannels";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMObjectBookmark: &'static str = "WMDM/ObjectBookmark";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMOwner: &'static str = "WMDM/Owner";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMParentalRating: &'static str = "WMDM/ParentalRating";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMPersistentUniqueID: &'static str = "WMDM/PersistentUniqueID";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMPlayCount: &'static str = "WMDM/PlayCount";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMProviderCopyright: &'static str = "WMDM/ProviderCopyright";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMQualitySetting: &'static str = "WMDM/QualitySetting";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMSampleRate: &'static str = "WMDM/SampleRate";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMScanType: &'static str = "WMDM/ScanType";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMSourceURL: &'static str = "WMDM/SourceURL";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMSubTitle: &'static str = "WMDM/SubTitle";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMSubTitleDescription: &'static str = "WMDM/SubTitleDescription";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMSupportedDeviceProperties: &'static str = "WMDM/SupportedDeviceProperties";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMSyncID: &'static str = "WMDM/SyncID";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMSyncRelationshipID: &'static str = "WMDM/SyncRelationshipID";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMSyncTime: &'static str = "WMDM/SyncTime";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMTimeBookmark: &'static str = "WMDM/TimeBookmark";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMTimeToLive: &'static str = "WMDM/TimeToLive";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMTitle: &'static str = "WMDM/Title";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMTotalBitrate: &'static str = "WMDM/TotalBitrate";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMTrack: &'static str = "WMDM/Track";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMTrackMood: &'static str = "WMDM/TrackMood";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMUserEffectiveRating: &'static str = "WMDM/UserEffectiveRating";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMUserLastPlayTime: &'static str = "WMDM/UserLastPlayTime";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMUserRating: &'static str = "WMDM/UserRating";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMUserRatingOnDevice: &'static str = "WMDM/UserRatingOnDevice";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMVideoBitrate: &'static str = "WMDM/VideoBitrate";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMWebmaster: &'static str = "WMDM/Webmaster";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMWidth: &'static str = "WMDM/Width";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMYear: &'static str = "WMDM/Year";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWMDMediaClassPrimaryID: &'static str = "WMDM/MediaClassPrimaryID";
#[doc = "*Required features: 'Win32_Media_DeviceManager'*"]
pub const g_wszWPDPassthroughPropertyValues: &'static str = "WPD/PassthroughPropertyValues";

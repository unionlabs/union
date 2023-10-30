#[link(name = "windows")]
extern "system" {
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
    pub fn D3DPERF_BeginEvent(col: u32, wszname: ::windows_sys::core::PCWSTR) -> i32;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
    pub fn D3DPERF_EndEvent() -> i32;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
    pub fn D3DPERF_GetStatus() -> u32;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn D3DPERF_QueryRepeatFrame() -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
    pub fn D3DPERF_SetMarker(col: u32, wszname: ::windows_sys::core::PCWSTR);
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
    pub fn D3DPERF_SetOptions(dwoptions: u32);
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
    pub fn D3DPERF_SetRegion(col: u32, wszname: ::windows_sys::core::PCWSTR);
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
    pub fn Direct3DCreate9(sdkversion: u32) -> IDirect3D9;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
    pub fn Direct3DCreate9Ex(sdkversion: u32, param1: *mut IDirect3D9Ex) -> ::windows_sys::core::HRESULT;
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3D9_RESOURCE_PRIORITY_HIGH: u32 = 2684354560u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3D9_RESOURCE_PRIORITY_LOW: u32 = 1342177280u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3D9_RESOURCE_PRIORITY_MAXIMUM: u32 = 3355443200u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3D9_RESOURCE_PRIORITY_MINIMUM: u32 = 671088640u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3D9_RESOURCE_PRIORITY_NORMAL: u32 = 2013265920u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3D9b_SDK_VERSION: u32 = 31u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DADAPTER_DEFAULT: u32 = 0u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DADAPTER_IDENTIFIER9 {
    pub Driver: [super::super::Foundation::CHAR; 512],
    pub Description: [super::super::Foundation::CHAR; 512],
    pub DeviceName: [super::super::Foundation::CHAR; 32],
    pub DriverVersion: i64,
    pub VendorId: u32,
    pub DeviceId: u32,
    pub SubSysId: u32,
    pub Revision: u32,
    pub DeviceIdentifier: ::windows_sys::core::GUID,
    pub WHQLLevel: u32,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DADAPTER_IDENTIFIER9 {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DADAPTER_IDENTIFIER9 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(4))]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DADAPTER_IDENTIFIER9 {
    pub Driver: [super::super::Foundation::CHAR; 512],
    pub Description: [super::super::Foundation::CHAR; 512],
    pub DeviceName: [super::super::Foundation::CHAR; 32],
    pub DriverVersion: i64,
    pub VendorId: u32,
    pub DeviceId: u32,
    pub SubSysId: u32,
    pub Revision: u32,
    pub DeviceIdentifier: ::windows_sys::core::GUID,
    pub WHQLLevel: u32,
}
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DADAPTER_IDENTIFIER9 {}
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DADAPTER_IDENTIFIER9 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
pub struct D3DAES_CTR_IV {
    pub IV: u64,
    pub Count: u64,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
impl ::core::marker::Copy for D3DAES_CTR_IV {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
impl ::core::clone::Clone for D3DAES_CTR_IV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(4))]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
#[cfg(target_arch = "x86")]
pub struct D3DAES_CTR_IV {
    pub IV: u64,
    pub Count: u64,
}
#[cfg(target_arch = "x86")]
impl ::core::marker::Copy for D3DAES_CTR_IV {}
#[cfg(target_arch = "x86")]
impl ::core::clone::Clone for D3DAES_CTR_IV {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DAUTHENTICATEDCHANNELTYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DAUTHENTICATEDCHANNEL_D3D9: D3DAUTHENTICATEDCHANNELTYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DAUTHENTICATEDCHANNEL_DRIVER_SOFTWARE: D3DAUTHENTICATEDCHANNELTYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DAUTHENTICATEDCHANNEL_DRIVER_HARDWARE: D3DAUTHENTICATEDCHANNELTYPE = 3i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DAUTHENTICATEDCHANNEL_CONFIGURECRYPTOSESSION {
    pub Parameters: D3DAUTHENTICATEDCHANNEL_CONFIGURE_INPUT,
    pub DXVA2DecodeHandle: super::super::Foundation::HANDLE,
    pub CryptoSessionHandle: super::super::Foundation::HANDLE,
    pub DeviceHandle: super::super::Foundation::HANDLE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_CONFIGURECRYPTOSESSION {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_CONFIGURECRYPTOSESSION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DAUTHENTICATEDCHANNEL_CONFIGUREINITIALIZE {
    pub Parameters: D3DAUTHENTICATEDCHANNEL_CONFIGURE_INPUT,
    pub StartSequenceQuery: u32,
    pub StartSequenceConfigure: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_CONFIGUREINITIALIZE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_CONFIGUREINITIALIZE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DAUTHENTICATEDCHANNEL_CONFIGUREPROTECTION {
    pub Parameters: D3DAUTHENTICATEDCHANNEL_CONFIGURE_INPUT,
    pub Protections: D3DAUTHENTICATEDCHANNEL_PROTECTION_FLAGS,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_CONFIGUREPROTECTION {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_CONFIGUREPROTECTION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DAUTHENTICATEDCHANNEL_CONFIGURESHAREDRESOURCE {
    pub Parameters: D3DAUTHENTICATEDCHANNEL_CONFIGURE_INPUT,
    pub ProcessIdentiferType: D3DAUTHENTICATEDCHANNEL_PROCESSIDENTIFIERTYPE,
    pub ProcessHandle: super::super::Foundation::HANDLE,
    pub AllowAccess: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_CONFIGURESHAREDRESOURCE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_CONFIGURESHAREDRESOURCE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DAUTHENTICATEDCHANNEL_CONFIGUREUNCOMPRESSEDENCRYPTION {
    pub Parameters: D3DAUTHENTICATEDCHANNEL_CONFIGURE_INPUT,
    pub EncryptionGuid: ::windows_sys::core::GUID,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_CONFIGUREUNCOMPRESSEDENCRYPTION {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_CONFIGUREUNCOMPRESSEDENCRYPTION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DAUTHENTICATEDCHANNEL_CONFIGURE_INPUT {
    pub omac: D3D_OMAC,
    pub ConfigureType: ::windows_sys::core::GUID,
    pub hChannel: super::super::Foundation::HANDLE,
    pub SequenceNumber: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_CONFIGURE_INPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_CONFIGURE_INPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DAUTHENTICATEDCHANNEL_CONFIGURE_OUTPUT {
    pub omac: D3D_OMAC,
    pub ConfigureType: ::windows_sys::core::GUID,
    pub hChannel: super::super::Foundation::HANDLE,
    pub SequenceNumber: u32,
    pub ReturnCode: ::windows_sys::core::HRESULT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_CONFIGURE_OUTPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_CONFIGURE_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DAUTHENTICATEDCHANNEL_PROCESSIDENTIFIERTYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const PROCESSIDTYPE_UNKNOWN: D3DAUTHENTICATEDCHANNEL_PROCESSIDENTIFIERTYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const PROCESSIDTYPE_DWM: D3DAUTHENTICATEDCHANNEL_PROCESSIDENTIFIERTYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const PROCESSIDTYPE_HANDLE: D3DAUTHENTICATEDCHANNEL_PROCESSIDENTIFIERTYPE = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DAUTHENTICATEDCHANNEL_PROTECTION_FLAGS {
    pub Anonymous: D3DAUTHENTICATEDCHANNEL_PROTECTION_FLAGS_0,
}
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_PROTECTION_FLAGS {}
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_PROTECTION_FLAGS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub union D3DAUTHENTICATEDCHANNEL_PROTECTION_FLAGS_0 {
    pub Anonymous: D3DAUTHENTICATEDCHANNEL_PROTECTION_FLAGS_0_0,
    pub Value: u32,
}
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_PROTECTION_FLAGS_0 {}
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_PROTECTION_FLAGS_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DAUTHENTICATEDCHANNEL_PROTECTION_FLAGS_0_0 {
    pub _bitfield: u32,
}
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_PROTECTION_FLAGS_0_0 {}
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_PROTECTION_FLAGS_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DAUTHENTICATEDCHANNEL_QUERYCHANNELTYPE_OUTPUT {
    pub Output: D3DAUTHENTICATEDCHANNEL_QUERY_OUTPUT,
    pub ChannelType: D3DAUTHENTICATEDCHANNELTYPE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_QUERYCHANNELTYPE_OUTPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_QUERYCHANNELTYPE_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DAUTHENTICATEDCHANNEL_QUERYCRYPTOSESSION_INPUT {
    pub Input: D3DAUTHENTICATEDCHANNEL_QUERY_INPUT,
    pub DXVA2DecodeHandle: super::super::Foundation::HANDLE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_QUERYCRYPTOSESSION_INPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_QUERYCRYPTOSESSION_INPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DAUTHENTICATEDCHANNEL_QUERYCRYPTOSESSION_OUTPUT {
    pub Output: D3DAUTHENTICATEDCHANNEL_QUERY_OUTPUT,
    pub DXVA2DecodeHandle: super::super::Foundation::HANDLE,
    pub CryptoSessionHandle: super::super::Foundation::HANDLE,
    pub DeviceHandle: super::super::Foundation::HANDLE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_QUERYCRYPTOSESSION_OUTPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_QUERYCRYPTOSESSION_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DAUTHENTICATEDCHANNEL_QUERYDEVICEHANDLE_OUTPUT {
    pub Output: D3DAUTHENTICATEDCHANNEL_QUERY_OUTPUT,
    pub DeviceHandle: super::super::Foundation::HANDLE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_QUERYDEVICEHANDLE_OUTPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_QUERYDEVICEHANDLE_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DAUTHENTICATEDCHANNEL_QUERYEVICTIONENCRYPTIONGUIDCOUNT_OUTPUT {
    pub Output: D3DAUTHENTICATEDCHANNEL_QUERY_OUTPUT,
    pub NumEncryptionGuids: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_QUERYEVICTIONENCRYPTIONGUIDCOUNT_OUTPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_QUERYEVICTIONENCRYPTIONGUIDCOUNT_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DAUTHENTICATEDCHANNEL_QUERYEVICTIONENCRYPTIONGUID_INPUT {
    pub Input: D3DAUTHENTICATEDCHANNEL_QUERY_INPUT,
    pub EncryptionGuidIndex: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_QUERYEVICTIONENCRYPTIONGUID_INPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_QUERYEVICTIONENCRYPTIONGUID_INPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DAUTHENTICATEDCHANNEL_QUERYEVICTIONENCRYPTIONGUID_OUTPUT {
    pub Output: D3DAUTHENTICATEDCHANNEL_QUERY_OUTPUT,
    pub EncryptionGuidIndex: u32,
    pub EncryptionGuid: ::windows_sys::core::GUID,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_QUERYEVICTIONENCRYPTIONGUID_OUTPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_QUERYEVICTIONENCRYPTIONGUID_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DAUTHENTICATEDCHANNEL_QUERYINFOBUSTYPE_OUTPUT {
    pub Output: D3DAUTHENTICATEDCHANNEL_QUERY_OUTPUT,
    pub BusType: D3DBUSTYPE,
    pub bAccessibleInContiguousBlocks: super::super::Foundation::BOOL,
    pub bAccessibleInNonContiguousBlocks: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_QUERYINFOBUSTYPE_OUTPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_QUERYINFOBUSTYPE_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DAUTHENTICATEDCHANNEL_QUERYOUTPUTIDCOUNT_INPUT {
    pub Input: D3DAUTHENTICATEDCHANNEL_QUERY_INPUT,
    pub DeviceHandle: super::super::Foundation::HANDLE,
    pub CryptoSessionHandle: super::super::Foundation::HANDLE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_QUERYOUTPUTIDCOUNT_INPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_QUERYOUTPUTIDCOUNT_INPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DAUTHENTICATEDCHANNEL_QUERYOUTPUTIDCOUNT_OUTPUT {
    pub Output: D3DAUTHENTICATEDCHANNEL_QUERY_OUTPUT,
    pub DeviceHandle: super::super::Foundation::HANDLE,
    pub CryptoSessionHandle: super::super::Foundation::HANDLE,
    pub NumOutputIDs: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_QUERYOUTPUTIDCOUNT_OUTPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_QUERYOUTPUTIDCOUNT_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DAUTHENTICATEDCHANNEL_QUERYOUTPUTID_INPUT {
    pub Input: D3DAUTHENTICATEDCHANNEL_QUERY_INPUT,
    pub DeviceHandle: super::super::Foundation::HANDLE,
    pub CryptoSessionHandle: super::super::Foundation::HANDLE,
    pub OutputIDIndex: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_QUERYOUTPUTID_INPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_QUERYOUTPUTID_INPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DAUTHENTICATEDCHANNEL_QUERYOUTPUTID_OUTPUT {
    pub Output: D3DAUTHENTICATEDCHANNEL_QUERY_OUTPUT,
    pub DeviceHandle: super::super::Foundation::HANDLE,
    pub CryptoSessionHandle: super::super::Foundation::HANDLE,
    pub OutputIDIndex: u32,
    pub OutputID: u64,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_QUERYOUTPUTID_OUTPUT {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_QUERYOUTPUTID_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(4))]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DAUTHENTICATEDCHANNEL_QUERYOUTPUTID_OUTPUT {
    pub Output: D3DAUTHENTICATEDCHANNEL_QUERY_OUTPUT,
    pub DeviceHandle: super::super::Foundation::HANDLE,
    pub CryptoSessionHandle: super::super::Foundation::HANDLE,
    pub OutputIDIndex: u32,
    pub OutputID: u64,
}
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_QUERYOUTPUTID_OUTPUT {}
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_QUERYOUTPUTID_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DAUTHENTICATEDCHANNEL_QUERYPROTECTION_OUTPUT {
    pub Output: D3DAUTHENTICATEDCHANNEL_QUERY_OUTPUT,
    pub ProtectionFlags: D3DAUTHENTICATEDCHANNEL_PROTECTION_FLAGS,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_QUERYPROTECTION_OUTPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_QUERYPROTECTION_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DAUTHENTICATEDCHANNEL_QUERYRESTRICTEDSHAREDRESOURCEPROCESSCOUNT_OUTPUT {
    pub Output: D3DAUTHENTICATEDCHANNEL_QUERY_OUTPUT,
    pub NumRestrictedSharedResourceProcesses: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_QUERYRESTRICTEDSHAREDRESOURCEPROCESSCOUNT_OUTPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_QUERYRESTRICTEDSHAREDRESOURCEPROCESSCOUNT_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DAUTHENTICATEDCHANNEL_QUERYRESTRICTEDSHAREDRESOURCEPROCESS_INPUT {
    pub Input: D3DAUTHENTICATEDCHANNEL_QUERY_INPUT,
    pub ProcessIndex: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_QUERYRESTRICTEDSHAREDRESOURCEPROCESS_INPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_QUERYRESTRICTEDSHAREDRESOURCEPROCESS_INPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DAUTHENTICATEDCHANNEL_QUERYRESTRICTEDSHAREDRESOURCEPROCESS_OUTPUT {
    pub Output: D3DAUTHENTICATEDCHANNEL_QUERY_OUTPUT,
    pub ProcessIndex: u32,
    pub ProcessIdentifer: D3DAUTHENTICATEDCHANNEL_PROCESSIDENTIFIERTYPE,
    pub ProcessHandle: super::super::Foundation::HANDLE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_QUERYRESTRICTEDSHAREDRESOURCEPROCESS_OUTPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_QUERYRESTRICTEDSHAREDRESOURCEPROCESS_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DAUTHENTICATEDCHANNEL_QUERYUNCOMPRESSEDENCRYPTIONLEVEL_OUTPUT {
    pub Output: D3DAUTHENTICATEDCHANNEL_QUERY_OUTPUT,
    pub EncryptionGuid: ::windows_sys::core::GUID,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_QUERYUNCOMPRESSEDENCRYPTIONLEVEL_OUTPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_QUERYUNCOMPRESSEDENCRYPTIONLEVEL_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DAUTHENTICATEDCHANNEL_QUERYUNRESTRICTEDPROTECTEDSHAREDRESOURCECOUNT_OUTPUT {
    pub Output: D3DAUTHENTICATEDCHANNEL_QUERY_OUTPUT,
    pub NumUnrestrictedProtectedSharedResources: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_QUERYUNRESTRICTEDPROTECTEDSHAREDRESOURCECOUNT_OUTPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_QUERYUNRESTRICTEDPROTECTEDSHAREDRESOURCECOUNT_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DAUTHENTICATEDCHANNEL_QUERY_INPUT {
    pub QueryType: ::windows_sys::core::GUID,
    pub hChannel: super::super::Foundation::HANDLE,
    pub SequenceNumber: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_QUERY_INPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_QUERY_INPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DAUTHENTICATEDCHANNEL_QUERY_OUTPUT {
    pub omac: D3D_OMAC,
    pub QueryType: ::windows_sys::core::GUID,
    pub hChannel: super::super::Foundation::HANDLE,
    pub SequenceNumber: u32,
    pub ReturnCode: ::windows_sys::core::HRESULT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DAUTHENTICATEDCHANNEL_QUERY_OUTPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DAUTHENTICATEDCHANNEL_QUERY_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
pub const D3DAUTHENTICATEDCONFIGURE_CRYPTOSESSION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1665584212, data2: 11516, data3: 19156, data4: [130, 36, 209, 88, 55, 222, 119, 0] };
pub const D3DAUTHENTICATEDCONFIGURE_ENCRYPTIONWHENACCESSIBLE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1107292806, data2: 27360, data3: 19779, data4: [157, 85, 164, 110, 158, 253, 21, 138] };
pub const D3DAUTHENTICATEDCONFIGURE_INITIALIZE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 101796827, data2: 13603, data3: 18186, data4: [141, 202, 251, 194, 132, 81, 84, 240] };
pub const D3DAUTHENTICATEDCONFIGURE_PROTECTION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1346721368, data2: 16199, data3: 17250, data4: [191, 153, 191, 223, 205, 233, 237, 41] };
pub const D3DAUTHENTICATEDCONFIGURE_SHAREDRESOURCE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 124964935, data2: 6976, data3: 18664, data4: [156, 166, 181, 245, 16, 222, 159, 1] };
pub const D3DAUTHENTICATEDQUERY_ACCESSIBILITYATTRIBUTES: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1645533650, data2: 17196, data3: 19131, data4: [159, 206, 33, 110, 234, 38, 158, 59] };
pub const D3DAUTHENTICATEDQUERY_CHANNELTYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3155892389, data2: 45563, data3: 17067, data4: [189, 148, 181, 130, 139, 75, 247, 190] };
pub const D3DAUTHENTICATEDQUERY_CRYPTOSESSION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 640960926, data2: 53272, data3: 19828, data4: [172, 23, 127, 114, 64, 89, 82, 141] };
pub const D3DAUTHENTICATEDQUERY_CURRENTENCRYPTIONWHENACCESSIBLE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3960967623, data2: 56019, data3: 20245, data4: [158, 195, 250, 169, 61, 96, 212, 240] };
pub const D3DAUTHENTICATEDQUERY_DEVICEHANDLE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3961279389, data2: 36095, data3: 20010, data4: [188, 196, 245, 105, 47, 153, 244, 128] };
pub const D3DAUTHENTICATEDQUERY_ENCRYPTIONWHENACCESSIBLEGUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4164573528, data2: 59782, data3: 19418, data4: [190, 176, 65, 31, 106, 122, 1, 183] };
pub const D3DAUTHENTICATEDQUERY_ENCRYPTIONWHENACCESSIBLEGUIDCOUNT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3004133478, data2: 8252, data3: 19207, data4: [147, 252, 206, 170, 253, 97, 36, 30] };
pub const D3DAUTHENTICATEDQUERY_OUTPUTID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2208160931, data2: 39758, data3: 16868, data4: [176, 83, 137, 43, 210, 161, 30, 231] };
pub const D3DAUTHENTICATEDQUERY_OUTPUTIDCOUNT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 738470750, data2: 35847, data3: 18133, data4: [170, 190, 143, 117, 203, 173, 76, 49] };
pub const D3DAUTHENTICATEDQUERY_PROTECTION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2823730564, data2: 50325, data3: 18602, data4: [185, 77, 139, 210, 214, 251, 206, 5] };
pub const D3DAUTHENTICATEDQUERY_RESTRICTEDSHAREDRESOURCEPROCESS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1687927515, data2: 61684, data3: 17977, data4: [161, 91, 36, 57, 63, 195, 171, 172] };
pub const D3DAUTHENTICATEDQUERY_RESTRICTEDSHAREDRESOURCEPROCESSCOUNT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 229771187, data2: 37968, data3: 18086, data4: [130, 222, 27, 150, 212, 79, 156, 242] };
pub const D3DAUTHENTICATEDQUERY_UNRESTRICTEDPROTECTEDSHAREDRESOURCECOUNT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 19860438, data2: 58978, data3: 17524, data4: [190, 253, 170, 83, 229, 20, 60, 109] };
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DBACKBUFFER_TYPE = u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBACKBUFFER_TYPE_MONO: D3DBACKBUFFER_TYPE = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBACKBUFFER_TYPE_LEFT: D3DBACKBUFFER_TYPE = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBACKBUFFER_TYPE_RIGHT: D3DBACKBUFFER_TYPE = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBACKBUFFER_TYPE_FORCE_DWORD: D3DBACKBUFFER_TYPE = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DBASISTYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBASIS_BEZIER: D3DBASISTYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBASIS_BSPLINE: D3DBASISTYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBASIS_CATMULL_ROM: D3DBASISTYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBASIS_FORCE_DWORD: D3DBASISTYPE = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DBLEND = u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBLEND_ZERO: D3DBLEND = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBLEND_ONE: D3DBLEND = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBLEND_SRCCOLOR: D3DBLEND = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBLEND_INVSRCCOLOR: D3DBLEND = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBLEND_SRCALPHA: D3DBLEND = 5u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBLEND_INVSRCALPHA: D3DBLEND = 6u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBLEND_DESTALPHA: D3DBLEND = 7u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBLEND_INVDESTALPHA: D3DBLEND = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBLEND_DESTCOLOR: D3DBLEND = 9u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBLEND_INVDESTCOLOR: D3DBLEND = 10u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBLEND_SRCALPHASAT: D3DBLEND = 11u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBLEND_BOTHSRCALPHA: D3DBLEND = 12u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBLEND_BOTHINVSRCALPHA: D3DBLEND = 13u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBLEND_BLENDFACTOR: D3DBLEND = 14u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBLEND_INVBLENDFACTOR: D3DBLEND = 15u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBLEND_SRCCOLOR2: D3DBLEND = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBLEND_INVSRCCOLOR2: D3DBLEND = 17u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBLEND_FORCE_DWORD: D3DBLEND = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DBLENDOP = u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBLENDOP_ADD: D3DBLENDOP = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBLENDOP_SUBTRACT: D3DBLENDOP = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBLENDOP_REVSUBTRACT: D3DBLENDOP = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBLENDOP_MIN: D3DBLENDOP = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBLENDOP_MAX: D3DBLENDOP = 5u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBLENDOP_FORCE_DWORD: D3DBLENDOP = 2147483647u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DBOX {
    pub Left: u32,
    pub Top: u32,
    pub Right: u32,
    pub Bottom: u32,
    pub Front: u32,
    pub Back: u32,
}
impl ::core::marker::Copy for D3DBOX {}
impl ::core::clone::Clone for D3DBOX {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DBUSTYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBUSTYPE_OTHER: D3DBUSTYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBUSTYPE_PCI: D3DBUSTYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBUSTYPE_PCIX: D3DBUSTYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBUSTYPE_PCIEXPRESS: D3DBUSTYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBUSTYPE_AGP: D3DBUSTYPE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBUSIMPL_MODIFIER_INSIDE_OF_CHIPSET: D3DBUSTYPE = 65536i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBUSIMPL_MODIFIER_TRACKS_ON_MOTHER_BOARD_TO_CHIP: D3DBUSTYPE = 131072i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBUSIMPL_MODIFIER_TRACKS_ON_MOTHER_BOARD_TO_SOCKET: D3DBUSTYPE = 196608i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBUSIMPL_MODIFIER_DAUGHTER_BOARD_CONNECTOR: D3DBUSTYPE = 262144i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBUSIMPL_MODIFIER_DAUGHTER_BOARD_CONNECTOR_INSIDE_OF_NUAE: D3DBUSTYPE = 327680i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DBUSIMPL_MODIFIER_NON_STANDARD: D3DBUSTYPE = -2147483648i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCAPS2_CANAUTOGENMIPMAP: i32 = 1073741824i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCAPS2_CANCALIBRATEGAMMA: i32 = 1048576i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCAPS2_CANMANAGERESOURCE: i32 = 268435456i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCAPS2_CANSHARERESOURCE: i32 = -2147483648i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCAPS2_DYNAMICTEXTURES: i32 = 536870912i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCAPS2_FULLSCREENGAMMA: i32 = 131072i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCAPS2_RESERVED: i32 = 33554432i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCAPS3_ALPHA_FULLSCREEN_FLIP_OR_DISCARD: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCAPS3_COPY_TO_SYSTEMMEM: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCAPS3_COPY_TO_VIDMEM: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCAPS3_DXVAHD: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCAPS3_DXVAHD_LIMITED: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCAPS3_LINEAR_TO_SRGB_PRESENTATION: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCAPS3_RESERVED: i32 = -2147483617i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DCAPS9 {
    pub DeviceType: D3DDEVTYPE,
    pub AdapterOrdinal: u32,
    pub Caps: u32,
    pub Caps2: u32,
    pub Caps3: u32,
    pub PresentationIntervals: u32,
    pub CursorCaps: u32,
    pub DevCaps: u32,
    pub PrimitiveMiscCaps: u32,
    pub RasterCaps: u32,
    pub ZCmpCaps: u32,
    pub SrcBlendCaps: u32,
    pub DestBlendCaps: u32,
    pub AlphaCmpCaps: u32,
    pub ShadeCaps: u32,
    pub TextureCaps: u32,
    pub TextureFilterCaps: u32,
    pub CubeTextureFilterCaps: u32,
    pub VolumeTextureFilterCaps: u32,
    pub TextureAddressCaps: u32,
    pub VolumeTextureAddressCaps: u32,
    pub LineCaps: u32,
    pub MaxTextureWidth: u32,
    pub MaxTextureHeight: u32,
    pub MaxVolumeExtent: u32,
    pub MaxTextureRepeat: u32,
    pub MaxTextureAspectRatio: u32,
    pub MaxAnisotropy: u32,
    pub MaxVertexW: f32,
    pub GuardBandLeft: f32,
    pub GuardBandTop: f32,
    pub GuardBandRight: f32,
    pub GuardBandBottom: f32,
    pub ExtentsAdjust: f32,
    pub StencilCaps: u32,
    pub FVFCaps: u32,
    pub TextureOpCaps: u32,
    pub MaxTextureBlendStages: u32,
    pub MaxSimultaneousTextures: u32,
    pub VertexProcessingCaps: u32,
    pub MaxActiveLights: u32,
    pub MaxUserClipPlanes: u32,
    pub MaxVertexBlendMatrices: u32,
    pub MaxVertexBlendMatrixIndex: u32,
    pub MaxPointSize: f32,
    pub MaxPrimitiveCount: u32,
    pub MaxVertexIndex: u32,
    pub MaxStreams: u32,
    pub MaxStreamStride: u32,
    pub VertexShaderVersion: u32,
    pub MaxVertexShaderConst: u32,
    pub PixelShaderVersion: u32,
    pub PixelShader1xMaxValue: f32,
    pub DevCaps2: u32,
    pub MaxNpatchTessellationLevel: f32,
    pub Reserved5: u32,
    pub MasterAdapterOrdinal: u32,
    pub AdapterOrdinalInGroup: u32,
    pub NumberOfAdaptersInGroup: u32,
    pub DeclTypes: u32,
    pub NumSimultaneousRTs: u32,
    pub StretchRectFilterCaps: u32,
    pub VS20Caps: D3DVSHADERCAPS2_0,
    pub PS20Caps: D3DPSHADERCAPS2_0,
    pub VertexTextureFilterCaps: u32,
    pub MaxVShaderInstructionsExecuted: u32,
    pub MaxPShaderInstructionsExecuted: u32,
    pub MaxVertexShader30InstructionSlots: u32,
    pub MaxPixelShader30InstructionSlots: u32,
}
impl ::core::marker::Copy for D3DCAPS9 {}
impl ::core::clone::Clone for D3DCAPS9 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCAPS_OVERLAY: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCAPS_READ_SCANLINE: i32 = 131072i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DCLIPSTATUS9 {
    pub ClipUnion: u32,
    pub ClipIntersection: u32,
}
impl ::core::marker::Copy for D3DCLIPSTATUS9 {}
impl ::core::clone::Clone for D3DCLIPSTATUS9 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DCMPFUNC = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCMP_NEVER: D3DCMPFUNC = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCMP_LESS: D3DCMPFUNC = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCMP_EQUAL: D3DCMPFUNC = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCMP_LESSEQUAL: D3DCMPFUNC = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCMP_GREATER: D3DCMPFUNC = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCMP_NOTEQUAL: D3DCMPFUNC = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCMP_GREATEREQUAL: D3DCMPFUNC = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCMP_ALWAYS: D3DCMPFUNC = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCMP_FORCE_DWORD: D3DCMPFUNC = 2147483647i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DCOLORVALUE {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
impl ::core::marker::Copy for D3DCOLORVALUE {}
impl ::core::clone::Clone for D3DCOLORVALUE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DCOMPOSERECTDESC {
    pub X: u16,
    pub Y: u16,
    pub Width: u16,
    pub Height: u16,
}
impl ::core::marker::Copy for D3DCOMPOSERECTDESC {}
impl ::core::clone::Clone for D3DCOMPOSERECTDESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DCOMPOSERECTDESTINATION {
    pub SrcRectIndex: u16,
    pub Reserved: u16,
    pub X: i16,
    pub Y: i16,
}
impl ::core::marker::Copy for D3DCOMPOSERECTDESTINATION {}
impl ::core::clone::Clone for D3DCOMPOSERECTDESTINATION {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DCOMPOSERECTSOP = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCOMPOSERECTS_COPY: D3DCOMPOSERECTSOP = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCOMPOSERECTS_OR: D3DCOMPOSERECTSOP = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCOMPOSERECTS_AND: D3DCOMPOSERECTSOP = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCOMPOSERECTS_NEG: D3DCOMPOSERECTSOP = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCOMPOSERECTS_FORCE_DWORD: D3DCOMPOSERECTSOP = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCOMPOSERECTS_MAXNUMRECTS: u32 = 65535u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCONVOLUTIONMONO_MAXHEIGHT: u32 = 7u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCONVOLUTIONMONO_MAXWIDTH: u32 = 7u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCPCAPS_CONTENTKEY: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCPCAPS_ENCRYPTEDREADBACK: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCPCAPS_ENCRYPTEDREADBACKKEY: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCPCAPS_ENCRYPTSLICEDATAONLY: u32 = 512u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCPCAPS_FRESHENSESSIONKEY: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCPCAPS_HARDWARE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCPCAPS_PARTIALDECRYPTION: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCPCAPS_PROTECTIONALWAYSON: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCPCAPS_SEQUENTIAL_CTR_IV: u32 = 256u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCPCAPS_SOFTWARE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCREATE_ADAPTERGROUP_DEVICE: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCREATE_DISABLE_DRIVER_MANAGEMENT: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCREATE_DISABLE_DRIVER_MANAGEMENT_EX: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCREATE_DISABLE_PRINTSCREEN: i32 = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCREATE_DISABLE_PSGP_THREADING: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCREATE_ENABLE_PRESENTSTATS: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCREATE_FPU_PRESERVE: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCREATE_HARDWARE_VERTEXPROCESSING: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCREATE_MIXED_VERTEXPROCESSING: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCREATE_MULTITHREADED: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCREATE_NOWINDOWCHANGES: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCREATE_PUREDEVICE: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCREATE_SCREENSAVER: i32 = 268435456i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCREATE_SOFTWARE_VERTEXPROCESSING: i32 = 32i32;
pub const D3DCRYPTOTYPE_AES128_CTR: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2607535889, data2: 20340, data3: 16841, data4: [158, 123, 11, 226, 215, 217, 59, 79] };
pub const D3DCRYPTOTYPE_PROPRIETARY: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2874055421, data2: 7452, data3: 18150, data4: [167, 47, 8, 105, 145, 123, 13, 232] };
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCS_BACK: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCS_BOTTOM: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCS_FRONT: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCS_LEFT: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCS_PLANE0: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCS_PLANE1: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCS_PLANE2: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCS_PLANE3: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCS_PLANE4: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCS_PLANE5: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCS_RIGHT: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCS_TOP: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DCUBEMAP_FACES = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCUBEMAP_FACE_POSITIVE_X: D3DCUBEMAP_FACES = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCUBEMAP_FACE_NEGATIVE_X: D3DCUBEMAP_FACES = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCUBEMAP_FACE_POSITIVE_Y: D3DCUBEMAP_FACES = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCUBEMAP_FACE_NEGATIVE_Y: D3DCUBEMAP_FACES = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCUBEMAP_FACE_POSITIVE_Z: D3DCUBEMAP_FACES = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCUBEMAP_FACE_NEGATIVE_Z: D3DCUBEMAP_FACES = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCUBEMAP_FACE_FORCE_DWORD: D3DCUBEMAP_FACES = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DCULL = u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCULL_NONE: D3DCULL = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCULL_CW: D3DCULL = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCULL_CCW: D3DCULL = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCULL_FORCE_DWORD: D3DCULL = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCURSORCAPS_COLOR: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCURSORCAPS_LOWRES: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DCURSOR_IMMEDIATE_UPDATE: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DDEBUGMONITORTOKENS = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDMT_ENABLE: D3DDEBUGMONITORTOKENS = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDMT_DISABLE: D3DDEBUGMONITORTOKENS = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDMT_FORCE_DWORD: D3DDEBUGMONITORTOKENS = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DDECLMETHOD = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLMETHOD_DEFAULT: D3DDECLMETHOD = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLMETHOD_PARTIALU: D3DDECLMETHOD = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLMETHOD_PARTIALV: D3DDECLMETHOD = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLMETHOD_CROSSUV: D3DDECLMETHOD = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLMETHOD_UV: D3DDECLMETHOD = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLMETHOD_LOOKUP: D3DDECLMETHOD = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLMETHOD_LOOKUPPRESAMPLED: D3DDECLMETHOD = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DDECLTYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLTYPE_FLOAT1: D3DDECLTYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLTYPE_FLOAT2: D3DDECLTYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLTYPE_FLOAT3: D3DDECLTYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLTYPE_FLOAT4: D3DDECLTYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLTYPE_D3DCOLOR: D3DDECLTYPE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLTYPE_UBYTE4: D3DDECLTYPE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLTYPE_SHORT2: D3DDECLTYPE = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLTYPE_SHORT4: D3DDECLTYPE = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLTYPE_UBYTE4N: D3DDECLTYPE = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLTYPE_SHORT2N: D3DDECLTYPE = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLTYPE_SHORT4N: D3DDECLTYPE = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLTYPE_USHORT2N: D3DDECLTYPE = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLTYPE_USHORT4N: D3DDECLTYPE = 12i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLTYPE_UDEC3: D3DDECLTYPE = 13i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLTYPE_DEC3N: D3DDECLTYPE = 14i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLTYPE_FLOAT16_2: D3DDECLTYPE = 15i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLTYPE_FLOAT16_4: D3DDECLTYPE = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLTYPE_UNUSED: D3DDECLTYPE = 17i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DDECLUSAGE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLUSAGE_POSITION: D3DDECLUSAGE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLUSAGE_BLENDWEIGHT: D3DDECLUSAGE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLUSAGE_BLENDINDICES: D3DDECLUSAGE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLUSAGE_NORMAL: D3DDECLUSAGE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLUSAGE_PSIZE: D3DDECLUSAGE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLUSAGE_TEXCOORD: D3DDECLUSAGE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLUSAGE_TANGENT: D3DDECLUSAGE = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLUSAGE_BINORMAL: D3DDECLUSAGE = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLUSAGE_TESSFACTOR: D3DDECLUSAGE = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLUSAGE_POSITIONT: D3DDECLUSAGE = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLUSAGE_COLOR: D3DDECLUSAGE = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLUSAGE_FOG: D3DDECLUSAGE = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLUSAGE_DEPTH: D3DDECLUSAGE = 12i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDECLUSAGE_SAMPLE: D3DDECLUSAGE = 13i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DDEGREETYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDEGREE_LINEAR: D3DDEGREETYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDEGREE_QUADRATIC: D3DDEGREETYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDEGREE_CUBIC: D3DDEGREETYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDEGREE_QUINTIC: D3DDEGREETYPE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDEGREE_FORCE_DWORD: D3DDEGREETYPE = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDEVCAPS2_ADAPTIVETESSNPATCH: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDEVCAPS2_ADAPTIVETESSRTPATCH: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDEVCAPS2_CAN_STRETCHRECT_FROM_TEXTURES: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDEVCAPS2_DMAPNPATCH: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDEVCAPS2_PRESAMPLEDDMAPNPATCH: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDEVCAPS2_STREAMOFFSET: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDEVCAPS2_VERTEXELEMENTSCANSHARESTREAMOFFSET: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDEVCAPS_NPATCHES: i32 = 16777216i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDEVCAPS_PUREDEVICE: i32 = 1048576i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDEVCAPS_QUINTICRTPATCHES: i32 = 2097152i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDEVCAPS_RTPATCHES: i32 = 4194304i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDEVCAPS_RTPATCHHANDLEZERO: i32 = 8388608i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DDEVICE_CREATION_PARAMETERS {
    pub AdapterOrdinal: u32,
    pub DeviceType: D3DDEVTYPE,
    pub hFocusWindow: super::super::Foundation::HWND,
    pub BehaviorFlags: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DDEVICE_CREATION_PARAMETERS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DDEVICE_CREATION_PARAMETERS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DDEVINFO_D3D9BANDWIDTHTIMINGS {
    pub MaxBandwidthUtilized: f32,
    pub FrontEndUploadMemoryUtilizedPercent: f32,
    pub VertexRateUtilizedPercent: f32,
    pub TriangleSetupRateUtilizedPercent: f32,
    pub FillRateUtilizedPercent: f32,
}
impl ::core::marker::Copy for D3DDEVINFO_D3D9BANDWIDTHTIMINGS {}
impl ::core::clone::Clone for D3DDEVINFO_D3D9BANDWIDTHTIMINGS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DDEVINFO_D3D9CACHEUTILIZATION {
    pub TextureCacheHitRate: f32,
    pub PostTransformVertexCacheHitRate: f32,
}
impl ::core::marker::Copy for D3DDEVINFO_D3D9CACHEUTILIZATION {}
impl ::core::clone::Clone for D3DDEVINFO_D3D9CACHEUTILIZATION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DDEVINFO_D3D9INTERFACETIMINGS {
    pub WaitingForGPUToUseApplicationResourceTimePercent: f32,
    pub WaitingForGPUToAcceptMoreCommandsTimePercent: f32,
    pub WaitingForGPUToStayWithinLatencyTimePercent: f32,
    pub WaitingForGPUExclusiveResourceTimePercent: f32,
    pub WaitingForGPUOtherTimePercent: f32,
}
impl ::core::marker::Copy for D3DDEVINFO_D3D9INTERFACETIMINGS {}
impl ::core::clone::Clone for D3DDEVINFO_D3D9INTERFACETIMINGS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DDEVINFO_D3D9PIPELINETIMINGS {
    pub VertexProcessingTimePercent: f32,
    pub PixelProcessingTimePercent: f32,
    pub OtherGPUProcessingTimePercent: f32,
    pub GPUIdleTimePercent: f32,
}
impl ::core::marker::Copy for D3DDEVINFO_D3D9PIPELINETIMINGS {}
impl ::core::clone::Clone for D3DDEVINFO_D3D9PIPELINETIMINGS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DDEVINFO_D3D9STAGETIMINGS {
    pub MemoryProcessingPercent: f32,
    pub ComputationProcessingPercent: f32,
}
impl ::core::marker::Copy for D3DDEVINFO_D3D9STAGETIMINGS {}
impl ::core::clone::Clone for D3DDEVINFO_D3D9STAGETIMINGS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DDEVINFO_D3DVERTEXSTATS {
    pub NumRenderedTriangles: u32,
    pub NumExtraClippingTriangles: u32,
}
impl ::core::marker::Copy for D3DDEVINFO_D3DVERTEXSTATS {}
impl ::core::clone::Clone for D3DDEVINFO_D3DVERTEXSTATS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DDEVINFO_RESOURCEMANAGER {
    pub stats: [D3DRESOURCESTATS; 8],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DDEVINFO_RESOURCEMANAGER {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DDEVINFO_RESOURCEMANAGER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DDEVINFO_VCACHE {
    pub Pattern: u32,
    pub OptMethod: u32,
    pub CacheSize: u32,
    pub MagicNumber: u32,
}
impl ::core::marker::Copy for D3DDEVINFO_VCACHE {}
impl ::core::clone::Clone for D3DDEVINFO_VCACHE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DDEVTYPE = u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDEVTYPE_HAL: D3DDEVTYPE = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDEVTYPE_REF: D3DDEVTYPE = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDEVTYPE_SW: D3DDEVTYPE = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDEVTYPE_NULLREF: D3DDEVTYPE = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDEVTYPE_FORCE_DWORD: D3DDEVTYPE = 2147483647u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DDISPLAYMODE {
    pub Width: u32,
    pub Height: u32,
    pub RefreshRate: u32,
    pub Format: D3DFORMAT,
}
impl ::core::marker::Copy for D3DDISPLAYMODE {}
impl ::core::clone::Clone for D3DDISPLAYMODE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DDISPLAYMODEEX {
    pub Size: u32,
    pub Width: u32,
    pub Height: u32,
    pub RefreshRate: u32,
    pub Format: D3DFORMAT,
    pub ScanLineOrdering: D3DSCANLINEORDERING,
}
impl ::core::marker::Copy for D3DDISPLAYMODEEX {}
impl ::core::clone::Clone for D3DDISPLAYMODEEX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DDISPLAYMODEFILTER {
    pub Size: u32,
    pub Format: D3DFORMAT,
    pub ScanLineOrdering: D3DSCANLINEORDERING,
}
impl ::core::marker::Copy for D3DDISPLAYMODEFILTER {}
impl ::core::clone::Clone for D3DDISPLAYMODEFILTER {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DDISPLAYROTATION = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDISPLAYROTATION_IDENTITY: D3DDISPLAYROTATION = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDISPLAYROTATION_90: D3DDISPLAYROTATION = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDISPLAYROTATION_180: D3DDISPLAYROTATION = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDISPLAYROTATION_270: D3DDISPLAYROTATION = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDMAPSAMPLER: u32 = 256u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDTCAPS_DEC3N: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDTCAPS_FLOAT16_2: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDTCAPS_FLOAT16_4: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDTCAPS_SHORT2N: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDTCAPS_SHORT4N: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDTCAPS_UBYTE4: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDTCAPS_UBYTE4N: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDTCAPS_UDEC3: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDTCAPS_USHORT2N: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DDTCAPS_USHORT4N: i32 = 32i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DENCRYPTED_BLOCK_INFO {
    pub NumEncryptedBytesAtBeginning: u32,
    pub NumBytesInSkipPattern: u32,
    pub NumBytesInEncryptPattern: u32,
}
impl ::core::marker::Copy for D3DENCRYPTED_BLOCK_INFO {}
impl ::core::clone::Clone for D3DENCRYPTED_BLOCK_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DENUM_NO_DRIVERVERSION: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DENUM_WHQL_LEVEL: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DFILLMODE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFILL_POINT: D3DFILLMODE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFILL_WIREFRAME: D3DFILLMODE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFILL_SOLID: D3DFILLMODE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFILL_FORCE_DWORD: D3DFILLMODE = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_A1_SURFACE_MAXHEIGHT: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_A1_SURFACE_MAXWIDTH: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DFOGMODE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFOG_NONE: D3DFOGMODE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFOG_EXP: D3DFOGMODE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFOG_EXP2: D3DFOGMODE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFOG_LINEAR: D3DFOGMODE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFOG_FORCE_DWORD: D3DFOGMODE = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DFORMAT = u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_UNKNOWN: D3DFORMAT = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_R8G8B8: D3DFORMAT = 20u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_A8R8G8B8: D3DFORMAT = 21u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_X8R8G8B8: D3DFORMAT = 22u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_R5G6B5: D3DFORMAT = 23u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_X1R5G5B5: D3DFORMAT = 24u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_A1R5G5B5: D3DFORMAT = 25u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_A4R4G4B4: D3DFORMAT = 26u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_R3G3B2: D3DFORMAT = 27u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_A8: D3DFORMAT = 28u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_A8R3G3B2: D3DFORMAT = 29u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_X4R4G4B4: D3DFORMAT = 30u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_A2B10G10R10: D3DFORMAT = 31u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_A8B8G8R8: D3DFORMAT = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_X8B8G8R8: D3DFORMAT = 33u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_G16R16: D3DFORMAT = 34u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_A2R10G10B10: D3DFORMAT = 35u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_A16B16G16R16: D3DFORMAT = 36u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_A8P8: D3DFORMAT = 40u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_P8: D3DFORMAT = 41u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_L8: D3DFORMAT = 50u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_A8L8: D3DFORMAT = 51u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_A4L4: D3DFORMAT = 52u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_V8U8: D3DFORMAT = 60u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_L6V5U5: D3DFORMAT = 61u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_X8L8V8U8: D3DFORMAT = 62u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_Q8W8V8U8: D3DFORMAT = 63u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_V16U16: D3DFORMAT = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_A2W10V10U10: D3DFORMAT = 67u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_UYVY: D3DFORMAT = 1498831189u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_R8G8_B8G8: D3DFORMAT = 1195525970u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_YUY2: D3DFORMAT = 844715353u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_G8R8_G8B8: D3DFORMAT = 1111970375u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_DXT1: D3DFORMAT = 827611204u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_DXT2: D3DFORMAT = 844388420u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_DXT3: D3DFORMAT = 861165636u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_DXT4: D3DFORMAT = 877942852u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_DXT5: D3DFORMAT = 894720068u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_D16_LOCKABLE: D3DFORMAT = 70u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_D32: D3DFORMAT = 71u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_D15S1: D3DFORMAT = 73u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_D24S8: D3DFORMAT = 75u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_D24X8: D3DFORMAT = 77u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_D24X4S4: D3DFORMAT = 79u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_D16: D3DFORMAT = 80u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_D32F_LOCKABLE: D3DFORMAT = 82u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_D24FS8: D3DFORMAT = 83u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_D32_LOCKABLE: D3DFORMAT = 84u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_S8_LOCKABLE: D3DFORMAT = 85u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_L16: D3DFORMAT = 81u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_VERTEXDATA: D3DFORMAT = 100u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_INDEX16: D3DFORMAT = 101u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_INDEX32: D3DFORMAT = 102u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_Q16W16V16U16: D3DFORMAT = 110u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_MULTI2_ARGB8: D3DFORMAT = 827606349u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_R16F: D3DFORMAT = 111u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_G16R16F: D3DFORMAT = 112u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_A16B16G16R16F: D3DFORMAT = 113u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_R32F: D3DFORMAT = 114u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_G32R32F: D3DFORMAT = 115u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_A32B32G32R32F: D3DFORMAT = 116u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_CxV8U8: D3DFORMAT = 117u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_A1: D3DFORMAT = 118u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_A2B10G10R10_XR_BIAS: D3DFORMAT = 119u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_BINARYBUFFER: D3DFORMAT = 199u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFMT_FORCE_DWORD: D3DFORMAT = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFVFCAPS_PSIZE: i32 = 1048576i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFVF_LASTBETA_D3DCOLOR: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFVF_LASTBETA_UBYTE4: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFVF_PSIZE: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DFVF_XYZW: u32 = 16386u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DGAMMARAMP {
    pub red: [u16; 256],
    pub green: [u16; 256],
    pub blue: [u16; 256],
}
impl ::core::marker::Copy for D3DGAMMARAMP {}
impl ::core::clone::Clone for D3DGAMMARAMP {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DGETDATA_FLUSH: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DINDEXBUFFER_DESC {
    pub Format: D3DFORMAT,
    pub Type: D3DRESOURCETYPE,
    pub Usage: u32,
    pub Pool: D3DPOOL,
    pub Size: u32,
}
impl ::core::marker::Copy for D3DINDEXBUFFER_DESC {}
impl ::core::clone::Clone for D3DINDEXBUFFER_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DISSUE_BEGIN: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DISSUE_END: u32 = 1u32;
pub const D3DKEYEXCHANGE_DXVA: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1137932124, data2: 14565, data3: 18724, data4: [141, 134, 211, 252, 207, 21, 62, 155] };
pub const D3DKEYEXCHANGE_RSAES_OAEP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3247741077, data2: 55082, data3: 18973, data4: [142, 93, 237, 133, 125, 23, 21, 32] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Graphics_Direct3D\"`*"]
#[cfg(feature = "Win32_Graphics_Direct3D")]
pub struct D3DLIGHT9 {
    pub Type: D3DLIGHTTYPE,
    pub Diffuse: D3DCOLORVALUE,
    pub Specular: D3DCOLORVALUE,
    pub Ambient: D3DCOLORVALUE,
    pub Position: super::Direct3D::D3DVECTOR,
    pub Direction: super::Direct3D::D3DVECTOR,
    pub Range: f32,
    pub Falloff: f32,
    pub Attenuation0: f32,
    pub Attenuation1: f32,
    pub Attenuation2: f32,
    pub Theta: f32,
    pub Phi: f32,
}
#[cfg(feature = "Win32_Graphics_Direct3D")]
impl ::core::marker::Copy for D3DLIGHT9 {}
#[cfg(feature = "Win32_Graphics_Direct3D")]
impl ::core::clone::Clone for D3DLIGHT9 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DLIGHTTYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DLIGHT_POINT: D3DLIGHTTYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DLIGHT_SPOT: D3DLIGHTTYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DLIGHT_DIRECTIONAL: D3DLIGHTTYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DLIGHT_FORCE_DWORD: D3DLIGHTTYPE = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DLINECAPS_ALPHACMP: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DLINECAPS_ANTIALIAS: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DLINECAPS_BLEND: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DLINECAPS_FOG: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DLINECAPS_TEXTURE: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DLINECAPS_ZTEST: i32 = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DLOCKED_BOX {
    pub RowPitch: i32,
    pub SlicePitch: i32,
    pub pBits: *mut ::core::ffi::c_void,
}
impl ::core::marker::Copy for D3DLOCKED_BOX {}
impl ::core::clone::Clone for D3DLOCKED_BOX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DLOCKED_RECT {
    pub Pitch: i32,
    pub pBits: *mut ::core::ffi::c_void,
}
impl ::core::marker::Copy for D3DLOCKED_RECT {}
impl ::core::clone::Clone for D3DLOCKED_RECT {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DLOCK_DISCARD: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DLOCK_DONOTWAIT: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DLOCK_NOOVERWRITE: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DLOCK_NOSYSLOCK: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DLOCK_NO_DIRTY_UPDATE: i32 = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DLOCK_READONLY: i32 = 16i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DMATERIAL9 {
    pub Diffuse: D3DCOLORVALUE,
    pub Ambient: D3DCOLORVALUE,
    pub Specular: D3DCOLORVALUE,
    pub Emissive: D3DCOLORVALUE,
    pub Power: f32,
}
impl ::core::marker::Copy for D3DMATERIAL9 {}
impl ::core::clone::Clone for D3DMATERIAL9 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DMATERIALCOLORSOURCE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DMCS_MATERIAL: D3DMATERIALCOLORSOURCE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DMCS_COLOR1: D3DMATERIALCOLORSOURCE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DMCS_COLOR2: D3DMATERIALCOLORSOURCE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DMCS_FORCE_DWORD: D3DMATERIALCOLORSOURCE = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DMAX30SHADERINSTRUCTIONS: u32 = 32768u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
pub struct D3DMEMORYPRESSURE {
    pub BytesEvictedFromProcess: u64,
    pub SizeOfInefficientAllocation: u64,
    pub LevelOfEfficiency: u32,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
impl ::core::marker::Copy for D3DMEMORYPRESSURE {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
impl ::core::clone::Clone for D3DMEMORYPRESSURE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(4))]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
#[cfg(target_arch = "x86")]
pub struct D3DMEMORYPRESSURE {
    pub BytesEvictedFromProcess: u64,
    pub SizeOfInefficientAllocation: u64,
    pub LevelOfEfficiency: u32,
}
#[cfg(target_arch = "x86")]
impl ::core::marker::Copy for D3DMEMORYPRESSURE {}
#[cfg(target_arch = "x86")]
impl ::core::clone::Clone for D3DMEMORYPRESSURE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DMIN30SHADERINSTRUCTIONS: u32 = 512u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DMULTISAMPLE_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DMULTISAMPLE_NONE: D3DMULTISAMPLE_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DMULTISAMPLE_NONMASKABLE: D3DMULTISAMPLE_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DMULTISAMPLE_2_SAMPLES: D3DMULTISAMPLE_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DMULTISAMPLE_3_SAMPLES: D3DMULTISAMPLE_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DMULTISAMPLE_4_SAMPLES: D3DMULTISAMPLE_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DMULTISAMPLE_5_SAMPLES: D3DMULTISAMPLE_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DMULTISAMPLE_6_SAMPLES: D3DMULTISAMPLE_TYPE = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DMULTISAMPLE_7_SAMPLES: D3DMULTISAMPLE_TYPE = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DMULTISAMPLE_8_SAMPLES: D3DMULTISAMPLE_TYPE = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DMULTISAMPLE_9_SAMPLES: D3DMULTISAMPLE_TYPE = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DMULTISAMPLE_10_SAMPLES: D3DMULTISAMPLE_TYPE = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DMULTISAMPLE_11_SAMPLES: D3DMULTISAMPLE_TYPE = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DMULTISAMPLE_12_SAMPLES: D3DMULTISAMPLE_TYPE = 12i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DMULTISAMPLE_13_SAMPLES: D3DMULTISAMPLE_TYPE = 13i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DMULTISAMPLE_14_SAMPLES: D3DMULTISAMPLE_TYPE = 14i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DMULTISAMPLE_15_SAMPLES: D3DMULTISAMPLE_TYPE = 15i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DMULTISAMPLE_16_SAMPLES: D3DMULTISAMPLE_TYPE = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DMULTISAMPLE_FORCE_DWORD: D3DMULTISAMPLE_TYPE = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DOVERLAYCAPS_FULLRANGERGB: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DOVERLAYCAPS_LIMITEDRANGERGB: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DOVERLAYCAPS_STRETCHX: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DOVERLAYCAPS_STRETCHY: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DOVERLAYCAPS_YCbCr_BT601: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DOVERLAYCAPS_YCbCr_BT601_xvYCC: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DOVERLAYCAPS_YCbCr_BT709: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DOVERLAYCAPS_YCbCr_BT709_xvYCC: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DPATCHEDGESTYLE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPATCHEDGE_DISCRETE: D3DPATCHEDGESTYLE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPATCHEDGE_CONTINUOUS: D3DPATCHEDGESTYLE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPATCHEDGE_FORCE_DWORD: D3DPATCHEDGESTYLE = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPBLENDCAPS_BLENDFACTOR: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPBLENDCAPS_INVSRCCOLOR2: i32 = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPBLENDCAPS_SRCCOLOR2: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPMISCCAPS_BLENDOP: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPMISCCAPS_CLIPPLANESCALEDPOINTS: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPMISCCAPS_CLIPTLVERTS: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPMISCCAPS_COLORWRITEENABLE: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPMISCCAPS_FOGANDSPECULARALPHA: i32 = 65536i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPMISCCAPS_FOGVERTEXCLAMPED: i32 = 1048576i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPMISCCAPS_INDEPENDENTWRITEMASKS: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPMISCCAPS_MRTINDEPENDENTBITDEPTHS: i32 = 262144i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPMISCCAPS_MRTPOSTPIXELSHADERBLENDING: i32 = 524288i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPMISCCAPS_NULLREFERENCE: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPMISCCAPS_PERSTAGECONSTANT: i32 = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPMISCCAPS_POSTBLENDSRGBCONVERT: i32 = 2097152i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPMISCCAPS_SEPARATEALPHABLEND: i32 = 131072i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPMISCCAPS_TSSARGTEMP: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DPOOL = u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPOOL_DEFAULT: D3DPOOL = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPOOL_MANAGED: D3DPOOL = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPOOL_SYSTEMMEM: D3DPOOL = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPOOL_SCRATCH: D3DPOOL = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPOOL_FORCE_DWORD: D3DPOOL = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRASTERCAPS_COLORPERSPECTIVE: i32 = 4194304i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRASTERCAPS_DEPTHBIAS: i32 = 67108864i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRASTERCAPS_MULTISAMPLE_TOGGLE: i32 = 134217728i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRASTERCAPS_SCISSORTEST: i32 = 16777216i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRASTERCAPS_SLOPESCALEDEPTHBIAS: i32 = 33554432i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRESENTFLAG_DEVICECLIP: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRESENTFLAG_DISCARD_DEPTHSTENCIL: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRESENTFLAG_LOCKABLE_BACKBUFFER: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRESENTFLAG_NOAUTOROTATE: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRESENTFLAG_OVERLAY_LIMITEDRGB: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRESENTFLAG_OVERLAY_YCbCr_BT709: u32 = 256u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRESENTFLAG_OVERLAY_YCbCr_xvYCC: u32 = 512u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRESENTFLAG_RESTRICTED_CONTENT: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRESENTFLAG_RESTRICT_SHARED_RESOURCE_DRIVER: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRESENTFLAG_UNPRUNEDMODE: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRESENTFLAG_VIDEO: u32 = 16u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
pub struct D3DPRESENTSTATS {
    pub PresentCount: u32,
    pub PresentRefreshCount: u32,
    pub SyncRefreshCount: u32,
    pub SyncQPCTime: i64,
    pub SyncGPUTime: i64,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
impl ::core::marker::Copy for D3DPRESENTSTATS {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
impl ::core::clone::Clone for D3DPRESENTSTATS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(4))]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
#[cfg(target_arch = "x86")]
pub struct D3DPRESENTSTATS {
    pub PresentCount: u32,
    pub PresentRefreshCount: u32,
    pub SyncRefreshCount: u32,
    pub SyncQPCTime: i64,
    pub SyncGPUTime: i64,
}
#[cfg(target_arch = "x86")]
impl ::core::marker::Copy for D3DPRESENTSTATS {}
#[cfg(target_arch = "x86")]
impl ::core::clone::Clone for D3DPRESENTSTATS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRESENT_BACK_BUFFERS_MAX: i32 = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRESENT_BACK_BUFFERS_MAX_EX: i32 = 30i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRESENT_DONOTFLIP: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRESENT_DONOTWAIT: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRESENT_FLIPRESTART: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRESENT_FORCEIMMEDIATE: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRESENT_HIDEOVERLAY: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRESENT_INTERVAL_DEFAULT: i32 = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRESENT_INTERVAL_FOUR: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRESENT_INTERVAL_IMMEDIATE: i32 = -2147483648i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRESENT_INTERVAL_ONE: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRESENT_INTERVAL_THREE: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRESENT_INTERVAL_TWO: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRESENT_LINEAR_CONTENT: i32 = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DPRESENT_PARAMETERS {
    pub BackBufferWidth: u32,
    pub BackBufferHeight: u32,
    pub BackBufferFormat: D3DFORMAT,
    pub BackBufferCount: u32,
    pub MultiSampleType: D3DMULTISAMPLE_TYPE,
    pub MultiSampleQuality: u32,
    pub SwapEffect: D3DSWAPEFFECT,
    pub hDeviceWindow: super::super::Foundation::HWND,
    pub Windowed: super::super::Foundation::BOOL,
    pub EnableAutoDepthStencil: super::super::Foundation::BOOL,
    pub AutoDepthStencilFormat: D3DFORMAT,
    pub Flags: u32,
    pub FullScreen_RefreshRateInHz: u32,
    pub PresentationInterval: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DPRESENT_PARAMETERS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DPRESENT_PARAMETERS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRESENT_RATE_DEFAULT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRESENT_UPDATECOLORKEY: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRESENT_UPDATEOVERLAYONLY: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPRESENT_VIDEO_RESTRICT_TO_MONITOR: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DPRIMITIVETYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPT_POINTLIST: D3DPRIMITIVETYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPT_LINELIST: D3DPRIMITIVETYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPT_LINESTRIP: D3DPRIMITIVETYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPT_TRIANGLELIST: D3DPRIMITIVETYPE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPT_TRIANGLESTRIP: D3DPRIMITIVETYPE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPT_TRIANGLEFAN: D3DPRIMITIVETYPE = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPT_FORCE_DWORD: D3DPRIMITIVETYPE = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPS20CAPS_ARBITRARYSWIZZLE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPS20CAPS_GRADIENTINSTRUCTIONS: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPS20CAPS_NODEPENDENTREADLIMIT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPS20CAPS_NOTEXINSTRUCTIONLIMIT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPS20CAPS_PREDICATION: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPS20_MAX_DYNAMICFLOWCONTROLDEPTH: u32 = 24u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPS20_MAX_NUMINSTRUCTIONSLOTS: u32 = 512u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPS20_MAX_NUMTEMPS: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPS20_MAX_STATICFLOWCONTROLDEPTH: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPS20_MIN_DYNAMICFLOWCONTROLDEPTH: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPS20_MIN_NUMINSTRUCTIONSLOTS: u32 = 96u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPS20_MIN_NUMTEMPS: u32 = 12u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPS20_MIN_STATICFLOWCONTROLDEPTH: u32 = 0u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DPSHADERCAPS2_0 {
    pub Caps: u32,
    pub DynamicFlowControlDepth: i32,
    pub NumTemps: i32,
    pub StaticFlowControlDepth: i32,
    pub NumInstructionSlots: i32,
}
impl ::core::marker::Copy for D3DPSHADERCAPS2_0 {}
impl ::core::clone::Clone for D3DPSHADERCAPS2_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPTADDRESSCAPS_MIRRORONCE: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPTEXTURECAPS_CUBEMAP_POW2: i32 = 131072i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPTEXTURECAPS_MIPCUBEMAP: i32 = 65536i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPTEXTURECAPS_MIPMAP: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPTEXTURECAPS_MIPVOLUMEMAP: i32 = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPTEXTURECAPS_NOPROJECTEDBUMPENV: i32 = 2097152i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPTEXTURECAPS_VOLUMEMAP: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPTEXTURECAPS_VOLUMEMAP_POW2: i32 = 262144i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPTFILTERCAPS_CONVOLUTIONMONO: i32 = 262144i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPTFILTERCAPS_MAGFGAUSSIANQUAD: i32 = 268435456i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPTFILTERCAPS_MAGFPYRAMIDALQUAD: i32 = 134217728i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPTFILTERCAPS_MINFGAUSSIANQUAD: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DPTFILTERCAPS_MINFPYRAMIDALQUAD: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DQUERYTYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DQUERYTYPE_VCACHE: D3DQUERYTYPE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DQUERYTYPE_RESOURCEMANAGER: D3DQUERYTYPE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DQUERYTYPE_VERTEXSTATS: D3DQUERYTYPE = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DQUERYTYPE_EVENT: D3DQUERYTYPE = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DQUERYTYPE_OCCLUSION: D3DQUERYTYPE = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DQUERYTYPE_TIMESTAMP: D3DQUERYTYPE = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DQUERYTYPE_TIMESTAMPDISJOINT: D3DQUERYTYPE = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DQUERYTYPE_TIMESTAMPFREQ: D3DQUERYTYPE = 12i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DQUERYTYPE_PIPELINETIMINGS: D3DQUERYTYPE = 13i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DQUERYTYPE_INTERFACETIMINGS: D3DQUERYTYPE = 14i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DQUERYTYPE_VERTEXTIMINGS: D3DQUERYTYPE = 15i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DQUERYTYPE_PIXELTIMINGS: D3DQUERYTYPE = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DQUERYTYPE_BANDWIDTHTIMINGS: D3DQUERYTYPE = 17i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DQUERYTYPE_CACHEUTILIZATION: D3DQUERYTYPE = 18i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DQUERYTYPE_MEMORYPRESSURE: D3DQUERYTYPE = 19i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DRANGE {
    pub Offset: u32,
    pub Size: u32,
}
impl ::core::marker::Copy for D3DRANGE {}
impl ::core::clone::Clone for D3DRANGE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DRASTER_STATUS {
    pub InVBlank: super::super::Foundation::BOOL,
    pub ScanLine: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DRASTER_STATUS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DRASTER_STATUS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DRECT {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}
impl ::core::marker::Copy for D3DRECT {}
impl ::core::clone::Clone for D3DRECT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DRECTPATCH_INFO {
    pub StartVertexOffsetWidth: u32,
    pub StartVertexOffsetHeight: u32,
    pub Width: u32,
    pub Height: u32,
    pub Stride: u32,
    pub Basis: D3DBASISTYPE,
    pub Degree: D3DDEGREETYPE,
}
impl ::core::marker::Copy for D3DRECTPATCH_INFO {}
impl ::core::clone::Clone for D3DRECTPATCH_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DRENDERSTATETYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_ZENABLE: D3DRENDERSTATETYPE = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_FILLMODE: D3DRENDERSTATETYPE = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_SHADEMODE: D3DRENDERSTATETYPE = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_ZWRITEENABLE: D3DRENDERSTATETYPE = 14i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_ALPHATESTENABLE: D3DRENDERSTATETYPE = 15i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_LASTPIXEL: D3DRENDERSTATETYPE = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_SRCBLEND: D3DRENDERSTATETYPE = 19i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_DESTBLEND: D3DRENDERSTATETYPE = 20i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_CULLMODE: D3DRENDERSTATETYPE = 22i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_ZFUNC: D3DRENDERSTATETYPE = 23i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_ALPHAREF: D3DRENDERSTATETYPE = 24i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_ALPHAFUNC: D3DRENDERSTATETYPE = 25i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_DITHERENABLE: D3DRENDERSTATETYPE = 26i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_ALPHABLENDENABLE: D3DRENDERSTATETYPE = 27i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_FOGENABLE: D3DRENDERSTATETYPE = 28i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_SPECULARENABLE: D3DRENDERSTATETYPE = 29i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_FOGCOLOR: D3DRENDERSTATETYPE = 34i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_FOGTABLEMODE: D3DRENDERSTATETYPE = 35i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_FOGSTART: D3DRENDERSTATETYPE = 36i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_FOGEND: D3DRENDERSTATETYPE = 37i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_FOGDENSITY: D3DRENDERSTATETYPE = 38i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_RANGEFOGENABLE: D3DRENDERSTATETYPE = 48i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_STENCILENABLE: D3DRENDERSTATETYPE = 52i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_STENCILFAIL: D3DRENDERSTATETYPE = 53i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_STENCILZFAIL: D3DRENDERSTATETYPE = 54i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_STENCILPASS: D3DRENDERSTATETYPE = 55i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_STENCILFUNC: D3DRENDERSTATETYPE = 56i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_STENCILREF: D3DRENDERSTATETYPE = 57i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_STENCILMASK: D3DRENDERSTATETYPE = 58i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_STENCILWRITEMASK: D3DRENDERSTATETYPE = 59i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_TEXTUREFACTOR: D3DRENDERSTATETYPE = 60i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_WRAP0: D3DRENDERSTATETYPE = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_WRAP1: D3DRENDERSTATETYPE = 129i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_WRAP2: D3DRENDERSTATETYPE = 130i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_WRAP3: D3DRENDERSTATETYPE = 131i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_WRAP4: D3DRENDERSTATETYPE = 132i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_WRAP5: D3DRENDERSTATETYPE = 133i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_WRAP6: D3DRENDERSTATETYPE = 134i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_WRAP7: D3DRENDERSTATETYPE = 135i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_CLIPPING: D3DRENDERSTATETYPE = 136i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_LIGHTING: D3DRENDERSTATETYPE = 137i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_AMBIENT: D3DRENDERSTATETYPE = 139i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_FOGVERTEXMODE: D3DRENDERSTATETYPE = 140i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_COLORVERTEX: D3DRENDERSTATETYPE = 141i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_LOCALVIEWER: D3DRENDERSTATETYPE = 142i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_NORMALIZENORMALS: D3DRENDERSTATETYPE = 143i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_DIFFUSEMATERIALSOURCE: D3DRENDERSTATETYPE = 145i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_SPECULARMATERIALSOURCE: D3DRENDERSTATETYPE = 146i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_AMBIENTMATERIALSOURCE: D3DRENDERSTATETYPE = 147i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_EMISSIVEMATERIALSOURCE: D3DRENDERSTATETYPE = 148i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_VERTEXBLEND: D3DRENDERSTATETYPE = 151i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_CLIPPLANEENABLE: D3DRENDERSTATETYPE = 152i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_POINTSIZE: D3DRENDERSTATETYPE = 154i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_POINTSIZE_MIN: D3DRENDERSTATETYPE = 155i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_POINTSPRITEENABLE: D3DRENDERSTATETYPE = 156i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_POINTSCALEENABLE: D3DRENDERSTATETYPE = 157i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_POINTSCALE_A: D3DRENDERSTATETYPE = 158i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_POINTSCALE_B: D3DRENDERSTATETYPE = 159i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_POINTSCALE_C: D3DRENDERSTATETYPE = 160i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_MULTISAMPLEANTIALIAS: D3DRENDERSTATETYPE = 161i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_MULTISAMPLEMASK: D3DRENDERSTATETYPE = 162i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_PATCHEDGESTYLE: D3DRENDERSTATETYPE = 163i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_DEBUGMONITORTOKEN: D3DRENDERSTATETYPE = 165i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_POINTSIZE_MAX: D3DRENDERSTATETYPE = 166i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_INDEXEDVERTEXBLENDENABLE: D3DRENDERSTATETYPE = 167i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_COLORWRITEENABLE: D3DRENDERSTATETYPE = 168i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_TWEENFACTOR: D3DRENDERSTATETYPE = 170i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_BLENDOP: D3DRENDERSTATETYPE = 171i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_POSITIONDEGREE: D3DRENDERSTATETYPE = 172i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_NORMALDEGREE: D3DRENDERSTATETYPE = 173i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_SCISSORTESTENABLE: D3DRENDERSTATETYPE = 174i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_SLOPESCALEDEPTHBIAS: D3DRENDERSTATETYPE = 175i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_ANTIALIASEDLINEENABLE: D3DRENDERSTATETYPE = 176i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_MINTESSELLATIONLEVEL: D3DRENDERSTATETYPE = 178i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_MAXTESSELLATIONLEVEL: D3DRENDERSTATETYPE = 179i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_ADAPTIVETESS_X: D3DRENDERSTATETYPE = 180i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_ADAPTIVETESS_Y: D3DRENDERSTATETYPE = 181i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_ADAPTIVETESS_Z: D3DRENDERSTATETYPE = 182i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_ADAPTIVETESS_W: D3DRENDERSTATETYPE = 183i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_ENABLEADAPTIVETESSELLATION: D3DRENDERSTATETYPE = 184i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_TWOSIDEDSTENCILMODE: D3DRENDERSTATETYPE = 185i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_CCW_STENCILFAIL: D3DRENDERSTATETYPE = 186i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_CCW_STENCILZFAIL: D3DRENDERSTATETYPE = 187i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_CCW_STENCILPASS: D3DRENDERSTATETYPE = 188i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_CCW_STENCILFUNC: D3DRENDERSTATETYPE = 189i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_COLORWRITEENABLE1: D3DRENDERSTATETYPE = 190i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_COLORWRITEENABLE2: D3DRENDERSTATETYPE = 191i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_COLORWRITEENABLE3: D3DRENDERSTATETYPE = 192i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_BLENDFACTOR: D3DRENDERSTATETYPE = 193i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_SRGBWRITEENABLE: D3DRENDERSTATETYPE = 194i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_DEPTHBIAS: D3DRENDERSTATETYPE = 195i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_WRAP8: D3DRENDERSTATETYPE = 198i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_WRAP9: D3DRENDERSTATETYPE = 199i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_WRAP10: D3DRENDERSTATETYPE = 200i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_WRAP11: D3DRENDERSTATETYPE = 201i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_WRAP12: D3DRENDERSTATETYPE = 202i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_WRAP13: D3DRENDERSTATETYPE = 203i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_WRAP14: D3DRENDERSTATETYPE = 204i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_WRAP15: D3DRENDERSTATETYPE = 205i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_SEPARATEALPHABLENDENABLE: D3DRENDERSTATETYPE = 206i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_SRCBLENDALPHA: D3DRENDERSTATETYPE = 207i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_DESTBLENDALPHA: D3DRENDERSTATETYPE = 208i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_BLENDOPALPHA: D3DRENDERSTATETYPE = 209i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRS_FORCE_DWORD: D3DRENDERSTATETYPE = 2147483647i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3DRESOURCESTATS {
    pub bThrashing: super::super::Foundation::BOOL,
    pub ApproxBytesDownloaded: u32,
    pub NumEvicts: u32,
    pub NumVidCreates: u32,
    pub LastPri: u32,
    pub NumUsed: u32,
    pub NumUsedInVidMem: u32,
    pub WorkingSet: u32,
    pub WorkingSetBytes: u32,
    pub TotalManaged: u32,
    pub TotalBytes: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3DRESOURCESTATS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3DRESOURCESTATS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DRESOURCETYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRTYPE_SURFACE: D3DRESOURCETYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRTYPE_VOLUME: D3DRESOURCETYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRTYPE_TEXTURE: D3DRESOURCETYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRTYPE_VOLUMETEXTURE: D3DRESOURCETYPE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRTYPE_CUBETEXTURE: D3DRESOURCETYPE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRTYPE_VERTEXBUFFER: D3DRESOURCETYPE = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRTYPE_INDEXBUFFER: D3DRESOURCETYPE = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRTYPE_FORCE_DWORD: D3DRESOURCETYPE = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DRTYPECOUNT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DSAMPLERSTATETYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSAMP_ADDRESSU: D3DSAMPLERSTATETYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSAMP_ADDRESSV: D3DSAMPLERSTATETYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSAMP_ADDRESSW: D3DSAMPLERSTATETYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSAMP_BORDERCOLOR: D3DSAMPLERSTATETYPE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSAMP_MAGFILTER: D3DSAMPLERSTATETYPE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSAMP_MINFILTER: D3DSAMPLERSTATETYPE = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSAMP_MIPFILTER: D3DSAMPLERSTATETYPE = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSAMP_MIPMAPLODBIAS: D3DSAMPLERSTATETYPE = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSAMP_MAXMIPLEVEL: D3DSAMPLERSTATETYPE = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSAMP_MAXANISOTROPY: D3DSAMPLERSTATETYPE = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSAMP_SRGBTEXTURE: D3DSAMPLERSTATETYPE = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSAMP_ELEMENTINDEX: D3DSAMPLERSTATETYPE = 12i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSAMP_DMAPOFFSET: D3DSAMPLERSTATETYPE = 13i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSAMP_FORCE_DWORD: D3DSAMPLERSTATETYPE = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DSAMPLER_TEXTURE_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSTT_UNKNOWN: D3DSAMPLER_TEXTURE_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSTT_2D: D3DSAMPLER_TEXTURE_TYPE = 268435456i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSTT_CUBE: D3DSAMPLER_TEXTURE_TYPE = 402653184i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSTT_VOLUME: D3DSAMPLER_TEXTURE_TYPE = 536870912i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSTT_FORCE_DWORD: D3DSAMPLER_TEXTURE_TYPE = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DSCANLINEORDERING = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSCANLINEORDERING_UNKNOWN: D3DSCANLINEORDERING = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSCANLINEORDERING_PROGRESSIVE: D3DSCANLINEORDERING = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSCANLINEORDERING_INTERLACED: D3DSCANLINEORDERING = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSGR_CALIBRATE: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSGR_NO_CALIBRATION: i32 = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DSHADEMODE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSHADE_FLAT: D3DSHADEMODE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSHADE_GOURAUD: D3DSHADEMODE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSHADE_PHONG: D3DSHADEMODE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSHADE_FORCE_DWORD: D3DSHADEMODE = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSHADER_ADDRESSMODE_SHIFT: u32 = 13u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DSHADER_ADDRESSMODE_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSHADER_ADDRMODE_ABSOLUTE: D3DSHADER_ADDRESSMODE_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSHADER_ADDRMODE_RELATIVE: D3DSHADER_ADDRESSMODE_TYPE = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSHADER_ADDRMODE_FORCE_DWORD: D3DSHADER_ADDRESSMODE_TYPE = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DSHADER_COMPARISON = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPC_RESERVED0: D3DSHADER_COMPARISON = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPC_GT: D3DSHADER_COMPARISON = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPC_EQ: D3DSHADER_COMPARISON = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPC_GE: D3DSHADER_COMPARISON = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPC_LT: D3DSHADER_COMPARISON = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPC_NE: D3DSHADER_COMPARISON = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPC_LE: D3DSHADER_COMPARISON = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPC_RESERVED1: D3DSHADER_COMPARISON = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSHADER_COMPARISON_SHIFT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DSHADER_INSTRUCTION_OPCODE_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_NOP: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_MOV: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_ADD: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_SUB: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_MAD: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_MUL: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_RCP: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_RSQ: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_DP3: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_DP4: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_MIN: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_MAX: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_SLT: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 12i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_SGE: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 13i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_EXP: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 14i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_LOG: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 15i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_LIT: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_DST: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 17i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_LRP: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 18i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_FRC: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 19i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_M4x4: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 20i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_M4x3: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 21i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_M3x4: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 22i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_M3x3: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 23i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_M3x2: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 24i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_CALL: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 25i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_CALLNZ: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 26i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_LOOP: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 27i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_RET: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 28i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_ENDLOOP: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 29i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_LABEL: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 30i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_DCL: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 31i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_POW: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_CRS: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 33i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_SGN: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 34i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_ABS: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 35i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_NRM: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 36i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_SINCOS: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 37i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_REP: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 38i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_ENDREP: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 39i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_IF: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 40i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_IFC: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 41i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_ELSE: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 42i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_ENDIF: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 43i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_BREAK: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 44i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_BREAKC: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 45i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_MOVA: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 46i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_DEFB: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 47i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_DEFI: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 48i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_TEXCOORD: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_TEXKILL: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 65i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_TEX: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 66i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_TEXBEM: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 67i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_TEXBEML: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 68i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_TEXREG2AR: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 69i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_TEXREG2GB: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 70i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_TEXM3x2PAD: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 71i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_TEXM3x2TEX: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 72i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_TEXM3x3PAD: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 73i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_TEXM3x3TEX: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 74i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_RESERVED0: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 75i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_TEXM3x3SPEC: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 76i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_TEXM3x3VSPEC: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 77i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_EXPP: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 78i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_LOGP: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 79i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_CND: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 80i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_DEF: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 81i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_TEXREG2RGB: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 82i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_TEXDP3TEX: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 83i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_TEXM3x2DEPTH: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 84i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_TEXDP3: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 85i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_TEXM3x3: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 86i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_TEXDEPTH: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 87i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_CMP: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 88i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_BEM: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 89i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_DP2ADD: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 90i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_DSX: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 91i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_DSY: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 92i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_TEXLDD: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 93i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_SETP: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 94i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_TEXLDL: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 95i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_BREAKP: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 96i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_PHASE: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 65533i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_COMMENT: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 65534i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_END: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 65535i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSIO_FORCE_DWORD: D3DSHADER_INSTRUCTION_OPCODE_TYPE = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DSHADER_MIN_PRECISION = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DMP_DEFAULT: D3DSHADER_MIN_PRECISION = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DMP_16: D3DSHADER_MIN_PRECISION = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DMP_2_8: D3DSHADER_MIN_PRECISION = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DSHADER_MISCTYPE_OFFSETS = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSMO_POSITION: D3DSHADER_MISCTYPE_OFFSETS = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSMO_FACE: D3DSHADER_MISCTYPE_OFFSETS = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DSHADER_PARAM_REGISTER_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPR_TEMP: D3DSHADER_PARAM_REGISTER_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPR_INPUT: D3DSHADER_PARAM_REGISTER_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPR_CONST: D3DSHADER_PARAM_REGISTER_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPR_ADDR: D3DSHADER_PARAM_REGISTER_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPR_TEXTURE: D3DSHADER_PARAM_REGISTER_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPR_RASTOUT: D3DSHADER_PARAM_REGISTER_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPR_ATTROUT: D3DSHADER_PARAM_REGISTER_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPR_TEXCRDOUT: D3DSHADER_PARAM_REGISTER_TYPE = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPR_OUTPUT: D3DSHADER_PARAM_REGISTER_TYPE = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPR_CONSTINT: D3DSHADER_PARAM_REGISTER_TYPE = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPR_COLOROUT: D3DSHADER_PARAM_REGISTER_TYPE = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPR_DEPTHOUT: D3DSHADER_PARAM_REGISTER_TYPE = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPR_SAMPLER: D3DSHADER_PARAM_REGISTER_TYPE = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPR_CONST2: D3DSHADER_PARAM_REGISTER_TYPE = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPR_CONST3: D3DSHADER_PARAM_REGISTER_TYPE = 12i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPR_CONST4: D3DSHADER_PARAM_REGISTER_TYPE = 13i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPR_CONSTBOOL: D3DSHADER_PARAM_REGISTER_TYPE = 14i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPR_LOOP: D3DSHADER_PARAM_REGISTER_TYPE = 15i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPR_TEMPFLOAT16: D3DSHADER_PARAM_REGISTER_TYPE = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPR_MISCTYPE: D3DSHADER_PARAM_REGISTER_TYPE = 17i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPR_LABEL: D3DSHADER_PARAM_REGISTER_TYPE = 18i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPR_PREDICATE: D3DSHADER_PARAM_REGISTER_TYPE = 19i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPR_FORCE_DWORD: D3DSHADER_PARAM_REGISTER_TYPE = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DSHADER_PARAM_SRCMOD_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPSM_NONE: D3DSHADER_PARAM_SRCMOD_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPSM_NEG: D3DSHADER_PARAM_SRCMOD_TYPE = 16777216i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPSM_BIAS: D3DSHADER_PARAM_SRCMOD_TYPE = 33554432i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPSM_BIASNEG: D3DSHADER_PARAM_SRCMOD_TYPE = 50331648i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPSM_SIGN: D3DSHADER_PARAM_SRCMOD_TYPE = 67108864i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPSM_SIGNNEG: D3DSHADER_PARAM_SRCMOD_TYPE = 83886080i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPSM_COMP: D3DSHADER_PARAM_SRCMOD_TYPE = 100663296i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPSM_X2: D3DSHADER_PARAM_SRCMOD_TYPE = 117440512i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPSM_X2NEG: D3DSHADER_PARAM_SRCMOD_TYPE = 134217728i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPSM_DZ: D3DSHADER_PARAM_SRCMOD_TYPE = 150994944i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPSM_DW: D3DSHADER_PARAM_SRCMOD_TYPE = 167772160i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPSM_ABS: D3DSHADER_PARAM_SRCMOD_TYPE = 184549376i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPSM_ABSNEG: D3DSHADER_PARAM_SRCMOD_TYPE = 201326592i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPSM_NOT: D3DSHADER_PARAM_SRCMOD_TYPE = 218103808i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPSM_FORCE_DWORD: D3DSHADER_PARAM_SRCMOD_TYPE = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSI_COISSUE: u32 = 1073741824u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSI_COMMENTSIZE_MASK: u32 = 2147418112u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSI_COMMENTSIZE_SHIFT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSI_INSTLENGTH_MASK: u32 = 251658240u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSI_INSTLENGTH_SHIFT: u32 = 24u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSI_OPCODE_MASK: u32 = 65535u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSPD_IUNKNOWN: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSP_DCL_USAGEINDEX_MASK: u32 = 983040u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSP_DCL_USAGEINDEX_SHIFT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSP_DCL_USAGE_MASK: u32 = 15u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSP_DCL_USAGE_SHIFT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSP_DSTMOD_MASK: u32 = 15728640u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSP_DSTMOD_SHIFT: u32 = 20u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSP_DSTSHIFT_MASK: u32 = 251658240u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSP_DSTSHIFT_SHIFT: u32 = 24u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSP_MIN_PRECISION_MASK: u32 = 49152u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSP_MIN_PRECISION_SHIFT: u32 = 14u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSP_OPCODESPECIFICCONTROL_MASK: u32 = 16711680u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSP_OPCODESPECIFICCONTROL_SHIFT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSP_REGNUM_MASK: u32 = 2047u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSP_REGTYPE_MASK: u32 = 1879048192u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSP_REGTYPE_MASK2: u32 = 6144u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSP_REGTYPE_SHIFT: u32 = 28u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSP_REGTYPE_SHIFT2: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSP_SRCMOD_MASK: u32 = 251658240u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSP_SRCMOD_SHIFT: u32 = 24u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSP_SWIZZLE_MASK: u32 = 16711680u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSP_SWIZZLE_SHIFT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSP_TEXTURETYPE_MASK: u32 = 2013265920u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSP_TEXTURETYPE_SHIFT: u32 = 27u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSP_WRITEMASK_0: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSP_WRITEMASK_1: u32 = 131072u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSP_WRITEMASK_2: u32 = 262144u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSP_WRITEMASK_3: u32 = 524288u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSP_WRITEMASK_ALL: u32 = 983040u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DSTATEBLOCKTYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSBT_ALL: D3DSTATEBLOCKTYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSBT_PIXELSTATE: D3DSTATEBLOCKTYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSBT_VERTEXSTATE: D3DSTATEBLOCKTYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSBT_FORCE_DWORD: D3DSTATEBLOCKTYPE = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSTENCILCAPS_TWOSIDED: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DSTENCILOP = u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSTENCILOP_KEEP: D3DSTENCILOP = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSTENCILOP_ZERO: D3DSTENCILOP = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSTENCILOP_REPLACE: D3DSTENCILOP = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSTENCILOP_INCRSAT: D3DSTENCILOP = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSTENCILOP_DECRSAT: D3DSTENCILOP = 5u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSTENCILOP_INVERT: D3DSTENCILOP = 6u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSTENCILOP_INCR: D3DSTENCILOP = 7u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSTENCILOP_DECR: D3DSTENCILOP = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSTENCILOP_FORCE_DWORD: D3DSTENCILOP = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSTREAMSOURCE_INDEXEDDATA: u32 = 1073741824u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSTREAMSOURCE_INSTANCEDATA: u32 = 2147483648u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DSURFACE_DESC {
    pub Format: D3DFORMAT,
    pub Type: D3DRESOURCETYPE,
    pub Usage: u32,
    pub Pool: D3DPOOL,
    pub MultiSampleType: D3DMULTISAMPLE_TYPE,
    pub MultiSampleQuality: u32,
    pub Width: u32,
    pub Height: u32,
}
impl ::core::marker::Copy for D3DSURFACE_DESC {}
impl ::core::clone::Clone for D3DSURFACE_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DSWAPEFFECT = u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSWAPEFFECT_DISCARD: D3DSWAPEFFECT = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSWAPEFFECT_FLIP: D3DSWAPEFFECT = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSWAPEFFECT_COPY: D3DSWAPEFFECT = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSWAPEFFECT_OVERLAY: D3DSWAPEFFECT = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSWAPEFFECT_FLIPEX: D3DSWAPEFFECT = 5u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSWAPEFFECT_FORCE_DWORD: D3DSWAPEFFECT = 2147483647u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTA_CONSTANT: u32 = 6u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTA_TEMP: u32 = 5u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTEXOPCAPS_LERP: i32 = 33554432i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTEXOPCAPS_MULTIPLYADD: i32 = 16777216i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DTEXTUREADDRESS = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTADDRESS_WRAP: D3DTEXTUREADDRESS = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTADDRESS_MIRROR: D3DTEXTUREADDRESS = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTADDRESS_CLAMP: D3DTEXTUREADDRESS = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTADDRESS_BORDER: D3DTEXTUREADDRESS = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTADDRESS_MIRRORONCE: D3DTEXTUREADDRESS = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTADDRESS_FORCE_DWORD: D3DTEXTUREADDRESS = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DTEXTUREFILTERTYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTEXF_NONE: D3DTEXTUREFILTERTYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTEXF_POINT: D3DTEXTUREFILTERTYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTEXF_LINEAR: D3DTEXTUREFILTERTYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTEXF_ANISOTROPIC: D3DTEXTUREFILTERTYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTEXF_PYRAMIDALQUAD: D3DTEXTUREFILTERTYPE = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTEXF_GAUSSIANQUAD: D3DTEXTUREFILTERTYPE = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTEXF_CONVOLUTIONMONO: D3DTEXTUREFILTERTYPE = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTEXF_FORCE_DWORD: D3DTEXTUREFILTERTYPE = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DTEXTUREOP = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTOP_DISABLE: D3DTEXTUREOP = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTOP_SELECTARG1: D3DTEXTUREOP = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTOP_SELECTARG2: D3DTEXTUREOP = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTOP_MODULATE: D3DTEXTUREOP = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTOP_MODULATE2X: D3DTEXTUREOP = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTOP_MODULATE4X: D3DTEXTUREOP = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTOP_ADD: D3DTEXTUREOP = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTOP_ADDSIGNED: D3DTEXTUREOP = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTOP_ADDSIGNED2X: D3DTEXTUREOP = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTOP_SUBTRACT: D3DTEXTUREOP = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTOP_ADDSMOOTH: D3DTEXTUREOP = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTOP_BLENDDIFFUSEALPHA: D3DTEXTUREOP = 12i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTOP_BLENDTEXTUREALPHA: D3DTEXTUREOP = 13i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTOP_BLENDFACTORALPHA: D3DTEXTUREOP = 14i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTOP_BLENDTEXTUREALPHAPM: D3DTEXTUREOP = 15i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTOP_BLENDCURRENTALPHA: D3DTEXTUREOP = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTOP_PREMODULATE: D3DTEXTUREOP = 17i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTOP_MODULATEALPHA_ADDCOLOR: D3DTEXTUREOP = 18i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTOP_MODULATECOLOR_ADDALPHA: D3DTEXTUREOP = 19i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTOP_MODULATEINVALPHA_ADDCOLOR: D3DTEXTUREOP = 20i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTOP_MODULATEINVCOLOR_ADDALPHA: D3DTEXTUREOP = 21i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTOP_BUMPENVMAP: D3DTEXTUREOP = 22i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTOP_BUMPENVMAPLUMINANCE: D3DTEXTUREOP = 23i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTOP_DOTPRODUCT3: D3DTEXTUREOP = 24i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTOP_MULTIPLYADD: D3DTEXTUREOP = 25i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTOP_LERP: D3DTEXTUREOP = 26i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTOP_FORCE_DWORD: D3DTEXTUREOP = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DTEXTURESTAGESTATETYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTSS_COLOROP: D3DTEXTURESTAGESTATETYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTSS_COLORARG1: D3DTEXTURESTAGESTATETYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTSS_COLORARG2: D3DTEXTURESTAGESTATETYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTSS_ALPHAOP: D3DTEXTURESTAGESTATETYPE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTSS_ALPHAARG1: D3DTEXTURESTAGESTATETYPE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTSS_ALPHAARG2: D3DTEXTURESTAGESTATETYPE = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTSS_BUMPENVMAT00: D3DTEXTURESTAGESTATETYPE = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTSS_BUMPENVMAT01: D3DTEXTURESTAGESTATETYPE = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTSS_BUMPENVMAT10: D3DTEXTURESTAGESTATETYPE = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTSS_BUMPENVMAT11: D3DTEXTURESTAGESTATETYPE = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTSS_TEXCOORDINDEX: D3DTEXTURESTAGESTATETYPE = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTSS_BUMPENVLSCALE: D3DTEXTURESTAGESTATETYPE = 22i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTSS_BUMPENVLOFFSET: D3DTEXTURESTAGESTATETYPE = 23i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTSS_TEXTURETRANSFORMFLAGS: D3DTEXTURESTAGESTATETYPE = 24i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTSS_COLORARG0: D3DTEXTURESTAGESTATETYPE = 26i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTSS_ALPHAARG0: D3DTEXTURESTAGESTATETYPE = 27i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTSS_RESULTARG: D3DTEXTURESTAGESTATETYPE = 28i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTSS_CONSTANT: D3DTEXTURESTAGESTATETYPE = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTSS_FORCE_DWORD: D3DTEXTURESTAGESTATETYPE = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DTEXTURETRANSFORMFLAGS = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTTFF_DISABLE: D3DTEXTURETRANSFORMFLAGS = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTTFF_COUNT1: D3DTEXTURETRANSFORMFLAGS = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTTFF_COUNT2: D3DTEXTURETRANSFORMFLAGS = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTTFF_COUNT3: D3DTEXTURETRANSFORMFLAGS = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTTFF_COUNT4: D3DTEXTURETRANSFORMFLAGS = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTTFF_PROJECTED: D3DTEXTURETRANSFORMFLAGS = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTTFF_FORCE_DWORD: D3DTEXTURETRANSFORMFLAGS = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DTRANSFORMSTATETYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTS_VIEW: D3DTRANSFORMSTATETYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTS_PROJECTION: D3DTRANSFORMSTATETYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTS_TEXTURE0: D3DTRANSFORMSTATETYPE = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTS_TEXTURE1: D3DTRANSFORMSTATETYPE = 17i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTS_TEXTURE2: D3DTRANSFORMSTATETYPE = 18i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTS_TEXTURE3: D3DTRANSFORMSTATETYPE = 19i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTS_TEXTURE4: D3DTRANSFORMSTATETYPE = 20i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTS_TEXTURE5: D3DTRANSFORMSTATETYPE = 21i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTS_TEXTURE6: D3DTRANSFORMSTATETYPE = 22i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTS_TEXTURE7: D3DTRANSFORMSTATETYPE = 23i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTS_FORCE_DWORD: D3DTRANSFORMSTATETYPE = 2147483647i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DTRIPATCH_INFO {
    pub StartVertexOffset: u32,
    pub NumVertices: u32,
    pub Basis: D3DBASISTYPE,
    pub Degree: D3DDEGREETYPE,
}
impl ::core::marker::Copy for D3DTRIPATCH_INFO {}
impl ::core::clone::Clone for D3DTRIPATCH_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DTSS_TCI_SPHEREMAP: u32 = 262144u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DUSAGE_AUTOGENMIPMAP: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DUSAGE_DEPTHSTENCIL: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DUSAGE_DMAP: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DUSAGE_DONOTCLIP: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DUSAGE_DYNAMIC: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DUSAGE_NONSECURE: i32 = 8388608i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DUSAGE_NPATCHES: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DUSAGE_POINTS: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DUSAGE_QUERY_FILTER: i32 = 131072i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DUSAGE_QUERY_LEGACYBUMPMAP: i32 = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DUSAGE_QUERY_POSTPIXELSHADER_BLENDING: i32 = 524288i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DUSAGE_QUERY_SRGBREAD: i32 = 65536i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DUSAGE_QUERY_SRGBWRITE: i32 = 262144i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DUSAGE_QUERY_VERTEXTEXTURE: i32 = 1048576i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DUSAGE_QUERY_WRAPANDMIP: i32 = 2097152i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DUSAGE_RENDERTARGET: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DUSAGE_RESTRICTED_CONTENT: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DUSAGE_RESTRICT_SHARED_RESOURCE: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DUSAGE_RESTRICT_SHARED_RESOURCE_DRIVER: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DUSAGE_RTPATCHES: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DUSAGE_SOFTWAREPROCESSING: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DUSAGE_TEXTAPI: i32 = 268435456i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DUSAGE_WRITEONLY: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DVERTEXBLENDFLAGS = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DVBF_DISABLE: D3DVERTEXBLENDFLAGS = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DVBF_1WEIGHTS: D3DVERTEXBLENDFLAGS = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DVBF_2WEIGHTS: D3DVERTEXBLENDFLAGS = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DVBF_3WEIGHTS: D3DVERTEXBLENDFLAGS = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DVBF_TWEENING: D3DVERTEXBLENDFLAGS = 255i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DVBF_0WEIGHTS: D3DVERTEXBLENDFLAGS = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DVBF_FORCE_DWORD: D3DVERTEXBLENDFLAGS = 2147483647i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DVERTEXBUFFER_DESC {
    pub Format: D3DFORMAT,
    pub Type: D3DRESOURCETYPE,
    pub Usage: u32,
    pub Pool: D3DPOOL,
    pub Size: u32,
    pub FVF: u32,
}
impl ::core::marker::Copy for D3DVERTEXBUFFER_DESC {}
impl ::core::clone::Clone for D3DVERTEXBUFFER_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DVERTEXELEMENT9 {
    pub Stream: u16,
    pub Offset: u16,
    pub Type: u8,
    pub Method: u8,
    pub Usage: u8,
    pub UsageIndex: u8,
}
impl ::core::marker::Copy for D3DVERTEXELEMENT9 {}
impl ::core::clone::Clone for D3DVERTEXELEMENT9 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DVERTEXTEXTURESAMPLER0: u32 = 257u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DVERTEXTEXTURESAMPLER1: u32 = 258u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DVERTEXTEXTURESAMPLER2: u32 = 259u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DVERTEXTEXTURESAMPLER3: u32 = 260u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DVIEWPORT9 {
    pub X: u32,
    pub Y: u32,
    pub Width: u32,
    pub Height: u32,
    pub MinZ: f32,
    pub MaxZ: f32,
}
impl ::core::marker::Copy for D3DVIEWPORT9 {}
impl ::core::clone::Clone for D3DVIEWPORT9 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DVOLUME_DESC {
    pub Format: D3DFORMAT,
    pub Type: D3DRESOURCETYPE,
    pub Usage: u32,
    pub Pool: D3DPOOL,
    pub Width: u32,
    pub Height: u32,
    pub Depth: u32,
}
impl ::core::marker::Copy for D3DVOLUME_DESC {}
impl ::core::clone::Clone for D3DVOLUME_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DVS20CAPS_PREDICATION: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DVS20_MAX_DYNAMICFLOWCONTROLDEPTH: u32 = 24u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DVS20_MAX_NUMTEMPS: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DVS20_MAX_STATICFLOWCONTROLDEPTH: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DVS20_MIN_DYNAMICFLOWCONTROLDEPTH: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DVS20_MIN_NUMTEMPS: u32 = 12u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DVS20_MIN_STATICFLOWCONTROLDEPTH: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3DVSHADERCAPS2_0 {
    pub Caps: u32,
    pub DynamicFlowControlDepth: i32,
    pub NumTemps: i32,
    pub StaticFlowControlDepth: i32,
}
impl ::core::marker::Copy for D3DVSHADERCAPS2_0 {}
impl ::core::clone::Clone for D3DVSHADERCAPS2_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DVS_ADDRESSMODE_SHIFT: u32 = 13u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DVS_ADDRESSMODE_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DVS_ADDRMODE_ABSOLUTE: D3DVS_ADDRESSMODE_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DVS_ADDRMODE_RELATIVE: D3DVS_ADDRESSMODE_TYPE = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DVS_ADDRMODE_FORCE_DWORD: D3DVS_ADDRESSMODE_TYPE = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DVS_RASTOUT_OFFSETS = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSRO_POSITION: D3DVS_RASTOUT_OFFSETS = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSRO_FOG: D3DVS_RASTOUT_OFFSETS = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSRO_POINT_SIZE: D3DVS_RASTOUT_OFFSETS = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DSRO_FORCE_DWORD: D3DVS_RASTOUT_OFFSETS = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DVS_SWIZZLE_MASK: u32 = 16711680u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DVS_SWIZZLE_SHIFT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DVTXPCAPS_NO_TEXGEN_NONLOCALVIEWER: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DVTXPCAPS_TEXGEN_SPHEREMAP: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DVTXPCAPS_TWEENING: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DWRAP_W: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub type D3DZBUFFERTYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DZB_FALSE: D3DZBUFFERTYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DZB_TRUE: D3DZBUFFERTYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DZB_USEW: D3DZBUFFERTYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3DZB_FORCE_DWORD: D3DZBUFFERTYPE = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3D_MAX_SIMULTANEOUS_RENDERTARGETS: u32 = 4u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub struct D3D_OMAC {
    pub Omac: [u8; 16],
}
impl ::core::marker::Copy for D3D_OMAC {}
impl ::core::clone::Clone for D3D_OMAC {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3D_OMAC_SIZE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const D3D_SDK_VERSION: u32 = 32u32;
pub type IDirect3D9 = *mut ::core::ffi::c_void;
pub type IDirect3D9Ex = *mut ::core::ffi::c_void;
pub type IDirect3DBaseTexture9 = *mut ::core::ffi::c_void;
pub type IDirect3DCubeTexture9 = *mut ::core::ffi::c_void;
pub type IDirect3DDevice9 = *mut ::core::ffi::c_void;
pub type IDirect3DDevice9Ex = *mut ::core::ffi::c_void;
pub type IDirect3DIndexBuffer9 = *mut ::core::ffi::c_void;
pub type IDirect3DPixelShader9 = *mut ::core::ffi::c_void;
pub type IDirect3DQuery9 = *mut ::core::ffi::c_void;
pub type IDirect3DResource9 = *mut ::core::ffi::c_void;
pub type IDirect3DStateBlock9 = *mut ::core::ffi::c_void;
pub type IDirect3DSurface9 = *mut ::core::ffi::c_void;
pub type IDirect3DSwapChain9 = *mut ::core::ffi::c_void;
pub type IDirect3DSwapChain9Ex = *mut ::core::ffi::c_void;
pub type IDirect3DTexture9 = *mut ::core::ffi::c_void;
pub type IDirect3DVertexBuffer9 = *mut ::core::ffi::c_void;
pub type IDirect3DVertexDeclaration9 = *mut ::core::ffi::c_void;
pub type IDirect3DVertexShader9 = *mut ::core::ffi::c_void;
pub type IDirect3DVolume9 = *mut ::core::ffi::c_void;
pub type IDirect3DVolumeTexture9 = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const MAXD3DDECLLENGTH: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const MAXD3DDECLUSAGEINDEX: u32 = 15u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const MAX_DEVICE_IDENTIFIER_STRING: u32 = 512u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D9\"`*"]
pub const _FACD3D: u32 = 2166u32;

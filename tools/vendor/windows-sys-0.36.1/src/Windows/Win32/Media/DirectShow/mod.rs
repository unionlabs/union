#[cfg(feature = "Win32_Media_DirectShow_Xml")]
pub mod Xml;
#[link(name = "windows")]
extern "system" {
    #[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
    pub fn AMGetErrorTextA(hr: ::windows_sys::core::HRESULT, pbuffer: ::windows_sys::core::PSTR, maxlen: u32) -> u32;
    #[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
    pub fn AMGetErrorTextW(hr: ::windows_sys::core::HRESULT, pbuffer: ::windows_sys::core::PWSTR, maxlen: u32) -> u32;
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type ADVISE_TYPE = u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ADVISE_NONE: ADVISE_TYPE = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ADVISE_CLIPPING: ADVISE_TYPE = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ADVISE_PALETTE: ADVISE_TYPE = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ADVISE_COLORKEY: ADVISE_TYPE = 4u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ADVISE_POSITION: ADVISE_TYPE = 8u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ADVISE_DISPLAY_CHANGE: ADVISE_TYPE = 16u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct ALLOCATOR_PROPERTIES {
    pub cBuffers: i32,
    pub cbBuffer: i32,
    pub cbAlign: i32,
    pub cbPrefix: i32,
}
impl ::core::marker::Copy for ALLOCATOR_PROPERTIES {}
impl ::core::clone::Clone for ALLOCATOR_PROPERTIES {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMCONTROL_COLORINFO_PRESENT: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMCONTROL_PAD_TO_16x9: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMCONTROL_PAD_TO_4x3: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMCONTROL_USED: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AMCOPPCommand {
    pub macKDI: ::windows_sys::core::GUID,
    pub guidCommandID: ::windows_sys::core::GUID,
    pub dwSequence: u32,
    pub cbSizeData: u32,
    pub CommandData: [u8; 4056],
}
impl ::core::marker::Copy for AMCOPPCommand {}
impl ::core::clone::Clone for AMCOPPCommand {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AMCOPPSignature {
    pub Signature: [u8; 256],
}
impl ::core::marker::Copy for AMCOPPSignature {}
impl ::core::clone::Clone for AMCOPPSignature {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AMCOPPStatusInput {
    pub rApp: ::windows_sys::core::GUID,
    pub guidStatusRequestID: ::windows_sys::core::GUID,
    pub dwSequence: u32,
    pub cbSizeData: u32,
    pub StatusData: [u8; 4056],
}
impl ::core::marker::Copy for AMCOPPStatusInput {}
impl ::core::clone::Clone for AMCOPPStatusInput {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AMCOPPStatusOutput {
    pub macKDI: ::windows_sys::core::GUID,
    pub cbSizeData: u32,
    pub COPPStatus: [u8; 4076],
}
impl ::core::marker::Copy for AMCOPPStatusOutput {}
impl ::core::clone::Clone for AMCOPPStatusOutput {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMCOPYPROTECT_RestrictDuplication: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMDDS_ALL: u32 = 255u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMDDS_DCIPS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMDDS_DEFAULT: u32 = 255u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMDDS_NONE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMDDS_PS: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMDDS_RGBFLP: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMDDS_RGBOFF: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMDDS_RGBOVR: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMDDS_YUVFLP: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMDDS_YUVOFF: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMDDS_YUVOVR: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AMExtendedSeekingCapabilities = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_EXSEEK_CANSEEK: AMExtendedSeekingCapabilities = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_EXSEEK_CANSCAN: AMExtendedSeekingCapabilities = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_EXSEEK_MARKERSEEK: AMExtendedSeekingCapabilities = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_EXSEEK_SCANWITHOUTCLOCK: AMExtendedSeekingCapabilities = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_EXSEEK_NOSTANDARDREPAINT: AMExtendedSeekingCapabilities = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_EXSEEK_BUFFERING: AMExtendedSeekingCapabilities = 32i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_EXSEEK_SENDS_VIDEOFRAMEREADY: AMExtendedSeekingCapabilities = 64i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMF_AUTOMATICGAIN: f64 = -1f64;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type AMGETERRORTEXTPROCA = ::core::option::Option<unsafe extern "system" fn(param0: ::windows_sys::core::HRESULT, param1: ::windows_sys::core::PCSTR, param2: u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type AMGETERRORTEXTPROCW = ::core::option::Option<unsafe extern "system" fn(param0: ::windows_sys::core::HRESULT, param1: ::windows_sys::core::PCWSTR, param2: u32) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMINTERLACE_1FieldPerSample: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMINTERLACE_DisplayModeBobOnly: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMINTERLACE_DisplayModeBobOrWeave: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMINTERLACE_DisplayModeMask: u32 = 192u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMINTERLACE_DisplayModeWeaveOnly: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMINTERLACE_Field1First: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMINTERLACE_FieldPatBothIrregular: u32 = 48u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMINTERLACE_FieldPatBothRegular: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMINTERLACE_FieldPatField1Only: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMINTERLACE_FieldPatField2Only: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMINTERLACE_FieldPatternMask: u32 = 48u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMINTERLACE_IsInterlaced: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMINTERLACE_UNUSED: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AMMSF_MMS_INIT_FLAGS = u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMMSF_NOGRAPHTHREAD: AMMSF_MMS_INIT_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AMMSF_MS_FLAGS = u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMMSF_ADDDEFAULTRENDERER: AMMSF_MS_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMMSF_CREATEPEER: AMMSF_MS_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMMSF_STOPIFNOSAMPLES: AMMSF_MS_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMMSF_NOSTALL: AMMSF_MS_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AMMSF_RENDER_FLAGS = u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMMSF_RENDERTYPEMASK: AMMSF_RENDER_FLAGS = 3u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMMSF_RENDERTOEXISTING: AMMSF_RENDER_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMMSF_RENDERALLSTREAMS: AMMSF_RENDER_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMMSF_NORENDER: AMMSF_RENDER_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMMSF_NOCLOCK: AMMSF_RENDER_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMMSF_RUN: AMMSF_RENDER_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AMOVERLAYFX = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMOVERFX_NOFX: AMOVERLAYFX = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMOVERFX_MIRRORLEFTRIGHT: AMOVERLAYFX = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMOVERFX_MIRRORUPDOWN: AMOVERLAYFX = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMOVERFX_DEINTERLACE: AMOVERLAYFX = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AMPROPERTY_PIN = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMPROPERTY_PIN_CATEGORY: AMPROPERTY_PIN = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMPROPERTY_PIN_MEDIUM: AMPROPERTY_PIN = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AMPlayListEventFlags = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMPLAYLISTEVENT_RESUME: AMPlayListEventFlags = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMPLAYLISTEVENT_BREAK: AMPlayListEventFlags = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMPLAYLISTEVENT_NEXT: AMPlayListEventFlags = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMPLAYLISTEVENT_MASK: AMPlayListEventFlags = 15i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMPLAYLISTEVENT_REFRESH: AMPlayListEventFlags = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AMPlayListFlags = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMPLAYLIST_STARTINSCANMODE: AMPlayListFlags = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMPLAYLIST_FORCEBANNER: AMPlayListFlags = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AMPlayListItemFlags = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMPLAYLISTITEM_CANSKIP: AMPlayListItemFlags = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMPLAYLISTITEM_CANBIND: AMPlayListItemFlags = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AMTVAudioEventType = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMTVAUDIO_EVENT_CHANGED: AMTVAudioEventType = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AMTunerEventType = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMTUNER_EVENT_CHANGED: AMTunerEventType = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AMTunerModeType = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMTUNER_MODE_DEFAULT: AMTunerModeType = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMTUNER_MODE_TV: AMTunerModeType = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMTUNER_MODE_FM_RADIO: AMTunerModeType = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMTUNER_MODE_AM_RADIO: AMTunerModeType = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMTUNER_MODE_DSS: AMTunerModeType = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AMTunerSignalStrength = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMTUNER_HASNOSIGNALSTRENGTH: AMTunerSignalStrength = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMTUNER_NOSIGNAL: AMTunerSignalStrength = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMTUNER_SIGNALPRESENT: AMTunerSignalStrength = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AMTunerSubChannel = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMTUNER_SUBCHAN_NO_TUNE: AMTunerSubChannel = -2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMTUNER_SUBCHAN_DEFAULT: AMTunerSubChannel = -1i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AMVABUFFERINFO {
    pub dwTypeIndex: u32,
    pub dwBufferIndex: u32,
    pub dwDataOffset: u32,
    pub dwDataSize: u32,
}
impl ::core::marker::Copy for AMVABUFFERINFO {}
impl ::core::clone::Clone for AMVABUFFERINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AMVABeginFrameInfo {
    pub dwDestSurfaceIndex: u32,
    pub pInputData: *mut ::core::ffi::c_void,
    pub dwSizeInputData: u32,
    pub pOutputData: *mut ::core::ffi::c_void,
    pub dwSizeOutputData: u32,
}
impl ::core::marker::Copy for AMVABeginFrameInfo {}
impl ::core::clone::Clone for AMVABeginFrameInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Graphics_DirectDraw\"`*"]
#[cfg(feature = "Win32_Graphics_DirectDraw")]
pub struct AMVACompBufferInfo {
    pub dwNumCompBuffers: u32,
    pub dwWidthToCreate: u32,
    pub dwHeightToCreate: u32,
    pub dwBytesToAllocate: u32,
    pub ddCompCaps: super::super::Graphics::DirectDraw::DDSCAPS2,
    pub ddPixelFormat: super::super::Graphics::DirectDraw::DDPIXELFORMAT,
}
#[cfg(feature = "Win32_Graphics_DirectDraw")]
impl ::core::marker::Copy for AMVACompBufferInfo {}
#[cfg(feature = "Win32_Graphics_DirectDraw")]
impl ::core::clone::Clone for AMVACompBufferInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AMVAEndFrameInfo {
    pub dwSizeMiscData: u32,
    pub pMiscData: *mut ::core::ffi::c_void,
}
impl ::core::marker::Copy for AMVAEndFrameInfo {}
impl ::core::clone::Clone for AMVAEndFrameInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AMVAInternalMemInfo {
    pub dwScratchMemAlloc: u32,
}
impl ::core::marker::Copy for AMVAInternalMemInfo {}
impl ::core::clone::Clone for AMVAInternalMemInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Graphics_DirectDraw\"`*"]
#[cfg(feature = "Win32_Graphics_DirectDraw")]
pub struct AMVAUncompBufferInfo {
    pub dwMinNumSurfaces: u32,
    pub dwMaxNumSurfaces: u32,
    pub ddUncompPixelFormat: super::super::Graphics::DirectDraw::DDPIXELFORMAT,
}
#[cfg(feature = "Win32_Graphics_DirectDraw")]
impl ::core::marker::Copy for AMVAUncompBufferInfo {}
#[cfg(feature = "Win32_Graphics_DirectDraw")]
impl ::core::clone::Clone for AMVAUncompBufferInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Graphics_DirectDraw\"`*"]
#[cfg(feature = "Win32_Graphics_DirectDraw")]
pub struct AMVAUncompDataInfo {
    pub dwUncompWidth: u32,
    pub dwUncompHeight: u32,
    pub ddUncompPixelFormat: super::super::Graphics::DirectDraw::DDPIXELFORMAT,
}
#[cfg(feature = "Win32_Graphics_DirectDraw")]
impl ::core::marker::Copy for AMVAUncompDataInfo {}
#[cfg(feature = "Win32_Graphics_DirectDraw")]
impl ::core::clone::Clone for AMVAUncompDataInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMVA_QUERYRENDERSTATUSF_READ: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMVA_TYPEINDEX_OUTPUTFRAME: u32 = 4294967295u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct AMVPDATAINFO {
    pub dwSize: u32,
    pub dwMicrosecondsPerField: u32,
    pub amvpDimInfo: AMVPDIMINFO,
    pub dwPictAspectRatioX: u32,
    pub dwPictAspectRatioY: u32,
    pub bEnableDoubleClock: super::super::Foundation::BOOL,
    pub bEnableVACT: super::super::Foundation::BOOL,
    pub bDataIsInterlaced: super::super::Foundation::BOOL,
    pub lHalfLinesOdd: i32,
    pub bFieldPolarityInverted: super::super::Foundation::BOOL,
    pub dwNumLinesInVREF: u32,
    pub lHalfLinesEven: i32,
    pub dwReserved1: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for AMVPDATAINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for AMVPDATAINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct AMVPDIMINFO {
    pub dwFieldWidth: u32,
    pub dwFieldHeight: u32,
    pub dwVBIWidth: u32,
    pub dwVBIHeight: u32,
    pub rcValidRegion: super::super::Foundation::RECT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for AMVPDIMINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for AMVPDIMINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AMVPSIZE {
    pub dwWidth: u32,
    pub dwHeight: u32,
}
impl ::core::marker::Copy for AMVPSIZE {}
impl ::core::clone::Clone for AMVPSIZE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AMVP_MODE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMVP_MODE_WEAVE: AMVP_MODE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMVP_MODE_BOBINTERLEAVED: AMVP_MODE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMVP_MODE_BOBNONINTERLEAVED: AMVP_MODE = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMVP_MODE_SKIPEVEN: AMVP_MODE = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMVP_MODE_SKIPODD: AMVP_MODE = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AMVP_SELECT_FORMAT_BY = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMVP_DO_NOT_CARE: AMVP_SELECT_FORMAT_BY = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMVP_BEST_BANDWIDTH: AMVP_SELECT_FORMAT_BY = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMVP_INPUT_SAME_AS_OUTPUT: AMVP_SELECT_FORMAT_BY = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct AM_AC3_ALTERNATE_AUDIO {
    pub fStereo: super::super::Foundation::BOOL,
    pub DualMode: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for AM_AC3_ALTERNATE_AUDIO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for AM_AC3_ALTERNATE_AUDIO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_AC3_ALTERNATE_AUDIO_1: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_AC3_ALTERNATE_AUDIO_2: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_AC3_ALTERNATE_AUDIO_BOTH: u32 = 3u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AM_AC3_BIT_STREAM_MODE {
    pub BitStreamMode: i32,
}
impl ::core::marker::Copy for AM_AC3_BIT_STREAM_MODE {}
impl ::core::clone::Clone for AM_AC3_BIT_STREAM_MODE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AM_AC3_DIALOGUE_LEVEL {
    pub DialogueLevel: u32,
}
impl ::core::marker::Copy for AM_AC3_DIALOGUE_LEVEL {}
impl ::core::clone::Clone for AM_AC3_DIALOGUE_LEVEL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct AM_AC3_DOWNMIX {
    pub fDownMix: super::super::Foundation::BOOL,
    pub fDolbySurround: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for AM_AC3_DOWNMIX {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for AM_AC3_DOWNMIX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct AM_AC3_ERROR_CONCEALMENT {
    pub fRepeatPreviousBlock: super::super::Foundation::BOOL,
    pub fErrorInCurrentBlock: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for AM_AC3_ERROR_CONCEALMENT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for AM_AC3_ERROR_CONCEALMENT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct AM_AC3_ROOM_TYPE {
    pub fLargeRoom: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for AM_AC3_ROOM_TYPE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for AM_AC3_ROOM_TYPE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_AC3_SERVICE_COMMENTARY: u32 = 5u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_AC3_SERVICE_DIALOG_ONLY: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_AC3_SERVICE_EMERGENCY_FLASH: u32 = 6u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_AC3_SERVICE_HEARING_IMPAIRED: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_AC3_SERVICE_MAIN_AUDIO: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_AC3_SERVICE_NO_DIALOG: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_AC3_SERVICE_VISUALLY_IMPAIRED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_AC3_SERVICE_VOICE_OVER: u32 = 7u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_ASPECT_RATIO_MODE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_ARMODE_STRETCHED: AM_ASPECT_RATIO_MODE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_ARMODE_LETTER_BOX: AM_ASPECT_RATIO_MODE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_ARMODE_CROP: AM_ASPECT_RATIO_MODE = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_ARMODE_STRETCHED_AS_PRIMARY: AM_ASPECT_RATIO_MODE = 3i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AM_COLCON {
    pub _bitfield1: u8,
    pub _bitfield2: u8,
    pub _bitfield3: u8,
    pub _bitfield4: u8,
}
impl ::core::marker::Copy for AM_COLCON {}
impl ::core::clone::Clone for AM_COLCON {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_CONTENTPROPERTY_AUTHOR: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_CONTENTPROPERTY_COPYRIGHT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_CONTENTPROPERTY_DESCRIPTION: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_CONTENTPROPERTY_TITLE: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AM_COPY_MACROVISION {
    pub MACROVISIONLevel: u32,
}
impl ::core::marker::Copy for AM_COPY_MACROVISION {}
impl ::core::clone::Clone for AM_COPY_MACROVISION {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_COPY_MACROVISION_LEVEL = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_MACROVISION_DISABLED: AM_COPY_MACROVISION_LEVEL = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_MACROVISION_LEVEL1: AM_COPY_MACROVISION_LEVEL = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_MACROVISION_LEVEL2: AM_COPY_MACROVISION_LEVEL = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_MACROVISION_LEVEL3: AM_COPY_MACROVISION_LEVEL = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_DIGITAL_CP = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DIGITAL_CP_OFF: AM_DIGITAL_CP = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DIGITAL_CP_ON: AM_DIGITAL_CP = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DIGITAL_CP_DVD_COMPLIANT: AM_DIGITAL_CP = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_DVDCOPYSTATE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVDCOPYSTATE_INITIALIZE: AM_DVDCOPYSTATE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVDCOPYSTATE_INITIALIZE_TITLE: AM_DVDCOPYSTATE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVDCOPYSTATE_AUTHENTICATION_NOT_REQUIRED: AM_DVDCOPYSTATE = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVDCOPYSTATE_AUTHENTICATION_REQUIRED: AM_DVDCOPYSTATE = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVDCOPYSTATE_DONE: AM_DVDCOPYSTATE = 4i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AM_DVDCOPY_BUSKEY {
    pub BusKey: [u8; 5],
    pub Reserved: [u8; 1],
}
impl ::core::marker::Copy for AM_DVDCOPY_BUSKEY {}
impl ::core::clone::Clone for AM_DVDCOPY_BUSKEY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AM_DVDCOPY_CHLGKEY {
    pub ChlgKey: [u8; 10],
    pub Reserved: [u8; 2],
}
impl ::core::marker::Copy for AM_DVDCOPY_CHLGKEY {}
impl ::core::clone::Clone for AM_DVDCOPY_CHLGKEY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AM_DVDCOPY_DISCKEY {
    pub DiscKey: [u8; 2048],
}
impl ::core::marker::Copy for AM_DVDCOPY_DISCKEY {}
impl ::core::clone::Clone for AM_DVDCOPY_DISCKEY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AM_DVDCOPY_SET_COPY_STATE {
    pub DVDCopyState: u32,
}
impl ::core::marker::Copy for AM_DVDCOPY_SET_COPY_STATE {}
impl ::core::clone::Clone for AM_DVDCOPY_SET_COPY_STATE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AM_DVDCOPY_TITLEKEY {
    pub KeyFlags: u32,
    pub Reserved1: [u32; 2],
    pub TitleKey: [u8; 6],
    pub Reserved2: [u8; 2],
}
impl ::core::marker::Copy for AM_DVDCOPY_TITLEKEY {}
impl ::core::clone::Clone for AM_DVDCOPY_TITLEKEY {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVD_CGMS_COPY_ONCE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVD_CGMS_COPY_PERMITTED: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVD_CGMS_COPY_PROTECT_MASK: u32 = 24u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVD_CGMS_NO_COPY: u32 = 24u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVD_CGMS_RESERVED_MASK: u32 = 120u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVD_COPYRIGHTED: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVD_COPYRIGHT_MASK: u32 = 64u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AM_DVD_ChangeRate {
    pub StartInTime: i64,
    pub StartOutTime: i64,
    pub Rate: i32,
}
impl ::core::marker::Copy for AM_DVD_ChangeRate {}
impl ::core::clone::Clone for AM_DVD_ChangeRate {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_DVD_GRAPH_FLAGS = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVD_HWDEC_PREFER: AM_DVD_GRAPH_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVD_HWDEC_ONLY: AM_DVD_GRAPH_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVD_SWDEC_PREFER: AM_DVD_GRAPH_FLAGS = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVD_SWDEC_ONLY: AM_DVD_GRAPH_FLAGS = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVD_NOVPE: AM_DVD_GRAPH_FLAGS = 256i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVD_DO_NOT_CLEAR: AM_DVD_GRAPH_FLAGS = 512i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVD_VMR9_ONLY: AM_DVD_GRAPH_FLAGS = 2048i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVD_EVR_ONLY: AM_DVD_GRAPH_FLAGS = 4096i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVD_EVR_QOS: AM_DVD_GRAPH_FLAGS = 8192i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVD_ADAPT_GRAPH: AM_DVD_GRAPH_FLAGS = 16384i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVD_MASK: AM_DVD_GRAPH_FLAGS = 65535i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVD_NOT_COPYRIGHTED: u32 = 0u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct AM_DVD_RENDERSTATUS {
    pub hrVPEStatus: ::windows_sys::core::HRESULT,
    pub bDvdVolInvalid: super::super::Foundation::BOOL,
    pub bDvdVolUnknown: super::super::Foundation::BOOL,
    pub bNoLine21In: super::super::Foundation::BOOL,
    pub bNoLine21Out: super::super::Foundation::BOOL,
    pub iNumStreams: i32,
    pub iNumStreamsFailed: i32,
    pub dwFailedStreamsFlag: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for AM_DVD_RENDERSTATUS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for AM_DVD_RENDERSTATUS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVD_SECTOR_NOT_PROTECTED: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVD_SECTOR_PROTECTED: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVD_SECTOR_PROTECT_MASK: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_DVD_STREAM_FLAGS = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVD_STREAM_VIDEO: AM_DVD_STREAM_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVD_STREAM_AUDIO: AM_DVD_STREAM_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_DVD_STREAM_SUBPIC: AM_DVD_STREAM_FLAGS = 4i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AM_DVD_YUV {
    pub Reserved: u8,
    pub Y: u8,
    pub U: u8,
    pub V: u8,
}
impl ::core::marker::Copy for AM_DVD_YUV {}
impl ::core::clone::Clone for AM_DVD_YUV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AM_DvdKaraokeData {
    pub dwDownmix: u32,
    pub dwSpeakerAssignment: u32,
}
impl ::core::marker::Copy for AM_DvdKaraokeData {}
impl ::core::clone::Clone for AM_DvdKaraokeData {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AM_ExactRateChange {
    pub OutputZeroTime: i64,
    pub Rate: i32,
}
impl ::core::marker::Copy for AM_ExactRateChange {}
impl ::core::clone::Clone for AM_ExactRateChange {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_FILESINK_FLAGS = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_FILE_OVERWRITE: AM_FILESINK_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_FILTER_FLAGS = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_FILTER_FLAGS_REMOVABLE: AM_FILTER_FLAGS = 1i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AM_FRAMESTEP_STEP {
    pub dwFramesToStep: u32,
}
impl ::core::marker::Copy for AM_FRAMESTEP_STEP {}
impl ::core::clone::Clone for AM_FRAMESTEP_STEP {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_GBF_NODDSURFACELOCK: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_GBF_NOTASYNCPOINT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_GBF_NOWAIT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_GBF_PREVFRAMESKIPPED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_GETDECODERCAP_QUERY_EVR_SUPPORT: u32 = 7u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_GETDECODERCAP_QUERY_VMR9_SUPPORT: u32 = 6u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_GETDECODERCAP_QUERY_VMR_SUPPORT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_GRAPH_CONFIG_RECONNECT_FLAGS = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_GRAPH_CONFIG_RECONNECT_DIRECTCONNECT: AM_GRAPH_CONFIG_RECONNECT_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_GRAPH_CONFIG_RECONNECT_CACHE_REMOVED_FILTERS: AM_GRAPH_CONFIG_RECONNECT_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_GRAPH_CONFIG_RECONNECT_USE_ONLY_CACHED_FILTERS: AM_GRAPH_CONFIG_RECONNECT_FLAGS = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_LINE21_CCLEVEL = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_L21_CCLEVEL_TC2: AM_LINE21_CCLEVEL = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_LINE21_CCSERVICE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_L21_CCSERVICE_None: AM_LINE21_CCSERVICE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_L21_CCSERVICE_Caption1: AM_LINE21_CCSERVICE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_L21_CCSERVICE_Caption2: AM_LINE21_CCSERVICE = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_L21_CCSERVICE_Text1: AM_LINE21_CCSERVICE = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_L21_CCSERVICE_Text2: AM_LINE21_CCSERVICE = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_L21_CCSERVICE_XDS: AM_LINE21_CCSERVICE = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_L21_CCSERVICE_DefChannel: AM_LINE21_CCSERVICE = 10i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_L21_CCSERVICE_Invalid: AM_LINE21_CCSERVICE = 11i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_LINE21_CCSTATE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_L21_CCSTATE_Off: AM_LINE21_CCSTATE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_L21_CCSTATE_On: AM_LINE21_CCSTATE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_LINE21_CCSTYLE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_L21_CCSTYLE_None: AM_LINE21_CCSTYLE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_L21_CCSTYLE_PopOn: AM_LINE21_CCSTYLE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_L21_CCSTYLE_PaintOn: AM_LINE21_CCSTYLE = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_L21_CCSTYLE_RollUp: AM_LINE21_CCSTYLE = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_LINE21_DRAWBGMODE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_L21_DRAWBGMODE_Opaque: AM_LINE21_DRAWBGMODE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_L21_DRAWBGMODE_Transparent: AM_LINE21_DRAWBGMODE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_LOADSTATUS_CLOSED: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_LOADSTATUS_CONNECTING: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_LOADSTATUS_LOADINGDESCR: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_LOADSTATUS_LOADINGMCAST: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_LOADSTATUS_LOCATING: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_LOADSTATUS_OPEN: u32 = 6u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_LOADSTATUS_OPENING: u32 = 5u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_MEDIAEVENT_FLAGS = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_MEDIAEVENT_NONOTIFY: AM_MEDIAEVENT_FLAGS = 1i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct AM_MEDIA_TYPE {
    pub majortype: ::windows_sys::core::GUID,
    pub subtype: ::windows_sys::core::GUID,
    pub bFixedSizeSamples: super::super::Foundation::BOOL,
    pub bTemporalCompression: super::super::Foundation::BOOL,
    pub lSampleSize: u32,
    pub formattype: ::windows_sys::core::GUID,
    pub pUnk: ::windows_sys::core::IUnknown,
    pub cbFormat: u32,
    pub pbFormat: *mut u8,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for AM_MEDIA_TYPE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for AM_MEDIA_TYPE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_MPEG2Level = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_MPEG2Level_Low: AM_MPEG2Level = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_MPEG2Level_Main: AM_MPEG2Level = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_MPEG2Level_High1440: AM_MPEG2Level = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_MPEG2Level_High: AM_MPEG2Level = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_MPEG2Profile = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_MPEG2Profile_Simple: AM_MPEG2Profile = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_MPEG2Profile_Main: AM_MPEG2Profile = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_MPEG2Profile_SNRScalable: AM_MPEG2Profile = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_MPEG2Profile_SpatiallyScalable: AM_MPEG2Profile = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_MPEG2Profile_High: AM_MPEG2Profile = 5i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct AM_MPEGSTREAMTYPE {
    pub dwStreamId: u32,
    pub dwReserved: u32,
    pub mt: AM_MEDIA_TYPE,
    pub bFormat: [u8; 1],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for AM_MPEGSTREAMTYPE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for AM_MPEGSTREAMTYPE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct AM_MPEGSYSTEMTYPE {
    pub dwBitRate: u32,
    pub cStreams: u32,
    pub Streams: [AM_MPEGSTREAMTYPE; 1],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for AM_MPEGSYSTEMTYPE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for AM_MPEGSYSTEMTYPE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_MPEG_AUDIO_DUAL_LEFT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_MPEG_AUDIO_DUAL_MERGE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_MPEG_AUDIO_DUAL_RIGHT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_PROPERTY_AC3 = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PROPERTY_AC3_ERROR_CONCEALMENT: AM_PROPERTY_AC3 = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PROPERTY_AC3_ALTERNATE_AUDIO: AM_PROPERTY_AC3 = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PROPERTY_AC3_DOWNMIX: AM_PROPERTY_AC3 = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PROPERTY_AC3_BIT_STREAM_MODE: AM_PROPERTY_AC3 = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PROPERTY_AC3_DIALOGUE_LEVEL: AM_PROPERTY_AC3 = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PROPERTY_AC3_LANGUAGE_CODE: AM_PROPERTY_AC3 = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PROPERTY_AC3_ROOM_TYPE: AM_PROPERTY_AC3 = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_PROPERTY_DVDCOPYPROT = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PROPERTY_DVDCOPY_CHLG_KEY: AM_PROPERTY_DVDCOPYPROT = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PROPERTY_DVDCOPY_DVD_KEY1: AM_PROPERTY_DVDCOPYPROT = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PROPERTY_DVDCOPY_DEC_KEY2: AM_PROPERTY_DVDCOPYPROT = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PROPERTY_DVDCOPY_TITLE_KEY: AM_PROPERTY_DVDCOPYPROT = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PROPERTY_COPY_MACROVISION: AM_PROPERTY_DVDCOPYPROT = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PROPERTY_DVDCOPY_REGION: AM_PROPERTY_DVDCOPYPROT = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PROPERTY_DVDCOPY_SET_COPY_STATE: AM_PROPERTY_DVDCOPYPROT = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PROPERTY_COPY_ANALOG_COMPONENT: AM_PROPERTY_DVDCOPYPROT = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PROPERTY_COPY_DIGITAL_CP: AM_PROPERTY_DVDCOPYPROT = 9i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PROPERTY_COPY_DVD_SRM: AM_PROPERTY_DVDCOPYPROT = 10i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PROPERTY_DVDCOPY_SUPPORTS_NEW_KEYCOUNT: AM_PROPERTY_DVDCOPYPROT = 11i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PROPERTY_DVDCOPY_DISC_KEY: AM_PROPERTY_DVDCOPYPROT = 128i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_PROPERTY_DVDKARAOKE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PROPERTY_DVDKARAOKE_ENABLE: AM_PROPERTY_DVDKARAOKE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PROPERTY_DVDKARAOKE_DATA: AM_PROPERTY_DVDKARAOKE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_PROPERTY_DVDSUBPIC = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PROPERTY_DVDSUBPIC_PALETTE: AM_PROPERTY_DVDSUBPIC = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PROPERTY_DVDSUBPIC_HLI: AM_PROPERTY_DVDSUBPIC = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PROPERTY_DVDSUBPIC_COMPOSIT_ON: AM_PROPERTY_DVDSUBPIC = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_PROPERTY_DVD_RATE_CHANGE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_RATE_ChangeRate: AM_PROPERTY_DVD_RATE_CHANGE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_RATE_FullDataRateMax: AM_PROPERTY_DVD_RATE_CHANGE = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_RATE_ReverseDecode: AM_PROPERTY_DVD_RATE_CHANGE = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_RATE_DecoderPosition: AM_PROPERTY_DVD_RATE_CHANGE = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_RATE_DecoderVersion: AM_PROPERTY_DVD_RATE_CHANGE = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_PROPERTY_FRAMESTEP = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PROPERTY_FRAMESTEP_STEP: AM_PROPERTY_FRAMESTEP = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PROPERTY_FRAMESTEP_CANCEL: AM_PROPERTY_FRAMESTEP = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PROPERTY_FRAMESTEP_CANSTEP: AM_PROPERTY_FRAMESTEP = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PROPERTY_FRAMESTEP_CANSTEPMULTIPLE: AM_PROPERTY_FRAMESTEP = 4i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AM_PROPERTY_SPHLI {
    pub HLISS: u16,
    pub Reserved: u16,
    pub StartPTM: u32,
    pub EndPTM: u32,
    pub StartX: u16,
    pub StartY: u16,
    pub StopX: u16,
    pub StopY: u16,
    pub ColCon: AM_COLCON,
}
impl ::core::marker::Copy for AM_PROPERTY_SPHLI {}
impl ::core::clone::Clone for AM_PROPERTY_SPHLI {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AM_PROPERTY_SPPAL {
    pub sppal: [AM_DVD_YUV; 16],
}
impl ::core::marker::Copy for AM_PROPERTY_SPPAL {}
impl ::core::clone::Clone for AM_PROPERTY_SPPAL {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_PROPERTY_TS_RATE_CHANGE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_RATE_SimpleRateChange: AM_PROPERTY_TS_RATE_CHANGE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_RATE_ExactRateChange: AM_PROPERTY_TS_RATE_CHANGE = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_RATE_MaxFullDataRate: AM_PROPERTY_TS_RATE_CHANGE = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_RATE_Step: AM_PROPERTY_TS_RATE_CHANGE = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_RATE_UseRateVersion: AM_PROPERTY_TS_RATE_CHANGE = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_RATE_QueryFullFrameRate: AM_PROPERTY_TS_RATE_CHANGE = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_RATE_QueryLastRateSegPTS: AM_PROPERTY_TS_RATE_CHANGE = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_RATE_CorrectTS: AM_PROPERTY_TS_RATE_CHANGE = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_RATE_ReverseMaxFullDataRate: AM_PROPERTY_TS_RATE_CHANGE = 9i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_RATE_ResetOnTimeDisc: AM_PROPERTY_TS_RATE_CHANGE = 10i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_RATE_QueryMapping: AM_PROPERTY_TS_RATE_CHANGE = 11i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_QUERY_DECODER_ATSC_HD_SUPPORT: u32 = 5u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_QUERY_DECODER_ATSC_SD_SUPPORT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_QUERY_DECODER_DVD_SUPPORT: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_QUERY_DECODER_DXVA_1_SUPPORT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_QUERY_DECODER_VMR_SUPPORT: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AM_QueryRate {
    pub lMaxForwardFullFrame: i32,
    pub lMaxReverseFullFrame: i32,
}
impl ::core::marker::Copy for AM_QueryRate {}
impl ::core::clone::Clone for AM_QueryRate {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_ReverseBlockEnd: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_ReverseBlockStart: u32 = 2u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct AM_SAMPLE2_PROPERTIES {
    pub cbData: u32,
    pub dwTypeSpecificFlags: u32,
    pub dwSampleFlags: u32,
    pub lActual: i32,
    pub tStart: i64,
    pub tStop: i64,
    pub dwStreamId: u32,
    pub pMediaType: *mut AM_MEDIA_TYPE,
    pub pbBuffer: *mut u8,
    pub cbBuffer: i32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for AM_SAMPLE2_PROPERTIES {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for AM_SAMPLE2_PROPERTIES {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_SAMPLE_PROPERTY_FLAGS = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_SAMPLE_SPLICEPOINT: AM_SAMPLE_PROPERTY_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_SAMPLE_PREROLL: AM_SAMPLE_PROPERTY_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_SAMPLE_DATADISCONTINUITY: AM_SAMPLE_PROPERTY_FLAGS = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_SAMPLE_TYPECHANGED: AM_SAMPLE_PROPERTY_FLAGS = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_SAMPLE_TIMEVALID: AM_SAMPLE_PROPERTY_FLAGS = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_SAMPLE_TIMEDISCONTINUITY: AM_SAMPLE_PROPERTY_FLAGS = 64i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_SAMPLE_FLUSH_ON_PAUSE: AM_SAMPLE_PROPERTY_FLAGS = 128i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_SAMPLE_STOPVALID: AM_SAMPLE_PROPERTY_FLAGS = 256i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_SAMPLE_ENDOFSTREAM: AM_SAMPLE_PROPERTY_FLAGS = 512i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_STREAM_MEDIA: AM_SAMPLE_PROPERTY_FLAGS = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_STREAM_CONTROL: AM_SAMPLE_PROPERTY_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_SEEKING_SEEKING_CAPABILITIES = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_SEEKING_CanSeekAbsolute: AM_SEEKING_SEEKING_CAPABILITIES = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_SEEKING_CanSeekForwards: AM_SEEKING_SEEKING_CAPABILITIES = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_SEEKING_CanSeekBackwards: AM_SEEKING_SEEKING_CAPABILITIES = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_SEEKING_CanGetCurrentPos: AM_SEEKING_SEEKING_CAPABILITIES = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_SEEKING_CanGetStopPos: AM_SEEKING_SEEKING_CAPABILITIES = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_SEEKING_CanGetDuration: AM_SEEKING_SEEKING_CAPABILITIES = 32i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_SEEKING_CanPlayBackwards: AM_SEEKING_SEEKING_CAPABILITIES = 64i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_SEEKING_CanDoSegments: AM_SEEKING_SEEKING_CAPABILITIES = 128i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_SEEKING_Source: AM_SEEKING_SEEKING_CAPABILITIES = 256i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_SEEKING_SeekingFlags = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_SEEKING_NoPositioning: AM_SEEKING_SeekingFlags = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_SEEKING_AbsolutePositioning: AM_SEEKING_SeekingFlags = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_SEEKING_RelativePositioning: AM_SEEKING_SeekingFlags = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_SEEKING_IncrementalPositioning: AM_SEEKING_SeekingFlags = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_SEEKING_PositioningBitsMask: AM_SEEKING_SeekingFlags = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_SEEKING_SeekToKeyFrame: AM_SEEKING_SeekingFlags = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_SEEKING_ReturnTime: AM_SEEKING_SeekingFlags = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_SEEKING_Segment: AM_SEEKING_SeekingFlags = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_SEEKING_NoFlush: AM_SEEKING_SeekingFlags = 32i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AM_STREAM_INFO {
    pub tStart: i64,
    pub tStop: i64,
    pub dwStartCookie: u32,
    pub dwStopCookie: u32,
    pub dwFlags: u32,
}
impl ::core::marker::Copy for AM_STREAM_INFO {}
impl ::core::clone::Clone for AM_STREAM_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_STREAM_INFO_FLAGS = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_STREAM_INFO_START_DEFINED: AM_STREAM_INFO_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_STREAM_INFO_STOP_DEFINED: AM_STREAM_INFO_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_STREAM_INFO_DISCARDING: AM_STREAM_INFO_FLAGS = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_STREAM_INFO_STOP_SEND_EXTRA: AM_STREAM_INFO_FLAGS = 16i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AM_SimpleRateChange {
    pub StartTime: i64,
    pub Rate: i32,
}
impl ::core::marker::Copy for AM_SimpleRateChange {}
impl ::core::clone::Clone for AM_SimpleRateChange {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_UseNewCSSKey: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_VIDEO_FLAG_B_SAMPLE: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_VIDEO_FLAG_FIELD1: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_VIDEO_FLAG_FIELD1FIRST: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_VIDEO_FLAG_FIELD2: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_VIDEO_FLAG_FIELD_MASK: i32 = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_VIDEO_FLAG_INTERLEAVED_FRAME: i32 = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_VIDEO_FLAG_IPB_MASK: i32 = 48i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_VIDEO_FLAG_I_SAMPLE: i32 = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_VIDEO_FLAG_P_SAMPLE: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_VIDEO_FLAG_REPEAT_FIELD: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_VIDEO_FLAG_WEAVE: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_WST_DRAWBGMODE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_WST_DRAWBGMODE_Opaque: AM_WST_DRAWBGMODE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_WST_DRAWBGMODE_Transparent: AM_WST_DRAWBGMODE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_WST_LEVEL = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_WST_LEVEL_1_5: AM_WST_LEVEL = 0i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AM_WST_PAGE {
    pub dwPageNr: u32,
    pub dwSubPageNr: u32,
    pub pucPageData: *mut u8,
}
impl ::core::marker::Copy for AM_WST_PAGE {}
impl ::core::clone::Clone for AM_WST_PAGE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_WST_SERVICE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_WST_SERVICE_None: AM_WST_SERVICE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_WST_SERVICE_Text: AM_WST_SERVICE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_WST_SERVICE_IDS: AM_WST_SERVICE = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_WST_SERVICE_Invalid: AM_WST_SERVICE = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_WST_STATE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_WST_STATE_Off: AM_WST_STATE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_WST_STATE_On: AM_WST_STATE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AM_WST_STYLE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_WST_STYLE_None: AM_WST_STYLE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_WST_STYLE_Invers: AM_WST_STYLE = 1i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct ANALOGVIDEOINFO {
    pub rcSource: super::super::Foundation::RECT,
    pub rcTarget: super::super::Foundation::RECT,
    pub dwActiveWidth: u32,
    pub dwActiveHeight: u32,
    pub AvgTimePerFrame: i64,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for ANALOGVIDEOINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for ANALOGVIDEOINFO {
    fn clone(&self) -> Self {
        *self
    }
}
pub const ANALOG_AUXIN_NETWORK_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1949235303, data2: 2529, data3: 16547, data4: [130, 211, 150, 105, 186, 53, 50, 95] };
pub const ANALOG_FM_NETWORK_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1999112315, data2: 11193, data3: 20016, data4: [128, 120, 68, 148, 118, 229, 157, 187] };
pub const ANALOG_TV_NETWORK_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3089160318, data2: 57571, data3: 18319, data4: [138, 56, 78, 19, 247, 179, 223, 66] };
pub const ATSCChannelTuneRequest: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 57259238, data2: 17846, data3: 4563, data4: [182, 80, 0, 192, 79, 121, 73, 142] };
pub const ATSCComponentType: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2833052629, data2: 1920, data3: 20212, data4: [138, 131, 44, 255, 170, 203, 138, 206] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type ATSCComponentTypeFlags = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ATSCCT_AC3: ATSCComponentTypeFlags = 1i32;
pub const ATSCLocator: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2289237787, data2: 39162, data3: 19834, data4: [141, 147, 201, 241, 5, 95, 133, 187] };
pub const ATSCTuningSpace: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2732787536, data2: 27709, data3: 4563, data4: [182, 83, 0, 192, 79, 121, 73, 142] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ATSC_EIT_TID: u32 = 203u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ATSC_ETM_LOCATION_IN_PTC_FOR_EVENT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ATSC_ETM_LOCATION_IN_PTC_FOR_PSIP: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ATSC_ETM_LOCATION_NOT_PRESENT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ATSC_ETM_LOCATION_RESERVED: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ATSC_ETT_TID: u32 = 204u32;
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct ATSC_FILTER_OPTIONS {
    pub fSpecifyEtmId: super::super::Foundation::BOOL,
    pub EtmId: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for ATSC_FILTER_OPTIONS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for ATSC_FILTER_OPTIONS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ATSC_MGT_PID: u32 = 8187u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ATSC_MGT_TID: u32 = 199u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ATSC_PIT_TID: u32 = 208u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ATSC_RRT_PID: u32 = 8187u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ATSC_RRT_TID: u32 = 202u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ATSC_STT_PID: u32 = 8187u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ATSC_STT_TID: u32 = 205u32;
pub const ATSC_TERRESTRIAL_TV_NETWORK_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 229453789, data2: 24535, data3: 4563, data4: [143, 80, 0, 192, 79, 121, 113, 226] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ATSC_VCT_CABL_TID: u32 = 201u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ATSC_VCT_PID: u32 = 8187u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ATSC_VCT_TERR_TID: u32 = 200u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AUDIO_STREAM_CONFIG_CAPS {
    pub guid: ::windows_sys::core::GUID,
    pub MinimumChannels: u32,
    pub MaximumChannels: u32,
    pub ChannelsGranularity: u32,
    pub MinimumBitsPerSample: u32,
    pub MaximumBitsPerSample: u32,
    pub BitsPerSampleGranularity: u32,
    pub MinimumSampleFrequency: u32,
    pub MaximumSampleFrequency: u32,
    pub SampleFrequencyGranularity: u32,
}
impl ::core::marker::Copy for AUDIO_STREAM_CONFIG_CAPS {}
impl ::core::clone::Clone for AUDIO_STREAM_CONFIG_CAPS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(2))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AVIEXTHEADER {
    pub fcc: u32,
    pub cb: u32,
    pub dwGrandFrames: u32,
    pub dwFuture: [u32; 61],
}
impl ::core::marker::Copy for AVIEXTHEADER {}
impl ::core::clone::Clone for AVIEXTHEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(2))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AVIFIELDINDEX {
    pub fcc: u32,
    pub cb: u32,
    pub wLongsPerEntry: u16,
    pub bIndexSubType: u8,
    pub bIndexType: u8,
    pub nEntriesInUse: u32,
    pub dwChunkId: u32,
    pub qwBaseOffset: u64,
    pub dwReserved3: u32,
    pub aIndex: [AVIFIELDINDEX_0; 1],
}
impl ::core::marker::Copy for AVIFIELDINDEX {}
impl ::core::clone::Clone for AVIFIELDINDEX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(2))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AVIFIELDINDEX_0 {
    pub dwOffset: u32,
    pub dwSize: u32,
    pub dwOffsetField2: u32,
}
impl ::core::marker::Copy for AVIFIELDINDEX_0 {}
impl ::core::clone::Clone for AVIFIELDINDEX_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AVIF_COPYRIGHTED: u32 = 131072u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AVIF_HASINDEX: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AVIF_ISINTERLEAVED: u32 = 256u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AVIF_MUSTUSEINDEX: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AVIF_TRUSTCKTYPE: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AVIF_WASCAPTUREFILE: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AVIIF_COMPRESSOR: u32 = 268369920u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AVIIF_COMPUSE: i32 = 268369920i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AVIIF_FIRSTPART: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AVIIF_KEYFRAME: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AVIIF_LASTPART: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AVIIF_LIST: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AVIIF_NOTIME: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AVIIF_NO_TIME: u32 = 256u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AVIINDEXENTRY {
    pub ckid: u32,
    pub dwFlags: u32,
    pub dwChunkOffset: u32,
    pub dwChunkLength: u32,
}
impl ::core::marker::Copy for AVIINDEXENTRY {}
impl ::core::clone::Clone for AVIINDEXENTRY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(2))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AVIMAINHEADER {
    pub fcc: u32,
    pub cb: u32,
    pub dwMicroSecPerFrame: u32,
    pub dwMaxBytesPerSec: u32,
    pub dwPaddingGranularity: u32,
    pub dwFlags: u32,
    pub dwTotalFrames: u32,
    pub dwInitialFrames: u32,
    pub dwStreams: u32,
    pub dwSuggestedBufferSize: u32,
    pub dwWidth: u32,
    pub dwHeight: u32,
    pub dwReserved: [u32; 4],
}
impl ::core::marker::Copy for AVIMAINHEADER {}
impl ::core::clone::Clone for AVIMAINHEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(2))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AVIMETAINDEX {
    pub fcc: u32,
    pub cb: u32,
    pub wLongsPerEntry: u16,
    pub bIndexSubType: u8,
    pub bIndexType: u8,
    pub nEntriesInUse: u32,
    pub dwChunkId: u32,
    pub dwReserved: [u32; 3],
    pub adwIndex: [u32; 1],
}
impl ::core::marker::Copy for AVIMETAINDEX {}
impl ::core::clone::Clone for AVIMETAINDEX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(2))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AVIOLDINDEX {
    pub fcc: u32,
    pub cb: u32,
    pub aIndex: [AVIOLDINDEX_0; 1],
}
impl ::core::marker::Copy for AVIOLDINDEX {}
impl ::core::clone::Clone for AVIOLDINDEX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(2))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AVIOLDINDEX_0 {
    pub dwChunkId: u32,
    pub dwFlags: u32,
    pub dwOffset: u32,
    pub dwSize: u32,
}
impl ::core::marker::Copy for AVIOLDINDEX_0 {}
impl ::core::clone::Clone for AVIOLDINDEX_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(feature = "Win32_Graphics_Gdi")]
pub struct AVIPALCHANGE {
    pub bFirstEntry: u8,
    pub bNumEntries: u8,
    pub wFlags: u16,
    pub peNew: [super::super::Graphics::Gdi::PALETTEENTRY; 1],
}
#[cfg(feature = "Win32_Graphics_Gdi")]
impl ::core::marker::Copy for AVIPALCHANGE {}
#[cfg(feature = "Win32_Graphics_Gdi")]
impl ::core::clone::Clone for AVIPALCHANGE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AVISF_DISABLED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AVISF_VIDEO_PALCHANGES: u32 = 65536u32;
#[repr(C, packed(2))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AVISTDINDEX {
    pub fcc: u32,
    pub cb: u32,
    pub wLongsPerEntry: u16,
    pub bIndexSubType: u8,
    pub bIndexType: u8,
    pub nEntriesInUse: u32,
    pub dwChunkId: u32,
    pub qwBaseOffset: u64,
    pub dwReserved_3: u32,
    pub aIndex: [AVISTDINDEX_ENTRY; 2044],
}
impl ::core::marker::Copy for AVISTDINDEX {}
impl ::core::clone::Clone for AVISTDINDEX {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AVISTDINDEX_DELTAFRAME: u32 = 2147483648u32;
#[repr(C, packed(2))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AVISTDINDEX_ENTRY {
    pub dwOffset: u32,
    pub dwSize: u32,
}
impl ::core::marker::Copy for AVISTDINDEX_ENTRY {}
impl ::core::clone::Clone for AVISTDINDEX_ENTRY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(2))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AVISTREAMHEADER {
    pub fcc: u32,
    pub cb: u32,
    pub fccType: u32,
    pub fccHandler: u32,
    pub dwFlags: u32,
    pub wPriority: u16,
    pub wLanguage: u16,
    pub dwInitialFrames: u32,
    pub dwScale: u32,
    pub dwRate: u32,
    pub dwStart: u32,
    pub dwLength: u32,
    pub dwSuggestedBufferSize: u32,
    pub dwQuality: u32,
    pub dwSampleSize: u32,
    pub rcFrame: AVISTREAMHEADER_0,
}
impl ::core::marker::Copy for AVISTREAMHEADER {}
impl ::core::clone::Clone for AVISTREAMHEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AVISTREAMHEADER_0 {
    pub left: i16,
    pub top: i16,
    pub right: i16,
    pub bottom: i16,
}
impl ::core::marker::Copy for AVISTREAMHEADER_0 {}
impl ::core::clone::Clone for AVISTREAMHEADER_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(2))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AVISUPERINDEX {
    pub fcc: u32,
    pub cb: u32,
    pub wLongsPerEntry: u16,
    pub bIndexSubType: u8,
    pub bIndexType: u8,
    pub nEntriesInUse: u32,
    pub dwChunkId: u32,
    pub dwReserved: [u32; 3],
    pub aIndex: [AVISUPERINDEX_0; 1022],
}
impl ::core::marker::Copy for AVISUPERINDEX {}
impl ::core::clone::Clone for AVISUPERINDEX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(2))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AVISUPERINDEX_0 {
    pub qwOffset: u64,
    pub dwSize: u32,
    pub dwDuration: u32,
}
impl ::core::marker::Copy for AVISUPERINDEX_0 {}
impl ::core::clone::Clone for AVISUPERINDEX_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct AVIStreamHeader {
    pub fccType: u32,
    pub fccHandler: u32,
    pub dwFlags: u32,
    pub wPriority: u16,
    pub wLanguage: u16,
    pub dwInitialFrames: u32,
    pub dwScale: u32,
    pub dwRate: u32,
    pub dwStart: u32,
    pub dwLength: u32,
    pub dwSuggestedBufferSize: u32,
    pub dwQuality: u32,
    pub dwSampleSize: u32,
    pub rcFrame: super::super::Foundation::RECT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for AVIStreamHeader {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for AVIStreamHeader {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(2))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AVITCDLINDEX_ENTRY {
    pub dwTick: u32,
    pub time: super::TIMECODE,
    pub dwSMPTEflags: u32,
    pub dwUser: u32,
    pub szReelId: [i8; 12],
}
impl ::core::marker::Copy for AVITCDLINDEX_ENTRY {}
impl ::core::clone::Clone for AVITCDLINDEX_ENTRY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(2))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AVITIMECODEINDEX {
    pub fcc: u32,
    pub cb: u32,
    pub wLongsPerEntry: u16,
    pub bIndexSubType: u8,
    pub bIndexType: u8,
    pub nEntriesInUse: u32,
    pub dwChunkId: u32,
    pub dwReserved: [u32; 3],
    pub aIndex: [TIMECODEDATA; 1022],
}
impl ::core::marker::Copy for AVITIMECODEINDEX {}
impl ::core::clone::Clone for AVITIMECODEINDEX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(2))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct AVITIMEDINDEX_ENTRY {
    pub dwOffset: u32,
    pub dwSize: u32,
    pub dwDuration: u32,
}
impl ::core::marker::Copy for AVITIMEDINDEX_ENTRY {}
impl ::core::clone::Clone for AVITIMEDINDEX_ENTRY {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AVI_HEADERSIZE: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AVI_INDEX_IS_DATA: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AVI_INDEX_OF_CHUNKS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AVI_INDEX_OF_INDEXES: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AVI_INDEX_OF_SUB_2FIELD: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AVI_INDEX_OF_TIMED_CHUNKS: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AVI_INDEX_SUB_2FIELD: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AVI_INDEX_SUB_DEFAULT: u32 = 0u32;
pub const AnalogAudioComponentType: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 682295301, data2: 59461, data3: 20474, data4: [170, 155, 244, 102, 82, 54, 20, 28] };
pub const AnalogLocator: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1231260561, data2: 18603, data3: 18615, data4: [164, 122, 125, 14, 117, 160, 142, 222] };
pub const AnalogRadioTuningSpace: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2322025292, data2: 8035, data3: 4563, data4: [182, 76, 0, 192, 79, 121, 73, 142] };
pub const AnalogTVTuningSpace: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2322025293, data2: 8035, data3: 4563, data4: [182, 76, 0, 192, 79, 121, 73, 142] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type AnalogVideoStandard = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AnalogVideo_None: AnalogVideoStandard = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AnalogVideo_NTSC_M: AnalogVideoStandard = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AnalogVideo_NTSC_M_J: AnalogVideoStandard = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AnalogVideo_NTSC_433: AnalogVideoStandard = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AnalogVideo_PAL_B: AnalogVideoStandard = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AnalogVideo_PAL_D: AnalogVideoStandard = 32i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AnalogVideo_PAL_G: AnalogVideoStandard = 64i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AnalogVideo_PAL_H: AnalogVideoStandard = 128i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AnalogVideo_PAL_I: AnalogVideoStandard = 256i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AnalogVideo_PAL_M: AnalogVideoStandard = 512i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AnalogVideo_PAL_N: AnalogVideoStandard = 1024i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AnalogVideo_PAL_60: AnalogVideoStandard = 2048i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AnalogVideo_SECAM_B: AnalogVideoStandard = 4096i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AnalogVideo_SECAM_D: AnalogVideoStandard = 8192i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AnalogVideo_SECAM_G: AnalogVideoStandard = 16384i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AnalogVideo_SECAM_H: AnalogVideoStandard = 32768i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AnalogVideo_SECAM_K: AnalogVideoStandard = 65536i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AnalogVideo_SECAM_K1: AnalogVideoStandard = 131072i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AnalogVideo_SECAM_L: AnalogVideoStandard = 262144i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AnalogVideo_SECAM_L1: AnalogVideoStandard = 524288i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AnalogVideo_PAL_N_COMBO: AnalogVideoStandard = 1048576i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AnalogVideoMask_MCE_NTSC: AnalogVideoStandard = 1052167i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AnalogVideoMask_MCE_PAL: AnalogVideoStandard = 496i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AnalogVideoMask_MCE_SECAM: AnalogVideoStandard = 1044480i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AnalogVideo_NTSC_Mask: u32 = 7u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AnalogVideo_PAL_Mask: u32 = 1052656u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AnalogVideo_SECAM_Mask: u32 = 1044480u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type ApplicationTypeType = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SCTE28_ConditionalAccess: ApplicationTypeType = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SCTE28_POD_Host_Binding_Information: ApplicationTypeType = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SCTE28_IPService: ApplicationTypeType = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SCTE28_NetworkInterface_SCTE55_2: ApplicationTypeType = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SCTE28_NetworkInterface_SCTE55_1: ApplicationTypeType = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SCTE28_CopyProtection: ApplicationTypeType = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SCTE28_Diagnostic: ApplicationTypeType = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SCTE28_Undesignated: ApplicationTypeType = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SCTE28_Reserved: ApplicationTypeType = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AudioType_Commentary: u32 = 5u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AudioType_Dialogue: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AudioType_Emergency: u32 = 6u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AudioType_Hearing_Impaired: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AudioType_Music_And_Effects: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AudioType_Reserved: i32 = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AudioType_Standard: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AudioType_Visually_Impaired: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AudioType_Voiceover: u32 = 7u32;
pub const AuxInTuningSpace: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4185299462, data2: 31434, data3: 20025, data4: [156, 251, 151, 187, 53, 240, 231, 126] };
pub const BDANETWORKTYPE_ATSC: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1905811281, data2: 7329, data3: 4563, data4: [156, 200, 0, 192, 79, 121, 113, 224] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDANODE_DESCRIPTOR {
    pub ulBdaNodeType: u32,
    pub guidFunction: ::windows_sys::core::GUID,
    pub guidName: ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for BDANODE_DESCRIPTOR {}
impl ::core::clone::Clone for BDANODE_DESCRIPTOR {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_BUFFER {
    pub lResult: i32,
    pub ulBufferSize: u32,
    pub argbBuffer: [u8; 1],
}
impl ::core::marker::Copy for BDA_BUFFER {}
impl ::core::clone::Clone for BDA_BUFFER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_CAS_CHECK_ENTITLEMENTTOKEN {
    pub lResult: i32,
    pub ulDescrambleStatus: u32,
}
impl ::core::marker::Copy for BDA_CAS_CHECK_ENTITLEMENTTOKEN {}
impl ::core::clone::Clone for BDA_CAS_CHECK_ENTITLEMENTTOKEN {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_CAS_CLOSEMMIDATA {
    pub ulDialogNumber: u32,
}
impl ::core::marker::Copy for BDA_CAS_CLOSEMMIDATA {}
impl ::core::clone::Clone for BDA_CAS_CLOSEMMIDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_CAS_CLOSE_MMIDIALOG {
    pub lResult: i32,
    pub SessionResult: u32,
}
impl ::core::marker::Copy for BDA_CAS_CLOSE_MMIDIALOG {}
impl ::core::clone::Clone for BDA_CAS_CLOSE_MMIDIALOG {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_CAS_OPENMMIDATA {
    pub ulDialogNumber: u32,
    pub ulDialogRequest: u32,
    pub uuidDialogType: ::windows_sys::core::GUID,
    pub usDialogDataLength: u16,
    pub argbDialogData: [u8; 1],
}
impl ::core::marker::Copy for BDA_CAS_OPENMMIDATA {}
impl ::core::clone::Clone for BDA_CAS_OPENMMIDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_CAS_REQUESTTUNERDATA {
    pub ucRequestPriority: u8,
    pub ucRequestReason: u8,
    pub ucRequestConsequences: u8,
    pub ulEstimatedTime: u32,
}
impl ::core::marker::Copy for BDA_CAS_REQUESTTUNERDATA {}
impl ::core::clone::Clone for BDA_CAS_REQUESTTUNERDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_CA_MODULE_UI {
    pub ulFormat: u32,
    pub ulbcDesc: u32,
    pub ulDesc: [u32; 1],
}
impl ::core::marker::Copy for BDA_CA_MODULE_UI {}
impl ::core::clone::Clone for BDA_CA_MODULE_UI {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type BDA_CHANGE_STATE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_CHANGES_COMPLETE: BDA_CHANGE_STATE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_CHANGES_PENDING: BDA_CHANGE_STATE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type BDA_CONDITIONALACCESS_MMICLOSEREASON = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CONDITIONALACCESS_UNSPECIFIED: BDA_CONDITIONALACCESS_MMICLOSEREASON = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CONDITIONALACCESS_CLOSED_ITSELF: BDA_CONDITIONALACCESS_MMICLOSEREASON = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CONDITIONALACCESS_TUNER_REQUESTED_CLOSE: BDA_CONDITIONALACCESS_MMICLOSEREASON = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CONDITIONALACCESS_DIALOG_TIMEOUT: BDA_CONDITIONALACCESS_MMICLOSEREASON = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CONDITIONALACCESS_DIALOG_FOCUS_CHANGE: BDA_CONDITIONALACCESS_MMICLOSEREASON = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CONDITIONALACCESS_DIALOG_USER_DISMISSED: BDA_CONDITIONALACCESS_MMICLOSEREASON = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CONDITIONALACCESS_DIALOG_USER_NOT_AVAILABLE: BDA_CONDITIONALACCESS_MMICLOSEREASON = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type BDA_CONDITIONALACCESS_REQUESTTYPE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CONDITIONALACCESS_ACCESS_UNSPECIFIED: BDA_CONDITIONALACCESS_REQUESTTYPE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CONDITIONALACCESS_ACCESS_NOT_POSSIBLE: BDA_CONDITIONALACCESS_REQUESTTYPE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CONDITIONALACCESS_ACCESS_POSSIBLE: BDA_CONDITIONALACCESS_REQUESTTYPE = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CONDITIONALACCESS_ACCESS_POSSIBLE_NO_STREAMING_DISRUPTION: BDA_CONDITIONALACCESS_REQUESTTYPE = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type BDA_CONDITIONALACCESS_SESSION_RESULT = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CONDITIONALACCESS_SUCCESSFULL: BDA_CONDITIONALACCESS_SESSION_RESULT = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CONDITIONALACCESS_ENDED_NOCHANGE: BDA_CONDITIONALACCESS_SESSION_RESULT = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CONDITIONALACCESS_ABORTED: BDA_CONDITIONALACCESS_SESSION_RESULT = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type BDA_Channel = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_UNDEFINED_CHANNEL: BDA_Channel = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type BDA_Channel_Bandwidth = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_CHAN_BANDWITH_NOT_SET: BDA_Channel_Bandwidth = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_CHAN_BANDWITH_NOT_DEFINED: BDA_Channel_Bandwidth = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type BDA_Comp_Flags = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDACOMP_NOT_DEFINED: BDA_Comp_Flags = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDACOMP_EXCLUDE_TS_FROM_TR: BDA_Comp_Flags = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDACOMP_INCLUDE_LOCATOR_IN_TR: BDA_Comp_Flags = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDACOMP_INCLUDE_COMPONENTS_IN_TR: BDA_Comp_Flags = 4i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_DEBUG_DATA {
    pub lResult: i32,
    pub uuidDebugDataType: ::windows_sys::core::GUID,
    pub ulDataSize: u32,
    pub argbDebugData: [u8; 1],
}
impl ::core::marker::Copy for BDA_DEBUG_DATA {}
impl ::core::clone::Clone for BDA_DEBUG_DATA {
    fn clone(&self) -> Self {
        *self
    }
}
pub const BDA_DEBUG_DATA_AVAILABLE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1774341972, data2: 39299, data3: 18814, data4: [180, 21, 40, 43, 228, 197, 85, 251] };
pub const BDA_DEBUG_DATA_TYPE_STRING: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2819024743, data2: 56924, data3: 17164, data4: [128, 191, 162, 30, 190, 6, 199, 72] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type BDA_DISCOVERY_STATE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_DISCOVERY_UNSPECIFIED: BDA_DISCOVERY_STATE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_DISCOVERY_REQUIRED: BDA_DISCOVERY_STATE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_DISCOVERY_COMPLETE: BDA_DISCOVERY_STATE = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_DISEQC_RESPONSE {
    pub ulRequestId: u32,
    pub ulPacketLength: u32,
    pub argbPacketData: [u8; 8],
}
impl ::core::marker::Copy for BDA_DISEQC_RESPONSE {}
impl ::core::clone::Clone for BDA_DISEQC_RESPONSE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_DISEQC_SEND {
    pub ulRequestId: u32,
    pub ulPacketLength: u32,
    pub argbPacketData: [u8; 8],
}
impl ::core::marker::Copy for BDA_DISEQC_SEND {}
impl ::core::clone::Clone for BDA_DISEQC_SEND {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_DRM_DRMSTATUS {
    pub lResult: i32,
    pub DRMuuid: ::windows_sys::core::GUID,
    pub ulDrmUuidListStringSize: u32,
    pub argbDrmUuidListString: [::windows_sys::core::GUID; 1],
}
impl ::core::marker::Copy for BDA_DRM_DRMSTATUS {}
impl ::core::clone::Clone for BDA_DRM_DRMSTATUS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_DVBT2_L1_SIGNALLING_DATA {
    pub L1Pre_TYPE: u8,
    pub L1Pre_BWT_S1_S2: u8,
    pub L1Pre_REPETITION_GUARD_PAPR: u8,
    pub L1Pre_MOD_COD_FEC: u8,
    pub L1Pre_POSTSIZE_INFO_PILOT: [u8; 5],
    pub L1Pre_TX_ID_AVAIL: u8,
    pub L1Pre_CELL_ID: [u8; 2],
    pub L1Pre_NETWORK_ID: [u8; 2],
    pub L1Pre_T2SYSTEM_ID: [u8; 2],
    pub L1Pre_NUM_T2_FRAMES: u8,
    pub L1Pre_NUM_DATA_REGENFLAG_L1POSTEXT: [u8; 2],
    pub L1Pre_NUMRF_CURRENTRF_RESERVED: [u8; 2],
    pub L1Pre_CRC32: [u8; 4],
    pub L1PostData: [u8; 1],
}
impl ::core::marker::Copy for BDA_DVBT2_L1_SIGNALLING_DATA {}
impl ::core::clone::Clone for BDA_DVBT2_L1_SIGNALLING_DATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type BDA_DigitalSignalStandard = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const Bda_DigitalStandard_None: BDA_DigitalSignalStandard = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const Bda_DigitalStandard_DVB_T: BDA_DigitalSignalStandard = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const Bda_DigitalStandard_DVB_S: BDA_DigitalSignalStandard = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const Bda_DigitalStandard_DVB_C: BDA_DigitalSignalStandard = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const Bda_DigitalStandard_ATSC: BDA_DigitalSignalStandard = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const Bda_DigitalStandard_ISDB_T: BDA_DigitalSignalStandard = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const Bda_DigitalStandard_ISDB_S: BDA_DigitalSignalStandard = 32i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const Bda_DigitalStandard_ISDB_C: BDA_DigitalSignalStandard = 64i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type BDA_DrmPairingError = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_DrmPairing_Succeeded: BDA_DrmPairingError = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_DrmPairing_HardwareFailure: BDA_DrmPairingError = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_DrmPairing_NeedRevocationData: BDA_DrmPairingError = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_DrmPairing_NeedIndiv: BDA_DrmPairingError = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_DrmPairing_Other: BDA_DrmPairingError = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_DrmPairing_DrmInitFailed: BDA_DrmPairingError = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_DrmPairing_DrmNotPaired: BDA_DrmPairingError = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_DrmPairing_DrmRePairSoon: BDA_DrmPairingError = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_DrmPairing_Aborted: BDA_DrmPairingError = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_DrmPairing_NeedSDKUpdate: BDA_DrmPairingError = 9i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_ETHERNET_ADDRESS {
    pub rgbAddress: [u8; 6],
}
impl ::core::marker::Copy for BDA_ETHERNET_ADDRESS {}
impl ::core::clone::Clone for BDA_ETHERNET_ADDRESS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_ETHERNET_ADDRESS_LIST {
    pub ulcAddresses: u32,
    pub rgAddressl: [BDA_ETHERNET_ADDRESS; 1],
}
impl ::core::marker::Copy for BDA_ETHERNET_ADDRESS_LIST {}
impl ::core::clone::Clone for BDA_ETHERNET_ADDRESS_LIST {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_EVENT_DATA {
    pub lResult: i32,
    pub ulEventID: u32,
    pub uuidEventType: ::windows_sys::core::GUID,
    pub ulEventDataLength: u32,
    pub argbEventData: [u8; 1],
}
impl ::core::marker::Copy for BDA_EVENT_DATA {}
impl ::core::clone::Clone for BDA_EVENT_DATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type BDA_EVENT_ID = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_EVENT_SIGNAL_LOSS: BDA_EVENT_ID = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_EVENT_SIGNAL_LOCK: BDA_EVENT_ID = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_EVENT_DATA_START: BDA_EVENT_ID = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_EVENT_DATA_STOP: BDA_EVENT_ID = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_EVENT_CHANNEL_ACQUIRED: BDA_EVENT_ID = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_EVENT_CHANNEL_LOST: BDA_EVENT_ID = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_EVENT_CHANNEL_SOURCE_CHANGED: BDA_EVENT_ID = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_EVENT_CHANNEL_ACTIVATED: BDA_EVENT_ID = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_EVENT_CHANNEL_DEACTIVATED: BDA_EVENT_ID = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_EVENT_SUBCHANNEL_ACQUIRED: BDA_EVENT_ID = 9i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_EVENT_SUBCHANNEL_LOST: BDA_EVENT_ID = 10i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_EVENT_SUBCHANNEL_SOURCE_CHANGED: BDA_EVENT_ID = 11i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_EVENT_SUBCHANNEL_ACTIVATED: BDA_EVENT_ID = 12i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_EVENT_SUBCHANNEL_DEACTIVATED: BDA_EVENT_ID = 13i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_EVENT_ACCESS_GRANTED: BDA_EVENT_ID = 14i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_EVENT_ACCESS_DENIED: BDA_EVENT_ID = 15i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_EVENT_OFFER_EXTENDED: BDA_EVENT_ID = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_EVENT_PURCHASE_COMPLETED: BDA_EVENT_ID = 17i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_EVENT_SMART_CARD_INSERTED: BDA_EVENT_ID = 18i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_EVENT_SMART_CARD_REMOVED: BDA_EVENT_ID = 19i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_ACCESS_DENIED: ::windows_sys::core::HRESULT = -1073479671i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_BUFFER_TOO_SMALL: ::windows_sys::core::HRESULT = -1073479669i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_DISABLED: ::windows_sys::core::HRESULT = -1073479666i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_FAILURE: ::windows_sys::core::HRESULT = -1073479679i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_INVALID_CAPTURE_TOKEN: ::windows_sys::core::HRESULT = -1073463294i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_INVALID_ENTITLEMENT_TOKEN: ::windows_sys::core::HRESULT = -1073463295i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_INVALID_HANDLE: ::windows_sys::core::HRESULT = -1073479674i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_INVALID_LANGUAGE: ::windows_sys::core::HRESULT = -1073479664i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_INVALID_PURCHASE_TOKEN: ::windows_sys::core::HRESULT = -1073463292i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_INVALID_SCHEMA: ::windows_sys::core::HRESULT = -1073479675i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_INVALID_TUNE_REQUEST: ::windows_sys::core::HRESULT = -1073467388i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_INVALID_TYPE: ::windows_sys::core::HRESULT = -1073479673i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_IPNETWORK_ADDRESS_NOT_FOUND: ::windows_sys::core::HRESULT = -1073459198i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_IPNETWORK_ERROR: ::windows_sys::core::HRESULT = -1073459199i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_IPNETWORK_TIMEOUT: ::windows_sys::core::HRESULT = -1073459197i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_IPNETWORK_UNAVAILABLE: ::windows_sys::core::HRESULT = -1073459196i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_NOT_FOUND: ::windows_sys::core::HRESULT = -1073479670i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_NOT_IMPLEMENTED: ::windows_sys::core::HRESULT = -1073479678i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_NO_HANDLER: ::windows_sys::core::HRESULT = -1073479665i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_NO_MORE_DATA: ::windows_sys::core::HRESULT = -1073475582i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_NO_MORE_EVENTS: ::windows_sys::core::HRESULT = -1073475583i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_NO_SUCH_COMMAND: ::windows_sys::core::HRESULT = -1073479677i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_OUT_OF_BOUNDS: ::windows_sys::core::HRESULT = -1073479676i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_OUT_OF_MEMORY: ::windows_sys::core::HRESULT = -1073479667i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_OUT_OF_RESOURCES: ::windows_sys::core::HRESULT = -1073479668i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_READ_ONLY: ::windows_sys::core::HRESULT = -1073479672i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_TIMEOUT_ELAPSED: ::windows_sys::core::HRESULT = -1073479663i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_TUNER_CONFLICT: ::windows_sys::core::HRESULT = -1073467389i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_TUNER_INITIALIZING: ::windows_sys::core::HRESULT = -1073467391i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_TUNER_REQUIRED: ::windows_sys::core::HRESULT = -1073467390i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_TUNE_FAILED_SDV01: ::windows_sys::core::HRESULT = -1073455103i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_TUNE_FAILED_SDV02: ::windows_sys::core::HRESULT = -1073455102i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_TUNE_FAILED_SDV03: ::windows_sys::core::HRESULT = -1073455101i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_TUNE_FAILED_SDV04: ::windows_sys::core::HRESULT = -1073455100i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_TUNE_FAILED_SDV05: ::windows_sys::core::HRESULT = -1073455099i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_TUNE_FAILED_SDV06: ::windows_sys::core::HRESULT = -1073455098i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_TUNE_FAILED_SDV07: ::windows_sys::core::HRESULT = -1073455097i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_TUNE_FAILED_SDV08: ::windows_sys::core::HRESULT = -1073455096i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_TUNE_FAILED_SDVFF: ::windows_sys::core::HRESULT = -1073454849i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_WMDRM_INVALID_CERTIFICATE: ::windows_sys::core::HRESULT = -1073418238i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_WMDRM_INVALID_DATE: ::windows_sys::core::HRESULT = -1073418235i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_WMDRM_INVALID_PROXIMITY: ::windows_sys::core::HRESULT = -1073418234i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_WMDRM_INVALID_SIGNATURE: ::windows_sys::core::HRESULT = -1073418239i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_WMDRM_INVALID_VERSION: ::windows_sys::core::HRESULT = -1073418236i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_WMDRM_KEY_ID_NOT_FOUND: ::windows_sys::core::HRESULT = -1073418232i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_E_WOULD_DISRUPT_STREAMING: ::windows_sys::core::HRESULT = -1073463293i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type BDA_Frequency = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_FREQUENCY_NOT_SET: BDA_Frequency = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_FREQUENCY_NOT_DEFINED: BDA_Frequency = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type BDA_Frequency_Multiplier = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_FREQUENCY_MULTIPLIER_NOT_SET: BDA_Frequency_Multiplier = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_FREQUENCY_MULTIPLIER_NOT_DEFINED: BDA_Frequency_Multiplier = 0i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_GDDS_DATA {
    pub lResult: i32,
    pub ulDataLength: u32,
    pub ulPercentageProgress: u32,
    pub argbData: [u8; 1],
}
impl ::core::marker::Copy for BDA_GDDS_DATA {}
impl ::core::clone::Clone for BDA_GDDS_DATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_GDDS_DATATYPE {
    pub lResult: i32,
    pub uuidDataType: ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for BDA_GDDS_DATATYPE {}
impl ::core::clone::Clone for BDA_GDDS_DATATYPE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_IPv4_ADDRESS {
    pub rgbAddress: [u8; 4],
}
impl ::core::marker::Copy for BDA_IPv4_ADDRESS {}
impl ::core::clone::Clone for BDA_IPv4_ADDRESS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_IPv4_ADDRESS_LIST {
    pub ulcAddresses: u32,
    pub rgAddressl: [BDA_IPv4_ADDRESS; 1],
}
impl ::core::marker::Copy for BDA_IPv4_ADDRESS_LIST {}
impl ::core::clone::Clone for BDA_IPv4_ADDRESS_LIST {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_IPv6_ADDRESS {
    pub rgbAddress: [u8; 6],
}
impl ::core::marker::Copy for BDA_IPv6_ADDRESS {}
impl ::core::clone::Clone for BDA_IPv6_ADDRESS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_IPv6_ADDRESS_LIST {
    pub ulcAddresses: u32,
    pub rgAddressl: [BDA_IPv6_ADDRESS; 1],
}
impl ::core::marker::Copy for BDA_IPv6_ADDRESS_LIST {}
impl ::core::clone::Clone for BDA_IPv6_ADDRESS_LIST {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_ISDBCAS_EMG_REQ {
    pub bCLA: u8,
    pub bINS: u8,
    pub bP1: u8,
    pub bP2: u8,
    pub bLC: u8,
    pub bCardId: [u8; 6],
    pub bProtocol: u8,
    pub bCABroadcasterGroupId: u8,
    pub bMessageControl: u8,
    pub bMessageCode: [u8; 1],
}
impl ::core::marker::Copy for BDA_ISDBCAS_EMG_REQ {}
impl ::core::clone::Clone for BDA_ISDBCAS_EMG_REQ {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_ISDBCAS_REQUESTHEADER {
    pub bInstruction: u8,
    pub bReserved: [u8; 3],
    pub ulDataLength: u32,
    pub argbIsdbCommand: [u8; 1],
}
impl ::core::marker::Copy for BDA_ISDBCAS_REQUESTHEADER {}
impl ::core::clone::Clone for BDA_ISDBCAS_REQUESTHEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_ISDBCAS_RESPONSEDATA {
    pub lResult: i32,
    pub ulRequestID: u32,
    pub ulIsdbStatus: u32,
    pub ulIsdbDataSize: u32,
    pub argbIsdbCommandData: [u8; 1],
}
impl ::core::marker::Copy for BDA_ISDBCAS_RESPONSEDATA {}
impl ::core::clone::Clone for BDA_ISDBCAS_RESPONSEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type BDA_LockType = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const Bda_LockType_None: BDA_LockType = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const Bda_LockType_PLL: BDA_LockType = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const Bda_LockType_DecoderDemod: BDA_LockType = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const Bda_LockType_Complete: BDA_LockType = 128i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type BDA_MULTICAST_MODE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_PROMISCUOUS_MULTICAST: BDA_MULTICAST_MODE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_FILTERED_MULTICAST: BDA_MULTICAST_MODE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_NO_MULTICAST: BDA_MULTICAST_MODE = 2i32;
#[repr(C, packed(2))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_MUX_PIDLISTITEM {
    pub usPIDNumber: u16,
    pub usProgramNumber: u16,
    pub ePIDType: MUX_PID_TYPE,
}
impl ::core::marker::Copy for BDA_MUX_PIDLISTITEM {}
impl ::core::clone::Clone for BDA_MUX_PIDLISTITEM {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_PID_MAP {
    pub MediaSampleContent: MEDIA_SAMPLE_CONTENT,
    pub ulcPIDs: u32,
    pub aulPIDs: [u32; 1],
}
impl ::core::marker::Copy for BDA_PID_MAP {}
impl ::core::clone::Clone for BDA_PID_MAP {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_PID_UNMAP {
    pub ulcPIDs: u32,
    pub aulPIDs: [u32; 1],
}
impl ::core::marker::Copy for BDA_PID_UNMAP {}
impl ::core::clone::Clone for BDA_PID_UNMAP {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_PLP_ID_NOT_SET: i32 = -1i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_PROGRAM_PID_LIST {
    pub ulProgramNumber: u32,
    pub ulcPIDs: u32,
    pub ulPID: [u32; 1],
}
impl ::core::marker::Copy for BDA_PROGRAM_PID_LIST {}
impl ::core::clone::Clone for BDA_PROGRAM_PID_LIST {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_RATING_PINRESET {
    pub bPinLength: u8,
    pub argbNewPin: [u8; 1],
}
impl ::core::marker::Copy for BDA_RATING_PINRESET {}
impl ::core::clone::Clone for BDA_RATING_PINRESET {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type BDA_Range = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_RANGE_NOT_SET: BDA_Range = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_RANGE_NOT_DEFINED: BDA_Range = 0i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_SCAN_CAPABILTIES {
    pub lResult: i32,
    pub ul64AnalogStandardsSupported: u64,
}
impl ::core::marker::Copy for BDA_SCAN_CAPABILTIES {}
impl ::core::clone::Clone for BDA_SCAN_CAPABILTIES {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_SCAN_START {
    pub lResult: i32,
    pub LowerFrequency: u32,
    pub HigerFrequency: u32,
}
impl ::core::marker::Copy for BDA_SCAN_START {}
impl ::core::clone::Clone for BDA_SCAN_START {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_SCAN_STATE {
    pub lResult: i32,
    pub ulSignalLock: u32,
    pub ulSecondsLeft: u32,
    pub ulCurrentFrequency: u32,
}
impl ::core::marker::Copy for BDA_SCAN_STATE {}
impl ::core::clone::Clone for BDA_SCAN_STATE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type BDA_SIGNAL_STATE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SIGNAL_UNAVAILABLE: BDA_SIGNAL_STATE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SIGNAL_INACTIVE: BDA_SIGNAL_STATE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SIGNAL_ACTIVE: BDA_SIGNAL_STATE = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_SIGNAL_TIMEOUTS {
    pub ulCarrierTimeoutMs: u32,
    pub ulScanningTimeoutMs: u32,
    pub ulTuningTimeoutMs: u32,
}
impl ::core::marker::Copy for BDA_SIGNAL_TIMEOUTS {}
impl ::core::clone::Clone for BDA_SIGNAL_TIMEOUTS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_STRING {
    pub lResult: i32,
    pub ulStringSize: u32,
    pub argbString: [u8; 1],
}
impl ::core::marker::Copy for BDA_STRING {}
impl ::core::clone::Clone for BDA_STRING {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type BDA_SignalType = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const Bda_SignalType_Unknown: BDA_SignalType = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const Bda_SignalType_Analog: BDA_SignalType = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const Bda_SignalType_Digital: BDA_SignalType = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_TABLE_SECTION {
    pub ulPrimarySectionId: u32,
    pub ulSecondarySectionId: u32,
    pub ulcbSectionLength: u32,
    pub argbSectionData: [u32; 1],
}
impl ::core::marker::Copy for BDA_TABLE_SECTION {}
impl ::core::clone::Clone for BDA_TABLE_SECTION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_TEMPLATE_CONNECTION {
    pub FromNodeType: u32,
    pub FromNodePinType: u32,
    pub ToNodeType: u32,
    pub ToNodePinType: u32,
}
impl ::core::marker::Copy for BDA_TEMPLATE_CONNECTION {}
impl ::core::clone::Clone for BDA_TEMPLATE_CONNECTION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_TEMPLATE_PIN_JOINT {
    pub uliTemplateConnection: u32,
    pub ulcInstancesMax: u32,
}
impl ::core::marker::Copy for BDA_TEMPLATE_PIN_JOINT {}
impl ::core::clone::Clone for BDA_TEMPLATE_PIN_JOINT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_TRANSPORT_INFO {
    pub ulcbPhyiscalPacket: u32,
    pub ulcbPhyiscalFrame: u32,
    pub ulcbPhyiscalFrameAlignment: u32,
    pub AvgTimePerFrame: i64,
}
impl ::core::marker::Copy for BDA_TRANSPORT_INFO {}
impl ::core::clone::Clone for BDA_TRANSPORT_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_TS_SELECTORINFO {
    pub bTSInfolength: u8,
    pub bReserved: [u8; 2],
    pub guidNetworkType: ::windows_sys::core::GUID,
    pub bTSIDCount: u8,
    pub usTSID: [u16; 1],
}
impl ::core::marker::Copy for BDA_TS_SELECTORINFO {}
impl ::core::clone::Clone for BDA_TS_SELECTORINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_TS_SELECTORINFO_ISDBS_EXT {
    pub bTMCC: [u8; 48],
}
impl ::core::marker::Copy for BDA_TS_SELECTORINFO_ISDBS_EXT {}
impl ::core::clone::Clone for BDA_TS_SELECTORINFO_ISDBS_EXT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_TUNER_DIAGNOSTICS {
    pub lResult: i32,
    pub ulSignalLevel: u32,
    pub ulSignalLevelQuality: u32,
    pub ulSignalNoiseRatio: u32,
}
impl ::core::marker::Copy for BDA_TUNER_DIAGNOSTICS {}
impl ::core::clone::Clone for BDA_TUNER_DIAGNOSTICS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_TUNER_TUNERSTATE {
    pub lResult: i32,
    pub ulTuneLength: u32,
    pub argbTuneData: [u8; 1],
}
impl ::core::marker::Copy for BDA_TUNER_TUNERSTATE {}
impl ::core::clone::Clone for BDA_TUNER_TUNERSTATE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_USERACTIVITY_INTERVAL {
    pub lResult: i32,
    pub ulActivityInterval: u32,
}
impl ::core::marker::Copy for BDA_USERACTIVITY_INTERVAL {}
impl ::core::clone::Clone for BDA_USERACTIVITY_INTERVAL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_WMDRMTUNER_PIDPROTECTION {
    pub lResult: i32,
    pub uuidKeyID: ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for BDA_WMDRMTUNER_PIDPROTECTION {}
impl ::core::clone::Clone for BDA_WMDRMTUNER_PIDPROTECTION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_WMDRMTUNER_PURCHASEENTITLEMENT {
    pub lResult: i32,
    pub ulDescrambleStatus: u32,
    pub ulCaptureTokenLength: u32,
    pub argbCaptureTokenBuffer: [u8; 1],
}
impl ::core::marker::Copy for BDA_WMDRMTUNER_PURCHASEENTITLEMENT {}
impl ::core::clone::Clone for BDA_WMDRMTUNER_PURCHASEENTITLEMENT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_WMDRM_KEYINFOLIST {
    pub lResult: i32,
    pub ulKeyuuidBufferLen: u32,
    pub argKeyuuidBuffer: [::windows_sys::core::GUID; 1],
}
impl ::core::marker::Copy for BDA_WMDRM_KEYINFOLIST {}
impl ::core::clone::Clone for BDA_WMDRM_KEYINFOLIST {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_WMDRM_RENEWLICENSE {
    pub lResult: i32,
    pub ulDescrambleStatus: u32,
    pub ulXmrLicenseOutputLength: u32,
    pub argbXmrLicenceOutputBuffer: [u8; 1],
}
impl ::core::marker::Copy for BDA_WMDRM_RENEWLICENSE {}
impl ::core::clone::Clone for BDA_WMDRM_RENEWLICENSE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BDA_WMDRM_STATUS {
    pub lResult: i32,
    pub ulMaxCaptureTokenSize: u32,
    pub uMaxStreamingPid: u32,
    pub ulMaxLicense: u32,
    pub ulMinSecurityLevel: u32,
    pub ulRevInfoSequenceNumber: u32,
    pub ulRevInfoIssuedTime: u64,
    pub ulRevListVersion: u32,
    pub ulRevInfoTTL: u32,
    pub ulState: u32,
}
impl ::core::marker::Copy for BDA_WMDRM_STATUS {}
impl ::core::clone::Clone for BDA_WMDRM_STATUS {
    fn clone(&self) -> Self {
        *self
    }
}
pub const BSKYB_TERRESTRIAL_TV_NETWORK_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2661172934, data2: 15034, data3: 20232, data4: [173, 14, 204, 90, 200, 20, 140, 43] };
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct BadSampleInfo {
    pub hrReason: ::windows_sys::core::HRESULT,
}
impl ::core::marker::Copy for BadSampleInfo {}
impl ::core::clone::Clone for BadSampleInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type BfEnTvRat_Attributes_CAE_TV = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CAE_IsBlocked: BfEnTvRat_Attributes_CAE_TV = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CAE_ValidAttrSubmask: BfEnTvRat_Attributes_CAE_TV = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type BfEnTvRat_Attributes_CAF_TV = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CAF_IsBlocked: BfEnTvRat_Attributes_CAF_TV = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CAF_ValidAttrSubmask: BfEnTvRat_Attributes_CAF_TV = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type BfEnTvRat_Attributes_MPAA = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPAA_IsBlocked: BfEnTvRat_Attributes_MPAA = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPAA_ValidAttrSubmask: BfEnTvRat_Attributes_MPAA = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type BfEnTvRat_Attributes_US_TV = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const US_TV_IsBlocked: BfEnTvRat_Attributes_US_TV = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const US_TV_IsViolent: BfEnTvRat_Attributes_US_TV = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const US_TV_IsSexualSituation: BfEnTvRat_Attributes_US_TV = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const US_TV_IsAdultLanguage: BfEnTvRat_Attributes_US_TV = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const US_TV_IsSexuallySuggestiveDialog: BfEnTvRat_Attributes_US_TV = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const US_TV_ValidAttrSubmask: BfEnTvRat_Attributes_US_TV = 31i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type BfEnTvRat_GenericAttributes = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BfAttrNone: BfEnTvRat_GenericAttributes = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BfIsBlocked: BfEnTvRat_GenericAttributes = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BfIsAttr_1: BfEnTvRat_GenericAttributes = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BfIsAttr_2: BfEnTvRat_GenericAttributes = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BfIsAttr_3: BfEnTvRat_GenericAttributes = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BfIsAttr_4: BfEnTvRat_GenericAttributes = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BfIsAttr_5: BfEnTvRat_GenericAttributes = 32i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BfIsAttr_6: BfEnTvRat_GenericAttributes = 64i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BfIsAttr_7: BfEnTvRat_GenericAttributes = 128i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BfValidAttrSubmask: BfEnTvRat_GenericAttributes = 255i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type BinaryConvolutionCodeRate = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_BCC_RATE_NOT_SET: BinaryConvolutionCodeRate = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_BCC_RATE_NOT_DEFINED: BinaryConvolutionCodeRate = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_BCC_RATE_1_2: BinaryConvolutionCodeRate = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_BCC_RATE_2_3: BinaryConvolutionCodeRate = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_BCC_RATE_3_4: BinaryConvolutionCodeRate = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_BCC_RATE_3_5: BinaryConvolutionCodeRate = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_BCC_RATE_4_5: BinaryConvolutionCodeRate = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_BCC_RATE_5_6: BinaryConvolutionCodeRate = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_BCC_RATE_5_11: BinaryConvolutionCodeRate = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_BCC_RATE_7_8: BinaryConvolutionCodeRate = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_BCC_RATE_1_4: BinaryConvolutionCodeRate = 9i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_BCC_RATE_1_3: BinaryConvolutionCodeRate = 10i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_BCC_RATE_2_5: BinaryConvolutionCodeRate = 11i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_BCC_RATE_6_7: BinaryConvolutionCodeRate = 12i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_BCC_RATE_8_9: BinaryConvolutionCodeRate = 13i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_BCC_RATE_9_10: BinaryConvolutionCodeRate = 14i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_BCC_RATE_MAX: BinaryConvolutionCodeRate = 15i32;
pub const BroadcastEventService: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 188742546, data2: 2329, data3: 18740, data4: [157, 91, 97, 156, 113, 157, 2, 2] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct CAPTURE_STREAMTIME {
    pub StreamTime: i64,
}
impl ::core::marker::Copy for CAPTURE_STREAMTIME {}
impl ::core::clone::Clone for CAPTURE_STREAMTIME {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CDEF_BYPASS_CLASS_MANAGER: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CDEF_CLASS_DEFAULT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CDEF_DEVMON_CMGR_DEVICE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CDEF_DEVMON_DMO: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CDEF_DEVMON_FILTER: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CDEF_DEVMON_PNP_DEVICE: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CDEF_DEVMON_SELECTIVE_MASK: u32 = 240u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CDEF_MERIT_ABOVE_DO_NOT_USE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CFSTR_VFW_FILTERLIST: &str = "Video for Windows 4 Filters";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CHARS_IN_GUID: u32 = 39u32;
pub const CLSID_AMAudioData: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4064707968, data2: 44938, data3: 4560, data4: [130, 18, 0, 192, 79, 195, 44, 69] };
pub const CLSID_AMAudioStream: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2224480320, data2: 44876, data3: 4560, data4: [130, 18, 0, 192, 79, 195, 44, 69] };
pub const CLSID_AMDirectDrawStream: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1237613796, data2: 39844, data3: 4560, data4: [130, 18, 0, 192, 79, 195, 44, 69] };
pub const CLSID_AMMediaTypeStream: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3473878908, data2: 63423, data3: 4560, data4: [144, 13, 0, 192, 79, 217, 24, 157] };
pub const CLSID_AMMultiMediaStream: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1237613797, data2: 39844, data3: 4560, data4: [130, 18, 0, 192, 79, 195, 44, 69] };
pub const CLSID_CPCAFiltersCategory: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229820, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
pub const CLSID_DMOFilterCategory: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3168106860, data2: 48466, data3: 19760, data4: [171, 118, 112, 249, 117, 184, 145, 153] };
pub const CLSID_DMOWrapperFilter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2485743683, data2: 48514, data3: 19965, data4: [176, 222, 129, 119, 115, 156, 109, 32] };
pub const CLSID_DTFilterEncProperties: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229698, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
pub const CLSID_DTFilterTagProperties: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229714, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
pub const CLSID_ETFilterEncProperties: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229697, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
pub const CLSID_ETFilterTagProperties: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229713, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
pub const CLSID_Mpeg2TableFilter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1965573617, data2: 30095, data3: 19587, data4: [160, 67, 66, 112, 197, 147, 48, 142] };
pub const CLSID_PBDA_AUX_DATA_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4249183091, data2: 13091, data3: 16528, data4: [173, 202, 142, 212, 95, 85, 207, 16] };
pub const CLSID_PBDA_Encoder_DATA_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1922029244, data2: 21830, data3: 18198, data4: [177, 3, 248, 153, 245, 161, 250, 104] };
pub const CLSID_PBDA_FDC_DATA_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3889953184, data2: 8875, data3: 16455, data4: [142, 103, 239, 154, 213, 4, 231, 41] };
pub const CLSID_PBDA_GDDS_DATA_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3356233203, data2: 24658, data3: 19478, data4: [159, 86, 196, 76, 33, 247, 60, 69] };
pub const CLSID_PTFilter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2631079447, data2: 45827, data3: 20374, data4: [131, 48, 46, 177, 115, 234, 77, 198] };
pub const CLSID_XDSCodecProperties: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229699, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
pub const CLSID_XDSCodecTagProperties: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229715, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct COLORKEY {
    pub KeyType: u32,
    pub PaletteIndex: u32,
    pub LowColorValue: u32,
    pub HighColorValue: u32,
}
impl ::core::marker::Copy for COLORKEY {}
impl ::core::clone::Clone for COLORKEY {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type COLORKEY_TYPE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CK_NOCOLORKEY: COLORKEY_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CK_INDEX: COLORKEY_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CK_RGB: COLORKEY_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type COMPLETION_STATUS_FLAGS = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COMPSTAT_NOUPDATEOK: COMPLETION_STATUS_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COMPSTAT_WAIT: COMPLETION_STATUS_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COMPSTAT_ABORT: COMPLETION_STATUS_FLAGS = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COMPONENT_TAG_CAPTION_MAX: u32 = 55u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COMPONENT_TAG_CAPTION_MIN: u32 = 48u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COMPONENT_TAG_SUPERIMPOSE_MAX: u32 = 63u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COMPONENT_TAG_SUPERIMPOSE_MIN: u32 = 56u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type COPPEventBlockReason = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_Unknown: COPPEventBlockReason = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_BadDriver: COPPEventBlockReason = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_NoCardHDCPSupport: COPPEventBlockReason = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_NoMonitorHDCPSupport: COPPEventBlockReason = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_BadCertificate: COPPEventBlockReason = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_InvalidBusProtection: COPPEventBlockReason = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_AeroGlassOff: COPPEventBlockReason = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_RogueApp: COPPEventBlockReason = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ForbiddenVideo: COPPEventBlockReason = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_Activate: COPPEventBlockReason = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_DigitalAudioUnprotected: COPPEventBlockReason = 9i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type COPP_ACP_Protection_Level = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ACP_Level0: COPP_ACP_Protection_Level = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ACP_LevelMin: COPP_ACP_Protection_Level = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ACP_Level1: COPP_ACP_Protection_Level = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ACP_Level2: COPP_ACP_Protection_Level = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ACP_Level3: COPP_ACP_Protection_Level = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ACP_LevelMax: COPP_ACP_Protection_Level = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ACP_ForceDWORD: COPP_ACP_Protection_Level = 2147483647i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type COPP_BusType = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_BusType_Unknown: COPP_BusType = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_BusType_PCI: COPP_BusType = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_BusType_PCIX: COPP_BusType = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_BusType_PCIExpress: COPP_BusType = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_BusType_AGP: COPP_BusType = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_BusType_Integrated: COPP_BusType = -2147483648i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_BusType_ForceDWORD: COPP_BusType = 2147483647i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type COPP_CGMSA_Protection_Level = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_CGMSA_Disabled: COPP_CGMSA_Protection_Level = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_CGMSA_LevelMin: COPP_CGMSA_Protection_Level = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_CGMSA_CopyFreely: COPP_CGMSA_Protection_Level = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_CGMSA_CopyNoMore: COPP_CGMSA_Protection_Level = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_CGMSA_CopyOneGeneration: COPP_CGMSA_Protection_Level = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_CGMSA_CopyNever: COPP_CGMSA_Protection_Level = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_CGMSA_RedistributionControlRequired: COPP_CGMSA_Protection_Level = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_CGMSA_LevelMax: COPP_CGMSA_Protection_Level = 12i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_CGMSA_ForceDWORD: COPP_CGMSA_Protection_Level = 2147483647i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type COPP_ConnectorType = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ConnectorType_Unknown: COPP_ConnectorType = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ConnectorType_VGA: COPP_ConnectorType = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ConnectorType_SVideo: COPP_ConnectorType = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ConnectorType_CompositeVideo: COPP_ConnectorType = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ConnectorType_ComponentVideo: COPP_ConnectorType = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ConnectorType_DVI: COPP_ConnectorType = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ConnectorType_HDMI: COPP_ConnectorType = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ConnectorType_LVDS: COPP_ConnectorType = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ConnectorType_TMDS: COPP_ConnectorType = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ConnectorType_D_JPN: COPP_ConnectorType = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ConnectorType_Internal: COPP_ConnectorType = -2147483648i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ConnectorType_ForceDWORD: COPP_ConnectorType = 2147483647i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_DefaultProtectionLevel: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type COPP_HDCP_Protection_Level = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_HDCP_Level0: COPP_HDCP_Protection_Level = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_HDCP_LevelMin: COPP_HDCP_Protection_Level = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_HDCP_Level1: COPP_HDCP_Protection_Level = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_HDCP_LevelMax: COPP_HDCP_Protection_Level = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_HDCP_ForceDWORD: COPP_HDCP_Protection_Level = 2147483647i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type COPP_ImageAspectRatio_EN300294 = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_AspectRatio_EN300294_FullFormat4by3: COPP_ImageAspectRatio_EN300294 = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_AspectRatio_EN300294_Box14by9Center: COPP_ImageAspectRatio_EN300294 = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_AspectRatio_EN300294_Box14by9Top: COPP_ImageAspectRatio_EN300294 = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_AspectRatio_EN300294_Box16by9Center: COPP_ImageAspectRatio_EN300294 = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_AspectRatio_EN300294_Box16by9Top: COPP_ImageAspectRatio_EN300294 = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_AspectRatio_EN300294_BoxGT16by9Center: COPP_ImageAspectRatio_EN300294 = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_AspectRatio_EN300294_FullFormat4by3ProtectedCenter: COPP_ImageAspectRatio_EN300294 = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_AspectRatio_EN300294_FullFormat16by9Anamorphic: COPP_ImageAspectRatio_EN300294 = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_AspectRatio_ForceDWORD: COPP_ImageAspectRatio_EN300294 = 2147483647i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ImageAspectRatio_EN300294_Mask: u32 = 7u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_NoProtectionLevelAvailable: i32 = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type COPP_StatusFlags = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_StatusNormal: COPP_StatusFlags = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_LinkLost: COPP_StatusFlags = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_RenegotiationRequired: COPP_StatusFlags = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_StatusFlagsReserved: COPP_StatusFlags = -4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type COPP_StatusHDCPFlags = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_HDCPRepeater: COPP_StatusHDCPFlags = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_HDCPFlagsReserved: COPP_StatusHDCPFlags = -2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type COPP_TVProtectionStandard = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ProtectionStandard_Unknown: COPP_TVProtectionStandard = -2147483648i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ProtectionStandard_None: COPP_TVProtectionStandard = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ProtectionStandard_IEC61880_525i: COPP_TVProtectionStandard = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ProtectionStandard_IEC61880_2_525i: COPP_TVProtectionStandard = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ProtectionStandard_IEC62375_625p: COPP_TVProtectionStandard = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ProtectionStandard_EIA608B_525: COPP_TVProtectionStandard = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ProtectionStandard_EN300294_625i: COPP_TVProtectionStandard = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ProtectionStandard_CEA805A_TypeA_525p: COPP_TVProtectionStandard = 32i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ProtectionStandard_CEA805A_TypeA_750p: COPP_TVProtectionStandard = 64i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ProtectionStandard_CEA805A_TypeA_1125i: COPP_TVProtectionStandard = 128i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ProtectionStandard_CEA805A_TypeB_525p: COPP_TVProtectionStandard = 256i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ProtectionStandard_CEA805A_TypeB_750p: COPP_TVProtectionStandard = 512i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ProtectionStandard_CEA805A_TypeB_1125i: COPP_TVProtectionStandard = 1024i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ProtectionStandard_ARIBTRB15_525i: COPP_TVProtectionStandard = 2048i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ProtectionStandard_ARIBTRB15_525p: COPP_TVProtectionStandard = 4096i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ProtectionStandard_ARIBTRB15_750p: COPP_TVProtectionStandard = 8192i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ProtectionStandard_ARIBTRB15_1125i: COPP_TVProtectionStandard = 16384i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ProtectionStandard_Mask: COPP_TVProtectionStandard = -2147450881i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const COPP_ProtectionStandard_Reserved: COPP_TVProtectionStandard = 2147450880i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type CPEventBitShift = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CPEVENT_BITSHIFT_RATINGS: CPEventBitShift = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CPEVENT_BITSHIFT_COPP: CPEventBitShift = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CPEVENT_BITSHIFT_LICENSE: CPEventBitShift = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CPEVENT_BITSHIFT_ROLLBACK: CPEventBitShift = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CPEVENT_BITSHIFT_SAC: CPEventBitShift = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CPEVENT_BITSHIFT_DOWNRES: CPEventBitShift = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CPEVENT_BITSHIFT_STUBLIB: CPEventBitShift = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CPEVENT_BITSHIFT_UNTRUSTEDGRAPH: CPEventBitShift = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CPEVENT_BITSHIFT_PENDING_CERTIFICATE: CPEventBitShift = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CPEVENT_BITSHIFT_NO_PLAYREADY: CPEventBitShift = 9i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type CPEvents = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CPEVENT_NONE: CPEvents = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CPEVENT_RATINGS: CPEvents = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CPEVENT_COPP: CPEvents = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CPEVENT_LICENSE: CPEvents = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CPEVENT_ROLLBACK: CPEvents = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CPEVENT_SAC: CPEvents = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CPEVENT_DOWNRES: CPEvents = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CPEVENT_STUBLIB: CPEvents = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CPEVENT_UNTRUSTEDGRAPH: CPEvents = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CPEVENT_PROTECTWINDOWED: CPEvents = 9i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type CPRecordingStatus = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const RECORDING_STOPPED: CPRecordingStatus = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const RECORDING_STARTED: CPRecordingStatus = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type CRID_LOCATION = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CRID_LOCATION_IN_DESCRIPTOR: CRID_LOCATION = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CRID_LOCATION_IN_CIT: CRID_LOCATION = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CRID_LOCATION_DVB_RESERVED1: CRID_LOCATION = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CRID_LOCATION_DVB_RESERVED2: CRID_LOCATION = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type CROSSBAR_DEFAULT_FLAGS = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DEF_MODE_PROFILE: CROSSBAR_DEFAULT_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DEF_MODE_STREAMS: CROSSBAR_DEFAULT_FLAGS = 2i32;
pub const CXDSData: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229812, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type CameraControlFlags = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CameraControl_Flags_Auto: CameraControlFlags = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CameraControl_Flags_Manual: CameraControlFlags = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type CameraControlProperty = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CameraControl_Pan: CameraControlProperty = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CameraControl_Tilt: CameraControlProperty = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CameraControl_Roll: CameraControlProperty = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CameraControl_Zoom: CameraControlProperty = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CameraControl_Exposure: CameraControlProperty = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CameraControl_Iris: CameraControlProperty = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CameraControl_Focus: CameraControlProperty = 6i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct ChannelChangeInfo {
    pub state: ChannelChangeSpanningEvent_State,
    pub TimeStamp: u64,
}
impl ::core::marker::Copy for ChannelChangeInfo {}
impl ::core::clone::Clone for ChannelChangeInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type ChannelChangeSpanningEvent_State = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ChannelChangeSpanningEvent_Start: ChannelChangeSpanningEvent_State = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ChannelChangeSpanningEvent_End: ChannelChangeSpanningEvent_State = 2i32;
pub const ChannelIDTuneRequest: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 982788263, data2: 12708, data3: 17897, data4: [158, 251, 224, 85, 191, 123, 179, 219] };
pub const ChannelIDTuningSpace: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3431111215, data2: 13157, data3: 17983, data4: [175, 19, 129, 219, 182, 243, 165, 85] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct ChannelInfo {
    pub lFrequency: i32,
    pub Anonymous: ChannelInfo_0,
}
impl ::core::marker::Copy for ChannelInfo {}
impl ::core::clone::Clone for ChannelInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub union ChannelInfo_0 {
    pub DVB: ChannelInfo_0_2,
    pub DC: ChannelInfo_0_1,
    pub ATSC: ChannelInfo_0_0,
}
impl ::core::marker::Copy for ChannelInfo_0 {}
impl ::core::clone::Clone for ChannelInfo_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct ChannelInfo_0_0 {
    pub lProgNumber: i32,
}
impl ::core::marker::Copy for ChannelInfo_0_0 {}
impl ::core::clone::Clone for ChannelInfo_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct ChannelInfo_0_1 {
    pub lProgNumber: i32,
}
impl ::core::marker::Copy for ChannelInfo_0_1 {}
impl ::core::clone::Clone for ChannelInfo_0_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct ChannelInfo_0_2 {
    pub lONID: i32,
    pub lTSID: i32,
    pub lSID: i32,
}
impl ::core::marker::Copy for ChannelInfo_0_2 {}
impl ::core::clone::Clone for ChannelInfo_0_2 {
    fn clone(&self) -> Self {
        *self
    }
}
pub const ChannelTuneRequest: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 57259237, data2: 17846, data3: 4563, data4: [182, 80, 0, 192, 79, 121, 73, 142] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type ChannelType = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ChannelTypeNone: ChannelType = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ChannelTypeOther: ChannelType = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ChannelTypeVideo: ChannelType = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ChannelTypeAudio: ChannelType = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ChannelTypeText: ChannelType = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ChannelTypeSubtitles: ChannelType = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ChannelTypeCaptions: ChannelType = 32i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ChannelTypeSuperimpose: ChannelType = 64i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ChannelTypeData: ChannelType = 128i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct ChannelTypeInfo {
    pub channelType: ChannelType,
    pub timeStamp: u64,
}
impl ::core::marker::Copy for ChannelTypeInfo {}
impl ::core::clone::Clone for ChannelTypeInfo {
    fn clone(&self) -> Self {
        *self
    }
}
pub const Component: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1507608488, data2: 4460, data3: 4563, data4: [157, 142, 0, 192, 79, 114, 217, 128] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type ComponentCategory = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CategoryNotSet: ComponentCategory = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CategoryOther: ComponentCategory = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CategoryVideo: ComponentCategory = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CategoryAudio: ComponentCategory = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CategoryText: ComponentCategory = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CategorySubtitles: ComponentCategory = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CategoryCaptions: ComponentCategory = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CategorySuperimpose: ComponentCategory = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CategoryData: ComponentCategory = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CATEGORY_COUNT: ComponentCategory = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type ComponentStatus = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const StatusActive: ComponentStatus = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const StatusInactive: ComponentStatus = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const StatusUnavailable: ComponentStatus = 2i32;
pub const ComponentType: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2184525216, data2: 792, data3: 4563, data4: [157, 142, 0, 192, 79, 114, 217, 128] };
pub const ComponentTypes: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2711794116, data2: 3642, data3: 4563, data4: [157, 142, 0, 192, 79, 114, 217, 128] };
pub const Components: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2157667937, data2: 38084, data3: 18918, data4: [182, 236, 63, 15, 134, 34, 21, 170] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type CompressionCaps = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CompressionCaps_CanQuality: CompressionCaps = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CompressionCaps_CanCrunch: CompressionCaps = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CompressionCaps_CanKeyFrame: CompressionCaps = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CompressionCaps_CanBFrame: CompressionCaps = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CompressionCaps_CanWindow: CompressionCaps = 16i32;
pub const CreatePropBagOnRegKey: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2322025289, data2: 8035, data3: 4563, data4: [182, 76, 0, 192, 79, 121, 73, 142] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DDSFF_FLAGS = u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DDSFF_PROGRESSIVERENDER: DDSFF_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DECIMATION_USAGE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DECIMATION_LEGACY: DECIMATION_USAGE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DECIMATION_USE_DECODER_ONLY: DECIMATION_USAGE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DECIMATION_USE_VIDEOPORT_ONLY: DECIMATION_USAGE = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DECIMATION_USE_OVERLAY_ONLY: DECIMATION_USAGE = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DECIMATION_DEFAULT: DECIMATION_USAGE = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DECODER_CAP_NOTSUPPORTED: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DECODER_CAP_SUPPORTED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DESC_LINKAGE_TYPE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DESC_LINKAGE_RESERVED0: DESC_LINKAGE_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DESC_LINKAGE_INFORMATION: DESC_LINKAGE_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DESC_LINKAGE_EPG: DESC_LINKAGE_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DESC_LINKAGE_CA_REPLACEMENT: DESC_LINKAGE_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DESC_LINKAGE_COMPLETE_NET_BOUQUET_SI: DESC_LINKAGE_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DESC_LINKAGE_REPLACEMENT: DESC_LINKAGE_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DESC_LINKAGE_DATA: DESC_LINKAGE_TYPE = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DESC_LINKAGE_RESERVED1: DESC_LINKAGE_TYPE = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DESC_LINKAGE_USER: DESC_LINKAGE_TYPE = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DESC_LINKAGE_RESERVED2: DESC_LINKAGE_TYPE = 255i32;
pub const DIGITAL_CABLE_NETWORK_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 339224491, data2: 63355, data3: 18829, data4: [129, 202, 90, 0, 122, 236, 40, 191] };
pub const DIRECT_TV_SATELLITE_TV_NETWORK_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2478206901, data2: 37844, data3: 17187, data4: [146, 28, 193, 245, 45, 246, 29, 63] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DISPID_TUNER = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_UNIQUENAME: DISPID_TUNER = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_FRIENDLYNAME: DISPID_TUNER = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_CLSID: DISPID_TUNER = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_NETWORKTYPE: DISPID_TUNER = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS__NETWORKTYPE: DISPID_TUNER = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_CREATETUNEREQUEST: DISPID_TUNER = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_ENUMCATEGORYGUIDS: DISPID_TUNER = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_ENUMDEVICEMONIKERS: DISPID_TUNER = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_DEFAULTPREFERREDCOMPONENTTYPES: DISPID_TUNER = 9i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_FREQMAP: DISPID_TUNER = 10i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_DEFLOCATOR: DISPID_TUNER = 11i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_CLONE: DISPID_TUNER = 12i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TR_TUNINGSPACE: DISPID_TUNER = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TR_COMPONENTS: DISPID_TUNER = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TR_CLONE: DISPID_TUNER = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TR_LOCATOR: DISPID_TUNER = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_CT_CATEGORY: DISPID_TUNER = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_CT_MEDIAMAJORTYPE: DISPID_TUNER = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_CT__MEDIAMAJORTYPE: DISPID_TUNER = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_CT_MEDIASUBTYPE: DISPID_TUNER = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_CT__MEDIASUBTYPE: DISPID_TUNER = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_CT_MEDIAFORMATTYPE: DISPID_TUNER = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_CT__MEDIAFORMATTYPE: DISPID_TUNER = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_CT_MEDIATYPE: DISPID_TUNER = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_CT_CLONE: DISPID_TUNER = 9i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_LCT_LANGID: DISPID_TUNER = 100i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_MP2CT_TYPE: DISPID_TUNER = 200i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_ATSCCT_FLAGS: DISPID_TUNER = 300i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_L_CARRFREQ: DISPID_TUNER = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_L_INNERFECMETHOD: DISPID_TUNER = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_L_INNERFECRATE: DISPID_TUNER = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_L_OUTERFECMETHOD: DISPID_TUNER = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_L_OUTERFECRATE: DISPID_TUNER = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_L_MOD: DISPID_TUNER = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_L_SYMRATE: DISPID_TUNER = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_L_CLONE: DISPID_TUNER = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_L_ATSC_PHYS_CHANNEL: DISPID_TUNER = 201i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_L_ATSC_TSID: DISPID_TUNER = 202i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_L_ATSC_MP2_PROGNO: DISPID_TUNER = 203i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_L_DVBT_BANDWIDTH: DISPID_TUNER = 301i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_L_DVBT_LPINNERFECMETHOD: DISPID_TUNER = 302i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_L_DVBT_LPINNERFECRATE: DISPID_TUNER = 303i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_L_DVBT_GUARDINTERVAL: DISPID_TUNER = 304i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_L_DVBT_HALPHA: DISPID_TUNER = 305i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_L_DVBT_TRANSMISSIONMODE: DISPID_TUNER = 306i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_L_DVBT_INUSE: DISPID_TUNER = 307i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_L_DVBT2_PHYSICALLAYERPIPEID: DISPID_TUNER = 351i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_L_DVBS_POLARISATION: DISPID_TUNER = 401i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_L_DVBS_WEST: DISPID_TUNER = 402i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_L_DVBS_ORBITAL: DISPID_TUNER = 403i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_L_DVBS_AZIMUTH: DISPID_TUNER = 404i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_L_DVBS_ELEVATION: DISPID_TUNER = 405i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_L_DVBS2_DISEQ_LNB_SOURCE: DISPID_TUNER = 406i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_DVBS2_LOW_OSC_FREQ_OVERRIDE: DISPID_TUNER = 407i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_DVBS2_HI_OSC_FREQ_OVERRIDE: DISPID_TUNER = 408i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_DVBS2_LNB_SWITCH_FREQ_OVERRIDE: DISPID_TUNER = 409i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_DVBS2_SPECTRAL_INVERSION_OVERRIDE: DISPID_TUNER = 410i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_L_DVBS2_ROLLOFF: DISPID_TUNER = 411i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_L_DVBS2_PILOT: DISPID_TUNER = 412i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_L_ANALOG_STANDARD: DISPID_TUNER = 601i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_L_DTV_O_MAJOR_CHANNEL: DISPID_TUNER = 701i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_C_TYPE: DISPID_TUNER = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_C_STATUS: DISPID_TUNER = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_C_LANGID: DISPID_TUNER = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_C_DESCRIPTION: DISPID_TUNER = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_C_CLONE: DISPID_TUNER = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_C_MP2_PID: DISPID_TUNER = 101i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_C_MP2_PCRPID: DISPID_TUNER = 102i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_C_MP2_PROGNO: DISPID_TUNER = 103i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_C_ANALOG_AUDIO: DISPID_TUNER = 201i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_DVB_SYSTEMTYPE: DISPID_TUNER = 101i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_DVB2_NETWORK_ID: DISPID_TUNER = 102i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_DVBS_LOW_OSC_FREQ: DISPID_TUNER = 1001i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_DVBS_HI_OSC_FREQ: DISPID_TUNER = 1002i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_DVBS_LNB_SWITCH_FREQ: DISPID_TUNER = 1003i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_DVBS_INPUT_RANGE: DISPID_TUNER = 1004i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_DVBS_SPECTRAL_INVERSION: DISPID_TUNER = 1005i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_AR_MINFREQUENCY: DISPID_TUNER = 101i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_AR_MAXFREQUENCY: DISPID_TUNER = 102i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_AR_STEP: DISPID_TUNER = 103i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_AR_COUNTRYCODE: DISPID_TUNER = 104i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_AUX_COUNTRYCODE: DISPID_TUNER = 101i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_ATV_MINCHANNEL: DISPID_TUNER = 101i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_ATV_MAXCHANNEL: DISPID_TUNER = 102i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_ATV_INPUTTYPE: DISPID_TUNER = 103i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_ATV_COUNTRYCODE: DISPID_TUNER = 104i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_ATSC_MINMINORCHANNEL: DISPID_TUNER = 201i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_ATSC_MAXMINORCHANNEL: DISPID_TUNER = 202i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_ATSC_MINPHYSCHANNEL: DISPID_TUNER = 203i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_ATSC_MAXPHYSCHANNEL: DISPID_TUNER = 204i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_DC_MINMAJORCHANNEL: DISPID_TUNER = 301i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_DC_MAXMAJORCHANNEL: DISPID_TUNER = 302i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_DC_MINSOURCEID: DISPID_TUNER = 303i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_TUNER_TS_DC_MAXSOURCEID: DISPID_TUNER = 304i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_CHTUNER_ATVAC_CHANNEL: DISPID_TUNER = 101i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_CHTUNER_ATVDC_SYSTEM: DISPID_TUNER = 101i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_CHTUNER_ATVDC_CONTENT: DISPID_TUNER = 102i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_CHTUNER_CIDTR_CHANNELID: DISPID_TUNER = 101i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_CHTUNER_CTR_CHANNEL: DISPID_TUNER = 101i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_CHTUNER_ACTR_MINOR_CHANNEL: DISPID_TUNER = 201i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_CHTUNER_DCTR_MAJOR_CHANNEL: DISPID_TUNER = 301i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_CHTUNER_DCTR_SRCID: DISPID_TUNER = 302i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_DVBTUNER_DVBC_ATTRIBUTESVALID: DISPID_TUNER = 101i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_DVBTUNER_DVBC_PID: DISPID_TUNER = 102i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_DVBTUNER_DVBC_TAG: DISPID_TUNER = 103i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_DVBTUNER_DVBC_COMPONENTTYPE: DISPID_TUNER = 104i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_DVBTUNER_ONID: DISPID_TUNER = 101i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_DVBTUNER_TSID: DISPID_TUNER = 102i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_DVBTUNER_SID: DISPID_TUNER = 103i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_MP2TUNER_TSID: DISPID_TUNER = 101i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_MP2TUNER_PROGNO: DISPID_TUNER = 102i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPID_MP2TUNERFACTORY_CREATETUNEREQUEST: DISPID_TUNER = 1i32;
pub const DSATTRIB_BadSampleInfo: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3833884122, data2: 22584, data3: 17076, data4: [184, 151, 111, 126, 95, 170, 47, 47] };
pub const DSATTRIB_WMDRMProtectionInfo: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1081382275, data2: 27549, data3: 20204, data4: [180, 60, 103, 161, 128, 30, 26, 155] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DSHOW_STREAM_DESC {
    pub VersionNo: u32,
    pub StreamId: u32,
    pub Default: super::super::Foundation::BOOL,
    pub Creation: super::super::Foundation::BOOL,
    pub Reserved: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DSHOW_STREAM_DESC {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DSHOW_STREAM_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct DSMCC_ELEMENT {
    pub pid: u16,
    pub bComponentTag: u8,
    pub dwCarouselId: u32,
    pub dwTransactionId: u32,
    pub pNext: *mut DSMCC_ELEMENT,
}
impl ::core::marker::Copy for DSMCC_ELEMENT {}
impl ::core::clone::Clone for DSMCC_ELEMENT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DSMCC_FILTER_OPTIONS {
    pub fSpecifyProtocol: super::super::Foundation::BOOL,
    pub Protocol: u8,
    pub fSpecifyType: super::super::Foundation::BOOL,
    pub Type: u8,
    pub fSpecifyMessageId: super::super::Foundation::BOOL,
    pub MessageId: u16,
    pub fSpecifyTransactionId: super::super::Foundation::BOOL,
    pub fUseTrxIdMessageIdMask: super::super::Foundation::BOOL,
    pub TransactionId: u32,
    pub fSpecifyModuleVersion: super::super::Foundation::BOOL,
    pub ModuleVersion: u8,
    pub fSpecifyBlockNumber: super::super::Foundation::BOOL,
    pub BlockNumber: u16,
    pub fGetModuleCall: super::super::Foundation::BOOL,
    pub NumberOfBlocksInModule: u16,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DSMCC_FILTER_OPTIONS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DSMCC_FILTER_OPTIONS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct DSMCC_SECTION {
    pub TableId: u8,
    pub Header: DSMCC_SECTION_0,
    pub TableIdExtension: u16,
    pub Version: DSMCC_SECTION_1,
    pub SectionNumber: u8,
    pub LastSectionNumber: u8,
    pub ProtocolDiscriminator: u8,
    pub DsmccType: u8,
    pub MessageId: u16,
    pub TransactionId: u32,
    pub Reserved: u8,
    pub AdaptationLength: u8,
    pub MessageLength: u16,
    pub RemainingData: [u8; 1],
}
impl ::core::marker::Copy for DSMCC_SECTION {}
impl ::core::clone::Clone for DSMCC_SECTION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub union DSMCC_SECTION_0 {
    pub S: MPEG_HEADER_BITS_MIDL,
    pub W: u16,
}
impl ::core::marker::Copy for DSMCC_SECTION_0 {}
impl ::core::clone::Clone for DSMCC_SECTION_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub union DSMCC_SECTION_1 {
    pub S: MPEG_HEADER_VERSION_BITS_MIDL,
    pub B: u8,
}
impl ::core::marker::Copy for DSMCC_SECTION_1 {}
impl ::core::clone::Clone for DSMCC_SECTION_1 {
    fn clone(&self) -> Self {
        *self
    }
}
pub const DTFilter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229810, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DTV_CardStatus_Error: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DTV_CardStatus_FirmwareDownload: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DTV_CardStatus_Inserted: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DTV_CardStatus_Removed: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DTV_Entitlement_CanDecrypt: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DTV_Entitlement_NotEntitled: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DTV_Entitlement_TechnicalFailure: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DTV_MMIMessage_Close: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DTV_MMIMessage_Open: u32 = 0u32;
pub const DVBCLocator: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3308378621, data2: 38533, data3: 16424, data4: [139, 104, 110, 18, 50, 7, 159, 30] };
pub const DVBSLocator: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 502780198, data2: 16464, data3: 18416, data4: [167, 207, 76, 76, 169, 36, 19, 51] };
pub const DVBSTuningSpace: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3057653491, data2: 51618, data3: 16486, data4: [150, 240, 189, 149, 99, 49, 71, 38] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVBS_SCAN_TABLE_MAX_SIZE: u32 = 400u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DVBScramblingControlSpanningEvent {
    pub ulPID: u32,
    pub fScrambled: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DVBScramblingControlSpanningEvent {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DVBScramblingControlSpanningEvent {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVBSystemType = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_Cable: DVBSystemType = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_Terrestrial: DVBSystemType = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_Satellite: DVBSystemType = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISDB_Terrestrial: DVBSystemType = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISDB_Satellite: DVBSystemType = 4i32;
pub const DVBTLocator: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2631288577, data2: 48627, data3: 19732, data4: [142, 3, 241, 41, 131, 216, 102, 100] };
pub const DVBTLocator2: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4024695298, data2: 17879, data3: 18720, data4: [190, 150, 83, 250, 127, 53, 176, 230] };
pub const DVBTuneRequest: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 366366794, data2: 21652, data3: 18844, data4: [136, 108, 151, 60, 158, 83, 185, 241] };
pub const DVBTuningSpace: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3333507890, data2: 30378, data3: 19078, data4: [167, 172, 92, 121, 170, 245, 141, 167] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_BAT_PID: u32 = 17u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_BAT_TID: u32 = 74u32;
pub const DVB_CABLE_TV_NETWORK_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3691778023, data2: 1157, data3: 16998, data4: [185, 63, 104, 251, 248, 14, 216, 52] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_DIT_PID: u32 = 30u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_DIT_TID: u32 = 126u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_EIT_ACTUAL_TID: u32 = 78u32;
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DVB_EIT_FILTER_OPTIONS {
    pub fSpecifySegment: super::super::Foundation::BOOL,
    pub bSegment: u8,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DVB_EIT_FILTER_OPTIONS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DVB_EIT_FILTER_OPTIONS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_EIT_OTHER_TID: u32 = 79u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_EIT_PID: u32 = 18u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_NIT_ACTUAL_TID: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_NIT_OTHER_TID: u32 = 65u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_NIT_PID: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_RST_PID: u32 = 19u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_RST_TID: u32 = 113u32;
pub const DVB_SATELLITE_TV_NETWORK_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4199233370, data2: 17844, data3: 19781, data4: [132, 64, 38, 57, 87, 177, 22, 35] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_SDT_ACTUAL_TID: u32 = 66u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_SDT_OTHER_TID: u32 = 70u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_SDT_PID: u32 = 17u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_SIT_PID: u32 = 31u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_SIT_TID: u32 = 127u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVB_STRCONV_MODE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STRCONV_MODE_DVB: DVB_STRCONV_MODE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STRCONV_MODE_DVB_EMPHASIS: DVB_STRCONV_MODE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STRCONV_MODE_DVB_WITHOUT_EMPHASIS: DVB_STRCONV_MODE = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STRCONV_MODE_ISDB: DVB_STRCONV_MODE = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_ST_PID_16: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_ST_PID_17: u32 = 17u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_ST_PID_18: u32 = 18u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_ST_PID_19: u32 = 19u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_ST_PID_20: u32 = 20u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_ST_TID: u32 = 114u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_TDT_PID: u32 = 20u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_TDT_TID: u32 = 112u32;
pub const DVB_TERRESTRIAL_TV_NETWORK_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 560751327, data2: 28031, data3: 20122, data4: [133, 113, 5, 241, 78, 219, 118, 106] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_TOT_PID: u32 = 20u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVB_TOT_TID: u32 = 115u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVDFilterState = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdState_Undefined: DVDFilterState = -2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdState_Unitialized: DVDFilterState = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdState_Stopped: DVDFilterState = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdState_Paused: DVDFilterState = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdState_Running: DVDFilterState = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVDMenuIDConstants = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdMenu_Title: DVDMenuIDConstants = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdMenu_Root: DVDMenuIDConstants = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdMenu_Subpicture: DVDMenuIDConstants = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdMenu_Audio: DVDMenuIDConstants = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdMenu_Angle: DVDMenuIDConstants = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdMenu_Chapter: DVDMenuIDConstants = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVDSPExt = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdSPExt_NotSpecified: DVDSPExt = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdSPExt_Caption_Normal: DVDSPExt = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdSPExt_Caption_Big: DVDSPExt = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdSPExt_Caption_Children: DVDSPExt = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdSPExt_CC_Normal: DVDSPExt = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdSPExt_CC_Big: DVDSPExt = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdSPExt_CC_Children: DVDSPExt = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdSPExt_Forced: DVDSPExt = 9i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdSPExt_DirectorComments_Normal: DVDSPExt = 13i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdSPExt_DirectorComments_Big: DVDSPExt = 14i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdSPExt_DirectorComments_Children: DVDSPExt = 15i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVDTextStringType = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdStruct_Volume: DVDTextStringType = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdStruct_Title: DVDTextStringType = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdStruct_ParentalID: DVDTextStringType = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdStruct_PartOfTitle: DVDTextStringType = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdStruct_Cell: DVDTextStringType = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdStream_Audio: DVDTextStringType = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdStream_Subpicture: DVDTextStringType = 17i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdStream_Angle: DVDTextStringType = 18i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdChannel_Audio: DVDTextStringType = 32i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdGeneral_Name: DVDTextStringType = 48i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdGeneral_Comments: DVDTextStringType = 49i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdTitle_Series: DVDTextStringType = 56i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdTitle_Movie: DVDTextStringType = 57i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdTitle_Video: DVDTextStringType = 58i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdTitle_Album: DVDTextStringType = 59i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdTitle_Song: DVDTextStringType = 60i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdTitle_Other: DVDTextStringType = 63i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdTitle_Sub_Series: DVDTextStringType = 64i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdTitle_Sub_Movie: DVDTextStringType = 65i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdTitle_Sub_Video: DVDTextStringType = 66i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdTitle_Sub_Album: DVDTextStringType = 67i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdTitle_Sub_Song: DVDTextStringType = 68i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdTitle_Sub_Other: DVDTextStringType = 71i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdTitle_Orig_Series: DVDTextStringType = 72i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdTitle_Orig_Movie: DVDTextStringType = 73i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdTitle_Orig_Video: DVDTextStringType = 74i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdTitle_Orig_Album: DVDTextStringType = 75i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdTitle_Orig_Song: DVDTextStringType = 76i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdTitle_Orig_Other: DVDTextStringType = 79i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdOther_Scene: DVDTextStringType = 80i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdOther_Cut: DVDTextStringType = 81i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dvdOther_Take: DVDTextStringType = 82i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct DVD_ATR {
    pub ulCAT: u32,
    pub pbATRI: [u8; 768],
}
impl ::core::marker::Copy for DVD_ATR {}
impl ::core::clone::Clone for DVD_ATR {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVD_AUDIO_APPMODE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_AudioMode_None: DVD_AUDIO_APPMODE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_AudioMode_Karaoke: DVD_AUDIO_APPMODE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_AudioMode_Surround: DVD_AUDIO_APPMODE = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_AudioMode_Other: DVD_AUDIO_APPMODE = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_AUDIO_CAPS_AC3: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_AUDIO_CAPS_DTS: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_AUDIO_CAPS_LPCM: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_AUDIO_CAPS_MPEG2: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_AUDIO_CAPS_SDDS: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVD_AUDIO_FORMAT = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_AudioFormat_AC3: DVD_AUDIO_FORMAT = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_AudioFormat_MPEG1: DVD_AUDIO_FORMAT = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_AudioFormat_MPEG1_DRC: DVD_AUDIO_FORMAT = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_AudioFormat_MPEG2: DVD_AUDIO_FORMAT = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_AudioFormat_MPEG2_DRC: DVD_AUDIO_FORMAT = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_AudioFormat_LPCM: DVD_AUDIO_FORMAT = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_AudioFormat_DTS: DVD_AUDIO_FORMAT = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_AudioFormat_SDDS: DVD_AUDIO_FORMAT = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_AudioFormat_Other: DVD_AUDIO_FORMAT = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVD_AUDIO_LANG_EXT = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_AUD_EXT_NotSpecified: DVD_AUDIO_LANG_EXT = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_AUD_EXT_Captions: DVD_AUDIO_LANG_EXT = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_AUD_EXT_VisuallyImpaired: DVD_AUDIO_LANG_EXT = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_AUD_EXT_DirectorComments1: DVD_AUDIO_LANG_EXT = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_AUD_EXT_DirectorComments2: DVD_AUDIO_LANG_EXT = 4i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DVD_AudioAttributes {
    pub AppMode: DVD_AUDIO_APPMODE,
    pub AppModeData: u8,
    pub AudioFormat: DVD_AUDIO_FORMAT,
    pub Language: u32,
    pub LanguageExtension: DVD_AUDIO_LANG_EXT,
    pub fHasMultichannelInfo: super::super::Foundation::BOOL,
    pub dwFrequency: u32,
    pub bQuantization: u8,
    pub bNumberOfChannels: u8,
    pub dwReserved: [u32; 2],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DVD_AudioAttributes {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DVD_AudioAttributes {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVD_CMD_FLAGS = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_CMD_FLAG_None: DVD_CMD_FLAGS = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_CMD_FLAG_Flush: DVD_CMD_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_CMD_FLAG_SendEvents: DVD_CMD_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_CMD_FLAG_Block: DVD_CMD_FLAGS = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_CMD_FLAG_StartWhenRendered: DVD_CMD_FLAGS = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_CMD_FLAG_EndAfterRendered: DVD_CMD_FLAGS = 16i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct DVD_DECODER_CAPS {
    pub dwSize: u32,
    pub dwAudioCaps: u32,
    pub dFwdMaxRateVideo: f64,
    pub dFwdMaxRateAudio: f64,
    pub dFwdMaxRateSP: f64,
    pub dBwdMaxRateVideo: f64,
    pub dBwdMaxRateAudio: f64,
    pub dBwdMaxRateSP: f64,
    pub dwRes1: u32,
    pub dwRes2: u32,
    pub dwRes3: u32,
    pub dwRes4: u32,
}
impl ::core::marker::Copy for DVD_DECODER_CAPS {}
impl ::core::clone::Clone for DVD_DECODER_CAPS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_DEFAULT_AUDIO_STREAM: u32 = 15u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVD_DISC_SIDE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_SIDE_A: DVD_DISC_SIDE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_SIDE_B: DVD_DISC_SIDE = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVD_DOMAIN = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_DOMAIN_FirstPlay: DVD_DOMAIN = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_DOMAIN_VideoManagerMenu: DVD_DOMAIN = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_DOMAIN_VideoTitleSetMenu: DVD_DOMAIN = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_DOMAIN_Title: DVD_DOMAIN = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_DOMAIN_Stop: DVD_DOMAIN = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVD_ERROR = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_ERROR_Unexpected: DVD_ERROR = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_ERROR_CopyProtectFail: DVD_ERROR = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_ERROR_InvalidDVD1_0Disc: DVD_ERROR = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_ERROR_InvalidDiscRegion: DVD_ERROR = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_ERROR_LowParentalLevel: DVD_ERROR = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_ERROR_MacrovisionFail: DVD_ERROR = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_ERROR_IncompatibleSystemAndDecoderRegions: DVD_ERROR = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_ERROR_IncompatibleDiscAndDecoderRegions: DVD_ERROR = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_ERROR_CopyProtectOutputFail: DVD_ERROR = 9i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_ERROR_CopyProtectOutputNotSupported: DVD_ERROR = 10i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVD_FRAMERATE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_FPS_25: DVD_FRAMERATE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_FPS_30NonDrop: DVD_FRAMERATE = 3i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct DVD_HMSF_TIMECODE {
    pub bHours: u8,
    pub bMinutes: u8,
    pub bSeconds: u8,
    pub bFrames: u8,
}
impl ::core::marker::Copy for DVD_HMSF_TIMECODE {}
impl ::core::clone::Clone for DVD_HMSF_TIMECODE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVD_KARAOKE_ASSIGNMENT = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Assignment_reserved0: DVD_KARAOKE_ASSIGNMENT = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Assignment_reserved1: DVD_KARAOKE_ASSIGNMENT = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Assignment_LR: DVD_KARAOKE_ASSIGNMENT = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Assignment_LRM: DVD_KARAOKE_ASSIGNMENT = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Assignment_LR1: DVD_KARAOKE_ASSIGNMENT = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Assignment_LRM1: DVD_KARAOKE_ASSIGNMENT = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Assignment_LR12: DVD_KARAOKE_ASSIGNMENT = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Assignment_LRM12: DVD_KARAOKE_ASSIGNMENT = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVD_KARAOKE_CONTENTS = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Karaoke_GuideVocal1: DVD_KARAOKE_CONTENTS = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Karaoke_GuideVocal2: DVD_KARAOKE_CONTENTS = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Karaoke_GuideMelody1: DVD_KARAOKE_CONTENTS = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Karaoke_GuideMelody2: DVD_KARAOKE_CONTENTS = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Karaoke_GuideMelodyA: DVD_KARAOKE_CONTENTS = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Karaoke_GuideMelodyB: DVD_KARAOKE_CONTENTS = 32i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Karaoke_SoundEffectA: DVD_KARAOKE_CONTENTS = 64i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Karaoke_SoundEffectB: DVD_KARAOKE_CONTENTS = 128i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVD_KARAOKE_DOWNMIX = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Mix_0to0: DVD_KARAOKE_DOWNMIX = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Mix_1to0: DVD_KARAOKE_DOWNMIX = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Mix_2to0: DVD_KARAOKE_DOWNMIX = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Mix_3to0: DVD_KARAOKE_DOWNMIX = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Mix_4to0: DVD_KARAOKE_DOWNMIX = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Mix_Lto0: DVD_KARAOKE_DOWNMIX = 32i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Mix_Rto0: DVD_KARAOKE_DOWNMIX = 64i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Mix_0to1: DVD_KARAOKE_DOWNMIX = 256i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Mix_1to1: DVD_KARAOKE_DOWNMIX = 512i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Mix_2to1: DVD_KARAOKE_DOWNMIX = 1024i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Mix_3to1: DVD_KARAOKE_DOWNMIX = 2048i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Mix_4to1: DVD_KARAOKE_DOWNMIX = 4096i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Mix_Lto1: DVD_KARAOKE_DOWNMIX = 8192i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Mix_Rto1: DVD_KARAOKE_DOWNMIX = 16384i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DVD_KaraokeAttributes {
    pub bVersion: u8,
    pub fMasterOfCeremoniesInGuideVocal1: super::super::Foundation::BOOL,
    pub fDuet: super::super::Foundation::BOOL,
    pub ChannelAssignment: DVD_KARAOKE_ASSIGNMENT,
    pub wChannelContents: [u16; 8],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DVD_KaraokeAttributes {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DVD_KaraokeAttributes {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVD_MENU_ID = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_MENU_Title: DVD_MENU_ID = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_MENU_Root: DVD_MENU_ID = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_MENU_Subpicture: DVD_MENU_ID = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_MENU_Audio: DVD_MENU_ID = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_MENU_Angle: DVD_MENU_ID = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_MENU_Chapter: DVD_MENU_ID = 7i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct DVD_MUA_Coeff {
    pub log2_alpha: f64,
    pub log2_beta: f64,
}
impl ::core::marker::Copy for DVD_MUA_Coeff {}
impl ::core::clone::Clone for DVD_MUA_Coeff {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DVD_MUA_MixingInfo {
    pub fMixTo0: super::super::Foundation::BOOL,
    pub fMixTo1: super::super::Foundation::BOOL,
    pub fMix0InPhase: super::super::Foundation::BOOL,
    pub fMix1InPhase: super::super::Foundation::BOOL,
    pub dwSpeakerPosition: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DVD_MUA_MixingInfo {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DVD_MUA_MixingInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DVD_MenuAttributes {
    pub fCompatibleRegion: [super::super::Foundation::BOOL; 8],
    pub VideoAttributes: DVD_VideoAttributes,
    pub fAudioPresent: super::super::Foundation::BOOL,
    pub AudioAttributes: DVD_AudioAttributes,
    pub fSubpicturePresent: super::super::Foundation::BOOL,
    pub SubpictureAttributes: DVD_SubpictureAttributes,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DVD_MenuAttributes {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DVD_MenuAttributes {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DVD_MultichannelAudioAttributes {
    pub Info: [DVD_MUA_MixingInfo; 8],
    pub Coeff: [DVD_MUA_Coeff; 8],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DVD_MultichannelAudioAttributes {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DVD_MultichannelAudioAttributes {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVD_NavCmdType = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_NavCmdType_Pre: DVD_NavCmdType = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_NavCmdType_Post: DVD_NavCmdType = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_NavCmdType_Cell: DVD_NavCmdType = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_NavCmdType_Button: DVD_NavCmdType = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVD_OPTION_FLAG = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_ResetOnStop: DVD_OPTION_FLAG = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_NotifyParentalLevelChange: DVD_OPTION_FLAG = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_HMSF_TimeCodeEvents: DVD_OPTION_FLAG = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_AudioDuringFFwdRew: DVD_OPTION_FLAG = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_EnableNonblockingAPIs: DVD_OPTION_FLAG = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_CacheSizeInMB: DVD_OPTION_FLAG = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_EnablePortableBookmarks: DVD_OPTION_FLAG = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_EnableExtendedCopyProtectErrors: DVD_OPTION_FLAG = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_NotifyPositionChange: DVD_OPTION_FLAG = 9i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_IncreaseOutputControl: DVD_OPTION_FLAG = 10i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_EnableStreaming: DVD_OPTION_FLAG = 11i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_EnableESOutput: DVD_OPTION_FLAG = 12i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_EnableTitleLength: DVD_OPTION_FLAG = 13i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_DisableStillThrottle: DVD_OPTION_FLAG = 14i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_EnableLoggingEvents: DVD_OPTION_FLAG = 15i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_MaxReadBurstInKB: DVD_OPTION_FLAG = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_ReadBurstPeriodInMS: DVD_OPTION_FLAG = 17i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_RestartDisc: DVD_OPTION_FLAG = 18i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_EnableCC: DVD_OPTION_FLAG = 19i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVD_PARENTAL_LEVEL = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_PARENTAL_LEVEL_8: DVD_PARENTAL_LEVEL = 32768i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_PARENTAL_LEVEL_7: DVD_PARENTAL_LEVEL = 16384i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_PARENTAL_LEVEL_6: DVD_PARENTAL_LEVEL = 8192i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_PARENTAL_LEVEL_5: DVD_PARENTAL_LEVEL = 4096i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_PARENTAL_LEVEL_4: DVD_PARENTAL_LEVEL = 2048i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_PARENTAL_LEVEL_3: DVD_PARENTAL_LEVEL = 1024i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_PARENTAL_LEVEL_2: DVD_PARENTAL_LEVEL = 512i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_PARENTAL_LEVEL_1: DVD_PARENTAL_LEVEL = 256i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVD_PB_STOPPED = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_PB_STOPPED_Other: DVD_PB_STOPPED = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_PB_STOPPED_NoBranch: DVD_PB_STOPPED = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_PB_STOPPED_NoFirstPlayDomain: DVD_PB_STOPPED = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_PB_STOPPED_StopCommand: DVD_PB_STOPPED = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_PB_STOPPED_Reset: DVD_PB_STOPPED = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_PB_STOPPED_DiscEjected: DVD_PB_STOPPED = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_PB_STOPPED_IllegalNavCommand: DVD_PB_STOPPED = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_PB_STOPPED_PlayPeriodAutoStop: DVD_PB_STOPPED = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_PB_STOPPED_PlayChapterAutoStop: DVD_PB_STOPPED = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_PB_STOPPED_ParentalFailure: DVD_PB_STOPPED = 9i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_PB_STOPPED_RegionFailure: DVD_PB_STOPPED = 10i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_PB_STOPPED_MacrovisionFailure: DVD_PB_STOPPED = 11i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_PB_STOPPED_DiscReadError: DVD_PB_STOPPED = 12i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_PB_STOPPED_CopyProtectFailure: DVD_PB_STOPPED = 13i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_PB_STOPPED_CopyProtectOutputFailure: DVD_PB_STOPPED = 14i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_PB_STOPPED_CopyProtectOutputNotSupported: DVD_PB_STOPPED = 15i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct DVD_PLAYBACK_LOCATION {
    pub TitleNum: u32,
    pub ChapterNum: u32,
    pub TimeCode: u32,
}
impl ::core::marker::Copy for DVD_PLAYBACK_LOCATION {}
impl ::core::clone::Clone for DVD_PLAYBACK_LOCATION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct DVD_PLAYBACK_LOCATION2 {
    pub TitleNum: u32,
    pub ChapterNum: u32,
    pub TimeCode: DVD_HMSF_TIMECODE,
    pub TimeCodeFlags: u32,
}
impl ::core::marker::Copy for DVD_PLAYBACK_LOCATION2 {}
impl ::core::clone::Clone for DVD_PLAYBACK_LOCATION2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVD_PLAY_DIRECTION = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_DIR_FORWARD: DVD_PLAY_DIRECTION = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_DIR_BACKWARD: DVD_PLAY_DIRECTION = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVD_PREFERRED_DISPLAY_MODE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPLAY_CONTENT_DEFAULT: DVD_PREFERRED_DISPLAY_MODE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPLAY_16x9: DVD_PREFERRED_DISPLAY_MODE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPLAY_4x3_PANSCAN_PREFERRED: DVD_PREFERRED_DISPLAY_MODE = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DISPLAY_4x3_LETTERBOX_PREFERRED: DVD_PREFERRED_DISPLAY_MODE = 3i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct DVD_REGION {
    pub CopySystem: u8,
    pub RegionData: u8,
    pub SystemRegion: u8,
    pub ResetCount: u8,
}
impl ::core::marker::Copy for DVD_REGION {}
impl ::core::clone::Clone for DVD_REGION {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVD_RELATIVE_BUTTON = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Relative_Upper: DVD_RELATIVE_BUTTON = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Relative_Lower: DVD_RELATIVE_BUTTON = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Relative_Left: DVD_RELATIVE_BUTTON = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Relative_Right: DVD_RELATIVE_BUTTON = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_STREAM_DATA_CURRENT: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_STREAM_DATA_VMGM: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_STREAM_DATA_VTSM: u32 = 1025u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVD_SUBPICTURE_CODING = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_SPCoding_RunLength: DVD_SUBPICTURE_CODING = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_SPCoding_Extended: DVD_SUBPICTURE_CODING = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_SPCoding_Other: DVD_SUBPICTURE_CODING = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVD_SUBPICTURE_LANG_EXT = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_SP_EXT_NotSpecified: DVD_SUBPICTURE_LANG_EXT = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_SP_EXT_Caption_Normal: DVD_SUBPICTURE_LANG_EXT = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_SP_EXT_Caption_Big: DVD_SUBPICTURE_LANG_EXT = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_SP_EXT_Caption_Children: DVD_SUBPICTURE_LANG_EXT = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_SP_EXT_CC_Normal: DVD_SUBPICTURE_LANG_EXT = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_SP_EXT_CC_Big: DVD_SUBPICTURE_LANG_EXT = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_SP_EXT_CC_Children: DVD_SUBPICTURE_LANG_EXT = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_SP_EXT_Forced: DVD_SUBPICTURE_LANG_EXT = 9i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_SP_EXT_DirectorComments_Normal: DVD_SUBPICTURE_LANG_EXT = 13i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_SP_EXT_DirectorComments_Big: DVD_SUBPICTURE_LANG_EXT = 14i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_SP_EXT_DirectorComments_Children: DVD_SUBPICTURE_LANG_EXT = 15i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVD_SUBPICTURE_TYPE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_SPType_NotSpecified: DVD_SUBPICTURE_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_SPType_Language: DVD_SUBPICTURE_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_SPType_Other: DVD_SUBPICTURE_TYPE = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct DVD_SubpictureAttributes {
    pub Type: DVD_SUBPICTURE_TYPE,
    pub CodingMode: DVD_SUBPICTURE_CODING,
    pub Language: u32,
    pub LanguageExtension: DVD_SUBPICTURE_LANG_EXT,
}
impl ::core::marker::Copy for DVD_SubpictureAttributes {}
impl ::core::clone::Clone for DVD_SubpictureAttributes {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct DVD_TIMECODE {
    pub _bitfield: u32,
}
impl ::core::marker::Copy for DVD_TIMECODE {}
impl ::core::clone::Clone for DVD_TIMECODE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVD_TIMECODE_FLAGS = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_TC_FLAG_25fps: DVD_TIMECODE_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_TC_FLAG_30fps: DVD_TIMECODE_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_TC_FLAG_DropFrame: DVD_TIMECODE_FLAGS = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_TC_FLAG_Interpolated: DVD_TIMECODE_FLAGS = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVD_TITLE_APPMODE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_AppMode_Not_Specified: DVD_TITLE_APPMODE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_AppMode_Karaoke: DVD_TITLE_APPMODE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_AppMode_Other: DVD_TITLE_APPMODE = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_TITLE_MENU: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVD_TextCharSet = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_CharSet_Unicode: DVD_TextCharSet = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_CharSet_ISO646: DVD_TextCharSet = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_CharSet_JIS_Roman_Kanji: DVD_TextCharSet = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_CharSet_ISO8859_1: DVD_TextCharSet = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_CharSet_ShiftJIS_Kanji_Roman_Katakana: DVD_TextCharSet = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVD_TextStringType = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Struct_Volume: DVD_TextStringType = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Struct_Title: DVD_TextStringType = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Struct_ParentalID: DVD_TextStringType = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Struct_PartOfTitle: DVD_TextStringType = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Struct_Cell: DVD_TextStringType = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Stream_Audio: DVD_TextStringType = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Stream_Subpicture: DVD_TextStringType = 17i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Stream_Angle: DVD_TextStringType = 18i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Channel_Audio: DVD_TextStringType = 32i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_General_Name: DVD_TextStringType = 48i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_General_Comments: DVD_TextStringType = 49i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Title_Series: DVD_TextStringType = 56i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Title_Movie: DVD_TextStringType = 57i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Title_Video: DVD_TextStringType = 58i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Title_Album: DVD_TextStringType = 59i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Title_Song: DVD_TextStringType = 60i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Title_Other: DVD_TextStringType = 63i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Title_Sub_Series: DVD_TextStringType = 64i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Title_Sub_Movie: DVD_TextStringType = 65i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Title_Sub_Video: DVD_TextStringType = 66i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Title_Sub_Album: DVD_TextStringType = 67i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Title_Sub_Song: DVD_TextStringType = 68i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Title_Sub_Other: DVD_TextStringType = 71i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Title_Orig_Series: DVD_TextStringType = 72i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Title_Orig_Movie: DVD_TextStringType = 73i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Title_Orig_Video: DVD_TextStringType = 74i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Title_Orig_Album: DVD_TextStringType = 75i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Title_Orig_Song: DVD_TextStringType = 76i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Title_Orig_Other: DVD_TextStringType = 79i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Other_Scene: DVD_TextStringType = 80i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Other_Cut: DVD_TextStringType = 81i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_Other_Take: DVD_TextStringType = 82i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DVD_TitleAttributes {
    pub Anonymous: DVD_TitleAttributes_0,
    pub VideoAttributes: DVD_VideoAttributes,
    pub ulNumberOfAudioStreams: u32,
    pub AudioAttributes: [DVD_AudioAttributes; 8],
    pub MultichannelAudioAttributes: [DVD_MultichannelAudioAttributes; 8],
    pub ulNumberOfSubpictureStreams: u32,
    pub SubpictureAttributes: [DVD_SubpictureAttributes; 32],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DVD_TitleAttributes {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DVD_TitleAttributes {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub union DVD_TitleAttributes_0 {
    pub AppMode: DVD_TITLE_APPMODE,
    pub TitleLength: DVD_HMSF_TIMECODE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DVD_TitleAttributes_0 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DVD_TitleAttributes_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVD_VIDEO_COMPRESSION = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_VideoCompression_Other: DVD_VIDEO_COMPRESSION = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_VideoCompression_MPEG1: DVD_VIDEO_COMPRESSION = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_VideoCompression_MPEG2: DVD_VIDEO_COMPRESSION = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DVD_VideoAttributes {
    pub fPanscanPermitted: super::super::Foundation::BOOL,
    pub fLetterboxPermitted: super::super::Foundation::BOOL,
    pub ulAspectX: u32,
    pub ulAspectY: u32,
    pub ulFrameRate: u32,
    pub ulFrameHeight: u32,
    pub Compression: DVD_VIDEO_COMPRESSION,
    pub fLine21Field1InGOP: super::super::Foundation::BOOL,
    pub fLine21Field2InGOP: super::super::Foundation::BOOL,
    pub ulSourceResolutionX: u32,
    pub ulSourceResolutionY: u32,
    pub fIsSourceLetterboxed: super::super::Foundation::BOOL,
    pub fIsFilmMode: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DVD_VideoAttributes {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DVD_VideoAttributes {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DVD_WARNING = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_WARNING_InvalidDVD1_0Disc: DVD_WARNING = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_WARNING_FormatNotSupported: DVD_WARNING = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_WARNING_IllegalNavCommand: DVD_WARNING = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_WARNING_Open: DVD_WARNING = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_WARNING_Seek: DVD_WARNING = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVD_WARNING_Read: DVD_WARNING = 6i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct DVINFO {
    pub dwDVAAuxSrc: u32,
    pub dwDVAAuxCtl: u32,
    pub dwDVAAuxSrc1: u32,
    pub dwDVAAuxCtl1: u32,
    pub dwDVVAuxSrc: u32,
    pub dwDVVAuxCtl: u32,
    pub dwDVReserved: [u32; 2],
}
impl ::core::marker::Copy for DVINFO {}
impl ::core::clone::Clone for DVINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DVR_STREAM_DESC {
    pub Version: u32,
    pub StreamId: u32,
    pub Default: super::super::Foundation::BOOL,
    pub Creation: super::super::Foundation::BOOL,
    pub Reserved: u32,
    pub guidSubMediaType: ::windows_sys::core::GUID,
    pub guidFormatType: ::windows_sys::core::GUID,
    pub MediaType: AM_MEDIA_TYPE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DVR_STREAM_DESC {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DVR_STREAM_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DWORD_ALLPARAMS: i32 = -1i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Media_MediaFoundation\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct3D9", feature = "Win32_Media_MediaFoundation"))]
pub struct DXVA2SW_CALLBACKS {
    pub Size: u32,
    pub GetVideoProcessorRenderTargetCount: PDXVA2SW_GETVIDEOPROCESSORRENDERTARGETCOUNT,
    pub GetVideoProcessorRenderTargets: PDXVA2SW_GETVIDEOPROCESSORRENDERTARGETS,
    pub GetVideoProcessorCaps: PDXVA2SW_GETVIDEOPROCESSORCAPS,
    pub GetVideoProcessorSubStreamFormatCount: PDXVA2SW_GETVIDEOPROCESSORSUBSTREAMFORMATCOUNT,
    pub GetVideoProcessorSubStreamFormats: PDXVA2SW_GETVIDEOPROCESSORSUBSTREAMFORMATS,
    pub GetProcAmpRange: PDXVA2SW_GETPROCAMPRANGE,
    pub GetFilterPropertyRange: PDXVA2SW_GETFILTERPROPERTYRANGE,
    pub CreateVideoProcessDevice: PDXVA2SW_CREATEVIDEOPROCESSDEVICE,
    pub DestroyVideoProcessDevice: PDXVA2SW_DESTROYVIDEOPROCESSDEVICE,
    pub VideoProcessBeginFrame: PDXVA2SW_VIDEOPROCESSBEGINFRAME,
    pub VideoProcessEndFrame: PDXVA2SW_VIDEOPROCESSENDFRAME,
    pub VideoProcessSetRenderTarget: PDXVA2SW_VIDEOPROCESSSETRENDERTARGET,
    pub VideoProcessBlt: PDXVA2SW_VIDEOPROCESSBLT,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct3D9", feature = "Win32_Media_MediaFoundation"))]
impl ::core::marker::Copy for DXVA2SW_CALLBACKS {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct3D9", feature = "Win32_Media_MediaFoundation"))]
impl ::core::clone::Clone for DXVA2SW_CALLBACKS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_System_Diagnostics_Etw\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Diagnostics_Etw"))]
pub struct DXVA2TraceVideoProcessBltData {
    pub wmiHeader: super::super::System::Diagnostics::Etw::EVENT_TRACE_HEADER,
    pub pObject: u64,
    pub pRenderTarget: u64,
    pub TargetFrameTime: u64,
    pub TargetRect: super::super::Foundation::RECT,
    pub Enter: super::super::Foundation::BOOL,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Diagnostics_Etw"))]
impl ::core::marker::Copy for DXVA2TraceVideoProcessBltData {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Diagnostics_Etw"))]
impl ::core::clone::Clone for DXVA2TraceVideoProcessBltData {
    fn clone(&self) -> Self {
        *self
    }
}
pub const DXVA2Trace_Control: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2688052853, data2: 63244, data3: 17996, data4: [169, 206, 51, 196, 78, 9, 22, 35] };
pub const DXVA2Trace_DecodeDevBeginFrame: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2681318646, data2: 17611, data3: 17975, data4: [188, 98, 44, 17, 169, 96, 143, 144] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_System_Diagnostics_Etw\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Diagnostics_Etw"))]
pub struct DXVA2Trace_DecodeDevBeginFrameData {
    pub wmiHeader: super::super::System::Diagnostics::Etw::EVENT_TRACE_HEADER,
    pub pObject: u64,
    pub pRenderTarget: u64,
    pub Enter: super::super::Foundation::BOOL,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Diagnostics_Etw"))]
impl ::core::marker::Copy for DXVA2Trace_DecodeDevBeginFrameData {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Diagnostics_Etw"))]
impl ::core::clone::Clone for DXVA2Trace_DecodeDevBeginFrameData {
    fn clone(&self) -> Self {
        *self
    }
}
pub const DXVA2Trace_DecodeDevCreated: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3034453921, data2: 50610, data3: 17662, data4: [134, 213, 217, 122, 100, 129, 20, 255] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_System_Diagnostics_Etw\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Diagnostics_Etw"))]
pub struct DXVA2Trace_DecodeDevCreatedData {
    pub wmiHeader: super::super::System::Diagnostics::Etw::EVENT_TRACE_HEADER,
    pub pObject: u64,
    pub pD3DDevice: u64,
    pub DeviceGuid: ::windows_sys::core::GUID,
    pub Width: u32,
    pub Height: u32,
    pub Enter: super::super::Foundation::BOOL,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Diagnostics_Etw"))]
impl ::core::marker::Copy for DXVA2Trace_DecodeDevCreatedData {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Diagnostics_Etw"))]
impl ::core::clone::Clone for DXVA2Trace_DecodeDevCreatedData {
    fn clone(&self) -> Self {
        *self
    }
}
pub const DXVA2Trace_DecodeDevDestroyed: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2235481586, data2: 16736, data3: 16925, data4: [136, 147, 99, 220, 234, 79, 24, 187] };
pub const DXVA2Trace_DecodeDevEndFrame: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2679360307, data2: 18396, data3: 18585, data4: [152, 200, 192, 198, 205, 124, 211, 203] };
pub const DXVA2Trace_DecodeDevExecute: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2232085324, data2: 53658, data3: 17929, data4: [179, 180, 188, 191, 14, 34, 18, 30] };
pub const DXVA2Trace_DecodeDevGetBuffer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1471228155, data2: 29387, data3: 16695, data4: [165, 117, 217, 31, 163, 22, 8, 151] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_System_Diagnostics_Etw\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Diagnostics_Etw"))]
pub struct DXVA2Trace_DecodeDevGetBufferData {
    pub wmiHeader: super::super::System::Diagnostics::Etw::EVENT_TRACE_HEADER,
    pub pObject: u64,
    pub BufferType: u32,
    pub Enter: super::super::Foundation::BOOL,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Diagnostics_Etw"))]
impl ::core::marker::Copy for DXVA2Trace_DecodeDevGetBufferData {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Diagnostics_Etw"))]
impl ::core::clone::Clone for DXVA2Trace_DecodeDevGetBufferData {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_System_Diagnostics_Etw\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Diagnostics_Etw"))]
pub struct DXVA2Trace_DecodeDeviceData {
    pub wmiHeader: super::super::System::Diagnostics::Etw::EVENT_TRACE_HEADER,
    pub pObject: u64,
    pub Enter: super::super::Foundation::BOOL,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Diagnostics_Etw"))]
impl ::core::marker::Copy for DXVA2Trace_DecodeDeviceData {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Diagnostics_Etw"))]
impl ::core::clone::Clone for DXVA2Trace_DecodeDeviceData {
    fn clone(&self) -> Self {
        *self
    }
}
pub const DXVA2Trace_VideoProcessBlt: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1762172096, data2: 29099, data3: 17104, data4: [149, 58, 40, 135, 191, 5, 168, 175] };
pub const DXVA2Trace_VideoProcessDevCreated: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2304051398, data2: 21517, data3: 19591, data4: [152, 248, 141, 203, 242, 218, 187, 42] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_System_Diagnostics_Etw\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Diagnostics_Etw"))]
pub struct DXVA2Trace_VideoProcessDevCreatedData {
    pub wmiHeader: super::super::System::Diagnostics::Etw::EVENT_TRACE_HEADER,
    pub pObject: u64,
    pub pD3DDevice: u64,
    pub DeviceGuid: ::windows_sys::core::GUID,
    pub RTFourCC: u32,
    pub Width: u32,
    pub Height: u32,
    pub Enter: super::super::Foundation::BOOL,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Diagnostics_Etw"))]
impl ::core::marker::Copy for DXVA2Trace_VideoProcessDevCreatedData {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Diagnostics_Etw"))]
impl ::core::clone::Clone for DXVA2Trace_VideoProcessDevCreatedData {
    fn clone(&self) -> Self {
        *self
    }
}
pub const DXVA2Trace_VideoProcessDevDestroyed: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4185862321, data2: 64329, data3: 17095, data4: [142, 232, 136, 189, 250, 146, 212, 226] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_System_Diagnostics_Etw\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Diagnostics_Etw"))]
pub struct DXVA2Trace_VideoProcessDeviceData {
    pub wmiHeader: super::super::System::Diagnostics::Etw::EVENT_TRACE_HEADER,
    pub pObject: u64,
    pub Enter: super::super::Foundation::BOOL,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Diagnostics_Etw"))]
impl ::core::marker::Copy for DXVA2Trace_VideoProcessDeviceData {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Diagnostics_Etw"))]
impl ::core::clone::Clone for DXVA2Trace_VideoProcessDeviceData {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DXVA2_DestinationFlags = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA2_DestinationFlag_Background_Changed: DXVA2_DestinationFlags = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA2_DestinationFlag_TargetRect_Changed: DXVA2_DestinationFlags = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA2_DestinationFlag_ColorData_Changed: DXVA2_DestinationFlags = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA2_DestinationFlag_Alpha_Changed: DXVA2_DestinationFlags = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA2_DestinationFlag_RFF: DXVA2_DestinationFlags = 65536i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA2_DestinationFlag_TFF: DXVA2_DestinationFlags = 131072i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA2_DestinationFlag_RFF_TFF_Present: DXVA2_DestinationFlags = 262144i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA2_DestinationFlagMask: DXVA2_DestinationFlags = -65521i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DXVA2_SampleFlags = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA2_SampleFlag_Palette_Changed: DXVA2_SampleFlags = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA2_SampleFlag_SrcRect_Changed: DXVA2_SampleFlags = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA2_SampleFlag_DstRect_Changed: DXVA2_SampleFlags = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA2_SampleFlag_ColorData_Changed: DXVA2_SampleFlags = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA2_SampleFlag_PlanarAlpha_Changed: DXVA2_SampleFlags = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA2_SampleFlag_RFF: DXVA2_SampleFlags = 65536i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA2_SampleFlag_TFF: DXVA2_SampleFlags = 131072i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA2_SampleFlag_RFF_TFF_Present: DXVA2_SampleFlags = 262144i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA2_SampleFlagsMask: DXVA2_SampleFlags = -65505i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_Media_MediaFoundation\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_MediaFoundation"))]
pub struct DXVA2_VIDEOPROCESSBLT {
    pub TargetFrame: i64,
    pub TargetRect: super::super::Foundation::RECT,
    pub ConstrictionSize: super::super::Foundation::SIZE,
    pub StreamingFlags: u32,
    pub BackgroundColor: super::MediaFoundation::DXVA2_AYUVSample16,
    pub DestFormat: super::MediaFoundation::DXVA2_ExtendedFormat,
    pub DestFlags: u32,
    pub ProcAmpValues: super::MediaFoundation::DXVA2_ProcAmpValues,
    pub Alpha: super::MediaFoundation::DXVA2_Fixed32,
    pub NoiseFilterLuma: super::MediaFoundation::DXVA2_FilterValues,
    pub NoiseFilterChroma: super::MediaFoundation::DXVA2_FilterValues,
    pub DetailFilterLuma: super::MediaFoundation::DXVA2_FilterValues,
    pub DetailFilterChroma: super::MediaFoundation::DXVA2_FilterValues,
    pub pSrcSurfaces: *mut DXVA2_VIDEOSAMPLE,
    pub NumSrcSurfaces: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_MediaFoundation"))]
impl ::core::marker::Copy for DXVA2_VIDEOPROCESSBLT {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_MediaFoundation"))]
impl ::core::clone::Clone for DXVA2_VIDEOPROCESSBLT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_Media_MediaFoundation\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_MediaFoundation"))]
pub struct DXVA2_VIDEOSAMPLE {
    pub Start: i64,
    pub End: i64,
    pub SampleFormat: super::MediaFoundation::DXVA2_ExtendedFormat,
    pub SampleFlags: u32,
    pub SrcResource: *mut ::core::ffi::c_void,
    pub SrcRect: super::super::Foundation::RECT,
    pub DstRect: super::super::Foundation::RECT,
    pub Pal: [super::MediaFoundation::DXVA2_AYUVSample8; 16],
    pub PlanarAlpha: super::MediaFoundation::DXVA2_Fixed32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_MediaFoundation"))]
impl ::core::marker::Copy for DXVA2_VIDEOSAMPLE {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_MediaFoundation"))]
impl ::core::clone::Clone for DXVA2_VIDEOSAMPLE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_ALPHA_BLEND_COMBINATION_BUFFER: u32 = 13u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_ALPHA_BLEND_COMBINATION_FUNCTION: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_ALPHA_BLEND_DATA_LOAD_FUNCTION: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_AYUV_BUFFER: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_BIDIRECTIONAL_AVERAGING_H263_TRUNC: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_BIDIRECTIONAL_AVERAGING_MPEG2_ROUND: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_BITSTREAM_CONCEALMENT_METHOD_BACKWARD: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_BITSTREAM_CONCEALMENT_METHOD_FORWARD: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_BITSTREAM_CONCEALMENT_METHOD_INTRA: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_BITSTREAM_CONCEALMENT_METHOD_UNSPECIFIED: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_BITSTREAM_CONCEALMENT_NEED_LIKELY: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_BITSTREAM_CONCEALMENT_NEED_MILD: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_BITSTREAM_CONCEALMENT_NEED_SEVERE: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_BITSTREAM_CONCEALMENT_NEED_UNLIKELY: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_BITSTREAM_DATA_BUFFER: u32 = 7u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_CHROMA_FORMAT_420: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_CHROMA_FORMAT_422: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_CHROMA_FORMAT_444: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_COMPBUFFER_TYPE_THAT_IS_NOT_USED: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_CONFIG_BLEND_TYPE_BACK_HARDWARE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_CONFIG_BLEND_TYPE_FRONT_BUFFER: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_CONFIG_DATA_TYPE_AI44: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_CONFIG_DATA_TYPE_AYUV: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_CONFIG_DATA_TYPE_DPXD: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_CONFIG_DATA_TYPE_IA44: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_COPPCommandFnCode: u32 = 4u32;
pub const DXVA_COPPDevice: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3527768797, data2: 35225, data3: 17901, data4: [138, 138, 209, 170, 4, 123, 164, 213] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_COPPGetCertificateLengthFnCode: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_COPPKeyExchangeFnCode: u32 = 2u32;
pub const DXVA_COPPQueryBusData: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3337934451, data2: 24948, data3: 16772, data4: [142, 53, 246, 219, 82, 0, 188, 186] };
pub const DXVA_COPPQueryConnectorType: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2177941461, data2: 27390, data3: 18626, data4: [153, 192, 149, 160, 143, 151, 197, 218] };
pub const DXVA_COPPQueryDisplayData: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3619625891, data2: 44307, data3: 20366, data4: [175, 152, 13, 203, 60, 162, 4, 204] };
pub const DXVA_COPPQueryGlobalProtectionLevel: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 425140490, data2: 30566, data3: 17706, data4: [185, 154, 210, 122, 237, 84, 240, 58] };
pub const DXVA_COPPQueryHDCPKeyData: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 230006132, data2: 43410, data3: 18734, data4: [160, 189, 194, 63, 218, 86, 78, 0] };
pub const DXVA_COPPQueryLocalProtectionLevel: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2986825815, data2: 16090, data3: 19805, data4: [136, 219, 116, 143, 140, 26, 5, 73] };
pub const DXVA_COPPQueryProtectionType: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 955426817, data2: 39532, data3: 18619, data4: [145, 7, 182, 105, 110, 111, 23, 151] };
pub const DXVA_COPPQuerySignaling: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1714005393, data2: 15225, data3: 19699, data4: [146, 74, 17, 232, 231, 129, 22, 113] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_COPPQueryStatusFnCode: u32 = 5u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_COPPSequenceStartFnCode: u32 = 3u32;
pub const DXVA_COPPSetProtectionLevel: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2612605564, data2: 20149, data3: 18215, data4: [159, 0, 180, 43, 9, 25, 192, 218] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct DXVA_COPPSetProtectionLevelCmdData {
    pub ProtType: u32,
    pub ProtLevel: u32,
    pub ExtendedInfoChangeMask: u32,
    pub ExtendedInfoData: u32,
}
impl ::core::marker::Copy for DXVA_COPPSetProtectionLevelCmdData {}
impl ::core::clone::Clone for DXVA_COPPSetProtectionLevelCmdData {
    fn clone(&self) -> Self {
        *self
    }
}
pub const DXVA_COPPSetSignaling: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 161886629, data2: 54916, data3: 19552, data4: [142, 77, 211, 187, 15, 11, 227, 238] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct DXVA_COPPSetSignalingCmdData {
    pub ActiveTVProtectionStandard: u32,
    pub AspectRatioChangeMask1: u32,
    pub AspectRatioData1: u32,
    pub AspectRatioChangeMask2: u32,
    pub AspectRatioData2: u32,
    pub AspectRatioChangeMask3: u32,
    pub AspectRatioData3: u32,
    pub ExtendedInfoChangeMask: [u32; 4],
    pub ExtendedInfoData: [u32; 4],
    pub Reserved: u32,
}
impl ::core::marker::Copy for DXVA_COPPSetSignalingCmdData {}
impl ::core::clone::Clone for DXVA_COPPSetSignalingCmdData {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct DXVA_COPPStatusData {
    pub rApp: ::windows_sys::core::GUID,
    pub dwFlags: u32,
    pub dwData: u32,
    pub ExtendedInfoValidMask: u32,
    pub ExtendedInfoData: u32,
}
impl ::core::marker::Copy for DXVA_COPPStatusData {}
impl ::core::clone::Clone for DXVA_COPPStatusData {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct DXVA_COPPStatusDisplayData {
    pub rApp: ::windows_sys::core::GUID,
    pub dwFlags: u32,
    pub DisplayWidth: u32,
    pub DisplayHeight: u32,
    pub Format: u32,
    pub d3dFormat: u32,
    pub FreqNumerator: u32,
    pub FreqDenominator: u32,
}
impl ::core::marker::Copy for DXVA_COPPStatusDisplayData {}
impl ::core::clone::Clone for DXVA_COPPStatusDisplayData {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct DXVA_COPPStatusHDCPKeyData {
    pub rApp: ::windows_sys::core::GUID,
    pub dwFlags: u32,
    pub dwHDCPFlags: u32,
    pub BKey: ::windows_sys::core::GUID,
    pub Reserved1: ::windows_sys::core::GUID,
    pub Reserved2: ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for DXVA_COPPStatusHDCPKeyData {}
impl ::core::clone::Clone for DXVA_COPPStatusHDCPKeyData {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct DXVA_COPPStatusSignalingCmdData {
    pub rApp: ::windows_sys::core::GUID,
    pub dwFlags: u32,
    pub AvailableTVProtectionStandards: u32,
    pub ActiveTVProtectionStandard: u32,
    pub TVType: u32,
    pub AspectRatioValidMask1: u32,
    pub AspectRatioData1: u32,
    pub AspectRatioValidMask2: u32,
    pub AspectRatioData2: u32,
    pub AspectRatioValidMask3: u32,
    pub AspectRatioData3: u32,
    pub ExtendedInfoValidMask: [u32; 4],
    pub ExtendedInfoData: [u32; 4],
}
impl ::core::marker::Copy for DXVA_COPPStatusSignalingCmdData {}
impl ::core::clone::Clone for DXVA_COPPStatusSignalingCmdData {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_DCCMD_SURFACE_BUFFER: u32 = 12u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_DEBLOCKING_CONTROL_BUFFER: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_DEBLOCKING_FILTER_FUNCTION: u32 = 5u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_DPXD_SURFACE_BUFFER: u32 = 10u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_DeinterlaceBltExFnCode: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_DeinterlaceBltFnCode: u32 = 1u32;
pub const DXVA_DeinterlaceBobDevice: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 861578094, data2: 30852, data3: 17316, data4: [156, 145, 127, 135, 250, 243, 227, 126] };
pub const DXVA_DeinterlaceContainerDevice: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 243649427, data2: 12358, data3: 20464, data4: [174, 204, 213, 140, 181, 240, 53, 253] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_DeinterlaceQueryAvailableModesFnCode: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_DeinterlaceQueryModeCapsFnCode: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_ENCRYPTPROTOCOLFUNCFLAG_ACCEL: u32 = 16776968u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_ENCRYPTPROTOCOLFUNCFLAG_HOST: u32 = 16776960u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_EXECUTE_RETURN_DATA_ERROR_MINOR: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_EXECUTE_RETURN_DATA_ERROR_SEVERE: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_EXECUTE_RETURN_DATA_ERROR_SIGNIF: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_EXECUTE_RETURN_OK: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_EXECUTE_RETURN_OTHER_ERROR_SEVERE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_ExtColorData_ShiftBase: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_FILM_GRAIN_BUFFER: u32 = 17u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_FILM_GRAIN_SYNTHESIS_FUNCTION: u32 = 6u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_HIGHLIGHT_BUFFER: u32 = 11u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_IA44_SURFACE_BUFFER: u32 = 9u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_INVERSE_QUANTIZATION_MATRIX_BUFFER: u32 = 5u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_MACROBLOCK_CONTROL_BUFFER: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_MOTION_VECTOR_BUFFER: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_MV_PRECISION_AND_CHROMA_RELATION_H261: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_MV_PRECISION_AND_CHROMA_RELATION_H263: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_MV_PRECISION_AND_CHROMA_RELATION_MPEG2: u32 = 0u32;
pub const DXVA_ModeAV1_VLD_12bit_Profile2: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 387084297, data2: 40975, data3: 19681, data4: [153, 78, 191, 64, 129, 246, 243, 240] };
pub const DXVA_ModeAV1_VLD_12bit_Profile2_420: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 763412182, data2: 40108, data3: 18485, data4: [158, 145, 50, 123, 188, 79, 158, 232] };
pub const DXVA_ModeAV1_VLD_Profile0: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3099479243, data2: 53075, data3: 18106, data4: [141, 89, 214, 184, 166, 218, 93, 42] };
pub const DXVA_ModeAV1_VLD_Profile1: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1765211919, data2: 17841, data3: 16739, data4: [156, 193, 100, 110, 246, 148, 97, 8] };
pub const DXVA_ModeAV1_VLD_Profile2: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 207563425, data2: 58689, data3: 16521, data4: [187, 123, 152, 17, 10, 25, 215, 200] };
pub const DXVA_ModeH261_A: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487617, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const DXVA_ModeH261_B: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487618, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const DXVA_ModeH263_A: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487619, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const DXVA_ModeH263_B: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487620, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const DXVA_ModeH263_C: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487621, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const DXVA_ModeH263_D: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487622, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const DXVA_ModeH263_E: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487623, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const DXVA_ModeH263_F: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487624, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const DXVA_ModeH264_A: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487716, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const DXVA_ModeH264_B: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487717, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const DXVA_ModeH264_C: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487718, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const DXVA_ModeH264_D: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487719, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const DXVA_ModeH264_E: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487720, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const DXVA_ModeH264_F: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487721, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const DXVA_ModeH264_VLD_Multiview_NoFGT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1885052290, data2: 30415, data3: 18902, data4: [183, 230, 172, 136, 114, 219, 1, 60] };
pub const DXVA_ModeH264_VLD_Stereo_NoFGT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4188720315, data2: 49846, data3: 19708, data4: [135, 121, 87, 7, 177, 118, 5, 82] };
pub const DXVA_ModeH264_VLD_Stereo_Progressive_NoFGT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3617319130, data2: 3313, data3: 19585, data4: [184, 42, 105, 164, 226, 54, 244, 61] };
pub const DXVA_ModeH264_VLD_WithFMOASO_NoFGT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3589296121, data2: 13336, data3: 17880, data4: [149, 97, 50, 167, 106, 174, 45, 221] };
pub const DXVA_ModeHEVC_VLD_Main: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1527895323, data2: 12108, data3: 17490, data4: [188, 195, 9, 242, 161, 22, 12, 192] };
pub const DXVA_ModeHEVC_VLD_Main10: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 276492512, data2: 61210, data3: 19737, data4: [171, 168, 103, 161, 99, 7, 61, 19] };
pub const DXVA_ModeMPEG1_A: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487625, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const DXVA_ModeMPEG1_VLD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1866385177, data2: 14133, data3: 17100, data4: [128, 99, 101, 204, 60, 179, 102, 22] };
pub const DXVA_ModeMPEG2_A: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487626, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const DXVA_ModeMPEG2_B: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487627, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const DXVA_ModeMPEG2_C: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487628, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const DXVA_ModeMPEG2_D: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487629, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const DXVA_ModeMPEG2and1_VLD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2255052562, data2: 13326, data3: 20228, data4: [159, 211, 146, 83, 221, 50, 116, 96] };
pub const DXVA_ModeMPEG4pt2_VLD_AdvSimple_GMC: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2878966619, data2: 16984, data3: 17577, data4: [159, 235, 148, 229, 151, 166, 186, 174] };
pub const DXVA_ModeMPEG4pt2_VLD_AdvSimple_NoGMC: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3980495519, data2: 269, data3: 20186, data4: [154, 227, 154, 101, 53, 141, 141, 46] };
pub const DXVA_ModeMPEG4pt2_VLD_Simple: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4023799156, data2: 51688, data3: 16855, data4: [165, 233, 233, 176, 227, 159, 163, 25] };
pub const DXVA_ModeNone: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487616, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const DXVA_ModeVC1_A: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487776, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const DXVA_ModeVC1_B: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487777, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const DXVA_ModeVC1_C: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487778, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const DXVA_ModeVC1_D: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487779, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const DXVA_ModeVC1_D2010: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487780, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const DXVA_ModeVP8_VLD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2428017130, data2: 14946, data3: 18181, data4: [136, 179, 141, 240, 75, 39, 68, 231] };
pub const DXVA_ModeVP9_VLD_10bit_Profile2: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2764524015, data2: 28367, data3: 18602, data4: [132, 72, 80, 167, 161, 22, 95, 247] };
pub const DXVA_ModeVP9_VLD_Profile0: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1178011640, data2: 41424, data3: 17797, data4: [135, 109, 131, 170, 109, 96, 184, 158] };
pub const DXVA_ModeWMV8_A: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487744, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const DXVA_ModeWMV8_B: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487745, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const DXVA_ModeWMV9_A: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487760, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const DXVA_ModeWMV9_B: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487761, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const DXVA_ModeWMV9_C: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487764, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_NUM_TYPES_COMP_BUFFERS: u32 = 18u32;
pub const DXVA_NoEncrypt: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487824, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_NumMV_OBMC_off_BinPBwith4MV_off: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_NumMV_OBMC_off_BinPBwith4MV_on: u32 = 5u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_NumMV_OBMC_on__BinPB_off: u32 = 10u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_NumMV_OBMC_on__BinPB_on: u32 = 11u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_PICTURE_DECODE_BUFFER: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_PICTURE_DECODING_FUNCTION: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_PICTURE_RESAMPLE_BUFFER: u32 = 14u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_PICTURE_RESAMPLE_FUNCTION: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_PICTURE_STRUCTURE_BOTTOM_FIELD: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_PICTURE_STRUCTURE_FRAME: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_PICTURE_STRUCTURE_TOP_FIELD: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_ProcAmpControlBltFnCode: u32 = 1u32;
pub const DXVA_ProcAmpControlDevice: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2669676819, data2: 12285, data3: 16470, data4: [159, 30, 225, 181, 8, 242, 45, 207] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_ProcAmpControlQueryCapsFnCode: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_ProcAmpControlQueryRangeFnCode: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_QUERYORREPLYFUNCFLAG_ACCEL_LOCK_FALSE_PLUS: u32 = 16777215u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_QUERYORREPLYFUNCFLAG_ACCEL_LOCK_OK_COPY: u32 = 16777212u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_QUERYORREPLYFUNCFLAG_ACCEL_PROBE_FALSE_PLUS: u32 = 16777211u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_QUERYORREPLYFUNCFLAG_ACCEL_PROBE_OK_COPY: u32 = 16777208u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_QUERYORREPLYFUNCFLAG_ACCEL_PROBE_OK_PLUS: u32 = 16777209u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_QUERYORREPLYFUNCFLAG_DECODER_LOCK_QUERY: u32 = 16777205u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_QUERYORREPLYFUNCFLAG_DECODER_PROBE_QUERY: u32 = 16777201u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_READ_BACK_BUFFER: u32 = 15u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESIDUAL_DIFFERENCE_BUFFER: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_H261_A: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_H261_B: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_H263_A: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_H263_B: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_H263_C: u32 = 5u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_H263_D: u32 = 6u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_H263_E: u32 = 7u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_H263_F: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_H264_A: u32 = 100u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_H264_B: u32 = 101u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_H264_C: u32 = 102u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_H264_D: u32 = 103u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_H264_E: u32 = 104u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_H264_F: u32 = 105u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_H264_IDCT_FGT: u32 = 103u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_H264_IDCT_NOFGT: u32 = 102u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_H264_MOCOMP_FGT: u32 = 101u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_H264_MOCOMP_NOFGT: u32 = 100u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_H264_VLD_FGT: u32 = 105u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_H264_VLD_MULTIVIEW_NOFGT: u32 = 115u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_H264_VLD_NOFGT: u32 = 104u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_H264_VLD_STEREO_NOFGT: u32 = 114u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_H264_VLD_STEREO_PROGRESSIVE_NOFGT: u32 = 113u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_H264_VLD_WITHFMOASO_NOFGT: u32 = 112u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_MPEG1_A: u32 = 9u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_MPEG1_VLD: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_MPEG2_A: u32 = 10u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_MPEG2_B: u32 = 11u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_MPEG2_C: u32 = 12u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_MPEG2_D: u32 = 13u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_MPEG2and1_VLD: u32 = 17u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_MPEG4PT2_VLD_ADV_SIMPLE_GMC: u32 = 178u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_MPEG4PT2_VLD_ADV_SIMPLE_NOGMC: u32 = 177u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_MPEG4PT2_VLD_SIMPLE: u32 = 176u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_UNRESTRICTED: u32 = 65535u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_VC1_A: u32 = 160u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_VC1_B: u32 = 161u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_VC1_C: u32 = 162u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_VC1_D: u32 = 163u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_VC1_D2010: u32 = 164u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_VC1_IDCT: u32 = 162u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_VC1_MOCOMP: u32 = 161u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_VC1_POSTPROC: u32 = 160u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_VC1_VLD: u32 = 163u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_WMV8_A: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_WMV8_B: u32 = 129u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_WMV8_MOCOMP: u32 = 129u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_WMV8_POSTPROC: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_WMV9_A: u32 = 144u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_WMV9_B: u32 = 145u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_WMV9_C: u32 = 148u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_WMV9_IDCT: u32 = 148u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_WMV9_MOCOMP: u32 = 145u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_RESTRICTED_MODE_WMV9_POSTPROC: u32 = 144u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_SCAN_METHOD_ALTERNATE_HORIZONTAL: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_SCAN_METHOD_ALTERNATE_VERTICAL: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_SCAN_METHOD_ARBITRARY: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_SCAN_METHOD_ZIG_ZAG: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_SLICE_CONTROL_BUFFER: u32 = 6u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_STATUS_REPORTING_FUNCTION: u32 = 7u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_USUAL_BLOCK_HEIGHT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DXVA_USUAL_BLOCK_WIDTH: u32 = 8u32;
pub const DigitalCableLocator: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 62940182, data2: 53543, data3: 16506, data4: [171, 76, 253, 210, 121, 171, 190, 93] };
pub const DigitalCableTuneRequest: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 653003619, data2: 43664, data3: 17802, data4: [141, 244, 86, 89, 242, 200, 161, 138] };
pub const DigitalCableTuningSpace: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3652930798, data2: 47226, data3: 18417, data4: [172, 146, 176, 141, 156, 120, 19, 252] };
pub const DigitalLocator: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1850788877, data2: 49563, data3: 19446, data4: [129, 11, 91, 214, 7, 97, 245, 204] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DisplaySizeList = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dslDefaultSize: DisplaySizeList = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dslSourceSize: DisplaySizeList = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dslHalfSourceSize: DisplaySizeList = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dslDoubleSourceSize: DisplaySizeList = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dslFullScreen: DisplaySizeList = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dslHalfScreen: DisplaySizeList = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dslQuarterScreen: DisplaySizeList = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dslSixteenthScreen: DisplaySizeList = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type DownResEventParam = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DOWNRES_Always: DownResEventParam = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DOWNRES_InWindowOnly: DownResEventParam = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DOWNRES_Undefined: DownResEventParam = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct DualMonoInfo {
    pub LangID1: u16,
    pub LangID2: u16,
    pub lISOLangCode1: i32,
    pub lISOLangCode2: i32,
}
impl ::core::marker::Copy for DualMonoInfo {}
impl ::core::clone::Clone for DualMonoInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DvbParentalRatingDescriptor {
    pub ulNumParams: u32,
    pub pParams: [DvbParentalRatingParam; 1],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DvbParentalRatingDescriptor {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DvbParentalRatingDescriptor {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DvbParentalRatingParam {
    pub szCountryCode: [super::super::Foundation::CHAR; 4],
    pub bRating: u8,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DvbParentalRatingParam {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DvbParentalRatingParam {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct EALocationCodeType {
    pub LocationCodeScheme: LocationCodeSchemeType,
    pub state_code: u8,
    pub county_subdivision: u8,
    pub county_code: u16,
}
impl ::core::marker::Copy for EALocationCodeType {}
impl ::core::clone::Clone for EALocationCodeType {
    fn clone(&self) -> Self {
        *self
    }
}
pub const ECHOSTAR_SATELLITE_TV_NETWORK_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3304502043, data2: 50879, data3: 18265, data4: [136, 111, 167, 56, 109, 202, 39, 160] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_ACTIVATE: u32 = 19u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_BANDWIDTHCHANGE: u32 = 72u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_BUFFERING_DATA: u32 = 17u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_BUILT: u32 = 768u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_CLOCK_CHANGED: u32 = 13u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_CLOCK_UNSET: u32 = 81u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_CODECAPI_EVENT: u32 = 87u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_COMPLETE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_CONTENTPROPERTY_CHANGED: u32 = 71u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DEVICE_LOST: u32 = 31u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DISPLAY_CHANGED: u32 = 22u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVDBASE: u32 = 256u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_ANGLES_AVAILABLE: u32 = 275u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_ANGLE_CHANGE: u32 = 262u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_AUDIO_STREAM_CHANGE: u32 = 260u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_BUTTON_AUTO_ACTIVATED: u32 = 277u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_BUTTON_CHANGE: u32 = 263u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_BeginNavigationCommands: u32 = 291u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_CHAPTER_AUTOSTOP: u32 = 270u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_CHAPTER_START: u32 = 259u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_CMD_END: u32 = 279u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_CMD_START: u32 = 278u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_CURRENT_HMSF_TIME: u32 = 282u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_CURRENT_TIME: u32 = 267u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_DISC_EJECTED: u32 = 280u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_DISC_INSERTED: u32 = 281u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_DOMAIN_CHANGE: u32 = 257u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_ERROR: u32 = 268u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_GPRM_Change: u32 = 289u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_KARAOKE_MODE: u32 = 283u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_NO_FP_PGC: u32 = 271u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_NavigationCommand: u32 = 292u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_PARENTAL_LEVEL_CHANGE: u32 = 273u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_PLAYBACK_RATE_CHANGE: u32 = 272u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_PLAYBACK_STOPPED: u32 = 274u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_PLAYPERIOD_AUTOSTOP: u32 = 276u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_PROGRAM_CELL_CHANGE: u32 = 284u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_PROGRAM_CHAIN_CHANGE: u32 = 286u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_SPRM_Change: u32 = 290u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_STILL_OFF: u32 = 266u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_STILL_ON: u32 = 265u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_SUBPICTURE_STREAM_CHANGE: u32 = 261u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_TITLE_CHANGE: u32 = 258u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_TITLE_SET_CHANGE: u32 = 285u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_VALID_UOPS_CHANGE: u32 = 264u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_VOBU_Offset: u32 = 287u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_VOBU_Timestamp: u32 = 288u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_DVD_WARNING: u32 = 269u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_END_OF_SEGMENT: u32 = 28u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_EOS_SOON: u32 = 70u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_ERRORABORT: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_ERRORABORTEX: u32 = 69u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_ERROR_STILLPLAYING: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_EXTDEVICE_MODE_CHANGE: u32 = 49u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_FILE_CLOSED: u32 = 68u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_FULLSCREEN_LOST: u32 = 18u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_GRAPH_CHANGED: u32 = 80u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_LENGTH_CHANGED: u32 = 30u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_LOADSTATUS: u32 = 67u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_MARKER_HIT: u32 = 66u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_NEED_RESTART: u32 = 20u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_NEW_PIN: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_NOTIFY_WINDOW: u32 = 25u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_OLE_EVENT: u32 = 24u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_OPENING_FILE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_PALETTE_CHANGED: u32 = 9u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_PAUSED: u32 = 14u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_PLEASE_REOPEN: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_PREPROCESS_COMPLETE: u32 = 86u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_PROCESSING_LATENCY: u32 = 33u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_QUALITY_CHANGE: u32 = 11u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_RENDER_FINISHED: u32 = 33u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_REPAINT: u32 = 5u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_SAMPLE_LATENCY: u32 = 34u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_SAMPLE_NEEDED: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_SCRUB_TIME: u32 = 35u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_SEGMENT_STARTED: u32 = 29u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_SHUTTING_DOWN: u32 = 12u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_SKIP_FRAMES: u32 = 37u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_SNDDEV_IN_ERROR: u32 = 512u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_SNDDEV_OUT_ERROR: u32 = 513u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_SND_DEVICE_ERROR_BASE: u32 = 512u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_STARVATION: u32 = 23u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_STATE_CHANGE: u32 = 50u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_STATUS: u32 = 65u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_STEP_COMPLETE: u32 = 36u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_STREAM_CONTROL_STARTED: u32 = 27u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_STREAM_CONTROL_STOPPED: u32 = 26u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_STREAM_ERROR_STILLPLAYING: u32 = 7u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_STREAM_ERROR_STOPPED: u32 = 6u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_SYSTEMBASE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_TIME: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_TIMECODE_AVAILABLE: u32 = 48u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_UNBUILT: u32 = 769u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_USER: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_USERABORT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_VIDEOFRAMEREADY: u32 = 73u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_VIDEO_SIZE_CHANGED: u32 = 10u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_VMR_RECONNECTION_FAILED: u32 = 85u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_VMR_RENDERDEVICE_SET: u32 = 83u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_VMR_SURFACE_FLIPPED: u32 = 84u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_WINDOW_DESTROYED: u32 = 21u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_WMT_EVENT: u32 = 594u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_WMT_EVENT_BASE: u32 = 593u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EC_WMT_INDEX_EVENT: u32 = 593u32;
pub const ESEventFactory: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2391410650, data2: 29176, data3: 16577, data4: [169, 41, 94, 58, 134, 138, 194, 198] };
pub const ESEventService: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3255060476, data2: 60512, data3: 18270, data4: [129, 63, 210, 176, 166, 222, 206, 254] };
pub const ETFilter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229809, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
pub const EVENTID_ARIBcontentSpanningEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 982859907, data2: 37840, data3: 17982, data4: [144, 178, 7, 66, 196, 150, 237, 240] };
pub const EVENTID_AudioDescriptorSpanningEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 276550684, data2: 42714, data3: 18065, data4: [131, 105, 17, 178, 205, 170, 40, 142] };
pub const EVENTID_AudioTypeSpanningEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1344061374, data2: 47177, data3: 17102, data4: [155, 233, 61, 184, 105, 251, 130, 179] };
pub const EVENTID_BDAConditionalAccessTAG: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4022576217, data2: 44683, data3: 19274, data4: [143, 233, 121, 160, 208, 151, 243, 234] };
pub const EVENTID_BDAEventingServicePendingEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1554323217, data2: 24028, data3: 16806, data4: [148, 48, 228, 27, 139, 59, 188, 91] };
pub const EVENTID_BDA_CASBroadcastMMI: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1734899440, data2: 4402, data3: 16460, data4: [167, 202, 231, 32, 105, 169, 213, 79] };
pub const EVENTID_BDA_CASCloseMMI: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1561285903, data2: 56878, data3: 18333, data4: [131, 69, 236, 14, 149, 87, 232, 162] };
pub const EVENTID_BDA_CASOpenMMI: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2245708053, data2: 58771, data3: 16653, data4: [132, 113, 214, 129, 33, 5, 242, 142] };
pub const EVENTID_BDA_CASReleaseTuner: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 549560683, data2: 17439, data3: 18853, data4: [187, 92, 233, 160, 68, 149, 198, 193] };
pub const EVENTID_BDA_CASRequestTuner: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3476662744, data2: 62931, data3: 18053, data4: [190, 87, 237, 129, 219, 164, 107, 39] };
pub const EVENTID_BDA_DiseqCResponseAvailable: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4020644088, data2: 7980, data3: 19303, data4: [158, 165, 172, 246, 250, 154, 31, 54] };
pub const EVENTID_BDA_EncoderSignalLock: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1590234809, data2: 14842, data3: 19708, data4: [185, 63, 0, 187, 17, 7, 127, 94] };
pub const EVENTID_BDA_FdcStatus: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 99767142, data2: 53483, data3: 17362, data4: [188, 60, 104, 43, 134, 61, 241, 66] };
pub const EVENTID_BDA_FdcTableSection: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1779226455, data2: 19683, data3: 20059, data4: [148, 68, 113, 135, 184, 113, 82, 197] };
pub const EVENTID_BDA_GPNVValueUpdate: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4285908620, data2: 62486, data3: 20094, data4: [191, 23, 109, 85, 197, 223, 21, 117] };
pub const EVENTID_BDA_GuideDataAvailable: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2564518266, data2: 18314, data3: 19668, data4: [146, 208, 149, 246, 107, 137, 229, 177] };
pub const EVENTID_BDA_GuideDataError: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2889073736, data2: 28531, data3: 20439, data4: [179, 65, 89, 76, 54, 13, 141, 116] };
pub const EVENTID_BDA_GuideServiceInformationUpdated: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2713971243, data2: 5983, data3: 17496, data4: [183, 53, 80, 125, 34, 219, 35, 166] };
pub const EVENTID_BDA_IsdbCASResponse: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3570080102, data2: 16828, data3: 19693, data4: [154, 32, 253, 206, 172, 120, 247, 13] };
pub const EVENTID_BDA_LbigsCloseConnectionHandle: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3270544281, data2: 26095, data3: 17172, data4: [150, 113, 233, 157, 76, 206, 11, 174] };
pub const EVENTID_BDA_LbigsOpenConnection: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 895616946, data2: 28465, data3: 20144, data4: [162, 113, 179, 250, 107, 183, 104, 15] };
pub const EVENTID_BDA_LbigsSendData: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 287516539, data2: 61894, data3: 16724, data4: [139, 13, 72, 230, 21, 112, 89, 170] };
pub const EVENTID_BDA_RatingPinReset: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3336587456, data2: 50548, data3: 19494, data4: [188, 218, 47, 77, 53, 235, 94, 133] };
pub const EVENTID_BDA_TransprtStreamSelectorInfo: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3289358213, data2: 2512, data3: 18588, data4: [158, 156, 10, 187, 181, 105, 81, 176] };
pub const EVENTID_BDA_TunerNoSignal: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3801823275, data2: 7901, data3: 18736, data4: [188, 70, 104, 47, 215, 45, 45, 251] };
pub const EVENTID_BDA_TunerSignalLock: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 410183488, data2: 62835, data3: 17051, data4: [160, 14, 217, 193, 228, 8, 175, 9] };
pub const EVENTID_BDA_UpdateDrmStatus: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1705440897, data2: 5218, data3: 18235, data4: [136, 206, 203, 115, 20, 39, 189, 181] };
pub const EVENTID_BDA_UpdateScanState: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1433414480, data2: 31561, data3: 17080, data4: [168, 47, 74, 251, 105, 27, 6, 40] };
pub const EVENTID_CADenialCountChanged: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 711312680, data2: 8777, data3: 16496, data4: [172, 22, 0, 57, 12, 223, 178, 221] };
pub const EVENTID_CASFailureSpanningEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3940037038, data2: 21801, data3: 19743, data4: [175, 206, 13, 140, 209, 37, 125, 48] };
pub const EVENTID_CSDescriptorSpanningEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4024924633, data2: 38896, data3: 18310, data4: [128, 13, 149, 207, 80, 93, 220, 102] };
pub const EVENTID_CandidatePostTuneData: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2667762640, data2: 40710, data3: 17257, data4: [159, 30, 58, 214, 202, 25, 128, 126] };
pub const EVENTID_CardStatusChanged: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2724592362, data2: 63604, data3: 19256, data4: [159, 247, 197, 61, 2, 150, 153, 150] };
pub const EVENTID_ChannelChangeSpanningEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2422719973, data2: 19548, data3: 16901, data4: [134, 200, 122, 254, 32, 254, 30, 250] };
pub const EVENTID_ChannelInfoSpanningEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1106472320, data2: 16690, data3: 19650, data4: [177, 33, 1, 164, 50, 25, 216, 27] };
pub const EVENTID_ChannelTypeSpanningEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1923816785, data2: 34770, data3: 18587, data4: [186, 17, 14, 8, 220, 33, 2, 67] };
pub const EVENTID_CtxADescriptorSpanningEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 984916710, data2: 16967, data3: 19252, data4: [137, 108, 48, 175, 165, 210, 28, 36] };
pub const EVENTID_DFNWithNoActualAVData: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4117274622, data2: 22009, data3: 19379, data4: [150, 190, 174, 151, 28, 99, 186, 224] };
pub const EVENTID_DRMParingStatusChanged: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 591605, data2: 61649, data3: 16854, data4: [167, 223, 64, 40, 105, 118, 105, 246] };
pub const EVENTID_DRMParingStepComplete: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1529790328, data2: 46930, data3: 17440, data4: [180, 30, 164, 114, 220, 149, 130, 142] };
pub const EVENTID_DTFilterCOPPBlock: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229802, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
pub const EVENTID_DTFilterCOPPUnblock: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229800, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
pub const EVENTID_DTFilterDataFormatFailure: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229805, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
pub const EVENTID_DTFilterDataFormatOK: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229804, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
pub const EVENTID_DTFilterRatingChange: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229794, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
pub const EVENTID_DTFilterRatingsBlock: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229795, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
pub const EVENTID_DTFilterRatingsUnblock: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229796, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
pub const EVENTID_DTFilterXDSPacket: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229797, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
pub const EVENTID_DVBScramblingControlSpanningEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1272242628, data2: 37025, data3: 16649, data4: [130, 54, 39, 240, 14, 125, 204, 91] };
pub const EVENTID_DemultiplexerFilterDiscontinuity: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 370497392, data2: 44757, data3: 18268, data4: [187, 152, 149, 163, 48, 112, 223, 12] };
pub const EVENTID_DualMonoSpanningEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2846006102, data2: 43083, data3: 18572, data4: [137, 213, 13, 78, 118, 87, 200, 206] };
pub const EVENTID_DvbParentalRatingDescriptor: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 711435661, data2: 60581, data3: 20140, data4: [171, 203, 231, 52, 211, 119, 109, 10] };
pub const EVENTID_EASMessageReceived: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3507354069, data2: 49761, data3: 19333, data4: [158, 138, 81, 123, 50, 153, 202, 178] };
pub const EVENTID_ETDTFilterLicenseFailure: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229807, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
pub const EVENTID_ETDTFilterLicenseOK: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229806, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
pub const EVENTID_ETFilterCopyNever: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229808, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
pub const EVENTID_ETFilterCopyOnce: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229803, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
pub const EVENTID_ETFilterEncryptionOff: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229799, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
pub const EVENTID_ETFilterEncryptionOn: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229798, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
pub const EVENTID_EmmMessageSpanningEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1810891368, data2: 20350, data3: 17044, data4: [170, 135, 233, 233, 83, 228, 63, 20] };
pub const EVENTID_EncDecFilterError: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229801, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
pub const EVENTID_EncDecFilterEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1243301467, data2: 4025, data3: 16729, data4: [175, 189, 227, 48, 6, 160, 249, 244] };
pub const EVENTID_EntitlementChanged: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2423369053, data2: 9049, data3: 19605, data4: [134, 148, 175, 168, 29, 112, 191, 213] };
pub const EVENTID_FormatNotSupportedEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 615655434, data2: 45738, data3: 18295, data4: [191, 101, 99, 243, 94, 123, 2, 74] };
pub const EVENTID_LanguageSpanningEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3801245293, data2: 39938, data3: 17549, data4: [170, 141, 120, 26, 147, 253, 195, 149] };
pub const EVENTID_MMIMessage: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 86780335, data2: 2468, data3: 19347, data4: [137, 15, 189, 106, 52, 137, 104, 164] };
pub const EVENTID_NewSignalAcquired: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3363751213, data2: 52504, data3: 16458, data4: [160, 118, 192, 42, 39, 61, 61, 231] };
pub const EVENTID_PBDAParentalControlEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4182223493, data2: 64338, data3: 18664, data4: [185, 197, 225, 225, 244, 17, 165, 26] };
pub const EVENTID_PIDListSpanningEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1207734117, data2: 58043, data3: 17972, data4: [156, 239, 253, 191, 230, 38, 29, 92] };
pub const EVENTID_PSITable: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 463222531, data2: 54343, data3: 19990, data4: [151, 187, 1, 121, 159, 192, 49, 237] };
pub const EVENTID_RRTSpanningEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4140812532, data2: 55955, data3: 20271, data4: [191, 248, 186, 30, 230, 252, 163, 162] };
pub const EVENTID_SBE2RecControlStarted: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2305206430, data2: 63550, data3: 19470, data4: [188, 59, 191, 167, 100, 158, 4, 203] };
pub const EVENTID_SBE2RecControlStopped: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1162550984, data2: 3227, data3: 19626, data4: [177, 161, 30, 122, 38, 102, 246, 195] };
pub const EVENTID_STBChannelNumber: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 398776112, data2: 53488, data3: 16698, data4: [140, 153, 80, 4, 105, 222, 53, 173] };
pub const EVENTID_ServiceTerminated: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 169695516, data2: 57554, data3: 20366, data4: [137, 96, 35, 53, 190, 244, 92, 203] };
pub const EVENTID_SignalAndServiceStatusSpanningEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2154350027, data2: 15364, data3: 18731, data4: [180, 125, 3, 8, 130, 13, 206, 81] };
pub const EVENTID_SignalStatusChanged: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1839004402, data2: 28717, data3: 19201, data4: [141, 255, 104, 146, 173, 32, 209, 145] };
pub const EVENTID_StreamIDSpanningEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3404835688, data2: 57683, data3: 19777, data4: [166, 179, 167, 201, 152, 219, 117, 238] };
pub const EVENTID_StreamTypeSpanningEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2192518844, data2: 12454, data3: 16996, data4: [168, 11, 173, 46, 19, 114, 172, 96] };
pub const EVENTID_SubtitleSpanningEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1573830728, data2: 53433, data3: 16739, data4: [135, 44, 79, 50, 34, 59, 232, 138] };
pub const EVENTID_TeletextSpanningEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2509887824, data2: 24371, data3: 17943, data4: [175, 124, 30, 84, 181, 16, 218, 163] };
pub const EVENTID_TuneFailureEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3648161714, data2: 11773, data3: 17258, data4: [148, 133, 153, 215, 212, 171, 90, 105] };
pub const EVENTID_TuneFailureSpanningEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1871357013, data2: 24289, data3: 18603, data4: [162, 124, 76, 141, 112, 185, 174, 186] };
pub const EVENTID_TuningChanged: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2642305589, data2: 19325, data3: 16989, data4: [166, 209, 215, 23, 195, 59, 156, 76] };
pub const EVENTID_TuningChanging: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2199403523, data2: 49310, data3: 17860, data4: [167, 25, 128, 122, 148, 149, 43, 249] };
pub const EVENTID_XDSCodecDuplicateXDSRating: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229791, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
pub const EVENTID_XDSCodecNewXDSPacket: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229793, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
pub const EVENTID_XDSCodecNewXDSRating: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229792, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
pub const EVENTTYPE_CASDescrambleFailureEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2987556162, data2: 31717, data3: 20299, data4: [145, 48, 102, 121, 137, 159, 79, 75] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const E_PROP_ID_UNSUPPORTED: ::windows_sys::core::HRESULT = -2147023728i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const E_PROP_SET_UNSUPPORTED: ::windows_sys::core::HRESULT = -2147023726i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type EnTag_Mode = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EnTag_Remove: EnTag_Mode = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EnTag_Once: EnTag_Mode = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const EnTag_Repeat: EnTag_Mode = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type EnTvRat_CAE_TV = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CAE_TV_Exempt: EnTvRat_CAE_TV = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CAE_TV_C: EnTvRat_CAE_TV = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CAE_TV_C8: EnTvRat_CAE_TV = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CAE_TV_G: EnTvRat_CAE_TV = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CAE_TV_PG: EnTvRat_CAE_TV = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CAE_TV_14: EnTvRat_CAE_TV = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CAE_TV_18: EnTvRat_CAE_TV = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CAE_TV_Reserved: EnTvRat_CAE_TV = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type EnTvRat_CAF_TV = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CAF_TV_Exempt: EnTvRat_CAF_TV = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CAF_TV_G: EnTvRat_CAF_TV = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CAF_TV_8: EnTvRat_CAF_TV = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CAF_TV_13: EnTvRat_CAF_TV = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CAF_TV_16: EnTvRat_CAF_TV = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CAF_TV_18: EnTvRat_CAF_TV = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CAF_TV_Reserved6: EnTvRat_CAF_TV = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CAF_TV_Reserved: EnTvRat_CAF_TV = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type EnTvRat_GenericLevel = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TvRat_0: EnTvRat_GenericLevel = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TvRat_1: EnTvRat_GenericLevel = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TvRat_2: EnTvRat_GenericLevel = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TvRat_3: EnTvRat_GenericLevel = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TvRat_4: EnTvRat_GenericLevel = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TvRat_5: EnTvRat_GenericLevel = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TvRat_6: EnTvRat_GenericLevel = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TvRat_7: EnTvRat_GenericLevel = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TvRat_8: EnTvRat_GenericLevel = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TvRat_9: EnTvRat_GenericLevel = 9i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TvRat_10: EnTvRat_GenericLevel = 10i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TvRat_11: EnTvRat_GenericLevel = 11i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TvRat_12: EnTvRat_GenericLevel = 12i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TvRat_13: EnTvRat_GenericLevel = 13i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TvRat_14: EnTvRat_GenericLevel = 14i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TvRat_15: EnTvRat_GenericLevel = 15i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TvRat_16: EnTvRat_GenericLevel = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TvRat_17: EnTvRat_GenericLevel = 17i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TvRat_18: EnTvRat_GenericLevel = 18i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TvRat_19: EnTvRat_GenericLevel = 19i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TvRat_20: EnTvRat_GenericLevel = 20i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TvRat_21: EnTvRat_GenericLevel = 21i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TvRat_kLevels: EnTvRat_GenericLevel = 22i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TvRat_Unblock: EnTvRat_GenericLevel = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TvRat_LevelDontKnow: EnTvRat_GenericLevel = 255i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type EnTvRat_MPAA = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPAA_NotApplicable: EnTvRat_MPAA = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPAA_G: EnTvRat_MPAA = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPAA_PG: EnTvRat_MPAA = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPAA_PG13: EnTvRat_MPAA = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPAA_R: EnTvRat_MPAA = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPAA_NC17: EnTvRat_MPAA = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPAA_X: EnTvRat_MPAA = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPAA_NotRated: EnTvRat_MPAA = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type EnTvRat_System = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPAA: EnTvRat_System = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const US_TV: EnTvRat_System = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const Canadian_English: EnTvRat_System = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const Canadian_French: EnTvRat_System = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const Reserved4: EnTvRat_System = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const System5: EnTvRat_System = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const System6: EnTvRat_System = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const Reserved7: EnTvRat_System = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PBDA: EnTvRat_System = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AgeBased: EnTvRat_System = 9i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TvRat_kSystems: EnTvRat_System = 10i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TvRat_SystemDontKnow: EnTvRat_System = 255i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type EnTvRat_US_TV = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const US_TV_None: EnTvRat_US_TV = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const US_TV_Y: EnTvRat_US_TV = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const US_TV_Y7: EnTvRat_US_TV = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const US_TV_G: EnTvRat_US_TV = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const US_TV_PG: EnTvRat_US_TV = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const US_TV_14: EnTvRat_US_TV = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const US_TV_MA: EnTvRat_US_TV = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const US_TV_None7: EnTvRat_US_TV = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type EncDecEvents = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ENCDEC_CPEVENT: EncDecEvents = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ENCDEC_RECORDING_STATUS: EncDecEvents = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type EntitlementType = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const Entitled: EntitlementType = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const NotEntitled: EntitlementType = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TechnicalFailure: EntitlementType = 2i32;
pub const EvalRat: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3318072817, data2: 15036, data3: 4566, data4: [178, 91, 0, 192, 79, 160, 192, 38] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type FECMethod = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_FEC_METHOD_NOT_SET: FECMethod = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_FEC_METHOD_NOT_DEFINED: FECMethod = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_FEC_VITERBI: FECMethod = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_FEC_RS_204_188: FECMethod = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_FEC_LDPC: FECMethod = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_FEC_BCH: FECMethod = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_FEC_RS_147_130: FECMethod = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_FEC_MAX: FECMethod = 6i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct FILTER_INFO {
    pub achName: [u16; 128],
    pub pGraph: IFilterGraph,
}
impl ::core::marker::Copy for FILTER_INFO {}
impl ::core::clone::Clone for FILTER_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type FILTER_STATE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const State_Stopped: FILTER_STATE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const State_Paused: FILTER_STATE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const State_Running: FILTER_STATE = 2i32;
pub const FORMATTYPE_CPFilters_Processed: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1731834735, data2: 7519, data3: 19138, data4: [129, 146, 40, 187, 14, 115, 209, 106] };
pub const FORMATTYPE_ETDTFilter_Tagged: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229777, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
pub const FilgraphManager: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3828804531, data2: 21071, data3: 4558, data4: [159, 83, 0, 32, 175, 11, 167, 112] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type FormatNotSupportedEvents = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const FORMATNOTSUPPORTED_CLEAR: FormatNotSupportedEvents = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const FORMATNOTSUPPORTED_NOTSUPPORTED: FormatNotSupportedEvents = 1i32;
pub const GUID_TIME_MUSIC: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 91538589, data2: 23300, data3: 19221, data4: [165, 66, 174, 40, 32, 48, 17, 123] };
pub const GUID_TIME_REFERENCE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2477617451, data2: 55968, data3: 20478, data4: [188, 129, 176, 206, 80, 15, 205, 217] };
pub const GUID_TIME_SAMPLES: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2824420613, data2: 3139, data3: 18820, data4: [154, 99, 151, 175, 158, 2, 196, 192] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type GuardInterval = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_GUARD_NOT_SET: GuardInterval = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_GUARD_NOT_DEFINED: GuardInterval = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_GUARD_1_32: GuardInterval = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_GUARD_1_16: GuardInterval = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_GUARD_1_8: GuardInterval = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_GUARD_1_4: GuardInterval = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_GUARD_1_128: GuardInterval = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_GUARD_19_128: GuardInterval = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_GUARD_19_256: GuardInterval = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_GUARD_MAX: GuardInterval = 8i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_Audio\"`*"]
#[cfg(feature = "Win32_Media_Audio")]
pub struct HEAACWAVEFORMAT {
    pub wfInfo: HEAACWAVEINFO,
    pub pbAudioSpecificConfig: [u8; 1],
}
#[cfg(feature = "Win32_Media_Audio")]
impl ::core::marker::Copy for HEAACWAVEFORMAT {}
#[cfg(feature = "Win32_Media_Audio")]
impl ::core::clone::Clone for HEAACWAVEFORMAT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_Audio\"`*"]
#[cfg(feature = "Win32_Media_Audio")]
pub struct HEAACWAVEINFO {
    pub wfx: super::Audio::WAVEFORMATEX,
    pub wPayloadType: u16,
    pub wAudioProfileLevelIndication: u16,
    pub wStructType: u16,
    pub wReserved1: u16,
    pub dwReserved2: u32,
}
#[cfg(feature = "Win32_Media_Audio")]
impl ::core::marker::Copy for HEAACWAVEINFO {}
#[cfg(feature = "Win32_Media_Audio")]
impl ::core::clone::Clone for HEAACWAVEINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type HierarchyAlpha = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_HALPHA_NOT_SET: HierarchyAlpha = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_HALPHA_NOT_DEFINED: HierarchyAlpha = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_HALPHA_1: HierarchyAlpha = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_HALPHA_2: HierarchyAlpha = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_HALPHA_4: HierarchyAlpha = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_HALPHA_MAX: HierarchyAlpha = 4i32;
pub type IAMAnalogVideoDecoder = *mut ::core::ffi::c_void;
pub type IAMAnalogVideoEncoder = *mut ::core::ffi::c_void;
pub type IAMAsyncReaderTimestampScaling = *mut ::core::ffi::c_void;
pub type IAMAudioInputMixer = *mut ::core::ffi::c_void;
pub type IAMAudioRendererStats = *mut ::core::ffi::c_void;
pub type IAMBufferNegotiation = *mut ::core::ffi::c_void;
pub type IAMCameraControl = *mut ::core::ffi::c_void;
pub type IAMCertifiedOutputProtection = *mut ::core::ffi::c_void;
pub type IAMChannelInfo = *mut ::core::ffi::c_void;
pub type IAMClockAdjust = *mut ::core::ffi::c_void;
pub type IAMClockSlave = *mut ::core::ffi::c_void;
pub type IAMCollection = *mut ::core::ffi::c_void;
pub type IAMCopyCaptureFileProgress = *mut ::core::ffi::c_void;
pub type IAMCrossbar = *mut ::core::ffi::c_void;
pub type IAMDecoderCaps = *mut ::core::ffi::c_void;
pub type IAMDevMemoryAllocator = *mut ::core::ffi::c_void;
pub type IAMDevMemoryControl = *mut ::core::ffi::c_void;
pub type IAMDeviceRemoval = *mut ::core::ffi::c_void;
pub type IAMDirectSound = *mut ::core::ffi::c_void;
pub type IAMDroppedFrames = *mut ::core::ffi::c_void;
pub type IAMExtDevice = *mut ::core::ffi::c_void;
pub type IAMExtTransport = *mut ::core::ffi::c_void;
pub type IAMExtendedErrorInfo = *mut ::core::ffi::c_void;
pub type IAMExtendedSeeking = *mut ::core::ffi::c_void;
pub type IAMFilterGraphCallback = *mut ::core::ffi::c_void;
pub type IAMFilterMiscFlags = *mut ::core::ffi::c_void;
pub type IAMGraphBuilderCallback = *mut ::core::ffi::c_void;
pub type IAMGraphStreams = *mut ::core::ffi::c_void;
pub type IAMLatency = *mut ::core::ffi::c_void;
pub type IAMLine21Decoder = *mut ::core::ffi::c_void;
pub type IAMMediaContent = *mut ::core::ffi::c_void;
pub type IAMMediaContent2 = *mut ::core::ffi::c_void;
pub type IAMMediaStream = *mut ::core::ffi::c_void;
pub type IAMMediaTypeSample = *mut ::core::ffi::c_void;
pub type IAMMediaTypeStream = *mut ::core::ffi::c_void;
pub type IAMMultiMediaStream = *mut ::core::ffi::c_void;
pub type IAMNetShowConfig = *mut ::core::ffi::c_void;
pub type IAMNetShowExProps = *mut ::core::ffi::c_void;
pub type IAMNetShowPreroll = *mut ::core::ffi::c_void;
pub type IAMNetworkStatus = *mut ::core::ffi::c_void;
pub type IAMOpenProgress = *mut ::core::ffi::c_void;
pub type IAMOverlayFX = *mut ::core::ffi::c_void;
pub type IAMParse = *mut ::core::ffi::c_void;
pub type IAMPhysicalPinInfo = *mut ::core::ffi::c_void;
pub type IAMPlayList = *mut ::core::ffi::c_void;
pub type IAMPlayListItem = *mut ::core::ffi::c_void;
pub type IAMPluginControl = *mut ::core::ffi::c_void;
pub type IAMPushSource = *mut ::core::ffi::c_void;
pub type IAMRebuild = *mut ::core::ffi::c_void;
pub type IAMResourceControl = *mut ::core::ffi::c_void;
pub type IAMStats = *mut ::core::ffi::c_void;
pub type IAMStreamConfig = *mut ::core::ffi::c_void;
pub type IAMStreamControl = *mut ::core::ffi::c_void;
pub type IAMStreamSelect = *mut ::core::ffi::c_void;
pub type IAMTVAudio = *mut ::core::ffi::c_void;
pub type IAMTVAudioNotification = *mut ::core::ffi::c_void;
pub type IAMTVTuner = *mut ::core::ffi::c_void;
pub type IAMTimecodeDisplay = *mut ::core::ffi::c_void;
pub type IAMTimecodeGenerator = *mut ::core::ffi::c_void;
pub type IAMTimecodeReader = *mut ::core::ffi::c_void;
pub type IAMTuner = *mut ::core::ffi::c_void;
pub type IAMTunerNotification = *mut ::core::ffi::c_void;
pub type IAMVfwCaptureDialogs = *mut ::core::ffi::c_void;
pub type IAMVfwCompressDialogs = *mut ::core::ffi::c_void;
pub type IAMVideoAccelerator = *mut ::core::ffi::c_void;
pub type IAMVideoAcceleratorNotify = *mut ::core::ffi::c_void;
pub type IAMVideoCompression = *mut ::core::ffi::c_void;
pub type IAMVideoControl = *mut ::core::ffi::c_void;
pub type IAMVideoDecimationProperties = *mut ::core::ffi::c_void;
pub type IAMVideoProcAmp = *mut ::core::ffi::c_void;
pub type IAMWstDecoder = *mut ::core::ffi::c_void;
pub type IAMovieSetup = *mut ::core::ffi::c_void;
pub type IATSCChannelTuneRequest = *mut ::core::ffi::c_void;
pub type IATSCComponentType = *mut ::core::ffi::c_void;
pub type IATSCLocator = *mut ::core::ffi::c_void;
pub type IATSCLocator2 = *mut ::core::ffi::c_void;
pub type IATSCTuningSpace = *mut ::core::ffi::c_void;
pub type IATSC_EIT = *mut ::core::ffi::c_void;
pub type IATSC_ETT = *mut ::core::ffi::c_void;
pub type IATSC_MGT = *mut ::core::ffi::c_void;
pub type IATSC_STT = *mut ::core::ffi::c_void;
pub type IATSC_VCT = *mut ::core::ffi::c_void;
pub type IAnalogAudioComponentType = *mut ::core::ffi::c_void;
pub type IAnalogLocator = *mut ::core::ffi::c_void;
pub type IAnalogRadioTuningSpace = *mut ::core::ffi::c_void;
pub type IAnalogRadioTuningSpace2 = *mut ::core::ffi::c_void;
pub type IAnalogTVTuningSpace = *mut ::core::ffi::c_void;
pub type IAsyncReader = *mut ::core::ffi::c_void;
pub type IAtscContentAdvisoryDescriptor = *mut ::core::ffi::c_void;
pub type IAtscPsipParser = *mut ::core::ffi::c_void;
pub type IAttributeGet = *mut ::core::ffi::c_void;
pub type IAttributeSet = *mut ::core::ffi::c_void;
pub type IAudioData = *mut ::core::ffi::c_void;
pub type IAudioMediaStream = *mut ::core::ffi::c_void;
pub type IAudioStreamSample = *mut ::core::ffi::c_void;
pub type IAuxInTuningSpace = *mut ::core::ffi::c_void;
pub type IAuxInTuningSpace2 = *mut ::core::ffi::c_void;
pub type IBDAComparable = *mut ::core::ffi::c_void;
pub type IBDACreateTuneRequestEx = *mut ::core::ffi::c_void;
pub type IBDA_AUX = *mut ::core::ffi::c_void;
pub type IBDA_AutoDemodulate = *mut ::core::ffi::c_void;
pub type IBDA_AutoDemodulateEx = *mut ::core::ffi::c_void;
pub type IBDA_ConditionalAccess = *mut ::core::ffi::c_void;
pub type IBDA_ConditionalAccessEx = *mut ::core::ffi::c_void;
pub type IBDA_DRIDRMService = *mut ::core::ffi::c_void;
pub type IBDA_DRIWMDRMSession = *mut ::core::ffi::c_void;
pub type IBDA_DRM = *mut ::core::ffi::c_void;
pub type IBDA_DRMService = *mut ::core::ffi::c_void;
pub type IBDA_DeviceControl = *mut ::core::ffi::c_void;
pub type IBDA_DiagnosticProperties = *mut ::core::ffi::c_void;
pub type IBDA_DigitalDemodulator = *mut ::core::ffi::c_void;
pub type IBDA_DigitalDemodulator2 = *mut ::core::ffi::c_void;
pub type IBDA_DigitalDemodulator3 = *mut ::core::ffi::c_void;
pub type IBDA_DiseqCommand = *mut ::core::ffi::c_void;
pub type IBDA_EasMessage = *mut ::core::ffi::c_void;
pub type IBDA_Encoder = *mut ::core::ffi::c_void;
pub type IBDA_EthernetFilter = *mut ::core::ffi::c_void;
pub type IBDA_EventingService = *mut ::core::ffi::c_void;
pub type IBDA_FDC = *mut ::core::ffi::c_void;
pub type IBDA_FrequencyFilter = *mut ::core::ffi::c_void;
pub type IBDA_GuideDataDeliveryService = *mut ::core::ffi::c_void;
pub type IBDA_IPSinkControl = *mut ::core::ffi::c_void;
pub type IBDA_IPSinkInfo = *mut ::core::ffi::c_void;
pub type IBDA_IPV4Filter = *mut ::core::ffi::c_void;
pub type IBDA_IPV6Filter = *mut ::core::ffi::c_void;
pub type IBDA_ISDBConditionalAccess = *mut ::core::ffi::c_void;
pub type IBDA_LNBInfo = *mut ::core::ffi::c_void;
pub type IBDA_MUX = *mut ::core::ffi::c_void;
pub type IBDA_NameValueService = *mut ::core::ffi::c_void;
pub type IBDA_NetworkProvider = *mut ::core::ffi::c_void;
pub type IBDA_NullTransform = *mut ::core::ffi::c_void;
pub type IBDA_PinControl = *mut ::core::ffi::c_void;
pub type IBDA_SignalProperties = *mut ::core::ffi::c_void;
pub type IBDA_SignalStatistics = *mut ::core::ffi::c_void;
pub type IBDA_TIF_REGISTRATION = *mut ::core::ffi::c_void;
pub type IBDA_Topology = *mut ::core::ffi::c_void;
pub type IBDA_TransportStreamInfo = *mut ::core::ffi::c_void;
pub type IBDA_TransportStreamSelector = *mut ::core::ffi::c_void;
pub type IBDA_UserActivityService = *mut ::core::ffi::c_void;
pub type IBDA_VoidTransform = *mut ::core::ffi::c_void;
pub type IBDA_WMDRMSession = *mut ::core::ffi::c_void;
pub type IBDA_WMDRMTuner = *mut ::core::ffi::c_void;
pub type IBPCSatelliteTuner = *mut ::core::ffi::c_void;
pub type IBaseFilter = *mut ::core::ffi::c_void;
pub type IBaseVideoMixer = *mut ::core::ffi::c_void;
pub type IBasicAudio = *mut ::core::ffi::c_void;
pub type IBasicVideo = *mut ::core::ffi::c_void;
pub type IBasicVideo2 = *mut ::core::ffi::c_void;
pub type IBroadcastEvent = *mut ::core::ffi::c_void;
pub type IBroadcastEventEx = *mut ::core::ffi::c_void;
pub type IBufferingTime = *mut ::core::ffi::c_void;
pub type ICAT = *mut ::core::ffi::c_void;
pub type ICCSubStreamFiltering = *mut ::core::ffi::c_void;
pub type ICameraControl = *mut ::core::ffi::c_void;
pub type ICaptionServiceDescriptor = *mut ::core::ffi::c_void;
pub type ICaptureGraphBuilder = *mut ::core::ffi::c_void;
pub type ICaptureGraphBuilder2 = *mut ::core::ffi::c_void;
pub type IChannelIDTuneRequest = *mut ::core::ffi::c_void;
pub type IChannelTuneRequest = *mut ::core::ffi::c_void;
pub type IComponent = *mut ::core::ffi::c_void;
pub type IComponentType = *mut ::core::ffi::c_void;
pub type IComponentTypes = *mut ::core::ffi::c_void;
pub type IComponents = *mut ::core::ffi::c_void;
pub type IComponentsOld = *mut ::core::ffi::c_void;
pub type IConfigAsfWriter = *mut ::core::ffi::c_void;
pub type IConfigAsfWriter2 = *mut ::core::ffi::c_void;
pub type IConfigAviMux = *mut ::core::ffi::c_void;
pub type IConfigInterleaving = *mut ::core::ffi::c_void;
pub type ICreateDevEnum = *mut ::core::ffi::c_void;
pub type ICreatePropBagOnRegKey = *mut ::core::ffi::c_void;
pub type IDDrawExclModeVideo = *mut ::core::ffi::c_void;
pub type IDDrawExclModeVideoCallback = *mut ::core::ffi::c_void;
pub type IDMOWrapperFilter = *mut ::core::ffi::c_void;
pub type IDShowPlugin = *mut ::core::ffi::c_void;
pub type IDTFilter = *mut ::core::ffi::c_void;
pub type IDTFilter2 = *mut ::core::ffi::c_void;
pub type IDTFilter3 = *mut ::core::ffi::c_void;
pub type IDTFilterConfig = *mut ::core::ffi::c_void;
pub type IDTFilterEvents = *mut ::core::ffi::c_void;
pub type IDTFilterLicenseRenewal = *mut ::core::ffi::c_void;
pub type IDVBCLocator = *mut ::core::ffi::c_void;
pub type IDVBSLocator = *mut ::core::ffi::c_void;
pub type IDVBSLocator2 = *mut ::core::ffi::c_void;
pub type IDVBSTuningSpace = *mut ::core::ffi::c_void;
pub type IDVBTLocator = *mut ::core::ffi::c_void;
pub type IDVBTLocator2 = *mut ::core::ffi::c_void;
pub type IDVBTuneRequest = *mut ::core::ffi::c_void;
pub type IDVBTuningSpace = *mut ::core::ffi::c_void;
pub type IDVBTuningSpace2 = *mut ::core::ffi::c_void;
pub type IDVB_BAT = *mut ::core::ffi::c_void;
pub type IDVB_DIT = *mut ::core::ffi::c_void;
pub type IDVB_EIT = *mut ::core::ffi::c_void;
pub type IDVB_EIT2 = *mut ::core::ffi::c_void;
pub type IDVB_NIT = *mut ::core::ffi::c_void;
pub type IDVB_RST = *mut ::core::ffi::c_void;
pub type IDVB_SDT = *mut ::core::ffi::c_void;
pub type IDVB_SIT = *mut ::core::ffi::c_void;
pub type IDVB_ST = *mut ::core::ffi::c_void;
pub type IDVB_TDT = *mut ::core::ffi::c_void;
pub type IDVB_TOT = *mut ::core::ffi::c_void;
pub type IDVEnc = *mut ::core::ffi::c_void;
pub type IDVRGB219 = *mut ::core::ffi::c_void;
pub type IDVSplitter = *mut ::core::ffi::c_void;
pub type IDecimateVideoImage = *mut ::core::ffi::c_void;
pub type IDeferredCommand = *mut ::core::ffi::c_void;
pub type IDigitalCableLocator = *mut ::core::ffi::c_void;
pub type IDigitalCableTuneRequest = *mut ::core::ffi::c_void;
pub type IDigitalCableTuningSpace = *mut ::core::ffi::c_void;
pub type IDigitalLocator = *mut ::core::ffi::c_void;
pub type IDirectDrawMediaSample = *mut ::core::ffi::c_void;
pub type IDirectDrawMediaSampleAllocator = *mut ::core::ffi::c_void;
pub type IDirectDrawMediaStream = *mut ::core::ffi::c_void;
pub type IDirectDrawStreamSample = *mut ::core::ffi::c_void;
pub type IDirectDrawVideo = *mut ::core::ffi::c_void;
pub type IDistributorNotify = *mut ::core::ffi::c_void;
pub type IDrawVideoImage = *mut ::core::ffi::c_void;
pub type IDvbCableDeliverySystemDescriptor = *mut ::core::ffi::c_void;
pub type IDvbComponentDescriptor = *mut ::core::ffi::c_void;
pub type IDvbContentDescriptor = *mut ::core::ffi::c_void;
pub type IDvbContentIdentifierDescriptor = *mut ::core::ffi::c_void;
pub type IDvbDataBroadcastDescriptor = *mut ::core::ffi::c_void;
pub type IDvbDataBroadcastIDDescriptor = *mut ::core::ffi::c_void;
pub type IDvbDefaultAuthorityDescriptor = *mut ::core::ffi::c_void;
pub type IDvbExtendedEventDescriptor = *mut ::core::ffi::c_void;
pub type IDvbFrequencyListDescriptor = *mut ::core::ffi::c_void;
pub type IDvbHDSimulcastLogicalChannelDescriptor = *mut ::core::ffi::c_void;
pub type IDvbLinkageDescriptor = *mut ::core::ffi::c_void;
pub type IDvbLogicalChannel2Descriptor = *mut ::core::ffi::c_void;
pub type IDvbLogicalChannelDescriptor = *mut ::core::ffi::c_void;
pub type IDvbLogicalChannelDescriptor2 = *mut ::core::ffi::c_void;
pub type IDvbMultilingualServiceNameDescriptor = *mut ::core::ffi::c_void;
pub type IDvbNetworkNameDescriptor = *mut ::core::ffi::c_void;
pub type IDvbParentalRatingDescriptor = *mut ::core::ffi::c_void;
pub type IDvbPrivateDataSpecifierDescriptor = *mut ::core::ffi::c_void;
pub type IDvbSatelliteDeliverySystemDescriptor = *mut ::core::ffi::c_void;
pub type IDvbServiceAttributeDescriptor = *mut ::core::ffi::c_void;
pub type IDvbServiceDescriptor = *mut ::core::ffi::c_void;
pub type IDvbServiceDescriptor2 = *mut ::core::ffi::c_void;
pub type IDvbServiceListDescriptor = *mut ::core::ffi::c_void;
pub type IDvbShortEventDescriptor = *mut ::core::ffi::c_void;
pub type IDvbSiParser = *mut ::core::ffi::c_void;
pub type IDvbSiParser2 = *mut ::core::ffi::c_void;
pub type IDvbSubtitlingDescriptor = *mut ::core::ffi::c_void;
pub type IDvbTeletextDescriptor = *mut ::core::ffi::c_void;
pub type IDvbTerrestrial2DeliverySystemDescriptor = *mut ::core::ffi::c_void;
pub type IDvbTerrestrialDeliverySystemDescriptor = *mut ::core::ffi::c_void;
pub type IDvdCmd = *mut ::core::ffi::c_void;
pub type IDvdControl = *mut ::core::ffi::c_void;
pub type IDvdControl2 = *mut ::core::ffi::c_void;
pub type IDvdGraphBuilder = *mut ::core::ffi::c_void;
pub type IDvdInfo = *mut ::core::ffi::c_void;
pub type IDvdInfo2 = *mut ::core::ffi::c_void;
pub type IDvdState = *mut ::core::ffi::c_void;
pub type IESCloseMmiEvent = *mut ::core::ffi::c_void;
pub type IESEvent = *mut ::core::ffi::c_void;
pub type IESEventFactory = *mut ::core::ffi::c_void;
pub type IESEventService = *mut ::core::ffi::c_void;
pub type IESEventServiceConfiguration = *mut ::core::ffi::c_void;
pub type IESEvents = *mut ::core::ffi::c_void;
pub type IESFileExpiryDateEvent = *mut ::core::ffi::c_void;
pub type IESIsdbCasResponseEvent = *mut ::core::ffi::c_void;
pub type IESLicenseRenewalResultEvent = *mut ::core::ffi::c_void;
pub type IESOpenMmiEvent = *mut ::core::ffi::c_void;
pub type IESRequestTunerEvent = *mut ::core::ffi::c_void;
pub type IESValueUpdatedEvent = *mut ::core::ffi::c_void;
pub type IETFilter = *mut ::core::ffi::c_void;
pub type IETFilterConfig = *mut ::core::ffi::c_void;
pub type IETFilterEvents = *mut ::core::ffi::c_void;
pub type IEncoderAPI = *mut ::core::ffi::c_void;
pub type IEnumComponentTypes = *mut ::core::ffi::c_void;
pub type IEnumComponents = *mut ::core::ffi::c_void;
pub type IEnumFilters = *mut ::core::ffi::c_void;
pub type IEnumGuideDataProperties = *mut ::core::ffi::c_void;
pub type IEnumMSVidGraphSegment = *mut ::core::ffi::c_void;
pub type IEnumMediaTypes = *mut ::core::ffi::c_void;
pub type IEnumPIDMap = *mut ::core::ffi::c_void;
pub type IEnumPins = *mut ::core::ffi::c_void;
pub type IEnumRegFilters = *mut ::core::ffi::c_void;
pub type IEnumStreamBufferRecordingAttrib = *mut ::core::ffi::c_void;
pub type IEnumStreamIdMap = *mut ::core::ffi::c_void;
pub type IEnumTuneRequests = *mut ::core::ffi::c_void;
pub type IEnumTuningSpaces = *mut ::core::ffi::c_void;
pub type IEvalRat = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type IFILTERMAPPER_MERIT = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MERIT_PREFERRED: IFILTERMAPPER_MERIT = 8388608i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MERIT_NORMAL: IFILTERMAPPER_MERIT = 6291456i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MERIT_UNLIKELY: IFILTERMAPPER_MERIT = 4194304i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MERIT_DO_NOT_USE: IFILTERMAPPER_MERIT = 2097152i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MERIT_SW_COMPRESSOR: IFILTERMAPPER_MERIT = 1048576i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MERIT_HW_COMPRESSOR: IFILTERMAPPER_MERIT = 1048656i32;
pub type IFileSinkFilter = *mut ::core::ffi::c_void;
pub type IFileSinkFilter2 = *mut ::core::ffi::c_void;
pub type IFileSourceFilter = *mut ::core::ffi::c_void;
pub type IFilterChain = *mut ::core::ffi::c_void;
pub type IFilterGraph = *mut ::core::ffi::c_void;
pub type IFilterGraph2 = *mut ::core::ffi::c_void;
pub type IFilterGraph3 = *mut ::core::ffi::c_void;
pub type IFilterInfo = *mut ::core::ffi::c_void;
pub type IFilterMapper = *mut ::core::ffi::c_void;
pub type IFilterMapper2 = *mut ::core::ffi::c_void;
pub type IFilterMapper3 = *mut ::core::ffi::c_void;
pub type IFrequencyMap = *mut ::core::ffi::c_void;
pub type IFullScreenVideo = *mut ::core::ffi::c_void;
pub type IFullScreenVideoEx = *mut ::core::ffi::c_void;
pub type IGenericDescriptor = *mut ::core::ffi::c_void;
pub type IGenericDescriptor2 = *mut ::core::ffi::c_void;
pub type IGetCapabilitiesKey = *mut ::core::ffi::c_void;
pub type IGpnvsCommonBase = *mut ::core::ffi::c_void;
pub type IGraphBuilder = *mut ::core::ffi::c_void;
pub type IGraphConfig = *mut ::core::ffi::c_void;
pub type IGraphConfigCallback = *mut ::core::ffi::c_void;
pub type IGraphVersion = *mut ::core::ffi::c_void;
pub type IGuideData = *mut ::core::ffi::c_void;
pub type IGuideDataEvent = *mut ::core::ffi::c_void;
pub type IGuideDataLoader = *mut ::core::ffi::c_void;
pub type IGuideDataProperty = *mut ::core::ffi::c_void;
pub type IIPDVDec = *mut ::core::ffi::c_void;
pub type IISDBSLocator = *mut ::core::ffi::c_void;
pub type IISDB_BIT = *mut ::core::ffi::c_void;
pub type IISDB_CDT = *mut ::core::ffi::c_void;
pub type IISDB_EMM = *mut ::core::ffi::c_void;
pub type IISDB_LDT = *mut ::core::ffi::c_void;
pub type IISDB_NBIT = *mut ::core::ffi::c_void;
pub type IISDB_SDT = *mut ::core::ffi::c_void;
pub type IISDB_SDTT = *mut ::core::ffi::c_void;
pub type IIsdbAudioComponentDescriptor = *mut ::core::ffi::c_void;
pub type IIsdbCAContractInformationDescriptor = *mut ::core::ffi::c_void;
pub type IIsdbCADescriptor = *mut ::core::ffi::c_void;
pub type IIsdbCAServiceDescriptor = *mut ::core::ffi::c_void;
pub type IIsdbComponentGroupDescriptor = *mut ::core::ffi::c_void;
pub type IIsdbDataContentDescriptor = *mut ::core::ffi::c_void;
pub type IIsdbDigitalCopyControlDescriptor = *mut ::core::ffi::c_void;
pub type IIsdbDownloadContentDescriptor = *mut ::core::ffi::c_void;
pub type IIsdbEmergencyInformationDescriptor = *mut ::core::ffi::c_void;
pub type IIsdbEventGroupDescriptor = *mut ::core::ffi::c_void;
pub type IIsdbHierarchicalTransmissionDescriptor = *mut ::core::ffi::c_void;
pub type IIsdbLogoTransmissionDescriptor = *mut ::core::ffi::c_void;
pub type IIsdbSIParameterDescriptor = *mut ::core::ffi::c_void;
pub type IIsdbSeriesDescriptor = *mut ::core::ffi::c_void;
pub type IIsdbSiParser2 = *mut ::core::ffi::c_void;
pub type IIsdbTSInformationDescriptor = *mut ::core::ffi::c_void;
pub type IIsdbTerrestrialDeliverySystemDescriptor = *mut ::core::ffi::c_void;
pub type IKsNodeControl = *mut ::core::ffi::c_void;
pub type IKsTopologyInfo = *mut ::core::ffi::c_void;
pub type ILanguageComponentType = *mut ::core::ffi::c_void;
pub type ILocator = *mut ::core::ffi::c_void;
pub type IMPEG2Component = *mut ::core::ffi::c_void;
pub type IMPEG2ComponentType = *mut ::core::ffi::c_void;
pub type IMPEG2PIDMap = *mut ::core::ffi::c_void;
pub type IMPEG2StreamIdMap = *mut ::core::ffi::c_void;
pub type IMPEG2TuneRequest = *mut ::core::ffi::c_void;
pub type IMPEG2TuneRequestFactory = *mut ::core::ffi::c_void;
pub type IMPEG2TuneRequestSupport = *mut ::core::ffi::c_void;
pub type IMPEG2_TIF_CONTROL = *mut ::core::ffi::c_void;
pub type IMSEventBinder = *mut ::core::ffi::c_void;
pub type IMSVidAnalogTuner = *mut ::core::ffi::c_void;
pub type IMSVidAnalogTuner2 = *mut ::core::ffi::c_void;
pub type IMSVidAnalogTunerEvent = *mut ::core::ffi::c_void;
pub type IMSVidAudioRenderer = *mut ::core::ffi::c_void;
pub type IMSVidAudioRendererDevices = *mut ::core::ffi::c_void;
pub type IMSVidAudioRendererEvent = *mut ::core::ffi::c_void;
pub type IMSVidAudioRendererEvent2 = *mut ::core::ffi::c_void;
pub type IMSVidClosedCaptioning = *mut ::core::ffi::c_void;
pub type IMSVidClosedCaptioning2 = *mut ::core::ffi::c_void;
pub type IMSVidClosedCaptioning3 = *mut ::core::ffi::c_void;
pub type IMSVidCompositionSegment = *mut ::core::ffi::c_void;
pub type IMSVidCtl = *mut ::core::ffi::c_void;
pub type IMSVidDataServices = *mut ::core::ffi::c_void;
pub type IMSVidDataServicesEvent = *mut ::core::ffi::c_void;
pub type IMSVidDevice = *mut ::core::ffi::c_void;
pub type IMSVidDevice2 = *mut ::core::ffi::c_void;
pub type IMSVidDeviceEvent = *mut ::core::ffi::c_void;
pub type IMSVidEVR = *mut ::core::ffi::c_void;
pub type IMSVidEVREvent = *mut ::core::ffi::c_void;
pub type IMSVidEncoder = *mut ::core::ffi::c_void;
pub type IMSVidFeature = *mut ::core::ffi::c_void;
pub type IMSVidFeatureEvent = *mut ::core::ffi::c_void;
pub type IMSVidFeatures = *mut ::core::ffi::c_void;
pub type IMSVidFilePlayback = *mut ::core::ffi::c_void;
pub type IMSVidFilePlayback2 = *mut ::core::ffi::c_void;
pub type IMSVidFilePlaybackEvent = *mut ::core::ffi::c_void;
pub type IMSVidGenericSink = *mut ::core::ffi::c_void;
pub type IMSVidGenericSink2 = *mut ::core::ffi::c_void;
pub type IMSVidGraphSegment = *mut ::core::ffi::c_void;
pub type IMSVidGraphSegmentContainer = *mut ::core::ffi::c_void;
pub type IMSVidGraphSegmentUserInput = *mut ::core::ffi::c_void;
pub type IMSVidInputDevice = *mut ::core::ffi::c_void;
pub type IMSVidInputDeviceEvent = *mut ::core::ffi::c_void;
pub type IMSVidInputDevices = *mut ::core::ffi::c_void;
pub type IMSVidOutputDevice = *mut ::core::ffi::c_void;
pub type IMSVidOutputDeviceEvent = *mut ::core::ffi::c_void;
pub type IMSVidOutputDevices = *mut ::core::ffi::c_void;
pub type IMSVidPlayback = *mut ::core::ffi::c_void;
pub type IMSVidPlaybackEvent = *mut ::core::ffi::c_void;
pub type IMSVidRect = *mut ::core::ffi::c_void;
pub type IMSVidStreamBufferRecordingControl = *mut ::core::ffi::c_void;
pub type IMSVidStreamBufferSink = *mut ::core::ffi::c_void;
pub type IMSVidStreamBufferSink2 = *mut ::core::ffi::c_void;
pub type IMSVidStreamBufferSink3 = *mut ::core::ffi::c_void;
pub type IMSVidStreamBufferSinkEvent = *mut ::core::ffi::c_void;
pub type IMSVidStreamBufferSinkEvent2 = *mut ::core::ffi::c_void;
pub type IMSVidStreamBufferSinkEvent3 = *mut ::core::ffi::c_void;
pub type IMSVidStreamBufferSinkEvent4 = *mut ::core::ffi::c_void;
pub type IMSVidStreamBufferSource = *mut ::core::ffi::c_void;
pub type IMSVidStreamBufferSource2 = *mut ::core::ffi::c_void;
pub type IMSVidStreamBufferSourceEvent = *mut ::core::ffi::c_void;
pub type IMSVidStreamBufferSourceEvent2 = *mut ::core::ffi::c_void;
pub type IMSVidStreamBufferSourceEvent3 = *mut ::core::ffi::c_void;
pub type IMSVidStreamBufferV2SourceEvent = *mut ::core::ffi::c_void;
pub type IMSVidTuner = *mut ::core::ffi::c_void;
pub type IMSVidTunerEvent = *mut ::core::ffi::c_void;
pub type IMSVidVMR9 = *mut ::core::ffi::c_void;
pub type IMSVidVRGraphSegment = *mut ::core::ffi::c_void;
pub type IMSVidVideoInputDevice = *mut ::core::ffi::c_void;
pub type IMSVidVideoRenderer = *mut ::core::ffi::c_void;
pub type IMSVidVideoRenderer2 = *mut ::core::ffi::c_void;
pub type IMSVidVideoRendererDevices = *mut ::core::ffi::c_void;
pub type IMSVidVideoRendererEvent = *mut ::core::ffi::c_void;
pub type IMSVidVideoRendererEvent2 = *mut ::core::ffi::c_void;
pub type IMSVidWebDVD = *mut ::core::ffi::c_void;
pub type IMSVidWebDVD2 = *mut ::core::ffi::c_void;
pub type IMSVidWebDVDAdm = *mut ::core::ffi::c_void;
pub type IMSVidWebDVDEvent = *mut ::core::ffi::c_void;
pub type IMSVidXDS = *mut ::core::ffi::c_void;
pub type IMSVidXDSEvent = *mut ::core::ffi::c_void;
pub type IMceBurnerControl = *mut ::core::ffi::c_void;
pub type IMediaControl = *mut ::core::ffi::c_void;
pub type IMediaEvent = *mut ::core::ffi::c_void;
pub type IMediaEventEx = *mut ::core::ffi::c_void;
pub type IMediaEventSink = *mut ::core::ffi::c_void;
pub type IMediaFilter = *mut ::core::ffi::c_void;
pub type IMediaParamInfo = *mut ::core::ffi::c_void;
pub type IMediaParams = *mut ::core::ffi::c_void;
pub type IMediaPosition = *mut ::core::ffi::c_void;
pub type IMediaPropertyBag = *mut ::core::ffi::c_void;
pub type IMediaSample = *mut ::core::ffi::c_void;
pub type IMediaSample2 = *mut ::core::ffi::c_void;
pub type IMediaSample2Config = *mut ::core::ffi::c_void;
pub type IMediaSeeking = *mut ::core::ffi::c_void;
pub type IMediaStream = *mut ::core::ffi::c_void;
pub type IMediaStreamFilter = *mut ::core::ffi::c_void;
pub type IMediaTypeInfo = *mut ::core::ffi::c_void;
pub type IMemAllocator = *mut ::core::ffi::c_void;
pub type IMemAllocatorCallbackTemp = *mut ::core::ffi::c_void;
pub type IMemAllocatorNotifyCallbackTemp = *mut ::core::ffi::c_void;
pub type IMemInputPin = *mut ::core::ffi::c_void;
pub type IMemoryData = *mut ::core::ffi::c_void;
pub type IMixerOCX = *mut ::core::ffi::c_void;
pub type IMixerOCXNotify = *mut ::core::ffi::c_void;
pub type IMixerPinConfig = *mut ::core::ffi::c_void;
pub type IMixerPinConfig2 = *mut ::core::ffi::c_void;
pub type IMpeg2Data = *mut ::core::ffi::c_void;
pub type IMpeg2Demultiplexer = *mut ::core::ffi::c_void;
pub type IMpeg2Stream = *mut ::core::ffi::c_void;
pub type IMpeg2TableFilter = *mut ::core::ffi::c_void;
pub type IMpegAudioDecoder = *mut ::core::ffi::c_void;
pub type IMultiMediaStream = *mut ::core::ffi::c_void;
pub type IOverlay = *mut ::core::ffi::c_void;
pub type IOverlayNotify = *mut ::core::ffi::c_void;
pub type IOverlayNotify2 = *mut ::core::ffi::c_void;
pub type IPAT = *mut ::core::ffi::c_void;
pub type IPBDAAttributesDescriptor = *mut ::core::ffi::c_void;
pub type IPBDAEntitlementDescriptor = *mut ::core::ffi::c_void;
pub type IPBDASiParser = *mut ::core::ffi::c_void;
pub type IPBDA_EIT = *mut ::core::ffi::c_void;
pub type IPBDA_Services = *mut ::core::ffi::c_void;
pub type IPMT = *mut ::core::ffi::c_void;
pub type IPSITables = *mut ::core::ffi::c_void;
pub type IPTFilterLicenseRenewal = *mut ::core::ffi::c_void;
pub type IPersistMediaPropertyBag = *mut ::core::ffi::c_void;
pub type IPersistTuneXml = *mut ::core::ffi::c_void;
pub type IPersistTuneXmlUtility = *mut ::core::ffi::c_void;
pub type IPersistTuneXmlUtility2 = *mut ::core::ffi::c_void;
pub type IPin = *mut ::core::ffi::c_void;
pub type IPinConnection = *mut ::core::ffi::c_void;
pub type IPinFlowControl = *mut ::core::ffi::c_void;
pub type IPinInfo = *mut ::core::ffi::c_void;
pub type IQualProp = *mut ::core::ffi::c_void;
pub type IQualityControl = *mut ::core::ffi::c_void;
pub type IQueueCommand = *mut ::core::ffi::c_void;
pub type IRegFilterInfo = *mut ::core::ffi::c_void;
pub type IRegisterServiceProvider = *mut ::core::ffi::c_void;
pub type IRegisterTuner = *mut ::core::ffi::c_void;
pub type IResourceConsumer = *mut ::core::ffi::c_void;
pub type IResourceManager = *mut ::core::ffi::c_void;
pub type ISBE2Crossbar = *mut ::core::ffi::c_void;
pub type ISBE2EnumStream = *mut ::core::ffi::c_void;
pub type ISBE2FileScan = *mut ::core::ffi::c_void;
pub type ISBE2GlobalEvent = *mut ::core::ffi::c_void;
pub type ISBE2GlobalEvent2 = *mut ::core::ffi::c_void;
pub type ISBE2MediaTypeProfile = *mut ::core::ffi::c_void;
pub type ISBE2SpanningEvent = *mut ::core::ffi::c_void;
pub type ISBE2StreamMap = *mut ::core::ffi::c_void;
pub type ISCTE_EAS = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type ISDBCAS_REQUEST_ID = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISDBCAS_REQUEST_ID_EMG: ISDBCAS_REQUEST_ID = 56i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISDBCAS_REQUEST_ID_EMD: ISDBCAS_REQUEST_ID = 58i32;
pub const ISDBSLocator: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1694805997, data2: 42537, data3: 17756, data4: [167, 241, 4, 150, 77, 234, 92, 196] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISDB_BIT_PID: u32 = 36u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISDB_BIT_TID: u32 = 196u32;
pub const ISDB_CABLE_TV_NETWORK_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3379879349, data2: 16894, data3: 19237, data4: [151, 65, 146, 240, 73, 241, 213, 209] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISDB_CDT_PID: u32 = 41u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISDB_CDT_TID: u32 = 200u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISDB_EMM_TID: u32 = 133u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISDB_LDT_PID: u32 = 37u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISDB_LDT_TID: u32 = 199u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISDB_NBIT_MSG_TID: u32 = 197u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISDB_NBIT_PID: u32 = 37u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISDB_NBIT_REF_TID: u32 = 198u32;
pub const ISDB_SATELLITE_TV_NETWORK_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2963596960, data2: 27162, data3: 19331, data4: [187, 91, 144, 62, 29, 144, 230, 182] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISDB_SDTT_ALT_PID: u32 = 40u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISDB_SDTT_PID: u32 = 35u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISDB_SDTT_TID: u32 = 195u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISDB_ST_TID: u32 = 114u32;
pub const ISDB_S_NETWORK_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2716303874, data2: 5209, data3: 16817, data4: [156, 169, 42, 146, 88, 122, 66, 204] };
pub const ISDB_TERRESTRIAL_TV_NETWORK_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2500034415, data2: 15047, data3: 17490, data4: [182, 196, 69, 169, 206, 146, 146, 162] };
pub const ISDB_T_NETWORK_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4231550374, data2: 51457, data3: 20270, data4: [171, 168, 144, 129, 90, 252, 108, 131] };
pub type ISIInbandEPG = *mut ::core::ffi::c_void;
pub type ISIInbandEPGEvent = *mut ::core::ffi::c_void;
pub type IScanningTuner = *mut ::core::ffi::c_void;
pub type IScanningTunerEx = *mut ::core::ffi::c_void;
pub type ISectionList = *mut ::core::ffi::c_void;
pub type ISeekingPassThru = *mut ::core::ffi::c_void;
pub type ISelector = *mut ::core::ffi::c_void;
pub type IServiceLocationDescriptor = *mut ::core::ffi::c_void;
pub type ISpecifyParticularPages = *mut ::core::ffi::c_void;
pub type IStreamBufferConfigure = *mut ::core::ffi::c_void;
pub type IStreamBufferConfigure2 = *mut ::core::ffi::c_void;
pub type IStreamBufferConfigure3 = *mut ::core::ffi::c_void;
pub type IStreamBufferDataCounters = *mut ::core::ffi::c_void;
pub type IStreamBufferInitialize = *mut ::core::ffi::c_void;
pub type IStreamBufferMediaSeeking = *mut ::core::ffi::c_void;
pub type IStreamBufferMediaSeeking2 = *mut ::core::ffi::c_void;
pub type IStreamBufferRecComp = *mut ::core::ffi::c_void;
pub type IStreamBufferRecordControl = *mut ::core::ffi::c_void;
pub type IStreamBufferRecordingAttribute = *mut ::core::ffi::c_void;
pub type IStreamBufferSink = *mut ::core::ffi::c_void;
pub type IStreamBufferSink2 = *mut ::core::ffi::c_void;
pub type IStreamBufferSink3 = *mut ::core::ffi::c_void;
pub type IStreamBufferSource = *mut ::core::ffi::c_void;
pub type IStreamBuilder = *mut ::core::ffi::c_void;
pub type IStreamSample = *mut ::core::ffi::c_void;
pub type ITSDT = *mut ::core::ffi::c_void;
pub type ITuneRequest = *mut ::core::ffi::c_void;
pub type ITuneRequestInfo = *mut ::core::ffi::c_void;
pub type ITuneRequestInfoEx = *mut ::core::ffi::c_void;
pub type ITuner = *mut ::core::ffi::c_void;
pub type ITunerCap = *mut ::core::ffi::c_void;
pub type ITunerCapEx = *mut ::core::ffi::c_void;
pub type ITuningSpace = *mut ::core::ffi::c_void;
pub type ITuningSpaceContainer = *mut ::core::ffi::c_void;
pub type ITuningSpaces = *mut ::core::ffi::c_void;
pub type IVMRAspectRatioControl = *mut ::core::ffi::c_void;
pub type IVMRAspectRatioControl9 = *mut ::core::ffi::c_void;
pub type IVMRDeinterlaceControl = *mut ::core::ffi::c_void;
pub type IVMRDeinterlaceControl9 = *mut ::core::ffi::c_void;
pub type IVMRFilterConfig = *mut ::core::ffi::c_void;
pub type IVMRFilterConfig9 = *mut ::core::ffi::c_void;
pub type IVMRImageCompositor = *mut ::core::ffi::c_void;
pub type IVMRImageCompositor9 = *mut ::core::ffi::c_void;
pub type IVMRImagePresenter = *mut ::core::ffi::c_void;
pub type IVMRImagePresenter9 = *mut ::core::ffi::c_void;
pub type IVMRImagePresenterConfig = *mut ::core::ffi::c_void;
pub type IVMRImagePresenterConfig9 = *mut ::core::ffi::c_void;
pub type IVMRImagePresenterExclModeConfig = *mut ::core::ffi::c_void;
pub type IVMRMixerBitmap = *mut ::core::ffi::c_void;
pub type IVMRMixerBitmap9 = *mut ::core::ffi::c_void;
pub type IVMRMixerControl = *mut ::core::ffi::c_void;
pub type IVMRMixerControl9 = *mut ::core::ffi::c_void;
pub type IVMRMonitorConfig = *mut ::core::ffi::c_void;
pub type IVMRMonitorConfig9 = *mut ::core::ffi::c_void;
pub type IVMRSurface = *mut ::core::ffi::c_void;
pub type IVMRSurface9 = *mut ::core::ffi::c_void;
pub type IVMRSurfaceAllocator = *mut ::core::ffi::c_void;
pub type IVMRSurfaceAllocator9 = *mut ::core::ffi::c_void;
pub type IVMRSurfaceAllocatorEx9 = *mut ::core::ffi::c_void;
pub type IVMRSurfaceAllocatorNotify = *mut ::core::ffi::c_void;
pub type IVMRSurfaceAllocatorNotify9 = *mut ::core::ffi::c_void;
pub type IVMRVideoStreamControl = *mut ::core::ffi::c_void;
pub type IVMRVideoStreamControl9 = *mut ::core::ffi::c_void;
pub type IVMRWindowlessControl = *mut ::core::ffi::c_void;
pub type IVMRWindowlessControl9 = *mut ::core::ffi::c_void;
pub type IVPBaseConfig = *mut ::core::ffi::c_void;
pub type IVPBaseNotify = *mut ::core::ffi::c_void;
pub type IVPConfig = *mut ::core::ffi::c_void;
pub type IVPManager = *mut ::core::ffi::c_void;
pub type IVPNotify = *mut ::core::ffi::c_void;
pub type IVPNotify2 = *mut ::core::ffi::c_void;
pub type IVPVBIConfig = *mut ::core::ffi::c_void;
pub type IVPVBINotify = *mut ::core::ffi::c_void;
pub type IVideoEncoder = *mut ::core::ffi::c_void;
pub type IVideoFrameStep = *mut ::core::ffi::c_void;
pub type IVideoProcAmp = *mut ::core::ffi::c_void;
pub type IVideoWindow = *mut ::core::ffi::c_void;
pub type IXDSCodec = *mut ::core::ffi::c_void;
pub type IXDSCodecConfig = *mut ::core::ffi::c_void;
pub type IXDSCodecEvents = *mut ::core::ffi::c_void;
pub type IXDSToRat = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type InterleavingMode = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const INTERLEAVE_NONE: InterleavingMode = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const INTERLEAVE_CAPTURE: InterleavingMode = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const INTERLEAVE_FULL: InterleavingMode = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const INTERLEAVE_NONE_BUFFERED: InterleavingMode = 3i32;
pub const KSCATEGORY_BDA_IP_SINK: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1905811274, data2: 7329, data3: 4563, data4: [156, 200, 0, 192, 79, 121, 113, 224] };
pub const KSCATEGORY_BDA_NETWORK_EPG: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1905811273, data2: 7329, data3: 4563, data4: [156, 200, 0, 192, 79, 121, 113, 224] };
pub const KSCATEGORY_BDA_NETWORK_PROVIDER: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1905811275, data2: 7329, data3: 4563, data4: [156, 200, 0, 192, 79, 121, 113, 224] };
pub const KSCATEGORY_BDA_NETWORK_TUNER: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1905811272, data2: 7329, data3: 4563, data4: [156, 200, 0, 192, 79, 121, 113, 224] };
pub const KSCATEGORY_BDA_RECEIVER_COMPONENT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4245314292, data2: 46109, data3: 4562, data4: [156, 149, 0, 192, 79, 121, 113, 224] };
pub const KSCATEGORY_BDA_TRANSPORT_INFORMATION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2732787535, data2: 27709, data3: 4563, data4: [182, 83, 0, 192, 79, 121, 73, 142] };
pub const KSDATAFORMAT_SPECIFIER_BDA_IP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1804145696, data2: 56073, data3: 4562, data4: [143, 50, 0, 192, 79, 121, 113, 226] };
pub const KSDATAFORMAT_SPECIFIER_BDA_TRANSPORT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2381162237, data2: 44127, data3: 17204, data4: [142, 207, 164, 186, 143, 167, 208, 240] };
pub const KSDATAFORMAT_SUBTYPE_ATSC_SI: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3016178044, data2: 54019, data3: 16717, data4: [179, 60, 78, 210, 201, 210, 151, 51] };
pub const KSDATAFORMAT_SUBTYPE_BDA_IP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1520050492, data2: 56072, data3: 4562, data4: [143, 50, 0, 192, 79, 121, 113, 226] };
pub const KSDATAFORMAT_SUBTYPE_BDA_IP_CONTROL: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1234720488, data2: 59483, data3: 18669, data4: [155, 234, 65, 13, 13, 212, 239, 129] };
pub const KSDATAFORMAT_SUBTYPE_BDA_MPEG2_TRANSPORT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4105089858, data2: 809, data3: 20445, data4: [168, 253, 74, 255, 73, 38, 201, 120] };
pub const KSDATAFORMAT_SUBTYPE_BDA_OPENCABLE_OOB_PSIP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2501322715, data2: 53966, data3: 17704, data4: [150, 246, 51, 1, 250, 187, 45, 224] };
pub const KSDATAFORMAT_SUBTYPE_BDA_OPENCABLE_PSIP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1982742374, data2: 13167, data3: 18641, data4: [191, 131, 43, 0, 53, 44, 17, 240] };
pub const KSDATAFORMAT_SUBTYPE_DVB_SI: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3923587491, data2: 8733, data3: 19163, data4: [133, 50, 154, 243, 9, 193, 164, 8] };
pub const KSDATAFORMAT_SUBTYPE_ISDB_SI: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1244588953, data2: 25688, data3: 17720, data4: [177, 135, 4, 1, 124, 65, 65, 63] };
pub const KSDATAFORMAT_SUBTYPE_PBDA_TRANSPORT_RAW: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 226159938, data2: 52122, data3: 4571, data4: [151, 5, 0, 80, 86, 192, 0, 8] };
pub const KSDATAFORMAT_TYPE_BDA_ANTENNA: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1905811265, data2: 7329, data3: 4563, data4: [156, 200, 0, 192, 79, 121, 113, 224] };
pub const KSDATAFORMAT_TYPE_BDA_IF_SIGNAL: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1639844679, data2: 42475, data3: 18843, data4: [154, 133, 91, 22, 192, 127, 18, 88] };
pub const KSDATAFORMAT_TYPE_BDA_IP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3797908366, data2: 52428, data3: 4562, data4: [143, 37, 0, 192, 79, 121, 113, 226] };
pub const KSDATAFORMAT_TYPE_BDA_IP_CONTROL: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3671938969, data2: 32091, data3: 19299, data4: [128, 251, 209, 68, 47, 38, 182, 33] };
pub const KSDATAFORMAT_TYPE_MPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1163859820, data2: 19206, data3: 18382, data4: [154, 239, 140, 174, 247, 61, 247, 181] };
pub const KSDATAFORMAT_TYPE_MPEG2_SECTIONS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1163859820, data2: 19206, data3: 18382, data4: [154, 239, 140, 174, 247, 61, 247, 181] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_KernelStreaming"))]
pub struct KSEVENTDATA_BDA_RF_TUNER_SCAN_S {
    pub EventData: super::KernelStreaming::KSEVENTDATA,
    pub StartFrequency: u32,
    pub EndFrequency: u32,
    pub LockRequested: BDA_LockType,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_KernelStreaming"))]
impl ::core::marker::Copy for KSEVENTDATA_BDA_RF_TUNER_SCAN_S {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_KernelStreaming"))]
impl ::core::clone::Clone for KSEVENTDATA_BDA_RF_TUNER_SCAN_S {
    fn clone(&self) -> Self {
        *self
    }
}
pub const KSEVENTSETID_BdaCAEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1217154252, data2: 46952, data3: 16681, data4: [142, 177, 176, 10, 7, 31, 144, 104] };
pub const KSEVENTSETID_BdaDiseqCEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2333719536, data2: 16772, data3: 17324, data4: [173, 60, 12, 136, 155, 228, 194, 18] };
pub const KSEVENTSETID_BdaEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2927515058, data2: 38615, data3: 20009, data4: [144, 143, 98, 249, 91, 42, 22, 121] };
pub const KSEVENTSETID_BdaPinEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 273121741, data2: 20669, data3: 16597, data4: [149, 251, 8, 126, 14, 134, 165, 145] };
pub const KSEVENTSETID_BdaTunerEvent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2864029207, data2: 457, data3: 20159, data4: [147, 242, 252, 59, 121, 180, 111, 145] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSEVENT_BDA_EVENT_TYPE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSEVENT_BDA_EVENT_PENDINGEVENT: KSEVENT_BDA_EVENT_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSEVENT_BDA_TUNER = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSEVENT_BDA_TUNER_SCAN: KSEVENT_BDA_TUNER = 0i32;
pub const KSMETHODSETID_BdaChangeSync: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4245314291, data2: 46109, data3: 4562, data4: [156, 149, 0, 192, 79, 121, 113, 224] };
pub const KSMETHODSETID_BdaConditionalAccessService: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 281990068, data2: 12811, data3: 16831, data4: [152, 36, 27, 46, 104, 231, 30, 185] };
pub const KSMETHODSETID_BdaDebug: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 222990572, data2: 50845, data3: 20194, data4: [140, 90, 251, 31, 99, 165, 13, 161] };
pub const KSMETHODSETID_BdaDeviceConfiguration: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1905811269, data2: 7329, data3: 4563, data4: [156, 200, 0, 192, 79, 121, 113, 224] };
pub const KSMETHODSETID_BdaDrmService: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3220616635, data2: 45230, data3: 18508, data4: [157, 202, 115, 82, 143, 176, 180, 110] };
pub const KSMETHODSETID_BdaEventing: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4187263706, data2: 24979, data3: 20144, data4: [134, 144, 102, 134, 203, 255, 113, 62] };
pub const KSMETHODSETID_BdaGuideDataDeliveryService: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2375898466, data2: 5513, data3: 16765, data4: [153, 206, 172, 83, 29, 218, 25, 249] };
pub const KSMETHODSETID_BdaIsdbConditionalAccess: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1583924775, data2: 5826, data3: 20076, data4: [177, 226, 208, 1, 112, 205, 170, 15] };
pub const KSMETHODSETID_BdaMux: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2485825516, data2: 19461, data3: 19572, data4: [184, 235, 135, 6, 194, 164, 148, 63] };
pub const KSMETHODSETID_BdaNameValue: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 920679172, data2: 40717, data3: 20104, data4: [145, 24, 172, 11, 163, 23, 183, 242] };
pub const KSMETHODSETID_BdaNameValueA: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 203688301, data2: 24565, data3: 18398, data4: [168, 86, 6, 46, 88, 126, 55, 39] };
pub const KSMETHODSETID_BdaScanning: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 317409759, data2: 25161, data3: 18419, data4: [177, 144, 226, 30, 110, 47, 138, 156] };
pub const KSMETHODSETID_BdaTSSelector: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 500150249, data2: 46174, data3: 16819, data4: [187, 42, 86, 30, 177, 41, 174, 152] };
pub const KSMETHODSETID_BdaTuner: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3077836847, data2: 44039, data3: 18314, data4: [130, 40, 39, 66, 217, 97, 250, 126] };
pub const KSMETHODSETID_BdaUserActivity: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3987064884, data2: 17713, data3: 18492, data4: [190, 10, 148, 230, 201, 111, 243, 150] };
pub const KSMETHODSETID_BdaWmdrmSession: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1273428541, data2: 1997, data3: 16697, data4: [139, 128, 140, 24, 186, 58, 236, 136] };
pub const KSMETHODSETID_BdaWmdrmTuner: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2262399439, data2: 43175, data3: 20372, data4: [181, 251, 20, 192, 172, 166, 143, 230] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSMETHOD_BDA_CAS_SERVICE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_CAS_CHECKENTITLEMENTTOKEN: KSMETHOD_BDA_CAS_SERVICE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_CAS_SETCAPTURETOKEN: KSMETHOD_BDA_CAS_SERVICE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_CAS_OPENBROADCASTMMI: KSMETHOD_BDA_CAS_SERVICE = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_CAS_CLOSEMMIDIALOG: KSMETHOD_BDA_CAS_SERVICE = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSMETHOD_BDA_CHANGE_SYNC = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_START_CHANGES: KSMETHOD_BDA_CHANGE_SYNC = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_CHECK_CHANGES: KSMETHOD_BDA_CHANGE_SYNC = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_COMMIT_CHANGES: KSMETHOD_BDA_CHANGE_SYNC = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_GET_CHANGE_STATE: KSMETHOD_BDA_CHANGE_SYNC = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSMETHOD_BDA_DEBUG_SERVICE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_DEBUG_LEVEL: KSMETHOD_BDA_DEBUG_SERVICE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_DEBUG_DATA: KSMETHOD_BDA_DEBUG_SERVICE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSMETHOD_BDA_DEVICE_CONFIGURATION = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_CREATE_PIN_FACTORY: KSMETHOD_BDA_DEVICE_CONFIGURATION = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_DELETE_PIN_FACTORY: KSMETHOD_BDA_DEVICE_CONFIGURATION = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_CREATE_TOPOLOGY: KSMETHOD_BDA_DEVICE_CONFIGURATION = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSMETHOD_BDA_DRM = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_DRM_CURRENT: KSMETHOD_BDA_DRM = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_DRM_DRMSTATUS: KSMETHOD_BDA_DRM = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSMETHOD_BDA_EVENTING_SERVICE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_EVENT_DATA: KSMETHOD_BDA_EVENTING_SERVICE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_EVENT_COMPLETE: KSMETHOD_BDA_EVENTING_SERVICE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSMETHOD_BDA_GDDS_SERVICE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_GDDS_DATATYPE: KSMETHOD_BDA_GDDS_SERVICE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_GDDS_DATA: KSMETHOD_BDA_GDDS_SERVICE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_GDDS_TUNEXMLFROMIDX: KSMETHOD_BDA_GDDS_SERVICE = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_GDDS_GETSERVICES: KSMETHOD_BDA_GDDS_SERVICE = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_GDDS_SERVICEFROMTUNEXML: KSMETHOD_BDA_GDDS_SERVICE = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_GDDS_DATAUPDATE: KSMETHOD_BDA_GDDS_SERVICE = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSMETHOD_BDA_GPNV_SERVICE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_GPNV_GETVALUE: KSMETHOD_BDA_GPNV_SERVICE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_GPNV_SETVALUE: KSMETHOD_BDA_GPNV_SERVICE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_GPNV_NAMEFROMINDEX: KSMETHOD_BDA_GPNV_SERVICE = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_GPNV_GETVALUEUPDATENAME: KSMETHOD_BDA_GPNV_SERVICE = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSMETHOD_BDA_ISDB_CAS = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_ISDBCAS_SETREQUEST: KSMETHOD_BDA_ISDB_CAS = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_ISDBCAS_RESPONSEDATA: KSMETHOD_BDA_ISDB_CAS = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSMETHOD_BDA_MUX_SERVICE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_MUX_GETPIDLIST: KSMETHOD_BDA_MUX_SERVICE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_MUX_SETPIDLIST: KSMETHOD_BDA_MUX_SERVICE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSMETHOD_BDA_SCAN_SERVICE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_SCAN_CAPABILTIES: KSMETHOD_BDA_SCAN_SERVICE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_SCANNING_STATE: KSMETHOD_BDA_SCAN_SERVICE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_SCAN_FILTER: KSMETHOD_BDA_SCAN_SERVICE = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_SCAN_START: KSMETHOD_BDA_SCAN_SERVICE = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_SCAN_RESUME: KSMETHOD_BDA_SCAN_SERVICE = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_SCAN_STOP: KSMETHOD_BDA_SCAN_SERVICE = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSMETHOD_BDA_TS_SELECTOR = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_TS_SELECTOR_SETTSID: KSMETHOD_BDA_TS_SELECTOR = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_TS_SELECTOR_GETTSINFORMATION: KSMETHOD_BDA_TS_SELECTOR = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSMETHOD_BDA_TUNER_SERVICE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_TUNER_SETTUNER: KSMETHOD_BDA_TUNER_SERVICE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_TUNER_GETTUNERSTATE: KSMETHOD_BDA_TUNER_SERVICE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_TUNER_SIGNALNOISERATIO: KSMETHOD_BDA_TUNER_SERVICE = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSMETHOD_BDA_USERACTIVITY_SERVICE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_USERACTIVITY_USEREASON: KSMETHOD_BDA_USERACTIVITY_SERVICE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_USERACTIVITY_INTERVAL: KSMETHOD_BDA_USERACTIVITY_SERVICE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_USERACTIVITY_DETECTED: KSMETHOD_BDA_USERACTIVITY_SERVICE = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSMETHOD_BDA_WMDRM = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_WMDRM_STATUS: KSMETHOD_BDA_WMDRM = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_WMDRM_REVINFO: KSMETHOD_BDA_WMDRM = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_WMDRM_CRL: KSMETHOD_BDA_WMDRM = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_WMDRM_MESSAGE: KSMETHOD_BDA_WMDRM = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_WMDRM_REISSUELICENSE: KSMETHOD_BDA_WMDRM = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_WMDRM_RENEWLICENSE: KSMETHOD_BDA_WMDRM = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_WMDRM_LICENSE: KSMETHOD_BDA_WMDRM = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_WMDRM_KEYINFO: KSMETHOD_BDA_WMDRM = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSMETHOD_BDA_WMDRM_TUNER = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_WMDRMTUNER_CANCELCAPTURETOKEN: KSMETHOD_BDA_WMDRM_TUNER = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_WMDRMTUNER_SETPIDPROTECTION: KSMETHOD_BDA_WMDRM_TUNER = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_WMDRMTUNER_GETPIDPROTECTION: KSMETHOD_BDA_WMDRM_TUNER = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_WMDRMTUNER_SETSYNCVALUE: KSMETHOD_BDA_WMDRM_TUNER = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_WMDRMTUNER_STARTCODEPROFILE: KSMETHOD_BDA_WMDRM_TUNER = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSMETHOD_BDA_WMDRMTUNER_PURCHASE_ENTITLEMENT: KSMETHOD_BDA_WMDRM_TUNER = 5i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub struct KSM_BDA_BUFFER {
    pub NodeMethod: super::KernelStreaming::KSM_NODE,
    pub ulBufferSize: u32,
    pub argbBuffer: [u8; 1],
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSM_BDA_BUFFER {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSM_BDA_BUFFER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub struct KSM_BDA_CAS_CAPTURETOKEN {
    pub NodeMethod: super::KernelStreaming::KSM_NODE,
    pub ulTokenLength: u32,
    pub argbToken: [u8; 1],
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSM_BDA_CAS_CAPTURETOKEN {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSM_BDA_CAS_CAPTURETOKEN {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_KernelStreaming"))]
pub struct KSM_BDA_CAS_CLOSEMMIDIALOG {
    pub NodeMethod: super::KernelStreaming::KSM_NODE,
    pub ulDialogRequest: u32,
    pub cLanguage: [super::super::Foundation::CHAR; 12],
    pub ulDialogNumber: u32,
    pub ulReason: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_KernelStreaming"))]
impl ::core::marker::Copy for KSM_BDA_CAS_CLOSEMMIDIALOG {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_KernelStreaming"))]
impl ::core::clone::Clone for KSM_BDA_CAS_CLOSEMMIDIALOG {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_KernelStreaming"))]
pub struct KSM_BDA_CAS_ENTITLEMENTTOKEN {
    pub NodeMethod: super::KernelStreaming::KSM_NODE,
    pub ulDialogRequest: u32,
    pub cLanguage: [super::super::Foundation::CHAR; 12],
    pub ulRequestType: u32,
    pub ulEntitlementTokenLen: u32,
    pub argbEntitlementToken: [u8; 1],
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_KernelStreaming"))]
impl ::core::marker::Copy for KSM_BDA_CAS_ENTITLEMENTTOKEN {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_KernelStreaming"))]
impl ::core::clone::Clone for KSM_BDA_CAS_ENTITLEMENTTOKEN {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_KernelStreaming"))]
pub struct KSM_BDA_CAS_OPENBROADCASTMMI {
    pub NodeMethod: super::KernelStreaming::KSM_NODE,
    pub ulDialogRequest: u32,
    pub cLanguage: [super::super::Foundation::CHAR; 12],
    pub ulEventId: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_KernelStreaming"))]
impl ::core::marker::Copy for KSM_BDA_CAS_OPENBROADCASTMMI {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_KernelStreaming"))]
impl ::core::clone::Clone for KSM_BDA_CAS_OPENBROADCASTMMI {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub struct KSM_BDA_DEBUG_LEVEL {
    pub Method: super::KernelStreaming::KSIDENTIFIER,
    pub ucDebugLevel: u8,
    pub ulDebugStringSize: u32,
    pub argbDebugString: [u8; 1],
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSM_BDA_DEBUG_LEVEL {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSM_BDA_DEBUG_LEVEL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub struct KSM_BDA_DRM_SETDRM {
    pub NodeMethod: super::KernelStreaming::KSM_NODE,
    pub NewDRMuuid: ::windows_sys::core::GUID,
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSM_BDA_DRM_SETDRM {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSM_BDA_DRM_SETDRM {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub struct KSM_BDA_EVENT_COMPLETE {
    pub Method: super::KernelStreaming::KSIDENTIFIER,
    pub ulEventID: u32,
    pub ulEventResult: u32,
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSM_BDA_EVENT_COMPLETE {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSM_BDA_EVENT_COMPLETE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub struct KSM_BDA_GDDS_SERVICEFROMTUNEXML {
    pub Method: super::KernelStreaming::KSIDENTIFIER,
    pub ulTuneXmlLength: u32,
    pub argbTuneXml: [u8; 1],
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSM_BDA_GDDS_SERVICEFROMTUNEXML {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSM_BDA_GDDS_SERVICEFROMTUNEXML {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub struct KSM_BDA_GDDS_TUNEXMLFROMIDX {
    pub Method: super::KernelStreaming::KSIDENTIFIER,
    pub ulIdx: u64,
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSM_BDA_GDDS_TUNEXMLFROMIDX {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSM_BDA_GDDS_TUNEXMLFROMIDX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_KernelStreaming"))]
pub struct KSM_BDA_GPNV_GETVALUE {
    pub Method: super::KernelStreaming::KSIDENTIFIER,
    pub ulNameLength: u32,
    pub cLanguage: [super::super::Foundation::CHAR; 12],
    pub argbData: [u8; 1],
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_KernelStreaming"))]
impl ::core::marker::Copy for KSM_BDA_GPNV_GETVALUE {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_KernelStreaming"))]
impl ::core::clone::Clone for KSM_BDA_GPNV_GETVALUE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub struct KSM_BDA_GPNV_NAMEINDEX {
    pub Method: super::KernelStreaming::KSIDENTIFIER,
    pub ulValueNameIndex: u32,
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSM_BDA_GPNV_NAMEINDEX {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSM_BDA_GPNV_NAMEINDEX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_KernelStreaming"))]
pub struct KSM_BDA_GPNV_SETVALUE {
    pub Method: super::KernelStreaming::KSIDENTIFIER,
    pub ulDialogRequest: u32,
    pub cLanguage: [super::super::Foundation::CHAR; 12],
    pub ulNameLength: u32,
    pub ulValueLength: u32,
    pub argbName: [u8; 1],
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_KernelStreaming"))]
impl ::core::marker::Copy for KSM_BDA_GPNV_SETVALUE {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_KernelStreaming"))]
impl ::core::clone::Clone for KSM_BDA_GPNV_SETVALUE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub struct KSM_BDA_ISDBCAS_REQUEST {
    pub NodeMethod: super::KernelStreaming::KSM_NODE,
    pub ulRequestID: u32,
    pub ulIsdbCommandSize: u32,
    pub argbIsdbCommandData: [u8; 1],
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSM_BDA_ISDBCAS_REQUEST {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSM_BDA_ISDBCAS_REQUEST {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub struct KSM_BDA_PIN {
    pub Method: super::KernelStreaming::KSIDENTIFIER,
    pub Anonymous: KSM_BDA_PIN_0,
    pub Reserved: u32,
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSM_BDA_PIN {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSM_BDA_PIN {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub union KSM_BDA_PIN_0 {
    pub PinId: u32,
    pub PinType: u32,
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSM_BDA_PIN_0 {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSM_BDA_PIN_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub struct KSM_BDA_PIN_PAIR {
    pub Method: super::KernelStreaming::KSIDENTIFIER,
    pub Anonymous1: KSM_BDA_PIN_PAIR_0,
    pub Anonymous2: KSM_BDA_PIN_PAIR_1,
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSM_BDA_PIN_PAIR {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSM_BDA_PIN_PAIR {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub union KSM_BDA_PIN_PAIR_0 {
    pub InputPinId: u32,
    pub InputPinType: u32,
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSM_BDA_PIN_PAIR_0 {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSM_BDA_PIN_PAIR_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub union KSM_BDA_PIN_PAIR_1 {
    pub OutputPinId: u32,
    pub OutputPinType: u32,
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSM_BDA_PIN_PAIR_1 {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSM_BDA_PIN_PAIR_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub struct KSM_BDA_SCAN_CAPABILTIES {
    pub Method: super::KernelStreaming::KSIDENTIFIER,
    pub uuidBroadcastStandard: ::windows_sys::core::GUID,
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSM_BDA_SCAN_CAPABILTIES {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSM_BDA_SCAN_CAPABILTIES {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub struct KSM_BDA_SCAN_FILTER {
    pub Method: super::KernelStreaming::KSIDENTIFIER,
    pub ulScanModulationTypeSize: u32,
    pub AnalogVideoStandards: u64,
    pub argbScanModulationTypes: [u8; 1],
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSM_BDA_SCAN_FILTER {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSM_BDA_SCAN_FILTER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub struct KSM_BDA_SCAN_START {
    pub Method: super::KernelStreaming::KSIDENTIFIER,
    pub LowerFrequency: u32,
    pub HigherFrequency: u32,
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSM_BDA_SCAN_START {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSM_BDA_SCAN_START {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub struct KSM_BDA_TS_SELECTOR_SETTSID {
    pub NodeMethod: super::KernelStreaming::KSM_NODE,
    pub usTSID: u16,
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSM_BDA_TS_SELECTOR_SETTSID {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSM_BDA_TS_SELECTOR_SETTSID {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub struct KSM_BDA_TUNER_TUNEREQUEST {
    pub Method: super::KernelStreaming::KSIDENTIFIER,
    pub ulTuneLength: u32,
    pub argbTuneData: [u8; 1],
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSM_BDA_TUNER_TUNEREQUEST {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSM_BDA_TUNER_TUNEREQUEST {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub struct KSM_BDA_USERACTIVITY_USEREASON {
    pub Method: super::KernelStreaming::KSIDENTIFIER,
    pub ulUseReason: u32,
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSM_BDA_USERACTIVITY_USEREASON {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSM_BDA_USERACTIVITY_USEREASON {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub struct KSM_BDA_WMDRMTUNER_GETPIDPROTECTION {
    pub NodeMethod: super::KernelStreaming::KSM_NODE,
    pub ulPID: u32,
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSM_BDA_WMDRMTUNER_GETPIDPROTECTION {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSM_BDA_WMDRMTUNER_GETPIDPROTECTION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_KernelStreaming"))]
pub struct KSM_BDA_WMDRMTUNER_PURCHASEENTITLEMENT {
    pub NodeMethod: super::KernelStreaming::KSM_NODE,
    pub ulDialogRequest: u32,
    pub cLanguage: [super::super::Foundation::CHAR; 12],
    pub ulPurchaseTokenLength: u32,
    pub argbDataBuffer: [u8; 1],
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_KernelStreaming"))]
impl ::core::marker::Copy for KSM_BDA_WMDRMTUNER_PURCHASEENTITLEMENT {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_KernelStreaming"))]
impl ::core::clone::Clone for KSM_BDA_WMDRMTUNER_PURCHASEENTITLEMENT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub struct KSM_BDA_WMDRMTUNER_SETPIDPROTECTION {
    pub NodeMethod: super::KernelStreaming::KSM_NODE,
    pub ulPID: u32,
    pub uuidKeyID: ::windows_sys::core::GUID,
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSM_BDA_WMDRMTUNER_SETPIDPROTECTION {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSM_BDA_WMDRMTUNER_SETPIDPROTECTION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub struct KSM_BDA_WMDRMTUNER_SYNCVALUE {
    pub NodeMethod: super::KernelStreaming::KSM_NODE,
    pub ulSyncValue: u32,
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSM_BDA_WMDRMTUNER_SYNCVALUE {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSM_BDA_WMDRMTUNER_SYNCVALUE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub struct KSM_BDA_WMDRM_LICENSE {
    pub NodeMethod: super::KernelStreaming::KSM_NODE,
    pub uuidKeyID: ::windows_sys::core::GUID,
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSM_BDA_WMDRM_LICENSE {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSM_BDA_WMDRM_LICENSE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub struct KSM_BDA_WMDRM_RENEWLICENSE {
    pub NodeMethod: super::KernelStreaming::KSM_NODE,
    pub ulXMRLicenseLength: u32,
    pub ulEntitlementTokenLength: u32,
    pub argbDataBuffer: [u8; 1],
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSM_BDA_WMDRM_RENEWLICENSE {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSM_BDA_WMDRM_RENEWLICENSE {
    fn clone(&self) -> Self {
        *self
    }
}
pub const KSNODE_BDA_8PSK_DEMODULATOR: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3914834151, data2: 56728, data3: 19004, data4: [129, 11, 53, 37, 21, 122, 182, 46] };
pub const KSNODE_BDA_8VSB_DEMODULATOR: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1905811279, data2: 7329, data3: 4563, data4: [156, 200, 0, 192, 79, 121, 113, 224] };
pub const KSNODE_BDA_ANALOG_DEMODULATOR: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1666036121, data2: 10205, data3: 18104, data4: [172, 251, 236, 201, 142, 97, 162, 173] };
pub const KSNODE_BDA_COFDM_DEMODULATOR: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 766275077, data2: 60862, data3: 19356, data4: [179, 135, 27, 111, 173, 125, 100, 149] };
pub const KSNODE_BDA_COMMON_CA_POD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3628005628, data2: 62392, data3: 17835, data4: [139, 113, 236, 247, 195, 57, 222, 180] };
pub const KSNODE_BDA_DRI_DRM: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1335209332, data2: 52987, data3: 17106, data4: [148, 169, 104, 197, 178, 193, 170, 190] };
pub const KSNODE_BDA_IP_SINK: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1905811278, data2: 7329, data3: 4563, data4: [156, 200, 0, 192, 79, 121, 113, 224] };
pub const KSNODE_BDA_ISDB_S_DEMODULATOR: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3990758154, data2: 36998, data3: 17197, data4: [184, 165, 102, 112, 38, 56, 7, 233] };
pub const KSNODE_BDA_ISDB_T_DEMODULATOR: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4243208931, data2: 11442, data3: 17997, data4: [143, 93, 48, 92, 11, 183, 120, 162] };
pub const KSNODE_BDA_OPENCABLE_POD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 878187168, data2: 64380, data3: 18320, data4: [170, 126, 177, 219, 136, 172, 25, 201] };
pub const KSNODE_BDA_PBDA_CAS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3223750303, data2: 28969, data3: 20081, data4: [134, 150, 236, 143, 117, 41, 155, 119] };
pub const KSNODE_BDA_PBDA_DRM: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2666446083, data2: 61089, data3: 17679, data4: [150, 174, 99, 62, 109, 230, 60, 206] };
pub const KSNODE_BDA_PBDA_ISDBCAS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4073663155, data2: 23453, data3: 16558, data4: [171, 124, 78, 122, 208, 189, 28, 82] };
pub const KSNODE_BDA_PBDA_MUX: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4169955207, data2: 26232, data3: 20299, data4: [161, 62, 218, 9, 134, 29, 104, 43] };
pub const KSNODE_BDA_PBDA_TUNER: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2858320518, data2: 22844, data3: 18809, data4: [148, 148, 70, 162, 169, 223, 224, 118] };
pub const KSNODE_BDA_PID_FILTER: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4114687881, data2: 45216, data3: 17633, data4: [174, 79, 238, 153, 155, 27, 127, 190] };
pub const KSNODE_BDA_QAM_DEMODULATOR: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1905811277, data2: 7329, data3: 4563, data4: [156, 200, 0, 192, 79, 121, 113, 224] };
pub const KSNODE_BDA_QPSK_DEMODULATOR: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1670433029, data2: 10177, data3: 19815, data4: [189, 183, 119, 197, 13, 7, 147, 0] };
pub const KSNODE_BDA_RF_TUNER: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1905811276, data2: 7329, data3: 4563, data4: [156, 200, 0, 192, 79, 121, 113, 224] };
pub const KSNODE_BDA_TS_SELECTOR: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1591603589, data2: 65233, data3: 20293, data4: [150, 133, 187, 183, 60, 50, 60, 252] };
pub const KSNODE_BDA_VIDEO_ENCODER: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3649317347, data2: 26057, data3: 19140, data4: [147, 170, 118, 103, 130, 131, 59, 122] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSPROPERTY_BDA_AUTODEMODULATE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_AUTODEMODULATE_START: KSPROPERTY_BDA_AUTODEMODULATE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_AUTODEMODULATE_STOP: KSPROPERTY_BDA_AUTODEMODULATE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSPROPERTY_BDA_CA = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_ECM_MAP_STATUS: KSPROPERTY_BDA_CA = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_CA_MODULE_STATUS: KSPROPERTY_BDA_CA = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_CA_SMART_CARD_STATUS: KSPROPERTY_BDA_CA = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_CA_MODULE_UI: KSPROPERTY_BDA_CA = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_CA_SET_PROGRAM_PIDS: KSPROPERTY_BDA_CA = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_CA_REMOVE_PROGRAM: KSPROPERTY_BDA_CA = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSPROPERTY_BDA_CA_EVENT = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSEVENT_BDA_PROGRAM_FLOW_STATUS_CHANGED: KSPROPERTY_BDA_CA_EVENT = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSEVENT_BDA_CA_MODULE_STATUS_CHANGED: KSPROPERTY_BDA_CA_EVENT = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSEVENT_BDA_CA_SMART_CARD_STATUS_CHANGED: KSPROPERTY_BDA_CA_EVENT = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSEVENT_BDA_CA_MODULE_UI_REQUESTED: KSPROPERTY_BDA_CA_EVENT = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSPROPERTY_BDA_DIGITAL_DEMODULATOR = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_MODULATION_TYPE: KSPROPERTY_BDA_DIGITAL_DEMODULATOR = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_INNER_FEC_TYPE: KSPROPERTY_BDA_DIGITAL_DEMODULATOR = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_INNER_FEC_RATE: KSPROPERTY_BDA_DIGITAL_DEMODULATOR = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_OUTER_FEC_TYPE: KSPROPERTY_BDA_DIGITAL_DEMODULATOR = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_OUTER_FEC_RATE: KSPROPERTY_BDA_DIGITAL_DEMODULATOR = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_SYMBOL_RATE: KSPROPERTY_BDA_DIGITAL_DEMODULATOR = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_SPECTRAL_INVERSION: KSPROPERTY_BDA_DIGITAL_DEMODULATOR = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_GUARD_INTERVAL: KSPROPERTY_BDA_DIGITAL_DEMODULATOR = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_TRANSMISSION_MODE: KSPROPERTY_BDA_DIGITAL_DEMODULATOR = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_ROLL_OFF: KSPROPERTY_BDA_DIGITAL_DEMODULATOR = 9i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_PILOT: KSPROPERTY_BDA_DIGITAL_DEMODULATOR = 10i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_SIGNALTIMEOUTS: KSPROPERTY_BDA_DIGITAL_DEMODULATOR = 11i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_PLP_NUMBER: KSPROPERTY_BDA_DIGITAL_DEMODULATOR = 12i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSPROPERTY_BDA_DISEQC_COMMAND = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_DISEQC_ENABLE: KSPROPERTY_BDA_DISEQC_COMMAND = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_DISEQC_LNB_SOURCE: KSPROPERTY_BDA_DISEQC_COMMAND = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_DISEQC_USETONEBURST: KSPROPERTY_BDA_DISEQC_COMMAND = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_DISEQC_REPEATS: KSPROPERTY_BDA_DISEQC_COMMAND = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_DISEQC_SEND: KSPROPERTY_BDA_DISEQC_COMMAND = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_DISEQC_RESPONSE: KSPROPERTY_BDA_DISEQC_COMMAND = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSPROPERTY_BDA_DISEQC_EVENT = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSEVENT_BDA_DISEQC_DATA_RECEIVED: KSPROPERTY_BDA_DISEQC_EVENT = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSPROPERTY_BDA_ETHERNET_FILTER = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_ETHERNET_FILTER_MULTICAST_LIST_SIZE: KSPROPERTY_BDA_ETHERNET_FILTER = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_ETHERNET_FILTER_MULTICAST_LIST: KSPROPERTY_BDA_ETHERNET_FILTER = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_ETHERNET_FILTER_MULTICAST_MODE: KSPROPERTY_BDA_ETHERNET_FILTER = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSPROPERTY_BDA_FREQUENCY_FILTER = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_RF_TUNER_FREQUENCY: KSPROPERTY_BDA_FREQUENCY_FILTER = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_RF_TUNER_POLARITY: KSPROPERTY_BDA_FREQUENCY_FILTER = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_RF_TUNER_RANGE: KSPROPERTY_BDA_FREQUENCY_FILTER = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_RF_TUNER_TRANSPONDER: KSPROPERTY_BDA_FREQUENCY_FILTER = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_RF_TUNER_BANDWIDTH: KSPROPERTY_BDA_FREQUENCY_FILTER = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_RF_TUNER_FREQUENCY_MULTIPLIER: KSPROPERTY_BDA_FREQUENCY_FILTER = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_RF_TUNER_CAPS: KSPROPERTY_BDA_FREQUENCY_FILTER = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_RF_TUNER_SCAN_STATUS: KSPROPERTY_BDA_FREQUENCY_FILTER = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_RF_TUNER_STANDARD: KSPROPERTY_BDA_FREQUENCY_FILTER = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_RF_TUNER_STANDARD_MODE: KSPROPERTY_BDA_FREQUENCY_FILTER = 9i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSPROPERTY_BDA_IPv4_FILTER = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_IPv4_FILTER_MULTICAST_LIST_SIZE: KSPROPERTY_BDA_IPv4_FILTER = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_IPv4_FILTER_MULTICAST_LIST: KSPROPERTY_BDA_IPv4_FILTER = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_IPv4_FILTER_MULTICAST_MODE: KSPROPERTY_BDA_IPv4_FILTER = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSPROPERTY_BDA_IPv6_FILTER = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_IPv6_FILTER_MULTICAST_LIST_SIZE: KSPROPERTY_BDA_IPv6_FILTER = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_IPv6_FILTER_MULTICAST_LIST: KSPROPERTY_BDA_IPv6_FILTER = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_IPv6_FILTER_MULTICAST_MODE: KSPROPERTY_BDA_IPv6_FILTER = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSPROPERTY_BDA_LNB_INFO = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_LNB_LOF_LOW_BAND: KSPROPERTY_BDA_LNB_INFO = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_LNB_LOF_HIGH_BAND: KSPROPERTY_BDA_LNB_INFO = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_LNB_SWITCH_FREQUENCY: KSPROPERTY_BDA_LNB_INFO = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSPROPERTY_BDA_NULL_TRANSFORM = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_NULL_TRANSFORM_START: KSPROPERTY_BDA_NULL_TRANSFORM = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_NULL_TRANSFORM_STOP: KSPROPERTY_BDA_NULL_TRANSFORM = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSPROPERTY_BDA_PIDFILTER = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_PIDFILTER_MAP_PIDS: KSPROPERTY_BDA_PIDFILTER = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_PIDFILTER_UNMAP_PIDS: KSPROPERTY_BDA_PIDFILTER = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_PIDFILTER_LIST_PIDS: KSPROPERTY_BDA_PIDFILTER = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSPROPERTY_BDA_PIN_CONTROL = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_PIN_ID: KSPROPERTY_BDA_PIN_CONTROL = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_PIN_TYPE: KSPROPERTY_BDA_PIN_CONTROL = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSPROPERTY_BDA_PIN_EVENT = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSEVENT_BDA_PIN_CONNECTED: KSPROPERTY_BDA_PIN_EVENT = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSEVENT_BDA_PIN_DISCONNECTED: KSPROPERTY_BDA_PIN_EVENT = 1i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub struct KSPROPERTY_BDA_RF_TUNER_CAPS_S {
    pub Property: super::KernelStreaming::KSP_NODE,
    pub Mode: u32,
    pub AnalogStandardsSupported: u32,
    pub DigitalStandardsSupported: u32,
    pub MinFrequency: u32,
    pub MaxFrequency: u32,
    pub SettlingTime: u32,
    pub AnalogSensingRange: u32,
    pub DigitalSensingRange: u32,
    pub MilliSecondsPerMHz: u32,
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSPROPERTY_BDA_RF_TUNER_CAPS_S {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSPROPERTY_BDA_RF_TUNER_CAPS_S {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub struct KSPROPERTY_BDA_RF_TUNER_SCAN_STATUS_S {
    pub Property: super::KernelStreaming::KSP_NODE,
    pub CurrentFrequency: u32,
    pub FrequencyRangeMin: u32,
    pub FrequencyRangeMax: u32,
    pub MilliSecondsLeft: u32,
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSPROPERTY_BDA_RF_TUNER_SCAN_STATUS_S {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSPROPERTY_BDA_RF_TUNER_SCAN_STATUS_S {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_KernelStreaming"))]
pub struct KSPROPERTY_BDA_RF_TUNER_STANDARD_MODE_S {
    pub Property: super::KernelStreaming::KSP_NODE,
    pub AutoDetect: super::super::Foundation::BOOL,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_KernelStreaming"))]
impl ::core::marker::Copy for KSPROPERTY_BDA_RF_TUNER_STANDARD_MODE_S {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_KernelStreaming"))]
impl ::core::clone::Clone for KSPROPERTY_BDA_RF_TUNER_STANDARD_MODE_S {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub struct KSPROPERTY_BDA_RF_TUNER_STANDARD_S {
    pub Property: super::KernelStreaming::KSP_NODE,
    pub SignalType: BDA_SignalType,
    pub SignalStandard: u32,
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSPROPERTY_BDA_RF_TUNER_STANDARD_S {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSPROPERTY_BDA_RF_TUNER_STANDARD_S {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSPROPERTY_BDA_SIGNAL_STATS = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_SIGNAL_STRENGTH: KSPROPERTY_BDA_SIGNAL_STATS = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_SIGNAL_QUALITY: KSPROPERTY_BDA_SIGNAL_STATS = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_SIGNAL_PRESENT: KSPROPERTY_BDA_SIGNAL_STATS = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_SIGNAL_LOCKED: KSPROPERTY_BDA_SIGNAL_STATS = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_SAMPLE_TIME: KSPROPERTY_BDA_SIGNAL_STATS = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_SIGNAL_LOCK_CAPS: KSPROPERTY_BDA_SIGNAL_STATS = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_SIGNAL_LOCK_TYPE: KSPROPERTY_BDA_SIGNAL_STATS = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSPROPERTY_BDA_TOPOLOGY = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_NODE_TYPES: KSPROPERTY_BDA_TOPOLOGY = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_PIN_TYPES: KSPROPERTY_BDA_TOPOLOGY = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_TEMPLATE_CONNECTIONS: KSPROPERTY_BDA_TOPOLOGY = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_NODE_METHODS: KSPROPERTY_BDA_TOPOLOGY = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_NODE_PROPERTIES: KSPROPERTY_BDA_TOPOLOGY = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_NODE_EVENTS: KSPROPERTY_BDA_TOPOLOGY = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_CONTROLLING_PIN_ID: KSPROPERTY_BDA_TOPOLOGY = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_NODE_DESCRIPTORS: KSPROPERTY_BDA_TOPOLOGY = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSPROPERTY_BDA_VOID_TRANSFORM = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_VOID_TRANSFORM_START: KSPROPERTY_BDA_VOID_TRANSFORM = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_VOID_TRANSFORM_STOP: KSPROPERTY_BDA_VOID_TRANSFORM = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSPROPERTY_IDS_BDA_TABLE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_BDA_TABLE_SECTION: KSPROPERTY_IDS_BDA_TABLE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type KSPROPERTY_IPSINK = u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_IPSINK_MULTICASTLIST: KSPROPERTY_IPSINK = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_IPSINK_ADAPTER_DESCRIPTION: KSPROPERTY_IPSINK = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const KSPROPERTY_IPSINK_ADAPTER_ADDRESS: KSPROPERTY_IPSINK = 2u32;
pub const KSPROPSETID_BdaAutodemodulate: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3723582226, data2: 48421, data3: 4562, data4: [156, 160, 0, 192, 79, 121, 113, 224] };
pub const KSPROPSETID_BdaCA: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2959685478, data2: 21112, data3: 20166, data4: [185, 225, 60, 228, 5, 96, 239, 90] };
pub const KSPROPSETID_BdaDigitalDemodulator: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4012962681, data2: 39003, data3: 19728, data4: [182, 64, 167, 157, 94, 4, 225, 224] };
pub const KSPROPSETID_BdaDiseqCommand: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4165872304, data2: 15467, data3: 17891, data4: [160, 252, 134, 105, 212, 184, 31, 17] };
pub const KSPROPSETID_BdaEthernetFilter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1905811267, data2: 7329, data3: 4563, data4: [156, 200, 0, 192, 79, 121, 113, 224] };
pub const KSPROPSETID_BdaFrequencyFilter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1905811271, data2: 7329, data3: 4563, data4: [156, 200, 0, 192, 79, 121, 113, 224] };
pub const KSPROPSETID_BdaIPv4Filter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1905811268, data2: 7329, data3: 4563, data4: [156, 200, 0, 192, 79, 121, 113, 224] };
pub const KSPROPSETID_BdaIPv6Filter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3782761076, data2: 10787, data3: 20403, data4: [146, 69, 168, 248, 128, 23, 239, 51] };
pub const KSPROPSETID_BdaLNBInfo: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2569859330, data2: 18937, data3: 18201, data4: [166, 100, 196, 242, 62, 36, 8, 244] };
pub const KSPROPSETID_BdaNullTransform: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3723582221, data2: 48421, data3: 4562, data4: [156, 160, 0, 192, 79, 121, 113, 224] };
pub const KSPROPSETID_BdaPIDFilter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3500572005, data2: 2271, data3: 20460, data4: [133, 51, 229, 181, 80, 65, 11, 133] };
pub const KSPROPSETID_BdaPinControl: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 233654741, data2: 43191, data3: 19805, data4: [151, 161, 18, 176, 193, 149, 135, 77] };
pub const KSPROPSETID_BdaSignalStats: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 323473670, data2: 53050, data3: 17034, data4: [165, 203, 172, 13, 154, 42, 67, 56] };
pub const KSPROPSETID_BdaTableSection: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1366006213, data2: 38684, data3: 19119, data4: [179, 243, 217, 253, 168, 161, 94, 22] };
pub const KSPROPSETID_BdaTopology: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2706303029, data2: 2595, data3: 4563, data4: [156, 199, 0, 192, 79, 121, 113, 224] };
pub const KSPROPSETID_BdaVoidTransform: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1905811270, data2: 7329, data3: 4563, data4: [156, 200, 0, 192, 79, 121, 113, 224] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub struct KSP_BDA_NODE_PIN {
    pub Property: super::KernelStreaming::KSIDENTIFIER,
    pub ulNodeType: u32,
    pub ulInputPinId: u32,
    pub ulOutputPinId: u32,
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSP_BDA_NODE_PIN {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSP_BDA_NODE_PIN {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub struct KSP_NODE_ESPID {
    pub Property: super::KernelStreaming::KSP_NODE,
    pub EsPid: u32,
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KSP_NODE_ESPID {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KSP_NODE_ESPID {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct KS_BDA_FRAME_INFO {
    pub ExtendedHeaderSize: u32,
    pub dwFrameFlags: u32,
    pub ulEvent: u32,
    pub ulChannelNumber: u32,
    pub ulSubchannelNumber: u32,
    pub ulReason: u32,
}
impl ::core::marker::Copy for KS_BDA_FRAME_INFO {}
impl ::core::clone::Clone for KS_BDA_FRAME_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub struct KS_DATARANGE_BDA_ANTENNA {
    pub DataRange: super::KernelStreaming::KSDATAFORMAT,
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KS_DATARANGE_BDA_ANTENNA {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KS_DATARANGE_BDA_ANTENNA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_KernelStreaming\"`*"]
#[cfg(feature = "Win32_Media_KernelStreaming")]
pub struct KS_DATARANGE_BDA_TRANSPORT {
    pub DataRange: super::KernelStreaming::KSDATAFORMAT,
    pub BdaTransportInfo: BDA_TRANSPORT_INFO,
}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::marker::Copy for KS_DATARANGE_BDA_TRANSPORT {}
#[cfg(feature = "Win32_Media_KernelStreaming")]
impl ::core::clone::Clone for KS_DATARANGE_BDA_TRANSPORT {
    fn clone(&self) -> Self {
        *self
    }
}
pub const LIBID_QuartzNetTypeLib: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1453877425, data2: 2772, data3: 4558, data4: [176, 58, 0, 32, 175, 11, 167, 112] };
pub const LIBID_QuartzTypeLib: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1453877424, data2: 2772, data3: 4558, data4: [176, 58, 0, 32, 175, 11, 167, 112] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type LNB_Source = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_LNB_SOURCE_NOT_SET: LNB_Source = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_LNB_SOURCE_NOT_DEFINED: LNB_Source = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_LNB_SOURCE_A: LNB_Source = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_LNB_SOURCE_B: LNB_Source = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_LNB_SOURCE_C: LNB_Source = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_LNB_SOURCE_D: LNB_Source = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_LNB_SOURCE_MAX: LNB_Source = 5i32;
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct LONG_SECTION {
    pub TableId: u8,
    pub Header: LONG_SECTION_0,
    pub TableIdExtension: u16,
    pub Version: LONG_SECTION_1,
    pub SectionNumber: u8,
    pub LastSectionNumber: u8,
    pub RemainingData: [u8; 1],
}
impl ::core::marker::Copy for LONG_SECTION {}
impl ::core::clone::Clone for LONG_SECTION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub union LONG_SECTION_0 {
    pub S: MPEG_HEADER_BITS_MIDL,
    pub W: u16,
}
impl ::core::marker::Copy for LONG_SECTION_0 {}
impl ::core::clone::Clone for LONG_SECTION_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub union LONG_SECTION_1 {
    pub S: MPEG_HEADER_VERSION_BITS_MIDL,
    pub B: u8,
}
impl ::core::marker::Copy for LONG_SECTION_1 {}
impl ::core::clone::Clone for LONG_SECTION_1 {
    fn clone(&self) -> Self {
        *self
    }
}
pub const LanguageComponentType: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 467967792, data2: 3611, data3: 4563, data4: [157, 142, 0, 192, 79, 114, 217, 128] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct LanguageInfo {
    pub LangID: u16,
    pub lISOLangCode: i32,
}
impl ::core::marker::Copy for LanguageInfo {}
impl ::core::clone::Clone for LanguageInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type LicenseEventBlockReason = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const LIC_BadLicense: LicenseEventBlockReason = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const LIC_NeedIndiv: LicenseEventBlockReason = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const LIC_Expired: LicenseEventBlockReason = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const LIC_NeedActivation: LicenseEventBlockReason = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const LIC_ExtenderBlocked: LicenseEventBlockReason = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type LocationCodeSchemeType = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SCTE_18: LocationCodeSchemeType = 0i32;
pub const Locator: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 143181955, data2: 44111, data3: 18755, data4: [181, 22, 44, 56, 217, 179, 69, 98] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MAX_COUNTRY_CODE_STRING: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MAX_DEINTERLACE_DEVICE_GUIDS: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MAX_DEINTERLACE_SURFACES: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MAX_ERROR_TEXT_LEN: u32 = 160u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MAX_FILTER_NAME: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MAX_PIN_NAME: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MAX_SIZE_MPEG1_SEQUENCE_INFO: u32 = 140u32;
pub const MEDIASUBTYPE_CPFilters_Processed: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1185791272, data2: 28624, data3: 18326, data4: [147, 178, 21, 92, 81, 220, 4, 141] };
pub const MEDIASUBTYPE_ETDTFilter_Tagged: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229776, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
pub const MEDIATYPE_MPEG2_PACK: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 911358739, data2: 36581, data3: 4561, data4: [140, 163, 0, 96, 176, 87, 102, 74] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type MEDIA_SAMPLE_CONTENT = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MEDIA_TRANSPORT_PACKET: MEDIA_SAMPLE_CONTENT = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MEDIA_ELEMENTARY_STREAM: MEDIA_SAMPLE_CONTENT = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MEDIA_MPEG2_PSI: MEDIA_SAMPLE_CONTENT = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MEDIA_TRANSPORT_PAYLOAD: MEDIA_SAMPLE_CONTENT = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MIN_DIMENSION: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type MMSSF_GET_INFORMATION_FLAGS = u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MMSSF_HASCLOCK: MMSSF_GET_INFORMATION_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MMSSF_SUPPORTSEEK: MMSSF_GET_INFORMATION_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MMSSF_ASYNCHRONOUS: MMSSF_GET_INFORMATION_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPBOOL_FALSE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPBOOL_TRUE: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct MPEG1VIDEOINFO {
    pub hdr: VIDEOINFOHEADER,
    pub dwStartTimeCode: u32,
    pub cbSequenceHeader: u32,
    pub bSequenceHeader: [u8; 1],
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for MPEG1VIDEOINFO {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for MPEG1VIDEOINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_Audio\"`*"]
#[cfg(feature = "Win32_Media_Audio")]
pub struct MPEG1WAVEFORMAT {
    pub wfx: super::Audio::WAVEFORMATEX,
    pub fwHeadLayer: u16,
    pub dwHeadBitrate: u32,
    pub fwHeadMode: u16,
    pub fwHeadModeExt: u16,
    pub wHeadEmphasis: u16,
    pub fwHeadFlags: u16,
    pub dwPTSLow: u32,
    pub dwPTSHigh: u32,
}
#[cfg(feature = "Win32_Media_Audio")]
impl ::core::marker::Copy for MPEG1WAVEFORMAT {}
#[cfg(feature = "Win32_Media_Audio")]
impl ::core::clone::Clone for MPEG1WAVEFORMAT {
    fn clone(&self) -> Self {
        *self
    }
}
pub const MPEG2Component: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 89961175, data2: 10601, data3: 17869, data4: [145, 75, 118, 137, 7, 34, 241, 18] };
pub const MPEG2ComponentType: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1098909939, data2: 53095, data3: 18024, data4: [150, 40, 16, 220, 82, 190, 29, 8] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type MPEG2StreamType = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_UNITIALIZED_MPEG2STREAMTYPE: MPEG2StreamType = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const Reserved1: MPEG2StreamType = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISO_IEC_11172_2_VIDEO: MPEG2StreamType = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISO_IEC_13818_2_VIDEO: MPEG2StreamType = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISO_IEC_11172_3_AUDIO: MPEG2StreamType = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISO_IEC_13818_3_AUDIO: MPEG2StreamType = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISO_IEC_13818_1_PRIVATE_SECTION: MPEG2StreamType = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISO_IEC_13818_1_PES: MPEG2StreamType = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISO_IEC_13522_MHEG: MPEG2StreamType = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ANNEX_A_DSM_CC: MPEG2StreamType = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ITU_T_REC_H_222_1: MPEG2StreamType = 9i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISO_IEC_13818_6_TYPE_A: MPEG2StreamType = 10i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISO_IEC_13818_6_TYPE_B: MPEG2StreamType = 11i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISO_IEC_13818_6_TYPE_C: MPEG2StreamType = 12i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISO_IEC_13818_6_TYPE_D: MPEG2StreamType = 13i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISO_IEC_13818_1_AUXILIARY: MPEG2StreamType = 14i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISO_IEC_13818_7_AUDIO: MPEG2StreamType = 15i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISO_IEC_14496_2_VISUAL: MPEG2StreamType = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISO_IEC_14496_3_AUDIO: MPEG2StreamType = 17i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISO_IEC_14496_1_IN_PES: MPEG2StreamType = 18i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISO_IEC_14496_1_IN_SECTION: MPEG2StreamType = 19i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISO_IEC_13818_6_DOWNLOAD: MPEG2StreamType = 20i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const METADATA_IN_PES: MPEG2StreamType = 21i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const METADATA_IN_SECTION: MPEG2StreamType = 22i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const METADATA_IN_DATA_CAROUSEL: MPEG2StreamType = 23i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const METADATA_IN_OBJECT_CAROUSEL: MPEG2StreamType = 24i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const METADATA_IN_DOWNLOAD_PROTOCOL: MPEG2StreamType = 25i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const IRPM_STREAMM: MPEG2StreamType = 26i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ITU_T_H264: MPEG2StreamType = 27i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISO_IEC_13818_1_RESERVED: MPEG2StreamType = 28i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const USER_PRIVATE: MPEG2StreamType = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const HEVC_VIDEO_OR_TEMPORAL_VIDEO: MPEG2StreamType = 36i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const HEVC_TEMPORAL_VIDEO_SUBSET: MPEG2StreamType = 37i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ISO_IEC_USER_PRIVATE: MPEG2StreamType = 128i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DOLBY_AC3_AUDIO: MPEG2StreamType = 129i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DOLBY_DIGITAL_PLUS_AUDIO_ATSC: MPEG2StreamType = 135i32;
pub const MPEG2TuneRequest: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 156609634, data2: 48942, data3: 19642, data4: [162, 185, 166, 63, 119, 45, 70, 207] };
pub const MPEG2TuneRequestFactory: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 744744171, data2: 19690, data3: 16824, data4: [145, 156, 233, 71, 234, 25, 167, 124] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct MPEG2VIDEOINFO {
    pub hdr: VIDEOINFOHEADER2,
    pub dwStartTimeCode: u32,
    pub cbSequenceHeader: u32,
    pub dwProfile: u32,
    pub dwLevel: u32,
    pub dwFlags: MPEG2VIDEOINFO_FLAGS,
    pub dwSequenceHeader: [u32; 1],
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for MPEG2VIDEOINFO {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for MPEG2VIDEOINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type MPEG2VIDEOINFO_FLAGS = u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMMPEG2_DoPanScan: MPEG2VIDEOINFO_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMMPEG2_DVDLine21Field1: MPEG2VIDEOINFO_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMMPEG2_DVDLine21Field2: MPEG2VIDEOINFO_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMMPEG2_SourceIsLetterboxed: MPEG2VIDEOINFO_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMMPEG2_FilmCameraMode: MPEG2VIDEOINFO_FLAGS = 16u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMMPEG2_LetterboxAnalogOut: MPEG2VIDEOINFO_FLAGS = 32u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMMPEG2_DSS_UserData: MPEG2VIDEOINFO_FLAGS = 64u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMMPEG2_DVB_UserData: MPEG2VIDEOINFO_FLAGS = 128u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMMPEG2_27MhzTimebase: MPEG2VIDEOINFO_FLAGS = 256u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMMPEG2_WidescreenAnalogOut: MPEG2VIDEOINFO_FLAGS = 512u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_BASE: u32 = 512u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_ALREADY_INITIALIZED: ::windows_sys::core::HRESULT = -2147220991i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_BUFFER_TOO_SMALL: ::windows_sys::core::HRESULT = -2147220967i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_DATA_SOURCE_FAILED: ::windows_sys::core::HRESULT = -2147220970i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_DII_NOT_FOUND: ::windows_sys::core::HRESULT = -2147220969i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_DSHOW_PIN_NOT_FOUND: ::windows_sys::core::HRESULT = -2147220968i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_DSI_NOT_FOUND: ::windows_sys::core::HRESULT = -2147220982i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_FILE_OFFSET_TOO_BIG: ::windows_sys::core::HRESULT = -2147220974i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_INCORRECT_DESCRIPTOR_TAG: ::windows_sys::core::HRESULT = -2147220963i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_INVALID_CAROUSEL_ID: ::windows_sys::core::HRESULT = -2147220980i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_INVALID_SG_OBJECT_KIND: ::windows_sys::core::HRESULT = -2147220978i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_INVALID_UDP_PORT: ::windows_sys::core::HRESULT = -2147220971i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_MALFORMED_DSMCC_MESSAGE: ::windows_sys::core::HRESULT = -2147220979i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_MALFORMED_TABLE: ::windows_sys::core::HRESULT = -2147220989i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_MISSING_SECTIONS: ::windows_sys::core::HRESULT = -2147220966i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_NEXT_TABLE_OPS_NOT_AVAILABLE: ::windows_sys::core::HRESULT = -2147220964i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_NOT_PRESENT: ::windows_sys::core::HRESULT = -2147220987i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_OBJECT_KIND_NOT_A_DIRECTORY: ::windows_sys::core::HRESULT = -2147220976i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_OBJECT_KIND_NOT_A_FILE: ::windows_sys::core::HRESULT = -2147220975i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_OBJECT_NOT_FOUND: ::windows_sys::core::HRESULT = -2147220977i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_OUT_OF_BOUNDS: ::windows_sys::core::HRESULT = -2147220990i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_REGISTRY_ACCESS_FAILED: ::windows_sys::core::HRESULT = -2147220972i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_SECTION_NOT_FOUND: ::windows_sys::core::HRESULT = -2147220986i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_SERVER_UNAVAILABLE: ::windows_sys::core::HRESULT = -2147220981i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_SERVICE_ID_NOT_FOUND: ::windows_sys::core::HRESULT = -2147220984i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_SERVICE_PMT_NOT_FOUND: ::windows_sys::core::HRESULT = -2147220983i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_STREAM_STOPPED: ::windows_sys::core::HRESULT = -2147220973i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_TOO_MANY_SECTIONS: ::windows_sys::core::HRESULT = -2147220965i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_TX_STREAM_UNAVAILABLE: ::windows_sys::core::HRESULT = -2147220985i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_UNDEFINED: ::windows_sys::core::HRESULT = -2147220988i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_E_UNINITIALIZED: ::windows_sys::core::HRESULT = -2147220992i32;
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct MPEG2_FILTER {
    pub bVersionNumber: u8,
    pub wFilterSize: u16,
    pub fUseRawFilteringBits: super::super::Foundation::BOOL,
    pub Filter: [u8; 16],
    pub Mask: [u8; 16],
    pub fSpecifyTableIdExtension: super::super::Foundation::BOOL,
    pub TableIdExtension: u16,
    pub fSpecifyVersion: super::super::Foundation::BOOL,
    pub Version: u8,
    pub fSpecifySectionNumber: super::super::Foundation::BOOL,
    pub SectionNumber: u8,
    pub fSpecifyCurrentNext: super::super::Foundation::BOOL,
    pub fNext: super::super::Foundation::BOOL,
    pub fSpecifyDsmccOptions: super::super::Foundation::BOOL,
    pub Dsmcc: DSMCC_FILTER_OPTIONS,
    pub fSpecifyAtscOptions: super::super::Foundation::BOOL,
    pub Atsc: ATSC_FILTER_OPTIONS,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for MPEG2_FILTER {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for MPEG2_FILTER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct MPEG2_FILTER2 {
    pub Anonymous: MPEG2_FILTER2_0,
    pub fSpecifyDvbEitOptions: super::super::Foundation::BOOL,
    pub DvbEit: DVB_EIT_FILTER_OPTIONS,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for MPEG2_FILTER2 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for MPEG2_FILTER2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub union MPEG2_FILTER2_0 {
    pub Anonymous: MPEG2_FILTER2_0_0,
    pub bVersion1Bytes: [u8; 124],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for MPEG2_FILTER2_0 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for MPEG2_FILTER2_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct MPEG2_FILTER2_0_0 {
    pub bVersionNumber: u8,
    pub wFilterSize: u16,
    pub fUseRawFilteringBits: super::super::Foundation::BOOL,
    pub Filter: [u8; 16],
    pub Mask: [u8; 16],
    pub fSpecifyTableIdExtension: super::super::Foundation::BOOL,
    pub TableIdExtension: u16,
    pub fSpecifyVersion: super::super::Foundation::BOOL,
    pub Version: u8,
    pub fSpecifySectionNumber: super::super::Foundation::BOOL,
    pub SectionNumber: u8,
    pub fSpecifyCurrentNext: super::super::Foundation::BOOL,
    pub fNext: super::super::Foundation::BOOL,
    pub fSpecifyDsmccOptions: super::super::Foundation::BOOL,
    pub Dsmcc: DSMCC_FILTER_OPTIONS,
    pub fSpecifyAtscOptions: super::super::Foundation::BOOL,
    pub Atsc: ATSC_FILTER_OPTIONS,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for MPEG2_FILTER2_0_0 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for MPEG2_FILTER2_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_FILTER_VERSION_1_SIZE: u32 = 124u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_FILTER_VERSION_2_SIZE: u32 = 133u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_PROGRAM_DIRECTORY_PES_PACKET: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_PROGRAM_ELEMENTARY_STREAM: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_PROGRAM_PACK_HEADER: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_PROGRAM_PES_STREAM: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_PROGRAM_STREAM_MAP: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_PROGRAM_SYSTEM_HEADER: u32 = 5u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_S_MORE_DATA_AVAILABLE: ::windows_sys::core::HRESULT = 262656i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_S_MPE_INFO_FOUND: ::windows_sys::core::HRESULT = 262660i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_S_MPE_INFO_NOT_FOUND: ::windows_sys::core::HRESULT = 262661i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_S_NEW_MODULE_VERSION: ::windows_sys::core::HRESULT = 262662i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_S_NO_MORE_DATA_AVAILABLE: ::windows_sys::core::HRESULT = 262657i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_S_SG_INFO_FOUND: ::windows_sys::core::HRESULT = 262658i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG2_S_SG_INFO_NOT_FOUND: ::windows_sys::core::HRESULT = 262659i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct MPEG2_TRANSPORT_STRIDE {
    pub dwOffset: u32,
    pub dwPacketLength: u32,
    pub dwStride: u32,
}
impl ::core::marker::Copy for MPEG2_TRANSPORT_STRIDE {}
impl ::core::clone::Clone for MPEG2_TRANSPORT_STRIDE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Media_Audio\"`*"]
#[cfg(feature = "Win32_Media_Audio")]
pub struct MPEGLAYER3WAVEFORMAT {
    pub wfx: super::Audio::WAVEFORMATEX,
    pub wID: u16,
    pub fdwFlags: MPEGLAYER3WAVEFORMAT_FLAGS,
    pub nBlockSize: u16,
    pub nFramesPerBlock: u16,
    pub nCodecDelay: u16,
}
#[cfg(feature = "Win32_Media_Audio")]
impl ::core::marker::Copy for MPEGLAYER3WAVEFORMAT {}
#[cfg(feature = "Win32_Media_Audio")]
impl ::core::clone::Clone for MPEGLAYER3WAVEFORMAT {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type MPEGLAYER3WAVEFORMAT_FLAGS = u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEGLAYER3_FLAG_PADDING_ISO: MPEGLAYER3WAVEFORMAT_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEGLAYER3_FLAG_PADDING_ON: MPEGLAYER3WAVEFORMAT_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEGLAYER3_FLAG_PADDING_OFF: MPEGLAYER3WAVEFORMAT_FLAGS = 2u32;
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct MPEG_BCS_DEMUX {
    pub AVMGraphId: u32,
}
impl ::core::marker::Copy for MPEG_BCS_DEMUX {}
impl ::core::clone::Clone for MPEG_BCS_DEMUX {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG_CAT_PID: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG_CAT_TID: u32 = 1u32;
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct MPEG_CONTEXT {
    pub Type: MPEG_CONTEXT_TYPE,
    pub U: MPEG_CONTEXT_0,
}
impl ::core::marker::Copy for MPEG_CONTEXT {}
impl ::core::clone::Clone for MPEG_CONTEXT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub union MPEG_CONTEXT_0 {
    pub Demux: MPEG_BCS_DEMUX,
    pub Winsock: MPEG_WINSOCK,
}
impl ::core::marker::Copy for MPEG_CONTEXT_0 {}
impl ::core::clone::Clone for MPEG_CONTEXT_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type MPEG_CONTEXT_TYPE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG_CONTEXT_BCS_DEMUX: MPEG_CONTEXT_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG_CONTEXT_WINSOCK: MPEG_CONTEXT_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type MPEG_CURRENT_NEXT_BIT = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG_SECTION_IS_NEXT: MPEG_CURRENT_NEXT_BIT = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG_SECTION_IS_CURRENT: MPEG_CURRENT_NEXT_BIT = 1i32;
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct MPEG_DATE {
    pub Date: u8,
    pub Month: u8,
    pub Year: u16,
}
impl ::core::marker::Copy for MPEG_DATE {}
impl ::core::clone::Clone for MPEG_DATE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct MPEG_DATE_AND_TIME {
    pub D: MPEG_DATE,
    pub T: MPEG_TIME,
}
impl ::core::marker::Copy for MPEG_DATE_AND_TIME {}
impl ::core::clone::Clone for MPEG_DATE_AND_TIME {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct MPEG_HEADER_BITS {
    pub _bitfield: u16,
}
impl ::core::marker::Copy for MPEG_HEADER_BITS {}
impl ::core::clone::Clone for MPEG_HEADER_BITS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct MPEG_HEADER_BITS_MIDL {
    pub Bits: u16,
}
impl ::core::marker::Copy for MPEG_HEADER_BITS_MIDL {}
impl ::core::clone::Clone for MPEG_HEADER_BITS_MIDL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct MPEG_HEADER_VERSION_BITS {
    pub _bitfield: u8,
}
impl ::core::marker::Copy for MPEG_HEADER_VERSION_BITS {}
impl ::core::clone::Clone for MPEG_HEADER_VERSION_BITS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct MPEG_HEADER_VERSION_BITS_MIDL {
    pub Bits: u8,
}
impl ::core::marker::Copy for MPEG_HEADER_VERSION_BITS_MIDL {}
impl ::core::clone::Clone for MPEG_HEADER_VERSION_BITS_MIDL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct MPEG_PACKET_LIST {
    pub wPacketCount: u16,
    pub PacketList: [*mut MPEG_RQST_PACKET; 1],
}
impl ::core::marker::Copy for MPEG_PACKET_LIST {}
impl ::core::clone::Clone for MPEG_PACKET_LIST {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG_PAT_PID: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG_PAT_TID: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG_PMT_TID: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type MPEG_REQUEST_TYPE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG_RQST_UNKNOWN: MPEG_REQUEST_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG_RQST_GET_SECTION: MPEG_REQUEST_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG_RQST_GET_SECTION_ASYNC: MPEG_REQUEST_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG_RQST_GET_TABLE: MPEG_REQUEST_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG_RQST_GET_TABLE_ASYNC: MPEG_REQUEST_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG_RQST_GET_SECTIONS_STREAM: MPEG_REQUEST_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG_RQST_GET_PES_STREAM: MPEG_REQUEST_TYPE = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG_RQST_GET_TS_STREAM: MPEG_REQUEST_TYPE = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG_RQST_START_MPE_STREAM: MPEG_REQUEST_TYPE = 8i32;
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct MPEG_RQST_PACKET {
    pub dwLength: u32,
    pub pSection: *mut SECTION,
}
impl ::core::marker::Copy for MPEG_RQST_PACKET {}
impl ::core::clone::Clone for MPEG_RQST_PACKET {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct MPEG_SERVICE_REQUEST {
    pub Type: MPEG_REQUEST_TYPE,
    pub Context: MPEG_CONTEXT,
    pub Pid: u16,
    pub TableId: u8,
    pub Filter: MPEG2_FILTER,
    pub Flags: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for MPEG_SERVICE_REQUEST {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for MPEG_SERVICE_REQUEST {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct MPEG_SERVICE_RESPONSE {
    pub IPAddress: u32,
    pub Port: u16,
}
impl ::core::marker::Copy for MPEG_SERVICE_RESPONSE {}
impl ::core::clone::Clone for MPEG_SERVICE_RESPONSE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct MPEG_STREAM_BUFFER {
    pub hr: ::windows_sys::core::HRESULT,
    pub dwDataBufferSize: u32,
    pub dwSizeOfDataRead: u32,
    pub pDataBuffer: *mut u8,
}
impl ::core::marker::Copy for MPEG_STREAM_BUFFER {}
impl ::core::clone::Clone for MPEG_STREAM_BUFFER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct MPEG_STREAM_FILTER {
    pub wPidValue: u16,
    pub dwFilterSize: u32,
    pub fCrcEnabled: super::super::Foundation::BOOL,
    pub rgchFilter: [u8; 16],
    pub rgchMask: [u8; 16],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for MPEG_STREAM_FILTER {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for MPEG_STREAM_FILTER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct MPEG_TIME {
    pub Hours: u8,
    pub Minutes: u8,
    pub Seconds: u8,
}
impl ::core::marker::Copy for MPEG_TIME {}
impl ::core::clone::Clone for MPEG_TIME {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG_TSDT_PID: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPEG_TSDT_TID: u32 = 3u32;
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct MPEG_WINSOCK {
    pub AVMGraphId: u32,
}
impl ::core::marker::Copy for MPEG_WINSOCK {}
impl ::core::clone::Clone for MPEG_WINSOCK {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct MPE_ELEMENT {
    pub pid: u16,
    pub bComponentTag: u8,
    pub pNext: *mut MPE_ELEMENT,
}
impl ::core::marker::Copy for MPE_ELEMENT {}
impl ::core::clone::Clone for MPE_ELEMENT {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPF_ENVLP_BEGIN_CURRENTVAL: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPF_ENVLP_BEGIN_NEUTRALVAL: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPF_ENVLP_STANDARD: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPF_PUNCHIN_NOW: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPF_PUNCHIN_REFTIME: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPF_PUNCHIN_STOPPED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type MP_CURVE_TYPE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MP_CURVE_JUMP: MP_CURVE_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MP_CURVE_LINEAR: MP_CURVE_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MP_CURVE_SQUARE: MP_CURVE_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MP_CURVE_INVSQUARE: MP_CURVE_TYPE = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MP_CURVE_SINE: MP_CURVE_TYPE = 16i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct MP_ENVELOPE_SEGMENT {
    pub rtStart: i64,
    pub rtEnd: i64,
    pub valStart: f32,
    pub valEnd: f32,
    pub iCurve: MP_CURVE_TYPE,
    pub flags: u32,
}
impl ::core::marker::Copy for MP_ENVELOPE_SEGMENT {}
impl ::core::clone::Clone for MP_ENVELOPE_SEGMENT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct MP_PARAMINFO {
    pub mpType: MP_TYPE,
    pub mopCaps: u32,
    pub mpdMinValue: f32,
    pub mpdMaxValue: f32,
    pub mpdNeutralValue: f32,
    pub szUnitText: [u16; 32],
    pub szLabel: [u16; 32],
}
impl ::core::marker::Copy for MP_PARAMINFO {}
impl ::core::clone::Clone for MP_PARAMINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type MP_TYPE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPT_INT: MP_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPT_FLOAT: MP_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPT_BOOL: MP_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPT_ENUM: MP_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MPT_MAX: MP_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MSDRI_S_MMI_PENDING: ::windows_sys::core::HRESULT = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MSDRI_S_PENDING: ::windows_sys::core::HRESULT = 1i32;
pub const MSEventBinder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1467984408, data2: 17688, data3: 17502, data4: [143, 112, 20, 115, 248, 207, 75, 164] };
pub const MSPID_PrimaryAudio: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2740974955, data2: 40922, data3: 4560, data4: [143, 223, 0, 192, 79, 217, 24, 157] };
pub const MSPID_PrimaryVideo: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2740974954, data2: 40922, data3: 4560, data4: [143, 223, 0, 192, 79, 217, 24, 157] };
pub const MSVidAnalogCaptureToCCA: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2485877001, data2: 41614, data3: 18849, data4: [162, 7, 52, 235, 203, 203, 75, 59] };
pub const MSVidAnalogCaptureToDataServices: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3312463062, data2: 39801, data3: 4563, data4: [182, 84, 0, 192, 79, 121, 73, 142] };
pub const MSVidAnalogCaptureToOverlayMixer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3783980890, data2: 2223, data3: 4563, data4: [182, 74, 0, 192, 79, 121, 73, 142] };
pub const MSVidAnalogCaptureToStreamBufferSink: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2672879793, data2: 38192, data3: 19932, data4: [130, 94, 26, 248, 29, 71, 174, 214] };
pub const MSVidAnalogCaptureToXDS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 893441088, data2: 23325, data3: 18891, data4: [130, 26, 232, 75, 140, 240, 101, 167] };
pub const MSVidAnalogTVToEncoder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 680867425, data2: 561, data3: 16859, data4: [137, 134, 33, 255, 67, 136, 238, 155] };
pub const MSVidAnalogTunerDevice: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 471192708, data2: 37149, data3: 4562, data4: [182, 50, 0, 192, 79, 121, 73, 142] };
pub const MSVidAudioRenderer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 934294852, data2: 42184, data3: 4562, data4: [182, 52, 0, 192, 79, 121, 73, 142] };
pub const MSVidAudioRendererDevices: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3312463055, data2: 39801, data3: 4563, data4: [182, 84, 0, 192, 79, 121, 73, 142] };
pub const MSVidBDATunerDevice: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2732787534, data2: 27709, data3: 4563, data4: [182, 83, 0, 192, 79, 121, 73, 142] };
pub const MSVidCCA: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2249529383, data2: 58491, data3: 17902, data4: [132, 33, 209, 14, 110, 105, 9, 121] };
pub const MSVidCCAToStreamBufferSink: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1056402792, data2: 34401, data3: 18499, data4: [139, 143, 195, 113, 99, 216, 201, 206] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
#[repr(transparent)]
pub struct MSVidCCService(pub i32);
impl MSVidCCService {
    pub const None: Self = Self(0i32);
    pub const Caption1: Self = Self(1i32);
    pub const Caption2: Self = Self(2i32);
    pub const Text1: Self = Self(3i32);
    pub const Text2: Self = Self(4i32);
    pub const XDS: Self = Self(5i32);
}
impl ::core::marker::Copy for MSVidCCService {}
impl ::core::clone::Clone for MSVidCCService {
    fn clone(&self) -> Self {
        *self
    }
}
pub const MSVidCCToAR: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3613603018, data2: 55454, data3: 19375, data4: [134, 171, 221, 181, 147, 114, 175, 194] };
pub const MSVidCCToVMR: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3300861828, data2: 44544, data3: 16826, data4: [152, 40, 156, 149, 59, 211, 197, 74] };
pub const MSVidClosedCaptioning: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2140975437, data2: 18660, data3: 17334, data4: [147, 70, 26, 235, 195, 156, 100, 211] };
pub const MSVidClosedCaptioningSI: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2465040575, data2: 34718, data3: 17551, data4: [182, 182, 163, 133, 188, 235, 132, 109] };
pub const MSVidCtl: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2968383843, data2: 37130, data3: 4562, data4: [182, 50, 0, 192, 79, 121, 73, 142] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type MSVidCtlButtonstate = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MSVIDCTL_LEFT_BUTTON: MSVidCtlButtonstate = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MSVIDCTL_RIGHT_BUTTON: MSVidCtlButtonstate = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MSVIDCTL_MIDDLE_BUTTON: MSVidCtlButtonstate = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MSVIDCTL_X_BUTTON1: MSVidCtlButtonstate = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MSVIDCTL_X_BUTTON2: MSVidCtlButtonstate = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MSVIDCTL_SHIFT: MSVidCtlButtonstate = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MSVIDCTL_CTRL: MSVidCtlButtonstate = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MSVIDCTL_ALT: MSVidCtlButtonstate = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type MSVidCtlStateList = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STATE_UNBUILT: MSVidCtlStateList = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STATE_STOP: MSVidCtlStateList = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STATE_PAUSE: MSVidCtlStateList = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STATE_PLAY: MSVidCtlStateList = 2i32;
pub const MSVidDataServices: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 859907520, data2: 30693, data3: 4563, data4: [182, 83, 0, 192, 79, 121, 73, 142] };
pub const MSVidDataServicesToStreamBufferSink: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 955266086, data2: 59451, data3: 20072, data4: [182, 91, 220, 174, 115, 48, 72, 56] };
pub const MSVidDataServicesToXDS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 69856366, data2: 4420, data3: 19437, data4: [184, 139, 47, 185, 137, 154, 74, 61] };
pub const MSVidDevice: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1849706351, data2: 40009, data3: 19518, data4: [139, 185, 133, 135, 149, 142, 255, 116] };
pub const MSVidDevice2: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 815366013, data2: 46005, data3: 18972, data4: [152, 58, 31, 232, 9, 140, 183, 125] };
pub const MSVidDigitalCaptureToCCA: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1943093815, data2: 47579, data3: 20218, data4: [166, 221, 132, 53, 4, 33, 251, 47] };
pub const MSVidDigitalCaptureToITV: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1569616887, data2: 18825, data3: 19144, data4: [138, 152, 57, 186, 13, 50, 83, 2] };
pub const MSVidDigitalCaptureToStreamBufferSink: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2883846197, data2: 10179, data3: 18991, data4: [129, 83, 102, 36, 71, 22, 8, 175] };
pub const MSVidEVR: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3293735074, data2: 64129, data3: 19993, data4: [177, 227, 114, 237, 189, 96, 174, 218] };
pub const MSVidEncoder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3142782051, data2: 55775, data3: 19273, data4: [148, 57, 99, 69, 57, 98, 229, 152] };
pub const MSVidEncoderToStreamBufferSink: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2696524951, data2: 44988, data3: 17837, data4: [168, 166, 155, 7, 124, 64, 212, 242] };
pub const MSVidFeature: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2001228555, data2: 49290, data3: 18410, data4: [178, 76, 190, 134, 149, 255, 64, 95] };
pub const MSVidFeatures: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3312463056, data2: 39801, data3: 4563, data4: [182, 84, 0, 192, 79, 121, 73, 142] };
pub const MSVidFilePlaybackDevice: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 934294844, data2: 42184, data3: 4562, data4: [182, 52, 0, 192, 79, 121, 73, 142] };
pub const MSVidFilePlaybackToAudioRenderer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3424908599, data2: 6356, data3: 20174, data4: [147, 189, 32, 122, 132, 114, 105, 121] };
pub const MSVidFilePlaybackToVideoRenderer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3020015083, data2: 33879, data3: 17023, data4: [132, 234, 164, 210, 54, 51, 100, 176] };
pub const MSVidGenericComposite: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 660913381, data2: 52281, data3: 4562, data4: [182, 57, 0, 192, 79, 121, 73, 142] };
pub const MSVidGenericSink: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1247308239, data2: 37533, data3: 16448, data4: [174, 3, 252, 175, 197, 185, 205, 66] };
pub const MSVidITVCapture: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1463853826, data2: 61195, data3: 17870, data4: [191, 59, 68, 112, 161, 74, 137, 128] };
pub const MSVidITVPlayback: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2658762448, data2: 21075, data3: 16963, data4: [169, 183, 189, 6, 197, 143, 142, 243] };
pub const MSVidITVToStreamBufferSink: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2461616168, data2: 6903, data3: 20078, data4: [158, 191, 119, 6, 87, 247, 122, 245] };
pub const MSVidInputDevice: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2887348978, data2: 5002, data3: 19619, data4: [144, 218, 174, 81, 17, 46, 218, 40] };
pub const MSVidInputDevices: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3312463052, data2: 39801, data3: 4563, data4: [182, 84, 0, 192, 79, 121, 73, 142] };
pub const MSVidMPEG2DecoderToClosedCaptioning: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1792184033, data2: 20482, data3: 20081, data4: [170, 247, 189, 7, 121, 7, 177, 164] };
pub const MSVidOutput: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2280360205, data2: 941, data3: 20125, data4: [152, 102, 55, 110, 94, 197, 114, 237] };
pub const MSVidOutputDevices: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3312463053, data2: 39801, data3: 4563, data4: [182, 84, 0, 192, 79, 121, 73, 142] };
pub const MSVidRect: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3410130662, data2: 32095, data3: 19697, data4: [151, 39, 98, 156, 94, 109, 182, 174] };
pub const MSVidSBESourceToCC: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2442373369, data2: 3258, data3: 16398, data4: [170, 151, 235, 71, 9, 22, 69, 118] };
pub const MSVidSBESourceToGenericSink: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2568857573, data2: 38207, data3: 17243, data4: [190, 94, 185, 42, 5, 237, 252, 66] };
pub const MSVidSBESourceToITV: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 579946380, data2: 24291, data3: 19439, data4: [171, 93, 181, 255, 44, 245, 131, 82] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type MSVidSegmentType = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MSVidSEG_SOURCE: MSVidSegmentType = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MSVidSEG_XFORM: MSVidSegmentType = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MSVidSEG_DEST: MSVidSegmentType = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type MSVidSinkStreams = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MSVidSink_Video: MSVidSinkStreams = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MSVidSink_Audio: MSVidSinkStreams = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MSVidSink_Other: MSVidSinkStreams = 4i32;
pub const MSVidStreamBufferRecordingControl: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3400523139, data2: 52988, data3: 20029, data4: [186, 3, 23, 95, 23, 162, 79, 145] };
pub const MSVidStreamBufferSink: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2658642628, data2: 13797, data3: 17057, data4: [189, 194, 143, 63, 243, 153, 132, 124] };
pub const MSVidStreamBufferSource: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2911785229, data2: 8575, data3: 16539, data4: [128, 118, 41, 197, 231, 59, 152, 232] };
pub const MSVidStreamBufferSourceToVideoRenderer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1011288284, data2: 45441, data3: 18088, data4: [141, 168, 74, 176, 55, 23, 88, 205] };
pub const MSVidStreamBufferV2Source: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4248116897, data2: 16755, data3: 19188, data4: [130, 29, 128, 212, 174, 151, 144, 72] };
pub const MSVidVMR9: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 618412405, data2: 2495, data3: 16945, data4: [134, 85, 62, 231, 31, 67, 131, 125] };
pub const MSVidVideoInputDevice: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2515829259, data2: 47930, data3: 20013, data4: [188, 100, 91, 129, 123, 194, 195, 14] };
pub const MSVidVideoPlaybackDevice: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 428922420, data2: 6750, data3: 16497, data4: [163, 74, 83, 170, 255, 206, 159, 54] };
pub const MSVidVideoRenderer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 934294851, data2: 42184, data3: 4562, data4: [182, 52, 0, 192, 79, 121, 73, 142] };
pub const MSVidVideoRendererDevices: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3312463054, data2: 39801, data3: 4563, data4: [182, 84, 0, 192, 79, 121, 73, 142] };
pub const MSVidWebDVD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 18560537, data2: 65123, data3: 18452, data4: [138, 132, 21, 161, 148, 206, 156, 227] };
pub const MSVidWebDVDAdm: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4202444635, data2: 26279, data3: 17024, data4: [135, 157, 253, 69, 156, 132, 187, 2] };
pub const MSVidWebDVDToAudioRenderer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2365858702, data2: 40913, data3: 16838, data4: [141, 227, 158, 30, 227, 9, 233, 53] };
pub const MSVidWebDVDToVideoRenderer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 645771443, data2: 21987, data3: 18690, data4: [148, 155, 223, 143, 92, 236, 1, 145] };
pub const MSVidXDS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 21622495, data2: 53391, data3: 16706, data4: [141, 115, 210, 57, 3, 210, 30, 144] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type MSViddispidList = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidInputs: MSViddispidList = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidOutputs: MSViddispidList = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_Inputs: MSViddispidList = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_Outputs: MSViddispidList = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidVideoRenderers: MSViddispidList = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidAudioRenderers: MSViddispidList = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidFeatures: MSViddispidList = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidInput: MSViddispidList = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidOutput: MSViddispidList = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidVideoRenderer: MSViddispidList = 9i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidAudioRenderer: MSViddispidList = 10i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidSelectedFeatures: MSViddispidList = 11i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidView: MSViddispidList = 12i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidBuild: MSViddispidList = 13i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidPause: MSViddispidList = 14i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidRun: MSViddispidList = 15i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidStop: MSViddispidList = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidDecompose: MSViddispidList = 17i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidDisplaySize: MSViddispidList = 18i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidMaintainAspectRatio: MSViddispidList = 19i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidColorKey: MSViddispidList = 20i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidStateChange: MSViddispidList = 21i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidgetState: MSViddispidList = 22i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidunbind: MSViddispidList = 23i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidbind: MSViddispidList = 24i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidDisableVideo: MSViddispidList = 25i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidDisableAudio: MSViddispidList = 26i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidViewNext: MSViddispidList = 27i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidServiceP: MSViddispidList = 28i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type MUX_PID_TYPE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PID_OTHER: MUX_PID_TYPE = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PID_ELEMENTARY_STREAM: MUX_PID_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PID_MPEG2_SECTION_PSI_SI: MUX_PID_TYPE = 1i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct MainAVIHeader {
    pub dwMicroSecPerFrame: u32,
    pub dwMaxBytesPerSec: u32,
    pub dwPaddingGranularity: u32,
    pub dwFlags: u32,
    pub dwTotalFrames: u32,
    pub dwInitialFrames: u32,
    pub dwStreams: u32,
    pub dwSuggestedBufferSize: u32,
    pub dwWidth: u32,
    pub dwHeight: u32,
    pub dwReserved: [u32; 4],
}
impl ::core::marker::Copy for MainAVIHeader {}
impl ::core::clone::Clone for MainAVIHeader {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type ModulationType = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_NOT_SET: ModulationType = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_NOT_DEFINED: ModulationType = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_16QAM: ModulationType = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_32QAM: ModulationType = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_64QAM: ModulationType = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_80QAM: ModulationType = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_96QAM: ModulationType = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_112QAM: ModulationType = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_128QAM: ModulationType = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_160QAM: ModulationType = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_192QAM: ModulationType = 9i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_224QAM: ModulationType = 10i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_256QAM: ModulationType = 11i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_320QAM: ModulationType = 12i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_384QAM: ModulationType = 13i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_448QAM: ModulationType = 14i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_512QAM: ModulationType = 15i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_640QAM: ModulationType = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_768QAM: ModulationType = 17i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_896QAM: ModulationType = 18i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_1024QAM: ModulationType = 19i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_QPSK: ModulationType = 20i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_BPSK: ModulationType = 21i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_OQPSK: ModulationType = 22i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_8VSB: ModulationType = 23i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_16VSB: ModulationType = 24i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_ANALOG_AMPLITUDE: ModulationType = 25i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_ANALOG_FREQUENCY: ModulationType = 26i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_8PSK: ModulationType = 27i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_RF: ModulationType = 28i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_16APSK: ModulationType = 29i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_32APSK: ModulationType = 30i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_NBC_QPSK: ModulationType = 31i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_NBC_8PSK: ModulationType = 32i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_DIRECTV: ModulationType = 33i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_ISDB_T_TMCC: ModulationType = 34i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_ISDB_S_TMCC: ModulationType = 35i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_MOD_MAX: ModulationType = 36i32;
pub const Mpeg2Data: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3328631061, data2: 47970, data3: 16423, data4: [161, 19, 130, 214, 67, 254, 45, 153] };
pub const Mpeg2DataLib: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3685706779, data2: 46756, data3: 18584, data4: [174, 101, 32, 79, 13, 149, 9, 161] };
pub const Mpeg2Stream: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4179465927, data2: 34057, data3: 19723, data4: [171, 38, 160, 221, 16, 144, 75, 183] };
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct Mpeg2TableSampleHdr {
    pub SectionCount: u8,
    pub Reserved: [u8; 3],
    pub SectionOffsets: [i32; 1],
}
impl ::core::marker::Copy for Mpeg2TableSampleHdr {}
impl ::core::clone::Clone for Mpeg2TableSampleHdr {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct NORMALIZEDRECT {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}
impl ::core::marker::Copy for NORMALIZEDRECT {}
impl ::core::clone::Clone for NORMALIZEDRECT {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type OA_BOOL = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const OATRUE: OA_BOOL = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const OAFALSE: OA_BOOL = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const OCUR_PAIRING_PROTOCOL_VERSION: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type OUTPUT_STATE = u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const Disabled: OUTPUT_STATE = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ReadData: OUTPUT_STATE = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const RenderData: OUTPUT_STATE = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PARENTAL_CONTROL_ATTRIB_DIALOGUE: u32 = 515u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PARENTAL_CONTROL_ATTRIB_FANTASY: u32 = 516u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PARENTAL_CONTROL_ATTRIB_LANGUAGE: u32 = 513u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PARENTAL_CONTROL_ATTRIB_SEXUAL: u32 = 514u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PARENTAL_CONTROL_ATTRIB_VIOLENCE: u32 = 512u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PARENTAL_CONTROL_CONTENT_RATING: u32 = 256u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PARENTAL_CONTROL_TIME_RANGE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PARENTAL_CONTROL_VALUE_UNDEFINED: u32 = 0u32;
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct PBDAParentalControl {
    pub rating_system_count: u32,
    pub rating_systems: *mut RATING_SYSTEM,
}
impl ::core::marker::Copy for PBDAParentalControl {}
impl ::core::clone::Clone for PBDAParentalControl {
    fn clone(&self) -> Self {
        *self
    }
}
pub const PBDA_ALWAYS_TUNE_IN_MUX: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 505246017, data2: 22591, data3: 19138, data4: [176, 25, 31, 67, 14, 218, 15, 76] };
pub const PBDA_AUX_CONNECTOR_TYPE_Composite: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4129917772, data2: 50981, data3: 19778, data4: [132, 155, 65, 11, 187, 20, 234, 98] };
pub const PBDA_AUX_CONNECTOR_TYPE_SVideo: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2699625972, data2: 9417, data3: 19028, data4: [183, 97, 33, 51, 85, 239, 193, 58] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PBDA_Encoder_Audio_AlgorithmType_AC3: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PBDA_Encoder_Audio_AlgorithmType_MPEG1LayerII: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PBDA_Encoder_BitrateMode_Average: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PBDA_Encoder_BitrateMode_Constant: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PBDA_Encoder_BitrateMode_Variable: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PBDA_Encoder_Video_AVC: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PBDA_Encoder_Video_H264: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PBDA_Encoder_Video_MPEG2PartII: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PBDA_Encoder_Video_MPEG4Part10: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PBDA_PAIRING_PROTOCOL_VERSION: u32 = 3u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct PBDA_TAG_ATTRIBUTE {
    pub TableUUId: ::windows_sys::core::GUID,
    pub TableId: u8,
    pub VersionNo: u16,
    pub TableDataSize: u32,
    pub TableData: [u8; 1],
}
impl ::core::marker::Copy for PBDA_TAG_ATTRIBUTE {}
impl ::core::clone::Clone for PBDA_TAG_ATTRIBUTE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Media_MediaFoundation\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct3D9", feature = "Win32_Media_MediaFoundation"))]
pub type PDXVA2SW_CREATEVIDEOPROCESSDEVICE = ::core::option::Option<unsafe extern "system" fn(pd3dd9: super::super::Graphics::Direct3D9::IDirect3DDevice9, pvideodesc: *const super::MediaFoundation::DXVA2_VideoDesc, rendertargetformat: super::super::Graphics::Direct3D9::D3DFORMAT, maxsubstreams: u32, phdevice: *mut super::super::Foundation::HANDLE) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDXVA2SW_DESTROYVIDEOPROCESSDEVICE = ::core::option::Option<unsafe extern "system" fn(hdevice: super::super::Foundation::HANDLE) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Media_MediaFoundation\"`*"]
#[cfg(all(feature = "Win32_Graphics_Direct3D9", feature = "Win32_Media_MediaFoundation"))]
pub type PDXVA2SW_GETFILTERPROPERTYRANGE = ::core::option::Option<unsafe extern "system" fn(pvideodesc: *const super::MediaFoundation::DXVA2_VideoDesc, rendertargetformat: super::super::Graphics::Direct3D9::D3DFORMAT, filtersetting: u32, prange: *mut super::MediaFoundation::DXVA2_ValueRange) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Media_MediaFoundation\"`*"]
#[cfg(all(feature = "Win32_Graphics_Direct3D9", feature = "Win32_Media_MediaFoundation"))]
pub type PDXVA2SW_GETPROCAMPRANGE = ::core::option::Option<unsafe extern "system" fn(pvideodesc: *const super::MediaFoundation::DXVA2_VideoDesc, rendertargetformat: super::super::Graphics::Direct3D9::D3DFORMAT, procampcap: u32, prange: *mut super::MediaFoundation::DXVA2_ValueRange) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Media_MediaFoundation\"`*"]
#[cfg(all(feature = "Win32_Graphics_Direct3D9", feature = "Win32_Media_MediaFoundation"))]
pub type PDXVA2SW_GETVIDEOPROCESSORCAPS = ::core::option::Option<unsafe extern "system" fn(pvideodesc: *const super::MediaFoundation::DXVA2_VideoDesc, rendertargetformat: super::super::Graphics::Direct3D9::D3DFORMAT, pcaps: *mut super::MediaFoundation::DXVA2_VideoProcessorCaps) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Media_MediaFoundation\"`*"]
#[cfg(all(feature = "Win32_Graphics_Direct3D9", feature = "Win32_Media_MediaFoundation"))]
pub type PDXVA2SW_GETVIDEOPROCESSORRENDERTARGETCOUNT = ::core::option::Option<unsafe extern "system" fn(pvideodesc: *const super::MediaFoundation::DXVA2_VideoDesc, pcount: *mut u32) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Media_MediaFoundation\"`*"]
#[cfg(all(feature = "Win32_Graphics_Direct3D9", feature = "Win32_Media_MediaFoundation"))]
pub type PDXVA2SW_GETVIDEOPROCESSORRENDERTARGETS = ::core::option::Option<unsafe extern "system" fn(pvideodesc: *const super::MediaFoundation::DXVA2_VideoDesc, count: u32, pformats: *mut super::super::Graphics::Direct3D9::D3DFORMAT) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Media_MediaFoundation\"`*"]
#[cfg(all(feature = "Win32_Graphics_Direct3D9", feature = "Win32_Media_MediaFoundation"))]
pub type PDXVA2SW_GETVIDEOPROCESSORSUBSTREAMFORMATCOUNT = ::core::option::Option<unsafe extern "system" fn(pvideodesc: *const super::MediaFoundation::DXVA2_VideoDesc, rendertargetformat: super::super::Graphics::Direct3D9::D3DFORMAT, pcount: *mut u32) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Media_MediaFoundation\"`*"]
#[cfg(all(feature = "Win32_Graphics_Direct3D9", feature = "Win32_Media_MediaFoundation"))]
pub type PDXVA2SW_GETVIDEOPROCESSORSUBSTREAMFORMATS = ::core::option::Option<unsafe extern "system" fn(pvideodesc: *const super::MediaFoundation::DXVA2_VideoDesc, rendertargetformat: super::super::Graphics::Direct3D9::D3DFORMAT, count: u32, pformats: *mut super::super::Graphics::Direct3D9::D3DFORMAT) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDXVA2SW_VIDEOPROCESSBEGINFRAME = ::core::option::Option<unsafe extern "system" fn(hdevice: super::super::Foundation::HANDLE) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_Media_MediaFoundation\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Media_MediaFoundation"))]
pub type PDXVA2SW_VIDEOPROCESSBLT = ::core::option::Option<unsafe extern "system" fn(hdevice: super::super::Foundation::HANDLE, pblt: *const DXVA2_VIDEOPROCESSBLT) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDXVA2SW_VIDEOPROCESSENDFRAME = ::core::option::Option<unsafe extern "system" fn(hdevice: super::super::Foundation::HANDLE, phandlecomplete: *mut super::super::Foundation::HANDLE) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Direct3D9\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct3D9"))]
pub type PDXVA2SW_VIDEOPROCESSSETRENDERTARGET = ::core::option::Option<unsafe extern "system" fn(hdevice: super::super::Foundation::HANDLE, prendertarget: super::super::Graphics::Direct3D9::IDirect3DSurface9) -> ::windows_sys::core::HRESULT>;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct PIC_SEQ_SAMPLE {
    pub _bitfield: u32,
}
impl ::core::marker::Copy for PIC_SEQ_SAMPLE {}
impl ::core::clone::Clone for PIC_SEQ_SAMPLE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct PIDListSpanningEvent {
    pub wPIDCount: u16,
    pub pulPIDs: [u32; 1],
}
impl ::core::marker::Copy for PIDListSpanningEvent {}
impl ::core::clone::Clone for PIDListSpanningEvent {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct PID_BITS {
    pub _bitfield: u16,
}
impl ::core::marker::Copy for PID_BITS {}
impl ::core::clone::Clone for PID_BITS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct PID_BITS_MIDL {
    pub Bits: u16,
}
impl ::core::marker::Copy for PID_BITS_MIDL {}
impl ::core::clone::Clone for PID_BITS_MIDL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct PID_MAP {
    pub ulPID: u32,
    pub MediaSampleContent: MEDIA_SAMPLE_CONTENT,
}
impl ::core::marker::Copy for PID_MAP {}
impl ::core::clone::Clone for PID_MAP {
    fn clone(&self) -> Self {
        *self
    }
}
pub const PINNAME_BDA_ANALOG_AUDIO: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3532281866, data2: 39711, data3: 19212, data4: [156, 51, 155, 240, 168, 234, 99, 107] };
pub const PINNAME_BDA_ANALOG_VIDEO: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1544323713, data2: 22119, data3: 18540, data4: [132, 130, 99, 227, 31, 1, 166, 233] };
pub const PINNAME_BDA_FM_RADIO: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3531956205, data2: 45779, data3: 20203, data4: [155, 208, 25, 52, 54, 162, 248, 144] };
pub const PINNAME_BDA_IF_PIN: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 446515778, data2: 62413, data3: 18593, data4: [154, 234, 113, 222, 19, 60, 190, 20] };
pub const PINNAME_BDA_OPENCABLE_PSIP_PIN: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 695972100, data2: 58825, data3: 19150, data4: [177, 35, 149, 195, 203, 178, 77, 79] };
pub const PINNAME_BDA_TRANSPORT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2015455873, data2: 53160, data3: 18750, data4: [151, 17, 54, 166, 28, 8, 189, 157] };
pub const PINNAME_IPSINK_INPUT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1071643248, data2: 44186, data3: 4562, data4: [143, 23, 0, 192, 79, 121, 113, 226] };
pub const PINNAME_MPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3249565043, data2: 7611, data3: 4563, data4: [143, 70, 0, 192, 79, 121, 113, 226] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type PIN_DIRECTION = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PINDIR_INPUT: PIN_DIRECTION = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PINDIR_OUTPUT: PIN_DIRECTION = 1i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_System_Com\"`*"]
#[cfg(feature = "Win32_System_Com")]
pub struct PIN_INFO {
    pub pFilter: IBaseFilter,
    pub dir: PIN_DIRECTION,
    pub achName: [u16; 128],
}
#[cfg(feature = "Win32_System_Com")]
impl ::core::marker::Copy for PIN_INFO {}
#[cfg(feature = "Win32_System_Com")]
impl ::core::clone::Clone for PIN_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
pub const PersistTuneXmlUtility: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3882886832, data2: 47487, data3: 19643, data4: [183, 251, 244, 240, 58, 214, 159, 17] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type PhysicalConnectorType = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PhysConn_Video_Tuner: PhysicalConnectorType = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PhysConn_Video_Composite: PhysicalConnectorType = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PhysConn_Video_SVideo: PhysicalConnectorType = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PhysConn_Video_RGB: PhysicalConnectorType = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PhysConn_Video_YRYBY: PhysicalConnectorType = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PhysConn_Video_SerialDigital: PhysicalConnectorType = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PhysConn_Video_ParallelDigital: PhysicalConnectorType = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PhysConn_Video_SCSI: PhysicalConnectorType = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PhysConn_Video_AUX: PhysicalConnectorType = 9i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PhysConn_Video_1394: PhysicalConnectorType = 10i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PhysConn_Video_USB: PhysicalConnectorType = 11i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PhysConn_Video_VideoDecoder: PhysicalConnectorType = 12i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PhysConn_Video_VideoEncoder: PhysicalConnectorType = 13i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PhysConn_Video_SCART: PhysicalConnectorType = 14i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PhysConn_Video_Black: PhysicalConnectorType = 15i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PhysConn_Audio_Tuner: PhysicalConnectorType = 4096i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PhysConn_Audio_Line: PhysicalConnectorType = 4097i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PhysConn_Audio_Mic: PhysicalConnectorType = 4098i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PhysConn_Audio_AESDigital: PhysicalConnectorType = 4099i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PhysConn_Audio_SPDIFDigital: PhysicalConnectorType = 4100i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PhysConn_Audio_SCSI: PhysicalConnectorType = 4101i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PhysConn_Audio_AUX: PhysicalConnectorType = 4102i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PhysConn_Audio_1394: PhysicalConnectorType = 4103i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PhysConn_Audio_USB: PhysicalConnectorType = 4104i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PhysConn_Audio_AudioDecoder: PhysicalConnectorType = 4105i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type Pilot = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_PILOT_NOT_SET: Pilot = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_PILOT_NOT_DEFINED: Pilot = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_PILOT_OFF: Pilot = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_PILOT_ON: Pilot = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_PILOT_MAX: Pilot = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type Polarisation = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_POLARISATION_NOT_SET: Polarisation = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_POLARISATION_NOT_DEFINED: Polarisation = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_POLARISATION_LINEAR_H: Polarisation = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_POLARISATION_LINEAR_V: Polarisation = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_POLARISATION_CIRCULAR_L: Polarisation = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_POLARISATION_CIRCULAR_R: Polarisation = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_POLARISATION_MAX: Polarisation = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type PositionModeList = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const FrameMode: PositionModeList = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TenthsSecondsMode: PositionModeList = 1i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct ProgramElement {
    pub wProgramNumber: u16,
    pub wProgramMapPID: u16,
}
impl ::core::marker::Copy for ProgramElement {}
impl ::core::clone::Clone for ProgramElement {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type ProtType = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PROT_COPY_FREE: ProtType = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PROT_COPY_ONCE: ProtType = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PROT_COPY_NEVER: ProtType = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PROT_COPY_NEVER_REALLY: ProtType = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PROT_COPY_NO_MORE: ProtType = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PROT_COPY_FREE_CIT: ProtType = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PROT_COPY_BF: ProtType = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PROT_COPY_CN_RECORDING_STOP: ProtType = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PROT_COPY_FREE_SECURE: ProtType = 9i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const PROT_COPY_INVALID: ProtType = 50i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct Quality {
    pub Type: QualityMessageType,
    pub Proportion: i32,
    pub Late: i64,
    pub TimeStamp: i64,
}
impl ::core::marker::Copy for Quality {}
impl ::core::clone::Clone for Quality {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type QualityMessageType = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const Famine: QualityMessageType = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const Flood: QualityMessageType = 1i32;
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct RATING_ATTRIBUTE {
    pub rating_attribute_id: u32,
    pub rating_attribute_value: u32,
}
impl ::core::marker::Copy for RATING_ATTRIBUTE {}
impl ::core::clone::Clone for RATING_ATTRIBUTE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct RATING_INFO {
    pub rating_system_count: u32,
    pub lpratingsystem: *mut RATING_SYSTEM,
}
impl ::core::marker::Copy for RATING_INFO {}
impl ::core::clone::Clone for RATING_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct RATING_SYSTEM {
    pub rating_system_id: ::windows_sys::core::GUID,
    pub _bitfield: u8,
    pub country_code: [u8; 3],
    pub rating_attribute_count: u32,
    pub lpratingattrib: *mut RATING_ATTRIBUTE,
}
impl ::core::marker::Copy for RATING_SYSTEM {}
impl ::core::clone::Clone for RATING_SYSTEM {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type RECORDING_TYPE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const RECORDING_TYPE_CONTENT: RECORDING_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const RECORDING_TYPE_REFERENCE: RECORDING_TYPE = 1i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct REGFILTER {
    pub Clsid: ::windows_sys::core::GUID,
    pub Name: ::windows_sys::core::PWSTR,
}
impl ::core::marker::Copy for REGFILTER {}
impl ::core::clone::Clone for REGFILTER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct REGFILTER2 {
    pub dwVersion: u32,
    pub dwMerit: u32,
    pub Anonymous: REGFILTER2_0,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for REGFILTER2 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for REGFILTER2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub union REGFILTER2_0 {
    pub Anonymous1: REGFILTER2_0_0,
    pub Anonymous2: REGFILTER2_0_1,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for REGFILTER2_0 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for REGFILTER2_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct REGFILTER2_0_0 {
    pub cPins: u32,
    pub rgPins: *const REGFILTERPINS,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for REGFILTER2_0_0 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for REGFILTER2_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct REGFILTER2_0_1 {
    pub cPins2: u32,
    pub rgPins2: *const REGFILTERPINS2,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for REGFILTER2_0_1 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for REGFILTER2_0_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct REGFILTERPINS {
    pub strName: ::windows_sys::core::PWSTR,
    pub bRendered: super::super::Foundation::BOOL,
    pub bOutput: super::super::Foundation::BOOL,
    pub bZero: super::super::Foundation::BOOL,
    pub bMany: super::super::Foundation::BOOL,
    pub clsConnectsToFilter: *const ::windows_sys::core::GUID,
    pub strConnectsToPin: ::windows_sys::core::PCWSTR,
    pub nMediaTypes: u32,
    pub lpMediaType: *const REGPINTYPES,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for REGFILTERPINS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for REGFILTERPINS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct REGFILTERPINS2 {
    pub dwFlags: u32,
    pub cInstances: u32,
    pub nMediaTypes: u32,
    pub lpMediaType: *const REGPINTYPES,
    pub nMediums: u32,
    pub lpMedium: *const REGPINMEDIUM,
    pub clsPinCategory: *const ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for REGFILTERPINS2 {}
impl ::core::clone::Clone for REGFILTERPINS2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct REGPINMEDIUM {
    pub clsMedium: ::windows_sys::core::GUID,
    pub dw1: u32,
    pub dw2: u32,
}
impl ::core::marker::Copy for REGPINMEDIUM {}
impl ::core::clone::Clone for REGPINMEDIUM {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct REGPINTYPES {
    pub clsMajorType: *const ::windows_sys::core::GUID,
    pub clsMinorType: *const ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for REGPINTYPES {}
impl ::core::clone::Clone for REGPINTYPES {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type REG_PINFLAG = u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const REG_PINFLAG_B_ZERO: REG_PINFLAG = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const REG_PINFLAG_B_RENDERER: REG_PINFLAG = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const REG_PINFLAG_B_MANY: REG_PINFLAG = 4u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const REG_PINFLAG_B_OUTPUT: REG_PINFLAG = 8u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const REQUIRED_PARENTAL_CONTROL_TIME_RANGE: u32 = 2u32;
#[repr(C, packed(2))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct RIFFCHUNK {
    pub fcc: u32,
    pub cb: u32,
}
impl ::core::marker::Copy for RIFFCHUNK {}
impl ::core::clone::Clone for RIFFCHUNK {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(2))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct RIFFLIST {
    pub fcc: u32,
    pub cb: u32,
    pub fccListType: u32,
}
impl ::core::marker::Copy for RIFFLIST {}
impl ::core::clone::Clone for RIFFLIST {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type RecordingType = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CONTENT: RecordingType = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const REFERENCE: RecordingType = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type RevokedComponent = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const REVOKED_COPP: RevokedComponent = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const REVOKED_SAC: RevokedComponent = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const REVOKED_APP_STUB: RevokedComponent = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const REVOKED_SECURE_PIPELINE: RevokedComponent = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const REVOKED_MAX_TYPES: RevokedComponent = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type RollOff = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_ROLL_OFF_NOT_SET: RollOff = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_ROLL_OFF_NOT_DEFINED: RollOff = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_ROLL_OFF_20: RollOff = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_ROLL_OFF_25: RollOff = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_ROLL_OFF_35: RollOff = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_ROLL_OFF_MAX: RollOff = 4i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct SAMPLE_LIVE_STREAM_TIME {
    pub qwStreamTime: u64,
    pub qwLiveTime: u64,
}
impl ::core::marker::Copy for SAMPLE_LIVE_STREAM_TIME {}
impl ::core::clone::Clone for SAMPLE_LIVE_STREAM_TIME {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SAMPLE_SEQ_CONTENT_B_FRAME: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SAMPLE_SEQ_CONTENT_I_FRAME: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SAMPLE_SEQ_CONTENT_NONREF_FRAME: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SAMPLE_SEQ_CONTENT_P_FRAME: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SAMPLE_SEQ_CONTENT_REF_FRAME: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SAMPLE_SEQ_CONTENT_STANDALONE_FRAME: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SAMPLE_SEQ_CONTENT_UNKNOWN: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SAMPLE_SEQ_FRAME_START: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SAMPLE_SEQ_GOP_HEADER: u32 = 2u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct SAMPLE_SEQ_OFFSET {
    pub _bitfield: u32,
}
impl ::core::marker::Copy for SAMPLE_SEQ_OFFSET {}
impl ::core::clone::Clone for SAMPLE_SEQ_OFFSET {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SAMPLE_SEQ_PICTURE_HEADER: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SAMPLE_SEQ_SEEK_POINT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SAMPLE_SEQ_SEQUENCE_HEADER: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SAMPLE_SEQ_SEQUENCE_START: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct SBE2_STREAM_DESC {
    pub Version: u32,
    pub StreamId: u32,
    pub Default: u32,
    pub Reserved: u32,
}
impl ::core::marker::Copy for SBE2_STREAM_DESC {}
impl ::core::clone::Clone for SBE2_STREAM_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
pub const SBE2_STREAM_DESC_EVENT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 588489965, data2: 48941, data3: 17743, data4: [173, 138, 217, 91, 167, 249, 31, 238] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SBE2_STREAM_DESC_VERSION: u32 = 1u32;
pub const SBE2_V1_STREAMS_CREATION_EVENT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1036041, data2: 38901, data3: 18092, data4: [151, 105, 122, 131, 179, 83, 132, 251] };
pub const SBE2_V2_STREAMS_CREATION_EVENT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2804232355, data2: 836, data3: 19627, data4: [162, 208, 254, 147, 125, 189, 202, 179] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct SBE_PIN_DATA {
    pub cDataBytes: u64,
    pub cSamplesProcessed: u64,
    pub cDiscontinuities: u64,
    pub cSyncPoints: u64,
    pub cTimestamps: u64,
}
impl ::core::marker::Copy for SBE_PIN_DATA {}
impl ::core::clone::Clone for SBE_PIN_DATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SCTE_EAS_IB_PID: u32 = 8187u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SCTE_EAS_OOB_PID: u32 = 8188u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SCTE_EAS_TID: u32 = 216u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct SECTION {
    pub TableId: u8,
    pub Header: SECTION_0,
    pub SectionData: [u8; 1],
}
impl ::core::marker::Copy for SECTION {}
impl ::core::clone::Clone for SECTION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub union SECTION_0 {
    pub S: MPEG_HEADER_BITS_MIDL,
    pub W: u16,
}
impl ::core::marker::Copy for SECTION_0 {}
impl ::core::clone::Clone for SECTION_0 {
    fn clone(&self) -> Self {
        *self
    }
}
pub const SID_DRMSecureServiceChannel: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229764, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
pub const SID_MSVidCtl_CurrentAudioEndpoint: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3483011316, data2: 43983, data3: 20184, data4: [155, 116, 125, 179, 52, 69, 69, 158] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type SNDDEV_ERR = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SNDDEV_ERROR_Open: SNDDEV_ERR = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SNDDEV_ERROR_Close: SNDDEV_ERR = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SNDDEV_ERROR_GetCaps: SNDDEV_ERR = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SNDDEV_ERROR_PrepareHeader: SNDDEV_ERR = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SNDDEV_ERROR_UnprepareHeader: SNDDEV_ERR = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SNDDEV_ERROR_Reset: SNDDEV_ERR = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SNDDEV_ERROR_Restart: SNDDEV_ERR = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SNDDEV_ERROR_GetPosition: SNDDEV_ERR = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SNDDEV_ERROR_Write: SNDDEV_ERR = 9i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SNDDEV_ERROR_Pause: SNDDEV_ERR = 10i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SNDDEV_ERROR_Stop: SNDDEV_ERR = 11i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SNDDEV_ERROR_Start: SNDDEV_ERR = 12i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SNDDEV_ERROR_AddBuffer: SNDDEV_ERR = 13i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SNDDEV_ERROR_Query: SNDDEV_ERR = 14i32;
pub const SPECIFYPAGES_STATISTICS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1279490962, data2: 28318, data3: 4561, data4: [167, 4, 0, 96, 151, 196, 228, 118] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type SSUPDATE_TYPE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SSUPDATE_ASYNC: SSUPDATE_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SSUPDATE_CONTINUOUS: SSUPDATE_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STDINDEXSIZE: u32 = 16384u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct STREAMBUFFER_ATTRIBUTE {
    pub pszName: ::windows_sys::core::PWSTR,
    pub StreamBufferAttributeType: STREAMBUFFER_ATTR_DATATYPE,
    pub pbAttribute: *mut u8,
    pub cbLength: u16,
}
impl ::core::marker::Copy for STREAMBUFFER_ATTRIBUTE {}
impl ::core::clone::Clone for STREAMBUFFER_ATTRIBUTE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type STREAMBUFFER_ATTR_DATATYPE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STREAMBUFFER_TYPE_DWORD: STREAMBUFFER_ATTR_DATATYPE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STREAMBUFFER_TYPE_STRING: STREAMBUFFER_ATTR_DATATYPE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STREAMBUFFER_TYPE_BINARY: STREAMBUFFER_ATTR_DATATYPE = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STREAMBUFFER_TYPE_BOOL: STREAMBUFFER_ATTR_DATATYPE = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STREAMBUFFER_TYPE_QWORD: STREAMBUFFER_ATTR_DATATYPE = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STREAMBUFFER_TYPE_WORD: STREAMBUFFER_ATTR_DATATYPE = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STREAMBUFFER_TYPE_GUID: STREAMBUFFER_ATTR_DATATYPE = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STREAMBUFFER_EC_BASE: u32 = 806u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STREAMBUFFER_EC_CONTENT_BECOMING_STALE: i32 = 809i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STREAMBUFFER_EC_PRIMARY_AUDIO: i32 = 814i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STREAMBUFFER_EC_RATE_CHANGED: i32 = 813i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STREAMBUFFER_EC_RATE_CHANGING_FOR_SETPOSITIONS: i32 = 815i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STREAMBUFFER_EC_READ_FAILURE: i32 = 812i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STREAMBUFFER_EC_SETPOSITIONS_EVENTS_DONE: i32 = 816i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STREAMBUFFER_EC_STALE_DATA_READ: i32 = 807i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STREAMBUFFER_EC_STALE_FILE_DELETED: i32 = 808i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STREAMBUFFER_EC_TIMEHOLE: i32 = 806i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STREAMBUFFER_EC_WRITE_FAILURE: i32 = 810i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STREAMBUFFER_EC_WRITE_FAILURE_CLEAR: i32 = 811i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type STREAMIF_CONSTANTS = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MAX_NUMBER_OF_STREAMS: STREAMIF_CONSTANTS = 16i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct STREAM_ID_MAP {
    pub stream_id: u32,
    pub dwMediaSampleContent: u32,
    pub ulSubstreamFilterValue: u32,
    pub iDataOffset: i32,
}
impl ::core::marker::Copy for STREAM_ID_MAP {}
impl ::core::clone::Clone for STREAM_ID_MAP {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type STREAM_STATE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STREAMSTATE_STOP: STREAM_STATE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STREAMSTATE_RUN: STREAM_STATE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type STREAM_TYPE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STREAMTYPE_READ: STREAM_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STREAMTYPE_WRITE: STREAM_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const STREAMTYPE_TRANSFORM: STREAM_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SUBSTREAM_FILTER_VAL_NONE: u32 = 268435456u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type ScanModulationTypes = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_16QAM: ScanModulationTypes = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_32QAM: ScanModulationTypes = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_64QAM: ScanModulationTypes = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_80QAM: ScanModulationTypes = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_96QAM: ScanModulationTypes = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_112QAM: ScanModulationTypes = 32i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_128QAM: ScanModulationTypes = 64i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_160QAM: ScanModulationTypes = 128i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_192QAM: ScanModulationTypes = 256i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_224QAM: ScanModulationTypes = 512i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_256QAM: ScanModulationTypes = 1024i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_320QAM: ScanModulationTypes = 2048i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_384QAM: ScanModulationTypes = 4096i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_448QAM: ScanModulationTypes = 8192i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_512QAM: ScanModulationTypes = 16384i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_640QAM: ScanModulationTypes = 32768i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_768QAM: ScanModulationTypes = 65536i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_896QAM: ScanModulationTypes = 131072i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_1024QAM: ScanModulationTypes = 262144i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_QPSK: ScanModulationTypes = 524288i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_BPSK: ScanModulationTypes = 1048576i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_OQPSK: ScanModulationTypes = 2097152i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_8VSB: ScanModulationTypes = 4194304i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_16VSB: ScanModulationTypes = 8388608i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_AM_RADIO: ScanModulationTypes = 16777216i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_FM_RADIO: ScanModulationTypes = 33554432i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_8PSK: ScanModulationTypes = 67108864i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_RF: ScanModulationTypes = 134217728i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ScanModulationTypesMask_MCE_DigitalCable: ScanModulationTypes = 11i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ScanModulationTypesMask_MCE_TerrestrialATSC: ScanModulationTypes = 23i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ScanModulationTypesMask_MCE_AnalogTv: ScanModulationTypes = 28i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ScanModulationTypesMask_MCE_All_TV: ScanModulationTypes = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ScanModulationTypesMask_DVBC: ScanModulationTypes = 75i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_16APSK: ScanModulationTypes = 268435456i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SCAN_MOD_32APSK: ScanModulationTypes = 536870912i32;
pub const SectionList: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1943690500, data2: 17223, data3: 17875, data4: [169, 220, 250, 233, 221, 190, 85, 141] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type SegDispidList = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidName: SegDispidList = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidStatus: SegDispidList = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidDevImageSourceWidth: SegDispidList = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidDevImageSourceHeight: SegDispidList = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidDevCountryCode: SegDispidList = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidDevOverScan: SegDispidList = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidSegment: SegDispidList = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidDevVolume: SegDispidList = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidDevBalance: SegDispidList = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidDevPower: SegDispidList = 9i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidTuneChan: SegDispidList = 10i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidDevVideoSubchannel: SegDispidList = 11i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidDevAudioSubchannel: SegDispidList = 12i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidChannelAvailable: SegDispidList = 13i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidDevVideoFrequency: SegDispidList = 14i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidDevAudioFrequency: SegDispidList = 15i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidCount: SegDispidList = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidDevFileName: SegDispidList = 17i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidVisible: SegDispidList = 18i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidOwner: SegDispidList = 19i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidMessageDrain: SegDispidList = 20i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidViewable: SegDispidList = 21i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidDevView: SegDispidList = 22i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidKSCat: SegDispidList = 23i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidCLSID: SegDispidList = 24i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_KSCat: SegDispidList = 25i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_CLSID: SegDispidList = 26i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidTune: SegDispidList = 27i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidTS: SegDispidList = 28i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidDevSAP: SegDispidList = 29i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidClip: SegDispidList = 30i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidRequestedClipRect: SegDispidList = 31i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidClippedSourceRect: SegDispidList = 32i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidAvailableSourceRect: SegDispidList = 33i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidMediaPosition: SegDispidList = 34i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidDevRun: SegDispidList = 35i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidDevPause: SegDispidList = 36i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidDevStop: SegDispidList = 37i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidCCEnable: SegDispidList = 38i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidDevStep: SegDispidList = 39i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidDevCanStep: SegDispidList = 40i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidSourceSize: SegDispidList = 41i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_playtitle: SegDispidList = 42i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_playchapterintitle: SegDispidList = 43i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_playchapter: SegDispidList = 44i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_playchaptersautostop: SegDispidList = 45i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_playattime: SegDispidList = 46i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_playattimeintitle: SegDispidList = 47i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_playperiodintitleautostop: SegDispidList = 48i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_replaychapter: SegDispidList = 49i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_playprevchapter: SegDispidList = 50i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_playnextchapter: SegDispidList = 51i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_playforwards: SegDispidList = 52i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_playbackwards: SegDispidList = 53i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_stilloff: SegDispidList = 54i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_audiolanguage: SegDispidList = 55i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_showmenu: SegDispidList = 56i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_resume: SegDispidList = 57i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_returnfromsubmenu: SegDispidList = 58i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_buttonsavailable: SegDispidList = 59i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_currentbutton: SegDispidList = 60i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_SelectAndActivateButton: SegDispidList = 61i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_ActivateButton: SegDispidList = 62i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_SelectRightButton: SegDispidList = 63i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_SelectLeftButton: SegDispidList = 64i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_SelectLowerButton: SegDispidList = 65i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_SelectUpperButton: SegDispidList = 66i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_ActivateAtPosition: SegDispidList = 67i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_SelectAtPosition: SegDispidList = 68i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_ButtonAtPosition: SegDispidList = 69i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_NumberOfChapters: SegDispidList = 70i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_TotalTitleTime: SegDispidList = 71i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_TitlesAvailable: SegDispidList = 72i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_VolumesAvailable: SegDispidList = 73i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_CurrentVolume: SegDispidList = 74i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_CurrentDiscSide: SegDispidList = 75i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_CurrentDomain: SegDispidList = 76i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_CurrentChapter: SegDispidList = 77i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_CurrentTitle: SegDispidList = 78i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_CurrentTime: SegDispidList = 79i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_FramesPerSecond: SegDispidList = 80i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_DVDTimeCode2bstr: SegDispidList = 81i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_DVDDirectory: SegDispidList = 82i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_IsSubpictureStreamEnabled: SegDispidList = 83i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_IsAudioStreamEnabled: SegDispidList = 84i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_CurrentSubpictureStream: SegDispidList = 85i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_SubpictureLanguage: SegDispidList = 86i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_CurrentAudioStream: SegDispidList = 87i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_AudioStreamsAvailable: SegDispidList = 88i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_AnglesAvailable: SegDispidList = 89i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_CurrentAngle: SegDispidList = 90i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_CCActive: SegDispidList = 91i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_CurrentCCService: SegDispidList = 92i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_SubpictureStreamsAvailable: SegDispidList = 93i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_SubpictureOn: SegDispidList = 94i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_DVDUniqueID: SegDispidList = 95i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_EnableResetOnStop: SegDispidList = 96i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_AcceptParentalLevelChange: SegDispidList = 97i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_NotifyParentalLevelChange: SegDispidList = 98i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_SelectParentalCountry: SegDispidList = 99i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_SelectParentalLevel: SegDispidList = 100i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_TitleParentalLevels: SegDispidList = 101i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_PlayerParentalCountry: SegDispidList = 102i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_PlayerParentalLevel: SegDispidList = 103i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_Eject: SegDispidList = 104i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_UOPValid: SegDispidList = 105i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_SPRM: SegDispidList = 106i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_GPRM: SegDispidList = 107i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_DVDTextStringType: SegDispidList = 108i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_DVDTextString: SegDispidList = 109i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_DVDTextNumberOfStrings: SegDispidList = 110i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_DVDTextNumberOfLanguages: SegDispidList = 111i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_DVDTextLanguageLCID: SegDispidList = 112i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_RegionChange: SegDispidList = 113i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_DVDAdm: SegDispidList = 114i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_DeleteBookmark: SegDispidList = 115i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_RestoreBookmark: SegDispidList = 116i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_SaveBookmark: SegDispidList = 117i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_SelectDefaultAudioLanguage: SegDispidList = 118i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_SelectDefaultSubpictureLanguage: SegDispidList = 119i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_PreferredSubpictureStream: SegDispidList = 120i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_DefaultMenuLanguage: SegDispidList = 121i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_DefaultSubpictureLanguage: SegDispidList = 122i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_DefaultAudioLanguage: SegDispidList = 123i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_DefaultSubpictureLanguageExt: SegDispidList = 124i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_DefaultAudioLanguageExt: SegDispidList = 125i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_LanguageFromLCID: SegDispidList = 126i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_KaraokeAudioPresentationMode: SegDispidList = 127i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_KaraokeChannelContent: SegDispidList = 128i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_KaraokeChannelAssignment: SegDispidList = 129i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_RestorePreferredSettings: SegDispidList = 130i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_ButtonRect: SegDispidList = 131i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_DVDScreenInMouseCoordinates: SegDispidList = 132i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_CustomCompositorClass: SegDispidList = 133i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidCustomCompositorClass: SegDispidList = 134i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_CustomCompositor: SegDispidList = 135i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidMixerBitmap: SegDispidList = 136i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_MixerBitmap: SegDispidList = 137i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidMixerBitmapOpacity: SegDispidList = 138i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidMixerBitmapRect: SegDispidList = 139i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidSetupMixerBitmap: SegDispidList = 140i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidUsingOverlay: SegDispidList = 141i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidDisplayChange: SegDispidList = 142i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidRePaint: SegDispidList = 143i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_IsEqualDevice: SegDispidList = 144i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidrate: SegDispidList = 145i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidposition: SegDispidList = 146i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidpositionmode: SegDispidList = 147i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidlength: SegDispidList = 148i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidChangePassword: SegDispidList = 149i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidSaveParentalLevel: SegDispidList = 150i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidSaveParentalCountry: SegDispidList = 151i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidConfirmPassword: SegDispidList = 152i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidGetParentalLevel: SegDispidList = 153i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidGetParentalCountry: SegDispidList = 154i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidDefaultAudioLCID: SegDispidList = 155i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidDefaultSubpictureLCID: SegDispidList = 156i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidDefaultMenuLCID: SegDispidList = 157i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidBookmarkOnStop: SegDispidList = 158i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidMaxVidRect: SegDispidList = 159i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidMinVidRect: SegDispidList = 160i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidCapture: SegDispidList = 161i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_DecimateInput: SegDispidList = 162i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidAlloctor: SegDispidList = 163i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_Allocator: SegDispidList = 164i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidAllocPresentID: SegDispidList = 165i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidSetAllocator: SegDispidList = 166i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_SetAllocator: SegDispidList = 167i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidStreamBufferSinkName: SegDispidList = 168i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidStreamBufferSourceName: SegDispidList = 169i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidStreamBufferContentRecording: SegDispidList = 170i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidStreamBufferReferenceRecording: SegDispidList = 171i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidstarttime: SegDispidList = 172i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidstoptime: SegDispidList = 173i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidrecordingstopped: SegDispidList = 174i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidrecordingstarted: SegDispidList = 175i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidNameSetLock: SegDispidList = 176i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidrecordingtype: SegDispidList = 177i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidstart: SegDispidList = 178i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidRecordingAttribute: SegDispidList = 179i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_RecordingAttribute: SegDispidList = 180i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidSBEConfigure: SegDispidList = 181i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_CurrentRatings: SegDispidList = 182i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_MaxRatingsLevel: SegDispidList = 183i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_audioencoderint: SegDispidList = 184i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_videoencoderint: SegDispidList = 185i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidService: SegDispidList = 186i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_BlockUnrated: SegDispidList = 187i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_UnratedDelay: SegDispidList = 188i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_SuppressEffects: SegDispidList = 189i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidsbesource: SegDispidList = 190i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidSetSinkFilter: SegDispidList = 191i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_SinkStreams: SegDispidList = 192i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidTVFormats: SegDispidList = 193i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidModes: SegDispidList = 194i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidAuxInputs: SegDispidList = 195i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidTeleTextFilter: SegDispidList = 196i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_channelchangeint: SegDispidList = 197i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidUnlockProfile: SegDispidList = 198i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_AddFilter: SegDispidList = 199i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidSetMinSeek: SegDispidList = 200i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidRateEx: SegDispidList = 201i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidaudiocounter: SegDispidList = 202i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidvideocounter: SegDispidList = 203i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidcccounter: SegDispidList = 204i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidwstcounter: SegDispidList = 205i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_audiocounter: SegDispidList = 206i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_videocounter: SegDispidList = 207i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_cccounter: SegDispidList = 208i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_wstcounter: SegDispidList = 209i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidaudioanalysis: SegDispidList = 210i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidvideoanalysis: SegDispidList = 211i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispiddataanalysis: SegDispidList = 212i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidaudio_analysis: SegDispidList = 213i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidvideo_analysis: SegDispidList = 214i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispiddata_analysis: SegDispidList = 215i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_resetFilterList: SegDispidList = 216i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidDevicePath: SegDispidList = 217i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_SourceFilter: SegDispidList = 218i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid__SourceFilter: SegDispidList = 219i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidUserEvent: SegDispidList = 220i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispid_Bookmark: SegDispidList = 221i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const LastReservedDeviceDispid: SegDispidList = 16383i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type SegEventidList = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidStateChange: SegEventidList = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidOnTuneChanged: SegEventidList = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidEndOfMedia: SegEventidList = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidDVDNotify: SegEventidList = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidPlayForwards: SegEventidList = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidPlayBackwards: SegEventidList = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidShowMenu: SegEventidList = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidResume: SegEventidList = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidSelectOrActivateButton: SegEventidList = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidStillOff: SegEventidList = 9i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidPauseOn: SegEventidList = 10i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidChangeCurrentAudioStream: SegEventidList = 11i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidChangeCurrentSubpictureStream: SegEventidList = 12i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidChangeCurrentAngle: SegEventidList = 13i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidPlayAtTimeInTitle: SegEventidList = 14i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidPlayAtTime: SegEventidList = 15i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidPlayChapterInTitle: SegEventidList = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidPlayChapter: SegEventidList = 17i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidReplayChapter: SegEventidList = 18i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidPlayNextChapter: SegEventidList = 19i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidStop: SegEventidList = 20i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidReturnFromSubmenu: SegEventidList = 21i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidPlayTitle: SegEventidList = 22i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidPlayPrevChapter: SegEventidList = 23i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidChangeKaraokePresMode: SegEventidList = 24i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidChangeVideoPresMode: SegEventidList = 25i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidOverlayUnavailable: SegEventidList = 26i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidSinkCertificateFailure: SegEventidList = 27i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidSinkCertificateSuccess: SegEventidList = 28i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidSourceCertificateFailure: SegEventidList = 29i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidSourceCertificateSuccess: SegEventidList = 30i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidRatingsBlocked: SegEventidList = 31i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidRatingsUnlocked: SegEventidList = 32i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidRatingsChanged: SegEventidList = 33i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidWriteFailure: SegEventidList = 34i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidTimeHole: SegEventidList = 35i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidStaleDataRead: SegEventidList = 36i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidContentBecomingStale: SegEventidList = 37i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidStaleFileDeleted: SegEventidList = 38i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidEncryptionOn: SegEventidList = 39i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidEncryptionOff: SegEventidList = 40i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidRateChange: SegEventidList = 41i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidLicenseChange: SegEventidList = 42i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidCOPPBlocked: SegEventidList = 43i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidCOPPUnblocked: SegEventidList = 44i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidlicenseerrorcode: SegEventidList = 45i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidBroadcastEvent: SegEventidList = 46i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidBroadcastEventEx: SegEventidList = 47i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidContentPrimarilyAudio: SegEventidList = 48i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidAVDecAudioDualMonoEvent: SegEventidList = 49i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidAVAudioSampleRateEvent: SegEventidList = 50i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidAVAudioChannelConfigEvent: SegEventidList = 51i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidAVAudioChannelCountEvent: SegEventidList = 52i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidAVDecCommonMeanBitRateEvent: SegEventidList = 53i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidAVDDSurroundModeEvent: SegEventidList = 54i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidAVDecCommonInputFormatEvent: SegEventidList = 55i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const dispidAVDecCommonOutputFormatEvent: SegEventidList = 56i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const eventidWriteFailureClear: SegEventidList = 57i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const LastReservedDeviceEvent: SegEventidList = 16383i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type SignalAndServiceStatusSpanningEvent_State = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SignalAndServiceStatusSpanningEvent_None: SignalAndServiceStatusSpanningEvent_State = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SignalAndServiceStatusSpanningEvent_Clear: SignalAndServiceStatusSpanningEvent_State = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SignalAndServiceStatusSpanningEvent_NoTVSignal: SignalAndServiceStatusSpanningEvent_State = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SignalAndServiceStatusSpanningEvent_ServiceOffAir: SignalAndServiceStatusSpanningEvent_State = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SignalAndServiceStatusSpanningEvent_WeakTVSignal: SignalAndServiceStatusSpanningEvent_State = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SignalAndServiceStatusSpanningEvent_NoSubscription: SignalAndServiceStatusSpanningEvent_State = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SignalAndServiceStatusSpanningEvent_AllAVScrambled: SignalAndServiceStatusSpanningEvent_State = 5i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct SmartCardApplication {
    pub ApplicationType: ApplicationTypeType,
    pub ApplicationVersion: u16,
    pub pbstrApplicationName: super::super::Foundation::BSTR,
    pub pbstrApplicationURL: super::super::Foundation::BSTR,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for SmartCardApplication {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for SmartCardApplication {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type SmartCardAssociationType = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const NotAssociated: SmartCardAssociationType = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const Associated: SmartCardAssociationType = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AssociationUnknown: SmartCardAssociationType = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type SmartCardStatusType = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CardInserted: SmartCardStatusType = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CardRemoved: SmartCardStatusType = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CardError: SmartCardStatusType = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CardDataChanged: SmartCardStatusType = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const CardFirmwareUpgrade: SmartCardStatusType = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type SourceSizeList = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const sslFullSize: SourceSizeList = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const sslClipByOverScan: SourceSizeList = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const sslClipByClipRect: SourceSizeList = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct SpanningEventDescriptor {
    pub wDataLen: u16,
    pub wProgNumber: u16,
    pub wSID: u16,
    pub bDescriptor: [u8; 1],
}
impl ::core::marker::Copy for SpanningEventDescriptor {}
impl ::core::clone::Clone for SpanningEventDescriptor {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct SpanningEventEmmMessage {
    pub bCAbroadcasterGroupId: u8,
    pub bMessageControl: u8,
    pub wServiceId: u16,
    pub wTableIdExtension: u16,
    pub bDeletionStatus: u8,
    pub bDisplayingDuration1: u8,
    pub bDisplayingDuration2: u8,
    pub bDisplayingDuration3: u8,
    pub bDisplayingCycle: u8,
    pub bFormatVersion: u8,
    pub bDisplayPosition: u8,
    pub wMessageLength: u16,
    pub szMessageArea: [u16; 1],
}
impl ::core::marker::Copy for SpanningEventEmmMessage {}
impl ::core::clone::Clone for SpanningEventEmmMessage {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type SpectralInversion = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SPECTRAL_INVERSION_NOT_SET: SpectralInversion = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SPECTRAL_INVERSION_NOT_DEFINED: SpectralInversion = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SPECTRAL_INVERSION_AUTOMATIC: SpectralInversion = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SPECTRAL_INVERSION_NORMAL: SpectralInversion = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SPECTRAL_INVERSION_INVERTED: SpectralInversion = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_SPECTRAL_INVERSION_MAX: SpectralInversion = 4i32;
pub const SystemTuningSpaces: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3492457552, data2: 638, data3: 4563, data4: [157, 142, 0, 192, 79, 114, 217, 128] };
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct TID_EXTENSION {
    pub wTidExt: u16,
    pub wCount: u16,
}
impl ::core::marker::Copy for TID_EXTENSION {}
impl ::core::clone::Clone for TID_EXTENSION {
    fn clone(&self) -> Self {
        *self
    }
}
pub const TIFLoad: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 350979912, data2: 5971, data3: 17299, data4: [149, 174, 79, 126, 122, 135, 170, 214] };
#[repr(C, packed(2))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct TIMECODEDATA {
    pub time: super::TIMECODE,
    pub dwSMPTEflags: u32,
    pub dwUser: u32,
}
impl ::core::marker::Copy for TIMECODEDATA {}
impl ::core::clone::Clone for TIMECODEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TIMECODE_RATE_30DROP: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TIMECODE_SMPTE_BINARY_GROUP: u32 = 7u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TIMECODE_SMPTE_COLOR_FRAME: u32 = 8u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct TRANSPORT_PROPERTIES {
    pub PID: u32,
    pub PCR: i64,
    pub Fields: TRANSPORT_PROPERTIES_0,
}
impl ::core::marker::Copy for TRANSPORT_PROPERTIES {}
impl ::core::clone::Clone for TRANSPORT_PROPERTIES {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub union TRANSPORT_PROPERTIES_0 {
    pub Others: TRANSPORT_PROPERTIES_0_0,
    pub Value: i64,
}
impl ::core::marker::Copy for TRANSPORT_PROPERTIES_0 {}
impl ::core::clone::Clone for TRANSPORT_PROPERTIES_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct TRANSPORT_PROPERTIES_0_0 {
    pub _bitfield: i64,
}
impl ::core::marker::Copy for TRANSPORT_PROPERTIES_0_0 {}
impl ::core::clone::Clone for TRANSPORT_PROPERTIES_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(feature = "Win32_Graphics_Gdi")]
pub struct TRUECOLORINFO {
    pub dwBitMasks: [u32; 3],
    pub bmiColors: [super::super::Graphics::Gdi::RGBQUAD; 256],
}
#[cfg(feature = "Win32_Graphics_Gdi")]
impl ::core::marker::Copy for TRUECOLORINFO {}
#[cfg(feature = "Win32_Graphics_Gdi")]
impl ::core::clone::Clone for TRUECOLORINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type TVAudioMode = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMTVAUDIO_MODE_MONO: TVAudioMode = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMTVAUDIO_MODE_STEREO: TVAudioMode = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMTVAUDIO_MODE_LANG_A: TVAudioMode = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMTVAUDIO_MODE_LANG_B: TVAudioMode = 32i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMTVAUDIO_MODE_LANG_C: TVAudioMode = 64i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMTVAUDIO_PRESET_STEREO: TVAudioMode = 512i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMTVAUDIO_PRESET_LANG_A: TVAudioMode = 4096i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMTVAUDIO_PRESET_LANG_B: TVAudioMode = 8192i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMTVAUDIO_PRESET_LANG_C: TVAudioMode = 16384i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type TransmissionMode = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_XMIT_MODE_NOT_SET: TransmissionMode = -1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_XMIT_MODE_NOT_DEFINED: TransmissionMode = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_XMIT_MODE_2K: TransmissionMode = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_XMIT_MODE_8K: TransmissionMode = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_XMIT_MODE_4K: TransmissionMode = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_XMIT_MODE_2K_INTERLEAVED: TransmissionMode = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_XMIT_MODE_4K_INTERLEAVED: TransmissionMode = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_XMIT_MODE_1K: TransmissionMode = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_XMIT_MODE_16K: TransmissionMode = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_XMIT_MODE_32K: TransmissionMode = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const BDA_XMIT_MODE_MAX: TransmissionMode = 9i32;
pub const TuneRequest: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3027111224, data2: 43829, data3: 18950, data4: [161, 55, 112, 87, 107, 1, 179, 159] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type TunerInputType = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TunerInputCable: TunerInputType = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const TunerInputAntenna: TunerInputType = 1i32;
pub const TunerMarshaler: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1681413899, data2: 3080, data3: 18981, data4: [149, 4, 128, 18, 187, 77, 80, 207] };
pub const TuningSpace: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1610466790, data2: 47162, data3: 19285, data4: [182, 232, 198, 158, 118, 95, 233, 219] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct UDCR_TAG {
    pub bVersion: u8,
    pub KID: [u8; 25],
    pub ullBaseCounter: u64,
    pub ullBaseCounterRange: u64,
    pub fScrambled: super::super::Foundation::BOOL,
    pub bStreamMark: u8,
    pub dwReserved1: u32,
    pub dwReserved2: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for UDCR_TAG {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for UDCR_TAG {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type UICloseReasonType = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const NotReady: UICloseReasonType = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const UserClosed: UICloseReasonType = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const SystemClosed: UICloseReasonType = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DeviceClosed: UICloseReasonType = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ErrorClosed: UICloseReasonType = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VALID_UOP_FLAG = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const UOP_FLAG_Play_Title_Or_AtTime: VALID_UOP_FLAG = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const UOP_FLAG_Play_Chapter: VALID_UOP_FLAG = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const UOP_FLAG_Play_Title: VALID_UOP_FLAG = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const UOP_FLAG_Stop: VALID_UOP_FLAG = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const UOP_FLAG_ReturnFromSubMenu: VALID_UOP_FLAG = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const UOP_FLAG_Play_Chapter_Or_AtTime: VALID_UOP_FLAG = 32i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const UOP_FLAG_PlayPrev_Or_Replay_Chapter: VALID_UOP_FLAG = 64i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const UOP_FLAG_PlayNext_Chapter: VALID_UOP_FLAG = 128i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const UOP_FLAG_Play_Forwards: VALID_UOP_FLAG = 256i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const UOP_FLAG_Play_Backwards: VALID_UOP_FLAG = 512i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const UOP_FLAG_ShowMenu_Title: VALID_UOP_FLAG = 1024i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const UOP_FLAG_ShowMenu_Root: VALID_UOP_FLAG = 2048i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const UOP_FLAG_ShowMenu_SubPic: VALID_UOP_FLAG = 4096i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const UOP_FLAG_ShowMenu_Audio: VALID_UOP_FLAG = 8192i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const UOP_FLAG_ShowMenu_Angle: VALID_UOP_FLAG = 16384i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const UOP_FLAG_ShowMenu_Chapter: VALID_UOP_FLAG = 32768i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const UOP_FLAG_Resume: VALID_UOP_FLAG = 65536i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const UOP_FLAG_Select_Or_Activate_Button: VALID_UOP_FLAG = 131072i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const UOP_FLAG_Still_Off: VALID_UOP_FLAG = 262144i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const UOP_FLAG_Pause_On: VALID_UOP_FLAG = 524288i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const UOP_FLAG_Select_Audio_Stream: VALID_UOP_FLAG = 1048576i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const UOP_FLAG_Select_SubPic_Stream: VALID_UOP_FLAG = 2097152i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const UOP_FLAG_Select_Angle: VALID_UOP_FLAG = 4194304i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const UOP_FLAG_Select_Karaoke_Audio_Presentation_Mode: VALID_UOP_FLAG = 8388608i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const UOP_FLAG_Select_Video_Mode_Preference: VALID_UOP_FLAG = 16777216i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VA_COLOR_PRIMARIES = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_PRIMARIES_ITU_R_BT_709: VA_COLOR_PRIMARIES = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_PRIMARIES_UNSPECIFIED: VA_COLOR_PRIMARIES = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_PRIMARIES_ITU_R_BT_470_SYSTEM_M: VA_COLOR_PRIMARIES = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_PRIMARIES_ITU_R_BT_470_SYSTEM_B_G: VA_COLOR_PRIMARIES = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_PRIMARIES_SMPTE_170M: VA_COLOR_PRIMARIES = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_PRIMARIES_SMPTE_240M: VA_COLOR_PRIMARIES = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_PRIMARIES_H264_GENERIC_FILM: VA_COLOR_PRIMARIES = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VA_MATRIX_COEFFICIENTS = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_MATRIX_COEFF_H264_RGB: VA_MATRIX_COEFFICIENTS = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_MATRIX_COEFF_ITU_R_BT_709: VA_MATRIX_COEFFICIENTS = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_MATRIX_COEFF_UNSPECIFIED: VA_MATRIX_COEFFICIENTS = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_MATRIX_COEFF_FCC: VA_MATRIX_COEFFICIENTS = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_MATRIX_COEFF_ITU_R_BT_470_SYSTEM_B_G: VA_MATRIX_COEFFICIENTS = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_MATRIX_COEFF_SMPTE_170M: VA_MATRIX_COEFFICIENTS = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_MATRIX_COEFF_SMPTE_240M: VA_MATRIX_COEFFICIENTS = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_MATRIX_COEFF_H264_YCgCo: VA_MATRIX_COEFFICIENTS = 8i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct VA_OPTIONAL_VIDEO_PROPERTIES {
    pub dwPictureHeight: u16,
    pub dwPictureWidth: u16,
    pub dwAspectRatioX: u16,
    pub dwAspectRatioY: u16,
    pub VAVideoFormat: VA_VIDEO_FORMAT,
    pub VAColorPrimaries: VA_COLOR_PRIMARIES,
    pub VATransferCharacteristics: VA_TRANSFER_CHARACTERISTICS,
    pub VAMatrixCoefficients: VA_MATRIX_COEFFICIENTS,
}
impl ::core::marker::Copy for VA_OPTIONAL_VIDEO_PROPERTIES {}
impl ::core::clone::Clone for VA_OPTIONAL_VIDEO_PROPERTIES {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VA_TRANSFER_CHARACTERISTICS = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_TRANSFER_CHARACTERISTICS_ITU_R_BT_709: VA_TRANSFER_CHARACTERISTICS = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_TRANSFER_CHARACTERISTICS_UNSPECIFIED: VA_TRANSFER_CHARACTERISTICS = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_TRANSFER_CHARACTERISTICS_ITU_R_BT_470_SYSTEM_M: VA_TRANSFER_CHARACTERISTICS = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_TRANSFER_CHARACTERISTICS_ITU_R_BT_470_SYSTEM_B_G: VA_TRANSFER_CHARACTERISTICS = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_TRANSFER_CHARACTERISTICS_SMPTE_170M: VA_TRANSFER_CHARACTERISTICS = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_TRANSFER_CHARACTERISTICS_SMPTE_240M: VA_TRANSFER_CHARACTERISTICS = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_TRANSFER_CHARACTERISTICS_LINEAR: VA_TRANSFER_CHARACTERISTICS = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_TRANSFER_CHARACTERISTICS_H264_LOG_100_TO_1: VA_TRANSFER_CHARACTERISTICS = 9i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_TRANSFER_CHARACTERISTICS_H264_LOG_316_TO_1: VA_TRANSFER_CHARACTERISTICS = 10i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VA_VIDEO_FORMAT = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_VIDEO_COMPONENT: VA_VIDEO_FORMAT = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_VIDEO_PAL: VA_VIDEO_FORMAT = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_VIDEO_NTSC: VA_VIDEO_FORMAT = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_VIDEO_SECAM: VA_VIDEO_FORMAT = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_VIDEO_MAC: VA_VIDEO_FORMAT = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VA_VIDEO_UNSPECIFIED: VA_VIDEO_FORMAT = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_ADVISE_ALREADY_SET: ::windows_sys::core::HRESULT = -2147220938i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_ALREADY_CANCELLED: ::windows_sys::core::HRESULT = -2147220940i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_ALREADY_COMMITTED: ::windows_sys::core::HRESULT = -2147220977i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_ALREADY_CONNECTED: ::windows_sys::core::HRESULT = -2147220988i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_BADALIGN: ::windows_sys::core::HRESULT = -2147220978i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_BAD_KEY: ::windows_sys::core::HRESULT = -2147220494i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_BAD_VIDEOCD: ::windows_sys::core::HRESULT = -2147220887i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_BUFFERS_OUTSTANDING: ::windows_sys::core::HRESULT = -2147220976i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_BUFFER_NOTSET: ::windows_sys::core::HRESULT = -2147220980i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_BUFFER_OVERFLOW: ::windows_sys::core::HRESULT = -2147220979i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_BUFFER_UNDERFLOW: ::windows_sys::core::HRESULT = -2147220892i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_CANNOT_CONNECT: ::windows_sys::core::HRESULT = -2147220969i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_CANNOT_LOAD_SOURCE_FILTER: ::windows_sys::core::HRESULT = -2147220927i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_CANNOT_RENDER: ::windows_sys::core::HRESULT = -2147220968i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_CERTIFICATION_FAILURE: ::windows_sys::core::HRESULT = -2147220843i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_CHANGING_FORMAT: ::windows_sys::core::HRESULT = -2147220967i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_CIRCULAR_GRAPH: ::windows_sys::core::HRESULT = -2147220943i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_CODECAPI_ENUMERATED: ::windows_sys::core::HRESULT = -2147220719i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_CODECAPI_LINEAR_RANGE: ::windows_sys::core::HRESULT = -2147220720i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_CODECAPI_NO_CURRENT_VALUE: ::windows_sys::core::HRESULT = -2147220716i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_CODECAPI_NO_DEFAULT: ::windows_sys::core::HRESULT = -2147220717i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_COLOR_KEY_SET: ::windows_sys::core::HRESULT = -2147220962i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_COPYPROT_FAILED: ::windows_sys::core::HRESULT = -2147220867i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_CORRUPT_GRAPH_FILE: ::windows_sys::core::HRESULT = -2147220939i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DDRAW_CAPS_NOT_SUITABLE: ::windows_sys::core::HRESULT = -2147220877i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DDRAW_VERSION_NOT_SUITABLE: ::windows_sys::core::HRESULT = -2147220868i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DUPLICATE_NAME: ::windows_sys::core::HRESULT = -2147220947i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DVD_CHAPTER_DOES_NOT_EXIST: ::windows_sys::core::HRESULT = -2147220715i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DVD_CMD_CANCELLED: ::windows_sys::core::HRESULT = -2147220861i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DVD_DECNOTENOUGH: ::windows_sys::core::HRESULT = -2147220869i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DVD_GRAPHNOTREADY: ::windows_sys::core::HRESULT = -2147220871i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DVD_INCOMPATIBLE_REGION: ::windows_sys::core::HRESULT = -2147220857i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DVD_INVALIDDOMAIN: ::windows_sys::core::HRESULT = -2147220873i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DVD_INVALID_DISC: ::windows_sys::core::HRESULT = -2147220847i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DVD_LOW_PARENTAL_LEVEL: ::windows_sys::core::HRESULT = -2147220854i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DVD_MENU_DOES_NOT_EXIST: ::windows_sys::core::HRESULT = -2147220862i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DVD_NONBLOCKING: ::windows_sys::core::HRESULT = -2147220836i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DVD_NON_EVR_RENDERER_IN_FILTER_GRAPH: ::windows_sys::core::HRESULT = -2147220834i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DVD_NOT_IN_KARAOKE_MODE: ::windows_sys::core::HRESULT = -2147220853i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DVD_NO_ATTRIBUTES: ::windows_sys::core::HRESULT = -2147220856i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DVD_NO_BUTTON: ::windows_sys::core::HRESULT = -2147220872i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DVD_NO_GOUP_PGC: ::windows_sys::core::HRESULT = -2147220855i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DVD_NO_RESUME_INFORMATION: ::windows_sys::core::HRESULT = -2147220846i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DVD_OPERATION_INHIBITED: ::windows_sys::core::HRESULT = -2147220874i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DVD_RENDERFAIL: ::windows_sys::core::HRESULT = -2147220870i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DVD_RESOLUTION_ERROR: ::windows_sys::core::HRESULT = -2147220833i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DVD_STATE_CORRUPT: ::windows_sys::core::HRESULT = -2147220859i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DVD_STATE_WRONG_DISC: ::windows_sys::core::HRESULT = -2147220858i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DVD_STATE_WRONG_VERSION: ::windows_sys::core::HRESULT = -2147220860i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DVD_STREAM_DISABLED: ::windows_sys::core::HRESULT = -2147220849i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DVD_TITLE_UNKNOWN: ::windows_sys::core::HRESULT = -2147220848i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DVD_TOO_MANY_RENDERERS_IN_FILTER_GRAPH: ::windows_sys::core::HRESULT = -2147220835i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DVD_VMR9_INCOMPATIBLEDEC: ::windows_sys::core::HRESULT = -2147220838i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_DVD_WRONG_SPEED: ::windows_sys::core::HRESULT = -2147220863i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_ENUM_OUT_OF_RANGE: ::windows_sys::core::HRESULT = -2147220944i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_ENUM_OUT_OF_SYNC: ::windows_sys::core::HRESULT = -2147220989i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_FILE_TOO_SHORT: ::windows_sys::core::HRESULT = -2147220925i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_FILTER_ACTIVE: ::windows_sys::core::HRESULT = -2147220987i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_FRAME_STEP_UNSUPPORTED: ::windows_sys::core::HRESULT = -2147220850i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_INVALIDMEDIATYPE: ::windows_sys::core::HRESULT = -2147220992i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_INVALIDSUBTYPE: ::windows_sys::core::HRESULT = -2147220991i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_INVALID_CLSID: ::windows_sys::core::HRESULT = -2147220921i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_INVALID_DIRECTION: ::windows_sys::core::HRESULT = -2147220984i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_INVALID_FILE_FORMAT: ::windows_sys::core::HRESULT = -2147220945i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_INVALID_FILE_VERSION: ::windows_sys::core::HRESULT = -2147220924i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_INVALID_MEDIA_TYPE: ::windows_sys::core::HRESULT = -2147220920i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_INVALID_RECT: ::windows_sys::core::HRESULT = -2147220951i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_IN_FULLSCREEN_MODE: ::windows_sys::core::HRESULT = -2147220933i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_MEDIA_TIME_NOT_SET: ::windows_sys::core::HRESULT = -2147220911i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_MONO_AUDIO_HW: ::windows_sys::core::HRESULT = -2147220909i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_MPEG_NOT_CONSTRAINED: ::windows_sys::core::HRESULT = -2147220898i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NEED_OWNER: ::windows_sys::core::HRESULT = -2147220990i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NOT_ALLOWED_TO_SAVE: ::windows_sys::core::HRESULT = -2147220942i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NOT_COMMITTED: ::windows_sys::core::HRESULT = -2147220975i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NOT_CONNECTED: ::windows_sys::core::HRESULT = -2147220983i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NOT_FOUND: ::windows_sys::core::HRESULT = -2147220970i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NOT_IN_GRAPH: ::windows_sys::core::HRESULT = -2147220897i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NOT_OVERLAY_CONNECTION: ::windows_sys::core::HRESULT = -2147220965i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NOT_PAUSED: ::windows_sys::core::HRESULT = -2147220955i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NOT_RUNNING: ::windows_sys::core::HRESULT = -2147220954i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NOT_SAMPLE_CONNECTION: ::windows_sys::core::HRESULT = -2147220964i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NOT_STOPPED: ::windows_sys::core::HRESULT = -2147220956i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NO_ACCEPTABLE_TYPES: ::windows_sys::core::HRESULT = -2147220985i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NO_ADVISE_SET: ::windows_sys::core::HRESULT = -2147220935i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NO_ALLOCATOR: ::windows_sys::core::HRESULT = -2147220982i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NO_AUDIO_HARDWARE: ::windows_sys::core::HRESULT = -2147220906i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NO_CAPTURE_HARDWARE: ::windows_sys::core::HRESULT = -2147220875i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NO_CLOCK: ::windows_sys::core::HRESULT = -2147220973i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NO_COLOR_KEY_FOUND: ::windows_sys::core::HRESULT = -2147220961i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NO_COLOR_KEY_SET: ::windows_sys::core::HRESULT = -2147220966i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NO_COPP_HW: ::windows_sys::core::HRESULT = -2147220837i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NO_DECOMPRESSOR: ::windows_sys::core::HRESULT = -2147220907i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NO_DISPLAY_PALETTE: ::windows_sys::core::HRESULT = -2147220959i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NO_FULLSCREEN: ::windows_sys::core::HRESULT = -2147220934i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NO_INTERFACE: ::windows_sys::core::HRESULT = -2147220971i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NO_MODEX_AVAILABLE: ::windows_sys::core::HRESULT = -2147220936i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NO_PALETTE_AVAILABLE: ::windows_sys::core::HRESULT = -2147220960i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NO_SINK: ::windows_sys::core::HRESULT = -2147220972i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NO_TIME_FORMAT: ::windows_sys::core::HRESULT = -2147220895i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NO_TIME_FORMAT_SET: ::windows_sys::core::HRESULT = -2147220910i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NO_TRANSPORT: ::windows_sys::core::HRESULT = -2147220890i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NO_TYPES: ::windows_sys::core::HRESULT = -2147220986i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_NO_VP_HARDWARE: ::windows_sys::core::HRESULT = -2147220876i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_OUT_OF_VIDEO_MEMORY: ::windows_sys::core::HRESULT = -2147220879i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_PALETTE_SET: ::windows_sys::core::HRESULT = -2147220963i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_PIN_ALREADY_BLOCKED: ::windows_sys::core::HRESULT = -2147220844i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_PIN_ALREADY_BLOCKED_ON_THIS_THREAD: ::windows_sys::core::HRESULT = -2147220845i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_PROCESSOR_NOT_SUITABLE: ::windows_sys::core::HRESULT = -2147220901i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_READ_ONLY: ::windows_sys::core::HRESULT = -2147220894i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_RPZA: ::windows_sys::core::HRESULT = -2147220903i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_RUNTIME_ERROR: ::windows_sys::core::HRESULT = -2147220981i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_SAMPLE_REJECTED: ::windows_sys::core::HRESULT = -2147220949i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_SAMPLE_REJECTED_EOS: ::windows_sys::core::HRESULT = -2147220948i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_SAMPLE_TIME_NOT_SET: ::windows_sys::core::HRESULT = -2147220919i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_SIZENOTSET: ::windows_sys::core::HRESULT = -2147220974i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_START_TIME_AFTER_END: ::windows_sys::core::HRESULT = -2147220952i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_STATE_CHANGED: ::windows_sys::core::HRESULT = -2147220957i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_TIMEOUT: ::windows_sys::core::HRESULT = -2147220946i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_TIME_ALREADY_PASSED: ::windows_sys::core::HRESULT = -2147220941i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_TIME_EXPIRED: ::windows_sys::core::HRESULT = -2147220865i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_TOO_MANY_COLORS: ::windows_sys::core::HRESULT = -2147220958i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_TYPE_NOT_ACCEPTED: ::windows_sys::core::HRESULT = -2147220950i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_UNKNOWN_FILE_TYPE: ::windows_sys::core::HRESULT = -2147220928i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_UNSUPPORTED_AUDIO: ::windows_sys::core::HRESULT = -2147220900i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_UNSUPPORTED_STREAM: ::windows_sys::core::HRESULT = -2147220891i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_UNSUPPORTED_VIDEO: ::windows_sys::core::HRESULT = -2147220899i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_VMR_NOT_IN_MIXER_MODE: ::windows_sys::core::HRESULT = -2147220842i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_VMR_NO_AP_SUPPLIED: ::windows_sys::core::HRESULT = -2147220841i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_VMR_NO_DEINTERLACE_HW: ::windows_sys::core::HRESULT = -2147220840i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_VMR_NO_PROCAMP_HW: ::windows_sys::core::HRESULT = -2147220839i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_VP_NEGOTIATION_FAILED: ::windows_sys::core::HRESULT = -2147220878i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_E_WRONG_STATE: ::windows_sys::core::HRESULT = -2147220953i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct VFW_FILTERLIST {
    pub cFilters: u32,
    pub aClsId: [::windows_sys::core::GUID; 1],
}
impl ::core::marker::Copy for VFW_FILTERLIST {}
impl ::core::clone::Clone for VFW_FILTERLIST {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_FIRST_CODE: u32 = 512u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_S_AUDIO_NOT_RENDERED: ::windows_sys::core::HRESULT = 262744i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_S_CANT_CUE: ::windows_sys::core::HRESULT = 262760i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_S_CONNECTIONS_DEFERRED: ::windows_sys::core::HRESULT = 262726i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_S_DUPLICATE_NAME: ::windows_sys::core::HRESULT = 262701i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_S_DVD_CHANNEL_CONTENTS_NOT_AVAILABLE: ::windows_sys::core::HRESULT = 262796i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_S_DVD_NON_ONE_SEQUENTIAL: ::windows_sys::core::HRESULT = 262784i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_S_DVD_NOT_ACCURATE: ::windows_sys::core::HRESULT = 262797i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_S_DVD_RENDER_STATUS: ::windows_sys::core::HRESULT = 262944i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_S_ESTIMATED: ::windows_sys::core::HRESULT = 262752i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_S_MEDIA_TYPE_IGNORED: ::windows_sys::core::HRESULT = 262740i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_S_NOPREVIEWPIN: ::windows_sys::core::HRESULT = 262782i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_S_NO_MORE_ITEMS: ::windows_sys::core::HRESULT = 262403i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_S_NO_STOP_TIME: ::windows_sys::core::HRESULT = 262768i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_S_PARTIAL_RENDER: ::windows_sys::core::HRESULT = 262722i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_S_RESERVED: ::windows_sys::core::HRESULT = 262755i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_S_RESOURCE_NOT_NEEDED: ::windows_sys::core::HRESULT = 262736i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_S_RPZA: ::windows_sys::core::HRESULT = 262746i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_S_SOME_DATA_IGNORED: ::windows_sys::core::HRESULT = 262725i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_S_STATE_INTERMEDIATE: ::windows_sys::core::HRESULT = 262711i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_S_STREAM_OFF: ::windows_sys::core::HRESULT = 262759i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VFW_S_VIDEO_NOT_RENDERED: ::windows_sys::core::HRESULT = 262743i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VIDEOENCODER_BITRATE_MODE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ConstantBitRate: VIDEOENCODER_BITRATE_MODE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VariableBitRateAverage: VIDEOENCODER_BITRATE_MODE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VariableBitRatePeak: VIDEOENCODER_BITRATE_MODE = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct VIDEOINFO {
    pub rcSource: super::super::Foundation::RECT,
    pub rcTarget: super::super::Foundation::RECT,
    pub dwBitRate: u32,
    pub dwBitErrorRate: u32,
    pub AvgTimePerFrame: i64,
    pub bmiHeader: super::super::Graphics::Gdi::BITMAPINFOHEADER,
    pub Anonymous: VIDEOINFO_0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for VIDEOINFO {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for VIDEOINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub union VIDEOINFO_0 {
    pub bmiColors: [super::super::Graphics::Gdi::RGBQUAD; 256],
    pub dwBitMasks: [u32; 3],
    pub TrueColorInfo: TRUECOLORINFO,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for VIDEOINFO_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for VIDEOINFO_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct VIDEOINFOHEADER {
    pub rcSource: super::super::Foundation::RECT,
    pub rcTarget: super::super::Foundation::RECT,
    pub dwBitRate: u32,
    pub dwBitErrorRate: u32,
    pub AvgTimePerFrame: i64,
    pub bmiHeader: super::super::Graphics::Gdi::BITMAPINFOHEADER,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for VIDEOINFOHEADER {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for VIDEOINFOHEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct VIDEOINFOHEADER2 {
    pub rcSource: super::super::Foundation::RECT,
    pub rcTarget: super::super::Foundation::RECT,
    pub dwBitRate: u32,
    pub dwBitErrorRate: u32,
    pub AvgTimePerFrame: i64,
    pub dwInterlaceFlags: u32,
    pub dwCopyProtectFlags: u32,
    pub dwPictAspectRatioX: u32,
    pub dwPictAspectRatioY: u32,
    pub Anonymous: VIDEOINFOHEADER2_0,
    pub dwReserved2: u32,
    pub bmiHeader: super::super::Graphics::Gdi::BITMAPINFOHEADER,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for VIDEOINFOHEADER2 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for VIDEOINFOHEADER2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub union VIDEOINFOHEADER2_0 {
    pub dwControlFlags: u32,
    pub dwReserved1: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for VIDEOINFOHEADER2_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for VIDEOINFOHEADER2_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct VIDEO_STREAM_CONFIG_CAPS {
    pub guid: ::windows_sys::core::GUID,
    pub VideoStandard: u32,
    pub InputSize: super::super::Foundation::SIZE,
    pub MinCroppingSize: super::super::Foundation::SIZE,
    pub MaxCroppingSize: super::super::Foundation::SIZE,
    pub CropGranularityX: i32,
    pub CropGranularityY: i32,
    pub CropAlignX: i32,
    pub CropAlignY: i32,
    pub MinOutputSize: super::super::Foundation::SIZE,
    pub MaxOutputSize: super::super::Foundation::SIZE,
    pub OutputGranularityX: i32,
    pub OutputGranularityY: i32,
    pub StretchTapsX: i32,
    pub StretchTapsY: i32,
    pub ShrinkTapsX: i32,
    pub ShrinkTapsY: i32,
    pub MinFrameInterval: i64,
    pub MaxFrameInterval: i64,
    pub MinBitsPerSecond: i32,
    pub MaxBitsPerSecond: i32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for VIDEO_STREAM_CONFIG_CAPS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for VIDEO_STREAM_CONFIG_CAPS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Direct3D9\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct3D9"))]
pub struct VMR9AllocationInfo {
    pub dwFlags: u32,
    pub dwWidth: u32,
    pub dwHeight: u32,
    pub Format: super::super::Graphics::Direct3D9::D3DFORMAT,
    pub Pool: super::super::Graphics::Direct3D9::D3DPOOL,
    pub MinBuffers: u32,
    pub szAspectRatio: super::super::Foundation::SIZE,
    pub szNativeSize: super::super::Foundation::SIZE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct3D9"))]
impl ::core::marker::Copy for VMR9AllocationInfo {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct3D9"))]
impl ::core::clone::Clone for VMR9AllocationInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Direct3D9\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct3D9", feature = "Win32_Graphics_Gdi"))]
pub struct VMR9AlphaBitmap {
    pub dwFlags: u32,
    pub hdc: super::super::Graphics::Gdi::HDC,
    pub pDDS: super::super::Graphics::Direct3D9::IDirect3DSurface9,
    pub rSrc: super::super::Foundation::RECT,
    pub rDest: VMR9NormalizedRect,
    pub fAlpha: f32,
    pub clrSrcKey: u32,
    pub dwFilterMode: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct3D9", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for VMR9AlphaBitmap {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct3D9", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for VMR9AlphaBitmap {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VMR9AlphaBitmapFlags = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9AlphaBitmap_Disable: VMR9AlphaBitmapFlags = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9AlphaBitmap_hDC: VMR9AlphaBitmapFlags = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9AlphaBitmap_EntireDDS: VMR9AlphaBitmapFlags = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9AlphaBitmap_SrcColorKey: VMR9AlphaBitmapFlags = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9AlphaBitmap_SrcRect: VMR9AlphaBitmapFlags = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9AlphaBitmap_FilterMode: VMR9AlphaBitmapFlags = 32i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VMR9AspectRatioMode = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9ARMode_None: VMR9AspectRatioMode = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9ARMode_LetterBox: VMR9AspectRatioMode = 1i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct VMR9DeinterlaceCaps {
    pub dwSize: u32,
    pub dwNumPreviousOutputFrames: u32,
    pub dwNumForwardRefSamples: u32,
    pub dwNumBackwardRefSamples: u32,
    pub DeinterlaceTechnology: VMR9DeinterlaceTech,
}
impl ::core::marker::Copy for VMR9DeinterlaceCaps {}
impl ::core::clone::Clone for VMR9DeinterlaceCaps {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VMR9DeinterlacePrefs = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DeinterlacePref9_NextBest: VMR9DeinterlacePrefs = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DeinterlacePref9_BOB: VMR9DeinterlacePrefs = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DeinterlacePref9_Weave: VMR9DeinterlacePrefs = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DeinterlacePref9_Mask: VMR9DeinterlacePrefs = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VMR9DeinterlaceTech = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DeinterlaceTech9_Unknown: VMR9DeinterlaceTech = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DeinterlaceTech9_BOBLineReplicate: VMR9DeinterlaceTech = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DeinterlaceTech9_BOBVerticalStretch: VMR9DeinterlaceTech = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DeinterlaceTech9_MedianFiltering: VMR9DeinterlaceTech = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DeinterlaceTech9_EdgeFiltering: VMR9DeinterlaceTech = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DeinterlaceTech9_FieldAdaptive: VMR9DeinterlaceTech = 32i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DeinterlaceTech9_PixelAdaptive: VMR9DeinterlaceTech = 64i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DeinterlaceTech9_MotionVectorSteered: VMR9DeinterlaceTech = 128i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct VMR9Frequency {
    pub dwNumerator: u32,
    pub dwDenominator: u32,
}
impl ::core::marker::Copy for VMR9Frequency {}
impl ::core::clone::Clone for VMR9Frequency {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VMR9MixerPrefs = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref9_NoDecimation: VMR9MixerPrefs = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref9_DecimateOutput: VMR9MixerPrefs = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref9_ARAdjustXorY: VMR9MixerPrefs = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref9_NonSquareMixing: VMR9MixerPrefs = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref9_DecimateMask: VMR9MixerPrefs = 15i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref9_BiLinearFiltering: VMR9MixerPrefs = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref9_PointFiltering: VMR9MixerPrefs = 32i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref9_AnisotropicFiltering: VMR9MixerPrefs = 64i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref9_PyramidalQuadFiltering: VMR9MixerPrefs = 128i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref9_GaussianQuadFiltering: VMR9MixerPrefs = 256i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref9_FilteringReserved: VMR9MixerPrefs = 3584i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref9_FilteringMask: VMR9MixerPrefs = 4080i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref9_RenderTargetRGB: VMR9MixerPrefs = 4096i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref9_RenderTargetYUV: VMR9MixerPrefs = 8192i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref9_RenderTargetReserved: VMR9MixerPrefs = 1032192i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref9_RenderTargetMask: VMR9MixerPrefs = 1044480i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref9_DynamicSwitchToBOB: VMR9MixerPrefs = 1048576i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref9_DynamicDecimateBy2: VMR9MixerPrefs = 2097152i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref9_DynamicReserved: VMR9MixerPrefs = 12582912i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref9_DynamicMask: VMR9MixerPrefs = 15728640i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VMR9Mode = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9Mode_Windowed: VMR9Mode = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9Mode_Windowless: VMR9Mode = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9Mode_Renderless: VMR9Mode = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9Mode_Mask: VMR9Mode = 7i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct VMR9MonitorInfo {
    pub uDevID: u32,
    pub rcMonitor: super::super::Foundation::RECT,
    pub hMon: super::super::Graphics::Gdi::HMONITOR,
    pub dwFlags: u32,
    pub szDevice: [u16; 32],
    pub szDescription: [u16; 512],
    pub liDriverVersion: i64,
    pub dwVendorId: u32,
    pub dwDeviceId: u32,
    pub dwSubSysId: u32,
    pub dwRevision: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for VMR9MonitorInfo {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for VMR9MonitorInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct VMR9NormalizedRect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}
impl ::core::marker::Copy for VMR9NormalizedRect {}
impl ::core::clone::Clone for VMR9NormalizedRect {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VMR9PresentationFlags = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9Sample_SyncPoint: VMR9PresentationFlags = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9Sample_Preroll: VMR9PresentationFlags = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9Sample_Discontinuity: VMR9PresentationFlags = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9Sample_TimeValid: VMR9PresentationFlags = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9Sample_SrcDstRectsValid: VMR9PresentationFlags = 16i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Direct3D9\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct3D9"))]
pub struct VMR9PresentationInfo {
    pub dwFlags: u32,
    pub lpSurf: super::super::Graphics::Direct3D9::IDirect3DSurface9,
    pub rtStart: i64,
    pub rtEnd: i64,
    pub szAspectRatio: super::super::Foundation::SIZE,
    pub rcSrc: super::super::Foundation::RECT,
    pub rcDst: super::super::Foundation::RECT,
    pub dwReserved1: u32,
    pub dwReserved2: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct3D9"))]
impl ::core::marker::Copy for VMR9PresentationInfo {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct3D9"))]
impl ::core::clone::Clone for VMR9PresentationInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct VMR9ProcAmpControl {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub Brightness: f32,
    pub Contrast: f32,
    pub Hue: f32,
    pub Saturation: f32,
}
impl ::core::marker::Copy for VMR9ProcAmpControl {}
impl ::core::clone::Clone for VMR9ProcAmpControl {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VMR9ProcAmpControlFlags = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ProcAmpControl9_Brightness: VMR9ProcAmpControlFlags = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ProcAmpControl9_Contrast: VMR9ProcAmpControlFlags = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ProcAmpControl9_Hue: VMR9ProcAmpControlFlags = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ProcAmpControl9_Saturation: VMR9ProcAmpControlFlags = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const ProcAmpControl9_Mask: VMR9ProcAmpControlFlags = 15i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct VMR9ProcAmpControlRange {
    pub dwSize: u32,
    pub dwProperty: VMR9ProcAmpControlFlags,
    pub MinValue: f32,
    pub MaxValue: f32,
    pub DefaultValue: f32,
    pub StepSize: f32,
}
impl ::core::marker::Copy for VMR9ProcAmpControlRange {}
impl ::core::clone::Clone for VMR9ProcAmpControlRange {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VMR9RenderPrefs = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const RenderPrefs9_DoNotRenderBorder: VMR9RenderPrefs = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const RenderPrefs9_Mask: VMR9RenderPrefs = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VMR9SurfaceAllocationFlags = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9AllocFlag_3DRenderTarget: VMR9SurfaceAllocationFlags = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9AllocFlag_DXVATarget: VMR9SurfaceAllocationFlags = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9AllocFlag_TextureSurface: VMR9SurfaceAllocationFlags = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9AllocFlag_OffscreenSurface: VMR9SurfaceAllocationFlags = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9AllocFlag_RGBDynamicSwitch: VMR9SurfaceAllocationFlags = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9AllocFlag_UsageReserved: VMR9SurfaceAllocationFlags = 224i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9AllocFlag_UsageMask: VMR9SurfaceAllocationFlags = 255i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct VMR9VideoDesc {
    pub dwSize: u32,
    pub dwSampleWidth: u32,
    pub dwSampleHeight: u32,
    pub SampleFormat: VMR9_SampleFormat,
    pub dwFourCC: u32,
    pub InputSampleFreq: VMR9Frequency,
    pub OutputFrameFreq: VMR9Frequency,
}
impl ::core::marker::Copy for VMR9VideoDesc {}
impl ::core::clone::Clone for VMR9VideoDesc {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Graphics_Direct3D9\"`*"]
#[cfg(feature = "Win32_Graphics_Direct3D9")]
pub struct VMR9VideoStreamInfo {
    pub pddsVideoSurface: super::super::Graphics::Direct3D9::IDirect3DSurface9,
    pub dwWidth: u32,
    pub dwHeight: u32,
    pub dwStrmID: u32,
    pub fAlpha: f32,
    pub rNormal: VMR9NormalizedRect,
    pub rtStart: i64,
    pub rtEnd: i64,
    pub SampleFormat: VMR9_SampleFormat,
}
#[cfg(feature = "Win32_Graphics_Direct3D9")]
impl ::core::marker::Copy for VMR9VideoStreamInfo {}
#[cfg(feature = "Win32_Graphics_Direct3D9")]
impl ::core::clone::Clone for VMR9VideoStreamInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VMR9_SampleFormat = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9_SampleReserved: VMR9_SampleFormat = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9_SampleProgressiveFrame: VMR9_SampleFormat = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9_SampleFieldInterleavedEvenFirst: VMR9_SampleFormat = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9_SampleFieldInterleavedOddFirst: VMR9_SampleFormat = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9_SampleFieldSingleEven: VMR9_SampleFormat = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR9_SampleFieldSingleOdd: VMR9_SampleFormat = 6i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_DirectDraw", feature = "Win32_Graphics_Gdi"))]
pub struct VMRALLOCATIONINFO {
    pub dwFlags: u32,
    pub lpHdr: *mut super::super::Graphics::Gdi::BITMAPINFOHEADER,
    pub lpPixFmt: *mut super::super::Graphics::DirectDraw::DDPIXELFORMAT,
    pub szAspectRatio: super::super::Foundation::SIZE,
    pub dwMinBuffers: u32,
    pub dwMaxBuffers: u32,
    pub dwInterlaceFlags: u32,
    pub szNativeSize: super::super::Foundation::SIZE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_DirectDraw", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for VMRALLOCATIONINFO {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_DirectDraw", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for VMRALLOCATIONINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_DirectDraw", feature = "Win32_Graphics_Gdi"))]
pub struct VMRALPHABITMAP {
    pub dwFlags: u32,
    pub hdc: super::super::Graphics::Gdi::HDC,
    pub pDDS: super::super::Graphics::DirectDraw::IDirectDrawSurface7,
    pub rSrc: super::super::Foundation::RECT,
    pub rDest: NORMALIZEDRECT,
    pub fAlpha: f32,
    pub clrSrcKey: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_DirectDraw", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for VMRALPHABITMAP {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_DirectDraw", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for VMRALPHABITMAP {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMRBITMAP_DISABLE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMRBITMAP_ENTIREDDS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMRBITMAP_HDC: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMRBITMAP_SRCCOLORKEY: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMRBITMAP_SRCRECT: u32 = 16u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct VMRDeinterlaceCaps {
    pub dwSize: u32,
    pub dwNumPreviousOutputFrames: u32,
    pub dwNumForwardRefSamples: u32,
    pub dwNumBackwardRefSamples: u32,
    pub DeinterlaceTechnology: VMRDeinterlaceTech,
}
impl ::core::marker::Copy for VMRDeinterlaceCaps {}
impl ::core::clone::Clone for VMRDeinterlaceCaps {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VMRDeinterlacePrefs = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DeinterlacePref_NextBest: VMRDeinterlacePrefs = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DeinterlacePref_BOB: VMRDeinterlacePrefs = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DeinterlacePref_Weave: VMRDeinterlacePrefs = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DeinterlacePref_Mask: VMRDeinterlacePrefs = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VMRDeinterlaceTech = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DeinterlaceTech_Unknown: VMRDeinterlaceTech = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DeinterlaceTech_BOBLineReplicate: VMRDeinterlaceTech = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DeinterlaceTech_BOBVerticalStretch: VMRDeinterlaceTech = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DeinterlaceTech_MedianFiltering: VMRDeinterlaceTech = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DeinterlaceTech_EdgeFiltering: VMRDeinterlaceTech = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DeinterlaceTech_FieldAdaptive: VMRDeinterlaceTech = 32i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DeinterlaceTech_PixelAdaptive: VMRDeinterlaceTech = 64i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DeinterlaceTech_MotionVectorSteered: VMRDeinterlaceTech = 128i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct VMRFrequency {
    pub dwNumerator: u32,
    pub dwDenominator: u32,
}
impl ::core::marker::Copy for VMRFrequency {}
impl ::core::clone::Clone for VMRFrequency {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct VMRGUID {
    pub pGUID: *mut ::windows_sys::core::GUID,
    pub GUID: ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for VMRGUID {}
impl ::core::clone::Clone for VMRGUID {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct VMRMONITORINFO {
    pub guid: VMRGUID,
    pub rcMonitor: super::super::Foundation::RECT,
    pub hMon: super::super::Graphics::Gdi::HMONITOR,
    pub dwFlags: u32,
    pub szDevice: [u16; 32],
    pub szDescription: [u16; 256],
    pub liDriverVersion: i64,
    pub dwVendorId: u32,
    pub dwDeviceId: u32,
    pub dwSubSysId: u32,
    pub dwRevision: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for VMRMONITORINFO {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for VMRMONITORINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VMRMixerPrefs = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref_NoDecimation: VMRMixerPrefs = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref_DecimateOutput: VMRMixerPrefs = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref_ARAdjustXorY: VMRMixerPrefs = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref_DecimationReserved: VMRMixerPrefs = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref_DecimateMask: VMRMixerPrefs = 15i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref_BiLinearFiltering: VMRMixerPrefs = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref_PointFiltering: VMRMixerPrefs = 32i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref_FilteringMask: VMRMixerPrefs = 240i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref_RenderTargetRGB: VMRMixerPrefs = 256i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref_RenderTargetYUV: VMRMixerPrefs = 4096i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref_RenderTargetYUV420: VMRMixerPrefs = 512i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref_RenderTargetYUV422: VMRMixerPrefs = 1024i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref_RenderTargetYUV444: VMRMixerPrefs = 2048i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref_RenderTargetReserved: VMRMixerPrefs = 57344i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref_RenderTargetMask: VMRMixerPrefs = 65280i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref_DynamicSwitchToBOB: VMRMixerPrefs = 65536i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref_DynamicDecimateBy2: VMRMixerPrefs = 131072i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref_DynamicReserved: VMRMixerPrefs = 786432i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const MixerPref_DynamicMask: VMRMixerPrefs = 983040i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VMRMode = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMRMode_Windowed: VMRMode = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMRMode_Windowless: VMRMode = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMRMode_Renderless: VMRMode = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMRMode_Mask: VMRMode = 7i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_DirectDraw\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_DirectDraw"))]
pub struct VMRPRESENTATIONINFO {
    pub dwFlags: u32,
    pub lpSurf: super::super::Graphics::DirectDraw::IDirectDrawSurface7,
    pub rtStart: i64,
    pub rtEnd: i64,
    pub szAspectRatio: super::super::Foundation::SIZE,
    pub rcSrc: super::super::Foundation::RECT,
    pub rcDst: super::super::Foundation::RECT,
    pub dwTypeSpecificFlags: u32,
    pub dwInterlaceFlags: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_DirectDraw"))]
impl ::core::marker::Copy for VMRPRESENTATIONINFO {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_DirectDraw"))]
impl ::core::clone::Clone for VMRPRESENTATIONINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VMRPresentationFlags = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMRSample_SyncPoint: VMRPresentationFlags = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMRSample_Preroll: VMRPresentationFlags = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMRSample_Discontinuity: VMRPresentationFlags = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMRSample_TimeValid: VMRPresentationFlags = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMRSample_SrcDstRectsValid: VMRPresentationFlags = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VMRRenderPrefs = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const RenderPrefs_RestrictToInitialMonitor: VMRRenderPrefs = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const RenderPrefs_ForceOffscreen: VMRRenderPrefs = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const RenderPrefs_ForceOverlays: VMRRenderPrefs = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const RenderPrefs_AllowOverlays: VMRRenderPrefs = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const RenderPrefs_AllowOffscreen: VMRRenderPrefs = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const RenderPrefs_DoNotRenderColorKeyAndBorder: VMRRenderPrefs = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const RenderPrefs_Reserved: VMRRenderPrefs = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const RenderPrefs_PreferAGPMemWhenMixing: VMRRenderPrefs = 32i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const RenderPrefs_Mask: VMRRenderPrefs = 63i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VMRSurfaceAllocationFlags = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMAP_PIXELFORMAT_VALID: VMRSurfaceAllocationFlags = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMAP_3D_TARGET: VMRSurfaceAllocationFlags = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMAP_ALLOW_SYSMEM: VMRSurfaceAllocationFlags = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMAP_FORCE_SYSMEM: VMRSurfaceAllocationFlags = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMAP_DIRECTED_FLIP: VMRSurfaceAllocationFlags = 16i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMAP_DXVA_TARGET: VMRSurfaceAllocationFlags = 32i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Graphics_DirectDraw\"`*"]
#[cfg(feature = "Win32_Graphics_DirectDraw")]
pub struct VMRVIDEOSTREAMINFO {
    pub pddsVideoSurface: super::super::Graphics::DirectDraw::IDirectDrawSurface7,
    pub dwWidth: u32,
    pub dwHeight: u32,
    pub dwStrmID: u32,
    pub fAlpha: f32,
    pub ddClrKey: super::super::Graphics::DirectDraw::DDCOLORKEY,
    pub rNormal: NORMALIZEDRECT,
}
#[cfg(feature = "Win32_Graphics_DirectDraw")]
impl ::core::marker::Copy for VMRVIDEOSTREAMINFO {}
#[cfg(feature = "Win32_Graphics_DirectDraw")]
impl ::core::clone::Clone for VMRVIDEOSTREAMINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct VMRVideoDesc {
    pub dwSize: u32,
    pub dwSampleWidth: u32,
    pub dwSampleHeight: u32,
    pub SingleFieldPerSample: super::super::Foundation::BOOL,
    pub dwFourCC: u32,
    pub InputSampleFreq: VMRFrequency,
    pub OutputFrameFreq: VMRFrequency,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for VMRVideoDesc {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for VMRVideoDesc {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VMR_ASPECT_RATIO_MODE = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR_ARMODE_NONE: VMR_ASPECT_RATIO_MODE = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR_ARMODE_LETTER_BOX: VMR_ASPECT_RATIO_MODE = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR_NOTSUPPORTED: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR_RENDER_DEVICE_OVERLAY: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR_RENDER_DEVICE_SYSMEM: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR_RENDER_DEVICE_VIDMEM: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VMR_SUPPORTED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VfwCaptureDialogs = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VfwCaptureDialog_Source: VfwCaptureDialogs = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VfwCaptureDialog_Format: VfwCaptureDialogs = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VfwCaptureDialog_Display: VfwCaptureDialogs = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VfwCompressDialogs = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VfwCompressDialog_Config: VfwCompressDialogs = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VfwCompressDialog_About: VfwCompressDialogs = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VfwCompressDialog_QueryConfig: VfwCompressDialogs = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VfwCompressDialog_QueryAbout: VfwCompressDialogs = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VideoControlFlags = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VideoControlFlag_FlipHorizontal: VideoControlFlags = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VideoControlFlag_FlipVertical: VideoControlFlags = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VideoControlFlag_ExternalTriggerEnable: VideoControlFlags = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VideoControlFlag_Trigger: VideoControlFlags = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VideoCopyProtectionType = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VideoCopyProtectionMacrovisionBasic: VideoCopyProtectionType = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VideoCopyProtectionMacrovisionCBI: VideoCopyProtectionType = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VideoProcAmpFlags = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VideoProcAmp_Flags_Auto: VideoProcAmpFlags = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VideoProcAmp_Flags_Manual: VideoProcAmpFlags = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type VideoProcAmpProperty = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VideoProcAmp_Brightness: VideoProcAmpProperty = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VideoProcAmp_Contrast: VideoProcAmpProperty = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VideoProcAmp_Hue: VideoProcAmpProperty = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VideoProcAmp_Saturation: VideoProcAmpProperty = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VideoProcAmp_Sharpness: VideoProcAmpProperty = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VideoProcAmp_Gamma: VideoProcAmpProperty = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VideoProcAmp_ColorEnable: VideoProcAmpProperty = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VideoProcAmp_WhiteBalance: VideoProcAmpProperty = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VideoProcAmp_BacklightCompensation: VideoProcAmpProperty = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const VideoProcAmp_Gain: VideoProcAmpProperty = 9i32;
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct WMDRMProtectionInfo {
    pub wszKID: [u16; 25],
    pub qwCounter: u64,
    pub qwIndex: u64,
    pub bOffset: u8,
}
impl ::core::marker::Copy for WMDRMProtectionInfo {}
impl ::core::clone::Clone for WMDRMProtectionInfo {
    fn clone(&self) -> Self {
        *self
    }
}
pub const XDSCodec: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3301229811, data2: 73, data3: 20011, data4: [152, 251, 149, 55, 246, 206, 81, 109] };
pub const XDSToRat: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3318072816, data2: 15036, data3: 4566, data4: [178, 91, 0, 192, 79, 160, 192, 38] };
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type _AMRESCTL_RESERVEFLAGS = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMRESCTL_RESERVEFLAGS_RESERVE: _AMRESCTL_RESERVEFLAGS = 0i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMRESCTL_RESERVEFLAGS_UNRESERVE: _AMRESCTL_RESERVEFLAGS = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type _AMSTREAMSELECTENABLEFLAGS = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMSTREAMSELECTENABLE_ENABLE: _AMSTREAMSELECTENABLEFLAGS = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMSTREAMSELECTENABLE_ENABLEALL: _AMSTREAMSELECTENABLEFLAGS = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type _AMSTREAMSELECTINFOFLAGS = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMSTREAMSELECTINFO_ENABLED: _AMSTREAMSELECTINFOFLAGS = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AMSTREAMSELECTINFO_EXCLUSIVE: _AMSTREAMSELECTINFOFLAGS = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type _AM_AUDIO_RENDERER_STAT_PARAM = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_AUDREND_STAT_PARAM_BREAK_COUNT: _AM_AUDIO_RENDERER_STAT_PARAM = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_AUDREND_STAT_PARAM_SLAVE_MODE: _AM_AUDIO_RENDERER_STAT_PARAM = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_AUDREND_STAT_PARAM_SILENCE_DUR: _AM_AUDIO_RENDERER_STAT_PARAM = 3i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_AUDREND_STAT_PARAM_LAST_BUFFER_DUR: _AM_AUDIO_RENDERER_STAT_PARAM = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_AUDREND_STAT_PARAM_DISCONTINUITIES: _AM_AUDIO_RENDERER_STAT_PARAM = 5i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_AUDREND_STAT_PARAM_SLAVE_RATE: _AM_AUDIO_RENDERER_STAT_PARAM = 6i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_AUDREND_STAT_PARAM_SLAVE_DROPWRITE_DUR: _AM_AUDIO_RENDERER_STAT_PARAM = 7i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_AUDREND_STAT_PARAM_SLAVE_HIGHLOWERROR: _AM_AUDIO_RENDERER_STAT_PARAM = 8i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_AUDREND_STAT_PARAM_SLAVE_LASTHIGHLOWERROR: _AM_AUDIO_RENDERER_STAT_PARAM = 9i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_AUDREND_STAT_PARAM_SLAVE_ACCUMERROR: _AM_AUDIO_RENDERER_STAT_PARAM = 10i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_AUDREND_STAT_PARAM_BUFFERFULLNESS: _AM_AUDIO_RENDERER_STAT_PARAM = 11i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_AUDREND_STAT_PARAM_JITTER: _AM_AUDIO_RENDERER_STAT_PARAM = 12i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type _AM_FILTER_MISC_FLAGS = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_FILTER_MISC_FLAGS_IS_RENDERER: _AM_FILTER_MISC_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_FILTER_MISC_FLAGS_IS_SOURCE: _AM_FILTER_MISC_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type _AM_INTF_SEARCH_FLAGS = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_INTF_SEARCH_INPUT_PIN: _AM_INTF_SEARCH_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_INTF_SEARCH_OUTPUT_PIN: _AM_INTF_SEARCH_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_INTF_SEARCH_FILTER: _AM_INTF_SEARCH_FLAGS = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type _AM_OVERLAY_NOTIFY_FLAGS = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_OVERLAY_NOTIFY_VISIBLE_CHANGE: _AM_OVERLAY_NOTIFY_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_OVERLAY_NOTIFY_SOURCE_CHANGE: _AM_OVERLAY_NOTIFY_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_OVERLAY_NOTIFY_DEST_CHANGE: _AM_OVERLAY_NOTIFY_FLAGS = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type _AM_PIN_FLOW_CONTROL_BLOCK_FLAGS = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PIN_FLOW_CONTROL_BLOCK: _AM_PIN_FLOW_CONTROL_BLOCK_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type _AM_PUSHSOURCE_FLAGS = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PUSHSOURCECAPS_INTERNAL_RM: _AM_PUSHSOURCE_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PUSHSOURCECAPS_NOT_LIVE: _AM_PUSHSOURCE_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PUSHSOURCECAPS_PRIVATE_CLOCK: _AM_PUSHSOURCE_FLAGS = 4i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PUSHSOURCEREQS_USE_STREAM_CLOCK: _AM_PUSHSOURCE_FLAGS = 65536i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_PUSHSOURCEREQS_USE_CLOCK_CHAIN: _AM_PUSHSOURCE_FLAGS = 131072i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type _AM_RENSDEREXFLAGS = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const AM_RENDEREX_RENDERTOEXISTINGRENDERERS: _AM_RENSDEREXFLAGS = 1i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type _DVDECODERRESOLUTION = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVDECODERRESOLUTION_720x480: _DVDECODERRESOLUTION = 1000i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVDECODERRESOLUTION_360x240: _DVDECODERRESOLUTION = 1001i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVDECODERRESOLUTION_180x120: _DVDECODERRESOLUTION = 1002i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVDECODERRESOLUTION_88x60: _DVDECODERRESOLUTION = 1003i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type _DVENCODERFORMAT = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVENCODERFORMAT_DVSD: _DVENCODERFORMAT = 2007i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVENCODERFORMAT_DVHD: _DVENCODERFORMAT = 2008i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVENCODERFORMAT_DVSL: _DVENCODERFORMAT = 2009i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type _DVENCODERRESOLUTION = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVENCODERRESOLUTION_720x480: _DVENCODERRESOLUTION = 2012i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVENCODERRESOLUTION_360x240: _DVENCODERRESOLUTION = 2013i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVENCODERRESOLUTION_180x120: _DVENCODERRESOLUTION = 2014i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVENCODERRESOLUTION_88x60: _DVENCODERRESOLUTION = 2015i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type _DVENCODERVIDEOFORMAT = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVENCODERVIDEOFORMAT_NTSC: _DVENCODERVIDEOFORMAT = 2000i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVENCODERVIDEOFORMAT_PAL: _DVENCODERVIDEOFORMAT = 2001i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type _DVRESOLUTION = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVRESOLUTION_FULL: _DVRESOLUTION = 1000i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVRESOLUTION_HALF: _DVRESOLUTION = 1001i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVRESOLUTION_QUARTER: _DVRESOLUTION = 1002i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const DVRESOLUTION_DC: _DVRESOLUTION = 1003i32;
pub type _IMSVidCtlEvents = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub type _REM_FILTER_FLAGS = i32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const REMFILTERF_LEAVECONNECTED: _REM_FILTER_FLAGS = 1i32;
#[repr(C, packed(2))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct _avitcdlindex {
    pub fcc: u32,
    pub cb: u32,
    pub wLongsPerEntry: u16,
    pub bIndexSubType: u8,
    pub bIndexType: u8,
    pub nEntriesInUse: u32,
    pub dwChunkId: u32,
    pub dwReserved: [u32; 3],
    pub aIndex: [AVITCDLINDEX_ENTRY; 584],
    pub adwTrailingFill: [u32; 3512],
}
impl ::core::marker::Copy for _avitcdlindex {}
impl ::core::clone::Clone for _avitcdlindex {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(2))]
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub struct _avitimedindex {
    pub fcc: u32,
    pub cb: u32,
    pub wLongsPerEntry: u16,
    pub bIndexSubType: u8,
    pub bIndexType: u8,
    pub nEntriesInUse: u32,
    pub dwChunkId: u32,
    pub qwBaseOffset: u64,
    pub dwReserved_3: u32,
    pub aIndex: [AVITIMEDINDEX_ENTRY; 1362],
    pub adwTrailingFill: [u32; 2734],
}
impl ::core::marker::Copy for _avitimedindex {}
impl ::core::clone::Clone for _avitimedindex {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszExcludeScriptStreamDeliverySynchronization: &str = "ExcludeScriptStreamDeliverySynchronization";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingAlbumArtist: &str = "WM/AlbumArtist";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingAlbumCoverURL: &str = "WM/AlbumCoverURL";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingAlbumTitle: &str = "WM/AlbumTitle";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingAspectRatioX: &str = "AspectRatioX";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingAspectRatioY: &str = "AspectRatioY";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingAuthor: &str = "Author";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingBannerImageData: &str = "BannerImageData";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingBannerImageType: &str = "BannerImageType";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingBannerImageURL: &str = "BannerImageURL";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingBitrate: &str = "Bitrate";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingBroadcast: &str = "Broadcast";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingComposer: &str = "WM/Composer";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingCopyright: &str = "Copyright";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingCopyrightURL: &str = "CopyrightURL";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingCurrentBitrate: &str = "CurrentBitrate";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingDRM_Flags: &str = "DRM_Flags";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingDRM_Level: &str = "DRM_Level";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingDescription: &str = "Description";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingDuration: &str = "Duration";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingFileSize: &str = "FileSize";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingGenre: &str = "WM/Genre";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingGenreID: &str = "WM/GenreID";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingHasArbitraryDataStream: &str = "HasArbitraryDataStream";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingHasAttachedImages: &str = "HasAttachedImages";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingHasAudio: &str = "HasAudio";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingHasFileTransferStream: &str = "HasFileTransferStream";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingHasImage: &str = "HasImage";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingHasScript: &str = "HasScript";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingHasVideo: &str = "HasVideo";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingIsVBR: &str = "IsVBR";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingLyrics: &str = "WM/Lyrics";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingMCDI: &str = "WM/MCDI";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingNSCAddress: &str = "NSC_Address";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingNSCDescription: &str = "NSC_Description";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingNSCEmail: &str = "NSC_Email";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingNSCName: &str = "NSC_Name";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingNSCPhone: &str = "NSC_Phone";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingNumberOfFrames: &str = "NumberOfFrames";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingOptimalBitrate: &str = "OptimalBitrate";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingPromotionURL: &str = "WM/PromotionURL";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingProtected: &str = "Is_Protected";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingRating: &str = "Rating";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingSeekable: &str = "Seekable";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingSignature_Name: &str = "Signature_Name";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingSkipBackward: &str = "Can_Skip_Backward";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingSkipForward: &str = "Can_Skip_Forward";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingStridable: &str = "Stridable";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingTitle: &str = "Title";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingToolName: &str = "WM/ToolName";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingToolVersion: &str = "WM/ToolVersion";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingTrack: &str = "WM/Track";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingTrackNumber: &str = "WM/TrackNumber";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingTrusted: &str = "Is_Trusted";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingUse_DRM: &str = "Use_DRM";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const g_wszStreamBufferRecordingYear: &str = "WM/Year";
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const iBLUE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const iEGA_COLORS: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const iGREEN: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const iMASK_COLORS: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const iMAXBITS: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const iPALETTE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const iPALETTE_COLORS: u32 = 256u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const iRED: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Media_DirectShow\"`*"]
pub const iTRUECOLOR: u32 = 16u32;

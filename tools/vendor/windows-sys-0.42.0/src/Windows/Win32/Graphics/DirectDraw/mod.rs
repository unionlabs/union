#[cfg_attr(windows, link(name = "windows"))]
extern "system" {
    #[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
    pub fn DirectDrawCreate(lpguid: *mut ::windows_sys::core::GUID, lplpdd: *mut IDirectDraw, punkouter: ::windows_sys::core::IUnknown) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
    pub fn DirectDrawCreateClipper(dwflags: u32, lplpddclipper: *mut IDirectDrawClipper, punkouter: ::windows_sys::core::IUnknown) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
    pub fn DirectDrawCreateEx(lpguid: *mut ::windows_sys::core::GUID, lplpdd: *mut *mut ::core::ffi::c_void, iid: *const ::windows_sys::core::GUID, punkouter: ::windows_sys::core::IUnknown) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DirectDrawEnumerateA(lpcallback: LPDDENUMCALLBACKA, lpcontext: *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn DirectDrawEnumerateExA(lpcallback: LPDDENUMCALLBACKEXA, lpcontext: *mut ::core::ffi::c_void, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn DirectDrawEnumerateExW(lpcallback: LPDDENUMCALLBACKEXW, lpcontext: *mut ::core::ffi::c_void, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DirectDrawEnumerateW(lpcallback: LPDDENUMCALLBACKW, lpcontext: *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
}
pub type IDDVideoPortContainer = *mut ::core::ffi::c_void;
pub type IDirectDraw = *mut ::core::ffi::c_void;
pub type IDirectDraw2 = *mut ::core::ffi::c_void;
pub type IDirectDraw4 = *mut ::core::ffi::c_void;
pub type IDirectDraw7 = *mut ::core::ffi::c_void;
pub type IDirectDrawClipper = *mut ::core::ffi::c_void;
pub type IDirectDrawColorControl = *mut ::core::ffi::c_void;
pub type IDirectDrawGammaControl = *mut ::core::ffi::c_void;
pub type IDirectDrawKernel = *mut ::core::ffi::c_void;
pub type IDirectDrawPalette = *mut ::core::ffi::c_void;
pub type IDirectDrawSurface = *mut ::core::ffi::c_void;
pub type IDirectDrawSurface2 = *mut ::core::ffi::c_void;
pub type IDirectDrawSurface3 = *mut ::core::ffi::c_void;
pub type IDirectDrawSurface4 = *mut ::core::ffi::c_void;
pub type IDirectDrawSurface7 = *mut ::core::ffi::c_void;
pub type IDirectDrawSurfaceKernel = *mut ::core::ffi::c_void;
pub type IDirectDrawVideoPort = *mut ::core::ffi::c_void;
pub type IDirectDrawVideoPortNotify = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const ACCESSRECT_BROKEN: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const ACCESSRECT_NOTHOLDINGWIN16LOCK: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const ACCESSRECT_VRAMSTYLE: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const CCHDEVICENAME: u32 = 32u32;
pub const CLSID_DirectDraw: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3619098336, data2: 17216, data3: 4559, data4: [176, 99, 0, 32, 175, 194, 205, 53] };
pub const CLSID_DirectDraw7: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1009799574, data2: 20699, data3: 4563, data4: [156, 254, 0, 192, 79, 217, 48, 197] };
pub const CLSID_DirectDrawClipper: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1496848288, data2: 32179, data3: 4559, data4: [162, 222, 0, 170, 0, 185, 51, 86] };
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFMT_INTERNAL_D15S1: u32 = 73u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFMT_INTERNAL_D24S8: u32 = 75u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFMT_INTERNAL_D24X8: u32 = 77u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFMT_INTERNAL_D32: u32 = 71u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFMT_INTERNAL_S1D15: u32 = 72u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFMT_INTERNAL_S8D24: u32 = 74u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFMT_INTERNAL_X8D24: u32 = 76u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFORMAT_MEMBEROFGROUP_ARGB: i32 = 524288i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFORMAT_OP_3DACCELERATION: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFORMAT_OP_AUTOGENMIPMAP: i32 = 4194304i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFORMAT_OP_BUMPMAP: i32 = 65536i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFORMAT_OP_CONVERT_TO_ARGB: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFORMAT_OP_CUBETEXTURE: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFORMAT_OP_DISPLAYMODE: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFORMAT_OP_DMAP: i32 = 131072i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFORMAT_OP_NOALPHABLEND: i32 = 2097152i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFORMAT_OP_NOFILTER: i32 = 262144i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFORMAT_OP_NOTEXCOORDWRAPNORMIP: i32 = 16777216i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFORMAT_OP_OFFSCREENPLAIN: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFORMAT_OP_OFFSCREEN_RENDERTARGET: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFORMAT_OP_PIXELSIZE: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFORMAT_OP_SAME_FORMAT_RENDERTARGET: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFORMAT_OP_SAME_FORMAT_UP_TO_ALPHA_RENDERTARGET: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFORMAT_OP_SRGBREAD: i32 = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFORMAT_OP_SRGBWRITE: i32 = 1048576i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFORMAT_OP_TEXTURE: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFORMAT_OP_VERTEXTEXTURE: i32 = 8388608i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFORMAT_OP_VOLUMETEXTURE: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFORMAT_OP_ZSTENCIL: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const D3DFORMAT_OP_ZSTENCIL_WITH_ARBITRARY_COLOR_DEPTH: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DCICOMMAND: u32 = 3075u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDABLT_SRCOVERDEST: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDAL_IMPLICIT: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBD_1: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBD_16: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBD_2: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBD_24: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBD_32: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBD_4: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBD_8: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLTFAST_DESTCOLORKEY: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLTFAST_DONOTWAIT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLTFAST_NOCOLORKEY: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLTFAST_SRCCOLORKEY: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLTFAST_WAIT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLTFX_ARITHSTRETCHY: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLTFX_MIRRORLEFTRIGHT: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLTFX_MIRRORUPDOWN: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLTFX_NOTEARING: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLTFX_ROTATE180: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLTFX_ROTATE270: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLTFX_ROTATE90: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLTFX_ZBUFFERBASEDEST: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLTFX_ZBUFFERRANGE: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_AFLAGS: i32 = -2147483648i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_ALPHADEST: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_ALPHADESTCONSTOVERRIDE: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_ALPHADESTNEG: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_ALPHADESTSURFACEOVERRIDE: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_ALPHAEDGEBLEND: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_ALPHASRC: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_ALPHASRCCONSTOVERRIDE: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_ALPHASRCNEG: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_ALPHASRCSURFACEOVERRIDE: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_ASYNC: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_COLORFILL: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_DDFX: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_DDROPS: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_DEPTHFILL: i32 = 33554432i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_DONOTWAIT: i32 = 134217728i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_EXTENDED_FLAGS: i32 = 1073741824i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_EXTENDED_LINEAR_CONTENT: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_KEYDEST: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_KEYDESTOVERRIDE: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_KEYSRC: i32 = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_KEYSRCOVERRIDE: i32 = 65536i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_LAST_PRESENTATION: i32 = 536870912i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_PRESENTATION: i32 = 268435456i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_ROP: i32 = 131072i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_ROTATIONANGLE: i32 = 262144i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_WAIT: i32 = 16777216i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_ZBUFFER: i32 = 524288i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_ZBUFFERDESTCONSTOVERRIDE: i32 = 1048576i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_ZBUFFERDESTOVERRIDE: i32 = 2097152i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_ZBUFFERSRCCONSTOVERRIDE: i32 = 4194304i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDBLT_ZBUFFERSRCOVERRIDE: i32 = 8388608i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_AUTOFLIPOVERLAY: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_CANAUTOGENMIPMAP: i32 = 1073741824i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_CANBOBHARDWARE: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_CANBOBINTERLEAVED: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_CANBOBNONINTERLEAVED: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_CANCALIBRATEGAMMA: i32 = 1048576i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_CANDROPZ16BIT: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_CANFLIPODDEVEN: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_CANMANAGERESOURCE: i32 = 268435456i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_CANMANAGETEXTURE: i32 = 8388608i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_CANRENDERWINDOWED: i32 = 524288i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_CANSHARERESOURCE: i32 = -2147483648i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_CERTIFIED: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_COLORCONTROLOVERLAY: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_COLORCONTROLPRIMARY: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_COPYFOURCC: i32 = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_DYNAMICTEXTURES: i32 = 536870912i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_FLIPINTERVAL: i32 = 2097152i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_FLIPNOVSYNC: i32 = 4194304i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_NO2DDURING3DSCENE: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_NONLOCALVIDMEM: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_NONLOCALVIDMEMCAPS: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_NOPAGELOCKREQUIRED: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_PRIMARYGAMMA: i32 = 131072i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_RESERVED1: i32 = 134217728i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_STEREO: i32 = 33554432i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_SYSTONONLOCAL_AS_SYSTOLOCAL: i32 = 67108864i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_TEXMANINNONLOCALVIDMEM: i32 = 16777216i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_VIDEOPORT: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS2_WIDESURFACES: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_3D: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_ALIGNBOUNDARYDEST: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_ALIGNBOUNDARYSRC: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_ALIGNSIZEDEST: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_ALIGNSIZESRC: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_ALIGNSTRIDE: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_ALPHA: i32 = 8388608i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_BANKSWITCHED: i32 = 134217728i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_BLT: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_BLTCOLORFILL: i32 = 67108864i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_BLTDEPTHFILL: i32 = 268435456i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_BLTFOURCC: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_BLTQUEUE: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_BLTSTRETCH: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_CANBLTSYSMEM: i32 = -2147483648i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_CANCLIP: i32 = 536870912i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_CANCLIPSTRETCHED: i32 = 1073741824i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_COLORKEY: i32 = 4194304i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_COLORKEYHWASSIST: i32 = 16777216i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_GDI: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_NOHARDWARE: i32 = 33554432i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_OVERLAY: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_OVERLAYCANTCLIP: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_OVERLAYFOURCC: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_OVERLAYSTRETCH: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_PALETTE: i32 = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_PALETTEVSYNC: i32 = 65536i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_READSCANLINE: i32 = 131072i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_RESERVED1: i32 = 262144i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_VBI: i32 = 524288i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_ZBLTS: i32 = 1048576i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCAPS_ZOVERLAYS: i32 = 2097152i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCKEYCAPS_DESTBLT: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCKEYCAPS_DESTBLTCLRSPACE: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCKEYCAPS_DESTBLTCLRSPACEYUV: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCKEYCAPS_DESTBLTYUV: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCKEYCAPS_DESTOVERLAY: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCKEYCAPS_DESTOVERLAYCLRSPACE: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCKEYCAPS_DESTOVERLAYCLRSPACEYUV: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCKEYCAPS_DESTOVERLAYONEACTIVE: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCKEYCAPS_DESTOVERLAYYUV: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCKEYCAPS_NOCOSTOVERLAY: i32 = 262144i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCKEYCAPS_SRCBLT: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCKEYCAPS_SRCBLTCLRSPACE: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCKEYCAPS_SRCBLTCLRSPACEYUV: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCKEYCAPS_SRCBLTYUV: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCKEYCAPS_SRCOVERLAY: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCKEYCAPS_SRCOVERLAYCLRSPACE: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCKEYCAPS_SRCOVERLAYCLRSPACEYUV: i32 = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCKEYCAPS_SRCOVERLAYONEACTIVE: i32 = 65536i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCKEYCAPS_SRCOVERLAYYUV: i32 = 131072i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCKEY_COLORSPACE: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCKEY_DESTBLT: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCKEY_DESTOVERLAY: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCKEY_SRCBLT: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCKEY_SRCOVERLAY: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCOLOR_BRIGHTNESS: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCOLOR_COLORENABLE: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCOLOR_CONTRAST: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCOLOR_GAMMA: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCOLOR_HUE: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCOLOR_SATURATION: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCOLOR_SHARPNESS: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCREATEDRIVEROBJECT: u32 = 10u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCREATE_EMULATIONONLY: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDCREATE_HARDWAREONLY: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDEDM_REFRESHRATES: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDEDM_STANDARDVGAMODES: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDEM_MODEFAILED: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDEM_MODEPASSED: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDENUMOVERLAYZ_BACKTOFRONT: i32 = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDENUMOVERLAYZ_FRONTTOBACK: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDENUMRET_CANCEL: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDENUMRET_OK: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDENUMSURFACES_ALL: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDENUMSURFACES_CANBECREATED: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDENUMSURFACES_DOESEXIST: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDENUMSURFACES_MATCH: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDENUMSURFACES_NOMATCH: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDENUM_ATTACHEDSECONDARYDEVICES: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDENUM_DETACHEDSECONDARYDEVICES: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDENUM_NONDISPLAYDEVICES: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDERR_NOTINITIALIZED: i32 = -2147221008i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFLIP_DONOTWAIT: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFLIP_EVEN: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFLIP_INTERVAL2: i32 = 33554432i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFLIP_INTERVAL3: i32 = 50331648i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFLIP_INTERVAL4: i32 = 67108864i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFLIP_NOVSYNC: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFLIP_ODD: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFLIP_STEREO: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFLIP_WAIT: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXALPHACAPS_BLTALPHAEDGEBLEND: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXALPHACAPS_BLTALPHAPIXELS: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXALPHACAPS_BLTALPHAPIXELSNEG: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXALPHACAPS_BLTALPHASURFACES: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXALPHACAPS_BLTALPHASURFACESNEG: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXALPHACAPS_OVERLAYALPHAEDGEBLEND: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXALPHACAPS_OVERLAYALPHAPIXELS: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXALPHACAPS_OVERLAYALPHAPIXELSNEG: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXALPHACAPS_OVERLAYALPHASURFACES: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXALPHACAPS_OVERLAYALPHASURFACESNEG: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_BLTALPHA: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_BLTARITHSTRETCHY: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_BLTARITHSTRETCHYN: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_BLTFILTER: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_BLTMIRRORLEFTRIGHT: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_BLTMIRRORUPDOWN: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_BLTROTATION: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_BLTROTATION90: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_BLTSHRINKX: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_BLTSHRINKXN: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_BLTSHRINKY: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_BLTSHRINKYN: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_BLTSTRETCHX: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_BLTSTRETCHXN: i32 = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_BLTSTRETCHY: i32 = 65536i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_BLTSTRETCHYN: i32 = 131072i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_OVERLAYALPHA: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_OVERLAYARITHSTRETCHY: i32 = 262144i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_OVERLAYARITHSTRETCHYN: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_OVERLAYDEINTERLACE: i32 = 536870912i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_OVERLAYFILTER: i32 = 262144i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_OVERLAYMIRRORLEFTRIGHT: i32 = 134217728i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_OVERLAYMIRRORUPDOWN: i32 = 268435456i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_OVERLAYSHRINKX: i32 = 524288i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_OVERLAYSHRINKXN: i32 = 1048576i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_OVERLAYSHRINKY: i32 = 2097152i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_OVERLAYSHRINKYN: i32 = 4194304i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_OVERLAYSTRETCHX: i32 = 8388608i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_OVERLAYSTRETCHXN: i32 = 16777216i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_OVERLAYSTRETCHY: i32 = 33554432i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDFXCAPS_OVERLAYSTRETCHYN: i32 = 67108864i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDGBS_CANBLT: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDGBS_ISBLTDONE: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDGDI_GETHOSTIDENTIFIER: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDGET32BITDRIVERNAME: u32 = 11u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDGFS_CANFLIP: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDGFS_ISFLIPDONE: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHALINFO_GETDRIVERINFO2: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHALINFO_GETDRIVERINFOSET: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHALINFO_ISPRIMARYDISPLAY: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHALINFO_MODEXILLEGAL: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_APP_DLLNAME: &str = "DDRAW.DLL";
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_CB32_CANCREATESURFACE: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_CB32_CREATEPALETTE: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_CB32_CREATESURFACE: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_CB32_DESTROYDRIVER: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_CB32_FLIPTOGDISURFACE: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_CB32_GETSCANLINE: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_CB32_MAPMEMORY: i32 = -2147483648i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_CB32_SETCOLORKEY: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_CB32_SETEXCLUSIVEMODE: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_CB32_SETMODE: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_CB32_WAITFORVERTICALBLANK: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_COLOR_COLORCONTROL: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_CREATESURFACEEX_SWAPHANDLES: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_D3DBUFCB32_CANCREATED3DBUF: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_D3DBUFCB32_CREATED3DBUF: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_D3DBUFCB32_DESTROYD3DBUF: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_D3DBUFCB32_LOCKD3DBUF: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_D3DBUFCB32_UNLOCKD3DBUF: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_DRIVER_DLLNAME: &str = "DDRAW16.DLL";
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_DRIVER_HANDLED: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_DRIVER_NOCKEYHW: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_DRIVER_NOTHANDLED: i32 = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_EXEBUFCB32_CANCREATEEXEBUF: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_EXEBUFCB32_CREATEEXEBUF: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_EXEBUFCB32_DESTROYEXEBUF: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_EXEBUFCB32_LOCKEXEBUF: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_EXEBUFCB32_UNLOCKEXEBUF: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_KERNEL_SYNCSURFACEDATA: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_KERNEL_SYNCVIDEOPORTDATA: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_MISC2CB32_ALPHABLT: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_MISC2CB32_CREATESURFACEEX: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_MISC2CB32_DESTROYDDLOCAL: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_MISC2CB32_GETDRIVERSTATE: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_MISCCB32_GETAVAILDRIVERMEMORY: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_MISCCB32_GETHEAPALIGNMENT: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_MISCCB32_GETSYSMEMBLTSTATUS: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_MISCCB32_UPDATENONLOCALHEAP: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_MOCOMP32_BEGINFRAME: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_MOCOMP32_CREATE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_MOCOMP32_DESTROY: u32 = 512u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_MOCOMP32_ENDFRAME: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_MOCOMP32_GETCOMPBUFFINFO: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_MOCOMP32_GETFORMATS: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_MOCOMP32_GETGUIDS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_MOCOMP32_GETINTERNALINFO: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_MOCOMP32_QUERYSTATUS: u32 = 256u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_MOCOMP32_RENDER: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_NTCB32_FLIPTOGDISURFACE: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_NTCB32_FREEDRIVERMEMORY: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_NTCB32_SETEXCLUSIVEMODE: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_PALCB32_DESTROYPALETTE: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_PALCB32_SETENTRIES: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_PLEASEALLOC_BLOCKSIZE: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_PLEASEALLOC_LINEARSIZE: i32 = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_PLEASEALLOC_USERMEM: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_PRIVATECAP_ATOMICSURFACECREATION: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_PRIVATECAP_NOTIFYPRIMARYCREATION: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_PRIVATECAP_RESERVED1: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_SURFCB32_ADDATTACHEDSURFACE: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_SURFCB32_BLT: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_SURFCB32_DESTROYSURFACE: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_SURFCB32_FLIP: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_SURFCB32_GETBLTSTATUS: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_SURFCB32_GETFLIPSTATUS: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_SURFCB32_LOCK: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_SURFCB32_RESERVED4: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_SURFCB32_SETCLIPLIST: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_SURFCB32_SETCOLORKEY: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_SURFCB32_SETOVERLAYPOSITION: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_SURFCB32_SETPALETTE: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_SURFCB32_UNLOCK: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_SURFCB32_UPDATEOVERLAY: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_VPORT32_CANCREATEVIDEOPORT: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_VPORT32_COLORCONTROL: i32 = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_VPORT32_CREATEVIDEOPORT: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_VPORT32_DESTROY: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_VPORT32_FLIP: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_VPORT32_GETAUTOFLIPSURF: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_VPORT32_GETBANDWIDTH: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_VPORT32_GETCONNECT: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_VPORT32_GETFIELD: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_VPORT32_GETFLIPSTATUS: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_VPORT32_GETINPUTFORMATS: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_VPORT32_GETLINE: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_VPORT32_GETOUTPUTFORMATS: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_VPORT32_GETSIGNALSTATUS: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_VPORT32_UPDATE: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDHAL_VPORT32_WAITFORSYNC: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDIRQ_BUSMASTER: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDIRQ_DISPLAY_VSYNC: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDIRQ_RESERVED1: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDIRQ_VPORT0_LINE: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDIRQ_VPORT0_VSYNC: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDIRQ_VPORT1_LINE: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDIRQ_VPORT1_VSYNC: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDIRQ_VPORT2_LINE: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDIRQ_VPORT2_VSYNC: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDIRQ_VPORT3_LINE: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDIRQ_VPORT3_VSYNC: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDIRQ_VPORT4_LINE: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDIRQ_VPORT4_VSYNC: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDIRQ_VPORT5_LINE: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDIRQ_VPORT5_VSYNC: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDIRQ_VPORT6_LINE: i32 = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDIRQ_VPORT6_VSYNC: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDIRQ_VPORT7_LINE: i32 = 131072i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDIRQ_VPORT7_VSYNC: i32 = 65536i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDIRQ_VPORT8_LINE: i32 = 524288i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDIRQ_VPORT8_VSYNC: i32 = 262144i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDIRQ_VPORT9_LINE: i32 = 131072i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDIRQ_VPORT9_VSYNC: i32 = 65536i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDKERNELCAPS_AUTOFLIP: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDKERNELCAPS_CAPTURE_INVERTED: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDKERNELCAPS_CAPTURE_NONLOCALVIDMEM: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDKERNELCAPS_CAPTURE_SYSMEM: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDKERNELCAPS_FIELDPOLARITY: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDKERNELCAPS_FLIPOVERLAY: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDKERNELCAPS_FLIPVIDEOPORT: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDKERNELCAPS_LOCK: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDKERNELCAPS_SETSTATE: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDKERNELCAPS_SKIPFIELDS: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDLOCK_DISCARDCONTENTS: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDLOCK_DONOTWAIT: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDLOCK_EVENT: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDLOCK_HASVOLUMETEXTUREBOXRECT: i32 = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDLOCK_NODIRTYUPDATE: i32 = 65536i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDLOCK_NOOVERWRITE: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDLOCK_NOSYSLOCK: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDLOCK_OKTOSWAP: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDLOCK_READONLY: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDLOCK_SURFACEMEMORYPTR: i32 = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDLOCK_WAIT: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDLOCK_WRITEONLY: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDMCQUERY_READ: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDMODEINFO_MAXREFRESH: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDMODEINFO_MODEX: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDMODEINFO_PALETTIZED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDMODEINFO_STANDARDVGA: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDMODEINFO_STEREO: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDMODEINFO_UNSUPPORTED: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDNEWCALLBACKFNS: u32 = 12u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOSDCAPS_MONOLITHICMIPMAP: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOSDCAPS_OPTCOMPRESSED: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOSDCAPS_OPTREORDERED: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOSDCAPS_VALIDOSCAPS: i32 = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOSDCAPS_VALIDSCAPS: i32 = 805324800i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOSD_ALL: i32 = 15i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOSD_COMPRESSION_RATIO: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOSD_GUID: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOSD_OSCAPS: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOSD_SCAPS: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVERFX_ARITHSTRETCHY: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVERFX_DEINTERLACE: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVERFX_MIRRORLEFTRIGHT: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVERFX_MIRRORUPDOWN: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVERZ_INSERTINBACKOF: i32 = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVERZ_INSERTINFRONTOF: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVERZ_MOVEBACKWARD: i32 = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVERZ_MOVEFORWARD: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVERZ_SENDTOBACK: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVERZ_SENDTOFRONT: i32 = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVER_ADDDIRTYRECT: i32 = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVER_ALPHADEST: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVER_ALPHADESTCONSTOVERRIDE: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVER_ALPHADESTNEG: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVER_ALPHADESTSURFACEOVERRIDE: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVER_ALPHAEDGEBLEND: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVER_ALPHASRC: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVER_ALPHASRCCONSTOVERRIDE: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVER_ALPHASRCNEG: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVER_ALPHASRCSURFACEOVERRIDE: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVER_ARGBSCALEFACTORS: i32 = 33554432i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVER_AUTOFLIP: i32 = 1048576i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVER_BOB: i32 = 2097152i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVER_BOBHARDWARE: i32 = 16777216i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVER_DDFX: i32 = 524288i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVER_DEGRADEARGBSCALING: i32 = 67108864i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVER_HIDE: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVER_INTERLEAVED: i32 = 8388608i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVER_KEYDEST: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVER_KEYDESTOVERRIDE: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVER_KEYSRC: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVER_KEYSRCOVERRIDE: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVER_OVERRIDEBOBWEAVE: i32 = 4194304i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVER_REFRESHALL: i32 = 131072i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVER_REFRESHDIRTYRECTS: i32 = 65536i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDOVER_SHOW: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPCAPS_1BIT: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPCAPS_2BIT: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPCAPS_4BIT: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPCAPS_8BIT: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPCAPS_8BITENTRIES: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPCAPS_ALLOW256: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPCAPS_ALPHA: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPCAPS_INITIALIZE: i32 = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPCAPS_PRIMARYSURFACE: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPCAPS_PRIMARYSURFACELEFT: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPCAPS_VSYNC: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPF_ALPHA: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPF_ALPHAPIXELS: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPF_ALPHAPREMULT: i32 = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPF_BUMPDUDV: i32 = 524288i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPF_BUMPLUMINANCE: i32 = 262144i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPF_COMPRESSED: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPF_D3DFORMAT: i32 = 2097152i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPF_FOURCC: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPF_LUMINANCE: i32 = 131072i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPF_NOVEL_TEXTURE_FORMAT: i32 = 1048576i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPF_PALETTEINDEXED1: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPF_PALETTEINDEXED2: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPF_PALETTEINDEXED4: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPF_PALETTEINDEXED8: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPF_PALETTEINDEXEDTO8: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPF_RGB: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPF_RGBTOYUV: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPF_STENCILBUFFER: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPF_YUV: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPF_ZBUFFER: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDPF_ZPIXELS: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWICLIP_INMASTERSPRITELIST: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWICLIP_ISINITIALIZED: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWICLIP_WATCHWINDOW: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWILCL_ACTIVENO: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWILCL_ACTIVEYES: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWILCL_ALLOWMODEX: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWILCL_ATTEMPTEDD3DCONTEXT: i32 = 262144i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWILCL_CREATEDWINDOW: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWILCL_CURSORCLIPPED: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWILCL_DIRECTDRAW7: i32 = 131072i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWILCL_DIRECTDRAW8: i32 = 2097152i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWILCL_DIRTYDC: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWILCL_DISABLEINACTIVATE: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWILCL_DX8DRIVER: i32 = 1048576i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWILCL_EXPLICITMONITOR: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWILCL_FPUPRESERVE: i32 = 524288i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWILCL_FPUSETUP: i32 = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWILCL_HASEXCLUSIVEMODE: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWILCL_HOOKEDHWND: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWILCL_ISFULLSCREEN: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWILCL_MODEHASBEENCHANGED: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWILCL_MULTITHREADED: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWILCL_POWEREDDOWN: i32 = 65536i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWILCL_SETCOOPCALLED: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWILCL_V1SCLBEHAVIOUR: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWIPAL_16: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWIPAL_2: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWIPAL_256: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWIPAL_4: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWIPAL_ALLOW256: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWIPAL_ALPHA: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWIPAL_DIRTY: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWIPAL_EXCLUSIVE: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWIPAL_GDI: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWIPAL_INHEL: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWIPAL_STORED_16: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWIPAL_STORED_24: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWIPAL_STORED_8: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWIPAL_STORED_8INDEX: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURFGBL_DDHELDONTFREE: i32 = 1048576i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURFGBL_DX8SURFACE: i32 = 524288i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURFGBL_FASTLOCKHELD: i32 = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURFGBL_HARDWAREOPDEST: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURFGBL_HARDWAREOPSOURCE: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURFGBL_IMPLICITHANDLE: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURFGBL_ISCLIENTMEM: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURFGBL_ISGDISURFACE: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURFGBL_LATEALLOCATELINEAR: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURFGBL_LOCKBROKEN: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURFGBL_LOCKNOTHOLDINGWIN16LOCK: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURFGBL_LOCKVRAMSTYLE: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURFGBL_MEMFREE: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURFGBL_NOTIFYWHENUNLOCKED: i32 = 2097152i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURFGBL_READONLYLOCKHELD: i32 = 65536i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURFGBL_RESERVED0: i32 = -2147483648i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURFGBL_SOFTWAREAUTOFLIP: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURFGBL_SYSMEMEXECUTEBUFFER: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURFGBL_SYSMEMREQUESTED: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURFGBL_VPORTDATA: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURFGBL_VPORTINTERLEAVED: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_ATTACHED: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_ATTACHED_FROM: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_BACKBUFFER: i32 = 134217728i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_DATAISALIASED: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_DCIBUSY: i32 = 536870912i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_DCILOCK: i32 = -2147483648i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_DRIVERMANAGED: i32 = 1073741824i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_FRONTBUFFER: i32 = 67108864i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_GETDCNULL: i32 = 1073741824i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_HASCKEYDESTBLT: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_HASCKEYDESTOVERLAY: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_HASCKEYSRCBLT: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_HASCKEYSRCOVERLAY: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_HASDC: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_HASOVERLAYDATA: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_HASPIXELFORMAT: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_HELCB: i32 = 33554432i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_HW_CKEYDESTBLT: i32 = 2097152i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_HW_CKEYDESTOVERLAY: i32 = 1048576i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_HW_CKEYSRCBLT: i32 = 8388608i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_HW_CKEYSRCOVERLAY: i32 = 4194304i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_IMPLICITCREATE: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_IMPLICITROOT: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_INMASTERSPRITELIST: i32 = 16777216i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_INVALID: i32 = 268435456i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_ISFREE: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_LOCKEXCLUDEDCURSOR: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_PARTOFPRIMARYCHAIN: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_SETGAMMA: i32 = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_STEREOSURFACELEFT: i32 = 536870912i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_SW_CKEYDESTBLT: i32 = 131072i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_SW_CKEYDESTOVERLAY: i32 = 65536i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_SW_CKEYSRCBLT: i32 = 524288i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWISURF_SW_CKEYSRCOVERLAY: i32 = 262144i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWIVPORT_COLORKEYANDINTERP: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWIVPORT_NOKERNELHANDLES: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWIVPORT_ON: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWIVPORT_SOFTWARE_AUTOFLIP: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWIVPORT_SOFTWARE_BOB: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWIVPORT_VBION: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWIVPORT_VIDEOON: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_ATTACHEDTODESKTOP: i32 = 16777216i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_BADPDEV: i32 = 1073741824i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_CHANGINGMODE: i32 = 4194304i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_DDRAWDATANOTFETCHED: i32 = 67108864i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_DISPLAYDRV: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_DRIVERINFO2: i32 = 536870912i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_EMULATIONINITIALIZED: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_EXTENDEDALIGNMENT: i32 = 2097152i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_FLIPPEDTOGDI: i32 = 131072i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_FULLSCREEN: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_GDIDRV: i32 = 8388608i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_GETCOLOR: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_HASCKEYDESTOVERLAY: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_HASCKEYSRCOVERLAY: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_HASGDIPALETTE: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_HASGDIPALETTE_EXCLUSIVE: i32 = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_MODECHANGED: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_MODEX: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_MODEXILLEGAL: i32 = 65536i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_NEEDSWIN16FORVRAMLOCK: i32 = 262144i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_NOEMULATION: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_NOHARDWARE: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_PALETTEINIT: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_PDEVICEVRAMBITCLEARED: i32 = 524288i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_SECONDARYDRIVERLOADED: i32 = 134217728i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_SETCOLOR: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_STANDARDVGA: i32 = 1048576i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_TESTINGMODES: i32 = 268435456i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_UMODELOADED: i32 = 33554432i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_VIRTUALDESKTOP: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_VPORTGETCOLOR: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_VPORTSETCOLOR: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_VPORTSTART: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_VPORTSTOP: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_VPORTUPDATE: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_xxxxxxxxx1: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDRAWI_xxxxxxxxx2: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_ADDITIONALPRIMARY: i32 = -2147483648i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_COMMANDBUFFER: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_CUBEMAP: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_CUBEMAP_NEGATIVEX: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_CUBEMAP_NEGATIVEY: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_CUBEMAP_NEGATIVEZ: i32 = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_CUBEMAP_POSITIVEX: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_CUBEMAP_POSITIVEY: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_CUBEMAP_POSITIVEZ: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_D3DTEXTUREMANAGE: i32 = 131072i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_DISCARDBACKBUFFER: i32 = 268435456i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_DONOTPERSIST: i32 = 262144i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_ENABLEALPHACHANNEL: i32 = 536870912i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_EXTENDEDFORMATPRIMARY: i32 = 1073741824i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_HARDWAREDEINTERLACE: i32 = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_HINTANTIALIASING: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_HINTDYNAMIC: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_HINTSTATIC: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_INDEXBUFFER: i32 = 67108864i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_MIPMAPSUBLEVEL: i32 = 65536i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_NOTUSERLOCKABLE: i32 = 4194304i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_NPATCHES: i32 = 33554432i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_OPAQUE: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_POINTS: i32 = 8388608i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_RESERVED1: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_RESERVED2: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_RESERVED3: i32 = 67108864i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_RESERVED4: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_RTPATCHES: i32 = 16777216i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_STEREOSURFACELEFT: i32 = 524288i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_TEXTUREMANAGE: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_VERTEXBUFFER: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS2_VOLUME: i32 = 2097152i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS3_AUTOGENMIPMAP: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS3_CREATESHAREDRESOURCE: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS3_DMAP: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS3_LIGHTWEIGHTMIPMAP: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS3_MULTISAMPLE_MASK: i32 = 31i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS3_MULTISAMPLE_QUALITY_MASK: i32 = 224i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS3_MULTISAMPLE_QUALITY_SHIFT: u32 = 5u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS3_OPENSHAREDRESOURCE: i32 = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS3_READONLYRESOURCE: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS3_RESERVED1: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS3_RESERVED2: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS3_VIDEO: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_3DDEVICE: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_ALLOCONLOAD: i32 = 67108864i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_ALPHA: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_BACKBUFFER: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_COMMANDBUFFER: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_COMPLEX: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_EXECUTEBUFFER: i32 = 8388608i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_FLIP: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_FRONTBUFFER: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_HWCODEC: i32 = 1048576i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_LIVEVIDEO: i32 = 524288i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_LOCALVIDMEM: i32 = 268435456i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_MIPMAP: i32 = 4194304i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_MODEX: i32 = 2097152i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_NONLOCALVIDMEM: i32 = 536870912i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_OFFSCREENPLAIN: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_OPTIMIZED: i32 = -2147483648i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_OVERLAY: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_OWNDC: i32 = 262144i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_PALETTE: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_PRIMARYSURFACE: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_PRIMARYSURFACELEFT: i32 = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_RESERVED1: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_RESERVED2: i32 = 8388608i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_RESERVED3: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_STANDARDVGAMODE: i32 = 1073741824i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_SYSTEMMEMORY: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_TEXTURE: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_VIDEOMEMORY: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_VIDEOPORT: i32 = 134217728i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_VISIBLE: i32 = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_WRITEONLY: i32 = 65536i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCAPS_ZBUFFER: i32 = 131072i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCL_ALLOWMODEX: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCL_ALLOWREBOOT: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCL_CREATEDEVICEWINDOW: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCL_EXCLUSIVE: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCL_FPUPRESERVE: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCL_FPUSETUP: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCL_FULLSCREEN: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCL_MULTITHREADED: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCL_NORMAL: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCL_NOWINDOWCHANGES: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCL_SETDEVICEWINDOW: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSCL_SETFOCUSWINDOW: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSDM_STANDARDVGAMODE: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSD_ALL: i32 = 16775662i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSD_ALPHABITDEPTH: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSD_BACKBUFFERCOUNT: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSD_CAPS: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSD_CKDESTBLT: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSD_CKDESTOVERLAY: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSD_CKSRCBLT: i32 = 65536i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSD_CKSRCOVERLAY: i32 = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSD_DEPTH: i32 = 8388608i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSD_FVF: i32 = 2097152i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSD_HEIGHT: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSD_LINEARSIZE: i32 = 524288i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSD_LPSURFACE: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSD_MIPMAPCOUNT: i32 = 131072i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSD_PITCH: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSD_PIXELFORMAT: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSD_REFRESHRATE: i32 = 262144i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSD_SRCVBHANDLE: i32 = 4194304i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSD_TEXTURESTAGE: i32 = 1048576i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSD_WIDTH: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSD_ZBUFFERBITDEPTH: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSETSURFACEDESC_PRESERVEDC: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSETSURFACEDESC_RECREATEDC: i32 = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSGR_CALIBRATE: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSKIP_ENABLENEXT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSKIP_SKIPNEXT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSMT_ISTESTREQUIRED: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSPD_IUNKNOWNPOINTER: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSPD_VOLATILE: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSVCAPS_RESERVED1: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSVCAPS_RESERVED2: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSVCAPS_RESERVED3: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSVCAPS_RESERVED4: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDSVCAPS_STEREOSEQUENTIAL: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDTRANSFER_CANCEL: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDTRANSFER_HALFLINES: u32 = 256u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDTRANSFER_INVERT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDTRANSFER_NONLOCALVIDMEM: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDTRANSFER_SYSTEMMEMORY: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDUNSUPPORTEDMODE: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVERSIONINFO: u32 = 13u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPBCAPS_DESTINATION: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPBCAPS_SOURCE: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPB_OVERLAY: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPB_TYPE: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPB_VIDEOPORT: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPCAPS_AUTOFLIP: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPCAPS_COLORCONTROL: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPCAPS_HARDWAREDEINTERLACE: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPCAPS_INTERLACED: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPCAPS_NONINTERLACED: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPCAPS_OVERSAMPLEDVBI: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPCAPS_READBACKFIELD: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPCAPS_READBACKLINE: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPCAPS_SHAREABLE: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPCAPS_SKIPEVENFIELDS: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPCAPS_SKIPODDFIELDS: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPCAPS_SYNCMASTER: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPCAPS_SYSTEMMEMORY: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPCAPS_VBIANDVIDEOINDEPENDENT: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPCAPS_VBISURFACE: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPCONNECT_DISCARDSVREFDATA: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPCONNECT_DOUBLECLOCK: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPCONNECT_HALFLINE: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPCONNECT_INTERLACED: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPCONNECT_INVERTPOLARITY: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPCONNECT_SHAREEVEN: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPCONNECT_SHAREODD: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPCONNECT_VACT: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPCREATE_VBIONLY: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPCREATE_VIDEOONLY: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPD_ALIGN: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPD_AUTOFLIP: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPD_CAPS: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPD_FILTERQUALITY: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPD_FX: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPD_HEIGHT: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPD_ID: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPD_PREFERREDAUTOFLIP: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPD_WIDTH: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPFLIP_VBI: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPFLIP_VIDEO: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPFORMAT_VBI: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPFORMAT_VIDEO: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPFX_CROPTOPDATA: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPFX_CROPX: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPFX_CROPY: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPFX_IGNOREVBIXCROP: i32 = 262144i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPFX_INTERLEAVE: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPFX_MIRRORLEFTRIGHT: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPFX_MIRRORUPDOWN: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPFX_PRESHRINKX: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPFX_PRESHRINKXB: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPFX_PRESHRINKXS: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPFX_PRESHRINKY: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPFX_PRESHRINKYB: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPFX_PRESHRINKYS: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPFX_PRESTRETCHX: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPFX_PRESTRETCHXN: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPFX_PRESTRETCHY: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPFX_PRESTRETCHYN: i32 = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPFX_VBICONVERT: i32 = 65536i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPFX_VBINOINTERLEAVE: i32 = 524288i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPFX_VBINOSCALE: i32 = 131072i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPSQ_NOSIGNAL: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPSQ_SIGNALOK: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPSTATUS_VBIONLY: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPSTATUS_VIDEOONLY: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPTARGET_VBI: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPTARGET_VIDEO: i32 = 1i32;
pub const DDVPTYPE_BROOKTREE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 324183392, data2: 55905, data3: 4559, data4: [155, 6, 0, 160, 201, 3, 163, 184] };
pub const DDVPTYPE_CCIR656: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4238550688, data2: 55904, data3: 4559, data4: [155, 6, 0, 160, 201, 3, 163, 184] };
pub const DDVPTYPE_E_HREFH_VREFH: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1425250688, data2: 55904, data3: 4559, data4: [155, 6, 0, 160, 201, 3, 163, 184] };
pub const DDVPTYPE_E_HREFH_VREFL: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2457350688, data2: 55904, data3: 4559, data4: [155, 6, 0, 160, 201, 3, 163, 184] };
pub const DDVPTYPE_E_HREFL_VREFH: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2692350688, data2: 55904, data3: 4559, data4: [155, 6, 0, 160, 201, 3, 163, 184] };
pub const DDVPTYPE_E_HREFL_VREFL: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3768350688, data2: 55904, data3: 4559, data4: [155, 6, 0, 160, 201, 3, 163, 184] };
pub const DDVPTYPE_PHILIPS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 858583392, data2: 55905, data3: 4559, data4: [155, 6, 0, 160, 201, 3, 163, 184] };
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPWAIT_BEGIN: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPWAIT_END: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVPWAIT_LINE: i32 = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVP_AUTOFLIP: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVP_CONVERT: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVP_CROP: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVP_HARDWAREDEINTERLACE: i32 = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVP_IGNOREVBIXCROP: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVP_INTERLEAVE: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVP_MIRRORLEFTRIGHT: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVP_MIRRORUPDOWN: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVP_OVERRIDEBOBWEAVE: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVP_PRESCALE: i32 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVP_SKIPEVENFIELDS: i32 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVP_SKIPODDFIELDS: i32 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVP_SYNCMASTER: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVP_VBICONVERT: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVP_VBINOINTERLEAVE: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDVP_VBINOSCALE: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDWAITVB_BLOCKBEGIN: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDWAITVB_BLOCKBEGINEVENT: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDWAITVB_BLOCKEND: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DDWAITVB_I_TESTVB: i32 = -2147483642i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DD_HAL_VERSION: u32 = 256u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DD_RUNTIME_VERSION: i32 = 2306i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DD_VERSION: i32 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DELETED_LASTONE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DELETED_NOTFOUND: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DELETED_OK: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DIRECTDRAW_VERSION: u32 = 1792u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DXAPI_HALVERSION: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DXERR_GENERIC: u32 = 2147500037u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DXERR_OUTOFCAPS: u32 = 2289434984u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DXERR_UNSUPPORTED: u32 = 2147500033u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const DX_OK: u32 = 0u32;
pub const GUID_ColorControlCallbacks: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4023782594, data2: 18919, data3: 4560, data4: [136, 157, 0, 170, 0, 187, 183, 106] };
pub const GUID_D3DCallbacks: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2079353232, data2: 34708, data3: 4560, data4: [145, 57, 8, 0, 54, 210, 239, 2] };
pub const GUID_D3DCallbacks2: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 195396833, data2: 28854, data3: 4560, data4: [136, 157, 0, 170, 0, 187, 183, 106] };
pub const GUID_D3DCallbacks3: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3723760176, data2: 60426, data3: 4560, data4: [169, 182, 0, 170, 0, 192, 153, 62] };
pub const GUID_D3DCaps: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2079353233, data2: 34708, data3: 4560, data4: [145, 57, 8, 0, 54, 210, 239, 2] };
pub const GUID_D3DExtendedCaps: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2112102272, data2: 40339, data3: 4560, data4: [137, 171, 0, 160, 201, 5, 65, 41] };
pub const GUID_D3DParseUnknownCommandCallback: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 772079520, data2: 39140, data3: 4561, data4: [140, 225, 0, 160, 201, 6, 41, 168] };
pub const GUID_DDMoreCaps: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2282467120, data2: 45104, data3: 4560, data4: [142, 167, 0, 96, 151, 151, 234, 91] };
pub const GUID_DDMoreSurfaceCaps: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 998900838, data2: 62057, data3: 4561, data4: [136, 11, 0, 192, 79, 217, 48, 197] };
pub const GUID_DDStereoMode: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4163376796, data2: 43240, data3: 4562, data4: [161, 242, 0, 160, 201, 131, 234, 246] };
pub const GUID_DxApi: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2323234544, data2: 47381, data3: 4560, data4: [145, 68, 8, 0, 54, 210, 239, 2] };
pub const GUID_GetHeapAlignment: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1121988374, data2: 31553, data3: 4562, data4: [139, 255, 0, 160, 201, 131, 234, 246] };
pub const GUID_KernelCallbacks: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2156279808, data2: 27398, data3: 4560, data4: [155, 6, 0, 160, 201, 3, 163, 184] };
pub const GUID_KernelCaps: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4289361216, data2: 31400, data3: 4560, data4: [155, 6, 0, 160, 201, 3, 163, 184] };
pub const GUID_Miscellaneous2Callbacks: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1080766208, data2: 15962, data3: 4561, data4: [182, 64, 0, 170, 0, 161, 249, 106] };
pub const GUID_MiscellaneousCallbacks: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4023782592, data2: 18919, data3: 4560, data4: [136, 157, 0, 170, 0, 187, 183, 106] };
pub const GUID_MotionCompCallbacks: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2970757952, data2: 23973, data3: 4561, data4: [143, 207, 0, 192, 79, 194, 155, 78] };
pub const GUID_NTCallbacks: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1877601502, data2: 57225, data3: 4561, data4: [157, 176, 0, 96, 8, 39, 113, 186] };
pub const GUID_NTPrivateDriverCaps: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4208028195, data2: 31590, data3: 4562, data4: [131, 215, 0, 192, 79, 124, 229, 140] };
pub const GUID_NonLocalVidMemCaps: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2261056128, data2: 36228, data3: 4560, data4: [148, 232, 0, 192, 79, 195, 65, 55] };
pub const GUID_OptSurfaceKmodeInfo: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3764159602, data2: 20948, data3: 4561, data4: [140, 206, 0, 160, 201, 6, 41, 168] };
pub const GUID_OptSurfaceUmodeInfo: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2641963012, data2: 24488, data3: 4561, data4: [140, 208, 0, 160, 201, 6, 41, 168] };
pub const GUID_UpdateNonLocalHeap: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1121988375, data2: 31553, data3: 4562, data4: [139, 255, 0, 160, 201, 131, 234, 246] };
pub const GUID_UserModeDriverInfo: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4038125794, data2: 24471, data3: 4561, data4: [140, 208, 0, 160, 201, 6, 41, 168] };
pub const GUID_UserModeDriverPassword: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2549637558, data2: 24737, data3: 4561, data4: [140, 208, 0, 160, 201, 6, 41, 168] };
pub const GUID_VPE2Callbacks: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1384653127, data2: 11591, data3: 18074, data4: [160, 209, 3, 69, 88, 144, 246, 200] };
pub const GUID_VideoPortCallbacks: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4023782593, data2: 18919, data3: 4560, data4: [136, 157, 0, 170, 0, 187, 183, 106] };
pub const GUID_VideoPortCaps: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4023782595, data2: 18919, data3: 4560, data4: [136, 157, 0, 170, 0, 187, 183, 106] };
pub const GUID_ZPixelFormats: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2475071616, data2: 14031, data3: 4561, data4: [155, 27, 0, 170, 0, 187, 184, 174] };
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const HEAPALIASINFO_MAPPEDDUMMY: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const HEAPALIASINFO_MAPPEDREAL: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const IRQINFO_HANDLED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const IRQINFO_NOTHANDLED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const MAX_AUTOFLIP_BUFFERS: u32 = 10u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const MAX_DDDEVICEID_STRING: u32 = 512u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const MAX_DRIVER_NAME: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const MAX_PALETTE_SIZE: u32 = 256u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const MDL_64_BIT_VA: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const MDL_ALLOCATED_FIXED_SIZE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const MDL_ALLOCATED_MUST_SUCCEED: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const MDL_IO_PAGE_READ: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const MDL_IO_SPACE: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const MDL_LOCK_HELD: u32 = 512u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const MDL_MAPPED_TO_SYSTEM_VA: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const MDL_MAPPING_CAN_FAIL: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const MDL_NETWORK_HEADER: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const MDL_PAGES_LOCKED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const MDL_PARENT_MAPPED_SYSTEM_VA: u32 = 256u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const MDL_PARTIAL: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const MDL_PARTIAL_HAS_BEEN_MAPPED: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const MDL_SCATTER_GATHER_VA: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const MDL_SOURCE_IS_NONPAGED_POOL: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const MDL_WRITE_OPERATION: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const OBJECT_ISROOT: i32 = -2147483648i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const PFINDEX_UNINITIALIZED: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const REGSTR_KEY_DDHW_DESCRIPTION: &str = "Description";
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const REGSTR_KEY_DDHW_DRIVERNAME: &str = "DriverName";
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const REGSTR_PATH_DDHW: &str = "Hardware\\DirectDrawDrivers";
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const ROP_HAS_PATTERN: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const ROP_HAS_SOURCE: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const SURFACEALIGN_DISCARDABLE: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const VIDMEM_HEAPDISABLED: i32 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const VIDMEM_ISHEAP: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const VIDMEM_ISLINEAR: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const VIDMEM_ISNONLOCAL: i32 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const VIDMEM_ISRECTANGULAR: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const VIDMEM_ISWC: i32 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const VMEMHEAP_ALIGNMENT: i32 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const VMEMHEAP_LINEAR: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const VMEMHEAP_RECTANGULAR: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub const _FACDD: u32 = 2166u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct ACCESSRECTLIST {
    pub lpLink: *mut ACCESSRECTLIST,
    pub rDest: super::super::Foundation::RECT,
    pub lpOwner: *mut DDRAWI_DIRECTDRAW_LCL,
    pub lpSurfaceData: *mut ::core::ffi::c_void,
    pub dwFlags: u32,
    pub lpHeapAliasInfo: *mut HEAPALIASINFO,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for ACCESSRECTLIST {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for ACCESSRECTLIST {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct ATTACHLIST {
    pub dwFlags: u32,
    pub lpLink: *mut ATTACHLIST,
    pub lpAttached: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub lpIAttached: *mut DDRAWI_DDRAWSURFACE_INT,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for ATTACHLIST {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for ATTACHLIST {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DBLNODE {
    pub next: *mut DBLNODE,
    pub prev: *mut DBLNODE,
    pub object: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub object_int: *mut DDRAWI_DDRAWSURFACE_INT,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DBLNODE {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DBLNODE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD32BITDRIVERDATA {
    pub szName: [super::super::Foundation::CHAR; 260],
    pub szEntryPoint: [super::super::Foundation::CHAR; 64],
    pub dwContext: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD32BITDRIVERDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD32BITDRIVERDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDARGB {
    pub blue: u8,
    pub green: u8,
    pub red: u8,
    pub alpha: u8,
}
impl ::core::marker::Copy for DDARGB {}
impl ::core::clone::Clone for DDARGB {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DDBLTBATCH {
    pub lprDest: *mut super::super::Foundation::RECT,
    pub lpDDSSrc: IDirectDrawSurface,
    pub lprSrc: *mut super::super::Foundation::RECT,
    pub dwFlags: u32,
    pub lpDDBltFx: *mut DDBLTFX,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DDBLTBATCH {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DDBLTBATCH {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDBLTFX {
    pub dwSize: u32,
    pub dwDDFX: u32,
    pub dwROP: u32,
    pub dwDDROP: u32,
    pub dwRotationAngle: u32,
    pub dwZBufferOpCode: u32,
    pub dwZBufferLow: u32,
    pub dwZBufferHigh: u32,
    pub dwZBufferBaseDest: u32,
    pub dwZDestConstBitDepth: u32,
    pub Anonymous1: DDBLTFX_0,
    pub dwZSrcConstBitDepth: u32,
    pub Anonymous2: DDBLTFX_1,
    pub dwAlphaEdgeBlendBitDepth: u32,
    pub dwAlphaEdgeBlend: u32,
    pub dwReserved: u32,
    pub dwAlphaDestConstBitDepth: u32,
    pub Anonymous3: DDBLTFX_2,
    pub dwAlphaSrcConstBitDepth: u32,
    pub Anonymous4: DDBLTFX_3,
    pub Anonymous5: DDBLTFX_4,
    pub ddckDestColorkey: DDCOLORKEY,
    pub ddckSrcColorkey: DDCOLORKEY,
}
impl ::core::marker::Copy for DDBLTFX {}
impl ::core::clone::Clone for DDBLTFX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub union DDBLTFX_0 {
    pub dwZDestConst: u32,
    pub lpDDSZBufferDest: IDirectDrawSurface,
}
impl ::core::marker::Copy for DDBLTFX_0 {}
impl ::core::clone::Clone for DDBLTFX_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub union DDBLTFX_1 {
    pub dwZSrcConst: u32,
    pub lpDDSZBufferSrc: IDirectDrawSurface,
}
impl ::core::marker::Copy for DDBLTFX_1 {}
impl ::core::clone::Clone for DDBLTFX_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub union DDBLTFX_2 {
    pub dwAlphaDestConst: u32,
    pub lpDDSAlphaDest: IDirectDrawSurface,
}
impl ::core::marker::Copy for DDBLTFX_2 {}
impl ::core::clone::Clone for DDBLTFX_2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub union DDBLTFX_3 {
    pub dwAlphaSrcConst: u32,
    pub lpDDSAlphaSrc: IDirectDrawSurface,
}
impl ::core::marker::Copy for DDBLTFX_3 {}
impl ::core::clone::Clone for DDBLTFX_3 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub union DDBLTFX_4 {
    pub dwFillColor: u32,
    pub dwFillDepth: u32,
    pub dwFillPixel: u32,
    pub lpDDSPattern: IDirectDrawSurface,
}
impl ::core::marker::Copy for DDBLTFX_4 {}
impl ::core::clone::Clone for DDBLTFX_4 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDBOBNEXTFIELDINFO {
    pub lpSurface: *mut DDSURFACEDATA,
}
impl ::core::marker::Copy for DDBOBNEXTFIELDINFO {}
impl ::core::clone::Clone for DDBOBNEXTFIELDINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDCAPS_DX1 {
    pub dwSize: u32,
    pub dwCaps: u32,
    pub dwCaps2: u32,
    pub dwCKeyCaps: u32,
    pub dwFXCaps: u32,
    pub dwFXAlphaCaps: u32,
    pub dwPalCaps: u32,
    pub dwSVCaps: u32,
    pub dwAlphaBltConstBitDepths: u32,
    pub dwAlphaBltPixelBitDepths: u32,
    pub dwAlphaBltSurfaceBitDepths: u32,
    pub dwAlphaOverlayConstBitDepths: u32,
    pub dwAlphaOverlayPixelBitDepths: u32,
    pub dwAlphaOverlaySurfaceBitDepths: u32,
    pub dwZBufferBitDepths: u32,
    pub dwVidMemTotal: u32,
    pub dwVidMemFree: u32,
    pub dwMaxVisibleOverlays: u32,
    pub dwCurrVisibleOverlays: u32,
    pub dwNumFourCCCodes: u32,
    pub dwAlignBoundarySrc: u32,
    pub dwAlignSizeSrc: u32,
    pub dwAlignBoundaryDest: u32,
    pub dwAlignSizeDest: u32,
    pub dwAlignStrideAlign: u32,
    pub dwRops: [u32; 8],
    pub ddsCaps: DDSCAPS,
    pub dwMinOverlayStretch: u32,
    pub dwMaxOverlayStretch: u32,
    pub dwMinLiveVideoStretch: u32,
    pub dwMaxLiveVideoStretch: u32,
    pub dwMinHwCodecStretch: u32,
    pub dwMaxHwCodecStretch: u32,
    pub dwReserved1: u32,
    pub dwReserved2: u32,
    pub dwReserved3: u32,
}
impl ::core::marker::Copy for DDCAPS_DX1 {}
impl ::core::clone::Clone for DDCAPS_DX1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDCAPS_DX3 {
    pub dwSize: u32,
    pub dwCaps: u32,
    pub dwCaps2: u32,
    pub dwCKeyCaps: u32,
    pub dwFXCaps: u32,
    pub dwFXAlphaCaps: u32,
    pub dwPalCaps: u32,
    pub dwSVCaps: u32,
    pub dwAlphaBltConstBitDepths: u32,
    pub dwAlphaBltPixelBitDepths: u32,
    pub dwAlphaBltSurfaceBitDepths: u32,
    pub dwAlphaOverlayConstBitDepths: u32,
    pub dwAlphaOverlayPixelBitDepths: u32,
    pub dwAlphaOverlaySurfaceBitDepths: u32,
    pub dwZBufferBitDepths: u32,
    pub dwVidMemTotal: u32,
    pub dwVidMemFree: u32,
    pub dwMaxVisibleOverlays: u32,
    pub dwCurrVisibleOverlays: u32,
    pub dwNumFourCCCodes: u32,
    pub dwAlignBoundarySrc: u32,
    pub dwAlignSizeSrc: u32,
    pub dwAlignBoundaryDest: u32,
    pub dwAlignSizeDest: u32,
    pub dwAlignStrideAlign: u32,
    pub dwRops: [u32; 8],
    pub ddsCaps: DDSCAPS,
    pub dwMinOverlayStretch: u32,
    pub dwMaxOverlayStretch: u32,
    pub dwMinLiveVideoStretch: u32,
    pub dwMaxLiveVideoStretch: u32,
    pub dwMinHwCodecStretch: u32,
    pub dwMaxHwCodecStretch: u32,
    pub dwReserved1: u32,
    pub dwReserved2: u32,
    pub dwReserved3: u32,
    pub dwSVBCaps: u32,
    pub dwSVBCKeyCaps: u32,
    pub dwSVBFXCaps: u32,
    pub dwSVBRops: [u32; 8],
    pub dwVSBCaps: u32,
    pub dwVSBCKeyCaps: u32,
    pub dwVSBFXCaps: u32,
    pub dwVSBRops: [u32; 8],
    pub dwSSBCaps: u32,
    pub dwSSBCKeyCaps: u32,
    pub dwSSBFXCaps: u32,
    pub dwSSBRops: [u32; 8],
    pub dwReserved4: u32,
    pub dwReserved5: u32,
    pub dwReserved6: u32,
}
impl ::core::marker::Copy for DDCAPS_DX3 {}
impl ::core::clone::Clone for DDCAPS_DX3 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDCAPS_DX5 {
    pub dwSize: u32,
    pub dwCaps: u32,
    pub dwCaps2: u32,
    pub dwCKeyCaps: u32,
    pub dwFXCaps: u32,
    pub dwFXAlphaCaps: u32,
    pub dwPalCaps: u32,
    pub dwSVCaps: u32,
    pub dwAlphaBltConstBitDepths: u32,
    pub dwAlphaBltPixelBitDepths: u32,
    pub dwAlphaBltSurfaceBitDepths: u32,
    pub dwAlphaOverlayConstBitDepths: u32,
    pub dwAlphaOverlayPixelBitDepths: u32,
    pub dwAlphaOverlaySurfaceBitDepths: u32,
    pub dwZBufferBitDepths: u32,
    pub dwVidMemTotal: u32,
    pub dwVidMemFree: u32,
    pub dwMaxVisibleOverlays: u32,
    pub dwCurrVisibleOverlays: u32,
    pub dwNumFourCCCodes: u32,
    pub dwAlignBoundarySrc: u32,
    pub dwAlignSizeSrc: u32,
    pub dwAlignBoundaryDest: u32,
    pub dwAlignSizeDest: u32,
    pub dwAlignStrideAlign: u32,
    pub dwRops: [u32; 8],
    pub ddsCaps: DDSCAPS,
    pub dwMinOverlayStretch: u32,
    pub dwMaxOverlayStretch: u32,
    pub dwMinLiveVideoStretch: u32,
    pub dwMaxLiveVideoStretch: u32,
    pub dwMinHwCodecStretch: u32,
    pub dwMaxHwCodecStretch: u32,
    pub dwReserved1: u32,
    pub dwReserved2: u32,
    pub dwReserved3: u32,
    pub dwSVBCaps: u32,
    pub dwSVBCKeyCaps: u32,
    pub dwSVBFXCaps: u32,
    pub dwSVBRops: [u32; 8],
    pub dwVSBCaps: u32,
    pub dwVSBCKeyCaps: u32,
    pub dwVSBFXCaps: u32,
    pub dwVSBRops: [u32; 8],
    pub dwSSBCaps: u32,
    pub dwSSBCKeyCaps: u32,
    pub dwSSBFXCaps: u32,
    pub dwSSBRops: [u32; 8],
    pub dwMaxVideoPorts: u32,
    pub dwCurrVideoPorts: u32,
    pub dwSVBCaps2: u32,
    pub dwNLVBCaps: u32,
    pub dwNLVBCaps2: u32,
    pub dwNLVBCKeyCaps: u32,
    pub dwNLVBFXCaps: u32,
    pub dwNLVBRops: [u32; 8],
}
impl ::core::marker::Copy for DDCAPS_DX5 {}
impl ::core::clone::Clone for DDCAPS_DX5 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDCAPS_DX6 {
    pub dwSize: u32,
    pub dwCaps: u32,
    pub dwCaps2: u32,
    pub dwCKeyCaps: u32,
    pub dwFXCaps: u32,
    pub dwFXAlphaCaps: u32,
    pub dwPalCaps: u32,
    pub dwSVCaps: u32,
    pub dwAlphaBltConstBitDepths: u32,
    pub dwAlphaBltPixelBitDepths: u32,
    pub dwAlphaBltSurfaceBitDepths: u32,
    pub dwAlphaOverlayConstBitDepths: u32,
    pub dwAlphaOverlayPixelBitDepths: u32,
    pub dwAlphaOverlaySurfaceBitDepths: u32,
    pub dwZBufferBitDepths: u32,
    pub dwVidMemTotal: u32,
    pub dwVidMemFree: u32,
    pub dwMaxVisibleOverlays: u32,
    pub dwCurrVisibleOverlays: u32,
    pub dwNumFourCCCodes: u32,
    pub dwAlignBoundarySrc: u32,
    pub dwAlignSizeSrc: u32,
    pub dwAlignBoundaryDest: u32,
    pub dwAlignSizeDest: u32,
    pub dwAlignStrideAlign: u32,
    pub dwRops: [u32; 8],
    pub ddsOldCaps: DDSCAPS,
    pub dwMinOverlayStretch: u32,
    pub dwMaxOverlayStretch: u32,
    pub dwMinLiveVideoStretch: u32,
    pub dwMaxLiveVideoStretch: u32,
    pub dwMinHwCodecStretch: u32,
    pub dwMaxHwCodecStretch: u32,
    pub dwReserved1: u32,
    pub dwReserved2: u32,
    pub dwReserved3: u32,
    pub dwSVBCaps: u32,
    pub dwSVBCKeyCaps: u32,
    pub dwSVBFXCaps: u32,
    pub dwSVBRops: [u32; 8],
    pub dwVSBCaps: u32,
    pub dwVSBCKeyCaps: u32,
    pub dwVSBFXCaps: u32,
    pub dwVSBRops: [u32; 8],
    pub dwSSBCaps: u32,
    pub dwSSBCKeyCaps: u32,
    pub dwSSBFXCaps: u32,
    pub dwSSBRops: [u32; 8],
    pub dwMaxVideoPorts: u32,
    pub dwCurrVideoPorts: u32,
    pub dwSVBCaps2: u32,
    pub dwNLVBCaps: u32,
    pub dwNLVBCaps2: u32,
    pub dwNLVBCKeyCaps: u32,
    pub dwNLVBFXCaps: u32,
    pub dwNLVBRops: [u32; 8],
    pub ddsCaps: DDSCAPS2,
}
impl ::core::marker::Copy for DDCAPS_DX6 {}
impl ::core::clone::Clone for DDCAPS_DX6 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDCAPS_DX7 {
    pub dwSize: u32,
    pub dwCaps: u32,
    pub dwCaps2: u32,
    pub dwCKeyCaps: u32,
    pub dwFXCaps: u32,
    pub dwFXAlphaCaps: u32,
    pub dwPalCaps: u32,
    pub dwSVCaps: u32,
    pub dwAlphaBltConstBitDepths: u32,
    pub dwAlphaBltPixelBitDepths: u32,
    pub dwAlphaBltSurfaceBitDepths: u32,
    pub dwAlphaOverlayConstBitDepths: u32,
    pub dwAlphaOverlayPixelBitDepths: u32,
    pub dwAlphaOverlaySurfaceBitDepths: u32,
    pub dwZBufferBitDepths: u32,
    pub dwVidMemTotal: u32,
    pub dwVidMemFree: u32,
    pub dwMaxVisibleOverlays: u32,
    pub dwCurrVisibleOverlays: u32,
    pub dwNumFourCCCodes: u32,
    pub dwAlignBoundarySrc: u32,
    pub dwAlignSizeSrc: u32,
    pub dwAlignBoundaryDest: u32,
    pub dwAlignSizeDest: u32,
    pub dwAlignStrideAlign: u32,
    pub dwRops: [u32; 8],
    pub ddsOldCaps: DDSCAPS,
    pub dwMinOverlayStretch: u32,
    pub dwMaxOverlayStretch: u32,
    pub dwMinLiveVideoStretch: u32,
    pub dwMaxLiveVideoStretch: u32,
    pub dwMinHwCodecStretch: u32,
    pub dwMaxHwCodecStretch: u32,
    pub dwReserved1: u32,
    pub dwReserved2: u32,
    pub dwReserved3: u32,
    pub dwSVBCaps: u32,
    pub dwSVBCKeyCaps: u32,
    pub dwSVBFXCaps: u32,
    pub dwSVBRops: [u32; 8],
    pub dwVSBCaps: u32,
    pub dwVSBCKeyCaps: u32,
    pub dwVSBFXCaps: u32,
    pub dwVSBRops: [u32; 8],
    pub dwSSBCaps: u32,
    pub dwSSBCKeyCaps: u32,
    pub dwSSBFXCaps: u32,
    pub dwSSBRops: [u32; 8],
    pub dwMaxVideoPorts: u32,
    pub dwCurrVideoPorts: u32,
    pub dwSVBCaps2: u32,
    pub dwNLVBCaps: u32,
    pub dwNLVBCaps2: u32,
    pub dwNLVBCKeyCaps: u32,
    pub dwNLVBFXCaps: u32,
    pub dwNLVBRops: [u32; 8],
    pub ddsCaps: DDSCAPS2,
}
impl ::core::marker::Copy for DDCAPS_DX7 {}
impl ::core::clone::Clone for DDCAPS_DX7 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDCOLORCONTROL {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub lBrightness: i32,
    pub lContrast: i32,
    pub lHue: i32,
    pub lSaturation: i32,
    pub lSharpness: i32,
    pub lGamma: i32,
    pub lColorEnable: i32,
    pub dwReserved1: u32,
}
impl ::core::marker::Copy for DDCOLORCONTROL {}
impl ::core::clone::Clone for DDCOLORCONTROL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDCOLORKEY {
    pub dwColorSpaceLowValue: u32,
    pub dwColorSpaceHighValue: u32,
}
impl ::core::marker::Copy for DDCOLORKEY {}
impl ::core::clone::Clone for DDCOLORKEY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDCOMPBUFFERINFO {
    pub dwSize: u32,
    pub dwNumCompBuffers: u32,
    pub dwWidthToCreate: u32,
    pub dwHeightToCreate: u32,
    pub dwBytesToAllocate: u32,
    pub ddCompCaps: DDSCAPS2,
    pub ddPixelFormat: DDPIXELFORMAT,
}
impl ::core::marker::Copy for DDCOMPBUFFERINFO {}
impl ::core::clone::Clone for DDCOMPBUFFERINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDCORECAPS {
    pub dwSize: u32,
    pub dwCaps: u32,
    pub dwCaps2: u32,
    pub dwCKeyCaps: u32,
    pub dwFXCaps: u32,
    pub dwFXAlphaCaps: u32,
    pub dwPalCaps: u32,
    pub dwSVCaps: u32,
    pub dwAlphaBltConstBitDepths: u32,
    pub dwAlphaBltPixelBitDepths: u32,
    pub dwAlphaBltSurfaceBitDepths: u32,
    pub dwAlphaOverlayConstBitDepths: u32,
    pub dwAlphaOverlayPixelBitDepths: u32,
    pub dwAlphaOverlaySurfaceBitDepths: u32,
    pub dwZBufferBitDepths: u32,
    pub dwVidMemTotal: u32,
    pub dwVidMemFree: u32,
    pub dwMaxVisibleOverlays: u32,
    pub dwCurrVisibleOverlays: u32,
    pub dwNumFourCCCodes: u32,
    pub dwAlignBoundarySrc: u32,
    pub dwAlignSizeSrc: u32,
    pub dwAlignBoundaryDest: u32,
    pub dwAlignSizeDest: u32,
    pub dwAlignStrideAlign: u32,
    pub dwRops: [u32; 8],
    pub ddsCaps: DDSCAPS,
    pub dwMinOverlayStretch: u32,
    pub dwMaxOverlayStretch: u32,
    pub dwMinLiveVideoStretch: u32,
    pub dwMaxLiveVideoStretch: u32,
    pub dwMinHwCodecStretch: u32,
    pub dwMaxHwCodecStretch: u32,
    pub dwReserved1: u32,
    pub dwReserved2: u32,
    pub dwReserved3: u32,
    pub dwSVBCaps: u32,
    pub dwSVBCKeyCaps: u32,
    pub dwSVBFXCaps: u32,
    pub dwSVBRops: [u32; 8],
    pub dwVSBCaps: u32,
    pub dwVSBCKeyCaps: u32,
    pub dwVSBFXCaps: u32,
    pub dwVSBRops: [u32; 8],
    pub dwSSBCaps: u32,
    pub dwSSBCKeyCaps: u32,
    pub dwSSBFXCaps: u32,
    pub dwSSBRops: [u32; 8],
    pub dwMaxVideoPorts: u32,
    pub dwCurrVideoPorts: u32,
    pub dwSVBCaps2: u32,
}
impl ::core::marker::Copy for DDCORECAPS {}
impl ::core::clone::Clone for DDCORECAPS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DDDEVICEIDENTIFIER {
    pub szDriver: [super::super::Foundation::CHAR; 512],
    pub szDescription: [super::super::Foundation::CHAR; 512],
    pub liDriverVersion: i64,
    pub dwVendorId: u32,
    pub dwDeviceId: u32,
    pub dwSubSysId: u32,
    pub dwRevision: u32,
    pub guidDeviceIdentifier: ::windows_sys::core::GUID,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DDDEVICEIDENTIFIER {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DDDEVICEIDENTIFIER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DDDEVICEIDENTIFIER2 {
    pub szDriver: [super::super::Foundation::CHAR; 512],
    pub szDescription: [super::super::Foundation::CHAR; 512],
    pub liDriverVersion: i64,
    pub dwVendorId: u32,
    pub dwDeviceId: u32,
    pub dwSubSysId: u32,
    pub dwRevision: u32,
    pub guidDeviceIdentifier: ::windows_sys::core::GUID,
    pub dwWHQLLevel: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DDDEVICEIDENTIFIER2 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DDDEVICEIDENTIFIER2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDENABLEIRQINFO {
    pub dwIRQSources: u32,
    pub dwLine: u32,
    pub IRQCallback: PDX_IRQCALLBACK,
    pub lpIRQData: *mut DX_IRQDATA,
}
impl ::core::marker::Copy for DDENABLEIRQINFO {}
impl ::core::clone::Clone for DDENABLEIRQINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDFLIPOVERLAYINFO {
    pub lpCurrentSurface: *mut DDSURFACEDATA,
    pub lpTargetSurface: *mut DDSURFACEDATA,
    pub dwFlags: u32,
}
impl ::core::marker::Copy for DDFLIPOVERLAYINFO {}
impl ::core::clone::Clone for DDFLIPOVERLAYINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDFLIPVIDEOPORTINFO {
    pub lpVideoPortData: *mut DDVIDEOPORTDATA,
    pub lpCurrentSurface: *mut DDSURFACEDATA,
    pub lpTargetSurface: *mut DDSURFACEDATA,
    pub dwFlipVPFlags: u32,
}
impl ::core::marker::Copy for DDFLIPVIDEOPORTINFO {}
impl ::core::clone::Clone for DDFLIPVIDEOPORTINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDGAMMARAMP {
    pub red: [u16; 256],
    pub green: [u16; 256],
    pub blue: [u16; 256],
}
impl ::core::marker::Copy for DDGAMMARAMP {}
impl ::core::clone::Clone for DDGAMMARAMP {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDGETCURRENTAUTOFLIPININFO {
    pub lpVideoPortData: *mut DDVIDEOPORTDATA,
}
impl ::core::marker::Copy for DDGETCURRENTAUTOFLIPININFO {}
impl ::core::clone::Clone for DDGETCURRENTAUTOFLIPININFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDGETCURRENTAUTOFLIPOUTINFO {
    pub dwSurfaceIndex: u32,
    pub dwVBISurfaceIndex: u32,
}
impl ::core::marker::Copy for DDGETCURRENTAUTOFLIPOUTINFO {}
impl ::core::clone::Clone for DDGETCURRENTAUTOFLIPOUTINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDGETIRQINFO {
    pub dwFlags: u32,
}
impl ::core::marker::Copy for DDGETIRQINFO {}
impl ::core::clone::Clone for DDGETIRQINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDGETPOLARITYININFO {
    pub lpVideoPortData: *mut DDVIDEOPORTDATA,
}
impl ::core::marker::Copy for DDGETPOLARITYININFO {}
impl ::core::clone::Clone for DDGETPOLARITYININFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDGETPOLARITYOUTINFO {
    pub bPolarity: u32,
}
impl ::core::marker::Copy for DDGETPOLARITYOUTINFO {}
impl ::core::clone::Clone for DDGETPOLARITYOUTINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDGETPREVIOUSAUTOFLIPININFO {
    pub lpVideoPortData: *mut DDVIDEOPORTDATA,
}
impl ::core::marker::Copy for DDGETPREVIOUSAUTOFLIPININFO {}
impl ::core::clone::Clone for DDGETPREVIOUSAUTOFLIPININFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDGETPREVIOUSAUTOFLIPOUTINFO {
    pub dwSurfaceIndex: u32,
    pub dwVBISurfaceIndex: u32,
}
impl ::core::marker::Copy for DDGETPREVIOUSAUTOFLIPOUTINFO {}
impl ::core::clone::Clone for DDGETPREVIOUSAUTOFLIPOUTINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDGETTRANSFERSTATUSOUTINFO {
    pub dwTransferID: usize,
}
impl ::core::marker::Copy for DDGETTRANSFERSTATUSOUTINFO {}
impl ::core::clone::Clone for DDGETTRANSFERSTATUSOUTINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHALDDRAWFNS {
    pub dwSize: u32,
    pub lpSetInfo: LPDDHAL_SETINFO,
    pub lpVidMemAlloc: LPDDHAL_VIDMEMALLOC,
    pub lpVidMemFree: LPDDHAL_VIDMEMFREE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHALDDRAWFNS {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHALDDRAWFNS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHALINFO {
    pub dwSize: u32,
    pub lpDDCallbacks: *mut DDHAL_DDCALLBACKS,
    pub lpDDSurfaceCallbacks: *mut DDHAL_DDSURFACECALLBACKS,
    pub lpDDPaletteCallbacks: *mut DDHAL_DDPALETTECALLBACKS,
    pub vmiData: VIDMEMINFO,
    pub ddCaps: DDCORECAPS,
    pub dwMonitorFrequency: u32,
    pub GetDriverInfo: LPDDHAL_GETDRIVERINFO,
    pub dwModeIndex: u32,
    pub lpdwFourCC: *mut u32,
    pub dwNumModes: u32,
    pub lpModeInfo: *mut DDHALMODEINFO,
    pub dwFlags: u32,
    pub lpPDevice: *mut ::core::ffi::c_void,
    pub hInstance: u32,
    pub lpD3DGlobalDriverData: usize,
    pub lpD3DHALCallbacks: usize,
    pub lpDDExeBufCallbacks: *mut DDHAL_DDEXEBUFCALLBACKS,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHALINFO {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHALINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDHALMODEINFO {
    pub dwWidth: u32,
    pub dwHeight: u32,
    pub lPitch: i32,
    pub dwBPP: u32,
    pub wFlags: u16,
    pub wRefreshRate: u16,
    pub dwRBitMask: u32,
    pub dwGBitMask: u32,
    pub dwBBitMask: u32,
    pub dwAlphaBitMask: u32,
}
impl ::core::marker::Copy for DDHALMODEINFO {}
impl ::core::clone::Clone for DDHALMODEINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_ADDATTACHEDSURFACEDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub lpDDSurface: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub lpSurfAttached: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub AddAttachedSurface: LPDDHALSURFCB_ADDATTACHEDSURFACE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_ADDATTACHEDSURFACEDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_ADDATTACHEDSURFACEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_BEGINMOCOMPFRAMEDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_LCL,
    pub lpMoComp: *mut DDRAWI_DDMOTIONCOMP_LCL,
    pub lpDestSurface: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub dwInputDataSize: u32,
    pub lpInputData: *mut ::core::ffi::c_void,
    pub dwOutputDataSize: u32,
    pub lpOutputData: *mut ::core::ffi::c_void,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub BeginMoCompFrame: LPDDHALMOCOMPCB_BEGINFRAME,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_BEGINMOCOMPFRAMEDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_BEGINMOCOMPFRAMEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_BLTDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub lpDDDestSurface: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub rDest: super::super::Foundation::RECTL,
    pub lpDDSrcSurface: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub rSrc: super::super::Foundation::RECTL,
    pub dwFlags: u32,
    pub dwROPFlags: u32,
    pub bltFX: DDBLTFX,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub Blt: LPDDHALSURFCB_BLT,
    pub IsClipped: super::super::Foundation::BOOL,
    pub rOrigDest: super::super::Foundation::RECTL,
    pub rOrigSrc: super::super::Foundation::RECTL,
    pub dwRectCnt: u32,
    pub prDestRects: *mut super::super::Foundation::RECT,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_BLTDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_BLTDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_CALLBACKS {
    pub cbDDCallbacks: DDHAL_DDCALLBACKS,
    pub cbDDSurfaceCallbacks: DDHAL_DDSURFACECALLBACKS,
    pub cbDDPaletteCallbacks: DDHAL_DDPALETTECALLBACKS,
    pub HALDD: DDHAL_DDCALLBACKS,
    pub HALDDSurface: DDHAL_DDSURFACECALLBACKS,
    pub HALDDPalette: DDHAL_DDPALETTECALLBACKS,
    pub HELDD: DDHAL_DDCALLBACKS,
    pub HELDDSurface: DDHAL_DDSURFACECALLBACKS,
    pub HELDDPalette: DDHAL_DDPALETTECALLBACKS,
    pub cbDDExeBufCallbacks: DDHAL_DDEXEBUFCALLBACKS,
    pub HALDDExeBuf: DDHAL_DDEXEBUFCALLBACKS,
    pub HELDDExeBuf: DDHAL_DDEXEBUFCALLBACKS,
    pub cbDDVideoPortCallbacks: DDHAL_DDVIDEOPORTCALLBACKS,
    pub HALDDVideoPort: DDHAL_DDVIDEOPORTCALLBACKS,
    pub cbDDColorControlCallbacks: DDHAL_DDCOLORCONTROLCALLBACKS,
    pub HALDDColorControl: DDHAL_DDCOLORCONTROLCALLBACKS,
    pub cbDDMiscellaneousCallbacks: DDHAL_DDMISCELLANEOUSCALLBACKS,
    pub HALDDMiscellaneous: DDHAL_DDMISCELLANEOUSCALLBACKS,
    pub cbDDKernelCallbacks: DDHAL_DDKERNELCALLBACKS,
    pub HALDDKernel: DDHAL_DDKERNELCALLBACKS,
    pub cbDDMotionCompCallbacks: DDHAL_DDMOTIONCOMPCALLBACKS,
    pub HALDDMotionComp: DDHAL_DDMOTIONCOMPCALLBACKS,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_CALLBACKS {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_CALLBACKS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_CANCREATESURFACEDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub lpDDSurfaceDesc: *mut DDSURFACEDESC,
    pub bIsDifferentPixelFormat: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub CanCreateSurface: LPDDHAL_CANCREATESURFACE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_CANCREATESURFACEDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_CANCREATESURFACEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_CANCREATEVPORTDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_LCL,
    pub lpDDVideoPortDesc: *mut DDVIDEOPORTDESC,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub CanCreateVideoPort: LPDDHALVPORTCB_CANCREATEVIDEOPORT,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_CANCREATEVPORTDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_CANCREATEVPORTDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_COLORCONTROLDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub lpDDSurface: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub lpColorData: *mut DDCOLORCONTROL,
    pub dwFlags: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub ColorControl: LPDDHALCOLORCB_COLORCONTROL,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_COLORCONTROLDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_COLORCONTROLDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_CREATEMOCOMPDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_LCL,
    pub lpMoComp: *mut DDRAWI_DDMOTIONCOMP_LCL,
    pub lpGuid: *mut ::windows_sys::core::GUID,
    pub dwUncompWidth: u32,
    pub dwUncompHeight: u32,
    pub ddUncompPixelFormat: DDPIXELFORMAT,
    pub lpData: *mut ::core::ffi::c_void,
    pub dwDataSize: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub CreateMoComp: LPDDHALMOCOMPCB_CREATE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_CREATEMOCOMPDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_CREATEMOCOMPDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_CREATEPALETTEDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub lpDDPalette: *mut DDRAWI_DDRAWPALETTE_GBL,
    pub lpColorTable: *mut super::Gdi::PALETTEENTRY,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub CreatePalette: LPDDHAL_CREATEPALETTE,
    pub is_excl: super::super::Foundation::BOOL,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_CREATEPALETTEDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_CREATEPALETTEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_CREATESURFACEDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub lpDDSurfaceDesc: *mut DDSURFACEDESC,
    pub lplpSList: *mut *mut DDRAWI_DDRAWSURFACE_LCL,
    pub dwSCnt: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub CreateSurface: LPDDHAL_CREATESURFACE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_CREATESURFACEDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_CREATESURFACEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_CREATESURFACEEXDATA {
    pub dwFlags: u32,
    pub lpDDLcl: *mut DDRAWI_DIRECTDRAW_LCL,
    pub lpDDSLcl: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub ddRVal: ::windows_sys::core::HRESULT,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_CREATESURFACEEXDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_CREATESURFACEEXDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_CREATEVPORTDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_LCL,
    pub lpDDVideoPortDesc: *mut DDVIDEOPORTDESC,
    pub lpVideoPort: *mut DDRAWI_DDVIDEOPORT_LCL,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub CreateVideoPort: LPDDHALVPORTCB_CREATEVIDEOPORT,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_CREATEVPORTDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_CREATEVPORTDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_DDCALLBACKS {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub DestroyDriver: LPDDHAL_DESTROYDRIVER,
    pub CreateSurface: LPDDHAL_CREATESURFACE,
    pub SetColorKey: LPDDHAL_SETCOLORKEY,
    pub SetMode: LPDDHAL_SETMODE,
    pub WaitForVerticalBlank: LPDDHAL_WAITFORVERTICALBLANK,
    pub CanCreateSurface: LPDDHAL_CANCREATESURFACE,
    pub CreatePalette: LPDDHAL_CREATEPALETTE,
    pub GetScanLine: LPDDHAL_GETSCANLINE,
    pub SetExclusiveMode: LPDDHAL_SETEXCLUSIVEMODE,
    pub FlipToGDISurface: LPDDHAL_FLIPTOGDISURFACE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_DDCALLBACKS {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_DDCALLBACKS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_DDCOLORCONTROLCALLBACKS {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub ColorControl: LPDDHALCOLORCB_COLORCONTROL,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_DDCOLORCONTROLCALLBACKS {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_DDCOLORCONTROLCALLBACKS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_DDEXEBUFCALLBACKS {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub CanCreateExecuteBuffer: LPDDHALEXEBUFCB_CANCREATEEXEBUF,
    pub CreateExecuteBuffer: LPDDHALEXEBUFCB_CREATEEXEBUF,
    pub DestroyExecuteBuffer: LPDDHALEXEBUFCB_DESTROYEXEBUF,
    pub LockExecuteBuffer: LPDDHALEXEBUFCB_LOCKEXEBUF,
    pub UnlockExecuteBuffer: LPDDHALEXEBUFCB_UNLOCKEXEBUF,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_DDEXEBUFCALLBACKS {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_DDEXEBUFCALLBACKS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_DDKERNELCALLBACKS {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub SyncSurfaceData: LPDDHALKERNELCB_SYNCSURFACE,
    pub SyncVideoPortData: LPDDHALKERNELCB_SYNCVIDEOPORT,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_DDKERNELCALLBACKS {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_DDKERNELCALLBACKS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_DDMISCELLANEOUS2CALLBACKS {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub Reserved: *mut ::core::ffi::c_void,
    pub CreateSurfaceEx: LPDDHAL_CREATESURFACEEX,
    pub GetDriverState: LPDDHAL_GETDRIVERSTATE,
    pub DestroyDDLocal: LPDDHAL_DESTROYDDLOCAL,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_DDMISCELLANEOUS2CALLBACKS {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_DDMISCELLANEOUS2CALLBACKS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_DDMISCELLANEOUSCALLBACKS {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub GetAvailDriverMemory: LPDDHAL_GETAVAILDRIVERMEMORY,
    pub UpdateNonLocalHeap: LPDDHAL_UPDATENONLOCALHEAP,
    pub GetHeapAlignment: LPDDHAL_GETHEAPALIGNMENT,
    pub GetSysmemBltStatus: LPDDHALSURFCB_GETBLTSTATUS,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_DDMISCELLANEOUSCALLBACKS {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_DDMISCELLANEOUSCALLBACKS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_DDMOTIONCOMPCALLBACKS {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub GetMoCompGuids: LPDDHALMOCOMPCB_GETGUIDS,
    pub GetMoCompFormats: LPDDHALMOCOMPCB_GETFORMATS,
    pub CreateMoComp: LPDDHALMOCOMPCB_CREATE,
    pub GetMoCompBuffInfo: LPDDHALMOCOMPCB_GETCOMPBUFFINFO,
    pub GetInternalMoCompInfo: LPDDHALMOCOMPCB_GETINTERNALINFO,
    pub BeginMoCompFrame: LPDDHALMOCOMPCB_BEGINFRAME,
    pub EndMoCompFrame: LPDDHALMOCOMPCB_ENDFRAME,
    pub RenderMoComp: LPDDHALMOCOMPCB_RENDER,
    pub QueryMoCompStatus: LPDDHALMOCOMPCB_QUERYSTATUS,
    pub DestroyMoComp: LPDDHALMOCOMPCB_DESTROY,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_DDMOTIONCOMPCALLBACKS {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_DDMOTIONCOMPCALLBACKS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_DDPALETTECALLBACKS {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub DestroyPalette: LPDDHALPALCB_DESTROYPALETTE,
    pub SetEntries: LPDDHALPALCB_SETENTRIES,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_DDPALETTECALLBACKS {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_DDPALETTECALLBACKS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_DDSURFACECALLBACKS {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub DestroySurface: LPDDHALSURFCB_DESTROYSURFACE,
    pub Flip: LPDDHALSURFCB_FLIP,
    pub SetClipList: LPDDHALSURFCB_SETCLIPLIST,
    pub Lock: LPDDHALSURFCB_LOCK,
    pub Unlock: LPDDHALSURFCB_UNLOCK,
    pub Blt: LPDDHALSURFCB_BLT,
    pub SetColorKey: LPDDHALSURFCB_SETCOLORKEY,
    pub AddAttachedSurface: LPDDHALSURFCB_ADDATTACHEDSURFACE,
    pub GetBltStatus: LPDDHALSURFCB_GETBLTSTATUS,
    pub GetFlipStatus: LPDDHALSURFCB_GETFLIPSTATUS,
    pub UpdateOverlay: LPDDHALSURFCB_UPDATEOVERLAY,
    pub SetOverlayPosition: LPDDHALSURFCB_SETOVERLAYPOSITION,
    pub reserved4: *mut ::core::ffi::c_void,
    pub SetPalette: LPDDHALSURFCB_SETPALETTE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_DDSURFACECALLBACKS {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_DDSURFACECALLBACKS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_DDVIDEOPORTCALLBACKS {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub CanCreateVideoPort: LPDDHALVPORTCB_CANCREATEVIDEOPORT,
    pub CreateVideoPort: LPDDHALVPORTCB_CREATEVIDEOPORT,
    pub FlipVideoPort: LPDDHALVPORTCB_FLIP,
    pub GetVideoPortBandwidth: LPDDHALVPORTCB_GETBANDWIDTH,
    pub GetVideoPortInputFormats: LPDDHALVPORTCB_GETINPUTFORMATS,
    pub GetVideoPortOutputFormats: LPDDHALVPORTCB_GETOUTPUTFORMATS,
    pub lpReserved1: *mut ::core::ffi::c_void,
    pub GetVideoPortField: LPDDHALVPORTCB_GETFIELD,
    pub GetVideoPortLine: LPDDHALVPORTCB_GETLINE,
    pub GetVideoPortConnectInfo: LPDDHALVPORTCB_GETVPORTCONNECT,
    pub DestroyVideoPort: LPDDHALVPORTCB_DESTROYVPORT,
    pub GetVideoPortFlipStatus: LPDDHALVPORTCB_GETFLIPSTATUS,
    pub UpdateVideoPort: LPDDHALVPORTCB_UPDATE,
    pub WaitForVideoPortSync: LPDDHALVPORTCB_WAITFORSYNC,
    pub GetVideoSignalStatus: LPDDHALVPORTCB_GETSIGNALSTATUS,
    pub ColorControl: LPDDHALVPORTCB_COLORCONTROL,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_DDVIDEOPORTCALLBACKS {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_DDVIDEOPORTCALLBACKS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_DESTROYDDLOCALDATA {
    pub dwFlags: u32,
    pub pDDLcl: *mut DDRAWI_DIRECTDRAW_LCL,
    pub ddRVal: ::windows_sys::core::HRESULT,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_DESTROYDDLOCALDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_DESTROYDDLOCALDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_DESTROYDRIVERDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub DestroyDriver: LPDDHAL_DESTROYDRIVER,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_DESTROYDRIVERDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_DESTROYDRIVERDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_DESTROYMOCOMPDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_LCL,
    pub lpMoComp: *mut DDRAWI_DDMOTIONCOMP_LCL,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub DestroyMoComp: LPDDHALMOCOMPCB_DESTROY,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_DESTROYMOCOMPDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_DESTROYMOCOMPDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_DESTROYPALETTEDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub lpDDPalette: *mut DDRAWI_DDRAWPALETTE_GBL,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub DestroyPalette: LPDDHALPALCB_DESTROYPALETTE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_DESTROYPALETTEDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_DESTROYPALETTEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_DESTROYSURFACEDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub lpDDSurface: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub DestroySurface: LPDDHALSURFCB_DESTROYSURFACE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_DESTROYSURFACEDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_DESTROYSURFACEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_DESTROYVPORTDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_LCL,
    pub lpVideoPort: *mut DDRAWI_DDVIDEOPORT_LCL,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub DestroyVideoPort: LPDDHALVPORTCB_DESTROYVPORT,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_DESTROYVPORTDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_DESTROYVPORTDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_DRVSETCOLORKEYDATA {
    pub lpDDSurface: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub dwFlags: u32,
    pub ckNew: DDCOLORKEY,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub SetColorKey: LPDDHAL_SETCOLORKEY,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_DRVSETCOLORKEYDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_DRVSETCOLORKEYDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_ENDMOCOMPFRAMEDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_LCL,
    pub lpMoComp: *mut DDRAWI_DDMOTIONCOMP_LCL,
    pub lpInputData: *mut ::core::ffi::c_void,
    pub dwInputDataSize: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub EndMoCompFrame: LPDDHALMOCOMPCB_ENDFRAME,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_ENDMOCOMPFRAMEDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_ENDMOCOMPFRAMEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_FLIPDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub lpSurfCurr: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub lpSurfTarg: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub dwFlags: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub Flip: LPDDHALSURFCB_FLIP,
    pub lpSurfCurrLeft: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub lpSurfTargLeft: *mut DDRAWI_DDRAWSURFACE_LCL,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_FLIPDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_FLIPDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_FLIPTOGDISURFACEDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub dwToGDI: u32,
    pub dwReserved: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub FlipToGDISurface: LPDDHAL_FLIPTOGDISURFACE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_FLIPTOGDISURFACEDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_FLIPTOGDISURFACEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_FLIPVPORTDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_LCL,
    pub lpVideoPort: *mut DDRAWI_DDVIDEOPORT_LCL,
    pub lpSurfCurr: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub lpSurfTarg: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub FlipVideoPort: LPDDHALVPORTCB_FLIP,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_FLIPVPORTDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_FLIPVPORTDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_GETAVAILDRIVERMEMORYDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub DDSCaps: DDSCAPS,
    pub dwTotal: u32,
    pub dwFree: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetAvailDriverMemory: LPDDHAL_GETAVAILDRIVERMEMORY,
    pub ddsCapsEx: DDSCAPSEX,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_GETAVAILDRIVERMEMORYDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_GETAVAILDRIVERMEMORYDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_GETBLTSTATUSDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub lpDDSurface: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub dwFlags: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetBltStatus: LPDDHALSURFCB_GETBLTSTATUS,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_GETBLTSTATUSDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_GETBLTSTATUSDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDHAL_GETDRIVERINFODATA {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub guidInfo: ::windows_sys::core::GUID,
    pub dwExpectedSize: u32,
    pub lpvData: *mut ::core::ffi::c_void,
    pub dwActualSize: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub dwContext: usize,
}
impl ::core::marker::Copy for DDHAL_GETDRIVERINFODATA {}
impl ::core::clone::Clone for DDHAL_GETDRIVERINFODATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDHAL_GETDRIVERSTATEDATA {
    pub dwFlags: u32,
    pub Anonymous: DDHAL_GETDRIVERSTATEDATA_0,
    pub lpdwStates: *mut u32,
    pub dwLength: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
}
impl ::core::marker::Copy for DDHAL_GETDRIVERSTATEDATA {}
impl ::core::clone::Clone for DDHAL_GETDRIVERSTATEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub union DDHAL_GETDRIVERSTATEDATA_0 {
    pub dwhContext: usize,
}
impl ::core::marker::Copy for DDHAL_GETDRIVERSTATEDATA_0 {}
impl ::core::clone::Clone for DDHAL_GETDRIVERSTATEDATA_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_GETFLIPSTATUSDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub lpDDSurface: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub dwFlags: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetFlipStatus: LPDDHALSURFCB_GETFLIPSTATUS,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_GETFLIPSTATUSDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_GETFLIPSTATUSDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDHAL_GETHEAPALIGNMENTDATA {
    pub dwInstance: usize,
    pub dwHeap: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetHeapAlignment: LPDDHAL_GETHEAPALIGNMENT,
    pub Alignment: HEAPALIGNMENT,
}
impl ::core::marker::Copy for DDHAL_GETHEAPALIGNMENTDATA {}
impl ::core::clone::Clone for DDHAL_GETHEAPALIGNMENTDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_GETINTERNALMOCOMPDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_LCL,
    pub lpGuid: *mut ::windows_sys::core::GUID,
    pub dwWidth: u32,
    pub dwHeight: u32,
    pub ddPixelFormat: DDPIXELFORMAT,
    pub dwScratchMemAlloc: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetInternalMoCompInfo: LPDDHALMOCOMPCB_GETINTERNALINFO,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_GETINTERNALMOCOMPDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_GETINTERNALMOCOMPDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_GETMOCOMPCOMPBUFFDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_LCL,
    pub lpGuid: *mut ::windows_sys::core::GUID,
    pub dwWidth: u32,
    pub dwHeight: u32,
    pub ddPixelFormat: DDPIXELFORMAT,
    pub dwNumTypesCompBuffs: u32,
    pub lpCompBuffInfo: *mut DDMCCOMPBUFFERINFO,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetMoCompBuffInfo: LPDDHALMOCOMPCB_GETCOMPBUFFINFO,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_GETMOCOMPCOMPBUFFDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_GETMOCOMPCOMPBUFFDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_GETMOCOMPFORMATSDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_LCL,
    pub lpGuid: *mut ::windows_sys::core::GUID,
    pub dwNumFormats: u32,
    pub lpFormats: *mut DDPIXELFORMAT,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetMoCompFormats: LPDDHALMOCOMPCB_GETFORMATS,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_GETMOCOMPFORMATSDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_GETMOCOMPFORMATSDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_GETMOCOMPGUIDSDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_LCL,
    pub dwNumGuids: u32,
    pub lpGuids: *mut ::windows_sys::core::GUID,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetMoCompGuids: LPDDHALMOCOMPCB_GETGUIDS,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_GETMOCOMPGUIDSDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_GETMOCOMPGUIDSDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_GETSCANLINEDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub dwScanLine: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetScanLine: LPDDHAL_GETSCANLINE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_GETSCANLINEDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_GETSCANLINEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_GETVPORTBANDWIDTHDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_LCL,
    pub lpVideoPort: *mut DDRAWI_DDVIDEOPORT_LCL,
    pub lpddpfFormat: *mut DDPIXELFORMAT,
    pub dwWidth: u32,
    pub dwHeight: u32,
    pub dwFlags: u32,
    pub lpBandwidth: *mut DDVIDEOPORTBANDWIDTH,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetVideoPortBandwidth: LPDDHALVPORTCB_GETBANDWIDTH,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_GETVPORTBANDWIDTHDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_GETVPORTBANDWIDTHDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_GETVPORTCONNECTDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_LCL,
    pub dwPortId: u32,
    pub lpConnect: *mut DDVIDEOPORTCONNECT,
    pub dwNumEntries: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetVideoPortConnectInfo: LPDDHALVPORTCB_GETVPORTCONNECT,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_GETVPORTCONNECTDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_GETVPORTCONNECTDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_GETVPORTFIELDDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_LCL,
    pub lpVideoPort: *mut DDRAWI_DDVIDEOPORT_LCL,
    pub bField: super::super::Foundation::BOOL,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetVideoPortField: LPDDHALVPORTCB_GETFIELD,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_GETVPORTFIELDDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_GETVPORTFIELDDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_GETVPORTFLIPSTATUSDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_LCL,
    pub fpSurface: usize,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetVideoPortFlipStatus: LPDDHALVPORTCB_GETFLIPSTATUS,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_GETVPORTFLIPSTATUSDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_GETVPORTFLIPSTATUSDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_GETVPORTINPUTFORMATDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_LCL,
    pub lpVideoPort: *mut DDRAWI_DDVIDEOPORT_LCL,
    pub dwFlags: u32,
    pub lpddpfFormat: *mut DDPIXELFORMAT,
    pub dwNumFormats: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetVideoPortInputFormats: LPDDHALVPORTCB_GETINPUTFORMATS,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_GETVPORTINPUTFORMATDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_GETVPORTINPUTFORMATDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_GETVPORTLINEDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_LCL,
    pub lpVideoPort: *mut DDRAWI_DDVIDEOPORT_LCL,
    pub dwLine: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetVideoPortLine: LPDDHALVPORTCB_GETLINE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_GETVPORTLINEDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_GETVPORTLINEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_GETVPORTOUTPUTFORMATDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_LCL,
    pub lpVideoPort: *mut DDRAWI_DDVIDEOPORT_LCL,
    pub dwFlags: u32,
    pub lpddpfInputFormat: *mut DDPIXELFORMAT,
    pub lpddpfOutputFormats: *mut DDPIXELFORMAT,
    pub dwNumFormats: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetVideoPortOutputFormats: LPDDHALVPORTCB_GETOUTPUTFORMATS,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_GETVPORTOUTPUTFORMATDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_GETVPORTOUTPUTFORMATDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_GETVPORTSIGNALDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_LCL,
    pub lpVideoPort: *mut DDRAWI_DDVIDEOPORT_LCL,
    pub dwStatus: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetVideoSignalStatus: LPDDHALVPORTCB_GETSIGNALSTATUS,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_GETVPORTSIGNALDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_GETVPORTSIGNALDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_LOCKDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub lpDDSurface: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub bHasRect: u32,
    pub rArea: super::super::Foundation::RECTL,
    pub lpSurfData: *mut ::core::ffi::c_void,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub Lock: LPDDHALSURFCB_LOCK,
    pub dwFlags: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_LOCKDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_LOCKDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_QUERYMOCOMPSTATUSDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_LCL,
    pub lpMoComp: *mut DDRAWI_DDMOTIONCOMP_LCL,
    pub lpSurface: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub dwFlags: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub QueryMoCompStatus: LPDDHALMOCOMPCB_QUERYSTATUS,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_QUERYMOCOMPSTATUSDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_QUERYMOCOMPSTATUSDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_RENDERMOCOMPDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_LCL,
    pub lpMoComp: *mut DDRAWI_DDMOTIONCOMP_LCL,
    pub dwNumBuffers: u32,
    pub lpBufferInfo: *mut DDMCBUFFERINFO,
    pub dwFunction: u32,
    pub lpInputData: *mut ::core::ffi::c_void,
    pub dwInputDataSize: u32,
    pub lpOutputData: *mut ::core::ffi::c_void,
    pub dwOutputDataSize: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub RenderMoComp: LPDDHALMOCOMPCB_RENDER,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_RENDERMOCOMPDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_RENDERMOCOMPDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_SETCLIPLISTDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub lpDDSurface: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub SetClipList: LPDDHALSURFCB_SETCLIPLIST,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_SETCLIPLISTDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_SETCLIPLISTDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_SETCOLORKEYDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub lpDDSurface: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub dwFlags: u32,
    pub ckNew: DDCOLORKEY,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub SetColorKey: LPDDHALSURFCB_SETCOLORKEY,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_SETCOLORKEYDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_SETCOLORKEYDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_SETENTRIESDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub lpDDPalette: *mut DDRAWI_DDRAWPALETTE_GBL,
    pub dwBase: u32,
    pub dwNumEntries: u32,
    pub lpEntries: *mut super::Gdi::PALETTEENTRY,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub SetEntries: LPDDHALPALCB_SETENTRIES,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_SETENTRIESDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_SETENTRIESDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_SETEXCLUSIVEMODEDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub dwEnterExcl: u32,
    pub dwReserved: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub SetExclusiveMode: LPDDHAL_SETEXCLUSIVEMODE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_SETEXCLUSIVEMODEDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_SETEXCLUSIVEMODEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_SETMODEDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub dwModeIndex: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub SetMode: LPDDHAL_SETMODE,
    pub inexcl: super::super::Foundation::BOOL,
    pub useRefreshRate: super::super::Foundation::BOOL,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_SETMODEDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_SETMODEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_SETOVERLAYPOSITIONDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub lpDDSrcSurface: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub lpDDDestSurface: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub lXPos: i32,
    pub lYPos: i32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub SetOverlayPosition: LPDDHALSURFCB_SETOVERLAYPOSITION,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_SETOVERLAYPOSITIONDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_SETOVERLAYPOSITIONDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_SETPALETTEDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub lpDDSurface: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub lpDDPalette: *mut DDRAWI_DDRAWPALETTE_GBL,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub SetPalette: LPDDHALSURFCB_SETPALETTE,
    pub Attach: super::super::Foundation::BOOL,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_SETPALETTEDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_SETPALETTEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_SYNCSURFACEDATA {
    pub dwSize: u32,
    pub lpDD: *mut DDRAWI_DIRECTDRAW_LCL,
    pub lpDDSurface: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub dwSurfaceOffset: u32,
    pub fpLockPtr: usize,
    pub lPitch: i32,
    pub dwOverlayOffset: u32,
    pub dwOverlaySrcWidth: u32,
    pub dwOverlaySrcHeight: u32,
    pub dwOverlayDestWidth: u32,
    pub dwOverlayDestHeight: u32,
    pub dwDriverReserved1: usize,
    pub dwDriverReserved2: usize,
    pub dwDriverReserved3: usize,
    pub ddRVal: ::windows_sys::core::HRESULT,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_SYNCSURFACEDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_SYNCSURFACEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_SYNCVIDEOPORTDATA {
    pub dwSize: u32,
    pub lpDD: *mut DDRAWI_DIRECTDRAW_LCL,
    pub lpVideoPort: *mut DDRAWI_DDVIDEOPORT_LCL,
    pub dwOriginOffset: u32,
    pub dwHeight: u32,
    pub dwVBIHeight: u32,
    pub dwDriverReserved1: usize,
    pub dwDriverReserved2: usize,
    pub dwDriverReserved3: usize,
    pub ddRVal: ::windows_sys::core::HRESULT,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_SYNCVIDEOPORTDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_SYNCVIDEOPORTDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_UNLOCKDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub lpDDSurface: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub Unlock: LPDDHALSURFCB_UNLOCK,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_UNLOCKDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_UNLOCKDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_UPDATENONLOCALHEAPDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub dwHeap: u32,
    pub fpGARTLin: usize,
    pub fpGARTDev: usize,
    pub ulPolicyMaxBytes: usize,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub UpdateNonLocalHeap: LPDDHAL_UPDATENONLOCALHEAP,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_UPDATENONLOCALHEAPDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_UPDATENONLOCALHEAPDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_UPDATEOVERLAYDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub lpDDDestSurface: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub rDest: super::super::Foundation::RECTL,
    pub lpDDSrcSurface: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub rSrc: super::super::Foundation::RECTL,
    pub dwFlags: u32,
    pub overlayFX: DDOVERLAYFX,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub UpdateOverlay: LPDDHALSURFCB_UPDATEOVERLAY,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_UPDATEOVERLAYDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_UPDATEOVERLAYDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_UPDATEVPORTDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_LCL,
    pub lpVideoPort: *mut DDRAWI_DDVIDEOPORT_LCL,
    pub lplpDDSurface: *mut *mut DDRAWI_DDRAWSURFACE_INT,
    pub lplpDDVBISurface: *mut *mut DDRAWI_DDRAWSURFACE_INT,
    pub lpVideoInfo: *mut DDVIDEOPORTINFO,
    pub dwFlags: u32,
    pub dwNumAutoflip: u32,
    pub dwNumVBIAutoflip: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub UpdateVideoPort: LPDDHALVPORTCB_UPDATE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_UPDATEVPORTDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_UPDATEVPORTDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_VPORTCOLORDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_LCL,
    pub lpVideoPort: *mut DDRAWI_DDVIDEOPORT_LCL,
    pub dwFlags: u32,
    pub lpColorData: *mut DDCOLORCONTROL,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub ColorControl: LPDDHALVPORTCB_COLORCONTROL,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_VPORTCOLORDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_VPORTCOLORDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_WAITFORVERTICALBLANKDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub dwFlags: u32,
    pub bIsInVB: u32,
    pub hEvent: usize,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub WaitForVerticalBlank: LPDDHAL_WAITFORVERTICALBLANK,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_WAITFORVERTICALBLANKDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_WAITFORVERTICALBLANKDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDHAL_WAITFORVPORTSYNCDATA {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_LCL,
    pub lpVideoPort: *mut DDRAWI_DDVIDEOPORT_LCL,
    pub dwFlags: u32,
    pub dwLine: u32,
    pub dwTimeOut: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub WaitForVideoPortSync: LPDDHALVPORTCB_WAITFORSYNC,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDHAL_WAITFORVPORTSYNCDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDHAL_WAITFORVPORTSYNCDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDKERNELCAPS {
    pub dwSize: u32,
    pub dwCaps: u32,
    pub dwIRQCaps: u32,
}
impl ::core::marker::Copy for DDKERNELCAPS {}
impl ::core::clone::Clone for DDKERNELCAPS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDLOCKININFO {
    pub lpSurfaceData: *mut DDSURFACEDATA,
}
impl ::core::marker::Copy for DDLOCKININFO {}
impl ::core::clone::Clone for DDLOCKININFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDLOCKOUTINFO {
    pub dwSurfacePtr: usize,
}
impl ::core::marker::Copy for DDLOCKOUTINFO {}
impl ::core::clone::Clone for DDLOCKOUTINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDMCBUFFERINFO {
    pub dwSize: u32,
    pub lpCompSurface: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub dwDataOffset: u32,
    pub dwDataSize: u32,
    pub lpPrivate: *mut ::core::ffi::c_void,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDMCBUFFERINFO {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDMCBUFFERINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDMCCOMPBUFFERINFO {
    pub dwSize: u32,
    pub dwNumCompBuffers: u32,
    pub dwWidthToCreate: u32,
    pub dwHeightToCreate: u32,
    pub dwBytesToAllocate: u32,
    pub ddCompCaps: DDSCAPS2,
    pub ddPixelFormat: DDPIXELFORMAT,
}
impl ::core::marker::Copy for DDMCCOMPBUFFERINFO {}
impl ::core::clone::Clone for DDMCCOMPBUFFERINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DDMOCOMPBUFFERINFO {
    pub dwSize: u32,
    pub lpCompSurface: *mut DD_SURFACE_LOCAL,
    pub dwDataOffset: u32,
    pub dwDataSize: u32,
    pub lpPrivate: *mut ::core::ffi::c_void,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DDMOCOMPBUFFERINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DDMOCOMPBUFFERINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDMONITORINFO {
    pub Manufacturer: u16,
    pub Product: u16,
    pub SerialNumber: u32,
    pub DeviceIdentifier: ::windows_sys::core::GUID,
    pub Mode640x480: i32,
    pub Mode800x600: i32,
    pub Mode1024x768: i32,
    pub Mode1280x1024: i32,
    pub Mode1600x1200: i32,
    pub ModeReserved1: i32,
    pub ModeReserved2: i32,
    pub ModeReserved3: i32,
}
impl ::core::marker::Copy for DDMONITORINFO {}
impl ::core::clone::Clone for DDMONITORINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDMORESURFACECAPS {
    pub dwSize: u32,
    pub ddsCapsMore: DDSCAPSEX,
    pub ddsExtendedHeapRestrictions: [DDMORESURFACECAPS_0; 1],
}
impl ::core::marker::Copy for DDMORESURFACECAPS {}
impl ::core::clone::Clone for DDMORESURFACECAPS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDMORESURFACECAPS_0 {
    pub ddsCapsEx: DDSCAPSEX,
    pub ddsCapsExAlt: DDSCAPSEX,
}
impl ::core::marker::Copy for DDMORESURFACECAPS_0 {}
impl ::core::clone::Clone for DDMORESURFACECAPS_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDNONLOCALVIDMEMCAPS {
    pub dwSize: u32,
    pub dwNLVBCaps: u32,
    pub dwNLVBCaps2: u32,
    pub dwNLVBCKeyCaps: u32,
    pub dwNLVBFXCaps: u32,
    pub dwNLVBRops: [u32; 8],
}
impl ::core::marker::Copy for DDNONLOCALVIDMEMCAPS {}
impl ::core::clone::Clone for DDNONLOCALVIDMEMCAPS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDNTCORECAPS {
    pub dwSize: u32,
    pub dwCaps: u32,
    pub dwCaps2: u32,
    pub dwCKeyCaps: u32,
    pub dwFXCaps: u32,
    pub dwFXAlphaCaps: u32,
    pub dwPalCaps: u32,
    pub dwSVCaps: u32,
    pub dwAlphaBltConstBitDepths: u32,
    pub dwAlphaBltPixelBitDepths: u32,
    pub dwAlphaBltSurfaceBitDepths: u32,
    pub dwAlphaOverlayConstBitDepths: u32,
    pub dwAlphaOverlayPixelBitDepths: u32,
    pub dwAlphaOverlaySurfaceBitDepths: u32,
    pub dwZBufferBitDepths: u32,
    pub dwVidMemTotal: u32,
    pub dwVidMemFree: u32,
    pub dwMaxVisibleOverlays: u32,
    pub dwCurrVisibleOverlays: u32,
    pub dwNumFourCCCodes: u32,
    pub dwAlignBoundarySrc: u32,
    pub dwAlignSizeSrc: u32,
    pub dwAlignBoundaryDest: u32,
    pub dwAlignSizeDest: u32,
    pub dwAlignStrideAlign: u32,
    pub dwRops: [u32; 8],
    pub ddsCaps: DDSCAPS,
    pub dwMinOverlayStretch: u32,
    pub dwMaxOverlayStretch: u32,
    pub dwMinLiveVideoStretch: u32,
    pub dwMaxLiveVideoStretch: u32,
    pub dwMinHwCodecStretch: u32,
    pub dwMaxHwCodecStretch: u32,
    pub dwReserved1: u32,
    pub dwReserved2: u32,
    pub dwReserved3: u32,
    pub dwSVBCaps: u32,
    pub dwSVBCKeyCaps: u32,
    pub dwSVBFXCaps: u32,
    pub dwSVBRops: [u32; 8],
    pub dwVSBCaps: u32,
    pub dwVSBCKeyCaps: u32,
    pub dwVSBFXCaps: u32,
    pub dwVSBRops: [u32; 8],
    pub dwSSBCaps: u32,
    pub dwSSBCKeyCaps: u32,
    pub dwSSBFXCaps: u32,
    pub dwSSBRops: [u32; 8],
    pub dwMaxVideoPorts: u32,
    pub dwCurrVideoPorts: u32,
    pub dwSVBCaps2: u32,
}
impl ::core::marker::Copy for DDNTCORECAPS {}
impl ::core::clone::Clone for DDNTCORECAPS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDOPTSURFACEDESC {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub ddSCaps: DDSCAPS2,
    pub ddOSCaps: DDOSCAPS,
    pub guid: ::windows_sys::core::GUID,
    pub dwCompressionRatio: u32,
}
impl ::core::marker::Copy for DDOPTSURFACEDESC {}
impl ::core::clone::Clone for DDOPTSURFACEDESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDOSCAPS {
    pub dwCaps: u32,
}
impl ::core::marker::Copy for DDOSCAPS {}
impl ::core::clone::Clone for DDOSCAPS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDOVERLAYFX {
    pub dwSize: u32,
    pub dwAlphaEdgeBlendBitDepth: u32,
    pub dwAlphaEdgeBlend: u32,
    pub dwReserved: u32,
    pub dwAlphaDestConstBitDepth: u32,
    pub Anonymous1: DDOVERLAYFX_0,
    pub dwAlphaSrcConstBitDepth: u32,
    pub Anonymous2: DDOVERLAYFX_1,
    pub dckDestColorkey: DDCOLORKEY,
    pub dckSrcColorkey: DDCOLORKEY,
    pub dwDDFX: u32,
    pub dwFlags: u32,
}
impl ::core::marker::Copy for DDOVERLAYFX {}
impl ::core::clone::Clone for DDOVERLAYFX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub union DDOVERLAYFX_0 {
    pub dwAlphaDestConst: u32,
    pub lpDDSAlphaDest: IDirectDrawSurface,
}
impl ::core::marker::Copy for DDOVERLAYFX_0 {}
impl ::core::clone::Clone for DDOVERLAYFX_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub union DDOVERLAYFX_1 {
    pub dwAlphaSrcConst: u32,
    pub lpDDSAlphaSrc: IDirectDrawSurface,
}
impl ::core::marker::Copy for DDOVERLAYFX_1 {}
impl ::core::clone::Clone for DDOVERLAYFX_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDPIXELFORMAT {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub dwFourCC: u32,
    pub Anonymous1: DDPIXELFORMAT_0,
    pub Anonymous2: DDPIXELFORMAT_1,
    pub Anonymous3: DDPIXELFORMAT_2,
    pub Anonymous4: DDPIXELFORMAT_3,
    pub Anonymous5: DDPIXELFORMAT_4,
}
impl ::core::marker::Copy for DDPIXELFORMAT {}
impl ::core::clone::Clone for DDPIXELFORMAT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub union DDPIXELFORMAT_0 {
    pub dwRGBBitCount: u32,
    pub dwYUVBitCount: u32,
    pub dwZBufferBitDepth: u32,
    pub dwAlphaBitDepth: u32,
    pub dwLuminanceBitCount: u32,
    pub dwBumpBitCount: u32,
    pub dwPrivateFormatBitCount: u32,
}
impl ::core::marker::Copy for DDPIXELFORMAT_0 {}
impl ::core::clone::Clone for DDPIXELFORMAT_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub union DDPIXELFORMAT_1 {
    pub dwRBitMask: u32,
    pub dwYBitMask: u32,
    pub dwStencilBitDepth: u32,
    pub dwLuminanceBitMask: u32,
    pub dwBumpDuBitMask: u32,
    pub dwOperations: u32,
}
impl ::core::marker::Copy for DDPIXELFORMAT_1 {}
impl ::core::clone::Clone for DDPIXELFORMAT_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub union DDPIXELFORMAT_2 {
    pub dwGBitMask: u32,
    pub dwUBitMask: u32,
    pub dwZBitMask: u32,
    pub dwBumpDvBitMask: u32,
    pub MultiSampleCaps: DDPIXELFORMAT_2_0,
}
impl ::core::marker::Copy for DDPIXELFORMAT_2 {}
impl ::core::clone::Clone for DDPIXELFORMAT_2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDPIXELFORMAT_2_0 {
    pub wFlipMSTypes: u16,
    pub wBltMSTypes: u16,
}
impl ::core::marker::Copy for DDPIXELFORMAT_2_0 {}
impl ::core::clone::Clone for DDPIXELFORMAT_2_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub union DDPIXELFORMAT_3 {
    pub dwBBitMask: u32,
    pub dwVBitMask: u32,
    pub dwStencilBitMask: u32,
    pub dwBumpLuminanceBitMask: u32,
}
impl ::core::marker::Copy for DDPIXELFORMAT_3 {}
impl ::core::clone::Clone for DDPIXELFORMAT_3 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub union DDPIXELFORMAT_4 {
    pub dwRGBAlphaBitMask: u32,
    pub dwYUVAlphaBitMask: u32,
    pub dwLuminanceAlphaBitMask: u32,
    pub dwRGBZBitMask: u32,
    pub dwYUVZBitMask: u32,
}
impl ::core::marker::Copy for DDPIXELFORMAT_4 {}
impl ::core::clone::Clone for DDPIXELFORMAT_4 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDRAWI_DDMOTIONCOMP_INT {
    pub lpVtbl: *mut ::core::ffi::c_void,
    pub lpLcl: *mut DDRAWI_DDMOTIONCOMP_LCL,
    pub lpLink: *mut DDRAWI_DDMOTIONCOMP_INT,
    pub dwIntRefCnt: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDRAWI_DDMOTIONCOMP_INT {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDRAWI_DDMOTIONCOMP_INT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDRAWI_DDMOTIONCOMP_LCL {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_LCL,
    pub guid: ::windows_sys::core::GUID,
    pub dwUncompWidth: u32,
    pub dwUncompHeight: u32,
    pub ddUncompPixelFormat: DDPIXELFORMAT,
    pub dwInternalFlags: u32,
    pub dwRefCnt: u32,
    pub dwProcessId: u32,
    pub hMoComp: super::super::Foundation::HANDLE,
    pub dwDriverReserved1: u32,
    pub dwDriverReserved2: u32,
    pub dwDriverReserved3: u32,
    pub lpDriverReserved1: *mut ::core::ffi::c_void,
    pub lpDriverReserved2: *mut ::core::ffi::c_void,
    pub lpDriverReserved3: *mut ::core::ffi::c_void,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDRAWI_DDMOTIONCOMP_LCL {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDRAWI_DDMOTIONCOMP_LCL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDRAWI_DDRAWCLIPPER_GBL {
    pub dwRefCnt: u32,
    pub dwFlags: u32,
    pub lpDD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub dwProcessId: u32,
    pub dwReserved1: usize,
    pub hWnd: usize,
    pub lpStaticClipList: *mut super::Gdi::RGNDATA,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDRAWI_DDRAWCLIPPER_GBL {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDRAWI_DDRAWCLIPPER_GBL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDRAWI_DDRAWCLIPPER_INT {
    pub lpVtbl: *mut ::core::ffi::c_void,
    pub lpLcl: *mut DDRAWI_DDRAWCLIPPER_LCL,
    pub lpLink: *mut DDRAWI_DDRAWCLIPPER_INT,
    pub dwIntRefCnt: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDRAWI_DDRAWCLIPPER_INT {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDRAWI_DDRAWCLIPPER_INT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDRAWI_DDRAWCLIPPER_LCL {
    pub lpClipMore: u32,
    pub lpGbl: *mut DDRAWI_DDRAWCLIPPER_GBL,
    pub lpDD_lcl: *mut DDRAWI_DIRECTDRAW_LCL,
    pub dwLocalRefCnt: u32,
    pub pUnkOuter: ::windows_sys::core::IUnknown,
    pub lpDD_int: *mut DDRAWI_DIRECTDRAW_INT,
    pub dwReserved1: usize,
    pub pAddrefedThisOwner: ::windows_sys::core::IUnknown,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDRAWI_DDRAWCLIPPER_LCL {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDRAWI_DDRAWCLIPPER_LCL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDRAWI_DDRAWPALETTE_GBL {
    pub dwRefCnt: u32,
    pub dwFlags: u32,
    pub lpDD_lcl: *mut DDRAWI_DIRECTDRAW_LCL,
    pub dwProcessId: u32,
    pub lpColorTable: *mut super::Gdi::PALETTEENTRY,
    pub Anonymous: DDRAWI_DDRAWPALETTE_GBL_0,
    pub dwDriverReserved: u32,
    pub dwContentsStamp: u32,
    pub dwSaveStamp: u32,
    pub dwHandle: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDRAWI_DDRAWPALETTE_GBL {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDRAWI_DDRAWPALETTE_GBL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub union DDRAWI_DDRAWPALETTE_GBL_0 {
    pub dwReserved1: usize,
    pub hHELGDIPalette: super::Gdi::HPALETTE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDRAWI_DDRAWPALETTE_GBL_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDRAWI_DDRAWPALETTE_GBL_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDRAWI_DDRAWPALETTE_INT {
    pub lpVtbl: *mut ::core::ffi::c_void,
    pub lpLcl: *mut DDRAWI_DDRAWPALETTE_LCL,
    pub lpLink: *mut DDRAWI_DDRAWPALETTE_INT,
    pub dwIntRefCnt: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDRAWI_DDRAWPALETTE_INT {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDRAWI_DDRAWPALETTE_INT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDRAWI_DDRAWPALETTE_LCL {
    pub lpPalMore: u32,
    pub lpGbl: *mut DDRAWI_DDRAWPALETTE_GBL,
    pub dwUnused0: usize,
    pub dwLocalRefCnt: u32,
    pub pUnkOuter: ::windows_sys::core::IUnknown,
    pub lpDD_lcl: *mut DDRAWI_DIRECTDRAW_LCL,
    pub dwReserved1: usize,
    pub dwDDRAWReserved1: usize,
    pub dwDDRAWReserved2: usize,
    pub dwDDRAWReserved3: usize,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDRAWI_DDRAWPALETTE_LCL {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDRAWI_DDRAWPALETTE_LCL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDRAWI_DDRAWSURFACE_GBL {
    pub dwRefCnt: u32,
    pub dwGlobalFlags: u32,
    pub Anonymous1: DDRAWI_DDRAWSURFACE_GBL_0,
    pub Anonymous2: DDRAWI_DDRAWSURFACE_GBL_1,
    pub Anonymous3: DDRAWI_DDRAWSURFACE_GBL_2,
    pub fpVidMem: usize,
    pub Anonymous4: DDRAWI_DDRAWSURFACE_GBL_3,
    pub wHeight: u16,
    pub wWidth: u16,
    pub dwUsageCount: u32,
    pub dwReserved1: usize,
    pub ddpfSurface: DDPIXELFORMAT,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDRAWI_DDRAWSURFACE_GBL {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDRAWI_DDRAWSURFACE_GBL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub union DDRAWI_DDRAWSURFACE_GBL_0 {
    pub lpRectList: *mut ACCESSRECTLIST,
    pub dwBlockSizeY: u32,
    pub lSlicePitch: i32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDRAWI_DDRAWSURFACE_GBL_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDRAWI_DDRAWSURFACE_GBL_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub union DDRAWI_DDRAWSURFACE_GBL_1 {
    pub lpVidMemHeap: *mut VMEMHEAP,
    pub dwBlockSizeX: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDRAWI_DDRAWSURFACE_GBL_1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDRAWI_DDRAWSURFACE_GBL_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub union DDRAWI_DDRAWSURFACE_GBL_2 {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub lpDDHandle: *mut ::core::ffi::c_void,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDRAWI_DDRAWSURFACE_GBL_2 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDRAWI_DDRAWSURFACE_GBL_2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub union DDRAWI_DDRAWSURFACE_GBL_3 {
    pub lPitch: i32,
    pub dwLinearSize: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDRAWI_DDRAWSURFACE_GBL_3 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDRAWI_DDRAWSURFACE_GBL_3 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDRAWI_DDRAWSURFACE_GBL_MORE {
    pub dwSize: u32,
    pub Anonymous: DDRAWI_DDRAWSURFACE_GBL_MORE_0,
    pub pPageTable: *mut u32,
    pub cPages: u32,
    pub dwSavedDCContext: usize,
    pub fpAliasedVidMem: usize,
    pub dwDriverReserved: usize,
    pub dwHELReserved: usize,
    pub cPageUnlocks: u32,
    pub hKernelSurface: usize,
    pub dwKernelRefCnt: u32,
    pub lpColorInfo: *mut DDCOLORCONTROL,
    pub fpNTAlias: usize,
    pub dwContentsStamp: u32,
    pub lpvUnswappedDriverReserved: *mut ::core::ffi::c_void,
    pub lpDDRAWReserved2: *mut ::core::ffi::c_void,
    pub dwDDRAWReserved1: u32,
    pub dwDDRAWReserved2: u32,
    pub fpAliasOfVidMem: usize,
}
impl ::core::marker::Copy for DDRAWI_DDRAWSURFACE_GBL_MORE {}
impl ::core::clone::Clone for DDRAWI_DDRAWSURFACE_GBL_MORE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub union DDRAWI_DDRAWSURFACE_GBL_MORE_0 {
    pub dwPhysicalPageTable: u32,
    pub fpPhysicalVidMem: usize,
}
impl ::core::marker::Copy for DDRAWI_DDRAWSURFACE_GBL_MORE_0 {}
impl ::core::clone::Clone for DDRAWI_DDRAWSURFACE_GBL_MORE_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDRAWI_DDRAWSURFACE_INT {
    pub lpVtbl: *mut ::core::ffi::c_void,
    pub lpLcl: *mut DDRAWI_DDRAWSURFACE_LCL,
    pub lpLink: *mut DDRAWI_DDRAWSURFACE_INT,
    pub dwIntRefCnt: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDRAWI_DDRAWSURFACE_INT {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDRAWI_DDRAWSURFACE_INT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDRAWI_DDRAWSURFACE_LCL {
    pub lpSurfMore: *mut DDRAWI_DDRAWSURFACE_MORE,
    pub lpGbl: *mut DDRAWI_DDRAWSURFACE_GBL,
    pub hDDSurface: usize,
    pub lpAttachList: *mut ATTACHLIST,
    pub lpAttachListFrom: *mut ATTACHLIST,
    pub dwLocalRefCnt: u32,
    pub dwProcessId: u32,
    pub dwFlags: u32,
    pub ddsCaps: DDSCAPS,
    pub Anonymous1: DDRAWI_DDRAWSURFACE_LCL_0,
    pub Anonymous2: DDRAWI_DDRAWSURFACE_LCL_1,
    pub dwModeCreatedIn: u32,
    pub dwBackBufferCount: u32,
    pub ddckCKDestBlt: DDCOLORKEY,
    pub ddckCKSrcBlt: DDCOLORKEY,
    pub hDC: usize,
    pub dwReserved1: usize,
    pub ddckCKSrcOverlay: DDCOLORKEY,
    pub ddckCKDestOverlay: DDCOLORKEY,
    pub lpSurfaceOverlaying: *mut DDRAWI_DDRAWSURFACE_INT,
    pub dbnOverlayNode: DBLNODE,
    pub rcOverlaySrc: super::super::Foundation::RECT,
    pub rcOverlayDest: super::super::Foundation::RECT,
    pub dwClrXparent: u32,
    pub dwAlpha: u32,
    pub lOverlayX: i32,
    pub lOverlayY: i32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDRAWI_DDRAWSURFACE_LCL {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDRAWI_DDRAWSURFACE_LCL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub union DDRAWI_DDRAWSURFACE_LCL_0 {
    pub lpDDPalette: *mut DDRAWI_DDRAWPALETTE_INT,
    pub lp16DDPalette: *mut DDRAWI_DDRAWPALETTE_INT,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDRAWI_DDRAWSURFACE_LCL_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDRAWI_DDRAWSURFACE_LCL_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub union DDRAWI_DDRAWSURFACE_LCL_1 {
    pub lpDDClipper: *mut DDRAWI_DDRAWCLIPPER_LCL,
    pub lp16DDClipper: *mut DDRAWI_DDRAWCLIPPER_INT,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDRAWI_DDRAWSURFACE_LCL_1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDRAWI_DDRAWSURFACE_LCL_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDRAWI_DDRAWSURFACE_MORE {
    pub dwSize: u32,
    pub lpIUnknowns: *mut IUNKNOWN_LIST,
    pub lpDD_lcl: *mut DDRAWI_DIRECTDRAW_LCL,
    pub dwPageLockCount: u32,
    pub dwBytesAllocated: u32,
    pub lpDD_int: *mut DDRAWI_DIRECTDRAW_INT,
    pub dwMipMapCount: u32,
    pub lpDDIClipper: *mut DDRAWI_DDRAWCLIPPER_INT,
    pub lpHeapAliasInfo: *mut HEAPALIASINFO,
    pub dwOverlayFlags: u32,
    pub rgjunc: *mut ::core::ffi::c_void,
    pub lpVideoPort: *mut DDRAWI_DDVIDEOPORT_LCL,
    pub lpddOverlayFX: *mut DDOVERLAYFX,
    pub ddsCapsEx: DDSCAPSEX,
    pub dwTextureStage: u32,
    pub lpDDRAWReserved: *mut ::core::ffi::c_void,
    pub lpDDRAWReserved2: *mut ::core::ffi::c_void,
    pub lpDDrawReserved3: *mut ::core::ffi::c_void,
    pub dwDDrawReserved4: u32,
    pub lpDDrawReserved5: *mut ::core::ffi::c_void,
    pub lpGammaRamp: *mut u32,
    pub lpOriginalGammaRamp: *mut u32,
    pub lpDDrawReserved6: *mut ::core::ffi::c_void,
    pub dwSurfaceHandle: u32,
    pub qwDDrawReserved8: [u32; 2],
    pub lpDDrawReserved9: *mut ::core::ffi::c_void,
    pub cSurfaces: u32,
    pub pCreatedDDSurfaceDesc2: *mut DDSURFACEDESC2,
    pub slist: *mut *mut DDRAWI_DDRAWSURFACE_LCL,
    pub dwFVF: u32,
    pub lpVB: *mut ::core::ffi::c_void,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDRAWI_DDRAWSURFACE_MORE {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDRAWI_DDRAWSURFACE_MORE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDRAWI_DDVIDEOPORT_INT {
    pub lpVtbl: *mut ::core::ffi::c_void,
    pub lpLcl: *mut DDRAWI_DDVIDEOPORT_LCL,
    pub lpLink: *mut DDRAWI_DDVIDEOPORT_INT,
    pub dwIntRefCnt: u32,
    pub dwFlags: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDRAWI_DDVIDEOPORT_INT {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDRAWI_DDVIDEOPORT_INT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDRAWI_DDVIDEOPORT_LCL {
    pub lpDD: *mut DDRAWI_DIRECTDRAW_LCL,
    pub ddvpDesc: DDVIDEOPORTDESC,
    pub ddvpInfo: DDVIDEOPORTINFO,
    pub lpSurface: *mut DDRAWI_DDRAWSURFACE_INT,
    pub lpVBISurface: *mut DDRAWI_DDRAWSURFACE_INT,
    pub lpFlipInts: *mut *mut DDRAWI_DDRAWSURFACE_INT,
    pub dwNumAutoflip: u32,
    pub dwProcessID: u32,
    pub dwStateFlags: u32,
    pub dwFlags: u32,
    pub dwRefCnt: u32,
    pub fpLastFlip: usize,
    pub dwReserved1: usize,
    pub dwReserved2: usize,
    pub hDDVideoPort: super::super::Foundation::HANDLE,
    pub dwNumVBIAutoflip: u32,
    pub lpVBIDesc: *mut DDVIDEOPORTDESC,
    pub lpVideoDesc: *mut DDVIDEOPORTDESC,
    pub lpVBIInfo: *mut DDVIDEOPORTINFO,
    pub lpVideoInfo: *mut DDVIDEOPORTINFO,
    pub dwVBIProcessID: u32,
    pub lpVPNotify: *mut DDRAWI_DDVIDEOPORT_INT,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDRAWI_DDVIDEOPORT_LCL {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDRAWI_DDVIDEOPORT_LCL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDRAWI_DIRECTDRAW_GBL {
    pub dwRefCnt: u32,
    pub dwFlags: u32,
    pub fpPrimaryOrig: usize,
    pub ddCaps: DDCORECAPS,
    pub dwInternal1: u32,
    pub dwUnused1: [u32; 9],
    pub lpDDCBtmp: *mut DDHAL_CALLBACKS,
    pub dsList: *mut DDRAWI_DDRAWSURFACE_INT,
    pub palList: *mut DDRAWI_DDRAWPALETTE_INT,
    pub clipperList: *mut DDRAWI_DDRAWCLIPPER_INT,
    pub lp16DD: *mut DDRAWI_DIRECTDRAW_GBL,
    pub dwMaxOverlays: u32,
    pub dwCurrOverlays: u32,
    pub dwMonitorFrequency: u32,
    pub ddHELCaps: DDCORECAPS,
    pub dwUnused2: [u32; 50],
    pub ddckCKDestOverlay: DDCOLORKEY,
    pub ddckCKSrcOverlay: DDCOLORKEY,
    pub vmiData: VIDMEMINFO,
    pub lpDriverHandle: *mut ::core::ffi::c_void,
    pub lpExclusiveOwner: *mut DDRAWI_DIRECTDRAW_LCL,
    pub dwModeIndex: u32,
    pub dwModeIndexOrig: u32,
    pub dwNumFourCC: u32,
    pub lpdwFourCC: *mut u32,
    pub dwNumModes: u32,
    pub lpModeInfo: *mut DDHALMODEINFO,
    pub plProcessList: PROCESS_LIST,
    pub dwSurfaceLockCount: u32,
    pub dwAliasedLockCnt: u32,
    pub dwReserved3: usize,
    pub hDD: usize,
    pub cObsolete: [super::super::Foundation::CHAR; 12],
    pub dwReserved1: u32,
    pub dwReserved2: u32,
    pub dbnOverlayRoot: DBLNODE,
    pub lpwPDeviceFlags: *mut u16,
    pub dwPDevice: u32,
    pub dwWin16LockCnt: u32,
    pub dwUnused3: u32,
    pub hInstance: u32,
    pub dwEvent16: u32,
    pub dwSaveNumModes: u32,
    pub lpD3DGlobalDriverData: usize,
    pub lpD3DHALCallbacks: usize,
    pub ddBothCaps: DDCORECAPS,
    pub lpDDVideoPortCaps: *mut DDVIDEOPORTCAPS,
    pub dvpList: *mut DDRAWI_DDVIDEOPORT_INT,
    pub lpD3DHALCallbacks2: usize,
    pub rectDevice: super::super::Foundation::RECT,
    pub cMonitors: u32,
    pub gpbmiSrc: *mut ::core::ffi::c_void,
    pub gpbmiDest: *mut ::core::ffi::c_void,
    pub phaiHeapAliases: *mut HEAPALIASINFO,
    pub hKernelHandle: usize,
    pub pfnNotifyProc: usize,
    pub lpDDKernelCaps: *mut DDKERNELCAPS,
    pub lpddNLVCaps: *mut DDNONLOCALVIDMEMCAPS,
    pub lpddNLVHELCaps: *mut DDNONLOCALVIDMEMCAPS,
    pub lpddNLVBothCaps: *mut DDNONLOCALVIDMEMCAPS,
    pub lpD3DExtendedCaps: usize,
    pub dwDOSBoxEvent: u32,
    pub rectDesktop: super::super::Foundation::RECT,
    pub cDriverName: [super::super::Foundation::CHAR; 32],
    pub lpD3DHALCallbacks3: usize,
    pub dwNumZPixelFormats: u32,
    pub lpZPixelFormats: *mut DDPIXELFORMAT,
    pub mcList: *mut DDRAWI_DDMOTIONCOMP_INT,
    pub hDDVxd: u32,
    pub ddsCapsMore: DDSCAPSEX,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDRAWI_DIRECTDRAW_GBL {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDRAWI_DIRECTDRAW_GBL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDRAWI_DIRECTDRAW_INT {
    pub lpVtbl: *mut ::core::ffi::c_void,
    pub lpLcl: *mut DDRAWI_DIRECTDRAW_LCL,
    pub lpLink: *mut DDRAWI_DIRECTDRAW_INT,
    pub dwIntRefCnt: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDRAWI_DIRECTDRAW_INT {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDRAWI_DIRECTDRAW_INT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DDRAWI_DIRECTDRAW_LCL {
    pub lpDDMore: u32,
    pub lpGbl: *mut DDRAWI_DIRECTDRAW_GBL,
    pub dwUnused0: u32,
    pub dwLocalFlags: u32,
    pub dwLocalRefCnt: u32,
    pub dwProcessId: u32,
    pub pUnkOuter: ::windows_sys::core::IUnknown,
    pub dwObsolete1: u32,
    pub hWnd: usize,
    pub hDC: usize,
    pub dwErrorMode: u32,
    pub lpPrimary: *mut DDRAWI_DDRAWSURFACE_INT,
    pub lpCB: *mut DDRAWI_DDRAWSURFACE_INT,
    pub dwPreferredMode: u32,
    pub hD3DInstance: super::super::Foundation::HINSTANCE,
    pub pD3DIUnknown: ::windows_sys::core::IUnknown,
    pub lpDDCB: *mut DDHAL_CALLBACKS,
    pub hDDVxd: usize,
    pub dwAppHackFlags: u32,
    pub hFocusWnd: usize,
    pub dwHotTracking: u32,
    pub dwIMEState: u32,
    pub hWndPopup: usize,
    pub hDD: usize,
    pub hGammaCalibrator: usize,
    pub lpGammaCalibrator: LPDDGAMMACALIBRATORPROC,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DDRAWI_DIRECTDRAW_LCL {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DDRAWI_DIRECTDRAW_LCL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDRGBA {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}
impl ::core::marker::Copy for DDRGBA {}
impl ::core::clone::Clone for DDRGBA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDSCAPS {
    pub dwCaps: u32,
}
impl ::core::marker::Copy for DDSCAPS {}
impl ::core::clone::Clone for DDSCAPS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDSCAPS2 {
    pub dwCaps: u32,
    pub dwCaps2: u32,
    pub dwCaps3: u32,
    pub Anonymous: DDSCAPS2_0,
}
impl ::core::marker::Copy for DDSCAPS2 {}
impl ::core::clone::Clone for DDSCAPS2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub union DDSCAPS2_0 {
    pub dwCaps4: u32,
    pub dwVolumeDepth: u32,
}
impl ::core::marker::Copy for DDSCAPS2_0 {}
impl ::core::clone::Clone for DDSCAPS2_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDSCAPSEX {
    pub dwCaps2: u32,
    pub dwCaps3: u32,
    pub Anonymous: DDSCAPSEX_0,
}
impl ::core::marker::Copy for DDSCAPSEX {}
impl ::core::clone::Clone for DDSCAPSEX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub union DDSCAPSEX_0 {
    pub dwCaps4: u32,
    pub dwVolumeDepth: u32,
}
impl ::core::marker::Copy for DDSCAPSEX_0 {}
impl ::core::clone::Clone for DDSCAPSEX_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDSETSTATEININFO {
    pub lpSurfaceData: *mut DDSURFACEDATA,
    pub lpVideoPortData: *mut DDVIDEOPORTDATA,
}
impl ::core::marker::Copy for DDSETSTATEININFO {}
impl ::core::clone::Clone for DDSETSTATEININFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DDSETSTATEOUTINFO {
    pub bSoftwareAutoflip: super::super::Foundation::BOOL,
    pub dwSurfaceIndex: u32,
    pub dwVBISurfaceIndex: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DDSETSTATEOUTINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DDSETSTATEOUTINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDSKIPNEXTFIELDINFO {
    pub lpVideoPortData: *mut DDVIDEOPORTDATA,
    pub dwSkipFlags: u32,
}
impl ::core::marker::Copy for DDSKIPNEXTFIELDINFO {}
impl ::core::clone::Clone for DDSKIPNEXTFIELDINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DDSTEREOMODE {
    pub dwSize: u32,
    pub dwHeight: u32,
    pub dwWidth: u32,
    pub dwBpp: u32,
    pub dwRefreshRate: u32,
    pub bSupported: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DDSTEREOMODE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DDSTEREOMODE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDSURFACEDATA {
    pub ddsCaps: u32,
    pub dwSurfaceOffset: u32,
    pub fpLockPtr: usize,
    pub dwWidth: u32,
    pub dwHeight: u32,
    pub lPitch: i32,
    pub dwOverlayFlags: u32,
    pub dwOverlayOffset: u32,
    pub dwOverlaySrcWidth: u32,
    pub dwOverlaySrcHeight: u32,
    pub dwOverlayDestWidth: u32,
    pub dwOverlayDestHeight: u32,
    pub dwVideoPortId: u32,
    pub dwFormatFlags: u32,
    pub dwFormatFourCC: u32,
    pub dwFormatBitCount: u32,
    pub dwRBitMask: u32,
    pub dwGBitMask: u32,
    pub dwBBitMask: u32,
    pub dwDriverReserved1: u32,
    pub dwDriverReserved2: u32,
    pub dwDriverReserved3: u32,
    pub dwDriverReserved4: u32,
}
impl ::core::marker::Copy for DDSURFACEDATA {}
impl ::core::clone::Clone for DDSURFACEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDSURFACEDESC {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub dwHeight: u32,
    pub dwWidth: u32,
    pub Anonymous1: DDSURFACEDESC_0,
    pub dwBackBufferCount: u32,
    pub Anonymous2: DDSURFACEDESC_1,
    pub dwAlphaBitDepth: u32,
    pub dwReserved: u32,
    pub lpSurface: *mut ::core::ffi::c_void,
    pub ddckCKDestOverlay: DDCOLORKEY,
    pub ddckCKDestBlt: DDCOLORKEY,
    pub ddckCKSrcOverlay: DDCOLORKEY,
    pub ddckCKSrcBlt: DDCOLORKEY,
    pub ddpfPixelFormat: DDPIXELFORMAT,
    pub ddsCaps: DDSCAPS,
}
impl ::core::marker::Copy for DDSURFACEDESC {}
impl ::core::clone::Clone for DDSURFACEDESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub union DDSURFACEDESC_0 {
    pub lPitch: i32,
    pub dwLinearSize: u32,
}
impl ::core::marker::Copy for DDSURFACEDESC_0 {}
impl ::core::clone::Clone for DDSURFACEDESC_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub union DDSURFACEDESC_1 {
    pub dwMipMapCount: u32,
    pub dwZBufferBitDepth: u32,
    pub dwRefreshRate: u32,
}
impl ::core::marker::Copy for DDSURFACEDESC_1 {}
impl ::core::clone::Clone for DDSURFACEDESC_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDSURFACEDESC2 {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub dwHeight: u32,
    pub dwWidth: u32,
    pub Anonymous1: DDSURFACEDESC2_0,
    pub Anonymous2: DDSURFACEDESC2_1,
    pub Anonymous3: DDSURFACEDESC2_2,
    pub dwAlphaBitDepth: u32,
    pub dwReserved: u32,
    pub lpSurface: *mut ::core::ffi::c_void,
    pub Anonymous4: DDSURFACEDESC2_3,
    pub ddckCKDestBlt: DDCOLORKEY,
    pub ddckCKSrcOverlay: DDCOLORKEY,
    pub ddckCKSrcBlt: DDCOLORKEY,
    pub Anonymous5: DDSURFACEDESC2_4,
    pub ddsCaps: DDSCAPS2,
    pub dwTextureStage: u32,
}
impl ::core::marker::Copy for DDSURFACEDESC2 {}
impl ::core::clone::Clone for DDSURFACEDESC2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub union DDSURFACEDESC2_0 {
    pub lPitch: i32,
    pub dwLinearSize: u32,
}
impl ::core::marker::Copy for DDSURFACEDESC2_0 {}
impl ::core::clone::Clone for DDSURFACEDESC2_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub union DDSURFACEDESC2_1 {
    pub dwBackBufferCount: u32,
    pub dwDepth: u32,
}
impl ::core::marker::Copy for DDSURFACEDESC2_1 {}
impl ::core::clone::Clone for DDSURFACEDESC2_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub union DDSURFACEDESC2_2 {
    pub dwMipMapCount: u32,
    pub dwRefreshRate: u32,
    pub dwSrcVBHandle: u32,
}
impl ::core::marker::Copy for DDSURFACEDESC2_2 {}
impl ::core::clone::Clone for DDSURFACEDESC2_2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub union DDSURFACEDESC2_3 {
    pub ddckCKDestOverlay: DDCOLORKEY,
    pub dwEmptyFaceColor: u32,
}
impl ::core::marker::Copy for DDSURFACEDESC2_3 {}
impl ::core::clone::Clone for DDSURFACEDESC2_3 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub union DDSURFACEDESC2_4 {
    pub ddpfPixelFormat: DDPIXELFORMAT,
    pub dwFVF: u32,
}
impl ::core::marker::Copy for DDSURFACEDESC2_4 {}
impl ::core::clone::Clone for DDSURFACEDESC2_4 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDTRANSFERININFO {
    pub lpSurfaceData: *mut DDSURFACEDATA,
    pub dwStartLine: u32,
    pub dwEndLine: u32,
    pub dwTransferID: usize,
    pub dwTransferFlags: u32,
    pub lpDestMDL: *mut MDL,
}
impl ::core::marker::Copy for DDTRANSFERININFO {}
impl ::core::clone::Clone for DDTRANSFERININFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDTRANSFEROUTINFO {
    pub dwBufferPolarity: u32,
}
impl ::core::marker::Copy for DDTRANSFEROUTINFO {}
impl ::core::clone::Clone for DDTRANSFEROUTINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDVERSIONDATA {
    pub dwHALVersion: u32,
    pub dwReserved1: usize,
    pub dwReserved2: usize,
}
impl ::core::marker::Copy for DDVERSIONDATA {}
impl ::core::clone::Clone for DDVERSIONDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDVIDEOPORTBANDWIDTH {
    pub dwSize: u32,
    pub dwCaps: u32,
    pub dwOverlay: u32,
    pub dwColorkey: u32,
    pub dwYInterpolate: u32,
    pub dwYInterpAndColorkey: u32,
    pub dwReserved1: usize,
    pub dwReserved2: usize,
}
impl ::core::marker::Copy for DDVIDEOPORTBANDWIDTH {}
impl ::core::clone::Clone for DDVIDEOPORTBANDWIDTH {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDVIDEOPORTCAPS {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub dwMaxWidth: u32,
    pub dwMaxVBIWidth: u32,
    pub dwMaxHeight: u32,
    pub dwVideoPortID: u32,
    pub dwCaps: u32,
    pub dwFX: u32,
    pub dwNumAutoFlipSurfaces: u32,
    pub dwAlignVideoPortBoundary: u32,
    pub dwAlignVideoPortPrescaleWidth: u32,
    pub dwAlignVideoPortCropBoundary: u32,
    pub dwAlignVideoPortCropWidth: u32,
    pub dwPreshrinkXStep: u32,
    pub dwPreshrinkYStep: u32,
    pub dwNumVBIAutoFlipSurfaces: u32,
    pub dwNumPreferredAutoflip: u32,
    pub wNumFilterTapsX: u16,
    pub wNumFilterTapsY: u16,
}
impl ::core::marker::Copy for DDVIDEOPORTCAPS {}
impl ::core::clone::Clone for DDVIDEOPORTCAPS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDVIDEOPORTCONNECT {
    pub dwSize: u32,
    pub dwPortWidth: u32,
    pub guidTypeID: ::windows_sys::core::GUID,
    pub dwFlags: u32,
    pub dwReserved1: usize,
}
impl ::core::marker::Copy for DDVIDEOPORTCONNECT {}
impl ::core::clone::Clone for DDVIDEOPORTCONNECT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDVIDEOPORTDATA {
    pub dwVideoPortId: u32,
    pub dwVPFlags: u32,
    pub dwOriginOffset: u32,
    pub dwHeight: u32,
    pub dwVBIHeight: u32,
    pub dwDriverReserved1: u32,
    pub dwDriverReserved2: u32,
    pub dwDriverReserved3: u32,
}
impl ::core::marker::Copy for DDVIDEOPORTDATA {}
impl ::core::clone::Clone for DDVIDEOPORTDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDVIDEOPORTDESC {
    pub dwSize: u32,
    pub dwFieldWidth: u32,
    pub dwVBIWidth: u32,
    pub dwFieldHeight: u32,
    pub dwMicrosecondsPerField: u32,
    pub dwMaxPixelsPerSecond: u32,
    pub dwVideoPortID: u32,
    pub dwReserved1: u32,
    pub VideoPortType: DDVIDEOPORTCONNECT,
    pub dwReserved2: usize,
    pub dwReserved3: usize,
}
impl ::core::marker::Copy for DDVIDEOPORTDESC {}
impl ::core::clone::Clone for DDVIDEOPORTDESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DDVIDEOPORTINFO {
    pub dwSize: u32,
    pub dwOriginX: u32,
    pub dwOriginY: u32,
    pub dwVPFlags: u32,
    pub rCrop: super::super::Foundation::RECT,
    pub dwPrescaleWidth: u32,
    pub dwPrescaleHeight: u32,
    pub lpddpfInputFormat: *mut DDPIXELFORMAT,
    pub lpddpfVBIInputFormat: *mut DDPIXELFORMAT,
    pub lpddpfVBIOutputFormat: *mut DDPIXELFORMAT,
    pub dwVBIHeight: u32,
    pub dwReserved1: usize,
    pub dwReserved2: usize,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DDVIDEOPORTINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DDVIDEOPORTINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DDVIDEOPORTNOTIFY {
    pub ApproximateTimeStamp: i64,
    pub lField: i32,
    pub dwSurfaceIndex: u32,
    pub lDone: i32,
}
impl ::core::marker::Copy for DDVIDEOPORTNOTIFY {}
impl ::core::clone::Clone for DDVIDEOPORTNOTIFY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DDVIDEOPORTSTATUS {
    pub dwSize: u32,
    pub bInUse: super::super::Foundation::BOOL,
    pub dwFlags: u32,
    pub dwReserved1: u32,
    pub VideoPortType: DDVIDEOPORTCONNECT,
    pub dwReserved2: usize,
    pub dwReserved3: usize,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DDVIDEOPORTSTATUS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DDVIDEOPORTSTATUS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_ADDATTACHEDSURFACEDATA {
    pub lpDD: *mut DD_DIRECTDRAW_GLOBAL,
    pub lpDDSurface: *mut DD_SURFACE_LOCAL,
    pub lpSurfAttached: *mut DD_SURFACE_LOCAL,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub AddAttachedSurface: *mut ::core::ffi::c_void,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_ADDATTACHEDSURFACEDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_ADDATTACHEDSURFACEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_ATTACHLIST {
    pub lpLink: *mut DD_ATTACHLIST,
    pub lpAttached: *mut DD_SURFACE_LOCAL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_ATTACHLIST {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_ATTACHLIST {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_BEGINMOCOMPFRAMEDATA {
    pub lpDD: *mut DD_DIRECTDRAW_LOCAL,
    pub lpMoComp: *mut DD_MOTIONCOMP_LOCAL,
    pub lpDestSurface: *mut DD_SURFACE_LOCAL,
    pub dwInputDataSize: u32,
    pub lpInputData: *mut ::core::ffi::c_void,
    pub dwOutputDataSize: u32,
    pub lpOutputData: *mut ::core::ffi::c_void,
    pub ddRVal: ::windows_sys::core::HRESULT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_BEGINMOCOMPFRAMEDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_BEGINMOCOMPFRAMEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_BLTDATA {
    pub lpDD: *mut DD_DIRECTDRAW_GLOBAL,
    pub lpDDDestSurface: *mut DD_SURFACE_LOCAL,
    pub rDest: super::super::Foundation::RECTL,
    pub lpDDSrcSurface: *mut DD_SURFACE_LOCAL,
    pub rSrc: super::super::Foundation::RECTL,
    pub dwFlags: u32,
    pub dwROPFlags: u32,
    pub bltFX: DDBLTFX,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub Blt: *mut ::core::ffi::c_void,
    pub IsClipped: super::super::Foundation::BOOL,
    pub rOrigDest: super::super::Foundation::RECTL,
    pub rOrigSrc: super::super::Foundation::RECTL,
    pub dwRectCnt: u32,
    pub prDestRects: *mut super::super::Foundation::RECT,
    pub dwAFlags: u32,
    pub ddargbScaleFactors: DDARGB,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_BLTDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_BLTDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DD_CALLBACKS {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub DestroyDriver: PDD_DESTROYDRIVER,
    pub CreateSurface: PDD_CREATESURFACE,
    pub SetColorKey: PDD_SETCOLORKEY,
    pub SetMode: PDD_SETMODE,
    pub WaitForVerticalBlank: PDD_WAITFORVERTICALBLANK,
    pub CanCreateSurface: PDD_CANCREATESURFACE,
    pub CreatePalette: PDD_CREATEPALETTE,
    pub GetScanLine: PDD_GETSCANLINE,
    pub MapMemory: PDD_MAPMEMORY,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DD_CALLBACKS {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DD_CALLBACKS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_CANCREATESURFACEDATA {
    pub lpDD: *mut DD_DIRECTDRAW_GLOBAL,
    pub lpDDSurfaceDesc: *mut DDSURFACEDESC,
    pub bIsDifferentPixelFormat: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub CanCreateSurface: *mut ::core::ffi::c_void,
}
impl ::core::marker::Copy for DD_CANCREATESURFACEDATA {}
impl ::core::clone::Clone for DD_CANCREATESURFACEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_CANCREATEVPORTDATA {
    pub lpDD: *mut DD_DIRECTDRAW_LOCAL,
    pub lpDDVideoPortDesc: *mut DDVIDEOPORTDESC,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub CanCreateVideoPort: *mut ::core::ffi::c_void,
}
impl ::core::marker::Copy for DD_CANCREATEVPORTDATA {}
impl ::core::clone::Clone for DD_CANCREATEVPORTDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_CLIPPER_GLOBAL {
    pub dwReserved1: usize,
}
impl ::core::marker::Copy for DD_CLIPPER_GLOBAL {}
impl ::core::clone::Clone for DD_CLIPPER_GLOBAL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_CLIPPER_LOCAL {
    pub dwReserved1: usize,
}
impl ::core::marker::Copy for DD_CLIPPER_LOCAL {}
impl ::core::clone::Clone for DD_CLIPPER_LOCAL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_COLORCONTROLCALLBACKS {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub ColorControl: PDD_COLORCB_COLORCONTROL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_COLORCONTROLCALLBACKS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_COLORCONTROLCALLBACKS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_COLORCONTROLDATA {
    pub lpDD: *mut DD_DIRECTDRAW_GLOBAL,
    pub lpDDSurface: *mut DD_SURFACE_LOCAL,
    pub lpColorData: *mut DDCOLORCONTROL,
    pub dwFlags: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub ColorControl: *mut ::core::ffi::c_void,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_COLORCONTROLDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_COLORCONTROLDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_CREATEMOCOMPDATA {
    pub lpDD: *mut DD_DIRECTDRAW_LOCAL,
    pub lpMoComp: *mut DD_MOTIONCOMP_LOCAL,
    pub lpGuid: *mut ::windows_sys::core::GUID,
    pub dwUncompWidth: u32,
    pub dwUncompHeight: u32,
    pub ddUncompPixelFormat: DDPIXELFORMAT,
    pub lpData: *mut ::core::ffi::c_void,
    pub dwDataSize: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
}
impl ::core::marker::Copy for DD_CREATEMOCOMPDATA {}
impl ::core::clone::Clone for DD_CREATEMOCOMPDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DD_CREATEPALETTEDATA {
    pub lpDD: *mut DD_DIRECTDRAW_GLOBAL,
    pub lpDDPalette: *mut DD_PALETTE_GLOBAL,
    pub lpColorTable: *mut super::Gdi::PALETTEENTRY,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub CreatePalette: *mut ::core::ffi::c_void,
    pub is_excl: super::super::Foundation::BOOL,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DD_CREATEPALETTEDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DD_CREATEPALETTEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_CREATESURFACEDATA {
    pub lpDD: *mut DD_DIRECTDRAW_GLOBAL,
    pub lpDDSurfaceDesc: *mut DDSURFACEDESC,
    pub lplpSList: *mut *mut DD_SURFACE_LOCAL,
    pub dwSCnt: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub CreateSurface: *mut ::core::ffi::c_void,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_CREATESURFACEDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_CREATESURFACEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_CREATESURFACEEXDATA {
    pub dwFlags: u32,
    pub lpDDLcl: *mut DD_DIRECTDRAW_LOCAL,
    pub lpDDSLcl: *mut DD_SURFACE_LOCAL,
    pub ddRVal: ::windows_sys::core::HRESULT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_CREATESURFACEEXDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_CREATESURFACEEXDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_CREATEVPORTDATA {
    pub lpDD: *mut DD_DIRECTDRAW_LOCAL,
    pub lpDDVideoPortDesc: *mut DDVIDEOPORTDESC,
    pub lpVideoPort: *mut DD_VIDEOPORT_LOCAL,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub CreateVideoPort: *mut ::core::ffi::c_void,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_CREATEVPORTDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_CREATEVPORTDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_D3DBUFCALLBACKS {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub CanCreateD3DBuffer: PDD_CANCREATESURFACE,
    pub CreateD3DBuffer: PDD_CREATESURFACE,
    pub DestroyD3DBuffer: PDD_SURFCB_DESTROYSURFACE,
    pub LockD3DBuffer: PDD_SURFCB_LOCK,
    pub UnlockD3DBuffer: PDD_SURFCB_UNLOCK,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_D3DBUFCALLBACKS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_D3DBUFCALLBACKS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_DESTROYDDLOCALDATA {
    pub dwFlags: u32,
    pub pDDLcl: *mut DD_DIRECTDRAW_LOCAL,
    pub ddRVal: ::windows_sys::core::HRESULT,
}
impl ::core::marker::Copy for DD_DESTROYDDLOCALDATA {}
impl ::core::clone::Clone for DD_DESTROYDDLOCALDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_DESTROYMOCOMPDATA {
    pub lpDD: *mut DD_DIRECTDRAW_LOCAL,
    pub lpMoComp: *mut DD_MOTIONCOMP_LOCAL,
    pub ddRVal: ::windows_sys::core::HRESULT,
}
impl ::core::marker::Copy for DD_DESTROYMOCOMPDATA {}
impl ::core::clone::Clone for DD_DESTROYMOCOMPDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_DESTROYPALETTEDATA {
    pub lpDD: *mut DD_DIRECTDRAW_GLOBAL,
    pub lpDDPalette: *mut DD_PALETTE_GLOBAL,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub DestroyPalette: *mut ::core::ffi::c_void,
}
impl ::core::marker::Copy for DD_DESTROYPALETTEDATA {}
impl ::core::clone::Clone for DD_DESTROYPALETTEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_DESTROYSURFACEDATA {
    pub lpDD: *mut DD_DIRECTDRAW_GLOBAL,
    pub lpDDSurface: *mut DD_SURFACE_LOCAL,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub DestroySurface: *mut ::core::ffi::c_void,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_DESTROYSURFACEDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_DESTROYSURFACEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_DESTROYVPORTDATA {
    pub lpDD: *mut DD_DIRECTDRAW_LOCAL,
    pub lpVideoPort: *mut DD_VIDEOPORT_LOCAL,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub DestroyVideoPort: *mut ::core::ffi::c_void,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_DESTROYVPORTDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_DESTROYVPORTDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_DIRECTDRAW_GLOBAL {
    pub dhpdev: *mut ::core::ffi::c_void,
    pub dwReserved1: usize,
    pub dwReserved2: usize,
    pub lpDDVideoPortCaps: *mut DDVIDEOPORTCAPS,
}
impl ::core::marker::Copy for DD_DIRECTDRAW_GLOBAL {}
impl ::core::clone::Clone for DD_DIRECTDRAW_GLOBAL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_DIRECTDRAW_LOCAL {
    pub lpGbl: *mut DD_DIRECTDRAW_GLOBAL,
}
impl ::core::marker::Copy for DD_DIRECTDRAW_LOCAL {}
impl ::core::clone::Clone for DD_DIRECTDRAW_LOCAL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_DRVSETCOLORKEYDATA {
    pub lpDDSurface: *mut DD_SURFACE_LOCAL,
    pub dwFlags: u32,
    pub ckNew: DDCOLORKEY,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub SetColorKey: *mut ::core::ffi::c_void,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_DRVSETCOLORKEYDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_DRVSETCOLORKEYDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_ENDMOCOMPFRAMEDATA {
    pub lpDD: *mut DD_DIRECTDRAW_LOCAL,
    pub lpMoComp: *mut DD_MOTIONCOMP_LOCAL,
    pub lpInputData: *mut ::core::ffi::c_void,
    pub dwInputDataSize: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
}
impl ::core::marker::Copy for DD_ENDMOCOMPFRAMEDATA {}
impl ::core::clone::Clone for DD_ENDMOCOMPFRAMEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_FLIPDATA {
    pub lpDD: *mut DD_DIRECTDRAW_GLOBAL,
    pub lpSurfCurr: *mut DD_SURFACE_LOCAL,
    pub lpSurfTarg: *mut DD_SURFACE_LOCAL,
    pub dwFlags: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub Flip: *mut ::core::ffi::c_void,
    pub lpSurfCurrLeft: *mut DD_SURFACE_LOCAL,
    pub lpSurfTargLeft: *mut DD_SURFACE_LOCAL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_FLIPDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_FLIPDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_FLIPTOGDISURFACEDATA {
    pub lpDD: *mut DD_DIRECTDRAW_GLOBAL,
    pub dwToGDI: u32,
    pub dwReserved: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub FlipToGDISurface: *mut ::core::ffi::c_void,
}
impl ::core::marker::Copy for DD_FLIPTOGDISURFACEDATA {}
impl ::core::clone::Clone for DD_FLIPTOGDISURFACEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_FLIPVPORTDATA {
    pub lpDD: *mut DD_DIRECTDRAW_LOCAL,
    pub lpVideoPort: *mut DD_VIDEOPORT_LOCAL,
    pub lpSurfCurr: *mut DD_SURFACE_LOCAL,
    pub lpSurfTarg: *mut DD_SURFACE_LOCAL,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub FlipVideoPort: *mut ::core::ffi::c_void,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_FLIPVPORTDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_FLIPVPORTDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_FREEDRIVERMEMORYDATA {
    pub lpDD: *mut DD_DIRECTDRAW_GLOBAL,
    pub lpDDSurface: *mut DD_SURFACE_LOCAL,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub FreeDriverMemory: *mut ::core::ffi::c_void,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_FREEDRIVERMEMORYDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_FREEDRIVERMEMORYDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_GETAVAILDRIVERMEMORYDATA {
    pub lpDD: *mut DD_DIRECTDRAW_GLOBAL,
    pub DDSCaps: DDSCAPS,
    pub dwTotal: u32,
    pub dwFree: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetAvailDriverMemory: *mut ::core::ffi::c_void,
}
impl ::core::marker::Copy for DD_GETAVAILDRIVERMEMORYDATA {}
impl ::core::clone::Clone for DD_GETAVAILDRIVERMEMORYDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_GETBLTSTATUSDATA {
    pub lpDD: *mut DD_DIRECTDRAW_GLOBAL,
    pub lpDDSurface: *mut DD_SURFACE_LOCAL,
    pub dwFlags: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetBltStatus: *mut ::core::ffi::c_void,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_GETBLTSTATUSDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_GETBLTSTATUSDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_GETDRIVERINFODATA {
    pub dhpdev: *mut ::core::ffi::c_void,
    pub dwSize: u32,
    pub dwFlags: u32,
    pub guidInfo: ::windows_sys::core::GUID,
    pub dwExpectedSize: u32,
    pub lpvData: *mut ::core::ffi::c_void,
    pub dwActualSize: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
}
impl ::core::marker::Copy for DD_GETDRIVERINFODATA {}
impl ::core::clone::Clone for DD_GETDRIVERINFODATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_GETDRIVERSTATEDATA {
    pub dwFlags: u32,
    pub Anonymous: DD_GETDRIVERSTATEDATA_0,
    pub lpdwStates: *mut u32,
    pub dwLength: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
}
impl ::core::marker::Copy for DD_GETDRIVERSTATEDATA {}
impl ::core::clone::Clone for DD_GETDRIVERSTATEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub union DD_GETDRIVERSTATEDATA_0 {
    pub lpDD: *mut DD_DIRECTDRAW_GLOBAL,
    pub dwhContext: usize,
}
impl ::core::marker::Copy for DD_GETDRIVERSTATEDATA_0 {}
impl ::core::clone::Clone for DD_GETDRIVERSTATEDATA_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_GETFLIPSTATUSDATA {
    pub lpDD: *mut DD_DIRECTDRAW_GLOBAL,
    pub lpDDSurface: *mut DD_SURFACE_LOCAL,
    pub dwFlags: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetFlipStatus: *mut ::core::ffi::c_void,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_GETFLIPSTATUSDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_GETFLIPSTATUSDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_GETHEAPALIGNMENTDATA {
    pub dwInstance: usize,
    pub dwHeap: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetHeapAlignment: *mut ::core::ffi::c_void,
    pub Alignment: HEAPALIGNMENT,
}
impl ::core::marker::Copy for DD_GETHEAPALIGNMENTDATA {}
impl ::core::clone::Clone for DD_GETHEAPALIGNMENTDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_GETINTERNALMOCOMPDATA {
    pub lpDD: *mut DD_DIRECTDRAW_LOCAL,
    pub lpGuid: *mut ::windows_sys::core::GUID,
    pub dwWidth: u32,
    pub dwHeight: u32,
    pub ddPixelFormat: DDPIXELFORMAT,
    pub dwScratchMemAlloc: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
}
impl ::core::marker::Copy for DD_GETINTERNALMOCOMPDATA {}
impl ::core::clone::Clone for DD_GETINTERNALMOCOMPDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_GETMOCOMPCOMPBUFFDATA {
    pub lpDD: *mut DD_DIRECTDRAW_LOCAL,
    pub lpGuid: *mut ::windows_sys::core::GUID,
    pub dwWidth: u32,
    pub dwHeight: u32,
    pub ddPixelFormat: DDPIXELFORMAT,
    pub dwNumTypesCompBuffs: u32,
    pub lpCompBuffInfo: *mut DDCOMPBUFFERINFO,
    pub ddRVal: ::windows_sys::core::HRESULT,
}
impl ::core::marker::Copy for DD_GETMOCOMPCOMPBUFFDATA {}
impl ::core::clone::Clone for DD_GETMOCOMPCOMPBUFFDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_GETMOCOMPFORMATSDATA {
    pub lpDD: *mut DD_DIRECTDRAW_LOCAL,
    pub lpGuid: *mut ::windows_sys::core::GUID,
    pub dwNumFormats: u32,
    pub lpFormats: *mut DDPIXELFORMAT,
    pub ddRVal: ::windows_sys::core::HRESULT,
}
impl ::core::marker::Copy for DD_GETMOCOMPFORMATSDATA {}
impl ::core::clone::Clone for DD_GETMOCOMPFORMATSDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_GETMOCOMPGUIDSDATA {
    pub lpDD: *mut DD_DIRECTDRAW_LOCAL,
    pub dwNumGuids: u32,
    pub lpGuids: *mut ::windows_sys::core::GUID,
    pub ddRVal: ::windows_sys::core::HRESULT,
}
impl ::core::marker::Copy for DD_GETMOCOMPGUIDSDATA {}
impl ::core::clone::Clone for DD_GETMOCOMPGUIDSDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_GETSCANLINEDATA {
    pub lpDD: *mut DD_DIRECTDRAW_GLOBAL,
    pub dwScanLine: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetScanLine: *mut ::core::ffi::c_void,
}
impl ::core::marker::Copy for DD_GETSCANLINEDATA {}
impl ::core::clone::Clone for DD_GETSCANLINEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_GETVPORTBANDWIDTHDATA {
    pub lpDD: *mut DD_DIRECTDRAW_LOCAL,
    pub lpVideoPort: *mut DD_VIDEOPORT_LOCAL,
    pub lpddpfFormat: *mut DDPIXELFORMAT,
    pub dwWidth: u32,
    pub dwHeight: u32,
    pub dwFlags: u32,
    pub lpBandwidth: *mut DDVIDEOPORTBANDWIDTH,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetVideoPortBandwidth: *mut ::core::ffi::c_void,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_GETVPORTBANDWIDTHDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_GETVPORTBANDWIDTHDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_GETVPORTCONNECTDATA {
    pub lpDD: *mut DD_DIRECTDRAW_LOCAL,
    pub dwPortId: u32,
    pub lpConnect: *mut DDVIDEOPORTCONNECT,
    pub dwNumEntries: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetVideoPortConnectInfo: *mut ::core::ffi::c_void,
}
impl ::core::marker::Copy for DD_GETVPORTCONNECTDATA {}
impl ::core::clone::Clone for DD_GETVPORTCONNECTDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_GETVPORTFIELDDATA {
    pub lpDD: *mut DD_DIRECTDRAW_LOCAL,
    pub lpVideoPort: *mut DD_VIDEOPORT_LOCAL,
    pub bField: super::super::Foundation::BOOL,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetVideoPortField: *mut ::core::ffi::c_void,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_GETVPORTFIELDDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_GETVPORTFIELDDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_GETVPORTFLIPSTATUSDATA {
    pub lpDD: *mut DD_DIRECTDRAW_LOCAL,
    pub fpSurface: usize,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetVideoPortFlipStatus: *mut ::core::ffi::c_void,
}
impl ::core::marker::Copy for DD_GETVPORTFLIPSTATUSDATA {}
impl ::core::clone::Clone for DD_GETVPORTFLIPSTATUSDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_GETVPORTINPUTFORMATDATA {
    pub lpDD: *mut DD_DIRECTDRAW_LOCAL,
    pub lpVideoPort: *mut DD_VIDEOPORT_LOCAL,
    pub dwFlags: u32,
    pub lpddpfFormat: *mut DDPIXELFORMAT,
    pub dwNumFormats: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetVideoPortInputFormats: *mut ::core::ffi::c_void,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_GETVPORTINPUTFORMATDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_GETVPORTINPUTFORMATDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_GETVPORTLINEDATA {
    pub lpDD: *mut DD_DIRECTDRAW_LOCAL,
    pub lpVideoPort: *mut DD_VIDEOPORT_LOCAL,
    pub dwLine: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetVideoPortLine: *mut ::core::ffi::c_void,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_GETVPORTLINEDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_GETVPORTLINEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_GETVPORTOUTPUTFORMATDATA {
    pub lpDD: *mut DD_DIRECTDRAW_LOCAL,
    pub lpVideoPort: *mut DD_VIDEOPORT_LOCAL,
    pub dwFlags: u32,
    pub lpddpfInputFormat: *mut DDPIXELFORMAT,
    pub lpddpfOutputFormats: *mut DDPIXELFORMAT,
    pub dwNumFormats: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetVideoPortInputFormats: *mut ::core::ffi::c_void,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_GETVPORTOUTPUTFORMATDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_GETVPORTOUTPUTFORMATDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_GETVPORTSIGNALDATA {
    pub lpDD: *mut DD_DIRECTDRAW_LOCAL,
    pub lpVideoPort: *mut DD_VIDEOPORT_LOCAL,
    pub dwStatus: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub GetVideoSignalStatus: *mut ::core::ffi::c_void,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_GETVPORTSIGNALDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_GETVPORTSIGNALDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_HALINFO {
    pub dwSize: u32,
    pub vmiData: VIDEOMEMORYINFO,
    pub ddCaps: DDNTCORECAPS,
    pub GetDriverInfo: PDD_GETDRIVERINFO,
    pub dwFlags: u32,
    pub lpD3DGlobalDriverData: *mut ::core::ffi::c_void,
    pub lpD3DHALCallbacks: *mut ::core::ffi::c_void,
    pub lpD3DBufCallbacks: *mut DD_D3DBUFCALLBACKS,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_HALINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_HALINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_HALINFO_V4 {
    pub dwSize: u32,
    pub vmiData: VIDEOMEMORYINFO,
    pub ddCaps: DDNTCORECAPS,
    pub GetDriverInfo: PDD_GETDRIVERINFO,
    pub dwFlags: u32,
}
impl ::core::marker::Copy for DD_HALINFO_V4 {}
impl ::core::clone::Clone for DD_HALINFO_V4 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_KERNELCALLBACKS {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub SyncSurfaceData: PDD_KERNELCB_SYNCSURFACE,
    pub SyncVideoPortData: PDD_KERNELCB_SYNCVIDEOPORT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_KERNELCALLBACKS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_KERNELCALLBACKS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_LOCKDATA {
    pub lpDD: *mut DD_DIRECTDRAW_GLOBAL,
    pub lpDDSurface: *mut DD_SURFACE_LOCAL,
    pub bHasRect: u32,
    pub rArea: super::super::Foundation::RECTL,
    pub lpSurfData: *mut ::core::ffi::c_void,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub Lock: *mut ::core::ffi::c_void,
    pub dwFlags: u32,
    pub fpProcess: usize,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_LOCKDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_LOCKDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_MAPMEMORYDATA {
    pub lpDD: *mut DD_DIRECTDRAW_GLOBAL,
    pub bMap: super::super::Foundation::BOOL,
    pub hProcess: super::super::Foundation::HANDLE,
    pub fpProcess: usize,
    pub ddRVal: ::windows_sys::core::HRESULT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_MAPMEMORYDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_MAPMEMORYDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_MISCELLANEOUS2CALLBACKS {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub AlphaBlt: PDD_ALPHABLT,
    pub CreateSurfaceEx: PDD_CREATESURFACEEX,
    pub GetDriverState: PDD_GETDRIVERSTATE,
    pub DestroyDDLocal: PDD_DESTROYDDLOCAL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_MISCELLANEOUS2CALLBACKS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_MISCELLANEOUS2CALLBACKS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_MISCELLANEOUSCALLBACKS {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub GetAvailDriverMemory: PDD_GETAVAILDRIVERMEMORY,
}
impl ::core::marker::Copy for DD_MISCELLANEOUSCALLBACKS {}
impl ::core::clone::Clone for DD_MISCELLANEOUSCALLBACKS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_MORECAPS {
    pub dwSize: u32,
    pub dwAlphaCaps: u32,
    pub dwSVBAlphaCaps: u32,
    pub dwVSBAlphaCaps: u32,
    pub dwSSBAlphaCaps: u32,
    pub dwFilterCaps: u32,
    pub dwSVBFilterCaps: u32,
    pub dwVSBFilterCaps: u32,
    pub dwSSBFilterCaps: u32,
}
impl ::core::marker::Copy for DD_MORECAPS {}
impl ::core::clone::Clone for DD_MORECAPS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_MORESURFACECAPS {
    pub dwSize: u32,
    pub ddsCapsMore: DDSCAPSEX,
    pub ddsExtendedHeapRestrictions: [DD_MORESURFACECAPS_0; 1],
}
impl ::core::marker::Copy for DD_MORESURFACECAPS {}
impl ::core::clone::Clone for DD_MORESURFACECAPS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_MORESURFACECAPS_0 {
    pub ddsCapsEx: DDSCAPSEX,
    pub ddsCapsExAlt: DDSCAPSEX,
}
impl ::core::marker::Copy for DD_MORESURFACECAPS_0 {}
impl ::core::clone::Clone for DD_MORESURFACECAPS_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_MOTIONCOMPCALLBACKS {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub GetMoCompGuids: PDD_MOCOMPCB_GETGUIDS,
    pub GetMoCompFormats: PDD_MOCOMPCB_GETFORMATS,
    pub CreateMoComp: PDD_MOCOMPCB_CREATE,
    pub GetMoCompBuffInfo: PDD_MOCOMPCB_GETCOMPBUFFINFO,
    pub GetInternalMoCompInfo: PDD_MOCOMPCB_GETINTERNALINFO,
    pub BeginMoCompFrame: PDD_MOCOMPCB_BEGINFRAME,
    pub EndMoCompFrame: PDD_MOCOMPCB_ENDFRAME,
    pub RenderMoComp: PDD_MOCOMPCB_RENDER,
    pub QueryMoCompStatus: PDD_MOCOMPCB_QUERYSTATUS,
    pub DestroyMoComp: PDD_MOCOMPCB_DESTROY,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_MOTIONCOMPCALLBACKS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_MOTIONCOMPCALLBACKS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_MOTIONCOMP_LOCAL {
    pub lpDD: *mut DD_DIRECTDRAW_LOCAL,
    pub guid: ::windows_sys::core::GUID,
    pub dwUncompWidth: u32,
    pub dwUncompHeight: u32,
    pub ddUncompPixelFormat: DDPIXELFORMAT,
    pub dwDriverReserved1: u32,
    pub dwDriverReserved2: u32,
    pub dwDriverReserved3: u32,
    pub lpDriverReserved1: *mut ::core::ffi::c_void,
    pub lpDriverReserved2: *mut ::core::ffi::c_void,
    pub lpDriverReserved3: *mut ::core::ffi::c_void,
}
impl ::core::marker::Copy for DD_MOTIONCOMP_LOCAL {}
impl ::core::clone::Clone for DD_MOTIONCOMP_LOCAL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_NONLOCALVIDMEMCAPS {
    pub dwSize: u32,
    pub dwNLVBCaps: u32,
    pub dwNLVBCaps2: u32,
    pub dwNLVBCKeyCaps: u32,
    pub dwNLVBFXCaps: u32,
    pub dwNLVBRops: [u32; 8],
}
impl ::core::marker::Copy for DD_NONLOCALVIDMEMCAPS {}
impl ::core::clone::Clone for DD_NONLOCALVIDMEMCAPS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_NTCALLBACKS {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub FreeDriverMemory: PDD_FREEDRIVERMEMORY,
    pub SetExclusiveMode: PDD_SETEXCLUSIVEMODE,
    pub FlipToGDISurface: PDD_FLIPTOGDISURFACE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_NTCALLBACKS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_NTCALLBACKS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_NTPRIVATEDRIVERCAPS {
    pub dwSize: u32,
    pub dwPrivateCaps: u32,
}
impl ::core::marker::Copy for DD_NTPRIVATEDRIVERCAPS {}
impl ::core::clone::Clone for DD_NTPRIVATEDRIVERCAPS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(feature = "Win32_Graphics_Gdi")]
pub struct DD_PALETTECALLBACKS {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub DestroyPalette: PDD_PALCB_DESTROYPALETTE,
    pub SetEntries: PDD_PALCB_SETENTRIES,
}
#[cfg(feature = "Win32_Graphics_Gdi")]
impl ::core::marker::Copy for DD_PALETTECALLBACKS {}
#[cfg(feature = "Win32_Graphics_Gdi")]
impl ::core::clone::Clone for DD_PALETTECALLBACKS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_PALETTE_GLOBAL {
    pub dwReserved1: usize,
}
impl ::core::marker::Copy for DD_PALETTE_GLOBAL {}
impl ::core::clone::Clone for DD_PALETTE_GLOBAL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_PALETTE_LOCAL {
    pub dwReserved0: u32,
    pub dwReserved1: usize,
}
impl ::core::marker::Copy for DD_PALETTE_LOCAL {}
impl ::core::clone::Clone for DD_PALETTE_LOCAL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_QUERYMOCOMPSTATUSDATA {
    pub lpDD: *mut DD_DIRECTDRAW_LOCAL,
    pub lpMoComp: *mut DD_MOTIONCOMP_LOCAL,
    pub lpSurface: *mut DD_SURFACE_LOCAL,
    pub dwFlags: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_QUERYMOCOMPSTATUSDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_QUERYMOCOMPSTATUSDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_RENDERMOCOMPDATA {
    pub lpDD: *mut DD_DIRECTDRAW_LOCAL,
    pub lpMoComp: *mut DD_MOTIONCOMP_LOCAL,
    pub dwNumBuffers: u32,
    pub lpBufferInfo: *mut DDMOCOMPBUFFERINFO,
    pub dwFunction: u32,
    pub lpInputData: *mut ::core::ffi::c_void,
    pub dwInputDataSize: u32,
    pub lpOutputData: *mut ::core::ffi::c_void,
    pub dwOutputDataSize: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_RENDERMOCOMPDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_RENDERMOCOMPDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_SETCLIPLISTDATA {
    pub lpDD: *mut DD_DIRECTDRAW_GLOBAL,
    pub lpDDSurface: *mut DD_SURFACE_LOCAL,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub SetClipList: *mut ::core::ffi::c_void,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_SETCLIPLISTDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_SETCLIPLISTDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_SETCOLORKEYDATA {
    pub lpDD: *mut DD_DIRECTDRAW_GLOBAL,
    pub lpDDSurface: *mut DD_SURFACE_LOCAL,
    pub dwFlags: u32,
    pub ckNew: DDCOLORKEY,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub SetColorKey: *mut ::core::ffi::c_void,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_SETCOLORKEYDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_SETCOLORKEYDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(feature = "Win32_Graphics_Gdi")]
pub struct DD_SETENTRIESDATA {
    pub lpDD: *mut DD_DIRECTDRAW_GLOBAL,
    pub lpDDPalette: *mut DD_PALETTE_GLOBAL,
    pub dwBase: u32,
    pub dwNumEntries: u32,
    pub lpEntries: *mut super::Gdi::PALETTEENTRY,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub SetEntries: *mut ::core::ffi::c_void,
}
#[cfg(feature = "Win32_Graphics_Gdi")]
impl ::core::marker::Copy for DD_SETENTRIESDATA {}
#[cfg(feature = "Win32_Graphics_Gdi")]
impl ::core::clone::Clone for DD_SETENTRIESDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_SETEXCLUSIVEMODEDATA {
    pub lpDD: *mut DD_DIRECTDRAW_GLOBAL,
    pub dwEnterExcl: u32,
    pub dwReserved: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub SetExclusiveMode: *mut ::core::ffi::c_void,
}
impl ::core::marker::Copy for DD_SETEXCLUSIVEMODEDATA {}
impl ::core::clone::Clone for DD_SETEXCLUSIVEMODEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_SETOVERLAYPOSITIONDATA {
    pub lpDD: *mut DD_DIRECTDRAW_GLOBAL,
    pub lpDDSrcSurface: *mut DD_SURFACE_LOCAL,
    pub lpDDDestSurface: *mut DD_SURFACE_LOCAL,
    pub lXPos: i32,
    pub lYPos: i32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub SetOverlayPosition: *mut ::core::ffi::c_void,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_SETOVERLAYPOSITIONDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_SETOVERLAYPOSITIONDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_SETPALETTEDATA {
    pub lpDD: *mut DD_DIRECTDRAW_GLOBAL,
    pub lpDDSurface: *mut DD_SURFACE_LOCAL,
    pub lpDDPalette: *mut DD_PALETTE_GLOBAL,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub SetPalette: *mut ::core::ffi::c_void,
    pub Attach: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_SETPALETTEDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_SETPALETTEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_STEREOMODE {
    pub dwSize: u32,
    pub dwHeight: u32,
    pub dwWidth: u32,
    pub dwBpp: u32,
    pub dwRefreshRate: u32,
    pub bSupported: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_STEREOMODE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_STEREOMODE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_SURFACECALLBACKS {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub DestroySurface: PDD_SURFCB_DESTROYSURFACE,
    pub Flip: PDD_SURFCB_FLIP,
    pub SetClipList: PDD_SURFCB_SETCLIPLIST,
    pub Lock: PDD_SURFCB_LOCK,
    pub Unlock: PDD_SURFCB_UNLOCK,
    pub Blt: PDD_SURFCB_BLT,
    pub SetColorKey: PDD_SURFCB_SETCOLORKEY,
    pub AddAttachedSurface: PDD_SURFCB_ADDATTACHEDSURFACE,
    pub GetBltStatus: PDD_SURFCB_GETBLTSTATUS,
    pub GetFlipStatus: PDD_SURFCB_GETFLIPSTATUS,
    pub UpdateOverlay: PDD_SURFCB_UPDATEOVERLAY,
    pub SetOverlayPosition: PDD_SURFCB_SETOVERLAYPOSITION,
    pub reserved4: *mut ::core::ffi::c_void,
    pub SetPalette: PDD_SURFCB_SETPALETTE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_SURFACECALLBACKS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_SURFACECALLBACKS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_SURFACE_GLOBAL {
    pub Anonymous1: DD_SURFACE_GLOBAL_0,
    pub Anonymous2: DD_SURFACE_GLOBAL_1,
    pub fpVidMem: usize,
    pub Anonymous3: DD_SURFACE_GLOBAL_2,
    pub yHint: i32,
    pub xHint: i32,
    pub wHeight: u32,
    pub wWidth: u32,
    pub dwReserved1: usize,
    pub ddpfSurface: DDPIXELFORMAT,
    pub fpHeapOffset: usize,
    pub hCreatorProcess: super::super::Foundation::HANDLE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_SURFACE_GLOBAL {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_SURFACE_GLOBAL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub union DD_SURFACE_GLOBAL_0 {
    pub dwBlockSizeY: u32,
    pub lSlicePitch: i32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_SURFACE_GLOBAL_0 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_SURFACE_GLOBAL_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub union DD_SURFACE_GLOBAL_1 {
    pub lpVidMemHeap: *mut VIDEOMEMORY,
    pub dwBlockSizeX: u32,
    pub dwUserMemSize: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_SURFACE_GLOBAL_1 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_SURFACE_GLOBAL_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub union DD_SURFACE_GLOBAL_2 {
    pub lPitch: i32,
    pub dwLinearSize: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_SURFACE_GLOBAL_2 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_SURFACE_GLOBAL_2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_SURFACE_INT {
    pub lpLcl: *mut DD_SURFACE_LOCAL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_SURFACE_INT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_SURFACE_INT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_SURFACE_LOCAL {
    pub lpGbl: *mut DD_SURFACE_GLOBAL,
    pub dwFlags: u32,
    pub ddsCaps: DDSCAPS,
    pub dwReserved1: usize,
    pub Anonymous1: DD_SURFACE_LOCAL_0,
    pub Anonymous2: DD_SURFACE_LOCAL_1,
    pub lpSurfMore: *mut DD_SURFACE_MORE,
    pub lpAttachList: *mut DD_ATTACHLIST,
    pub lpAttachListFrom: *mut DD_ATTACHLIST,
    pub rcOverlaySrc: super::super::Foundation::RECT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_SURFACE_LOCAL {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_SURFACE_LOCAL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub union DD_SURFACE_LOCAL_0 {
    pub ddckCKSrcOverlay: DDCOLORKEY,
    pub ddckCKSrcBlt: DDCOLORKEY,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_SURFACE_LOCAL_0 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_SURFACE_LOCAL_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub union DD_SURFACE_LOCAL_1 {
    pub ddckCKDestOverlay: DDCOLORKEY,
    pub ddckCKDestBlt: DDCOLORKEY,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_SURFACE_LOCAL_1 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_SURFACE_LOCAL_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_SURFACE_MORE {
    pub dwMipMapCount: u32,
    pub lpVideoPort: *mut DD_VIDEOPORT_LOCAL,
    pub dwOverlayFlags: u32,
    pub ddsCapsEx: DDSCAPSEX,
    pub dwSurfaceHandle: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_SURFACE_MORE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_SURFACE_MORE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_SYNCSURFACEDATA {
    pub lpDD: *mut DD_DIRECTDRAW_LOCAL,
    pub lpDDSurface: *mut DD_SURFACE_LOCAL,
    pub dwSurfaceOffset: u32,
    pub fpLockPtr: usize,
    pub lPitch: i32,
    pub dwOverlayOffset: u32,
    pub dwDriverReserved1: u32,
    pub dwDriverReserved2: u32,
    pub dwDriverReserved3: u32,
    pub dwDriverReserved4: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_SYNCSURFACEDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_SYNCSURFACEDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_SYNCVIDEOPORTDATA {
    pub lpDD: *mut DD_DIRECTDRAW_LOCAL,
    pub lpVideoPort: *mut DD_VIDEOPORT_LOCAL,
    pub dwOriginOffset: u32,
    pub dwHeight: u32,
    pub dwVBIHeight: u32,
    pub dwDriverReserved1: u32,
    pub dwDriverReserved2: u32,
    pub dwDriverReserved3: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_SYNCVIDEOPORTDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_SYNCVIDEOPORTDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_UNLOCKDATA {
    pub lpDD: *mut DD_DIRECTDRAW_GLOBAL,
    pub lpDDSurface: *mut DD_SURFACE_LOCAL,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub Unlock: *mut ::core::ffi::c_void,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_UNLOCKDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_UNLOCKDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_UPDATENONLOCALHEAPDATA {
    pub lpDD: *mut DD_DIRECTDRAW_GLOBAL,
    pub dwHeap: u32,
    pub fpGARTLin: usize,
    pub fpGARTDev: usize,
    pub ulPolicyMaxBytes: usize,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub UpdateNonLocalHeap: *mut ::core::ffi::c_void,
}
impl ::core::marker::Copy for DD_UPDATENONLOCALHEAPDATA {}
impl ::core::clone::Clone for DD_UPDATENONLOCALHEAPDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_UPDATEOVERLAYDATA {
    pub lpDD: *mut DD_DIRECTDRAW_GLOBAL,
    pub lpDDDestSurface: *mut DD_SURFACE_LOCAL,
    pub rDest: super::super::Foundation::RECTL,
    pub lpDDSrcSurface: *mut DD_SURFACE_LOCAL,
    pub rSrc: super::super::Foundation::RECTL,
    pub dwFlags: u32,
    pub overlayFX: DDOVERLAYFX,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub UpdateOverlay: *mut ::core::ffi::c_void,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_UPDATEOVERLAYDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_UPDATEOVERLAYDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_UPDATEVPORTDATA {
    pub lpDD: *mut DD_DIRECTDRAW_LOCAL,
    pub lpVideoPort: *mut DD_VIDEOPORT_LOCAL,
    pub lplpDDSurface: *mut *mut DD_SURFACE_INT,
    pub lplpDDVBISurface: *mut *mut DD_SURFACE_INT,
    pub lpVideoInfo: *mut DDVIDEOPORTINFO,
    pub dwFlags: u32,
    pub dwNumAutoflip: u32,
    pub dwNumVBIAutoflip: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub UpdateVideoPort: *mut ::core::ffi::c_void,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_UPDATEVPORTDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_UPDATEVPORTDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_VIDEOPORTCALLBACKS {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub CanCreateVideoPort: PDD_VPORTCB_CANCREATEVIDEOPORT,
    pub CreateVideoPort: PDD_VPORTCB_CREATEVIDEOPORT,
    pub FlipVideoPort: PDD_VPORTCB_FLIP,
    pub GetVideoPortBandwidth: PDD_VPORTCB_GETBANDWIDTH,
    pub GetVideoPortInputFormats: PDD_VPORTCB_GETINPUTFORMATS,
    pub GetVideoPortOutputFormats: PDD_VPORTCB_GETOUTPUTFORMATS,
    pub lpReserved1: *mut ::core::ffi::c_void,
    pub GetVideoPortField: PDD_VPORTCB_GETFIELD,
    pub GetVideoPortLine: PDD_VPORTCB_GETLINE,
    pub GetVideoPortConnectInfo: PDD_VPORTCB_GETVPORTCONNECT,
    pub DestroyVideoPort: PDD_VPORTCB_DESTROYVPORT,
    pub GetVideoPortFlipStatus: PDD_VPORTCB_GETFLIPSTATUS,
    pub UpdateVideoPort: PDD_VPORTCB_UPDATE,
    pub WaitForVideoPortSync: PDD_VPORTCB_WAITFORSYNC,
    pub GetVideoSignalStatus: PDD_VPORTCB_GETSIGNALSTATUS,
    pub ColorControl: PDD_VPORTCB_COLORCONTROL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_VIDEOPORTCALLBACKS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_VIDEOPORTCALLBACKS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_VIDEOPORT_LOCAL {
    pub lpDD: *mut DD_DIRECTDRAW_LOCAL,
    pub ddvpDesc: DDVIDEOPORTDESC,
    pub ddvpInfo: DDVIDEOPORTINFO,
    pub lpSurface: *mut DD_SURFACE_INT,
    pub lpVBISurface: *mut DD_SURFACE_INT,
    pub dwNumAutoflip: u32,
    pub dwNumVBIAutoflip: u32,
    pub dwReserved1: usize,
    pub dwReserved2: usize,
    pub dwReserved3: usize,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_VIDEOPORT_LOCAL {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_VIDEOPORT_LOCAL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_VPORTCOLORDATA {
    pub lpDD: *mut DD_DIRECTDRAW_LOCAL,
    pub lpVideoPort: *mut DD_VIDEOPORT_LOCAL,
    pub dwFlags: u32,
    pub lpColorData: *mut DDCOLORCONTROL,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub ColorControl: *mut ::core::ffi::c_void,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_VPORTCOLORDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_VPORTCOLORDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DD_WAITFORVERTICALBLANKDATA {
    pub lpDD: *mut DD_DIRECTDRAW_GLOBAL,
    pub dwFlags: u32,
    pub bIsInVB: u32,
    pub hEvent: usize,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub WaitForVerticalBlank: *mut ::core::ffi::c_void,
}
impl ::core::marker::Copy for DD_WAITFORVERTICALBLANKDATA {}
impl ::core::clone::Clone for DD_WAITFORVERTICALBLANKDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DD_WAITFORVPORTSYNCDATA {
    pub lpDD: *mut DD_DIRECTDRAW_LOCAL,
    pub lpVideoPort: *mut DD_VIDEOPORT_LOCAL,
    pub dwFlags: u32,
    pub dwLine: u32,
    pub dwTimeOut: u32,
    pub ddRVal: ::windows_sys::core::HRESULT,
    pub UpdateVideoPort: *mut ::core::ffi::c_void,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DD_WAITFORVPORTSYNCDATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DD_WAITFORVPORTSYNCDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DXAPI_INTERFACE {
    pub Size: u16,
    pub Version: u16,
    pub Context: *mut ::core::ffi::c_void,
    pub InterfaceReference: *mut ::core::ffi::c_void,
    pub InterfaceDereference: *mut ::core::ffi::c_void,
    pub DxGetIrqInfo: PDX_GETIRQINFO,
    pub DxEnableIrq: PDX_ENABLEIRQ,
    pub DxSkipNextField: PDX_SKIPNEXTFIELD,
    pub DxBobNextField: PDX_BOBNEXTFIELD,
    pub DxSetState: PDX_SETSTATE,
    pub DxLock: PDX_LOCK,
    pub DxFlipOverlay: PDX_FLIPOVERLAY,
    pub DxFlipVideoPort: PDX_FLIPVIDEOPORT,
    pub DxGetPolarity: PDX_GETPOLARITY,
    pub DxGetCurrentAutoflip: PDX_GETCURRENTAUTOFLIP,
    pub DxGetPreviousAutoflip: PDX_GETPREVIOUSAUTOFLIP,
    pub DxTransfer: PDX_TRANSFER,
    pub DxGetTransferStatus: PDX_GETTRANSFERSTATUS,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DXAPI_INTERFACE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DXAPI_INTERFACE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct DX_IRQDATA {
    pub dwIrqFlags: u32,
}
impl ::core::marker::Copy for DX_IRQDATA {}
impl ::core::clone::Clone for DX_IRQDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct HEAPALIAS {
    pub fpVidMem: usize,
    pub lpAlias: *mut ::core::ffi::c_void,
    pub dwAliasSize: u32,
}
impl ::core::marker::Copy for HEAPALIAS {}
impl ::core::clone::Clone for HEAPALIAS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct HEAPALIASINFO {
    pub dwRefCnt: u32,
    pub dwFlags: u32,
    pub dwNumHeaps: u32,
    pub lpAliases: *mut HEAPALIAS,
}
impl ::core::marker::Copy for HEAPALIASINFO {}
impl ::core::clone::Clone for HEAPALIASINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct HEAPALIGNMENT {
    pub dwSize: u32,
    pub ddsCaps: DDSCAPS,
    pub dwReserved: u32,
    pub ExecuteBuffer: SURFACEALIGNMENT,
    pub Overlay: SURFACEALIGNMENT,
    pub Texture: SURFACEALIGNMENT,
    pub ZBuffer: SURFACEALIGNMENT,
    pub AlphaBuffer: SURFACEALIGNMENT,
    pub Offscreen: SURFACEALIGNMENT,
    pub FlipTarget: SURFACEALIGNMENT,
}
impl ::core::marker::Copy for HEAPALIGNMENT {}
impl ::core::clone::Clone for HEAPALIGNMENT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct IUNKNOWN_LIST {
    pub lpLink: *mut IUNKNOWN_LIST,
    pub lpGuid: *mut ::windows_sys::core::GUID,
    pub lpIUnknown: ::windows_sys::core::IUnknown,
}
impl ::core::marker::Copy for IUNKNOWN_LIST {}
impl ::core::clone::Clone for IUNKNOWN_LIST {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct MDL {
    pub MdlNext: *mut MDL,
    pub MdlSize: i16,
    pub MdlFlags: i16,
    pub Process: *mut MDL_0,
    pub lpMappedSystemVa: *mut u32,
    pub lpStartVa: *mut u32,
    pub ByteCount: u32,
    pub ByteOffset: u32,
}
impl ::core::marker::Copy for MDL {}
impl ::core::clone::Clone for MDL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
pub struct MDL_0(pub u8);
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct PROCESS_LIST {
    pub lpLink: *mut PROCESS_LIST,
    pub dwProcessId: u32,
    pub dwRefCnt: u32,
    pub dwAlphaDepth: u32,
    pub dwZDepth: u32,
}
impl ::core::marker::Copy for PROCESS_LIST {}
impl ::core::clone::Clone for PROCESS_LIST {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct SURFACEALIGNMENT {
    pub Anonymous: SURFACEALIGNMENT_0,
}
impl ::core::marker::Copy for SURFACEALIGNMENT {}
impl ::core::clone::Clone for SURFACEALIGNMENT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub union SURFACEALIGNMENT_0 {
    pub Linear: SURFACEALIGNMENT_0_0,
    pub Rectangular: SURFACEALIGNMENT_0_1,
}
impl ::core::marker::Copy for SURFACEALIGNMENT_0 {}
impl ::core::clone::Clone for SURFACEALIGNMENT_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct SURFACEALIGNMENT_0_0 {
    pub dwStartAlignment: u32,
    pub dwPitchAlignment: u32,
    pub dwFlags: u32,
    pub dwReserved2: u32,
}
impl ::core::marker::Copy for SURFACEALIGNMENT_0_0 {}
impl ::core::clone::Clone for SURFACEALIGNMENT_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct SURFACEALIGNMENT_0_1 {
    pub dwXAlignment: u32,
    pub dwYAlignment: u32,
    pub dwFlags: u32,
    pub dwReserved2: u32,
}
impl ::core::marker::Copy for SURFACEALIGNMENT_0_1 {}
impl ::core::clone::Clone for SURFACEALIGNMENT_0_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct VIDEOMEMORY {
    pub dwFlags: u32,
    pub fpStart: usize,
    pub Anonymous1: VIDEOMEMORY_0,
    pub ddsCaps: DDSCAPS,
    pub ddsCapsAlt: DDSCAPS,
    pub Anonymous2: VIDEOMEMORY_1,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for VIDEOMEMORY {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for VIDEOMEMORY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub union VIDEOMEMORY_0 {
    pub fpEnd: usize,
    pub dwWidth: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for VIDEOMEMORY_0 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for VIDEOMEMORY_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub union VIDEOMEMORY_1 {
    pub lpHeap: *mut VMEMHEAP,
    pub dwHeight: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for VIDEOMEMORY_1 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for VIDEOMEMORY_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub struct VIDEOMEMORYINFO {
    pub fpPrimary: usize,
    pub dwFlags: u32,
    pub dwDisplayWidth: u32,
    pub dwDisplayHeight: u32,
    pub lDisplayPitch: i32,
    pub ddpfDisplay: DDPIXELFORMAT,
    pub dwOffscreenAlign: u32,
    pub dwOverlayAlign: u32,
    pub dwTextureAlign: u32,
    pub dwZBufferAlign: u32,
    pub dwAlphaAlign: u32,
    pub pvPrimary: *mut ::core::ffi::c_void,
}
impl ::core::marker::Copy for VIDEOMEMORYINFO {}
impl ::core::clone::Clone for VIDEOMEMORYINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct VIDMEM {
    pub dwFlags: u32,
    pub fpStart: usize,
    pub Anonymous1: VIDMEM_0,
    pub ddsCaps: DDSCAPS,
    pub ddsCapsAlt: DDSCAPS,
    pub Anonymous2: VIDMEM_1,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for VIDMEM {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for VIDMEM {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub union VIDMEM_0 {
    pub fpEnd: usize,
    pub dwWidth: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for VIDMEM_0 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for VIDMEM_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub union VIDMEM_1 {
    pub lpHeap: *mut VMEMHEAP,
    pub dwHeight: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for VIDMEM_1 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for VIDMEM_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct VIDMEMINFO {
    pub fpPrimary: usize,
    pub dwFlags: u32,
    pub dwDisplayWidth: u32,
    pub dwDisplayHeight: u32,
    pub lDisplayPitch: i32,
    pub ddpfDisplay: DDPIXELFORMAT,
    pub dwOffscreenAlign: u32,
    pub dwOverlayAlign: u32,
    pub dwTextureAlign: u32,
    pub dwZBufferAlign: u32,
    pub dwAlphaAlign: u32,
    pub dwNumHeaps: u32,
    pub pvmList: *mut VIDMEM,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for VIDMEMINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for VIDMEMINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct VMEMHEAP {
    pub dwFlags: u32,
    pub stride: u32,
    pub freeList: *mut ::core::ffi::c_void,
    pub allocList: *mut ::core::ffi::c_void,
    pub dwTotalSize: u32,
    pub fpGARTLin: usize,
    pub fpGARTDev: usize,
    pub dwCommitedSize: u32,
    pub dwCoalesceCount: u32,
    pub Alignment: HEAPALIGNMENT,
    pub ddsCapsEx: DDSCAPSEX,
    pub ddsCapsExAlt: DDSCAPSEX,
    pub liPhysAGPBase: i64,
    pub hdevAGP: super::super::Foundation::HANDLE,
    pub pvPhysRsrv: *mut ::core::ffi::c_void,
    pub pAgpCommitMask: *mut u8,
    pub dwAgpCommitMaskSize: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for VMEMHEAP {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for VMEMHEAP {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct VMEML {
    pub next: *mut VMEML,
    pub ptr: usize,
    pub size: u32,
    pub bDiscardable: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for VMEML {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for VMEML {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct VMEMR {
    pub next: *mut VMEMR,
    pub prev: *mut VMEMR,
    pub pUp: *mut VMEMR,
    pub pDown: *mut VMEMR,
    pub pLeft: *mut VMEMR,
    pub pRight: *mut VMEMR,
    pub ptr: usize,
    pub size: u32,
    pub x: u32,
    pub y: u32,
    pub cx: u32,
    pub cy: u32,
    pub flags: u32,
    pub pBits: usize,
    pub bDiscardable: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for VMEMR {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for VMEMR {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
pub struct _DDFXROP(pub u8);
#[repr(C)]
pub struct _DD_DESTROYDRIVERDATA(pub u8);
#[repr(C)]
pub struct _DD_GETVPORTAUTOFLIPSURFACEDATA(pub u8);
#[repr(C)]
pub struct _DD_SETMODEDATA(pub u8);
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type LPCLIPPERCALLBACK = ::core::option::Option<unsafe extern "system" fn(lpddclipper: IDirectDrawClipper, hwnd: super::super::Foundation::HWND, code: u32, lpcontext: *mut ::core::ffi::c_void) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type LPDD32BITDRIVERINIT = ::core::option::Option<unsafe extern "system" fn(dwcontext: u32) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type LPDDENUMCALLBACKA = ::core::option::Option<unsafe extern "system" fn(param0: *mut ::windows_sys::core::GUID, param1: ::windows_sys::core::PCSTR, param2: ::windows_sys::core::PCSTR, param3: *mut ::core::ffi::c_void) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDENUMCALLBACKEXA = ::core::option::Option<unsafe extern "system" fn(param0: *mut ::windows_sys::core::GUID, param1: ::windows_sys::core::PCSTR, param2: ::windows_sys::core::PCSTR, param3: *mut ::core::ffi::c_void, param4: super::Gdi::HMONITOR) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDENUMCALLBACKEXW = ::core::option::Option<unsafe extern "system" fn(param0: *mut ::windows_sys::core::GUID, param1: ::windows_sys::core::PCWSTR, param2: ::windows_sys::core::PCWSTR, param3: *mut ::core::ffi::c_void, param4: super::Gdi::HMONITOR) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type LPDDENUMCALLBACKW = ::core::option::Option<unsafe extern "system" fn(param0: *mut ::windows_sys::core::GUID, param1: ::windows_sys::core::PCWSTR, param2: ::windows_sys::core::PCWSTR, param3: *mut ::core::ffi::c_void) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type LPDDENUMMODESCALLBACK = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDSURFACEDESC, param1: *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type LPDDENUMMODESCALLBACK2 = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDSURFACEDESC2, param1: *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type LPDDENUMSURFACESCALLBACK = ::core::option::Option<unsafe extern "system" fn(param0: IDirectDrawSurface, param1: *mut DDSURFACEDESC, param2: *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type LPDDENUMSURFACESCALLBACK2 = ::core::option::Option<unsafe extern "system" fn(param0: IDirectDrawSurface4, param1: *mut DDSURFACEDESC2, param2: *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type LPDDENUMSURFACESCALLBACK7 = ::core::option::Option<unsafe extern "system" fn(param0: IDirectDrawSurface7, param1: *mut DDSURFACEDESC2, param2: *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type LPDDENUMVIDEOCALLBACK = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDVIDEOPORTCAPS, param1: *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type LPDDGAMMACALIBRATORPROC = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDGAMMARAMP, param1: *mut u8) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALCOLORCB_COLORCONTROL = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_COLORCONTROLDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALEXEBUFCB_CANCREATEEXEBUF = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_CANCREATESURFACEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALEXEBUFCB_CREATEEXEBUF = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_CREATESURFACEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALEXEBUFCB_DESTROYEXEBUF = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_DESTROYSURFACEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALEXEBUFCB_LOCKEXEBUF = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_LOCKDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALEXEBUFCB_UNLOCKEXEBUF = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_UNLOCKDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALKERNELCB_SYNCSURFACE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_SYNCSURFACEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALKERNELCB_SYNCVIDEOPORT = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_SYNCVIDEOPORTDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALMOCOMPCB_BEGINFRAME = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_BEGINMOCOMPFRAMEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALMOCOMPCB_CREATE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_CREATEMOCOMPDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALMOCOMPCB_DESTROY = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_DESTROYMOCOMPDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALMOCOMPCB_ENDFRAME = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_ENDMOCOMPFRAMEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALMOCOMPCB_GETCOMPBUFFINFO = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_GETMOCOMPCOMPBUFFDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALMOCOMPCB_GETFORMATS = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_GETMOCOMPFORMATSDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALMOCOMPCB_GETGUIDS = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_GETMOCOMPGUIDSDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALMOCOMPCB_GETINTERNALINFO = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_GETINTERNALMOCOMPDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALMOCOMPCB_QUERYSTATUS = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_QUERYMOCOMPSTATUSDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALMOCOMPCB_RENDER = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_RENDERMOCOMPDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALPALCB_DESTROYPALETTE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_DESTROYPALETTEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALPALCB_SETENTRIES = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_SETENTRIESDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALSURFCB_ADDATTACHEDSURFACE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_ADDATTACHEDSURFACEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALSURFCB_BLT = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_BLTDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALSURFCB_DESTROYSURFACE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_DESTROYSURFACEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALSURFCB_FLIP = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_FLIPDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALSURFCB_GETBLTSTATUS = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_GETBLTSTATUSDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALSURFCB_GETFLIPSTATUS = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_GETFLIPSTATUSDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALSURFCB_LOCK = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_LOCKDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALSURFCB_SETCLIPLIST = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_SETCLIPLISTDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALSURFCB_SETCOLORKEY = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_SETCOLORKEYDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALSURFCB_SETOVERLAYPOSITION = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_SETOVERLAYPOSITIONDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALSURFCB_SETPALETTE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_SETPALETTEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALSURFCB_UNLOCK = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_UNLOCKDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALSURFCB_UPDATEOVERLAY = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_UPDATEOVERLAYDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALVPORTCB_CANCREATEVIDEOPORT = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_CANCREATEVPORTDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALVPORTCB_COLORCONTROL = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_VPORTCOLORDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALVPORTCB_CREATEVIDEOPORT = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_CREATEVPORTDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALVPORTCB_DESTROYVPORT = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_DESTROYVPORTDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALVPORTCB_FLIP = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_FLIPVPORTDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALVPORTCB_GETBANDWIDTH = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_GETVPORTBANDWIDTHDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALVPORTCB_GETFIELD = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_GETVPORTFIELDDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALVPORTCB_GETFLIPSTATUS = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_GETVPORTFLIPSTATUSDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALVPORTCB_GETINPUTFORMATS = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_GETVPORTINPUTFORMATDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALVPORTCB_GETLINE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_GETVPORTLINEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALVPORTCB_GETOUTPUTFORMATS = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_GETVPORTOUTPUTFORMATDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALVPORTCB_GETSIGNALSTATUS = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_GETVPORTSIGNALDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALVPORTCB_GETVPORTCONNECT = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_GETVPORTCONNECTDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALVPORTCB_UPDATE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_UPDATEVPORTDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHALVPORTCB_WAITFORSYNC = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_WAITFORVPORTSYNCDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHAL_CANCREATESURFACE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_CANCREATESURFACEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHAL_CREATEPALETTE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_CREATEPALETTEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHAL_CREATESURFACE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_CREATESURFACEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHAL_CREATESURFACEEX = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_CREATESURFACEEXDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHAL_DESTROYDDLOCAL = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_DESTROYDDLOCALDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHAL_DESTROYDRIVER = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_DESTROYDRIVERDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHAL_FLIPTOGDISURFACE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_FLIPTOGDISURFACEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHAL_GETAVAILDRIVERMEMORY = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_GETAVAILDRIVERMEMORYDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type LPDDHAL_GETDRIVERINFO = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_GETDRIVERINFODATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type LPDDHAL_GETDRIVERSTATE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_GETDRIVERSTATEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type LPDDHAL_GETHEAPALIGNMENT = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_GETHEAPALIGNMENTDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHAL_GETSCANLINE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_GETSCANLINEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHAL_SETCOLORKEY = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_DRVSETCOLORKEYDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHAL_SETEXCLUSIVEMODE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_SETEXCLUSIVEMODEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHAL_SETINFO = ::core::option::Option<unsafe extern "system" fn(lpddhalinfo: *mut DDHALINFO, reset: super::super::Foundation::BOOL) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHAL_SETMODE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_SETMODEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHAL_UPDATENONLOCALHEAP = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_UPDATENONLOCALHEAPDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHAL_VIDMEMALLOC = ::core::option::Option<unsafe extern "system" fn(lpdd: *mut DDRAWI_DIRECTDRAW_GBL, heap: i32, dwwidth: u32, dwheight: u32) -> usize>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHAL_VIDMEMFREE = ::core::option::Option<unsafe extern "system" fn(lpdd: *mut DDRAWI_DIRECTDRAW_GBL, heap: i32, fpmem: usize)>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHAL_WAITFORVERTICALBLANK = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDHAL_WAITFORVERTICALBLANKDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDDHEL_INIT = ::core::option::Option<unsafe extern "system" fn(param0: *mut DDRAWI_DIRECTDRAW_GBL, param1: super::super::Foundation::BOOL) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDIRECTDRAWENUMERATEEXA = ::core::option::Option<unsafe extern "system" fn(lpcallback: LPDDENUMCALLBACKEXA, lpcontext: *mut ::core::ffi::c_void, dwflags: u32) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPDIRECTDRAWENUMERATEEXW = ::core::option::Option<unsafe extern "system" fn(lpcallback: LPDDENUMCALLBACKEXW, lpcontext: *mut ::core::ffi::c_void, dwflags: u32) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_ALPHABLT = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_BLTDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDD_CANCREATESURFACE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_CANCREATESURFACEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_COLORCB_COLORCONTROL = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_COLORCONTROLDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type PDD_CREATEPALETTE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_CREATEPALETTEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_CREATESURFACE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_CREATESURFACEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_CREATESURFACEEX = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_CREATESURFACEEXDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDD_DESTROYDDLOCAL = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_DESTROYDDLOCALDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDD_DESTROYDRIVER = ::core::option::Option<unsafe extern "system" fn(param0: *mut _DD_DESTROYDRIVERDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDD_FLIPTOGDISURFACE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_FLIPTOGDISURFACEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_FREEDRIVERMEMORY = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_FREEDRIVERMEMORYDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDD_GETAVAILDRIVERMEMORY = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_GETAVAILDRIVERMEMORYDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDD_GETDRIVERINFO = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_GETDRIVERINFODATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDD_GETDRIVERSTATE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_GETDRIVERSTATEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDD_GETSCANLINE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_GETSCANLINEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_KERNELCB_SYNCSURFACE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_SYNCSURFACEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_KERNELCB_SYNCVIDEOPORT = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_SYNCVIDEOPORTDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_MAPMEMORY = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_MAPMEMORYDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_MOCOMPCB_BEGINFRAME = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_BEGINMOCOMPFRAMEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDD_MOCOMPCB_CREATE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_CREATEMOCOMPDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDD_MOCOMPCB_DESTROY = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_DESTROYMOCOMPDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDD_MOCOMPCB_ENDFRAME = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_ENDMOCOMPFRAMEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDD_MOCOMPCB_GETCOMPBUFFINFO = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_GETMOCOMPCOMPBUFFDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDD_MOCOMPCB_GETFORMATS = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_GETMOCOMPFORMATSDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDD_MOCOMPCB_GETGUIDS = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_GETMOCOMPGUIDSDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDD_MOCOMPCB_GETINTERNALINFO = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_GETINTERNALMOCOMPDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_MOCOMPCB_QUERYSTATUS = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_QUERYMOCOMPSTATUSDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_MOCOMPCB_RENDER = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_RENDERMOCOMPDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDD_PALCB_DESTROYPALETTE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_DESTROYPALETTEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(feature = "Win32_Graphics_Gdi")]
pub type PDD_PALCB_SETENTRIES = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_SETENTRIESDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_SETCOLORKEY = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_DRVSETCOLORKEYDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDD_SETEXCLUSIVEMODE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_SETEXCLUSIVEMODEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDD_SETMODE = ::core::option::Option<unsafe extern "system" fn(param0: *mut _DD_SETMODEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_SURFCB_ADDATTACHEDSURFACE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_ADDATTACHEDSURFACEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_SURFCB_BLT = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_BLTDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_SURFCB_DESTROYSURFACE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_DESTROYSURFACEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_SURFCB_FLIP = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_FLIPDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_SURFCB_GETBLTSTATUS = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_GETBLTSTATUSDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_SURFCB_GETFLIPSTATUS = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_GETFLIPSTATUSDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_SURFCB_LOCK = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_LOCKDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_SURFCB_SETCLIPLIST = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_SETCLIPLISTDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_SURFCB_SETCOLORKEY = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_SETCOLORKEYDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_SURFCB_SETOVERLAYPOSITION = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_SETOVERLAYPOSITIONDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_SURFCB_SETPALETTE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_SETPALETTEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_SURFCB_UNLOCK = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_UNLOCKDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_SURFCB_UPDATEOVERLAY = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_UPDATEOVERLAYDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDD_VPORTCB_CANCREATEVIDEOPORT = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_CANCREATEVPORTDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_VPORTCB_COLORCONTROL = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_VPORTCOLORDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_VPORTCB_CREATEVIDEOPORT = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_CREATEVPORTDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_VPORTCB_DESTROYVPORT = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_DESTROYVPORTDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_VPORTCB_FLIP = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_FLIPVPORTDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDD_VPORTCB_GETAUTOFLIPSURF = ::core::option::Option<unsafe extern "system" fn(param0: *mut _DD_GETVPORTAUTOFLIPSURFACEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_VPORTCB_GETBANDWIDTH = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_GETVPORTBANDWIDTHDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_VPORTCB_GETFIELD = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_GETVPORTFIELDDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDD_VPORTCB_GETFLIPSTATUS = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_GETVPORTFLIPSTATUSDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_VPORTCB_GETINPUTFORMATS = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_GETVPORTINPUTFORMATDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_VPORTCB_GETLINE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_GETVPORTLINEDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_VPORTCB_GETOUTPUTFORMATS = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_GETVPORTOUTPUTFORMATDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_VPORTCB_GETSIGNALSTATUS = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_GETVPORTSIGNALDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDD_VPORTCB_GETVPORTCONNECT = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_GETVPORTCONNECTDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_VPORTCB_UPDATE = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_UPDATEVPORTDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDD_VPORTCB_WAITFORSYNC = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_WAITFORVPORTSYNCDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDD_WAITFORVERTICALBLANK = ::core::option::Option<unsafe extern "system" fn(param0: *mut DD_WAITFORVERTICALBLANKDATA) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDX_BOBNEXTFIELD = ::core::option::Option<unsafe extern "system" fn(param0: *mut ::core::ffi::c_void, param1: *mut DDBOBNEXTFIELDINFO, param2: *mut ::core::ffi::c_void) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDX_ENABLEIRQ = ::core::option::Option<unsafe extern "system" fn(param0: *mut ::core::ffi::c_void, param1: *mut DDENABLEIRQINFO, param2: *mut ::core::ffi::c_void) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDX_FLIPOVERLAY = ::core::option::Option<unsafe extern "system" fn(param0: *mut ::core::ffi::c_void, param1: *mut DDFLIPOVERLAYINFO, param2: *mut ::core::ffi::c_void) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDX_FLIPVIDEOPORT = ::core::option::Option<unsafe extern "system" fn(param0: *mut ::core::ffi::c_void, param1: *mut DDFLIPVIDEOPORTINFO, param2: *mut ::core::ffi::c_void) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDX_GETCURRENTAUTOFLIP = ::core::option::Option<unsafe extern "system" fn(param0: *mut ::core::ffi::c_void, param1: *mut DDGETCURRENTAUTOFLIPININFO, param2: *mut DDGETCURRENTAUTOFLIPOUTINFO) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDX_GETIRQINFO = ::core::option::Option<unsafe extern "system" fn(param0: *mut ::core::ffi::c_void, param1: *mut ::core::ffi::c_void, param2: *mut DDGETIRQINFO) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDX_GETPOLARITY = ::core::option::Option<unsafe extern "system" fn(param0: *mut ::core::ffi::c_void, param1: *mut DDGETPOLARITYININFO, param2: *mut DDGETPOLARITYOUTINFO) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDX_GETPREVIOUSAUTOFLIP = ::core::option::Option<unsafe extern "system" fn(param0: *mut ::core::ffi::c_void, param1: *mut DDGETPREVIOUSAUTOFLIPININFO, param2: *mut DDGETPREVIOUSAUTOFLIPOUTINFO) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDX_GETTRANSFERSTATUS = ::core::option::Option<unsafe extern "system" fn(param0: *mut ::core::ffi::c_void, param1: *mut ::core::ffi::c_void, param2: *mut DDGETTRANSFERSTATUSOUTINFO) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDX_IRQCALLBACK = ::core::option::Option<unsafe extern "system" fn(pirqdata: *mut DX_IRQDATA)>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDX_LOCK = ::core::option::Option<unsafe extern "system" fn(param0: *mut ::core::ffi::c_void, param1: *mut DDLOCKININFO, param2: *mut DDLOCKOUTINFO) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PDX_SETSTATE = ::core::option::Option<unsafe extern "system" fn(param0: *mut ::core::ffi::c_void, param1: *mut DDSETSTATEININFO, param2: *mut DDSETSTATEOUTINFO) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDX_SKIPNEXTFIELD = ::core::option::Option<unsafe extern "system" fn(param0: *mut ::core::ffi::c_void, param1: *mut DDSKIPNEXTFIELDINFO, param2: *mut ::core::ffi::c_void) -> u32>;
#[doc = "*Required features: `\"Win32_Graphics_DirectDraw\"`*"]
pub type PDX_TRANSFER = ::core::option::Option<unsafe extern "system" fn(param0: *mut ::core::ffi::c_void, param1: *mut DDTRANSFERININFO, param2: *mut DDTRANSFEROUTINFO) -> u32>;

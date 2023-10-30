#[cfg_attr(windows, link(name = "windows"))]
extern "system" {
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Graphics_Direct3D\"`*"]
    #[cfg(feature = "Win32_Graphics_Direct3D")]
    pub fn D3D10CompileEffectFromMemory(pdata: *const ::core::ffi::c_void, datalength: usize, psrcfilename: ::windows_sys::core::PCSTR, pdefines: *const super::Direct3D::D3D_SHADER_MACRO, pinclude: super::Direct3D::ID3DInclude, hlslflags: u32, fxflags: u32, ppcompiledeffect: *mut super::Direct3D::ID3DBlob, pperrors: *mut super::Direct3D::ID3DBlob) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Graphics_Direct3D\"`*"]
    #[cfg(feature = "Win32_Graphics_Direct3D")]
    pub fn D3D10CompileShader(psrcdata: ::windows_sys::core::PCSTR, srcdatasize: usize, pfilename: ::windows_sys::core::PCSTR, pdefines: *const super::Direct3D::D3D_SHADER_MACRO, pinclude: super::Direct3D::ID3DInclude, pfunctionname: ::windows_sys::core::PCSTR, pprofile: ::windows_sys::core::PCSTR, flags: u32, ppshader: *mut super::Direct3D::ID3DBlob, pperrormsgs: *mut super::Direct3D::ID3DBlob) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Graphics_Direct3D\"`*"]
    #[cfg(feature = "Win32_Graphics_Direct3D")]
    pub fn D3D10CreateBlob(numbytes: usize, ppbuffer: *mut super::Direct3D::ID3DBlob) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Dxgi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Dxgi"))]
    pub fn D3D10CreateDevice(padapter: super::Dxgi::IDXGIAdapter, drivertype: D3D10_DRIVER_TYPE, software: super::super::Foundation::HINSTANCE, flags: u32, sdkversion: u32, ppdevice: *mut ID3D10Device) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Dxgi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Dxgi"))]
    pub fn D3D10CreateDevice1(padapter: super::Dxgi::IDXGIAdapter, drivertype: D3D10_DRIVER_TYPE, software: super::super::Foundation::HINSTANCE, flags: u32, hardwarelevel: D3D10_FEATURE_LEVEL1, sdkversion: u32, ppdevice: *mut ID3D10Device1) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Dxgi_Common"))]
    pub fn D3D10CreateDeviceAndSwapChain(padapter: super::Dxgi::IDXGIAdapter, drivertype: D3D10_DRIVER_TYPE, software: super::super::Foundation::HINSTANCE, flags: u32, sdkversion: u32, pswapchaindesc: *const super::Dxgi::DXGI_SWAP_CHAIN_DESC, ppswapchain: *mut super::Dxgi::IDXGISwapChain, ppdevice: *mut ID3D10Device) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Dxgi_Common"))]
    pub fn D3D10CreateDeviceAndSwapChain1(padapter: super::Dxgi::IDXGIAdapter, drivertype: D3D10_DRIVER_TYPE, software: super::super::Foundation::HINSTANCE, flags: u32, hardwarelevel: D3D10_FEATURE_LEVEL1, sdkversion: u32, pswapchaindesc: *const super::Dxgi::DXGI_SWAP_CHAIN_DESC, ppswapchain: *mut super::Dxgi::IDXGISwapChain, ppdevice: *mut ID3D10Device1) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
    pub fn D3D10CreateEffectFromMemory(pdata: *const ::core::ffi::c_void, datalength: usize, fxflags: u32, pdevice: ID3D10Device, peffectpool: ID3D10EffectPool, ppeffect: *mut ID3D10Effect) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
    pub fn D3D10CreateEffectPoolFromMemory(pdata: *const ::core::ffi::c_void, datalength: usize, fxflags: u32, pdevice: ID3D10Device, ppeffectpool: *mut ID3D10EffectPool) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
    pub fn D3D10CreateStateBlock(pdevice: ID3D10Device, pstateblockmask: *const D3D10_STATE_BLOCK_MASK, ppstateblock: *mut ID3D10StateBlock) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Direct3D\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct3D"))]
    pub fn D3D10DisassembleEffect(peffect: ID3D10Effect, enablecolorcode: super::super::Foundation::BOOL, ppdisassembly: *mut super::Direct3D::ID3DBlob) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Direct3D\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct3D"))]
    pub fn D3D10DisassembleShader(pshader: *const ::core::ffi::c_void, bytecodelength: usize, enablecolorcode: super::super::Foundation::BOOL, pcomments: ::windows_sys::core::PCSTR, ppdisassembly: *mut super::Direct3D::ID3DBlob) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
    pub fn D3D10GetGeometryShaderProfile(pdevice: ID3D10Device) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Graphics_Direct3D\"`*"]
    #[cfg(feature = "Win32_Graphics_Direct3D")]
    pub fn D3D10GetInputAndOutputSignatureBlob(pshaderbytecode: *const ::core::ffi::c_void, bytecodelength: usize, ppsignatureblob: *mut super::Direct3D::ID3DBlob) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Graphics_Direct3D\"`*"]
    #[cfg(feature = "Win32_Graphics_Direct3D")]
    pub fn D3D10GetInputSignatureBlob(pshaderbytecode: *const ::core::ffi::c_void, bytecodelength: usize, ppsignatureblob: *mut super::Direct3D::ID3DBlob) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Graphics_Direct3D\"`*"]
    #[cfg(feature = "Win32_Graphics_Direct3D")]
    pub fn D3D10GetOutputSignatureBlob(pshaderbytecode: *const ::core::ffi::c_void, bytecodelength: usize, ppsignatureblob: *mut super::Direct3D::ID3DBlob) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
    pub fn D3D10GetPixelShaderProfile(pdevice: ID3D10Device) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Graphics_Direct3D\"`*"]
    #[cfg(feature = "Win32_Graphics_Direct3D")]
    pub fn D3D10GetShaderDebugInfo(pshaderbytecode: *const ::core::ffi::c_void, bytecodelength: usize, ppdebuginfo: *mut super::Direct3D::ID3DBlob) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
    pub fn D3D10GetVertexShaderProfile(pdevice: ID3D10Device) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Graphics_Direct3D\"`*"]
    #[cfg(feature = "Win32_Graphics_Direct3D")]
    pub fn D3D10PreprocessShader(psrcdata: ::windows_sys::core::PCSTR, srcdatasize: usize, pfilename: ::windows_sys::core::PCSTR, pdefines: *const super::Direct3D::D3D_SHADER_MACRO, pinclude: super::Direct3D::ID3DInclude, ppshadertext: *mut super::Direct3D::ID3DBlob, pperrormsgs: *mut super::Direct3D::ID3DBlob) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
    pub fn D3D10ReflectShader(pshaderbytecode: *const ::core::ffi::c_void, bytecodelength: usize, ppreflector: *mut ID3D10ShaderReflection) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
    pub fn D3D10StateBlockMaskDifference(pa: *const D3D10_STATE_BLOCK_MASK, pb: *const D3D10_STATE_BLOCK_MASK, presult: *mut D3D10_STATE_BLOCK_MASK) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
    pub fn D3D10StateBlockMaskDisableAll(pmask: *mut D3D10_STATE_BLOCK_MASK) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
    pub fn D3D10StateBlockMaskDisableCapture(pmask: *mut D3D10_STATE_BLOCK_MASK, statetype: D3D10_DEVICE_STATE_TYPES, rangestart: u32, rangelength: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
    pub fn D3D10StateBlockMaskEnableAll(pmask: *mut D3D10_STATE_BLOCK_MASK) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
    pub fn D3D10StateBlockMaskEnableCapture(pmask: *mut D3D10_STATE_BLOCK_MASK, statetype: D3D10_DEVICE_STATE_TYPES, rangestart: u32, rangelength: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn D3D10StateBlockMaskGetSetting(pmask: *const D3D10_STATE_BLOCK_MASK, statetype: D3D10_DEVICE_STATE_TYPES, entry: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
    pub fn D3D10StateBlockMaskIntersect(pa: *const D3D10_STATE_BLOCK_MASK, pb: *const D3D10_STATE_BLOCK_MASK, presult: *mut D3D10_STATE_BLOCK_MASK) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
    pub fn D3D10StateBlockMaskUnion(pa: *const D3D10_STATE_BLOCK_MASK, pb: *const D3D10_STATE_BLOCK_MASK, presult: *mut D3D10_STATE_BLOCK_MASK) -> ::windows_sys::core::HRESULT;
}
pub type ID3D10Asynchronous = *mut ::core::ffi::c_void;
pub type ID3D10BlendState = *mut ::core::ffi::c_void;
pub type ID3D10BlendState1 = *mut ::core::ffi::c_void;
pub type ID3D10Buffer = *mut ::core::ffi::c_void;
pub type ID3D10Counter = *mut ::core::ffi::c_void;
pub type ID3D10Debug = *mut ::core::ffi::c_void;
pub type ID3D10DepthStencilState = *mut ::core::ffi::c_void;
pub type ID3D10DepthStencilView = *mut ::core::ffi::c_void;
pub type ID3D10Device = *mut ::core::ffi::c_void;
pub type ID3D10Device1 = *mut ::core::ffi::c_void;
pub type ID3D10DeviceChild = *mut ::core::ffi::c_void;
pub type ID3D10Effect = *mut ::core::ffi::c_void;
pub type ID3D10EffectBlendVariable = *mut ::core::ffi::c_void;
pub type ID3D10EffectConstantBuffer = *mut ::core::ffi::c_void;
pub type ID3D10EffectDepthStencilVariable = *mut ::core::ffi::c_void;
pub type ID3D10EffectDepthStencilViewVariable = *mut ::core::ffi::c_void;
pub type ID3D10EffectMatrixVariable = *mut ::core::ffi::c_void;
pub type ID3D10EffectPass = *mut ::core::ffi::c_void;
pub type ID3D10EffectPool = *mut ::core::ffi::c_void;
pub type ID3D10EffectRasterizerVariable = *mut ::core::ffi::c_void;
pub type ID3D10EffectRenderTargetViewVariable = *mut ::core::ffi::c_void;
pub type ID3D10EffectSamplerVariable = *mut ::core::ffi::c_void;
pub type ID3D10EffectScalarVariable = *mut ::core::ffi::c_void;
pub type ID3D10EffectShaderResourceVariable = *mut ::core::ffi::c_void;
pub type ID3D10EffectShaderVariable = *mut ::core::ffi::c_void;
pub type ID3D10EffectStringVariable = *mut ::core::ffi::c_void;
pub type ID3D10EffectTechnique = *mut ::core::ffi::c_void;
pub type ID3D10EffectType = *mut ::core::ffi::c_void;
pub type ID3D10EffectVariable = *mut ::core::ffi::c_void;
pub type ID3D10EffectVectorVariable = *mut ::core::ffi::c_void;
pub type ID3D10GeometryShader = *mut ::core::ffi::c_void;
pub type ID3D10InfoQueue = *mut ::core::ffi::c_void;
pub type ID3D10InputLayout = *mut ::core::ffi::c_void;
pub type ID3D10Multithread = *mut ::core::ffi::c_void;
pub type ID3D10PixelShader = *mut ::core::ffi::c_void;
pub type ID3D10Predicate = *mut ::core::ffi::c_void;
pub type ID3D10Query = *mut ::core::ffi::c_void;
pub type ID3D10RasterizerState = *mut ::core::ffi::c_void;
pub type ID3D10RenderTargetView = *mut ::core::ffi::c_void;
pub type ID3D10Resource = *mut ::core::ffi::c_void;
pub type ID3D10SamplerState = *mut ::core::ffi::c_void;
pub type ID3D10ShaderReflection = *mut ::core::ffi::c_void;
pub type ID3D10ShaderReflection1 = *mut ::core::ffi::c_void;
pub type ID3D10ShaderReflectionConstantBuffer = *mut ::core::ffi::c_void;
pub type ID3D10ShaderReflectionType = *mut ::core::ffi::c_void;
pub type ID3D10ShaderReflectionVariable = *mut ::core::ffi::c_void;
pub type ID3D10ShaderResourceView = *mut ::core::ffi::c_void;
pub type ID3D10ShaderResourceView1 = *mut ::core::ffi::c_void;
pub type ID3D10StateBlock = *mut ::core::ffi::c_void;
pub type ID3D10SwitchToRef = *mut ::core::ffi::c_void;
pub type ID3D10Texture1D = *mut ::core::ffi::c_void;
pub type ID3D10Texture2D = *mut ::core::ffi::c_void;
pub type ID3D10Texture3D = *mut ::core::ffi::c_void;
pub type ID3D10VertexShader = *mut ::core::ffi::c_void;
pub type ID3D10View = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_16BIT_INDEX_STRIP_CUT_VALUE: u32 = 65535u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_1_DEFAULT_SAMPLE_MASK: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_1_FLOAT16_FUSED_TOLERANCE_IN_ULP: f64 = 0.6f64;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_1_FLOAT32_TO_INTEGER_TOLERANCE_IN_ULP: f32 = 0.6f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_1_GS_INPUT_REGISTER_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_1_IA_VERTEX_INPUT_RESOURCE_SLOT_COUNT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_1_IA_VERTEX_INPUT_STRUCTURE_ELEMENTS_COMPONENTS: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_1_IA_VERTEX_INPUT_STRUCTURE_ELEMENT_COUNT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_1_PS_OUTPUT_MASK_REGISTER_COMPONENTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_1_PS_OUTPUT_MASK_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_1_PS_OUTPUT_MASK_REGISTER_COUNT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_1_SHADER_MAJOR_VERSION: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_1_SHADER_MINOR_VERSION: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_1_SO_BUFFER_MAX_STRIDE_IN_BYTES: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_1_SO_BUFFER_MAX_WRITE_WINDOW_IN_BYTES: u32 = 256u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_1_SO_BUFFER_SLOT_COUNT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_1_SO_MULTIPLE_BUFFER_ELEMENTS_PER_BUFFER: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_1_SO_SINGLE_BUFFER_COMPONENT_LIMIT: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_1_STANDARD_VERTEX_ELEMENT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_1_SUBPIXEL_FRACTIONAL_BIT_COUNT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_1_VS_INPUT_REGISTER_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_1_VS_OUTPUT_REGISTER_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_32BIT_INDEX_STRIP_CUT_VALUE: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_8BIT_INDEX_STRIP_CUT_VALUE: u32 = 255u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_ALL_RESOURCES_BOUND: u32 = 2097152u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_ANISOTROPIC_FILTERING_BIT: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_APPEND_ALIGNED_ELEMENT: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_APPNAME_STRING: &str = "Name";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_APPSIZE_STRING: &str = "Size";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_ARRAY_AXIS_ADDRESS_RANGE_BIT_COUNT: u32 = 9u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BREAKON_CATEGORY: &str = "BreakOn_CATEGORY_%s";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BREAKON_ID_DECIMAL: &str = "BreakOn_ID_%d";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BREAKON_ID_STRING: &str = "BreakOn_ID_%s";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BREAKON_SEVERITY: &str = "BreakOn_SEVERITY_%s";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_CLIP_OR_CULL_DISTANCE_COUNT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_CLIP_OR_CULL_DISTANCE_ELEMENT_COUNT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_CONSTANT_BUFFER_API_SLOT_COUNT: u32 = 14u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_CONSTANT_BUFFER_COMPONENTS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_CONSTANT_BUFFER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_CONSTANT_BUFFER_HW_SLOT_COUNT: u32 = 15u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_CONSTANT_BUFFER_REGISTER_COMPONENTS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_CONSTANT_BUFFER_REGISTER_COUNT: u32 = 15u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_CONSTANT_BUFFER_REGISTER_READS_PER_INST: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_CONSTANT_BUFFER_REGISTER_READ_PORTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_FLOWCONTROL_NESTING_LIMIT: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_IMMEDIATE_CONSTANT_BUFFER_REGISTER_COMPONENTS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_IMMEDIATE_CONSTANT_BUFFER_REGISTER_COUNT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_IMMEDIATE_CONSTANT_BUFFER_REGISTER_READS_PER_INST: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_IMMEDIATE_CONSTANT_BUFFER_REGISTER_READ_PORTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_IMMEDIATE_VALUE_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_INPUT_RESOURCE_REGISTER_COMPONENTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_INPUT_RESOURCE_REGISTER_COUNT: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_INPUT_RESOURCE_REGISTER_READS_PER_INST: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_INPUT_RESOURCE_REGISTER_READ_PORTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_INPUT_RESOURCE_SLOT_COUNT: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_SAMPLER_REGISTER_COMPONENTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_SAMPLER_REGISTER_COUNT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_SAMPLER_REGISTER_READS_PER_INST: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_SAMPLER_REGISTER_READ_PORTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_SAMPLER_SLOT_COUNT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_SUBROUTINE_NESTING_LIMIT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_TEMP_REGISTER_COMPONENTS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_TEMP_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_TEMP_REGISTER_COUNT: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_TEMP_REGISTER_READS_PER_INST: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_TEMP_REGISTER_READ_PORTS: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_TEXCOORD_RANGE_REDUCTION_MAX: u32 = 10u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_TEXCOORD_RANGE_REDUCTION_MIN: i32 = -10i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_TEXEL_OFFSET_MAX_NEGATIVE: i32 = -8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMMONSHADER_TEXEL_OFFSET_MAX_POSITIVE: u32 = 7u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMPARISON_FILTERING_BIT: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEBUG_FEATURE_FINISH_PER_RENDER_OP: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEBUG_FEATURE_FLUSH_PER_RENDER_OP: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEBUG_FEATURE_PRESENT_PER_RENDER_OP: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEFAULT_BLEND_FACTOR_ALPHA: f32 = 1f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEFAULT_BLEND_FACTOR_BLUE: f32 = 1f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEFAULT_BLEND_FACTOR_GREEN: f32 = 1f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEFAULT_BLEND_FACTOR_RED: f32 = 1f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEFAULT_BORDER_COLOR_COMPONENT: f32 = 0f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEFAULT_DEPTH_BIAS: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEFAULT_DEPTH_BIAS_CLAMP: f32 = 0f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEFAULT_MAX_ANISOTROPY: f32 = 16f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEFAULT_MIP_LOD_BIAS: f32 = 0f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEFAULT_RENDER_TARGET_ARRAY_INDEX: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEFAULT_SAMPLE_MASK: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEFAULT_SCISSOR_ENDX: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEFAULT_SCISSOR_ENDY: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEFAULT_SCISSOR_STARTX: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEFAULT_SCISSOR_STARTY: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEFAULT_SLOPE_SCALED_DEPTH_BIAS: f32 = 0f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEFAULT_STENCIL_READ_MASK: u32 = 255u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEFAULT_STENCIL_REFERENCE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEFAULT_STENCIL_WRITE_MASK: u32 = 255u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEFAULT_VIEWPORT_AND_SCISSORRECT_INDEX: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEFAULT_VIEWPORT_HEIGHT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEFAULT_VIEWPORT_MAX_DEPTH: f32 = 0f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEFAULT_VIEWPORT_MIN_DEPTH: f32 = 0f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEFAULT_VIEWPORT_TOPLEFTX: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEFAULT_VIEWPORT_TOPLEFTY: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEFAULT_VIEWPORT_WIDTH: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_EFFECT_COMPILE_ALLOW_SLOW_OPS: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_EFFECT_COMPILE_CHILD_EFFECT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_EFFECT_SINGLE_THREADED: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_EFFECT_VARIABLE_ANNOTATION: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_EFFECT_VARIABLE_EXPLICIT_BIND_POINT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_EFFECT_VARIABLE_POOLED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_ENABLE_BREAK_ON_MESSAGE: &str = "EnableBreakOnMessage";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_ENABLE_UNBOUNDED_DESCRIPTOR_TABLES: u32 = 1048576u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FILTER_TYPE_MASK: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FLOAT16_FUSED_TOLERANCE_IN_ULP: f64 = 0.6f64;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FLOAT32_MAX: f32 = 340282350000000000000000000000000000000f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FLOAT32_TO_INTEGER_TOLERANCE_IN_ULP: f32 = 0.6f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FLOAT_TO_SRGB_EXPONENT_DENOMINATOR: f32 = 2.4f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FLOAT_TO_SRGB_EXPONENT_NUMERATOR: f32 = 1f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FLOAT_TO_SRGB_OFFSET: f32 = 0.055f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FLOAT_TO_SRGB_SCALE_1: f32 = 12.92f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FLOAT_TO_SRGB_SCALE_2: f32 = 1.055f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FLOAT_TO_SRGB_THRESHOLD: f32 = 0.0031308f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FTOI_INSTRUCTION_MAX_INPUT: f32 = 2147483600f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FTOI_INSTRUCTION_MIN_INPUT: f32 = -2147483600f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FTOU_INSTRUCTION_MAX_INPUT: f32 = 4294967300f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FTOU_INSTRUCTION_MIN_INPUT: f32 = 0f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_GS_INPUT_PRIM_CONST_REGISTER_COMPONENTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_GS_INPUT_PRIM_CONST_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_GS_INPUT_PRIM_CONST_REGISTER_COUNT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_GS_INPUT_PRIM_CONST_REGISTER_READS_PER_INST: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_GS_INPUT_PRIM_CONST_REGISTER_READ_PORTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_GS_INPUT_REGISTER_COMPONENTS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_GS_INPUT_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_GS_INPUT_REGISTER_COUNT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_GS_INPUT_REGISTER_READS_PER_INST: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_GS_INPUT_REGISTER_READ_PORTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_GS_INPUT_REGISTER_VERTICES: u32 = 6u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_GS_OUTPUT_ELEMENTS: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_GS_OUTPUT_REGISTER_COMPONENTS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_GS_OUTPUT_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_GS_OUTPUT_REGISTER_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_IA_DEFAULT_INDEX_BUFFER_OFFSET_IN_BYTES: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_IA_DEFAULT_PRIMITIVE_TOPOLOGY: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_IA_DEFAULT_VERTEX_BUFFER_OFFSET_IN_BYTES: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_IA_INDEX_INPUT_RESOURCE_SLOT_COUNT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_IA_INSTANCE_ID_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_IA_INTEGER_ARITHMETIC_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_IA_PRIMITIVE_ID_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_IA_VERTEX_ID_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_IA_VERTEX_INPUT_RESOURCE_SLOT_COUNT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_IA_VERTEX_INPUT_STRUCTURE_ELEMENTS_COMPONENTS: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_IA_VERTEX_INPUT_STRUCTURE_ELEMENT_COUNT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_INFOQUEUE_STORAGE_FILTER_OVERRIDE: &str = "InfoQueueStorageFilterOverride";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_INFO_QUEUE_DEFAULT_MESSAGE_COUNT_LIMIT: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_INTEGER_DIVIDE_BY_ZERO_QUOTIENT: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_INTEGER_DIVIDE_BY_ZERO_REMAINDER: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_LINEAR_GAMMA: f32 = 1f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MAG_FILTER_SHIFT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MAX_BORDER_COLOR_COMPONENT: f32 = 1f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MAX_DEPTH: f32 = 1f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MAX_MAXANISOTROPY: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MAX_MULTISAMPLE_SAMPLE_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MAX_POSITION_VALUE: f32 = 34028236000000000000000000000000000f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MAX_TEXTURE_DIMENSION_2_TO_EXP: u32 = 17u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MIN_BORDER_COLOR_COMPONENT: f32 = 0f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MIN_DEPTH: f32 = 0f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MIN_FILTER_SHIFT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MIN_MAXANISOTROPY: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MIP_FILTER_SHIFT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MIP_LOD_BIAS_MAX: f32 = 15.99f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MIP_LOD_BIAS_MIN: f32 = -16f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MIP_LOD_FRACTIONAL_BIT_COUNT: u32 = 6u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MIP_LOD_RANGE_BIT_COUNT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MULTISAMPLE_ANTIALIAS_LINE_WIDTH: f32 = 1.4f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MUTE_CATEGORY: &str = "Mute_CATEGORY_%s";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MUTE_DEBUG_OUTPUT: &str = "MuteDebugOutput";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MUTE_ID_DECIMAL: &str = "Mute_ID_%d";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MUTE_ID_STRING: &str = "Mute_ID_%s";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MUTE_SEVERITY: &str = "Mute_SEVERITY_%s";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_NONSAMPLE_FETCH_OUT_OF_RANGE_ACCESS_RESULT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_PIXEL_ADDRESS_RANGE_BIT_COUNT: u32 = 13u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_PRE_SCISSOR_PIXEL_ADDRESS_RANGE_BIT_COUNT: u32 = 15u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_PS_FRONTFACING_DEFAULT_VALUE: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_PS_FRONTFACING_FALSE_VALUE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_PS_FRONTFACING_TRUE_VALUE: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_PS_INPUT_REGISTER_COMPONENTS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_PS_INPUT_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_PS_INPUT_REGISTER_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_PS_INPUT_REGISTER_READS_PER_INST: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_PS_INPUT_REGISTER_READ_PORTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_PS_LEGACY_PIXEL_CENTER_FRACTIONAL_COMPONENT: f32 = 0f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_PS_OUTPUT_DEPTH_REGISTER_COMPONENTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_PS_OUTPUT_DEPTH_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_PS_OUTPUT_DEPTH_REGISTER_COUNT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_PS_OUTPUT_REGISTER_COMPONENTS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_PS_OUTPUT_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_PS_OUTPUT_REGISTER_COUNT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_PS_PIXEL_CENTER_FRACTIONAL_COMPONENT: f32 = 0.5f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_REGKEY_PATH: &str = "Software\\Microsoft\\Direct3D";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_REQ_BLEND_OBJECT_COUNT_PER_CONTEXT: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_REQ_BUFFER_RESOURCE_TEXEL_COUNT_2_TO_EXP: u32 = 27u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_REQ_CONSTANT_BUFFER_ELEMENT_COUNT: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_REQ_DEPTH_STENCIL_OBJECT_COUNT_PER_CONTEXT: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_REQ_DRAWINDEXED_INDEX_COUNT_2_TO_EXP: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_REQ_DRAW_VERTEX_COUNT_2_TO_EXP: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_REQ_FILTERING_HW_ADDRESSABLE_RESOURCE_DIMENSION: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_REQ_GS_INVOCATION_32BIT_OUTPUT_COMPONENT_LIMIT: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_REQ_IMMEDIATE_CONSTANT_BUFFER_ELEMENT_COUNT: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_REQ_MAXANISOTROPY: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_REQ_MIP_LEVELS: u32 = 14u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_REQ_MULTI_ELEMENT_STRUCTURE_SIZE_IN_BYTES: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_REQ_RASTERIZER_OBJECT_COUNT_PER_CONTEXT: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_REQ_RENDER_TO_BUFFER_WINDOW_WIDTH: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_REQ_RESOURCE_SIZE_IN_MEGABYTES: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_REQ_RESOURCE_VIEW_COUNT_PER_CONTEXT_2_TO_EXP: u32 = 20u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_REQ_SAMPLER_OBJECT_COUNT_PER_CONTEXT: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_REQ_TEXTURE1D_ARRAY_AXIS_DIMENSION: u32 = 512u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_REQ_TEXTURE1D_U_DIMENSION: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_REQ_TEXTURE2D_ARRAY_AXIS_DIMENSION: u32 = 512u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_REQ_TEXTURE2D_U_OR_V_DIMENSION: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_REQ_TEXTURE3D_U_V_OR_W_DIMENSION: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_REQ_TEXTURECUBE_DIMENSION: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_RESINFO_INSTRUCTION_MISSING_COMPONENT_RETVAL: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SDK_LAYERS_VERSION: u32 = 11u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SDK_VERSION: u32 = 29u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_AVOID_FLOW_CONTROL: u32 = 512u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_DEBUG: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_DEBUG_NAME_FOR_BINARY: u32 = 8388608u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_DEBUG_NAME_FOR_SOURCE: u32 = 4194304u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_ENABLE_BACKWARDS_COMPATIBILITY: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_ENABLE_STRICTNESS: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_FLAGS2_FORCE_ROOT_SIGNATURE_1_0: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_FLAGS2_FORCE_ROOT_SIGNATURE_1_1: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_FLAGS2_FORCE_ROOT_SIGNATURE_LATEST: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_FORCE_PS_SOFTWARE_NO_OPT: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_FORCE_VS_SOFTWARE_NO_OPT: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_IEEE_STRICTNESS: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_MAJOR_VERSION: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_MINOR_VERSION: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_NO_PRESHADER: u32 = 256u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_OPTIMIZATION_LEVEL0: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_OPTIMIZATION_LEVEL1: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_OPTIMIZATION_LEVEL3: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_PACK_MATRIX_COLUMN_MAJOR: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_PACK_MATRIX_ROW_MAJOR: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_PARTIAL_PRECISION: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_PREFER_FLOW_CONTROL: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_RESOURCES_MAY_ALIAS: u32 = 524288u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_SKIP_OPTIMIZATION: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_SKIP_VALIDATION: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_WARNINGS_ARE_ERRORS: u32 = 262144u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHIFT_INSTRUCTION_PAD_VALUE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHIFT_INSTRUCTION_SHIFT_VALUE_BIT_COUNT: u32 = 5u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SIMULTANEOUS_RENDER_TARGET_COUNT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SO_BUFFER_MAX_STRIDE_IN_BYTES: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SO_BUFFER_MAX_WRITE_WINDOW_IN_BYTES: u32 = 256u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SO_BUFFER_SLOT_COUNT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SO_DDI_REGISTER_INDEX_DENOTING_GAP: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SO_MULTIPLE_BUFFER_ELEMENTS_PER_BUFFER: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SO_SINGLE_BUFFER_COMPONENT_LIMIT: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SRGB_GAMMA: f32 = 2.2f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SRGB_TO_FLOAT_DENOMINATOR_1: f32 = 12.92f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SRGB_TO_FLOAT_DENOMINATOR_2: f32 = 1.055f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SRGB_TO_FLOAT_EXPONENT: f32 = 2.4f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SRGB_TO_FLOAT_OFFSET: f32 = 0.055f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SRGB_TO_FLOAT_THRESHOLD: f32 = 0.04045f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SRGB_TO_FLOAT_TOLERANCE_IN_ULP: f32 = 0.5f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_STANDARD_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_STANDARD_COMPONENT_BIT_COUNT_DOUBLED: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_STANDARD_MAXIMUM_ELEMENT_ALIGNMENT_BYTE_MULTIPLE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_STANDARD_PIXEL_COMPONENT_COUNT: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_STANDARD_PIXEL_ELEMENT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_STANDARD_VECTOR_SIZE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_STANDARD_VERTEX_ELEMENT_COUNT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_STANDARD_VERTEX_TOTAL_COMPONENT_COUNT: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SUBPIXEL_FRACTIONAL_BIT_COUNT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SUBTEXEL_FRACTIONAL_BIT_COUNT: u32 = 6u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_TEXEL_ADDRESS_RANGE_BIT_COUNT: u32 = 18u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_TEXT_1BIT_BIT: u32 = 2147483648u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_UNBOUND_MEMORY_ACCESS_RESULT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_UNMUTE_SEVERITY_INFO: &str = "Unmute_SEVERITY_INFO";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_VIEWPORT_AND_SCISSORRECT_MAX_INDEX: u32 = 15u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_VIEWPORT_AND_SCISSORRECT_OBJECT_COUNT_PER_PIPELINE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_VIEWPORT_BOUNDS_MAX: u32 = 16383u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_VIEWPORT_BOUNDS_MIN: i32 = -16384i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_VS_INPUT_REGISTER_COMPONENTS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_VS_INPUT_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_VS_INPUT_REGISTER_COUNT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_VS_INPUT_REGISTER_READS_PER_INST: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_VS_INPUT_REGISTER_READ_PORTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_VS_OUTPUT_REGISTER_COMPONENTS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_VS_OUTPUT_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_VS_OUTPUT_REGISTER_COUNT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_WHQL_CONTEXT_COUNT_FOR_RESOURCE_LIMIT: u32 = 10u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_WHQL_DRAWINDEXED_INDEX_COUNT_2_TO_EXP: u32 = 25u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_WHQL_DRAW_VERTEX_COUNT_2_TO_EXP: u32 = 25u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D_MAJOR_VERSION: u32 = 10u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D_MINOR_VERSION: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D_SPEC_DATE_DAY: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D_SPEC_DATE_MONTH: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D_SPEC_DATE_YEAR: u32 = 2006u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D_SPEC_VERSION: f64 = 1.050005f64;
pub const DXGI_DEBUG_D3D10: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 607865938, data2: 13830, data3: 19770, data4: [153, 215, 167, 231, 179, 62, 215, 6] };
pub const GUID_DeviceType: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3609393997, data2: 31336, data3: 17274, data4: [178, 12, 88, 4, 238, 36, 148, 166] };
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const _FACD3D10: u32 = 2169u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_ASYNC_GETDATA_FLAG = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_ASYNC_GETDATA_DONOTFLUSH: D3D10_ASYNC_GETDATA_FLAG = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_BIND_FLAG = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BIND_VERTEX_BUFFER: D3D10_BIND_FLAG = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BIND_INDEX_BUFFER: D3D10_BIND_FLAG = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BIND_CONSTANT_BUFFER: D3D10_BIND_FLAG = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BIND_SHADER_RESOURCE: D3D10_BIND_FLAG = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BIND_STREAM_OUTPUT: D3D10_BIND_FLAG = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BIND_RENDER_TARGET: D3D10_BIND_FLAG = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BIND_DEPTH_STENCIL: D3D10_BIND_FLAG = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_BLEND = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BLEND_ZERO: D3D10_BLEND = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BLEND_ONE: D3D10_BLEND = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BLEND_SRC_COLOR: D3D10_BLEND = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BLEND_INV_SRC_COLOR: D3D10_BLEND = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BLEND_SRC_ALPHA: D3D10_BLEND = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BLEND_INV_SRC_ALPHA: D3D10_BLEND = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BLEND_DEST_ALPHA: D3D10_BLEND = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BLEND_INV_DEST_ALPHA: D3D10_BLEND = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BLEND_DEST_COLOR: D3D10_BLEND = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BLEND_INV_DEST_COLOR: D3D10_BLEND = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BLEND_SRC_ALPHA_SAT: D3D10_BLEND = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BLEND_BLEND_FACTOR: D3D10_BLEND = 14i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BLEND_INV_BLEND_FACTOR: D3D10_BLEND = 15i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BLEND_SRC1_COLOR: D3D10_BLEND = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BLEND_INV_SRC1_COLOR: D3D10_BLEND = 17i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BLEND_SRC1_ALPHA: D3D10_BLEND = 18i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BLEND_INV_SRC1_ALPHA: D3D10_BLEND = 19i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_BLEND_OP = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BLEND_OP_ADD: D3D10_BLEND_OP = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BLEND_OP_SUBTRACT: D3D10_BLEND_OP = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BLEND_OP_REV_SUBTRACT: D3D10_BLEND_OP = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BLEND_OP_MIN: D3D10_BLEND_OP = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_BLEND_OP_MAX: D3D10_BLEND_OP = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_CLEAR_FLAG = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_CLEAR_DEPTH: D3D10_CLEAR_FLAG = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_CLEAR_STENCIL: D3D10_CLEAR_FLAG = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_COLOR_WRITE_ENABLE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COLOR_WRITE_ENABLE_RED: D3D10_COLOR_WRITE_ENABLE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COLOR_WRITE_ENABLE_GREEN: D3D10_COLOR_WRITE_ENABLE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COLOR_WRITE_ENABLE_BLUE: D3D10_COLOR_WRITE_ENABLE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COLOR_WRITE_ENABLE_ALPHA: D3D10_COLOR_WRITE_ENABLE = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COLOR_WRITE_ENABLE_ALL: D3D10_COLOR_WRITE_ENABLE = 15i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_COMPARISON_FUNC = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMPARISON_NEVER: D3D10_COMPARISON_FUNC = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMPARISON_LESS: D3D10_COMPARISON_FUNC = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMPARISON_EQUAL: D3D10_COMPARISON_FUNC = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMPARISON_LESS_EQUAL: D3D10_COMPARISON_FUNC = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMPARISON_GREATER: D3D10_COMPARISON_FUNC = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMPARISON_NOT_EQUAL: D3D10_COMPARISON_FUNC = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMPARISON_GREATER_EQUAL: D3D10_COMPARISON_FUNC = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COMPARISON_ALWAYS: D3D10_COMPARISON_FUNC = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_COUNTER = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COUNTER_GPU_IDLE: D3D10_COUNTER = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COUNTER_VERTEX_PROCESSING: D3D10_COUNTER = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COUNTER_GEOMETRY_PROCESSING: D3D10_COUNTER = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COUNTER_PIXEL_PROCESSING: D3D10_COUNTER = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COUNTER_OTHER_GPU_PROCESSING: D3D10_COUNTER = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COUNTER_HOST_ADAPTER_BANDWIDTH_UTILIZATION: D3D10_COUNTER = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COUNTER_LOCAL_VIDMEM_BANDWIDTH_UTILIZATION: D3D10_COUNTER = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COUNTER_VERTEX_THROUGHPUT_UTILIZATION: D3D10_COUNTER = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COUNTER_TRIANGLE_SETUP_THROUGHPUT_UTILIZATION: D3D10_COUNTER = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COUNTER_FILLRATE_THROUGHPUT_UTILIZATION: D3D10_COUNTER = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COUNTER_VS_MEMORY_LIMITED: D3D10_COUNTER = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COUNTER_VS_COMPUTATION_LIMITED: D3D10_COUNTER = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COUNTER_GS_MEMORY_LIMITED: D3D10_COUNTER = 12i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COUNTER_GS_COMPUTATION_LIMITED: D3D10_COUNTER = 13i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COUNTER_PS_MEMORY_LIMITED: D3D10_COUNTER = 14i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COUNTER_PS_COMPUTATION_LIMITED: D3D10_COUNTER = 15i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COUNTER_POST_TRANSFORM_CACHE_HIT_RATE: D3D10_COUNTER = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COUNTER_TEXTURE_CACHE_HIT_RATE: D3D10_COUNTER = 17i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COUNTER_DEVICE_DEPENDENT_0: D3D10_COUNTER = 1073741824i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_COUNTER_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COUNTER_TYPE_FLOAT32: D3D10_COUNTER_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COUNTER_TYPE_UINT16: D3D10_COUNTER_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COUNTER_TYPE_UINT32: D3D10_COUNTER_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_COUNTER_TYPE_UINT64: D3D10_COUNTER_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_CPU_ACCESS_FLAG = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_CPU_ACCESS_WRITE: D3D10_CPU_ACCESS_FLAG = 65536i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_CPU_ACCESS_READ: D3D10_CPU_ACCESS_FLAG = 131072i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_CREATE_DEVICE_FLAG = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_CREATE_DEVICE_SINGLETHREADED: D3D10_CREATE_DEVICE_FLAG = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_CREATE_DEVICE_DEBUG: D3D10_CREATE_DEVICE_FLAG = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_CREATE_DEVICE_SWITCH_TO_REF: D3D10_CREATE_DEVICE_FLAG = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_CREATE_DEVICE_PREVENT_INTERNAL_THREADING_OPTIMIZATIONS: D3D10_CREATE_DEVICE_FLAG = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_CREATE_DEVICE_ALLOW_NULL_FROM_MAP: D3D10_CREATE_DEVICE_FLAG = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_CREATE_DEVICE_BGRA_SUPPORT: D3D10_CREATE_DEVICE_FLAG = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_CREATE_DEVICE_PREVENT_ALTERING_LAYER_SETTINGS_FROM_REGISTRY: D3D10_CREATE_DEVICE_FLAG = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_CREATE_DEVICE_STRICT_VALIDATION: D3D10_CREATE_DEVICE_FLAG = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_CREATE_DEVICE_DEBUGGABLE: D3D10_CREATE_DEVICE_FLAG = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_CULL_MODE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_CULL_NONE: D3D10_CULL_MODE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_CULL_FRONT: D3D10_CULL_MODE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_CULL_BACK: D3D10_CULL_MODE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_DEPTH_WRITE_MASK = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEPTH_WRITE_MASK_ZERO: D3D10_DEPTH_WRITE_MASK = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DEPTH_WRITE_MASK_ALL: D3D10_DEPTH_WRITE_MASK = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_DEVICE_STATE_TYPES = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DST_SO_BUFFERS: D3D10_DEVICE_STATE_TYPES = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DST_OM_RENDER_TARGETS: D3D10_DEVICE_STATE_TYPES = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DST_OM_DEPTH_STENCIL_STATE: D3D10_DEVICE_STATE_TYPES = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DST_OM_BLEND_STATE: D3D10_DEVICE_STATE_TYPES = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DST_VS: D3D10_DEVICE_STATE_TYPES = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DST_VS_SAMPLERS: D3D10_DEVICE_STATE_TYPES = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DST_VS_SHADER_RESOURCES: D3D10_DEVICE_STATE_TYPES = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DST_VS_CONSTANT_BUFFERS: D3D10_DEVICE_STATE_TYPES = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DST_GS: D3D10_DEVICE_STATE_TYPES = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DST_GS_SAMPLERS: D3D10_DEVICE_STATE_TYPES = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DST_GS_SHADER_RESOURCES: D3D10_DEVICE_STATE_TYPES = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DST_GS_CONSTANT_BUFFERS: D3D10_DEVICE_STATE_TYPES = 12i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DST_PS: D3D10_DEVICE_STATE_TYPES = 13i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DST_PS_SAMPLERS: D3D10_DEVICE_STATE_TYPES = 14i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DST_PS_SHADER_RESOURCES: D3D10_DEVICE_STATE_TYPES = 15i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DST_PS_CONSTANT_BUFFERS: D3D10_DEVICE_STATE_TYPES = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DST_IA_VERTEX_BUFFERS: D3D10_DEVICE_STATE_TYPES = 17i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DST_IA_INDEX_BUFFER: D3D10_DEVICE_STATE_TYPES = 18i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DST_IA_INPUT_LAYOUT: D3D10_DEVICE_STATE_TYPES = 19i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DST_IA_PRIMITIVE_TOPOLOGY: D3D10_DEVICE_STATE_TYPES = 20i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DST_RS_VIEWPORTS: D3D10_DEVICE_STATE_TYPES = 21i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DST_RS_SCISSOR_RECTS: D3D10_DEVICE_STATE_TYPES = 22i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DST_RS_RASTERIZER_STATE: D3D10_DEVICE_STATE_TYPES = 23i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DST_PREDICATION: D3D10_DEVICE_STATE_TYPES = 24i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_DRIVER_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DRIVER_TYPE_HARDWARE: D3D10_DRIVER_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DRIVER_TYPE_REFERENCE: D3D10_DRIVER_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DRIVER_TYPE_NULL: D3D10_DRIVER_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DRIVER_TYPE_SOFTWARE: D3D10_DRIVER_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DRIVER_TYPE_WARP: D3D10_DRIVER_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_DSV_DIMENSION = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DSV_DIMENSION_UNKNOWN: D3D10_DSV_DIMENSION = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DSV_DIMENSION_TEXTURE1D: D3D10_DSV_DIMENSION = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DSV_DIMENSION_TEXTURE1DARRAY: D3D10_DSV_DIMENSION = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DSV_DIMENSION_TEXTURE2D: D3D10_DSV_DIMENSION = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DSV_DIMENSION_TEXTURE2DARRAY: D3D10_DSV_DIMENSION = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DSV_DIMENSION_TEXTURE2DMS: D3D10_DSV_DIMENSION = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_DSV_DIMENSION_TEXTURE2DMSARRAY: D3D10_DSV_DIMENSION = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_FEATURE_LEVEL1 = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FEATURE_LEVEL_10_0: D3D10_FEATURE_LEVEL1 = 40960i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FEATURE_LEVEL_10_1: D3D10_FEATURE_LEVEL1 = 41216i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FEATURE_LEVEL_9_1: D3D10_FEATURE_LEVEL1 = 37120i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FEATURE_LEVEL_9_2: D3D10_FEATURE_LEVEL1 = 37376i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FEATURE_LEVEL_9_3: D3D10_FEATURE_LEVEL1 = 37632i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_FILL_MODE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FILL_WIREFRAME: D3D10_FILL_MODE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FILL_SOLID: D3D10_FILL_MODE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_FILTER = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FILTER_MIN_MAG_MIP_POINT: D3D10_FILTER = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FILTER_MIN_MAG_POINT_MIP_LINEAR: D3D10_FILTER = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FILTER_MIN_POINT_MAG_LINEAR_MIP_POINT: D3D10_FILTER = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FILTER_MIN_POINT_MAG_MIP_LINEAR: D3D10_FILTER = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FILTER_MIN_LINEAR_MAG_MIP_POINT: D3D10_FILTER = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FILTER_MIN_LINEAR_MAG_POINT_MIP_LINEAR: D3D10_FILTER = 17i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FILTER_MIN_MAG_LINEAR_MIP_POINT: D3D10_FILTER = 20i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FILTER_MIN_MAG_MIP_LINEAR: D3D10_FILTER = 21i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FILTER_ANISOTROPIC: D3D10_FILTER = 85i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FILTER_COMPARISON_MIN_MAG_MIP_POINT: D3D10_FILTER = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FILTER_COMPARISON_MIN_MAG_POINT_MIP_LINEAR: D3D10_FILTER = 129i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FILTER_COMPARISON_MIN_POINT_MAG_LINEAR_MIP_POINT: D3D10_FILTER = 132i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FILTER_COMPARISON_MIN_POINT_MAG_MIP_LINEAR: D3D10_FILTER = 133i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FILTER_COMPARISON_MIN_LINEAR_MAG_MIP_POINT: D3D10_FILTER = 144i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FILTER_COMPARISON_MIN_LINEAR_MAG_POINT_MIP_LINEAR: D3D10_FILTER = 145i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FILTER_COMPARISON_MIN_MAG_LINEAR_MIP_POINT: D3D10_FILTER = 148i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FILTER_COMPARISON_MIN_MAG_MIP_LINEAR: D3D10_FILTER = 149i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FILTER_COMPARISON_ANISOTROPIC: D3D10_FILTER = 213i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FILTER_TEXT_1BIT: D3D10_FILTER = -2147483648i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_FILTER_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FILTER_TYPE_POINT: D3D10_FILTER_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FILTER_TYPE_LINEAR: D3D10_FILTER_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_FORMAT_SUPPORT = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FORMAT_SUPPORT_BUFFER: D3D10_FORMAT_SUPPORT = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FORMAT_SUPPORT_IA_VERTEX_BUFFER: D3D10_FORMAT_SUPPORT = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FORMAT_SUPPORT_IA_INDEX_BUFFER: D3D10_FORMAT_SUPPORT = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FORMAT_SUPPORT_SO_BUFFER: D3D10_FORMAT_SUPPORT = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FORMAT_SUPPORT_TEXTURE1D: D3D10_FORMAT_SUPPORT = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FORMAT_SUPPORT_TEXTURE2D: D3D10_FORMAT_SUPPORT = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FORMAT_SUPPORT_TEXTURE3D: D3D10_FORMAT_SUPPORT = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FORMAT_SUPPORT_TEXTURECUBE: D3D10_FORMAT_SUPPORT = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FORMAT_SUPPORT_SHADER_LOAD: D3D10_FORMAT_SUPPORT = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FORMAT_SUPPORT_SHADER_SAMPLE: D3D10_FORMAT_SUPPORT = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FORMAT_SUPPORT_SHADER_SAMPLE_COMPARISON: D3D10_FORMAT_SUPPORT = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FORMAT_SUPPORT_SHADER_SAMPLE_MONO_TEXT: D3D10_FORMAT_SUPPORT = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FORMAT_SUPPORT_MIP: D3D10_FORMAT_SUPPORT = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FORMAT_SUPPORT_MIP_AUTOGEN: D3D10_FORMAT_SUPPORT = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FORMAT_SUPPORT_RENDER_TARGET: D3D10_FORMAT_SUPPORT = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FORMAT_SUPPORT_BLENDABLE: D3D10_FORMAT_SUPPORT = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FORMAT_SUPPORT_DEPTH_STENCIL: D3D10_FORMAT_SUPPORT = 65536i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FORMAT_SUPPORT_CPU_LOCKABLE: D3D10_FORMAT_SUPPORT = 131072i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FORMAT_SUPPORT_MULTISAMPLE_RESOLVE: D3D10_FORMAT_SUPPORT = 262144i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FORMAT_SUPPORT_DISPLAY: D3D10_FORMAT_SUPPORT = 524288i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FORMAT_SUPPORT_CAST_WITHIN_BIT_LAYOUT: D3D10_FORMAT_SUPPORT = 1048576i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FORMAT_SUPPORT_MULTISAMPLE_RENDERTARGET: D3D10_FORMAT_SUPPORT = 2097152i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FORMAT_SUPPORT_MULTISAMPLE_LOAD: D3D10_FORMAT_SUPPORT = 4194304i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FORMAT_SUPPORT_SHADER_GATHER: D3D10_FORMAT_SUPPORT = 8388608i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_FORMAT_SUPPORT_BACK_BUFFER_CAST: D3D10_FORMAT_SUPPORT = 16777216i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_INPUT_CLASSIFICATION = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_INPUT_PER_VERTEX_DATA: D3D10_INPUT_CLASSIFICATION = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_INPUT_PER_INSTANCE_DATA: D3D10_INPUT_CLASSIFICATION = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_MAP = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MAP_READ: D3D10_MAP = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MAP_WRITE: D3D10_MAP = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MAP_READ_WRITE: D3D10_MAP = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MAP_WRITE_DISCARD: D3D10_MAP = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MAP_WRITE_NO_OVERWRITE: D3D10_MAP = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_MAP_FLAG = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MAP_FLAG_DO_NOT_WAIT: D3D10_MAP_FLAG = 1048576i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_MESSAGE_CATEGORY = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_CATEGORY_APPLICATION_DEFINED: D3D10_MESSAGE_CATEGORY = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_CATEGORY_MISCELLANEOUS: D3D10_MESSAGE_CATEGORY = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_CATEGORY_INITIALIZATION: D3D10_MESSAGE_CATEGORY = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_CATEGORY_CLEANUP: D3D10_MESSAGE_CATEGORY = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_CATEGORY_COMPILATION: D3D10_MESSAGE_CATEGORY = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_CATEGORY_STATE_CREATION: D3D10_MESSAGE_CATEGORY = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_CATEGORY_STATE_SETTING: D3D10_MESSAGE_CATEGORY = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_CATEGORY_STATE_GETTING: D3D10_MESSAGE_CATEGORY = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_CATEGORY_RESOURCE_MANIPULATION: D3D10_MESSAGE_CATEGORY = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_CATEGORY_EXECUTION: D3D10_MESSAGE_CATEGORY = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_CATEGORY_SHADER: D3D10_MESSAGE_CATEGORY = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_MESSAGE_ID = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_UNKNOWN: D3D10_MESSAGE_ID = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_IASETVERTEXBUFFERS_HAZARD: D3D10_MESSAGE_ID = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_IASETINDEXBUFFER_HAZARD: D3D10_MESSAGE_ID = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_VSSETSHADERRESOURCES_HAZARD: D3D10_MESSAGE_ID = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_VSSETCONSTANTBUFFERS_HAZARD: D3D10_MESSAGE_ID = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_GSSETSHADERRESOURCES_HAZARD: D3D10_MESSAGE_ID = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_GSSETCONSTANTBUFFERS_HAZARD: D3D10_MESSAGE_ID = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_PSSETSHADERRESOURCES_HAZARD: D3D10_MESSAGE_ID = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_PSSETCONSTANTBUFFERS_HAZARD: D3D10_MESSAGE_ID = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_OMSETRENDERTARGETS_HAZARD: D3D10_MESSAGE_ID = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_SOSETTARGETS_HAZARD: D3D10_MESSAGE_ID = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_STRING_FROM_APPLICATION: D3D10_MESSAGE_ID = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CORRUPTED_THIS: D3D10_MESSAGE_ID = 12i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CORRUPTED_PARAMETER1: D3D10_MESSAGE_ID = 13i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CORRUPTED_PARAMETER2: D3D10_MESSAGE_ID = 14i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CORRUPTED_PARAMETER3: D3D10_MESSAGE_ID = 15i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CORRUPTED_PARAMETER4: D3D10_MESSAGE_ID = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CORRUPTED_PARAMETER5: D3D10_MESSAGE_ID = 17i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CORRUPTED_PARAMETER6: D3D10_MESSAGE_ID = 18i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CORRUPTED_PARAMETER7: D3D10_MESSAGE_ID = 19i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CORRUPTED_PARAMETER8: D3D10_MESSAGE_ID = 20i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CORRUPTED_PARAMETER9: D3D10_MESSAGE_ID = 21i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CORRUPTED_PARAMETER10: D3D10_MESSAGE_ID = 22i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CORRUPTED_PARAMETER11: D3D10_MESSAGE_ID = 23i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CORRUPTED_PARAMETER12: D3D10_MESSAGE_ID = 24i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CORRUPTED_PARAMETER13: D3D10_MESSAGE_ID = 25i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CORRUPTED_PARAMETER14: D3D10_MESSAGE_ID = 26i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CORRUPTED_PARAMETER15: D3D10_MESSAGE_ID = 27i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CORRUPTED_MULTITHREADING: D3D10_MESSAGE_ID = 28i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_MESSAGE_REPORTING_OUTOFMEMORY: D3D10_MESSAGE_ID = 29i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_IASETINPUTLAYOUT_UNBINDDELETINGOBJECT: D3D10_MESSAGE_ID = 30i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_IASETVERTEXBUFFERS_UNBINDDELETINGOBJECT: D3D10_MESSAGE_ID = 31i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_IASETINDEXBUFFER_UNBINDDELETINGOBJECT: D3D10_MESSAGE_ID = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_VSSETSHADER_UNBINDDELETINGOBJECT: D3D10_MESSAGE_ID = 33i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_VSSETSHADERRESOURCES_UNBINDDELETINGOBJECT: D3D10_MESSAGE_ID = 34i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_VSSETCONSTANTBUFFERS_UNBINDDELETINGOBJECT: D3D10_MESSAGE_ID = 35i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_VSSETSAMPLERS_UNBINDDELETINGOBJECT: D3D10_MESSAGE_ID = 36i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_GSSETSHADER_UNBINDDELETINGOBJECT: D3D10_MESSAGE_ID = 37i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_GSSETSHADERRESOURCES_UNBINDDELETINGOBJECT: D3D10_MESSAGE_ID = 38i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_GSSETCONSTANTBUFFERS_UNBINDDELETINGOBJECT: D3D10_MESSAGE_ID = 39i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_GSSETSAMPLERS_UNBINDDELETINGOBJECT: D3D10_MESSAGE_ID = 40i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_SOSETTARGETS_UNBINDDELETINGOBJECT: D3D10_MESSAGE_ID = 41i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_PSSETSHADER_UNBINDDELETINGOBJECT: D3D10_MESSAGE_ID = 42i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_PSSETSHADERRESOURCES_UNBINDDELETINGOBJECT: D3D10_MESSAGE_ID = 43i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_PSSETCONSTANTBUFFERS_UNBINDDELETINGOBJECT: D3D10_MESSAGE_ID = 44i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_PSSETSAMPLERS_UNBINDDELETINGOBJECT: D3D10_MESSAGE_ID = 45i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_RSSETSTATE_UNBINDDELETINGOBJECT: D3D10_MESSAGE_ID = 46i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_OMSETBLENDSTATE_UNBINDDELETINGOBJECT: D3D10_MESSAGE_ID = 47i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_OMSETDEPTHSTENCILSTATE_UNBINDDELETINGOBJECT: D3D10_MESSAGE_ID = 48i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_OMSETRENDERTARGETS_UNBINDDELETINGOBJECT: D3D10_MESSAGE_ID = 49i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_SETPREDICATION_UNBINDDELETINGOBJECT: D3D10_MESSAGE_ID = 50i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_GETPRIVATEDATA_MOREDATA: D3D10_MESSAGE_ID = 51i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_SETPRIVATEDATA_INVALIDFREEDATA: D3D10_MESSAGE_ID = 52i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_SETPRIVATEDATA_INVALIDIUNKNOWN: D3D10_MESSAGE_ID = 53i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_SETPRIVATEDATA_INVALIDFLAGS: D3D10_MESSAGE_ID = 54i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_SETPRIVATEDATA_CHANGINGPARAMS: D3D10_MESSAGE_ID = 55i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_SETPRIVATEDATA_OUTOFMEMORY: D3D10_MESSAGE_ID = 56i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBUFFER_UNRECOGNIZEDFORMAT: D3D10_MESSAGE_ID = 57i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBUFFER_INVALIDSAMPLES: D3D10_MESSAGE_ID = 58i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBUFFER_UNRECOGNIZEDUSAGE: D3D10_MESSAGE_ID = 59i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBUFFER_UNRECOGNIZEDBINDFLAGS: D3D10_MESSAGE_ID = 60i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBUFFER_UNRECOGNIZEDCPUACCESSFLAGS: D3D10_MESSAGE_ID = 61i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBUFFER_UNRECOGNIZEDMISCFLAGS: D3D10_MESSAGE_ID = 62i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBUFFER_INVALIDCPUACCESSFLAGS: D3D10_MESSAGE_ID = 63i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBUFFER_INVALIDBINDFLAGS: D3D10_MESSAGE_ID = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBUFFER_INVALIDINITIALDATA: D3D10_MESSAGE_ID = 65i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBUFFER_INVALIDDIMENSIONS: D3D10_MESSAGE_ID = 66i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBUFFER_INVALIDMIPLEVELS: D3D10_MESSAGE_ID = 67i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBUFFER_INVALIDMISCFLAGS: D3D10_MESSAGE_ID = 68i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBUFFER_INVALIDARG_RETURN: D3D10_MESSAGE_ID = 69i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBUFFER_OUTOFMEMORY_RETURN: D3D10_MESSAGE_ID = 70i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBUFFER_NULLDESC: D3D10_MESSAGE_ID = 71i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBUFFER_INVALIDCONSTANTBUFFERBINDINGS: D3D10_MESSAGE_ID = 72i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBUFFER_LARGEALLOCATION: D3D10_MESSAGE_ID = 73i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE1D_UNRECOGNIZEDFORMAT: D3D10_MESSAGE_ID = 74i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE1D_UNSUPPORTEDFORMAT: D3D10_MESSAGE_ID = 75i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE1D_INVALIDSAMPLES: D3D10_MESSAGE_ID = 76i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE1D_UNRECOGNIZEDUSAGE: D3D10_MESSAGE_ID = 77i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE1D_UNRECOGNIZEDBINDFLAGS: D3D10_MESSAGE_ID = 78i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE1D_UNRECOGNIZEDCPUACCESSFLAGS: D3D10_MESSAGE_ID = 79i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE1D_UNRECOGNIZEDMISCFLAGS: D3D10_MESSAGE_ID = 80i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE1D_INVALIDCPUACCESSFLAGS: D3D10_MESSAGE_ID = 81i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE1D_INVALIDBINDFLAGS: D3D10_MESSAGE_ID = 82i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE1D_INVALIDINITIALDATA: D3D10_MESSAGE_ID = 83i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE1D_INVALIDDIMENSIONS: D3D10_MESSAGE_ID = 84i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE1D_INVALIDMIPLEVELS: D3D10_MESSAGE_ID = 85i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE1D_INVALIDMISCFLAGS: D3D10_MESSAGE_ID = 86i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE1D_INVALIDARG_RETURN: D3D10_MESSAGE_ID = 87i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE1D_OUTOFMEMORY_RETURN: D3D10_MESSAGE_ID = 88i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE1D_NULLDESC: D3D10_MESSAGE_ID = 89i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE1D_LARGEALLOCATION: D3D10_MESSAGE_ID = 90i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE2D_UNRECOGNIZEDFORMAT: D3D10_MESSAGE_ID = 91i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE2D_UNSUPPORTEDFORMAT: D3D10_MESSAGE_ID = 92i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE2D_INVALIDSAMPLES: D3D10_MESSAGE_ID = 93i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE2D_UNRECOGNIZEDUSAGE: D3D10_MESSAGE_ID = 94i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE2D_UNRECOGNIZEDBINDFLAGS: D3D10_MESSAGE_ID = 95i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE2D_UNRECOGNIZEDCPUACCESSFLAGS: D3D10_MESSAGE_ID = 96i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE2D_UNRECOGNIZEDMISCFLAGS: D3D10_MESSAGE_ID = 97i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE2D_INVALIDCPUACCESSFLAGS: D3D10_MESSAGE_ID = 98i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE2D_INVALIDBINDFLAGS: D3D10_MESSAGE_ID = 99i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE2D_INVALIDINITIALDATA: D3D10_MESSAGE_ID = 100i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE2D_INVALIDDIMENSIONS: D3D10_MESSAGE_ID = 101i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE2D_INVALIDMIPLEVELS: D3D10_MESSAGE_ID = 102i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE2D_INVALIDMISCFLAGS: D3D10_MESSAGE_ID = 103i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE2D_INVALIDARG_RETURN: D3D10_MESSAGE_ID = 104i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE2D_OUTOFMEMORY_RETURN: D3D10_MESSAGE_ID = 105i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE2D_NULLDESC: D3D10_MESSAGE_ID = 106i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE2D_LARGEALLOCATION: D3D10_MESSAGE_ID = 107i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE3D_UNRECOGNIZEDFORMAT: D3D10_MESSAGE_ID = 108i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE3D_UNSUPPORTEDFORMAT: D3D10_MESSAGE_ID = 109i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE3D_INVALIDSAMPLES: D3D10_MESSAGE_ID = 110i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE3D_UNRECOGNIZEDUSAGE: D3D10_MESSAGE_ID = 111i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE3D_UNRECOGNIZEDBINDFLAGS: D3D10_MESSAGE_ID = 112i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE3D_UNRECOGNIZEDCPUACCESSFLAGS: D3D10_MESSAGE_ID = 113i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE3D_UNRECOGNIZEDMISCFLAGS: D3D10_MESSAGE_ID = 114i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE3D_INVALIDCPUACCESSFLAGS: D3D10_MESSAGE_ID = 115i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE3D_INVALIDBINDFLAGS: D3D10_MESSAGE_ID = 116i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE3D_INVALIDINITIALDATA: D3D10_MESSAGE_ID = 117i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE3D_INVALIDDIMENSIONS: D3D10_MESSAGE_ID = 118i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE3D_INVALIDMIPLEVELS: D3D10_MESSAGE_ID = 119i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE3D_INVALIDMISCFLAGS: D3D10_MESSAGE_ID = 120i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE3D_INVALIDARG_RETURN: D3D10_MESSAGE_ID = 121i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE3D_OUTOFMEMORY_RETURN: D3D10_MESSAGE_ID = 122i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE3D_NULLDESC: D3D10_MESSAGE_ID = 123i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATETEXTURE3D_LARGEALLOCATION: D3D10_MESSAGE_ID = 124i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATESHADERRESOURCEVIEW_UNRECOGNIZEDFORMAT: D3D10_MESSAGE_ID = 125i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATESHADERRESOURCEVIEW_INVALIDDESC: D3D10_MESSAGE_ID = 126i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATESHADERRESOURCEVIEW_INVALIDFORMAT: D3D10_MESSAGE_ID = 127i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATESHADERRESOURCEVIEW_INVALIDDIMENSIONS: D3D10_MESSAGE_ID = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATESHADERRESOURCEVIEW_INVALIDRESOURCE: D3D10_MESSAGE_ID = 129i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATESHADERRESOURCEVIEW_TOOMANYOBJECTS: D3D10_MESSAGE_ID = 130i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATESHADERRESOURCEVIEW_INVALIDARG_RETURN: D3D10_MESSAGE_ID = 131i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATESHADERRESOURCEVIEW_OUTOFMEMORY_RETURN: D3D10_MESSAGE_ID = 132i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERENDERTARGETVIEW_UNRECOGNIZEDFORMAT: D3D10_MESSAGE_ID = 133i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERENDERTARGETVIEW_UNSUPPORTEDFORMAT: D3D10_MESSAGE_ID = 134i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERENDERTARGETVIEW_INVALIDDESC: D3D10_MESSAGE_ID = 135i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERENDERTARGETVIEW_INVALIDFORMAT: D3D10_MESSAGE_ID = 136i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERENDERTARGETVIEW_INVALIDDIMENSIONS: D3D10_MESSAGE_ID = 137i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERENDERTARGETVIEW_INVALIDRESOURCE: D3D10_MESSAGE_ID = 138i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERENDERTARGETVIEW_TOOMANYOBJECTS: D3D10_MESSAGE_ID = 139i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERENDERTARGETVIEW_INVALIDARG_RETURN: D3D10_MESSAGE_ID = 140i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERENDERTARGETVIEW_OUTOFMEMORY_RETURN: D3D10_MESSAGE_ID = 141i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEDEPTHSTENCILVIEW_UNRECOGNIZEDFORMAT: D3D10_MESSAGE_ID = 142i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEDEPTHSTENCILVIEW_INVALIDDESC: D3D10_MESSAGE_ID = 143i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEDEPTHSTENCILVIEW_INVALIDFORMAT: D3D10_MESSAGE_ID = 144i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEDEPTHSTENCILVIEW_INVALIDDIMENSIONS: D3D10_MESSAGE_ID = 145i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEDEPTHSTENCILVIEW_INVALIDRESOURCE: D3D10_MESSAGE_ID = 146i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEDEPTHSTENCILVIEW_TOOMANYOBJECTS: D3D10_MESSAGE_ID = 147i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEDEPTHSTENCILVIEW_INVALIDARG_RETURN: D3D10_MESSAGE_ID = 148i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEDEPTHSTENCILVIEW_OUTOFMEMORY_RETURN: D3D10_MESSAGE_ID = 149i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEINPUTLAYOUT_OUTOFMEMORY: D3D10_MESSAGE_ID = 150i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEINPUTLAYOUT_TOOMANYELEMENTS: D3D10_MESSAGE_ID = 151i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEINPUTLAYOUT_INVALIDFORMAT: D3D10_MESSAGE_ID = 152i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEINPUTLAYOUT_INCOMPATIBLEFORMAT: D3D10_MESSAGE_ID = 153i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEINPUTLAYOUT_INVALIDSLOT: D3D10_MESSAGE_ID = 154i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEINPUTLAYOUT_INVALIDINPUTSLOTCLASS: D3D10_MESSAGE_ID = 155i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEINPUTLAYOUT_STEPRATESLOTCLASSMISMATCH: D3D10_MESSAGE_ID = 156i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEINPUTLAYOUT_INVALIDSLOTCLASSCHANGE: D3D10_MESSAGE_ID = 157i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEINPUTLAYOUT_INVALIDSTEPRATECHANGE: D3D10_MESSAGE_ID = 158i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEINPUTLAYOUT_INVALIDALIGNMENT: D3D10_MESSAGE_ID = 159i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEINPUTLAYOUT_DUPLICATESEMANTIC: D3D10_MESSAGE_ID = 160i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEINPUTLAYOUT_UNPARSEABLEINPUTSIGNATURE: D3D10_MESSAGE_ID = 161i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEINPUTLAYOUT_NULLSEMANTIC: D3D10_MESSAGE_ID = 162i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEINPUTLAYOUT_MISSINGELEMENT: D3D10_MESSAGE_ID = 163i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEINPUTLAYOUT_NULLDESC: D3D10_MESSAGE_ID = 164i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEVERTEXSHADER_OUTOFMEMORY: D3D10_MESSAGE_ID = 165i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEVERTEXSHADER_INVALIDSHADERBYTECODE: D3D10_MESSAGE_ID = 166i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEVERTEXSHADER_INVALIDSHADERTYPE: D3D10_MESSAGE_ID = 167i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEGEOMETRYSHADER_OUTOFMEMORY: D3D10_MESSAGE_ID = 168i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEGEOMETRYSHADER_INVALIDSHADERBYTECODE: D3D10_MESSAGE_ID = 169i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEGEOMETRYSHADER_INVALIDSHADERTYPE: D3D10_MESSAGE_ID = 170i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_OUTOFMEMORY: D3D10_MESSAGE_ID = 171i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_INVALIDSHADERBYTECODE: D3D10_MESSAGE_ID = 172i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_INVALIDSHADERTYPE: D3D10_MESSAGE_ID = 173i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_INVALIDNUMENTRIES: D3D10_MESSAGE_ID = 174i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_OUTPUTSTREAMSTRIDEUNUSED: D3D10_MESSAGE_ID = 175i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_UNEXPECTEDDECL: D3D10_MESSAGE_ID = 176i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_EXPECTEDDECL: D3D10_MESSAGE_ID = 177i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_OUTPUTSLOT0EXPECTED: D3D10_MESSAGE_ID = 178i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_INVALIDOUTPUTSLOT: D3D10_MESSAGE_ID = 179i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_ONLYONEELEMENTPERSLOT: D3D10_MESSAGE_ID = 180i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_INVALIDCOMPONENTCOUNT: D3D10_MESSAGE_ID = 181i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_INVALIDSTARTCOMPONENTANDCOMPONENTCOUNT: D3D10_MESSAGE_ID = 182i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_INVALIDGAPDEFINITION: D3D10_MESSAGE_ID = 183i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_REPEATEDOUTPUT: D3D10_MESSAGE_ID = 184i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_INVALIDOUTPUTSTREAMSTRIDE: D3D10_MESSAGE_ID = 185i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_MISSINGSEMANTIC: D3D10_MESSAGE_ID = 186i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_MASKMISMATCH: D3D10_MESSAGE_ID = 187i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_CANTHAVEONLYGAPS: D3D10_MESSAGE_ID = 188i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_DECLTOOCOMPLEX: D3D10_MESSAGE_ID = 189i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_MISSINGOUTPUTSIGNATURE: D3D10_MESSAGE_ID = 190i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEPIXELSHADER_OUTOFMEMORY: D3D10_MESSAGE_ID = 191i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEPIXELSHADER_INVALIDSHADERBYTECODE: D3D10_MESSAGE_ID = 192i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEPIXELSHADER_INVALIDSHADERTYPE: D3D10_MESSAGE_ID = 193i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERASTERIZERSTATE_INVALIDFILLMODE: D3D10_MESSAGE_ID = 194i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERASTERIZERSTATE_INVALIDCULLMODE: D3D10_MESSAGE_ID = 195i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERASTERIZERSTATE_INVALIDDEPTHBIASCLAMP: D3D10_MESSAGE_ID = 196i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERASTERIZERSTATE_INVALIDSLOPESCALEDDEPTHBIAS: D3D10_MESSAGE_ID = 197i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERASTERIZERSTATE_TOOMANYOBJECTS: D3D10_MESSAGE_ID = 198i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERASTERIZERSTATE_NULLDESC: D3D10_MESSAGE_ID = 199i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEDEPTHSTENCILSTATE_INVALIDDEPTHWRITEMASK: D3D10_MESSAGE_ID = 200i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEDEPTHSTENCILSTATE_INVALIDDEPTHFUNC: D3D10_MESSAGE_ID = 201i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEDEPTHSTENCILSTATE_INVALIDFRONTFACESTENCILFAILOP: D3D10_MESSAGE_ID = 202i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEDEPTHSTENCILSTATE_INVALIDFRONTFACESTENCILZFAILOP: D3D10_MESSAGE_ID = 203i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEDEPTHSTENCILSTATE_INVALIDFRONTFACESTENCILPASSOP: D3D10_MESSAGE_ID = 204i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEDEPTHSTENCILSTATE_INVALIDFRONTFACESTENCILFUNC: D3D10_MESSAGE_ID = 205i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEDEPTHSTENCILSTATE_INVALIDBACKFACESTENCILFAILOP: D3D10_MESSAGE_ID = 206i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEDEPTHSTENCILSTATE_INVALIDBACKFACESTENCILZFAILOP: D3D10_MESSAGE_ID = 207i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEDEPTHSTENCILSTATE_INVALIDBACKFACESTENCILPASSOP: D3D10_MESSAGE_ID = 208i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEDEPTHSTENCILSTATE_INVALIDBACKFACESTENCILFUNC: D3D10_MESSAGE_ID = 209i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEDEPTHSTENCILSTATE_TOOMANYOBJECTS: D3D10_MESSAGE_ID = 210i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEDEPTHSTENCILSTATE_NULLDESC: D3D10_MESSAGE_ID = 211i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBLENDSTATE_INVALIDSRCBLEND: D3D10_MESSAGE_ID = 212i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBLENDSTATE_INVALIDDESTBLEND: D3D10_MESSAGE_ID = 213i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBLENDSTATE_INVALIDBLENDOP: D3D10_MESSAGE_ID = 214i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBLENDSTATE_INVALIDSRCBLENDALPHA: D3D10_MESSAGE_ID = 215i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBLENDSTATE_INVALIDDESTBLENDALPHA: D3D10_MESSAGE_ID = 216i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBLENDSTATE_INVALIDBLENDOPALPHA: D3D10_MESSAGE_ID = 217i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBLENDSTATE_INVALIDRENDERTARGETWRITEMASK: D3D10_MESSAGE_ID = 218i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBLENDSTATE_TOOMANYOBJECTS: D3D10_MESSAGE_ID = 219i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBLENDSTATE_NULLDESC: D3D10_MESSAGE_ID = 220i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATESAMPLERSTATE_INVALIDFILTER: D3D10_MESSAGE_ID = 221i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATESAMPLERSTATE_INVALIDADDRESSU: D3D10_MESSAGE_ID = 222i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATESAMPLERSTATE_INVALIDADDRESSV: D3D10_MESSAGE_ID = 223i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATESAMPLERSTATE_INVALIDADDRESSW: D3D10_MESSAGE_ID = 224i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATESAMPLERSTATE_INVALIDMIPLODBIAS: D3D10_MESSAGE_ID = 225i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATESAMPLERSTATE_INVALIDMAXANISOTROPY: D3D10_MESSAGE_ID = 226i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATESAMPLERSTATE_INVALIDCOMPARISONFUNC: D3D10_MESSAGE_ID = 227i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATESAMPLERSTATE_INVALIDMINLOD: D3D10_MESSAGE_ID = 228i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATESAMPLERSTATE_INVALIDMAXLOD: D3D10_MESSAGE_ID = 229i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATESAMPLERSTATE_TOOMANYOBJECTS: D3D10_MESSAGE_ID = 230i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATESAMPLERSTATE_NULLDESC: D3D10_MESSAGE_ID = 231i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEQUERYORPREDICATE_INVALIDQUERY: D3D10_MESSAGE_ID = 232i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEQUERYORPREDICATE_INVALIDMISCFLAGS: D3D10_MESSAGE_ID = 233i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEQUERYORPREDICATE_UNEXPECTEDMISCFLAG: D3D10_MESSAGE_ID = 234i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEQUERYORPREDICATE_NULLDESC: D3D10_MESSAGE_ID = 235i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_IASETPRIMITIVETOPOLOGY_TOPOLOGY_UNRECOGNIZED: D3D10_MESSAGE_ID = 236i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_IASETPRIMITIVETOPOLOGY_TOPOLOGY_UNDEFINED: D3D10_MESSAGE_ID = 237i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_IASETVERTEXBUFFERS_INVALIDBUFFER: D3D10_MESSAGE_ID = 238i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_IASETVERTEXBUFFERS_OFFSET_TOO_LARGE: D3D10_MESSAGE_ID = 239i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_IASETVERTEXBUFFERS_BUFFERS_EMPTY: D3D10_MESSAGE_ID = 240i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_IASETINDEXBUFFER_INVALIDBUFFER: D3D10_MESSAGE_ID = 241i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_IASETINDEXBUFFER_FORMAT_INVALID: D3D10_MESSAGE_ID = 242i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_IASETINDEXBUFFER_OFFSET_TOO_LARGE: D3D10_MESSAGE_ID = 243i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_IASETINDEXBUFFER_OFFSET_UNALIGNED: D3D10_MESSAGE_ID = 244i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_VSSETSHADERRESOURCES_VIEWS_EMPTY: D3D10_MESSAGE_ID = 245i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_VSSETCONSTANTBUFFERS_INVALIDBUFFER: D3D10_MESSAGE_ID = 246i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_VSSETCONSTANTBUFFERS_BUFFERS_EMPTY: D3D10_MESSAGE_ID = 247i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_VSSETSAMPLERS_SAMPLERS_EMPTY: D3D10_MESSAGE_ID = 248i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_GSSETSHADERRESOURCES_VIEWS_EMPTY: D3D10_MESSAGE_ID = 249i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_GSSETCONSTANTBUFFERS_INVALIDBUFFER: D3D10_MESSAGE_ID = 250i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_GSSETCONSTANTBUFFERS_BUFFERS_EMPTY: D3D10_MESSAGE_ID = 251i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_GSSETSAMPLERS_SAMPLERS_EMPTY: D3D10_MESSAGE_ID = 252i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_SOSETTARGETS_INVALIDBUFFER: D3D10_MESSAGE_ID = 253i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_SOSETTARGETS_OFFSET_UNALIGNED: D3D10_MESSAGE_ID = 254i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_PSSETSHADERRESOURCES_VIEWS_EMPTY: D3D10_MESSAGE_ID = 255i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_PSSETCONSTANTBUFFERS_INVALIDBUFFER: D3D10_MESSAGE_ID = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_PSSETCONSTANTBUFFERS_BUFFERS_EMPTY: D3D10_MESSAGE_ID = 257i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_PSSETSAMPLERS_SAMPLERS_EMPTY: D3D10_MESSAGE_ID = 258i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_RSSETVIEWPORTS_INVALIDVIEWPORT: D3D10_MESSAGE_ID = 259i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_RSSETSCISSORRECTS_INVALIDSCISSOR: D3D10_MESSAGE_ID = 260i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CLEARRENDERTARGETVIEW_DENORMFLUSH: D3D10_MESSAGE_ID = 261i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CLEARDEPTHSTENCILVIEW_DENORMFLUSH: D3D10_MESSAGE_ID = 262i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CLEARDEPTHSTENCILVIEW_INVALID: D3D10_MESSAGE_ID = 263i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_IAGETVERTEXBUFFERS_BUFFERS_EMPTY: D3D10_MESSAGE_ID = 264i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_VSGETSHADERRESOURCES_VIEWS_EMPTY: D3D10_MESSAGE_ID = 265i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_VSGETCONSTANTBUFFERS_BUFFERS_EMPTY: D3D10_MESSAGE_ID = 266i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_VSGETSAMPLERS_SAMPLERS_EMPTY: D3D10_MESSAGE_ID = 267i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_GSGETSHADERRESOURCES_VIEWS_EMPTY: D3D10_MESSAGE_ID = 268i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_GSGETCONSTANTBUFFERS_BUFFERS_EMPTY: D3D10_MESSAGE_ID = 269i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_GSGETSAMPLERS_SAMPLERS_EMPTY: D3D10_MESSAGE_ID = 270i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_SOGETTARGETS_BUFFERS_EMPTY: D3D10_MESSAGE_ID = 271i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_PSGETSHADERRESOURCES_VIEWS_EMPTY: D3D10_MESSAGE_ID = 272i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_PSGETCONSTANTBUFFERS_BUFFERS_EMPTY: D3D10_MESSAGE_ID = 273i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_PSGETSAMPLERS_SAMPLERS_EMPTY: D3D10_MESSAGE_ID = 274i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_RSGETVIEWPORTS_VIEWPORTS_EMPTY: D3D10_MESSAGE_ID = 275i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_RSGETSCISSORRECTS_RECTS_EMPTY: D3D10_MESSAGE_ID = 276i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_GENERATEMIPS_RESOURCE_INVALID: D3D10_MESSAGE_ID = 277i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_COPYSUBRESOURCEREGION_INVALIDDESTINATIONSUBRESOURCE: D3D10_MESSAGE_ID = 278i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_COPYSUBRESOURCEREGION_INVALIDSOURCESUBRESOURCE: D3D10_MESSAGE_ID = 279i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_COPYSUBRESOURCEREGION_INVALIDSOURCEBOX: D3D10_MESSAGE_ID = 280i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_COPYSUBRESOURCEREGION_INVALIDSOURCE: D3D10_MESSAGE_ID = 281i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_COPYSUBRESOURCEREGION_INVALIDDESTINATIONSTATE: D3D10_MESSAGE_ID = 282i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_COPYSUBRESOURCEREGION_INVALIDSOURCESTATE: D3D10_MESSAGE_ID = 283i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_COPYRESOURCE_INVALIDSOURCE: D3D10_MESSAGE_ID = 284i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_COPYRESOURCE_INVALIDDESTINATIONSTATE: D3D10_MESSAGE_ID = 285i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_COPYRESOURCE_INVALIDSOURCESTATE: D3D10_MESSAGE_ID = 286i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_UPDATESUBRESOURCE_INVALIDDESTINATIONSUBRESOURCE: D3D10_MESSAGE_ID = 287i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_UPDATESUBRESOURCE_INVALIDDESTINATIONBOX: D3D10_MESSAGE_ID = 288i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_UPDATESUBRESOURCE_INVALIDDESTINATIONSTATE: D3D10_MESSAGE_ID = 289i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_RESOLVESUBRESOURCE_DESTINATION_INVALID: D3D10_MESSAGE_ID = 290i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_RESOLVESUBRESOURCE_DESTINATION_SUBRESOURCE_INVALID: D3D10_MESSAGE_ID = 291i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_RESOLVESUBRESOURCE_SOURCE_INVALID: D3D10_MESSAGE_ID = 292i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_RESOLVESUBRESOURCE_SOURCE_SUBRESOURCE_INVALID: D3D10_MESSAGE_ID = 293i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_RESOLVESUBRESOURCE_FORMAT_INVALID: D3D10_MESSAGE_ID = 294i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_BUFFER_MAP_INVALIDMAPTYPE: D3D10_MESSAGE_ID = 295i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_BUFFER_MAP_INVALIDFLAGS: D3D10_MESSAGE_ID = 296i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_BUFFER_MAP_ALREADYMAPPED: D3D10_MESSAGE_ID = 297i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_BUFFER_MAP_DEVICEREMOVED_RETURN: D3D10_MESSAGE_ID = 298i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_BUFFER_UNMAP_NOTMAPPED: D3D10_MESSAGE_ID = 299i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_TEXTURE1D_MAP_INVALIDMAPTYPE: D3D10_MESSAGE_ID = 300i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_TEXTURE1D_MAP_INVALIDSUBRESOURCE: D3D10_MESSAGE_ID = 301i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_TEXTURE1D_MAP_INVALIDFLAGS: D3D10_MESSAGE_ID = 302i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_TEXTURE1D_MAP_ALREADYMAPPED: D3D10_MESSAGE_ID = 303i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_TEXTURE1D_MAP_DEVICEREMOVED_RETURN: D3D10_MESSAGE_ID = 304i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_TEXTURE1D_UNMAP_INVALIDSUBRESOURCE: D3D10_MESSAGE_ID = 305i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_TEXTURE1D_UNMAP_NOTMAPPED: D3D10_MESSAGE_ID = 306i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_TEXTURE2D_MAP_INVALIDMAPTYPE: D3D10_MESSAGE_ID = 307i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_TEXTURE2D_MAP_INVALIDSUBRESOURCE: D3D10_MESSAGE_ID = 308i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_TEXTURE2D_MAP_INVALIDFLAGS: D3D10_MESSAGE_ID = 309i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_TEXTURE2D_MAP_ALREADYMAPPED: D3D10_MESSAGE_ID = 310i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_TEXTURE2D_MAP_DEVICEREMOVED_RETURN: D3D10_MESSAGE_ID = 311i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_TEXTURE2D_UNMAP_INVALIDSUBRESOURCE: D3D10_MESSAGE_ID = 312i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_TEXTURE2D_UNMAP_NOTMAPPED: D3D10_MESSAGE_ID = 313i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_TEXTURE3D_MAP_INVALIDMAPTYPE: D3D10_MESSAGE_ID = 314i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_TEXTURE3D_MAP_INVALIDSUBRESOURCE: D3D10_MESSAGE_ID = 315i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_TEXTURE3D_MAP_INVALIDFLAGS: D3D10_MESSAGE_ID = 316i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_TEXTURE3D_MAP_ALREADYMAPPED: D3D10_MESSAGE_ID = 317i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_TEXTURE3D_MAP_DEVICEREMOVED_RETURN: D3D10_MESSAGE_ID = 318i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_TEXTURE3D_UNMAP_INVALIDSUBRESOURCE: D3D10_MESSAGE_ID = 319i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_TEXTURE3D_UNMAP_NOTMAPPED: D3D10_MESSAGE_ID = 320i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CHECKFORMATSUPPORT_FORMAT_DEPRECATED: D3D10_MESSAGE_ID = 321i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CHECKMULTISAMPLEQUALITYLEVELS_FORMAT_DEPRECATED: D3D10_MESSAGE_ID = 322i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_SETEXCEPTIONMODE_UNRECOGNIZEDFLAGS: D3D10_MESSAGE_ID = 323i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_SETEXCEPTIONMODE_INVALIDARG_RETURN: D3D10_MESSAGE_ID = 324i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_SETEXCEPTIONMODE_DEVICEREMOVED_RETURN: D3D10_MESSAGE_ID = 325i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_REF_SIMULATING_INFINITELY_FAST_HARDWARE: D3D10_MESSAGE_ID = 326i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_REF_THREADING_MODE: D3D10_MESSAGE_ID = 327i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_REF_UMDRIVER_EXCEPTION: D3D10_MESSAGE_ID = 328i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_REF_KMDRIVER_EXCEPTION: D3D10_MESSAGE_ID = 329i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_REF_HARDWARE_EXCEPTION: D3D10_MESSAGE_ID = 330i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_REF_ACCESSING_INDEXABLE_TEMP_OUT_OF_RANGE: D3D10_MESSAGE_ID = 331i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_REF_PROBLEM_PARSING_SHADER: D3D10_MESSAGE_ID = 332i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_REF_OUT_OF_MEMORY: D3D10_MESSAGE_ID = 333i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_REF_INFO: D3D10_MESSAGE_ID = 334i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_VERTEXPOS_OVERFLOW: D3D10_MESSAGE_ID = 335i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAWINDEXED_INDEXPOS_OVERFLOW: D3D10_MESSAGE_ID = 336i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAWINSTANCED_VERTEXPOS_OVERFLOW: D3D10_MESSAGE_ID = 337i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAWINSTANCED_INSTANCEPOS_OVERFLOW: D3D10_MESSAGE_ID = 338i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAWINDEXEDINSTANCED_INSTANCEPOS_OVERFLOW: D3D10_MESSAGE_ID = 339i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAWINDEXEDINSTANCED_INDEXPOS_OVERFLOW: D3D10_MESSAGE_ID = 340i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_VERTEX_SHADER_NOT_SET: D3D10_MESSAGE_ID = 341i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_SHADER_LINKAGE_SEMANTICNAME_NOT_FOUND: D3D10_MESSAGE_ID = 342i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_SHADER_LINKAGE_REGISTERINDEX: D3D10_MESSAGE_ID = 343i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_SHADER_LINKAGE_COMPONENTTYPE: D3D10_MESSAGE_ID = 344i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_SHADER_LINKAGE_REGISTERMASK: D3D10_MESSAGE_ID = 345i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_SHADER_LINKAGE_SYSTEMVALUE: D3D10_MESSAGE_ID = 346i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_SHADER_LINKAGE_NEVERWRITTEN_ALWAYSREADS: D3D10_MESSAGE_ID = 347i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_VERTEX_BUFFER_NOT_SET: D3D10_MESSAGE_ID = 348i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_INPUTLAYOUT_NOT_SET: D3D10_MESSAGE_ID = 349i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_CONSTANT_BUFFER_NOT_SET: D3D10_MESSAGE_ID = 350i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_CONSTANT_BUFFER_TOO_SMALL: D3D10_MESSAGE_ID = 351i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_SAMPLER_NOT_SET: D3D10_MESSAGE_ID = 352i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_SHADERRESOURCEVIEW_NOT_SET: D3D10_MESSAGE_ID = 353i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_VIEW_DIMENSION_MISMATCH: D3D10_MESSAGE_ID = 354i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_VERTEX_BUFFER_STRIDE_TOO_SMALL: D3D10_MESSAGE_ID = 355i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_VERTEX_BUFFER_TOO_SMALL: D3D10_MESSAGE_ID = 356i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_INDEX_BUFFER_NOT_SET: D3D10_MESSAGE_ID = 357i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_INDEX_BUFFER_FORMAT_INVALID: D3D10_MESSAGE_ID = 358i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_INDEX_BUFFER_TOO_SMALL: D3D10_MESSAGE_ID = 359i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_GS_INPUT_PRIMITIVE_MISMATCH: D3D10_MESSAGE_ID = 360i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_RESOURCE_RETURN_TYPE_MISMATCH: D3D10_MESSAGE_ID = 361i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_POSITION_NOT_PRESENT: D3D10_MESSAGE_ID = 362i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_OUTPUT_STREAM_NOT_SET: D3D10_MESSAGE_ID = 363i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_BOUND_RESOURCE_MAPPED: D3D10_MESSAGE_ID = 364i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_INVALID_PRIMITIVETOPOLOGY: D3D10_MESSAGE_ID = 365i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_VERTEX_OFFSET_UNALIGNED: D3D10_MESSAGE_ID = 366i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_VERTEX_STRIDE_UNALIGNED: D3D10_MESSAGE_ID = 367i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_INDEX_OFFSET_UNALIGNED: D3D10_MESSAGE_ID = 368i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_OUTPUT_STREAM_OFFSET_UNALIGNED: D3D10_MESSAGE_ID = 369i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_RESOURCE_FORMAT_LD_UNSUPPORTED: D3D10_MESSAGE_ID = 370i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_RESOURCE_FORMAT_SAMPLE_UNSUPPORTED: D3D10_MESSAGE_ID = 371i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_RESOURCE_FORMAT_SAMPLE_C_UNSUPPORTED: D3D10_MESSAGE_ID = 372i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_RESOURCE_MULTISAMPLE_UNSUPPORTED: D3D10_MESSAGE_ID = 373i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_SO_TARGETS_BOUND_WITHOUT_SOURCE: D3D10_MESSAGE_ID = 374i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_SO_STRIDE_LARGER_THAN_BUFFER: D3D10_MESSAGE_ID = 375i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_OM_RENDER_TARGET_DOES_NOT_SUPPORT_BLENDING: D3D10_MESSAGE_ID = 376i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_OM_DUAL_SOURCE_BLENDING_CAN_ONLY_HAVE_RENDER_TARGET_0: D3D10_MESSAGE_ID = 377i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_REMOVAL_PROCESS_AT_FAULT: D3D10_MESSAGE_ID = 378i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_REMOVAL_PROCESS_POSSIBLY_AT_FAULT: D3D10_MESSAGE_ID = 379i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_REMOVAL_PROCESS_NOT_AT_FAULT: D3D10_MESSAGE_ID = 380i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_OPEN_SHARED_RESOURCE_INVALIDARG_RETURN: D3D10_MESSAGE_ID = 381i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_OPEN_SHARED_RESOURCE_OUTOFMEMORY_RETURN: D3D10_MESSAGE_ID = 382i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_OPEN_SHARED_RESOURCE_BADINTERFACE_RETURN: D3D10_MESSAGE_ID = 383i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_VIEWPORT_NOT_SET: D3D10_MESSAGE_ID = 384i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEINPUTLAYOUT_TRAILING_DIGIT_IN_SEMANTIC: D3D10_MESSAGE_ID = 385i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_TRAILING_DIGIT_IN_SEMANTIC: D3D10_MESSAGE_ID = 386i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_RSSETVIEWPORTS_DENORMFLUSH: D3D10_MESSAGE_ID = 387i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_OMSETRENDERTARGETS_INVALIDVIEW: D3D10_MESSAGE_ID = 388i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_SETTEXTFILTERSIZE_INVALIDDIMENSIONS: D3D10_MESSAGE_ID = 389i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_SAMPLER_MISMATCH: D3D10_MESSAGE_ID = 390i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEINPUTLAYOUT_TYPE_MISMATCH: D3D10_MESSAGE_ID = 391i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_BLENDSTATE_GETDESC_LEGACY: D3D10_MESSAGE_ID = 392i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_SHADERRESOURCEVIEW_GETDESC_LEGACY: D3D10_MESSAGE_ID = 393i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEQUERY_OUTOFMEMORY_RETURN: D3D10_MESSAGE_ID = 394i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEPREDICATE_OUTOFMEMORY_RETURN: D3D10_MESSAGE_ID = 395i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATECOUNTER_OUTOFRANGE_COUNTER: D3D10_MESSAGE_ID = 396i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATECOUNTER_SIMULTANEOUS_ACTIVE_COUNTERS_EXHAUSTED: D3D10_MESSAGE_ID = 397i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATECOUNTER_UNSUPPORTED_WELLKNOWN_COUNTER: D3D10_MESSAGE_ID = 398i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATECOUNTER_OUTOFMEMORY_RETURN: D3D10_MESSAGE_ID = 399i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATECOUNTER_NONEXCLUSIVE_RETURN: D3D10_MESSAGE_ID = 400i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATECOUNTER_NULLDESC: D3D10_MESSAGE_ID = 401i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CHECKCOUNTER_OUTOFRANGE_COUNTER: D3D10_MESSAGE_ID = 402i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CHECKCOUNTER_UNSUPPORTED_WELLKNOWN_COUNTER: D3D10_MESSAGE_ID = 403i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_SETPREDICATION_INVALID_PREDICATE_STATE: D3D10_MESSAGE_ID = 404i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_QUERY_BEGIN_UNSUPPORTED: D3D10_MESSAGE_ID = 405i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_PREDICATE_BEGIN_DURING_PREDICATION: D3D10_MESSAGE_ID = 406i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_QUERY_BEGIN_DUPLICATE: D3D10_MESSAGE_ID = 407i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_QUERY_BEGIN_ABANDONING_PREVIOUS_RESULTS: D3D10_MESSAGE_ID = 408i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_PREDICATE_END_DURING_PREDICATION: D3D10_MESSAGE_ID = 409i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_QUERY_END_ABANDONING_PREVIOUS_RESULTS: D3D10_MESSAGE_ID = 410i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_QUERY_END_WITHOUT_BEGIN: D3D10_MESSAGE_ID = 411i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_QUERY_GETDATA_INVALID_DATASIZE: D3D10_MESSAGE_ID = 412i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_QUERY_GETDATA_INVALID_FLAGS: D3D10_MESSAGE_ID = 413i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_QUERY_GETDATA_INVALID_CALL: D3D10_MESSAGE_ID = 414i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_PS_OUTPUT_TYPE_MISMATCH: D3D10_MESSAGE_ID = 415i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_RESOURCE_FORMAT_GATHER_UNSUPPORTED: D3D10_MESSAGE_ID = 416i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_INVALID_USE_OF_CENTER_MULTISAMPLE_PATTERN: D3D10_MESSAGE_ID = 417i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_IASETVERTEXBUFFERS_STRIDE_TOO_LARGE: D3D10_MESSAGE_ID = 418i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_IASETVERTEXBUFFERS_INVALIDRANGE: D3D10_MESSAGE_ID = 419i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEINPUTLAYOUT_EMPTY_LAYOUT: D3D10_MESSAGE_ID = 420i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_DRAW_RESOURCE_SAMPLE_COUNT_MISMATCH: D3D10_MESSAGE_ID = 421i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_LIVE_OBJECT_SUMMARY: D3D10_MESSAGE_ID = 422i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_LIVE_BUFFER: D3D10_MESSAGE_ID = 423i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_LIVE_TEXTURE1D: D3D10_MESSAGE_ID = 424i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_LIVE_TEXTURE2D: D3D10_MESSAGE_ID = 425i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_LIVE_TEXTURE3D: D3D10_MESSAGE_ID = 426i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_LIVE_SHADERRESOURCEVIEW: D3D10_MESSAGE_ID = 427i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_LIVE_RENDERTARGETVIEW: D3D10_MESSAGE_ID = 428i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_LIVE_DEPTHSTENCILVIEW: D3D10_MESSAGE_ID = 429i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_LIVE_VERTEXSHADER: D3D10_MESSAGE_ID = 430i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_LIVE_GEOMETRYSHADER: D3D10_MESSAGE_ID = 431i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_LIVE_PIXELSHADER: D3D10_MESSAGE_ID = 432i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_LIVE_INPUTLAYOUT: D3D10_MESSAGE_ID = 433i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_LIVE_SAMPLER: D3D10_MESSAGE_ID = 434i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_LIVE_BLENDSTATE: D3D10_MESSAGE_ID = 435i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_LIVE_DEPTHSTENCILSTATE: D3D10_MESSAGE_ID = 436i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_LIVE_RASTERIZERSTATE: D3D10_MESSAGE_ID = 437i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_LIVE_QUERY: D3D10_MESSAGE_ID = 438i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_LIVE_PREDICATE: D3D10_MESSAGE_ID = 439i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_LIVE_COUNTER: D3D10_MESSAGE_ID = 440i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_LIVE_DEVICE: D3D10_MESSAGE_ID = 441i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_LIVE_SWAPCHAIN: D3D10_MESSAGE_ID = 442i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_D3D10_MESSAGES_END: D3D10_MESSAGE_ID = 443i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_D3D10L9_MESSAGES_START: D3D10_MESSAGE_ID = 1048576i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEDEPTHSTENCILSTATE_STENCIL_NO_TWO_SIDED: D3D10_MESSAGE_ID = 1048577i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERASTERIZERSTATE_DepthBiasClamp_NOT_SUPPORTED: D3D10_MESSAGE_ID = 1048578i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATESAMPLERSTATE_NO_COMPARISON_SUPPORT: D3D10_MESSAGE_ID = 1048579i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATESAMPLERSTATE_EXCESSIVE_ANISOTROPY: D3D10_MESSAGE_ID = 1048580i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATESAMPLERSTATE_BORDER_OUT_OF_RANGE: D3D10_MESSAGE_ID = 1048581i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_VSSETSAMPLERS_NOT_SUPPORTED: D3D10_MESSAGE_ID = 1048582i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_VSSETSAMPLERS_TOO_MANY_SAMPLERS: D3D10_MESSAGE_ID = 1048583i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_PSSETSAMPLERS_TOO_MANY_SAMPLERS: D3D10_MESSAGE_ID = 1048584i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERESOURCE_NO_ARRAYS: D3D10_MESSAGE_ID = 1048585i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERESOURCE_NO_VB_AND_IB_BIND: D3D10_MESSAGE_ID = 1048586i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERESOURCE_NO_TEXTURE_1D: D3D10_MESSAGE_ID = 1048587i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERESOURCE_DIMENSION_OUT_OF_RANGE: D3D10_MESSAGE_ID = 1048588i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERESOURCE_NOT_BINDABLE_AS_SHADER_RESOURCE: D3D10_MESSAGE_ID = 1048589i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_OMSETRENDERTARGETS_TOO_MANY_RENDER_TARGETS: D3D10_MESSAGE_ID = 1048590i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_OMSETRENDERTARGETS_NO_DIFFERING_BIT_DEPTHS: D3D10_MESSAGE_ID = 1048591i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_IASETVERTEXBUFFERS_BAD_BUFFER_INDEX: D3D10_MESSAGE_ID = 1048592i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_RSSETVIEWPORTS_TOO_MANY_VIEWPORTS: D3D10_MESSAGE_ID = 1048593i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_IASETPRIMITIVETOPOLOGY_ADJACENCY_UNSUPPORTED: D3D10_MESSAGE_ID = 1048594i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_RSSETSCISSORRECTS_TOO_MANY_SCISSORS: D3D10_MESSAGE_ID = 1048595i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_COPYRESOURCE_ONLY_TEXTURE_2D_WITHIN_GPU_MEMORY: D3D10_MESSAGE_ID = 1048596i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_COPYRESOURCE_NO_TEXTURE_3D_READBACK: D3D10_MESSAGE_ID = 1048597i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_COPYRESOURCE_NO_TEXTURE_ONLY_READBACK: D3D10_MESSAGE_ID = 1048598i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEINPUTLAYOUT_UNSUPPORTED_FORMAT: D3D10_MESSAGE_ID = 1048599i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBLENDSTATE_NO_ALPHA_TO_COVERAGE: D3D10_MESSAGE_ID = 1048600i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERASTERIZERSTATE_DepthClipEnable_MUST_BE_TRUE: D3D10_MESSAGE_ID = 1048601i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DRAWINDEXED_STARTINDEXLOCATION_MUST_BE_POSITIVE: D3D10_MESSAGE_ID = 1048602i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATESHADERRESOURCEVIEW_MUST_USE_LOWEST_LOD: D3D10_MESSAGE_ID = 1048603i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATESAMPLERSTATE_MINLOD_MUST_NOT_BE_FRACTIONAL: D3D10_MESSAGE_ID = 1048604i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATESAMPLERSTATE_MAXLOD_MUST_BE_FLT_MAX: D3D10_MESSAGE_ID = 1048605i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATESHADERRESOURCEVIEW_FIRSTARRAYSLICE_MUST_BE_ZERO: D3D10_MESSAGE_ID = 1048606i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATESHADERRESOURCEVIEW_CUBES_MUST_HAVE_6_SIDES: D3D10_MESSAGE_ID = 1048607i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERESOURCE_NOT_BINDABLE_AS_RENDER_TARGET: D3D10_MESSAGE_ID = 1048608i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERESOURCE_NO_DWORD_INDEX_BUFFER: D3D10_MESSAGE_ID = 1048609i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERESOURCE_MSAA_PRECLUDES_SHADER_RESOURCE: D3D10_MESSAGE_ID = 1048610i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERESOURCE_PRESENTATION_PRECLUDES_SHADER_RESOURCE: D3D10_MESSAGE_ID = 1048611i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBLENDSTATE_NO_INDEPENDENT_BLEND_ENABLE: D3D10_MESSAGE_ID = 1048612i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBLENDSTATE_NO_INDEPENDENT_WRITE_MASKS: D3D10_MESSAGE_ID = 1048613i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERESOURCE_NO_STREAM_OUT: D3D10_MESSAGE_ID = 1048614i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERESOURCE_ONLY_VB_IB_FOR_BUFFERS: D3D10_MESSAGE_ID = 1048615i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERESOURCE_NO_AUTOGEN_FOR_VOLUMES: D3D10_MESSAGE_ID = 1048616i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERESOURCE_DXGI_FORMAT_R8G8B8A8_CANNOT_BE_SHARED: D3D10_MESSAGE_ID = 1048617i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_VSSHADERRESOURCES_NOT_SUPPORTED: D3D10_MESSAGE_ID = 1048618i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_GEOMETRY_SHADER_NOT_SUPPORTED: D3D10_MESSAGE_ID = 1048619i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_STREAM_OUT_NOT_SUPPORTED: D3D10_MESSAGE_ID = 1048620i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_TEXT_FILTER_NOT_SUPPORTED: D3D10_MESSAGE_ID = 1048621i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBLENDSTATE_NO_SEPARATE_ALPHA_BLEND: D3D10_MESSAGE_ID = 1048622i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBLENDSTATE_NO_MRT_BLEND: D3D10_MESSAGE_ID = 1048623i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATEBLENDSTATE_OPERATION_NOT_SUPPORTED: D3D10_MESSAGE_ID = 1048624i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATESAMPLERSTATE_NO_MIRRORONCE: D3D10_MESSAGE_ID = 1048625i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DRAWINSTANCED_NOT_SUPPORTED: D3D10_MESSAGE_ID = 1048626i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DRAWINDEXEDINSTANCED_NOT_SUPPORTED_BELOW_9_3: D3D10_MESSAGE_ID = 1048627i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DRAWINDEXED_POINTLIST_UNSUPPORTED: D3D10_MESSAGE_ID = 1048628i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_SETBLENDSTATE_SAMPLE_MASK_CANNOT_BE_ZERO: D3D10_MESSAGE_ID = 1048629i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERESOURCE_DIMENSION_EXCEEDS_FEATURE_LEVEL_DEFINITION: D3D10_MESSAGE_ID = 1048630i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERESOURCE_ONLY_SINGLE_MIP_LEVEL_DEPTH_STENCIL_SUPPORTED: D3D10_MESSAGE_ID = 1048631i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_DEVICE_RSSETSCISSORRECTS_NEGATIVESCISSOR: D3D10_MESSAGE_ID = 1048632i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_SLOT_ZERO_MUST_BE_D3D10_INPUT_PER_VERTEX_DATA: D3D10_MESSAGE_ID = 1048633i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATERESOURCE_NON_POW_2_MIPMAP: D3D10_MESSAGE_ID = 1048634i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_CREATESAMPLERSTATE_BORDER_NOT_SUPPORTED: D3D10_MESSAGE_ID = 1048635i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_OMSETRENDERTARGETS_NO_SRGB_MRT: D3D10_MESSAGE_ID = 1048636i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_COPYRESOURCE_NO_3D_MISMATCHED_UPDATES: D3D10_MESSAGE_ID = 1048637i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_ID_D3D10L9_MESSAGES_END: D3D10_MESSAGE_ID = 1048638i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_MESSAGE_SEVERITY = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_SEVERITY_CORRUPTION: D3D10_MESSAGE_SEVERITY = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_SEVERITY_ERROR: D3D10_MESSAGE_SEVERITY = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_SEVERITY_WARNING: D3D10_MESSAGE_SEVERITY = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_SEVERITY_INFO: D3D10_MESSAGE_SEVERITY = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_MESSAGE_SEVERITY_MESSAGE: D3D10_MESSAGE_SEVERITY = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_QUERY = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_QUERY_EVENT: D3D10_QUERY = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_QUERY_OCCLUSION: D3D10_QUERY = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_QUERY_TIMESTAMP: D3D10_QUERY = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_QUERY_TIMESTAMP_DISJOINT: D3D10_QUERY = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_QUERY_PIPELINE_STATISTICS: D3D10_QUERY = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_QUERY_OCCLUSION_PREDICATE: D3D10_QUERY = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_QUERY_SO_STATISTICS: D3D10_QUERY = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_QUERY_SO_OVERFLOW_PREDICATE: D3D10_QUERY = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_QUERY_MISC_FLAG = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_QUERY_MISC_PREDICATEHINT: D3D10_QUERY_MISC_FLAG = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_RAISE_FLAG = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_RAISE_FLAG_DRIVER_INTERNAL_ERROR: D3D10_RAISE_FLAG = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_RESOURCE_DIMENSION = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_RESOURCE_DIMENSION_UNKNOWN: D3D10_RESOURCE_DIMENSION = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_RESOURCE_DIMENSION_BUFFER: D3D10_RESOURCE_DIMENSION = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_RESOURCE_DIMENSION_TEXTURE1D: D3D10_RESOURCE_DIMENSION = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_RESOURCE_DIMENSION_TEXTURE2D: D3D10_RESOURCE_DIMENSION = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_RESOURCE_DIMENSION_TEXTURE3D: D3D10_RESOURCE_DIMENSION = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_RESOURCE_MISC_FLAG = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_RESOURCE_MISC_GENERATE_MIPS: D3D10_RESOURCE_MISC_FLAG = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_RESOURCE_MISC_SHARED: D3D10_RESOURCE_MISC_FLAG = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_RESOURCE_MISC_TEXTURECUBE: D3D10_RESOURCE_MISC_FLAG = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_RESOURCE_MISC_SHARED_KEYEDMUTEX: D3D10_RESOURCE_MISC_FLAG = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_RESOURCE_MISC_GDI_COMPATIBLE: D3D10_RESOURCE_MISC_FLAG = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_RTV_DIMENSION = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_RTV_DIMENSION_UNKNOWN: D3D10_RTV_DIMENSION = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_RTV_DIMENSION_BUFFER: D3D10_RTV_DIMENSION = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_RTV_DIMENSION_TEXTURE1D: D3D10_RTV_DIMENSION = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_RTV_DIMENSION_TEXTURE1DARRAY: D3D10_RTV_DIMENSION = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_RTV_DIMENSION_TEXTURE2D: D3D10_RTV_DIMENSION = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_RTV_DIMENSION_TEXTURE2DARRAY: D3D10_RTV_DIMENSION = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_RTV_DIMENSION_TEXTURE2DMS: D3D10_RTV_DIMENSION = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_RTV_DIMENSION_TEXTURE2DMSARRAY: D3D10_RTV_DIMENSION = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_RTV_DIMENSION_TEXTURE3D: D3D10_RTV_DIMENSION = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_SHADER_DEBUG_REGTYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_DEBUG_REG_INPUT: D3D10_SHADER_DEBUG_REGTYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_DEBUG_REG_OUTPUT: D3D10_SHADER_DEBUG_REGTYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_DEBUG_REG_CBUFFER: D3D10_SHADER_DEBUG_REGTYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_DEBUG_REG_TBUFFER: D3D10_SHADER_DEBUG_REGTYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_DEBUG_REG_TEMP: D3D10_SHADER_DEBUG_REGTYPE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_DEBUG_REG_TEMPARRAY: D3D10_SHADER_DEBUG_REGTYPE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_DEBUG_REG_TEXTURE: D3D10_SHADER_DEBUG_REGTYPE = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_DEBUG_REG_SAMPLER: D3D10_SHADER_DEBUG_REGTYPE = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_DEBUG_REG_IMMEDIATECBUFFER: D3D10_SHADER_DEBUG_REGTYPE = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_DEBUG_REG_LITERAL: D3D10_SHADER_DEBUG_REGTYPE = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_DEBUG_REG_UNUSED: D3D10_SHADER_DEBUG_REGTYPE = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D11_SHADER_DEBUG_REG_INTERFACE_POINTERS: D3D10_SHADER_DEBUG_REGTYPE = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D11_SHADER_DEBUG_REG_UAV: D3D10_SHADER_DEBUG_REGTYPE = 12i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_DEBUG_REG_FORCE_DWORD: D3D10_SHADER_DEBUG_REGTYPE = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_SHADER_DEBUG_SCOPETYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_DEBUG_SCOPE_GLOBAL: D3D10_SHADER_DEBUG_SCOPETYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_DEBUG_SCOPE_BLOCK: D3D10_SHADER_DEBUG_SCOPETYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_DEBUG_SCOPE_FORLOOP: D3D10_SHADER_DEBUG_SCOPETYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_DEBUG_SCOPE_STRUCT: D3D10_SHADER_DEBUG_SCOPETYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_DEBUG_SCOPE_FUNC_PARAMS: D3D10_SHADER_DEBUG_SCOPETYPE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_DEBUG_SCOPE_STATEBLOCK: D3D10_SHADER_DEBUG_SCOPETYPE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_DEBUG_SCOPE_NAMESPACE: D3D10_SHADER_DEBUG_SCOPETYPE = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_DEBUG_SCOPE_ANNOTATION: D3D10_SHADER_DEBUG_SCOPETYPE = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_DEBUG_SCOPE_FORCE_DWORD: D3D10_SHADER_DEBUG_SCOPETYPE = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_SHADER_DEBUG_VARTYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_DEBUG_VAR_VARIABLE: D3D10_SHADER_DEBUG_VARTYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_DEBUG_VAR_FUNCTION: D3D10_SHADER_DEBUG_VARTYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_SHADER_DEBUG_VAR_FORCE_DWORD: D3D10_SHADER_DEBUG_VARTYPE = 2147483647i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_STANDARD_MULTISAMPLE_QUALITY_LEVELS = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_STANDARD_MULTISAMPLE_PATTERN: D3D10_STANDARD_MULTISAMPLE_QUALITY_LEVELS = -1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_CENTER_MULTISAMPLE_PATTERN: D3D10_STANDARD_MULTISAMPLE_QUALITY_LEVELS = -2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_STENCIL_OP = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_STENCIL_OP_KEEP: D3D10_STENCIL_OP = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_STENCIL_OP_ZERO: D3D10_STENCIL_OP = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_STENCIL_OP_REPLACE: D3D10_STENCIL_OP = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_STENCIL_OP_INCR_SAT: D3D10_STENCIL_OP = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_STENCIL_OP_DECR_SAT: D3D10_STENCIL_OP = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_STENCIL_OP_INVERT: D3D10_STENCIL_OP = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_STENCIL_OP_INCR: D3D10_STENCIL_OP = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_STENCIL_OP_DECR: D3D10_STENCIL_OP = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_TEXTURECUBE_FACE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_TEXTURECUBE_FACE_POSITIVE_X: D3D10_TEXTURECUBE_FACE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_TEXTURECUBE_FACE_NEGATIVE_X: D3D10_TEXTURECUBE_FACE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_TEXTURECUBE_FACE_POSITIVE_Y: D3D10_TEXTURECUBE_FACE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_TEXTURECUBE_FACE_NEGATIVE_Y: D3D10_TEXTURECUBE_FACE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_TEXTURECUBE_FACE_POSITIVE_Z: D3D10_TEXTURECUBE_FACE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_TEXTURECUBE_FACE_NEGATIVE_Z: D3D10_TEXTURECUBE_FACE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_TEXTURE_ADDRESS_MODE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_TEXTURE_ADDRESS_WRAP: D3D10_TEXTURE_ADDRESS_MODE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_TEXTURE_ADDRESS_MIRROR: D3D10_TEXTURE_ADDRESS_MODE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_TEXTURE_ADDRESS_CLAMP: D3D10_TEXTURE_ADDRESS_MODE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_TEXTURE_ADDRESS_BORDER: D3D10_TEXTURE_ADDRESS_MODE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_TEXTURE_ADDRESS_MIRROR_ONCE: D3D10_TEXTURE_ADDRESS_MODE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub type D3D10_USAGE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_USAGE_DEFAULT: D3D10_USAGE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_USAGE_IMMUTABLE: D3D10_USAGE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_USAGE_DYNAMIC: D3D10_USAGE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub const D3D10_USAGE_STAGING: D3D10_USAGE = 3i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D10_BLEND_DESC {
    pub AlphaToCoverageEnable: super::super::Foundation::BOOL,
    pub BlendEnable: [super::super::Foundation::BOOL; 8],
    pub SrcBlend: D3D10_BLEND,
    pub DestBlend: D3D10_BLEND,
    pub BlendOp: D3D10_BLEND_OP,
    pub SrcBlendAlpha: D3D10_BLEND,
    pub DestBlendAlpha: D3D10_BLEND,
    pub BlendOpAlpha: D3D10_BLEND_OP,
    pub RenderTargetWriteMask: [u8; 8],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D10_BLEND_DESC {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D10_BLEND_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D10_BLEND_DESC1 {
    pub AlphaToCoverageEnable: super::super::Foundation::BOOL,
    pub IndependentBlendEnable: super::super::Foundation::BOOL,
    pub RenderTarget: [D3D10_RENDER_TARGET_BLEND_DESC1; 8],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D10_BLEND_DESC1 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D10_BLEND_DESC1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_BOX {
    pub left: u32,
    pub top: u32,
    pub front: u32,
    pub right: u32,
    pub bottom: u32,
    pub back: u32,
}
impl ::core::marker::Copy for D3D10_BOX {}
impl ::core::clone::Clone for D3D10_BOX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_BUFFER_DESC {
    pub ByteWidth: u32,
    pub Usage: D3D10_USAGE,
    pub BindFlags: u32,
    pub CPUAccessFlags: u32,
    pub MiscFlags: u32,
}
impl ::core::marker::Copy for D3D10_BUFFER_DESC {}
impl ::core::clone::Clone for D3D10_BUFFER_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_BUFFER_RTV {
    pub Anonymous1: D3D10_BUFFER_RTV_0,
    pub Anonymous2: D3D10_BUFFER_RTV_1,
}
impl ::core::marker::Copy for D3D10_BUFFER_RTV {}
impl ::core::clone::Clone for D3D10_BUFFER_RTV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub union D3D10_BUFFER_RTV_0 {
    pub FirstElement: u32,
    pub ElementOffset: u32,
}
impl ::core::marker::Copy for D3D10_BUFFER_RTV_0 {}
impl ::core::clone::Clone for D3D10_BUFFER_RTV_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub union D3D10_BUFFER_RTV_1 {
    pub NumElements: u32,
    pub ElementWidth: u32,
}
impl ::core::marker::Copy for D3D10_BUFFER_RTV_1 {}
impl ::core::clone::Clone for D3D10_BUFFER_RTV_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_BUFFER_SRV {
    pub Anonymous1: D3D10_BUFFER_SRV_0,
    pub Anonymous2: D3D10_BUFFER_SRV_1,
}
impl ::core::marker::Copy for D3D10_BUFFER_SRV {}
impl ::core::clone::Clone for D3D10_BUFFER_SRV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub union D3D10_BUFFER_SRV_0 {
    pub FirstElement: u32,
    pub ElementOffset: u32,
}
impl ::core::marker::Copy for D3D10_BUFFER_SRV_0 {}
impl ::core::clone::Clone for D3D10_BUFFER_SRV_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub union D3D10_BUFFER_SRV_1 {
    pub NumElements: u32,
    pub ElementWidth: u32,
}
impl ::core::marker::Copy for D3D10_BUFFER_SRV_1 {}
impl ::core::clone::Clone for D3D10_BUFFER_SRV_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_COUNTER_DESC {
    pub Counter: D3D10_COUNTER,
    pub MiscFlags: u32,
}
impl ::core::marker::Copy for D3D10_COUNTER_DESC {}
impl ::core::clone::Clone for D3D10_COUNTER_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_COUNTER_INFO {
    pub LastDeviceDependentCounter: D3D10_COUNTER,
    pub NumSimultaneousCounters: u32,
    pub NumDetectableParallelUnits: u8,
}
impl ::core::marker::Copy for D3D10_COUNTER_INFO {}
impl ::core::clone::Clone for D3D10_COUNTER_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_DEPTH_STENCILOP_DESC {
    pub StencilFailOp: D3D10_STENCIL_OP,
    pub StencilDepthFailOp: D3D10_STENCIL_OP,
    pub StencilPassOp: D3D10_STENCIL_OP,
    pub StencilFunc: D3D10_COMPARISON_FUNC,
}
impl ::core::marker::Copy for D3D10_DEPTH_STENCILOP_DESC {}
impl ::core::clone::Clone for D3D10_DEPTH_STENCILOP_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D10_DEPTH_STENCIL_DESC {
    pub DepthEnable: super::super::Foundation::BOOL,
    pub DepthWriteMask: D3D10_DEPTH_WRITE_MASK,
    pub DepthFunc: D3D10_COMPARISON_FUNC,
    pub StencilEnable: super::super::Foundation::BOOL,
    pub StencilReadMask: u8,
    pub StencilWriteMask: u8,
    pub FrontFace: D3D10_DEPTH_STENCILOP_DESC,
    pub BackFace: D3D10_DEPTH_STENCILOP_DESC,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D10_DEPTH_STENCIL_DESC {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D10_DEPTH_STENCIL_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub struct D3D10_DEPTH_STENCIL_VIEW_DESC {
    pub Format: super::Dxgi::Common::DXGI_FORMAT,
    pub ViewDimension: D3D10_DSV_DIMENSION,
    pub Anonymous: D3D10_DEPTH_STENCIL_VIEW_DESC_0,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D10_DEPTH_STENCIL_VIEW_DESC {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D10_DEPTH_STENCIL_VIEW_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub union D3D10_DEPTH_STENCIL_VIEW_DESC_0 {
    pub Texture1D: D3D10_TEX1D_DSV,
    pub Texture1DArray: D3D10_TEX1D_ARRAY_DSV,
    pub Texture2D: D3D10_TEX2D_DSV,
    pub Texture2DArray: D3D10_TEX2D_ARRAY_DSV,
    pub Texture2DMS: D3D10_TEX2DMS_DSV,
    pub Texture2DMSArray: D3D10_TEX2DMS_ARRAY_DSV,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D10_DEPTH_STENCIL_VIEW_DESC_0 {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D10_DEPTH_STENCIL_VIEW_DESC_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D10_EFFECT_DESC {
    pub IsChildEffect: super::super::Foundation::BOOL,
    pub ConstantBuffers: u32,
    pub SharedConstantBuffers: u32,
    pub GlobalVariables: u32,
    pub SharedGlobalVariables: u32,
    pub Techniques: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D10_EFFECT_DESC {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D10_EFFECT_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D10_EFFECT_SHADER_DESC {
    pub pInputSignature: *const u8,
    pub IsInline: super::super::Foundation::BOOL,
    pub pBytecode: *const u8,
    pub BytecodeLength: u32,
    pub SODecl: ::windows_sys::core::PCSTR,
    pub NumInputSignatureEntries: u32,
    pub NumOutputSignatureEntries: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D10_EFFECT_SHADER_DESC {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D10_EFFECT_SHADER_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Graphics_Direct3D\"`*"]
#[cfg(feature = "Win32_Graphics_Direct3D")]
pub struct D3D10_EFFECT_TYPE_DESC {
    pub TypeName: ::windows_sys::core::PCSTR,
    pub Class: super::Direct3D::D3D_SHADER_VARIABLE_CLASS,
    pub Type: super::Direct3D::D3D_SHADER_VARIABLE_TYPE,
    pub Elements: u32,
    pub Members: u32,
    pub Rows: u32,
    pub Columns: u32,
    pub PackedSize: u32,
    pub UnpackedSize: u32,
    pub Stride: u32,
}
#[cfg(feature = "Win32_Graphics_Direct3D")]
impl ::core::marker::Copy for D3D10_EFFECT_TYPE_DESC {}
#[cfg(feature = "Win32_Graphics_Direct3D")]
impl ::core::clone::Clone for D3D10_EFFECT_TYPE_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_EFFECT_VARIABLE_DESC {
    pub Name: ::windows_sys::core::PCSTR,
    pub Semantic: ::windows_sys::core::PCSTR,
    pub Flags: u32,
    pub Annotations: u32,
    pub BufferOffset: u32,
    pub ExplicitBindPoint: u32,
}
impl ::core::marker::Copy for D3D10_EFFECT_VARIABLE_DESC {}
impl ::core::clone::Clone for D3D10_EFFECT_VARIABLE_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_INFO_QUEUE_FILTER {
    pub AllowList: D3D10_INFO_QUEUE_FILTER_DESC,
    pub DenyList: D3D10_INFO_QUEUE_FILTER_DESC,
}
impl ::core::marker::Copy for D3D10_INFO_QUEUE_FILTER {}
impl ::core::clone::Clone for D3D10_INFO_QUEUE_FILTER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_INFO_QUEUE_FILTER_DESC {
    pub NumCategories: u32,
    pub pCategoryList: *mut D3D10_MESSAGE_CATEGORY,
    pub NumSeverities: u32,
    pub pSeverityList: *mut D3D10_MESSAGE_SEVERITY,
    pub NumIDs: u32,
    pub pIDList: *mut D3D10_MESSAGE_ID,
}
impl ::core::marker::Copy for D3D10_INFO_QUEUE_FILTER_DESC {}
impl ::core::clone::Clone for D3D10_INFO_QUEUE_FILTER_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub struct D3D10_INPUT_ELEMENT_DESC {
    pub SemanticName: ::windows_sys::core::PCSTR,
    pub SemanticIndex: u32,
    pub Format: super::Dxgi::Common::DXGI_FORMAT,
    pub InputSlot: u32,
    pub AlignedByteOffset: u32,
    pub InputSlotClass: D3D10_INPUT_CLASSIFICATION,
    pub InstanceDataStepRate: u32,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D10_INPUT_ELEMENT_DESC {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D10_INPUT_ELEMENT_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_MAPPED_TEXTURE2D {
    pub pData: *mut ::core::ffi::c_void,
    pub RowPitch: u32,
}
impl ::core::marker::Copy for D3D10_MAPPED_TEXTURE2D {}
impl ::core::clone::Clone for D3D10_MAPPED_TEXTURE2D {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_MAPPED_TEXTURE3D {
    pub pData: *mut ::core::ffi::c_void,
    pub RowPitch: u32,
    pub DepthPitch: u32,
}
impl ::core::marker::Copy for D3D10_MAPPED_TEXTURE3D {}
impl ::core::clone::Clone for D3D10_MAPPED_TEXTURE3D {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_MESSAGE {
    pub Category: D3D10_MESSAGE_CATEGORY,
    pub Severity: D3D10_MESSAGE_SEVERITY,
    pub ID: D3D10_MESSAGE_ID,
    pub pDescription: *const u8,
    pub DescriptionByteLength: usize,
}
impl ::core::marker::Copy for D3D10_MESSAGE {}
impl ::core::clone::Clone for D3D10_MESSAGE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_PASS_DESC {
    pub Name: ::windows_sys::core::PCSTR,
    pub Annotations: u32,
    pub pIAInputSignature: *mut u8,
    pub IAInputSignatureSize: usize,
    pub StencilRef: u32,
    pub SampleMask: u32,
    pub BlendFactor: [f32; 4],
}
impl ::core::marker::Copy for D3D10_PASS_DESC {}
impl ::core::clone::Clone for D3D10_PASS_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_PASS_SHADER_DESC {
    pub pShaderVariable: ID3D10EffectShaderVariable,
    pub ShaderIndex: u32,
}
impl ::core::marker::Copy for D3D10_PASS_SHADER_DESC {}
impl ::core::clone::Clone for D3D10_PASS_SHADER_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_QUERY_DATA_PIPELINE_STATISTICS {
    pub IAVertices: u64,
    pub IAPrimitives: u64,
    pub VSInvocations: u64,
    pub GSInvocations: u64,
    pub GSPrimitives: u64,
    pub CInvocations: u64,
    pub CPrimitives: u64,
    pub PSInvocations: u64,
}
impl ::core::marker::Copy for D3D10_QUERY_DATA_PIPELINE_STATISTICS {}
impl ::core::clone::Clone for D3D10_QUERY_DATA_PIPELINE_STATISTICS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_QUERY_DATA_SO_STATISTICS {
    pub NumPrimitivesWritten: u64,
    pub PrimitivesStorageNeeded: u64,
}
impl ::core::marker::Copy for D3D10_QUERY_DATA_SO_STATISTICS {}
impl ::core::clone::Clone for D3D10_QUERY_DATA_SO_STATISTICS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D10_QUERY_DATA_TIMESTAMP_DISJOINT {
    pub Frequency: u64,
    pub Disjoint: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D10_QUERY_DATA_TIMESTAMP_DISJOINT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D10_QUERY_DATA_TIMESTAMP_DISJOINT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_QUERY_DESC {
    pub Query: D3D10_QUERY,
    pub MiscFlags: u32,
}
impl ::core::marker::Copy for D3D10_QUERY_DESC {}
impl ::core::clone::Clone for D3D10_QUERY_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D10_RASTERIZER_DESC {
    pub FillMode: D3D10_FILL_MODE,
    pub CullMode: D3D10_CULL_MODE,
    pub FrontCounterClockwise: super::super::Foundation::BOOL,
    pub DepthBias: i32,
    pub DepthBiasClamp: f32,
    pub SlopeScaledDepthBias: f32,
    pub DepthClipEnable: super::super::Foundation::BOOL,
    pub ScissorEnable: super::super::Foundation::BOOL,
    pub MultisampleEnable: super::super::Foundation::BOOL,
    pub AntialiasedLineEnable: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D10_RASTERIZER_DESC {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D10_RASTERIZER_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D10_RENDER_TARGET_BLEND_DESC1 {
    pub BlendEnable: super::super::Foundation::BOOL,
    pub SrcBlend: D3D10_BLEND,
    pub DestBlend: D3D10_BLEND,
    pub BlendOp: D3D10_BLEND_OP,
    pub SrcBlendAlpha: D3D10_BLEND,
    pub DestBlendAlpha: D3D10_BLEND,
    pub BlendOpAlpha: D3D10_BLEND_OP,
    pub RenderTargetWriteMask: u8,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D10_RENDER_TARGET_BLEND_DESC1 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D10_RENDER_TARGET_BLEND_DESC1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub struct D3D10_RENDER_TARGET_VIEW_DESC {
    pub Format: super::Dxgi::Common::DXGI_FORMAT,
    pub ViewDimension: D3D10_RTV_DIMENSION,
    pub Anonymous: D3D10_RENDER_TARGET_VIEW_DESC_0,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D10_RENDER_TARGET_VIEW_DESC {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D10_RENDER_TARGET_VIEW_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub union D3D10_RENDER_TARGET_VIEW_DESC_0 {
    pub Buffer: D3D10_BUFFER_RTV,
    pub Texture1D: D3D10_TEX1D_RTV,
    pub Texture1DArray: D3D10_TEX1D_ARRAY_RTV,
    pub Texture2D: D3D10_TEX2D_RTV,
    pub Texture2DArray: D3D10_TEX2D_ARRAY_RTV,
    pub Texture2DMS: D3D10_TEX2DMS_RTV,
    pub Texture2DMSArray: D3D10_TEX2DMS_ARRAY_RTV,
    pub Texture3D: D3D10_TEX3D_RTV,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D10_RENDER_TARGET_VIEW_DESC_0 {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D10_RENDER_TARGET_VIEW_DESC_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_SAMPLER_DESC {
    pub Filter: D3D10_FILTER,
    pub AddressU: D3D10_TEXTURE_ADDRESS_MODE,
    pub AddressV: D3D10_TEXTURE_ADDRESS_MODE,
    pub AddressW: D3D10_TEXTURE_ADDRESS_MODE,
    pub MipLODBias: f32,
    pub MaxAnisotropy: u32,
    pub ComparisonFunc: D3D10_COMPARISON_FUNC,
    pub BorderColor: [f32; 4],
    pub MinLOD: f32,
    pub MaxLOD: f32,
}
impl ::core::marker::Copy for D3D10_SAMPLER_DESC {}
impl ::core::clone::Clone for D3D10_SAMPLER_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Graphics_Direct3D\"`*"]
#[cfg(feature = "Win32_Graphics_Direct3D")]
pub struct D3D10_SHADER_BUFFER_DESC {
    pub Name: ::windows_sys::core::PCSTR,
    pub Type: super::Direct3D::D3D_CBUFFER_TYPE,
    pub Variables: u32,
    pub Size: u32,
    pub uFlags: u32,
}
#[cfg(feature = "Win32_Graphics_Direct3D")]
impl ::core::marker::Copy for D3D10_SHADER_BUFFER_DESC {}
#[cfg(feature = "Win32_Graphics_Direct3D")]
impl ::core::clone::Clone for D3D10_SHADER_BUFFER_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_SHADER_DEBUG_FILE_INFO {
    pub FileName: u32,
    pub FileNameLen: u32,
    pub FileData: u32,
    pub FileLen: u32,
}
impl ::core::marker::Copy for D3D10_SHADER_DEBUG_FILE_INFO {}
impl ::core::clone::Clone for D3D10_SHADER_DEBUG_FILE_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_SHADER_DEBUG_INFO {
    pub Size: u32,
    pub Creator: u32,
    pub EntrypointName: u32,
    pub ShaderTarget: u32,
    pub CompileFlags: u32,
    pub Files: u32,
    pub FileInfo: u32,
    pub Instructions: u32,
    pub InstructionInfo: u32,
    pub Variables: u32,
    pub VariableInfo: u32,
    pub InputVariables: u32,
    pub InputVariableInfo: u32,
    pub Tokens: u32,
    pub TokenInfo: u32,
    pub Scopes: u32,
    pub ScopeInfo: u32,
    pub ScopeVariables: u32,
    pub ScopeVariableInfo: u32,
    pub UintOffset: u32,
    pub StringOffset: u32,
}
impl ::core::marker::Copy for D3D10_SHADER_DEBUG_INFO {}
impl ::core::clone::Clone for D3D10_SHADER_DEBUG_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_SHADER_DEBUG_INPUT_INFO {
    pub Var: u32,
    pub InitialRegisterSet: D3D10_SHADER_DEBUG_REGTYPE,
    pub InitialBank: u32,
    pub InitialRegister: u32,
    pub InitialComponent: u32,
    pub InitialValue: u32,
}
impl ::core::marker::Copy for D3D10_SHADER_DEBUG_INPUT_INFO {}
impl ::core::clone::Clone for D3D10_SHADER_DEBUG_INPUT_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D10_SHADER_DEBUG_INST_INFO {
    pub Id: u32,
    pub Opcode: u32,
    pub uOutputs: u32,
    pub pOutputs: [D3D10_SHADER_DEBUG_OUTPUTREG_INFO; 2],
    pub TokenId: u32,
    pub NestingLevel: u32,
    pub Scopes: u32,
    pub ScopeInfo: u32,
    pub AccessedVars: u32,
    pub AccessedVarsInfo: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D10_SHADER_DEBUG_INST_INFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D10_SHADER_DEBUG_INST_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D10_SHADER_DEBUG_OUTPUTREG_INFO {
    pub OutputRegisterSet: D3D10_SHADER_DEBUG_REGTYPE,
    pub OutputReg: u32,
    pub TempArrayReg: u32,
    pub OutputComponents: [u32; 4],
    pub OutputVars: [D3D10_SHADER_DEBUG_OUTPUTVAR; 4],
    pub IndexReg: u32,
    pub IndexComp: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D10_SHADER_DEBUG_OUTPUTREG_INFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D10_SHADER_DEBUG_OUTPUTREG_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D10_SHADER_DEBUG_OUTPUTVAR {
    pub Var: u32,
    pub uValueMin: u32,
    pub uValueMax: u32,
    pub iValueMin: i32,
    pub iValueMax: i32,
    pub fValueMin: f32,
    pub fValueMax: f32,
    pub bNaNPossible: super::super::Foundation::BOOL,
    pub bInfPossible: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D10_SHADER_DEBUG_OUTPUTVAR {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D10_SHADER_DEBUG_OUTPUTVAR {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Graphics_Direct3D\"`*"]
#[cfg(feature = "Win32_Graphics_Direct3D")]
pub struct D3D10_SHADER_DEBUG_SCOPEVAR_INFO {
    pub TokenId: u32,
    pub VarType: D3D10_SHADER_DEBUG_VARTYPE,
    pub Class: super::Direct3D::D3D_SHADER_VARIABLE_CLASS,
    pub Rows: u32,
    pub Columns: u32,
    pub StructMemberScope: u32,
    pub uArrayIndices: u32,
    pub ArrayElements: u32,
    pub ArrayStrides: u32,
    pub uVariables: u32,
    pub uFirstVariable: u32,
}
#[cfg(feature = "Win32_Graphics_Direct3D")]
impl ::core::marker::Copy for D3D10_SHADER_DEBUG_SCOPEVAR_INFO {}
#[cfg(feature = "Win32_Graphics_Direct3D")]
impl ::core::clone::Clone for D3D10_SHADER_DEBUG_SCOPEVAR_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_SHADER_DEBUG_SCOPE_INFO {
    pub ScopeType: D3D10_SHADER_DEBUG_SCOPETYPE,
    pub Name: u32,
    pub uNameLen: u32,
    pub uVariables: u32,
    pub VariableData: u32,
}
impl ::core::marker::Copy for D3D10_SHADER_DEBUG_SCOPE_INFO {}
impl ::core::clone::Clone for D3D10_SHADER_DEBUG_SCOPE_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_SHADER_DEBUG_TOKEN_INFO {
    pub File: u32,
    pub Line: u32,
    pub Column: u32,
    pub TokenLength: u32,
    pub TokenId: u32,
}
impl ::core::marker::Copy for D3D10_SHADER_DEBUG_TOKEN_INFO {}
impl ::core::clone::Clone for D3D10_SHADER_DEBUG_TOKEN_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Graphics_Direct3D\"`*"]
#[cfg(feature = "Win32_Graphics_Direct3D")]
pub struct D3D10_SHADER_DEBUG_VAR_INFO {
    pub TokenId: u32,
    pub Type: super::Direct3D::D3D_SHADER_VARIABLE_TYPE,
    pub Register: u32,
    pub Component: u32,
    pub ScopeVar: u32,
    pub ScopeVarOffset: u32,
}
#[cfg(feature = "Win32_Graphics_Direct3D")]
impl ::core::marker::Copy for D3D10_SHADER_DEBUG_VAR_INFO {}
#[cfg(feature = "Win32_Graphics_Direct3D")]
impl ::core::clone::Clone for D3D10_SHADER_DEBUG_VAR_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Graphics_Direct3D\"`*"]
#[cfg(feature = "Win32_Graphics_Direct3D")]
pub struct D3D10_SHADER_DESC {
    pub Version: u32,
    pub Creator: ::windows_sys::core::PCSTR,
    pub Flags: u32,
    pub ConstantBuffers: u32,
    pub BoundResources: u32,
    pub InputParameters: u32,
    pub OutputParameters: u32,
    pub InstructionCount: u32,
    pub TempRegisterCount: u32,
    pub TempArrayCount: u32,
    pub DefCount: u32,
    pub DclCount: u32,
    pub TextureNormalInstructions: u32,
    pub TextureLoadInstructions: u32,
    pub TextureCompInstructions: u32,
    pub TextureBiasInstructions: u32,
    pub TextureGradientInstructions: u32,
    pub FloatInstructionCount: u32,
    pub IntInstructionCount: u32,
    pub UintInstructionCount: u32,
    pub StaticFlowControlCount: u32,
    pub DynamicFlowControlCount: u32,
    pub MacroInstructionCount: u32,
    pub ArrayInstructionCount: u32,
    pub CutInstructionCount: u32,
    pub EmitInstructionCount: u32,
    pub GSOutputTopology: super::Direct3D::D3D_PRIMITIVE_TOPOLOGY,
    pub GSMaxOutputVertexCount: u32,
}
#[cfg(feature = "Win32_Graphics_Direct3D")]
impl ::core::marker::Copy for D3D10_SHADER_DESC {}
#[cfg(feature = "Win32_Graphics_Direct3D")]
impl ::core::clone::Clone for D3D10_SHADER_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Graphics_Direct3D\"`*"]
#[cfg(feature = "Win32_Graphics_Direct3D")]
pub struct D3D10_SHADER_INPUT_BIND_DESC {
    pub Name: ::windows_sys::core::PCSTR,
    pub Type: super::Direct3D::D3D_SHADER_INPUT_TYPE,
    pub BindPoint: u32,
    pub BindCount: u32,
    pub uFlags: u32,
    pub ReturnType: super::Direct3D::D3D_RESOURCE_RETURN_TYPE,
    pub Dimension: super::Direct3D::D3D_SRV_DIMENSION,
    pub NumSamples: u32,
}
#[cfg(feature = "Win32_Graphics_Direct3D")]
impl ::core::marker::Copy for D3D10_SHADER_INPUT_BIND_DESC {}
#[cfg(feature = "Win32_Graphics_Direct3D")]
impl ::core::clone::Clone for D3D10_SHADER_INPUT_BIND_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Graphics_Direct3D\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(all(feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common"))]
pub struct D3D10_SHADER_RESOURCE_VIEW_DESC {
    pub Format: super::Dxgi::Common::DXGI_FORMAT,
    pub ViewDimension: super::Direct3D::D3D_SRV_DIMENSION,
    pub Anonymous: D3D10_SHADER_RESOURCE_VIEW_DESC_0,
}
#[cfg(all(feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common"))]
impl ::core::marker::Copy for D3D10_SHADER_RESOURCE_VIEW_DESC {}
#[cfg(all(feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common"))]
impl ::core::clone::Clone for D3D10_SHADER_RESOURCE_VIEW_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Graphics_Direct3D\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(all(feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common"))]
pub union D3D10_SHADER_RESOURCE_VIEW_DESC_0 {
    pub Buffer: D3D10_BUFFER_SRV,
    pub Texture1D: D3D10_TEX1D_SRV,
    pub Texture1DArray: D3D10_TEX1D_ARRAY_SRV,
    pub Texture2D: D3D10_TEX2D_SRV,
    pub Texture2DArray: D3D10_TEX2D_ARRAY_SRV,
    pub Texture2DMS: D3D10_TEX2DMS_SRV,
    pub Texture2DMSArray: D3D10_TEX2DMS_ARRAY_SRV,
    pub Texture3D: D3D10_TEX3D_SRV,
    pub TextureCube: D3D10_TEXCUBE_SRV,
}
#[cfg(all(feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common"))]
impl ::core::marker::Copy for D3D10_SHADER_RESOURCE_VIEW_DESC_0 {}
#[cfg(all(feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common"))]
impl ::core::clone::Clone for D3D10_SHADER_RESOURCE_VIEW_DESC_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Graphics_Direct3D\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(all(feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common"))]
pub struct D3D10_SHADER_RESOURCE_VIEW_DESC1 {
    pub Format: super::Dxgi::Common::DXGI_FORMAT,
    pub ViewDimension: super::Direct3D::D3D_SRV_DIMENSION,
    pub Anonymous: D3D10_SHADER_RESOURCE_VIEW_DESC1_0,
}
#[cfg(all(feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common"))]
impl ::core::marker::Copy for D3D10_SHADER_RESOURCE_VIEW_DESC1 {}
#[cfg(all(feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common"))]
impl ::core::clone::Clone for D3D10_SHADER_RESOURCE_VIEW_DESC1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Graphics_Direct3D\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(all(feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common"))]
pub union D3D10_SHADER_RESOURCE_VIEW_DESC1_0 {
    pub Buffer: D3D10_BUFFER_SRV,
    pub Texture1D: D3D10_TEX1D_SRV,
    pub Texture1DArray: D3D10_TEX1D_ARRAY_SRV,
    pub Texture2D: D3D10_TEX2D_SRV,
    pub Texture2DArray: D3D10_TEX2D_ARRAY_SRV,
    pub Texture2DMS: D3D10_TEX2DMS_SRV,
    pub Texture2DMSArray: D3D10_TEX2DMS_ARRAY_SRV,
    pub Texture3D: D3D10_TEX3D_SRV,
    pub TextureCube: D3D10_TEXCUBE_SRV,
    pub TextureCubeArray: D3D10_TEXCUBE_ARRAY_SRV1,
}
#[cfg(all(feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common"))]
impl ::core::marker::Copy for D3D10_SHADER_RESOURCE_VIEW_DESC1_0 {}
#[cfg(all(feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common"))]
impl ::core::clone::Clone for D3D10_SHADER_RESOURCE_VIEW_DESC1_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Graphics_Direct3D\"`*"]
#[cfg(feature = "Win32_Graphics_Direct3D")]
pub struct D3D10_SHADER_TYPE_DESC {
    pub Class: super::Direct3D::D3D_SHADER_VARIABLE_CLASS,
    pub Type: super::Direct3D::D3D_SHADER_VARIABLE_TYPE,
    pub Rows: u32,
    pub Columns: u32,
    pub Elements: u32,
    pub Members: u32,
    pub Offset: u32,
}
#[cfg(feature = "Win32_Graphics_Direct3D")]
impl ::core::marker::Copy for D3D10_SHADER_TYPE_DESC {}
#[cfg(feature = "Win32_Graphics_Direct3D")]
impl ::core::clone::Clone for D3D10_SHADER_TYPE_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_SHADER_VARIABLE_DESC {
    pub Name: ::windows_sys::core::PCSTR,
    pub StartOffset: u32,
    pub Size: u32,
    pub uFlags: u32,
    pub DefaultValue: *mut ::core::ffi::c_void,
}
impl ::core::marker::Copy for D3D10_SHADER_VARIABLE_DESC {}
impl ::core::clone::Clone for D3D10_SHADER_VARIABLE_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Graphics_Direct3D\"`*"]
#[cfg(feature = "Win32_Graphics_Direct3D")]
pub struct D3D10_SIGNATURE_PARAMETER_DESC {
    pub SemanticName: ::windows_sys::core::PCSTR,
    pub SemanticIndex: u32,
    pub Register: u32,
    pub SystemValueType: super::Direct3D::D3D_NAME,
    pub ComponentType: super::Direct3D::D3D_REGISTER_COMPONENT_TYPE,
    pub Mask: u8,
    pub ReadWriteMask: u8,
}
#[cfg(feature = "Win32_Graphics_Direct3D")]
impl ::core::marker::Copy for D3D10_SIGNATURE_PARAMETER_DESC {}
#[cfg(feature = "Win32_Graphics_Direct3D")]
impl ::core::clone::Clone for D3D10_SIGNATURE_PARAMETER_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_SO_DECLARATION_ENTRY {
    pub SemanticName: ::windows_sys::core::PCSTR,
    pub SemanticIndex: u32,
    pub StartComponent: u8,
    pub ComponentCount: u8,
    pub OutputSlot: u8,
}
impl ::core::marker::Copy for D3D10_SO_DECLARATION_ENTRY {}
impl ::core::clone::Clone for D3D10_SO_DECLARATION_ENTRY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_STATE_BLOCK_MASK {
    pub VS: u8,
    pub VSSamplers: [u8; 2],
    pub VSShaderResources: [u8; 16],
    pub VSConstantBuffers: [u8; 2],
    pub GS: u8,
    pub GSSamplers: [u8; 2],
    pub GSShaderResources: [u8; 16],
    pub GSConstantBuffers: [u8; 2],
    pub PS: u8,
    pub PSSamplers: [u8; 2],
    pub PSShaderResources: [u8; 16],
    pub PSConstantBuffers: [u8; 2],
    pub IAVertexBuffers: [u8; 2],
    pub IAIndexBuffer: u8,
    pub IAInputLayout: u8,
    pub IAPrimitiveTopology: u8,
    pub OMRenderTargets: u8,
    pub OMDepthStencilState: u8,
    pub OMBlendState: u8,
    pub RSViewports: u8,
    pub RSScissorRects: u8,
    pub RSRasterizerState: u8,
    pub SOBuffers: u8,
    pub Predication: u8,
}
impl ::core::marker::Copy for D3D10_STATE_BLOCK_MASK {}
impl ::core::clone::Clone for D3D10_STATE_BLOCK_MASK {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_SUBRESOURCE_DATA {
    pub pSysMem: *const ::core::ffi::c_void,
    pub SysMemPitch: u32,
    pub SysMemSlicePitch: u32,
}
impl ::core::marker::Copy for D3D10_SUBRESOURCE_DATA {}
impl ::core::clone::Clone for D3D10_SUBRESOURCE_DATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_TECHNIQUE_DESC {
    pub Name: ::windows_sys::core::PCSTR,
    pub Passes: u32,
    pub Annotations: u32,
}
impl ::core::marker::Copy for D3D10_TECHNIQUE_DESC {}
impl ::core::clone::Clone for D3D10_TECHNIQUE_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_TEX1D_ARRAY_DSV {
    pub MipSlice: u32,
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
}
impl ::core::marker::Copy for D3D10_TEX1D_ARRAY_DSV {}
impl ::core::clone::Clone for D3D10_TEX1D_ARRAY_DSV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_TEX1D_ARRAY_RTV {
    pub MipSlice: u32,
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
}
impl ::core::marker::Copy for D3D10_TEX1D_ARRAY_RTV {}
impl ::core::clone::Clone for D3D10_TEX1D_ARRAY_RTV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_TEX1D_ARRAY_SRV {
    pub MostDetailedMip: u32,
    pub MipLevels: u32,
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
}
impl ::core::marker::Copy for D3D10_TEX1D_ARRAY_SRV {}
impl ::core::clone::Clone for D3D10_TEX1D_ARRAY_SRV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_TEX1D_DSV {
    pub MipSlice: u32,
}
impl ::core::marker::Copy for D3D10_TEX1D_DSV {}
impl ::core::clone::Clone for D3D10_TEX1D_DSV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_TEX1D_RTV {
    pub MipSlice: u32,
}
impl ::core::marker::Copy for D3D10_TEX1D_RTV {}
impl ::core::clone::Clone for D3D10_TEX1D_RTV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_TEX1D_SRV {
    pub MostDetailedMip: u32,
    pub MipLevels: u32,
}
impl ::core::marker::Copy for D3D10_TEX1D_SRV {}
impl ::core::clone::Clone for D3D10_TEX1D_SRV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_TEX2DMS_ARRAY_DSV {
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
}
impl ::core::marker::Copy for D3D10_TEX2DMS_ARRAY_DSV {}
impl ::core::clone::Clone for D3D10_TEX2DMS_ARRAY_DSV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_TEX2DMS_ARRAY_RTV {
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
}
impl ::core::marker::Copy for D3D10_TEX2DMS_ARRAY_RTV {}
impl ::core::clone::Clone for D3D10_TEX2DMS_ARRAY_RTV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_TEX2DMS_ARRAY_SRV {
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
}
impl ::core::marker::Copy for D3D10_TEX2DMS_ARRAY_SRV {}
impl ::core::clone::Clone for D3D10_TEX2DMS_ARRAY_SRV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_TEX2DMS_DSV {
    pub UnusedField_NothingToDefine: u32,
}
impl ::core::marker::Copy for D3D10_TEX2DMS_DSV {}
impl ::core::clone::Clone for D3D10_TEX2DMS_DSV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_TEX2DMS_RTV {
    pub UnusedField_NothingToDefine: u32,
}
impl ::core::marker::Copy for D3D10_TEX2DMS_RTV {}
impl ::core::clone::Clone for D3D10_TEX2DMS_RTV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_TEX2DMS_SRV {
    pub UnusedField_NothingToDefine: u32,
}
impl ::core::marker::Copy for D3D10_TEX2DMS_SRV {}
impl ::core::clone::Clone for D3D10_TEX2DMS_SRV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_TEX2D_ARRAY_DSV {
    pub MipSlice: u32,
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
}
impl ::core::marker::Copy for D3D10_TEX2D_ARRAY_DSV {}
impl ::core::clone::Clone for D3D10_TEX2D_ARRAY_DSV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_TEX2D_ARRAY_RTV {
    pub MipSlice: u32,
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
}
impl ::core::marker::Copy for D3D10_TEX2D_ARRAY_RTV {}
impl ::core::clone::Clone for D3D10_TEX2D_ARRAY_RTV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_TEX2D_ARRAY_SRV {
    pub MostDetailedMip: u32,
    pub MipLevels: u32,
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
}
impl ::core::marker::Copy for D3D10_TEX2D_ARRAY_SRV {}
impl ::core::clone::Clone for D3D10_TEX2D_ARRAY_SRV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_TEX2D_DSV {
    pub MipSlice: u32,
}
impl ::core::marker::Copy for D3D10_TEX2D_DSV {}
impl ::core::clone::Clone for D3D10_TEX2D_DSV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_TEX2D_RTV {
    pub MipSlice: u32,
}
impl ::core::marker::Copy for D3D10_TEX2D_RTV {}
impl ::core::clone::Clone for D3D10_TEX2D_RTV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_TEX2D_SRV {
    pub MostDetailedMip: u32,
    pub MipLevels: u32,
}
impl ::core::marker::Copy for D3D10_TEX2D_SRV {}
impl ::core::clone::Clone for D3D10_TEX2D_SRV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_TEX3D_RTV {
    pub MipSlice: u32,
    pub FirstWSlice: u32,
    pub WSize: u32,
}
impl ::core::marker::Copy for D3D10_TEX3D_RTV {}
impl ::core::clone::Clone for D3D10_TEX3D_RTV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_TEX3D_SRV {
    pub MostDetailedMip: u32,
    pub MipLevels: u32,
}
impl ::core::marker::Copy for D3D10_TEX3D_SRV {}
impl ::core::clone::Clone for D3D10_TEX3D_SRV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_TEXCUBE_ARRAY_SRV1 {
    pub MostDetailedMip: u32,
    pub MipLevels: u32,
    pub First2DArrayFace: u32,
    pub NumCubes: u32,
}
impl ::core::marker::Copy for D3D10_TEXCUBE_ARRAY_SRV1 {}
impl ::core::clone::Clone for D3D10_TEXCUBE_ARRAY_SRV1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_TEXCUBE_SRV {
    pub MostDetailedMip: u32,
    pub MipLevels: u32,
}
impl ::core::marker::Copy for D3D10_TEXCUBE_SRV {}
impl ::core::clone::Clone for D3D10_TEXCUBE_SRV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub struct D3D10_TEXTURE1D_DESC {
    pub Width: u32,
    pub MipLevels: u32,
    pub ArraySize: u32,
    pub Format: super::Dxgi::Common::DXGI_FORMAT,
    pub Usage: D3D10_USAGE,
    pub BindFlags: u32,
    pub CPUAccessFlags: u32,
    pub MiscFlags: u32,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D10_TEXTURE1D_DESC {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D10_TEXTURE1D_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub struct D3D10_TEXTURE2D_DESC {
    pub Width: u32,
    pub Height: u32,
    pub MipLevels: u32,
    pub ArraySize: u32,
    pub Format: super::Dxgi::Common::DXGI_FORMAT,
    pub SampleDesc: super::Dxgi::Common::DXGI_SAMPLE_DESC,
    pub Usage: D3D10_USAGE,
    pub BindFlags: u32,
    pub CPUAccessFlags: u32,
    pub MiscFlags: u32,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D10_TEXTURE2D_DESC {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D10_TEXTURE2D_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub struct D3D10_TEXTURE3D_DESC {
    pub Width: u32,
    pub Height: u32,
    pub Depth: u32,
    pub MipLevels: u32,
    pub Format: super::Dxgi::Common::DXGI_FORMAT,
    pub Usage: D3D10_USAGE,
    pub BindFlags: u32,
    pub CPUAccessFlags: u32,
    pub MiscFlags: u32,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D10_TEXTURE3D_DESC {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D10_TEXTURE3D_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`*"]
pub struct D3D10_VIEWPORT {
    pub TopLeftX: i32,
    pub TopLeftY: i32,
    pub Width: u32,
    pub Height: u32,
    pub MinDepth: f32,
    pub MaxDepth: f32,
}
impl ::core::marker::Copy for D3D10_VIEWPORT {}
impl ::core::clone::Clone for D3D10_VIEWPORT {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Dxgi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Dxgi"))]
pub type PFN_D3D10_CREATE_DEVICE1 = ::core::option::Option<unsafe extern "system" fn(param0: super::Dxgi::IDXGIAdapter, param1: D3D10_DRIVER_TYPE, param2: super::super::Foundation::HINSTANCE, param3: u32, param4: D3D10_FEATURE_LEVEL1, param5: u32, param6: *mut ID3D10Device1) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D10\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Dxgi_Common"))]
pub type PFN_D3D10_CREATE_DEVICE_AND_SWAP_CHAIN1 = ::core::option::Option<unsafe extern "system" fn(param0: super::Dxgi::IDXGIAdapter, param1: D3D10_DRIVER_TYPE, param2: super::super::Foundation::HINSTANCE, param3: u32, param4: D3D10_FEATURE_LEVEL1, param5: u32, param6: *mut super::Dxgi::DXGI_SWAP_CHAIN_DESC, param7: *mut super::Dxgi::IDXGISwapChain, param8: *mut ID3D10Device1) -> ::windows_sys::core::HRESULT>;

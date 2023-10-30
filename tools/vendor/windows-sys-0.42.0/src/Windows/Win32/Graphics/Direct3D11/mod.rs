#[cfg_attr(windows, link(name = "windows"))]
extern "system" {
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Direct3D\"`, `\"Win32_Graphics_Dxgi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi"))]
    pub fn D3D11CreateDevice(padapter: super::Dxgi::IDXGIAdapter, drivertype: super::Direct3D::D3D_DRIVER_TYPE, software: super::super::Foundation::HINSTANCE, flags: D3D11_CREATE_DEVICE_FLAG, pfeaturelevels: *const super::Direct3D::D3D_FEATURE_LEVEL, featurelevels: u32, sdkversion: u32, ppdevice: *mut ID3D11Device, pfeaturelevel: *mut super::Direct3D::D3D_FEATURE_LEVEL, ppimmediatecontext: *mut ID3D11DeviceContext) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Direct3D\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common"))]
    pub fn D3D11CreateDeviceAndSwapChain(padapter: super::Dxgi::IDXGIAdapter, drivertype: super::Direct3D::D3D_DRIVER_TYPE, software: super::super::Foundation::HINSTANCE, flags: D3D11_CREATE_DEVICE_FLAG, pfeaturelevels: *const super::Direct3D::D3D_FEATURE_LEVEL, featurelevels: u32, sdkversion: u32, pswapchaindesc: *const super::Dxgi::DXGI_SWAP_CHAIN_DESC, ppswapchain: *mut super::Dxgi::IDXGISwapChain, ppdevice: *mut ID3D11Device, pfeaturelevel: *mut super::Direct3D::D3D_FEATURE_LEVEL, ppimmediatecontext: *mut ID3D11DeviceContext) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Direct3D\"`*"]
    #[cfg(feature = "Win32_Graphics_Direct3D")]
    pub fn D3DDisassemble11Trace(psrcdata: *const ::core::ffi::c_void, srcdatasize: usize, ptrace: ID3D11ShaderTrace, startstep: u32, numsteps: u32, flags: u32, ppdisassembly: *mut super::Direct3D::ID3DBlob) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
    pub fn D3DX11CreateFFT(pdevicecontext: ID3D11DeviceContext, pdesc: *const D3DX11_FFT_DESC, flags: u32, pbufferinfo: *mut D3DX11_FFT_BUFFER_INFO, ppfft: *mut ID3DX11FFT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
    pub fn D3DX11CreateFFT1DComplex(pdevicecontext: ID3D11DeviceContext, x: u32, flags: u32, pbufferinfo: *mut D3DX11_FFT_BUFFER_INFO, ppfft: *mut ID3DX11FFT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
    pub fn D3DX11CreateFFT1DReal(pdevicecontext: ID3D11DeviceContext, x: u32, flags: u32, pbufferinfo: *mut D3DX11_FFT_BUFFER_INFO, ppfft: *mut ID3DX11FFT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
    pub fn D3DX11CreateFFT2DComplex(pdevicecontext: ID3D11DeviceContext, x: u32, y: u32, flags: u32, pbufferinfo: *mut D3DX11_FFT_BUFFER_INFO, ppfft: *mut ID3DX11FFT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
    pub fn D3DX11CreateFFT2DReal(pdevicecontext: ID3D11DeviceContext, x: u32, y: u32, flags: u32, pbufferinfo: *mut D3DX11_FFT_BUFFER_INFO, ppfft: *mut ID3DX11FFT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
    pub fn D3DX11CreateFFT3DComplex(pdevicecontext: ID3D11DeviceContext, x: u32, y: u32, z: u32, flags: u32, pbufferinfo: *mut D3DX11_FFT_BUFFER_INFO, ppfft: *mut ID3DX11FFT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
    pub fn D3DX11CreateFFT3DReal(pdevicecontext: ID3D11DeviceContext, x: u32, y: u32, z: u32, flags: u32, pbufferinfo: *mut D3DX11_FFT_BUFFER_INFO, ppfft: *mut ID3DX11FFT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
    pub fn D3DX11CreateScan(pdevicecontext: ID3D11DeviceContext, maxelementscansize: u32, maxscancount: u32, ppscan: *mut ID3DX11Scan) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
    pub fn D3DX11CreateSegmentedScan(pdevicecontext: ID3D11DeviceContext, maxelementscansize: u32, ppscan: *mut ID3DX11SegmentedScan) -> ::windows_sys::core::HRESULT;
}
pub type ID3D11Asynchronous = *mut ::core::ffi::c_void;
pub type ID3D11AuthenticatedChannel = *mut ::core::ffi::c_void;
pub type ID3D11BlendState = *mut ::core::ffi::c_void;
pub type ID3D11BlendState1 = *mut ::core::ffi::c_void;
pub type ID3D11Buffer = *mut ::core::ffi::c_void;
pub type ID3D11ClassInstance = *mut ::core::ffi::c_void;
pub type ID3D11ClassLinkage = *mut ::core::ffi::c_void;
pub type ID3D11CommandList = *mut ::core::ffi::c_void;
pub type ID3D11ComputeShader = *mut ::core::ffi::c_void;
pub type ID3D11Counter = *mut ::core::ffi::c_void;
pub type ID3D11CryptoSession = *mut ::core::ffi::c_void;
pub type ID3D11Debug = *mut ::core::ffi::c_void;
pub type ID3D11DepthStencilState = *mut ::core::ffi::c_void;
pub type ID3D11DepthStencilView = *mut ::core::ffi::c_void;
pub type ID3D11Device = *mut ::core::ffi::c_void;
pub type ID3D11Device1 = *mut ::core::ffi::c_void;
pub type ID3D11Device2 = *mut ::core::ffi::c_void;
pub type ID3D11Device3 = *mut ::core::ffi::c_void;
pub type ID3D11Device4 = *mut ::core::ffi::c_void;
pub type ID3D11Device5 = *mut ::core::ffi::c_void;
pub type ID3D11DeviceChild = *mut ::core::ffi::c_void;
pub type ID3D11DeviceContext = *mut ::core::ffi::c_void;
pub type ID3D11DeviceContext1 = *mut ::core::ffi::c_void;
pub type ID3D11DeviceContext2 = *mut ::core::ffi::c_void;
pub type ID3D11DeviceContext3 = *mut ::core::ffi::c_void;
pub type ID3D11DeviceContext4 = *mut ::core::ffi::c_void;
pub type ID3D11DomainShader = *mut ::core::ffi::c_void;
pub type ID3D11Fence = *mut ::core::ffi::c_void;
pub type ID3D11FunctionLinkingGraph = *mut ::core::ffi::c_void;
pub type ID3D11FunctionParameterReflection = *mut ::core::ffi::c_void;
pub type ID3D11FunctionReflection = *mut ::core::ffi::c_void;
pub type ID3D11GeometryShader = *mut ::core::ffi::c_void;
pub type ID3D11HullShader = *mut ::core::ffi::c_void;
pub type ID3D11InfoQueue = *mut ::core::ffi::c_void;
pub type ID3D11InputLayout = *mut ::core::ffi::c_void;
pub type ID3D11LibraryReflection = *mut ::core::ffi::c_void;
pub type ID3D11Linker = *mut ::core::ffi::c_void;
pub type ID3D11LinkingNode = *mut ::core::ffi::c_void;
pub type ID3D11Module = *mut ::core::ffi::c_void;
pub type ID3D11ModuleInstance = *mut ::core::ffi::c_void;
pub type ID3D11Multithread = *mut ::core::ffi::c_void;
pub type ID3D11PixelShader = *mut ::core::ffi::c_void;
pub type ID3D11Predicate = *mut ::core::ffi::c_void;
pub type ID3D11Query = *mut ::core::ffi::c_void;
pub type ID3D11Query1 = *mut ::core::ffi::c_void;
pub type ID3D11RasterizerState = *mut ::core::ffi::c_void;
pub type ID3D11RasterizerState1 = *mut ::core::ffi::c_void;
pub type ID3D11RasterizerState2 = *mut ::core::ffi::c_void;
pub type ID3D11RefDefaultTrackingOptions = *mut ::core::ffi::c_void;
pub type ID3D11RefTrackingOptions = *mut ::core::ffi::c_void;
pub type ID3D11RenderTargetView = *mut ::core::ffi::c_void;
pub type ID3D11RenderTargetView1 = *mut ::core::ffi::c_void;
pub type ID3D11Resource = *mut ::core::ffi::c_void;
pub type ID3D11SamplerState = *mut ::core::ffi::c_void;
pub type ID3D11ShaderReflection = *mut ::core::ffi::c_void;
pub type ID3D11ShaderReflectionConstantBuffer = *mut ::core::ffi::c_void;
pub type ID3D11ShaderReflectionType = *mut ::core::ffi::c_void;
pub type ID3D11ShaderReflectionVariable = *mut ::core::ffi::c_void;
pub type ID3D11ShaderResourceView = *mut ::core::ffi::c_void;
pub type ID3D11ShaderResourceView1 = *mut ::core::ffi::c_void;
pub type ID3D11ShaderTrace = *mut ::core::ffi::c_void;
pub type ID3D11ShaderTraceFactory = *mut ::core::ffi::c_void;
pub type ID3D11SwitchToRef = *mut ::core::ffi::c_void;
pub type ID3D11Texture1D = *mut ::core::ffi::c_void;
pub type ID3D11Texture2D = *mut ::core::ffi::c_void;
pub type ID3D11Texture2D1 = *mut ::core::ffi::c_void;
pub type ID3D11Texture3D = *mut ::core::ffi::c_void;
pub type ID3D11Texture3D1 = *mut ::core::ffi::c_void;
pub type ID3D11TracingDevice = *mut ::core::ffi::c_void;
pub type ID3D11UnorderedAccessView = *mut ::core::ffi::c_void;
pub type ID3D11UnorderedAccessView1 = *mut ::core::ffi::c_void;
pub type ID3D11VertexShader = *mut ::core::ffi::c_void;
pub type ID3D11VideoContext = *mut ::core::ffi::c_void;
pub type ID3D11VideoContext1 = *mut ::core::ffi::c_void;
pub type ID3D11VideoContext2 = *mut ::core::ffi::c_void;
pub type ID3D11VideoContext3 = *mut ::core::ffi::c_void;
pub type ID3D11VideoDecoder = *mut ::core::ffi::c_void;
pub type ID3D11VideoDecoderOutputView = *mut ::core::ffi::c_void;
pub type ID3D11VideoDevice = *mut ::core::ffi::c_void;
pub type ID3D11VideoDevice1 = *mut ::core::ffi::c_void;
pub type ID3D11VideoDevice2 = *mut ::core::ffi::c_void;
pub type ID3D11VideoProcessor = *mut ::core::ffi::c_void;
pub type ID3D11VideoProcessorEnumerator = *mut ::core::ffi::c_void;
pub type ID3D11VideoProcessorEnumerator1 = *mut ::core::ffi::c_void;
pub type ID3D11VideoProcessorInputView = *mut ::core::ffi::c_void;
pub type ID3D11VideoProcessorOutputView = *mut ::core::ffi::c_void;
pub type ID3D11View = *mut ::core::ffi::c_void;
pub type ID3DDeviceContextState = *mut ::core::ffi::c_void;
pub type ID3DUserDefinedAnnotation = *mut ::core::ffi::c_void;
pub type ID3DX11FFT = *mut ::core::ffi::c_void;
pub type ID3DX11Scan = *mut ::core::ffi::c_void;
pub type ID3DX11SegmentedScan = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_16BIT_INDEX_STRIP_CUT_VALUE: u32 = 65535u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_1_UAV_SLOT_COUNT: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_2_TILED_RESOURCE_TILE_SIZE_IN_BYTES: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_32BIT_INDEX_STRIP_CUT_VALUE: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_4_VIDEO_DECODER_HISTOGRAM_OFFSET_ALIGNMENT: u32 = 256u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_4_VIDEO_DECODER_MAX_HISTOGRAM_COMPONENTS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_8BIT_INDEX_STRIP_CUT_VALUE: u32 = 255u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_ANISOTROPIC_FILTERING_BIT: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_APPEND_ALIGNED_ELEMENT: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_APPNAME_STRING: &str = "Name";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_APPSIZE_STRING: &str = "Size";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_ARRAY_AXIS_ADDRESS_RANGE_BIT_COUNT: u32 = 9u32;
pub const D3D11_AUTHENTICATED_CONFIGURE_CRYPTO_SESSION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1665584212, data2: 11516, data3: 19156, data4: [130, 36, 209, 88, 55, 222, 119, 0] };
pub const D3D11_AUTHENTICATED_CONFIGURE_ENCRYPTION_WHEN_ACCESSIBLE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1107292806, data2: 27360, data3: 19779, data4: [157, 85, 164, 110, 158, 253, 21, 138] };
pub const D3D11_AUTHENTICATED_CONFIGURE_INITIALIZE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 101796827, data2: 13603, data3: 18186, data4: [141, 202, 251, 194, 132, 81, 84, 240] };
pub const D3D11_AUTHENTICATED_CONFIGURE_PROTECTION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1346721368, data2: 16199, data3: 17250, data4: [191, 153, 191, 223, 205, 233, 237, 41] };
pub const D3D11_AUTHENTICATED_CONFIGURE_SHARED_RESOURCE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 124964935, data2: 6976, data3: 18664, data4: [156, 166, 181, 245, 16, 222, 159, 1] };
pub const D3D11_AUTHENTICATED_QUERY_ACCESSIBILITY_ATTRIBUTES: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1645533650, data2: 17196, data3: 19131, data4: [159, 206, 33, 110, 234, 38, 158, 59] };
pub const D3D11_AUTHENTICATED_QUERY_CHANNEL_TYPE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3155892389, data2: 45563, data3: 17067, data4: [189, 148, 181, 130, 139, 75, 247, 190] };
pub const D3D11_AUTHENTICATED_QUERY_CRYPTO_SESSION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 640960926, data2: 53272, data3: 19828, data4: [172, 23, 127, 114, 64, 89, 82, 141] };
pub const D3D11_AUTHENTICATED_QUERY_CURRENT_ENCRYPTION_WHEN_ACCESSIBLE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3960967623, data2: 56019, data3: 20245, data4: [158, 195, 250, 169, 61, 96, 212, 240] };
pub const D3D11_AUTHENTICATED_QUERY_DEVICE_HANDLE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3961279389, data2: 36095, data3: 20010, data4: [188, 196, 245, 105, 47, 153, 244, 128] };
pub const D3D11_AUTHENTICATED_QUERY_ENCRYPTION_WHEN_ACCESSIBLE_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4164573528, data2: 59782, data3: 19418, data4: [190, 176, 65, 31, 106, 122, 1, 183] };
pub const D3D11_AUTHENTICATED_QUERY_ENCRYPTION_WHEN_ACCESSIBLE_GUID_COUNT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3004133478, data2: 8252, data3: 19207, data4: [147, 252, 206, 170, 253, 97, 36, 30] };
pub const D3D11_AUTHENTICATED_QUERY_OUTPUT_ID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2208160931, data2: 39758, data3: 16868, data4: [176, 83, 137, 43, 210, 161, 30, 231] };
pub const D3D11_AUTHENTICATED_QUERY_OUTPUT_ID_COUNT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 738470750, data2: 35847, data3: 18133, data4: [170, 190, 143, 117, 203, 173, 76, 49] };
pub const D3D11_AUTHENTICATED_QUERY_PROTECTION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2823730564, data2: 50325, data3: 18602, data4: [185, 77, 139, 210, 214, 251, 206, 5] };
pub const D3D11_AUTHENTICATED_QUERY_RESTRICTED_SHARED_RESOURCE_PROCESS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1687927515, data2: 61684, data3: 17977, data4: [161, 91, 36, 57, 63, 195, 171, 172] };
pub const D3D11_AUTHENTICATED_QUERY_RESTRICTED_SHARED_RESOURCE_PROCESS_COUNT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 229771187, data2: 37968, data3: 18086, data4: [130, 222, 27, 150, 212, 79, 156, 242] };
pub const D3D11_AUTHENTICATED_QUERY_UNRESTRICTED_PROTECTED_SHARED_RESOURCE_COUNT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 19860438, data2: 58978, data3: 17524, data4: [190, 253, 170, 83, 229, 20, 60, 109] };
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BREAKON_CATEGORY: &str = "BreakOn_CATEGORY_%s";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BREAKON_ID_DECIMAL: &str = "BreakOn_ID_%d";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BREAKON_ID_STRING: &str = "BreakOn_ID_%s";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BREAKON_SEVERITY: &str = "BreakOn_SEVERITY_%s";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CLIP_OR_CULL_DISTANCE_COUNT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CLIP_OR_CULL_DISTANCE_ELEMENT_COUNT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_CONSTANT_BUFFER_API_SLOT_COUNT: u32 = 14u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_CONSTANT_BUFFER_COMPONENTS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_CONSTANT_BUFFER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_CONSTANT_BUFFER_HW_SLOT_COUNT: u32 = 15u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_CONSTANT_BUFFER_PARTIAL_UPDATE_EXTENTS_BYTE_ALIGNMENT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_CONSTANT_BUFFER_REGISTER_COMPONENTS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_CONSTANT_BUFFER_REGISTER_COUNT: u32 = 15u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_CONSTANT_BUFFER_REGISTER_READS_PER_INST: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_CONSTANT_BUFFER_REGISTER_READ_PORTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_FLOWCONTROL_NESTING_LIMIT: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_IMMEDIATE_CONSTANT_BUFFER_REGISTER_COMPONENTS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_IMMEDIATE_CONSTANT_BUFFER_REGISTER_COUNT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_IMMEDIATE_CONSTANT_BUFFER_REGISTER_READS_PER_INST: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_IMMEDIATE_CONSTANT_BUFFER_REGISTER_READ_PORTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_IMMEDIATE_VALUE_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_INPUT_RESOURCE_REGISTER_COMPONENTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_INPUT_RESOURCE_REGISTER_COUNT: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_INPUT_RESOURCE_REGISTER_READS_PER_INST: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_INPUT_RESOURCE_REGISTER_READ_PORTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_INPUT_RESOURCE_SLOT_COUNT: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_SAMPLER_REGISTER_COMPONENTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_SAMPLER_REGISTER_COUNT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_SAMPLER_REGISTER_READS_PER_INST: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_SAMPLER_REGISTER_READ_PORTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_SAMPLER_SLOT_COUNT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_SUBROUTINE_NESTING_LIMIT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_TEMP_REGISTER_COMPONENTS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_TEMP_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_TEMP_REGISTER_COUNT: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_TEMP_REGISTER_READS_PER_INST: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_TEMP_REGISTER_READ_PORTS: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_TEXCOORD_RANGE_REDUCTION_MAX: u32 = 10u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_TEXCOORD_RANGE_REDUCTION_MIN: i32 = -10i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_TEXEL_OFFSET_MAX_NEGATIVE: i32 = -8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMMONSHADER_TEXEL_OFFSET_MAX_POSITIVE: u32 = 7u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMPARISON_FILTERING_BIT: u32 = 128u32;
pub const D3D11_CRYPTO_TYPE_AES128_CTR: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2607535889, data2: 20340, data3: 16841, data4: [158, 123, 11, 226, 215, 217, 59, 79] };
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET00_MAX_BYTES_TGSM_WRITABLE_PER_THREAD: u32 = 256u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET00_MAX_NUM_THREADS_PER_GROUP: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET01_MAX_BYTES_TGSM_WRITABLE_PER_THREAD: u32 = 240u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET01_MAX_NUM_THREADS_PER_GROUP: u32 = 68u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET02_MAX_BYTES_TGSM_WRITABLE_PER_THREAD: u32 = 224u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET02_MAX_NUM_THREADS_PER_GROUP: u32 = 72u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET03_MAX_BYTES_TGSM_WRITABLE_PER_THREAD: u32 = 208u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET03_MAX_NUM_THREADS_PER_GROUP: u32 = 76u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET04_MAX_BYTES_TGSM_WRITABLE_PER_THREAD: u32 = 192u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET04_MAX_NUM_THREADS_PER_GROUP: u32 = 84u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET05_MAX_BYTES_TGSM_WRITABLE_PER_THREAD: u32 = 176u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET05_MAX_NUM_THREADS_PER_GROUP: u32 = 92u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET06_MAX_BYTES_TGSM_WRITABLE_PER_THREAD: u32 = 160u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET06_MAX_NUM_THREADS_PER_GROUP: u32 = 100u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET07_MAX_BYTES_TGSM_WRITABLE_PER_THREAD: u32 = 144u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET07_MAX_NUM_THREADS_PER_GROUP: u32 = 112u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET08_MAX_BYTES_TGSM_WRITABLE_PER_THREAD: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET08_MAX_NUM_THREADS_PER_GROUP: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET09_MAX_BYTES_TGSM_WRITABLE_PER_THREAD: u32 = 112u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET09_MAX_NUM_THREADS_PER_GROUP: u32 = 144u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET10_MAX_BYTES_TGSM_WRITABLE_PER_THREAD: u32 = 96u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET10_MAX_NUM_THREADS_PER_GROUP: u32 = 168u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET11_MAX_BYTES_TGSM_WRITABLE_PER_THREAD: u32 = 80u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET11_MAX_NUM_THREADS_PER_GROUP: u32 = 204u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET12_MAX_BYTES_TGSM_WRITABLE_PER_THREAD: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET12_MAX_NUM_THREADS_PER_GROUP: u32 = 256u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET13_MAX_BYTES_TGSM_WRITABLE_PER_THREAD: u32 = 48u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET13_MAX_NUM_THREADS_PER_GROUP: u32 = 340u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET14_MAX_BYTES_TGSM_WRITABLE_PER_THREAD: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET14_MAX_NUM_THREADS_PER_GROUP: u32 = 512u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET15_MAX_BYTES_TGSM_WRITABLE_PER_THREAD: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_BUCKET15_MAX_NUM_THREADS_PER_GROUP: u32 = 768u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_DISPATCH_MAX_THREAD_GROUPS_IN_Z_DIMENSION: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_RAW_UAV_BYTE_ALIGNMENT: u32 = 256u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_THREAD_GROUP_MAX_THREADS_PER_GROUP: u32 = 768u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_THREAD_GROUP_MAX_X: u32 = 768u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_THREAD_GROUP_MAX_Y: u32 = 768u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_4_X_UAV_REGISTER_COUNT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_DISPATCH_MAX_THREAD_GROUPS_PER_DIMENSION: u32 = 65535u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_TGSM_REGISTER_COUNT: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_TGSM_REGISTER_READS_PER_INST: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_TGSM_RESOURCE_REGISTER_COMPONENTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_TGSM_RESOURCE_REGISTER_READ_PORTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_THREADGROUPID_REGISTER_COMPONENTS: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_THREADGROUPID_REGISTER_COUNT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_THREADIDINGROUPFLATTENED_REGISTER_COMPONENTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_THREADIDINGROUPFLATTENED_REGISTER_COUNT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_THREADIDINGROUP_REGISTER_COMPONENTS: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_THREADIDINGROUP_REGISTER_COUNT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_THREADID_REGISTER_COMPONENTS: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_THREADID_REGISTER_COUNT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_THREAD_GROUP_MAX_THREADS_PER_GROUP: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_THREAD_GROUP_MAX_X: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_THREAD_GROUP_MAX_Y: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_THREAD_GROUP_MAX_Z: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_THREAD_GROUP_MIN_X: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_THREAD_GROUP_MIN_Y: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_THREAD_GROUP_MIN_Z: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CS_THREAD_LOCAL_TEMP_REGISTER_POOL: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEBUG_FEATURE_ALWAYS_DISCARD_OFFERED_RESOURCE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEBUG_FEATURE_AVOID_BEHAVIOR_CHANGING_DEBUG_AIDS: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEBUG_FEATURE_DISABLE_TILED_RESOURCE_MAPPING_TRACKING_AND_VALIDATION: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEBUG_FEATURE_FINISH_PER_RENDER_OP: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEBUG_FEATURE_FLUSH_PER_RENDER_OP: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEBUG_FEATURE_NEVER_DISCARD_OFFERED_RESOURCE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEBUG_FEATURE_PRESENT_PER_RENDER_OP: u32 = 4u32;
pub const D3D11_DECODER_BITSTREAM_ENCRYPTION_TYPE_CBCS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1110283033, data2: 40225, data3: 19383, data4: [147, 113, 250, 245, 168, 44, 62, 4] };
pub const D3D11_DECODER_BITSTREAM_ENCRYPTION_TYPE_CENC: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2957005365, data2: 49469, data3: 17650, data4: [154, 229, 221, 72, 224, 142, 91, 103] };
pub const D3D11_DECODER_ENCRYPTION_HW_CENC: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2312547407, data2: 2546, data3: 16937, data4: [178, 205, 55, 116, 10, 109, 253, 129] };
pub const D3D11_DECODER_PROFILE_AV1_VLD_12BIT_PROFILE2: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 387084297, data2: 40975, data3: 19681, data4: [153, 78, 191, 64, 129, 246, 243, 240] };
pub const D3D11_DECODER_PROFILE_AV1_VLD_12BIT_PROFILE2_420: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 763412182, data2: 40108, data3: 18485, data4: [158, 145, 50, 123, 188, 79, 158, 232] };
pub const D3D11_DECODER_PROFILE_AV1_VLD_PROFILE0: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3099479243, data2: 53075, data3: 18106, data4: [141, 89, 214, 184, 166, 218, 93, 42] };
pub const D3D11_DECODER_PROFILE_AV1_VLD_PROFILE1: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1765211919, data2: 17841, data3: 16739, data4: [156, 193, 100, 110, 246, 148, 97, 8] };
pub const D3D11_DECODER_PROFILE_AV1_VLD_PROFILE2: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 207563425, data2: 58689, data3: 16521, data4: [187, 123, 152, 17, 10, 25, 215, 200] };
pub const D3D11_DECODER_PROFILE_H264_IDCT_FGT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487719, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const D3D11_DECODER_PROFILE_H264_IDCT_NOFGT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487718, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const D3D11_DECODER_PROFILE_H264_MOCOMP_FGT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487717, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const D3D11_DECODER_PROFILE_H264_MOCOMP_NOFGT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487716, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const D3D11_DECODER_PROFILE_H264_VLD_FGT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487721, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const D3D11_DECODER_PROFILE_H264_VLD_MULTIVIEW_NOFGT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1885052290, data2: 30415, data3: 18902, data4: [183, 230, 172, 136, 114, 219, 1, 60] };
pub const D3D11_DECODER_PROFILE_H264_VLD_NOFGT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487720, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const D3D11_DECODER_PROFILE_H264_VLD_STEREO_NOFGT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4188720315, data2: 49846, data3: 19708, data4: [135, 121, 87, 7, 177, 118, 5, 82] };
pub const D3D11_DECODER_PROFILE_H264_VLD_STEREO_PROGRESSIVE_NOFGT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3617319130, data2: 3313, data3: 19585, data4: [184, 42, 105, 164, 226, 54, 244, 61] };
pub const D3D11_DECODER_PROFILE_H264_VLD_WITHFMOASO_NOFGT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3589296121, data2: 13336, data3: 17880, data4: [149, 97, 50, 167, 106, 174, 45, 221] };
pub const D3D11_DECODER_PROFILE_HEVC_VLD_MAIN: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1527895323, data2: 12108, data3: 17490, data4: [188, 195, 9, 242, 161, 22, 12, 192] };
pub const D3D11_DECODER_PROFILE_HEVC_VLD_MAIN10: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 276492512, data2: 61210, data3: 19737, data4: [171, 168, 103, 161, 99, 7, 61, 19] };
pub const D3D11_DECODER_PROFILE_MPEG1_VLD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1866385177, data2: 14133, data3: 17100, data4: [128, 99, 101, 204, 60, 179, 102, 22] };
pub const D3D11_DECODER_PROFILE_MPEG2_IDCT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3206720768, data2: 1002, data3: 18064, data4: [128, 119, 71, 51, 70, 32, 155, 126] };
pub const D3D11_DECODER_PROFILE_MPEG2_MOCOMP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3869897803, data2: 25008, data3: 17763, data4: [158, 164, 99, 210, 163, 198, 254, 102] };
pub const D3D11_DECODER_PROFILE_MPEG2_VLD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3995550079, data2: 24104, data3: 20069, data4: [190, 234, 29, 38, 181, 8, 173, 201] };
pub const D3D11_DECODER_PROFILE_MPEG2and1_VLD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2255052562, data2: 13326, data3: 20228, data4: [159, 211, 146, 83, 221, 50, 116, 96] };
pub const D3D11_DECODER_PROFILE_MPEG4PT2_VLD_ADVSIMPLE_GMC: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2878966619, data2: 16984, data3: 17577, data4: [159, 235, 148, 229, 151, 166, 186, 174] };
pub const D3D11_DECODER_PROFILE_MPEG4PT2_VLD_ADVSIMPLE_NOGMC: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3980495519, data2: 269, data3: 20186, data4: [154, 227, 154, 101, 53, 141, 141, 46] };
pub const D3D11_DECODER_PROFILE_MPEG4PT2_VLD_SIMPLE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4023799156, data2: 51688, data3: 16855, data4: [165, 233, 233, 176, 227, 159, 163, 25] };
pub const D3D11_DECODER_PROFILE_VC1_D2010: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487780, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const D3D11_DECODER_PROFILE_VC1_IDCT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487778, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const D3D11_DECODER_PROFILE_VC1_MOCOMP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487777, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const D3D11_DECODER_PROFILE_VC1_POSTPROC: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487776, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const D3D11_DECODER_PROFILE_VC1_VLD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487779, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const D3D11_DECODER_PROFILE_VP8_VLD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2428017130, data2: 14946, data3: 18181, data4: [136, 179, 141, 240, 75, 39, 68, 231] };
pub const D3D11_DECODER_PROFILE_VP9_VLD_10BIT_PROFILE2: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2764524015, data2: 28367, data3: 18602, data4: [132, 72, 80, 167, 161, 22, 95, 247] };
pub const D3D11_DECODER_PROFILE_VP9_VLD_PROFILE0: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1178011640, data2: 41424, data3: 17797, data4: [135, 109, 131, 170, 109, 96, 184, 158] };
pub const D3D11_DECODER_PROFILE_WMV8_MOCOMP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487745, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const D3D11_DECODER_PROFILE_WMV8_POSTPROC: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487744, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const D3D11_DECODER_PROFILE_WMV9_IDCT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487764, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const D3D11_DECODER_PROFILE_WMV9_MOCOMP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487761, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
pub const D3D11_DECODER_PROFILE_WMV9_POSTPROC: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 461487760, data2: 41159, data3: 4563, data4: [185, 132, 0, 192, 79, 46, 115, 197] };
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEFAULT_BLEND_FACTOR_ALPHA: f32 = 1f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEFAULT_BLEND_FACTOR_BLUE: f32 = 1f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEFAULT_BLEND_FACTOR_GREEN: f32 = 1f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEFAULT_BLEND_FACTOR_RED: f32 = 1f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEFAULT_BORDER_COLOR_COMPONENT: f32 = 0f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEFAULT_DEPTH_BIAS: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEFAULT_DEPTH_BIAS_CLAMP: f32 = 0f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEFAULT_MAX_ANISOTROPY: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEFAULT_MIP_LOD_BIAS: f32 = 0f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEFAULT_RENDER_TARGET_ARRAY_INDEX: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEFAULT_SAMPLE_MASK: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEFAULT_SCISSOR_ENDX: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEFAULT_SCISSOR_ENDY: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEFAULT_SCISSOR_STARTX: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEFAULT_SCISSOR_STARTY: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEFAULT_SLOPE_SCALED_DEPTH_BIAS: f32 = 0f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEFAULT_STENCIL_READ_MASK: u32 = 255u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEFAULT_STENCIL_REFERENCE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEFAULT_STENCIL_WRITE_MASK: u32 = 255u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEFAULT_VIEWPORT_AND_SCISSORRECT_INDEX: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEFAULT_VIEWPORT_HEIGHT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEFAULT_VIEWPORT_MAX_DEPTH: f32 = 0f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEFAULT_VIEWPORT_MIN_DEPTH: f32 = 0f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEFAULT_VIEWPORT_TOPLEFTX: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEFAULT_VIEWPORT_TOPLEFTY: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEFAULT_VIEWPORT_WIDTH: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DS_INPUT_CONTROL_POINTS_MAX_TOTAL_SCALARS: u32 = 3968u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DS_INPUT_CONTROL_POINT_REGISTER_COMPONENTS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DS_INPUT_CONTROL_POINT_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DS_INPUT_CONTROL_POINT_REGISTER_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DS_INPUT_CONTROL_POINT_REGISTER_READS_PER_INST: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DS_INPUT_CONTROL_POINT_REGISTER_READ_PORTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DS_INPUT_DOMAIN_POINT_REGISTER_COMPONENTS: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DS_INPUT_DOMAIN_POINT_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DS_INPUT_DOMAIN_POINT_REGISTER_COUNT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DS_INPUT_DOMAIN_POINT_REGISTER_READS_PER_INST: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DS_INPUT_DOMAIN_POINT_REGISTER_READ_PORTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DS_INPUT_PATCH_CONSTANT_REGISTER_COMPONENTS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DS_INPUT_PATCH_CONSTANT_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DS_INPUT_PATCH_CONSTANT_REGISTER_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DS_INPUT_PATCH_CONSTANT_REGISTER_READS_PER_INST: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DS_INPUT_PATCH_CONSTANT_REGISTER_READ_PORTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DS_INPUT_PRIMITIVE_ID_REGISTER_COMPONENTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DS_INPUT_PRIMITIVE_ID_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DS_INPUT_PRIMITIVE_ID_REGISTER_COUNT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DS_INPUT_PRIMITIVE_ID_REGISTER_READS_PER_INST: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DS_INPUT_PRIMITIVE_ID_REGISTER_READ_PORTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DS_OUTPUT_REGISTER_COMPONENTS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DS_OUTPUT_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DS_OUTPUT_REGISTER_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_ENABLE_BREAK_ON_MESSAGE: &str = "EnableBreakOnMessage";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_REDUCTION_TYPE_MASK: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_REDUCTION_TYPE_SHIFT: u32 = 7u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_TYPE_MASK: u32 = 3u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FLOAT16_FUSED_TOLERANCE_IN_ULP: f64 = 0.6f64;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FLOAT32_MAX: f32 = 340282350000000000000000000000000000000f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FLOAT32_TO_INTEGER_TOLERANCE_IN_ULP: f32 = 0.6f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FLOAT_TO_SRGB_EXPONENT_DENOMINATOR: f32 = 2.4f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FLOAT_TO_SRGB_EXPONENT_NUMERATOR: f32 = 1f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FLOAT_TO_SRGB_OFFSET: f32 = 0.055f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FLOAT_TO_SRGB_SCALE_1: f32 = 12.92f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FLOAT_TO_SRGB_SCALE_2: f32 = 1.055f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FLOAT_TO_SRGB_THRESHOLD: f32 = 0.0031308f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORCE_DEBUGGABLE: &str = "ForceDebuggable";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORCE_SHADER_SKIP_OPTIMIZATION: &str = "ForceShaderSkipOptimization";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FTOI_INSTRUCTION_MAX_INPUT: f32 = 2147483600f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FTOI_INSTRUCTION_MIN_INPUT: f32 = -2147483600f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FTOU_INSTRUCTION_MAX_INPUT: f32 = 4294967300f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FTOU_INSTRUCTION_MIN_INPUT: f32 = 0f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_GS_INPUT_INSTANCE_ID_READS_PER_INST: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_GS_INPUT_INSTANCE_ID_READ_PORTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_GS_INPUT_INSTANCE_ID_REGISTER_COMPONENTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_GS_INPUT_INSTANCE_ID_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_GS_INPUT_INSTANCE_ID_REGISTER_COUNT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_GS_INPUT_PRIM_CONST_REGISTER_COMPONENTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_GS_INPUT_PRIM_CONST_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_GS_INPUT_PRIM_CONST_REGISTER_COUNT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_GS_INPUT_PRIM_CONST_REGISTER_READS_PER_INST: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_GS_INPUT_PRIM_CONST_REGISTER_READ_PORTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_GS_INPUT_REGISTER_COMPONENTS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_GS_INPUT_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_GS_INPUT_REGISTER_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_GS_INPUT_REGISTER_READS_PER_INST: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_GS_INPUT_REGISTER_READ_PORTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_GS_INPUT_REGISTER_VERTICES: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_GS_MAX_INSTANCE_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_GS_MAX_OUTPUT_VERTEX_COUNT_ACROSS_INSTANCES: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_GS_OUTPUT_ELEMENTS: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_GS_OUTPUT_REGISTER_COMPONENTS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_GS_OUTPUT_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_GS_OUTPUT_REGISTER_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_CONTROL_POINT_PHASE_INPUT_REGISTER_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_CONTROL_POINT_PHASE_OUTPUT_REGISTER_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_CONTROL_POINT_REGISTER_COMPONENTS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_CONTROL_POINT_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_CONTROL_POINT_REGISTER_READS_PER_INST: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_CONTROL_POINT_REGISTER_READ_PORTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_FORK_PHASE_INSTANCE_COUNT_UPPER_BOUND: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_INPUT_FORK_INSTANCE_ID_REGISTER_COMPONENTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_INPUT_FORK_INSTANCE_ID_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_INPUT_FORK_INSTANCE_ID_REGISTER_COUNT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_INPUT_FORK_INSTANCE_ID_REGISTER_READS_PER_INST: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_INPUT_FORK_INSTANCE_ID_REGISTER_READ_PORTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_INPUT_JOIN_INSTANCE_ID_REGISTER_COMPONENTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_INPUT_JOIN_INSTANCE_ID_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_INPUT_JOIN_INSTANCE_ID_REGISTER_COUNT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_INPUT_JOIN_INSTANCE_ID_REGISTER_READS_PER_INST: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_INPUT_JOIN_INSTANCE_ID_REGISTER_READ_PORTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_INPUT_PRIMITIVE_ID_REGISTER_COMPONENTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_INPUT_PRIMITIVE_ID_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_INPUT_PRIMITIVE_ID_REGISTER_COUNT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_INPUT_PRIMITIVE_ID_REGISTER_READS_PER_INST: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_INPUT_PRIMITIVE_ID_REGISTER_READ_PORTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_JOIN_PHASE_INSTANCE_COUNT_UPPER_BOUND: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_MAXTESSFACTOR_LOWER_BOUND: f32 = 1f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_MAXTESSFACTOR_UPPER_BOUND: f32 = 64f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_OUTPUT_CONTROL_POINTS_MAX_TOTAL_SCALARS: u32 = 3968u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_OUTPUT_CONTROL_POINT_ID_REGISTER_COMPONENTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_OUTPUT_CONTROL_POINT_ID_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_OUTPUT_CONTROL_POINT_ID_REGISTER_COUNT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_OUTPUT_CONTROL_POINT_ID_REGISTER_READS_PER_INST: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_OUTPUT_CONTROL_POINT_ID_REGISTER_READ_PORTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_OUTPUT_PATCH_CONSTANT_REGISTER_COMPONENTS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_OUTPUT_PATCH_CONSTANT_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_OUTPUT_PATCH_CONSTANT_REGISTER_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_OUTPUT_PATCH_CONSTANT_REGISTER_READS_PER_INST: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_OUTPUT_PATCH_CONSTANT_REGISTER_READ_PORTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HS_OUTPUT_PATCH_CONSTANT_REGISTER_SCALAR_COMPONENTS: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_IA_DEFAULT_INDEX_BUFFER_OFFSET_IN_BYTES: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_IA_DEFAULT_PRIMITIVE_TOPOLOGY: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_IA_DEFAULT_VERTEX_BUFFER_OFFSET_IN_BYTES: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_IA_INDEX_INPUT_RESOURCE_SLOT_COUNT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_IA_INSTANCE_ID_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_IA_INTEGER_ARITHMETIC_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_IA_PATCH_MAX_CONTROL_POINT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_IA_PRIMITIVE_ID_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_IA_VERTEX_ID_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_IA_VERTEX_INPUT_RESOURCE_SLOT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_IA_VERTEX_INPUT_STRUCTURE_ELEMENTS_COMPONENTS: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_IA_VERTEX_INPUT_STRUCTURE_ELEMENT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_INFOQUEUE_STORAGE_FILTER_OVERRIDE: &str = "InfoQueueStorageFilterOverride";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_INFO_QUEUE_DEFAULT_MESSAGE_COUNT_LIMIT: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_INTEGER_DIVIDE_BY_ZERO_QUOTIENT: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_INTEGER_DIVIDE_BY_ZERO_REMAINDER: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_KEEP_RENDER_TARGETS_AND_DEPTH_STENCIL: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_KEEP_UNORDERED_ACCESS_VIEWS: u32 = 4294967295u32;
pub const D3D11_KEY_EXCHANGE_HW_PROTECTION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2971078026, data2: 25229, data3: 19875, data4: [173, 59, 130, 221, 176, 139, 73, 112] };
pub const D3D11_KEY_EXCHANGE_RSAES_OAEP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3247741077, data2: 55082, data3: 18973, data4: [142, 93, 237, 133, 125, 23, 21, 32] };
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_LINEAR_GAMMA: f32 = 1f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MAG_FILTER_SHIFT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MAJOR_VERSION: u32 = 11u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MAX_BORDER_COLOR_COMPONENT: f32 = 1f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MAX_DEPTH: f32 = 1f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MAX_MAXANISOTROPY: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MAX_MULTISAMPLE_SAMPLE_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MAX_POSITION_VALUE: f32 = 34028236000000000000000000000000000f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MAX_TEXTURE_DIMENSION_2_TO_EXP: u32 = 17u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MINOR_VERSION: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MIN_BORDER_COLOR_COMPONENT: f32 = 0f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MIN_DEPTH: f32 = 0f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MIN_FILTER_SHIFT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MIN_MAXANISOTROPY: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MIP_FILTER_SHIFT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MIP_LOD_BIAS_MAX: f32 = 15.99f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MIP_LOD_BIAS_MIN: f32 = -16f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MIP_LOD_FRACTIONAL_BIT_COUNT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MIP_LOD_RANGE_BIT_COUNT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MULTISAMPLE_ANTIALIAS_LINE_WIDTH: f32 = 1.4f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MUTE_CATEGORY: &str = "Mute_CATEGORY_%s";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MUTE_DEBUG_OUTPUT: &str = "MuteDebugOutput";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MUTE_ID_DECIMAL: &str = "Mute_ID_%d";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MUTE_ID_STRING: &str = "Mute_ID_%s";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MUTE_SEVERITY: &str = "Mute_SEVERITY_%s";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_NONSAMPLE_FETCH_OUT_OF_RANGE_ACCESS_RESULT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PACKED_TILE: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PIXEL_ADDRESS_RANGE_BIT_COUNT: u32 = 15u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PRE_SCISSOR_PIXEL_ADDRESS_RANGE_BIT_COUNT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PS_CS_UAV_REGISTER_COMPONENTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PS_CS_UAV_REGISTER_COUNT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PS_CS_UAV_REGISTER_READS_PER_INST: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PS_CS_UAV_REGISTER_READ_PORTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PS_FRONTFACING_DEFAULT_VALUE: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PS_FRONTFACING_FALSE_VALUE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PS_FRONTFACING_TRUE_VALUE: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PS_INPUT_REGISTER_COMPONENTS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PS_INPUT_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PS_INPUT_REGISTER_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PS_INPUT_REGISTER_READS_PER_INST: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PS_INPUT_REGISTER_READ_PORTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PS_LEGACY_PIXEL_CENTER_FRACTIONAL_COMPONENT: f32 = 0f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PS_OUTPUT_DEPTH_REGISTER_COMPONENTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PS_OUTPUT_DEPTH_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PS_OUTPUT_DEPTH_REGISTER_COUNT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PS_OUTPUT_MASK_REGISTER_COMPONENTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PS_OUTPUT_MASK_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PS_OUTPUT_MASK_REGISTER_COUNT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PS_OUTPUT_REGISTER_COMPONENTS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PS_OUTPUT_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PS_OUTPUT_REGISTER_COUNT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PS_PIXEL_CENTER_FRACTIONAL_COMPONENT: f32 = 0.5f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RAW_UAV_SRV_BYTE_ALIGNMENT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_REGKEY_PATH: &str = "Software\\Microsoft\\Direct3D";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_REQ_BLEND_OBJECT_COUNT_PER_DEVICE: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_REQ_BUFFER_RESOURCE_TEXEL_COUNT_2_TO_EXP: u32 = 27u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_REQ_CONSTANT_BUFFER_ELEMENT_COUNT: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_REQ_DEPTH_STENCIL_OBJECT_COUNT_PER_DEVICE: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_REQ_DRAWINDEXED_INDEX_COUNT_2_TO_EXP: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_REQ_DRAW_VERTEX_COUNT_2_TO_EXP: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_REQ_FILTERING_HW_ADDRESSABLE_RESOURCE_DIMENSION: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_REQ_GS_INVOCATION_32BIT_OUTPUT_COMPONENT_LIMIT: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_REQ_IMMEDIATE_CONSTANT_BUFFER_ELEMENT_COUNT: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_REQ_MAXANISOTROPY: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_REQ_MIP_LEVELS: u32 = 15u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_REQ_MULTI_ELEMENT_STRUCTURE_SIZE_IN_BYTES: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_REQ_RASTERIZER_OBJECT_COUNT_PER_DEVICE: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_REQ_RENDER_TO_BUFFER_WINDOW_WIDTH: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_REQ_RESOURCE_SIZE_IN_MEGABYTES_EXPRESSION_A_TERM: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_REQ_RESOURCE_SIZE_IN_MEGABYTES_EXPRESSION_B_TERM: f32 = 0.25f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_REQ_RESOURCE_SIZE_IN_MEGABYTES_EXPRESSION_C_TERM: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_REQ_RESOURCE_VIEW_COUNT_PER_DEVICE_2_TO_EXP: u32 = 20u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_REQ_SAMPLER_OBJECT_COUNT_PER_DEVICE: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_REQ_TEXTURE1D_ARRAY_AXIS_DIMENSION: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_REQ_TEXTURE1D_U_DIMENSION: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_REQ_TEXTURE2D_ARRAY_AXIS_DIMENSION: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_REQ_TEXTURE2D_U_OR_V_DIMENSION: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_REQ_TEXTURE3D_U_V_OR_W_DIMENSION: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_REQ_TEXTURECUBE_DIMENSION: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RESINFO_INSTRUCTION_MISSING_COMPONENT_RETVAL: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SDK_LAYERS_VERSION: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SDK_VERSION: u32 = 7u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_MAJOR_VERSION: u32 = 5u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_MAX_INSTANCES: u32 = 65535u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_MAX_INTERFACES: u32 = 253u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_MAX_INTERFACE_CALL_SITES: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_MAX_TYPES: u32 = 65535u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_MINOR_VERSION: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_TRACE_FLAG_RECORD_REGISTER_READS: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_TRACE_FLAG_RECORD_REGISTER_WRITES: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHIFT_INSTRUCTION_PAD_VALUE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHIFT_INSTRUCTION_SHIFT_VALUE_BIT_COUNT: u32 = 5u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SIMULTANEOUS_RENDER_TARGET_COUNT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SO_BUFFER_MAX_STRIDE_IN_BYTES: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SO_BUFFER_MAX_WRITE_WINDOW_IN_BYTES: u32 = 512u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SO_BUFFER_SLOT_COUNT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SO_DDI_REGISTER_INDEX_DENOTING_GAP: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SO_NO_RASTERIZED_STREAM: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SO_OUTPUT_COMPONENT_COUNT: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SO_STREAM_COUNT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SPEC_DATE_DAY: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SPEC_DATE_MONTH: u32 = 5u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SPEC_DATE_YEAR: u32 = 2011u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SPEC_VERSION: f64 = 1.07f64;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SRGB_GAMMA: f32 = 2.2f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SRGB_TO_FLOAT_DENOMINATOR_1: f32 = 12.92f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SRGB_TO_FLOAT_DENOMINATOR_2: f32 = 1.055f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SRGB_TO_FLOAT_EXPONENT: f32 = 2.4f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SRGB_TO_FLOAT_OFFSET: f32 = 0.055f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SRGB_TO_FLOAT_THRESHOLD: f32 = 0.04045f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SRGB_TO_FLOAT_TOLERANCE_IN_ULP: f32 = 0.5f32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_STANDARD_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_STANDARD_COMPONENT_BIT_COUNT_DOUBLED: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_STANDARD_MAXIMUM_ELEMENT_ALIGNMENT_BYTE_MULTIPLE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_STANDARD_PIXEL_COMPONENT_COUNT: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_STANDARD_PIXEL_ELEMENT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_STANDARD_VECTOR_SIZE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_STANDARD_VERTEX_ELEMENT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_STANDARD_VERTEX_TOTAL_COMPONENT_COUNT: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SUBPIXEL_FRACTIONAL_BIT_COUNT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SUBTEXEL_FRACTIONAL_BIT_COUNT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TESSELLATOR_MAX_EVEN_TESSELLATION_FACTOR: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TESSELLATOR_MAX_ISOLINE_DENSITY_TESSELLATION_FACTOR: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TESSELLATOR_MAX_ODD_TESSELLATION_FACTOR: u32 = 63u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TESSELLATOR_MAX_TESSELLATION_FACTOR: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TESSELLATOR_MIN_EVEN_TESSELLATION_FACTOR: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TESSELLATOR_MIN_ISOLINE_DENSITY_TESSELLATION_FACTOR: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TESSELLATOR_MIN_ODD_TESSELLATION_FACTOR: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TEXEL_ADDRESS_RANGE_BIT_COUNT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_COMPONENT_W: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_COMPONENT_X: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_COMPONENT_Y: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_COMPONENT_Z: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_MISC_GS_CUT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_MISC_GS_CUT_STREAM: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_MISC_GS_EMIT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_MISC_GS_EMIT_STREAM: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_MISC_HALT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_MISC_MESSAGE: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_MISC_PS_DISCARD: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_REGISTER_FLAGS_RELATIVE_INDEXING: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_UNBOUND_MEMORY_ACCESS_RESULT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_UNMUTE_SEVERITY_INFO: &str = "Unmute_SEVERITY_INFO";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIEWPORT_AND_SCISSORRECT_MAX_INDEX: u32 = 15u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIEWPORT_AND_SCISSORRECT_OBJECT_COUNT_PER_PIPELINE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIEWPORT_BOUNDS_MAX: u32 = 32767u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIEWPORT_BOUNDS_MIN: i32 = -32768i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VS_INPUT_REGISTER_COMPONENTS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VS_INPUT_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VS_INPUT_REGISTER_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VS_INPUT_REGISTER_READS_PER_INST: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VS_INPUT_REGISTER_READ_PORTS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VS_OUTPUT_REGISTER_COMPONENTS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VS_OUTPUT_REGISTER_COMPONENT_BIT_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VS_OUTPUT_REGISTER_COUNT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_WHQL_CONTEXT_COUNT_FOR_RESOURCE_LIMIT: u32 = 10u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_WHQL_DRAWINDEXED_INDEX_COUNT_2_TO_EXP: u32 = 25u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_WHQL_DRAW_VERTEX_COUNT_2_TO_EXP: u32 = 25u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3DCSX_DLL: &str = "d3dcsx_47.dll";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3DCSX_DLL_A: &str = "d3dcsx_47.dll";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3DCSX_DLL_W: &str = "d3dcsx_47.dll";
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3DX11_FFT_MAX_DIMENSIONS: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3DX11_FFT_MAX_PRECOMPUTE_BUFFERS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3DX11_FFT_MAX_TEMP_BUFFERS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D_RETURN_PARAMETER_INDEX: i32 = -1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D_SHADER_REQUIRES_11_1_DOUBLE_EXTENSIONS: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D_SHADER_REQUIRES_11_1_SHADER_EXTENSIONS: u32 = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D_SHADER_REQUIRES_64_UAVS: u32 = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D_SHADER_REQUIRES_DOUBLES: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D_SHADER_REQUIRES_EARLY_DEPTH_STENCIL: u32 = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D_SHADER_REQUIRES_LEVEL_9_COMPARISON_FILTERING: u32 = 128u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D_SHADER_REQUIRES_MINIMUM_PRECISION: u32 = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D_SHADER_REQUIRES_TILED_RESOURCES: u32 = 256u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D_SHADER_REQUIRES_UAVS_AT_EVERY_STAGE: u32 = 4u32;
pub const DXGI_DEBUG_D3D11: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1268330875, data2: 44089, data3: 19110, data4: [187, 11, 186, 160, 71, 132, 121, 143] };
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const _FACD3D11: u32 = 2172u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_1_CREATE_DEVICE_CONTEXT_STATE_FLAG = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_1_CREATE_DEVICE_CONTEXT_STATE_SINGLETHREADED: D3D11_1_CREATE_DEVICE_CONTEXT_STATE_FLAG = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_ASYNC_GETDATA_FLAG = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_ASYNC_GETDATA_DONOTFLUSH: D3D11_ASYNC_GETDATA_FLAG = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_AUTHENTICATED_CHANNEL_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_AUTHENTICATED_CHANNEL_D3D11: D3D11_AUTHENTICATED_CHANNEL_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_AUTHENTICATED_CHANNEL_DRIVER_SOFTWARE: D3D11_AUTHENTICATED_CHANNEL_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_AUTHENTICATED_CHANNEL_DRIVER_HARDWARE: D3D11_AUTHENTICATED_CHANNEL_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_AUTHENTICATED_PROCESS_IDENTIFIER_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PROCESSIDTYPE_UNKNOWN: D3D11_AUTHENTICATED_PROCESS_IDENTIFIER_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PROCESSIDTYPE_DWM: D3D11_AUTHENTICATED_PROCESS_IDENTIFIER_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PROCESSIDTYPE_HANDLE: D3D11_AUTHENTICATED_PROCESS_IDENTIFIER_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_BIND_FLAG = u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BIND_VERTEX_BUFFER: D3D11_BIND_FLAG = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BIND_INDEX_BUFFER: D3D11_BIND_FLAG = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BIND_CONSTANT_BUFFER: D3D11_BIND_FLAG = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BIND_SHADER_RESOURCE: D3D11_BIND_FLAG = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BIND_STREAM_OUTPUT: D3D11_BIND_FLAG = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BIND_RENDER_TARGET: D3D11_BIND_FLAG = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BIND_DEPTH_STENCIL: D3D11_BIND_FLAG = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BIND_UNORDERED_ACCESS: D3D11_BIND_FLAG = 128u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BIND_DECODER: D3D11_BIND_FLAG = 512u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BIND_VIDEO_ENCODER: D3D11_BIND_FLAG = 1024u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_BLEND = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BLEND_ZERO: D3D11_BLEND = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BLEND_ONE: D3D11_BLEND = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BLEND_SRC_COLOR: D3D11_BLEND = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BLEND_INV_SRC_COLOR: D3D11_BLEND = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BLEND_SRC_ALPHA: D3D11_BLEND = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BLEND_INV_SRC_ALPHA: D3D11_BLEND = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BLEND_DEST_ALPHA: D3D11_BLEND = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BLEND_INV_DEST_ALPHA: D3D11_BLEND = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BLEND_DEST_COLOR: D3D11_BLEND = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BLEND_INV_DEST_COLOR: D3D11_BLEND = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BLEND_SRC_ALPHA_SAT: D3D11_BLEND = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BLEND_BLEND_FACTOR: D3D11_BLEND = 14i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BLEND_INV_BLEND_FACTOR: D3D11_BLEND = 15i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BLEND_SRC1_COLOR: D3D11_BLEND = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BLEND_INV_SRC1_COLOR: D3D11_BLEND = 17i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BLEND_SRC1_ALPHA: D3D11_BLEND = 18i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BLEND_INV_SRC1_ALPHA: D3D11_BLEND = 19i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_BLEND_OP = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BLEND_OP_ADD: D3D11_BLEND_OP = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BLEND_OP_SUBTRACT: D3D11_BLEND_OP = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BLEND_OP_REV_SUBTRACT: D3D11_BLEND_OP = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BLEND_OP_MIN: D3D11_BLEND_OP = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BLEND_OP_MAX: D3D11_BLEND_OP = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_BUFFEREX_SRV_FLAG = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BUFFEREX_SRV_FLAG_RAW: D3D11_BUFFEREX_SRV_FLAG = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_BUFFER_UAV_FLAG = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BUFFER_UAV_FLAG_RAW: D3D11_BUFFER_UAV_FLAG = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BUFFER_UAV_FLAG_APPEND: D3D11_BUFFER_UAV_FLAG = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BUFFER_UAV_FLAG_COUNTER: D3D11_BUFFER_UAV_FLAG = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_BUS_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BUS_TYPE_OTHER: D3D11_BUS_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BUS_TYPE_PCI: D3D11_BUS_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BUS_TYPE_PCIX: D3D11_BUS_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BUS_TYPE_PCIEXPRESS: D3D11_BUS_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BUS_TYPE_AGP: D3D11_BUS_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BUS_IMPL_MODIFIER_INSIDE_OF_CHIPSET: D3D11_BUS_TYPE = 65536i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BUS_IMPL_MODIFIER_TRACKS_ON_MOTHER_BOARD_TO_CHIP: D3D11_BUS_TYPE = 131072i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BUS_IMPL_MODIFIER_TRACKS_ON_MOTHER_BOARD_TO_SOCKET: D3D11_BUS_TYPE = 196608i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BUS_IMPL_MODIFIER_DAUGHTER_BOARD_CONNECTOR: D3D11_BUS_TYPE = 262144i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BUS_IMPL_MODIFIER_DAUGHTER_BOARD_CONNECTOR_INSIDE_OF_NUAE: D3D11_BUS_TYPE = 327680i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_BUS_IMPL_MODIFIER_NON_STANDARD: D3D11_BUS_TYPE = -2147483648i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_CHECK_MULTISAMPLE_QUALITY_LEVELS_FLAG = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CHECK_MULTISAMPLE_QUALITY_LEVELS_TILED_RESOURCE: D3D11_CHECK_MULTISAMPLE_QUALITY_LEVELS_FLAG = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_CLEAR_FLAG = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CLEAR_DEPTH: D3D11_CLEAR_FLAG = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CLEAR_STENCIL: D3D11_CLEAR_FLAG = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_COLOR_WRITE_ENABLE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COLOR_WRITE_ENABLE_RED: D3D11_COLOR_WRITE_ENABLE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COLOR_WRITE_ENABLE_GREEN: D3D11_COLOR_WRITE_ENABLE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COLOR_WRITE_ENABLE_BLUE: D3D11_COLOR_WRITE_ENABLE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COLOR_WRITE_ENABLE_ALPHA: D3D11_COLOR_WRITE_ENABLE = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COLOR_WRITE_ENABLE_ALL: D3D11_COLOR_WRITE_ENABLE = 15i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_COMPARISON_FUNC = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMPARISON_NEVER: D3D11_COMPARISON_FUNC = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMPARISON_LESS: D3D11_COMPARISON_FUNC = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMPARISON_EQUAL: D3D11_COMPARISON_FUNC = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMPARISON_LESS_EQUAL: D3D11_COMPARISON_FUNC = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMPARISON_GREATER: D3D11_COMPARISON_FUNC = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMPARISON_NOT_EQUAL: D3D11_COMPARISON_FUNC = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMPARISON_GREATER_EQUAL: D3D11_COMPARISON_FUNC = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMPARISON_ALWAYS: D3D11_COMPARISON_FUNC = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_CONSERVATIVE_RASTERIZATION_MODE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CONSERVATIVE_RASTERIZATION_MODE_OFF: D3D11_CONSERVATIVE_RASTERIZATION_MODE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CONSERVATIVE_RASTERIZATION_MODE_ON: D3D11_CONSERVATIVE_RASTERIZATION_MODE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_CONSERVATIVE_RASTERIZATION_TIER = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CONSERVATIVE_RASTERIZATION_NOT_SUPPORTED: D3D11_CONSERVATIVE_RASTERIZATION_TIER = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CONSERVATIVE_RASTERIZATION_TIER_1: D3D11_CONSERVATIVE_RASTERIZATION_TIER = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CONSERVATIVE_RASTERIZATION_TIER_2: D3D11_CONSERVATIVE_RASTERIZATION_TIER = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CONSERVATIVE_RASTERIZATION_TIER_3: D3D11_CONSERVATIVE_RASTERIZATION_TIER = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_CONTENT_PROTECTION_CAPS = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CONTENT_PROTECTION_CAPS_SOFTWARE: D3D11_CONTENT_PROTECTION_CAPS = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CONTENT_PROTECTION_CAPS_HARDWARE: D3D11_CONTENT_PROTECTION_CAPS = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CONTENT_PROTECTION_CAPS_PROTECTION_ALWAYS_ON: D3D11_CONTENT_PROTECTION_CAPS = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CONTENT_PROTECTION_CAPS_PARTIAL_DECRYPTION: D3D11_CONTENT_PROTECTION_CAPS = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CONTENT_PROTECTION_CAPS_CONTENT_KEY: D3D11_CONTENT_PROTECTION_CAPS = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CONTENT_PROTECTION_CAPS_FRESHEN_SESSION_KEY: D3D11_CONTENT_PROTECTION_CAPS = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CONTENT_PROTECTION_CAPS_ENCRYPTED_READ_BACK: D3D11_CONTENT_PROTECTION_CAPS = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CONTENT_PROTECTION_CAPS_ENCRYPTED_READ_BACK_KEY: D3D11_CONTENT_PROTECTION_CAPS = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CONTENT_PROTECTION_CAPS_SEQUENTIAL_CTR_IV: D3D11_CONTENT_PROTECTION_CAPS = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CONTENT_PROTECTION_CAPS_ENCRYPT_SLICEDATA_ONLY: D3D11_CONTENT_PROTECTION_CAPS = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CONTENT_PROTECTION_CAPS_DECRYPTION_BLT: D3D11_CONTENT_PROTECTION_CAPS = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CONTENT_PROTECTION_CAPS_HARDWARE_PROTECT_UNCOMPRESSED: D3D11_CONTENT_PROTECTION_CAPS = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CONTENT_PROTECTION_CAPS_HARDWARE_PROTECTED_MEMORY_PAGEABLE: D3D11_CONTENT_PROTECTION_CAPS = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CONTENT_PROTECTION_CAPS_HARDWARE_TEARDOWN: D3D11_CONTENT_PROTECTION_CAPS = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CONTENT_PROTECTION_CAPS_HARDWARE_DRM_COMMUNICATION: D3D11_CONTENT_PROTECTION_CAPS = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CONTENT_PROTECTION_CAPS_HARDWARE_DRM_COMMUNICATION_MULTI_THREADED: D3D11_CONTENT_PROTECTION_CAPS = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_CONTEXT_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CONTEXT_TYPE_ALL: D3D11_CONTEXT_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CONTEXT_TYPE_3D: D3D11_CONTEXT_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CONTEXT_TYPE_COMPUTE: D3D11_CONTEXT_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CONTEXT_TYPE_COPY: D3D11_CONTEXT_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CONTEXT_TYPE_VIDEO: D3D11_CONTEXT_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_COPY_FLAGS = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COPY_NO_OVERWRITE: D3D11_COPY_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COPY_DISCARD: D3D11_COPY_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_COUNTER = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COUNTER_DEVICE_DEPENDENT_0: D3D11_COUNTER = 1073741824i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_COUNTER_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COUNTER_TYPE_FLOAT32: D3D11_COUNTER_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COUNTER_TYPE_UINT16: D3D11_COUNTER_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COUNTER_TYPE_UINT32: D3D11_COUNTER_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COUNTER_TYPE_UINT64: D3D11_COUNTER_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_CPU_ACCESS_FLAG = u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CPU_ACCESS_WRITE: D3D11_CPU_ACCESS_FLAG = 65536u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CPU_ACCESS_READ: D3D11_CPU_ACCESS_FLAG = 131072u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_CREATE_DEVICE_FLAG = u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CREATE_DEVICE_SINGLETHREADED: D3D11_CREATE_DEVICE_FLAG = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CREATE_DEVICE_DEBUG: D3D11_CREATE_DEVICE_FLAG = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CREATE_DEVICE_SWITCH_TO_REF: D3D11_CREATE_DEVICE_FLAG = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CREATE_DEVICE_PREVENT_INTERNAL_THREADING_OPTIMIZATIONS: D3D11_CREATE_DEVICE_FLAG = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CREATE_DEVICE_BGRA_SUPPORT: D3D11_CREATE_DEVICE_FLAG = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CREATE_DEVICE_DEBUGGABLE: D3D11_CREATE_DEVICE_FLAG = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CREATE_DEVICE_PREVENT_ALTERING_LAYER_SETTINGS_FROM_REGISTRY: D3D11_CREATE_DEVICE_FLAG = 128u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CREATE_DEVICE_DISABLE_GPU_TIMEOUT: D3D11_CREATE_DEVICE_FLAG = 256u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CREATE_DEVICE_VIDEO_SUPPORT: D3D11_CREATE_DEVICE_FLAG = 2048u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_CRYPTO_SESSION_KEY_EXCHANGE_FLAGS = u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CRYPTO_SESSION_KEY_EXCHANGE_FLAG_NONE: D3D11_CRYPTO_SESSION_KEY_EXCHANGE_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_CRYPTO_SESSION_STATUS = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CRYPTO_SESSION_STATUS_OK: D3D11_CRYPTO_SESSION_STATUS = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CRYPTO_SESSION_STATUS_KEY_LOST: D3D11_CRYPTO_SESSION_STATUS = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CRYPTO_SESSION_STATUS_KEY_AND_CONTENT_LOST: D3D11_CRYPTO_SESSION_STATUS = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_CULL_MODE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CULL_NONE: D3D11_CULL_MODE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CULL_FRONT: D3D11_CULL_MODE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CULL_BACK: D3D11_CULL_MODE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_DEPTH_WRITE_MASK = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEPTH_WRITE_MASK_ZERO: D3D11_DEPTH_WRITE_MASK = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEPTH_WRITE_MASK_ALL: D3D11_DEPTH_WRITE_MASK = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_DEVICE_CONTEXT_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEVICE_CONTEXT_IMMEDIATE: D3D11_DEVICE_CONTEXT_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DEVICE_CONTEXT_DEFERRED: D3D11_DEVICE_CONTEXT_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_DSV_DIMENSION = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DSV_DIMENSION_UNKNOWN: D3D11_DSV_DIMENSION = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DSV_DIMENSION_TEXTURE1D: D3D11_DSV_DIMENSION = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DSV_DIMENSION_TEXTURE1DARRAY: D3D11_DSV_DIMENSION = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DSV_DIMENSION_TEXTURE2D: D3D11_DSV_DIMENSION = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DSV_DIMENSION_TEXTURE2DARRAY: D3D11_DSV_DIMENSION = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DSV_DIMENSION_TEXTURE2DMS: D3D11_DSV_DIMENSION = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DSV_DIMENSION_TEXTURE2DMSARRAY: D3D11_DSV_DIMENSION = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_DSV_FLAG = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DSV_READ_ONLY_DEPTH: D3D11_DSV_FLAG = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DSV_READ_ONLY_STENCIL: D3D11_DSV_FLAG = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_FEATURE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FEATURE_THREADING: D3D11_FEATURE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FEATURE_DOUBLES: D3D11_FEATURE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FEATURE_FORMAT_SUPPORT: D3D11_FEATURE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FEATURE_FORMAT_SUPPORT2: D3D11_FEATURE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FEATURE_D3D10_X_HARDWARE_OPTIONS: D3D11_FEATURE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FEATURE_D3D11_OPTIONS: D3D11_FEATURE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FEATURE_ARCHITECTURE_INFO: D3D11_FEATURE = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FEATURE_D3D9_OPTIONS: D3D11_FEATURE = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FEATURE_SHADER_MIN_PRECISION_SUPPORT: D3D11_FEATURE = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FEATURE_D3D9_SHADOW_SUPPORT: D3D11_FEATURE = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FEATURE_D3D11_OPTIONS1: D3D11_FEATURE = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FEATURE_D3D9_SIMPLE_INSTANCING_SUPPORT: D3D11_FEATURE = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FEATURE_MARKER_SUPPORT: D3D11_FEATURE = 12i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FEATURE_D3D9_OPTIONS1: D3D11_FEATURE = 13i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FEATURE_D3D11_OPTIONS2: D3D11_FEATURE = 14i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FEATURE_D3D11_OPTIONS3: D3D11_FEATURE = 15i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FEATURE_GPU_VIRTUAL_ADDRESS_SUPPORT: D3D11_FEATURE = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FEATURE_D3D11_OPTIONS4: D3D11_FEATURE = 17i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FEATURE_SHADER_CACHE: D3D11_FEATURE = 18i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FEATURE_D3D11_OPTIONS5: D3D11_FEATURE = 19i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FEATURE_DISPLAYABLE: D3D11_FEATURE = 20i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_FEATURE_VIDEO = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FEATURE_VIDEO_DECODER_HISTOGRAM: D3D11_FEATURE_VIDEO = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_FENCE_FLAG = u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FENCE_FLAG_NONE: D3D11_FENCE_FLAG = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FENCE_FLAG_SHARED: D3D11_FENCE_FLAG = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FENCE_FLAG_SHARED_CROSS_ADAPTER: D3D11_FENCE_FLAG = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FENCE_FLAG_NON_MONITORED: D3D11_FENCE_FLAG = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_FILL_MODE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILL_WIREFRAME: D3D11_FILL_MODE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILL_SOLID: D3D11_FILL_MODE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_FILTER = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_MIN_MAG_MIP_POINT: D3D11_FILTER = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_MIN_MAG_POINT_MIP_LINEAR: D3D11_FILTER = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_MIN_POINT_MAG_LINEAR_MIP_POINT: D3D11_FILTER = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_MIN_POINT_MAG_MIP_LINEAR: D3D11_FILTER = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_MIN_LINEAR_MAG_MIP_POINT: D3D11_FILTER = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_MIN_LINEAR_MAG_POINT_MIP_LINEAR: D3D11_FILTER = 17i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_MIN_MAG_LINEAR_MIP_POINT: D3D11_FILTER = 20i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_MIN_MAG_MIP_LINEAR: D3D11_FILTER = 21i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_ANISOTROPIC: D3D11_FILTER = 85i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_COMPARISON_MIN_MAG_MIP_POINT: D3D11_FILTER = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_COMPARISON_MIN_MAG_POINT_MIP_LINEAR: D3D11_FILTER = 129i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_COMPARISON_MIN_POINT_MAG_LINEAR_MIP_POINT: D3D11_FILTER = 132i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_COMPARISON_MIN_POINT_MAG_MIP_LINEAR: D3D11_FILTER = 133i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_COMPARISON_MIN_LINEAR_MAG_MIP_POINT: D3D11_FILTER = 144i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_COMPARISON_MIN_LINEAR_MAG_POINT_MIP_LINEAR: D3D11_FILTER = 145i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_COMPARISON_MIN_MAG_LINEAR_MIP_POINT: D3D11_FILTER = 148i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_COMPARISON_MIN_MAG_MIP_LINEAR: D3D11_FILTER = 149i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_COMPARISON_ANISOTROPIC: D3D11_FILTER = 213i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_MINIMUM_MIN_MAG_MIP_POINT: D3D11_FILTER = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_MINIMUM_MIN_MAG_POINT_MIP_LINEAR: D3D11_FILTER = 257i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_MINIMUM_MIN_POINT_MAG_LINEAR_MIP_POINT: D3D11_FILTER = 260i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_MINIMUM_MIN_POINT_MAG_MIP_LINEAR: D3D11_FILTER = 261i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_MINIMUM_MIN_LINEAR_MAG_MIP_POINT: D3D11_FILTER = 272i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_MINIMUM_MIN_LINEAR_MAG_POINT_MIP_LINEAR: D3D11_FILTER = 273i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_MINIMUM_MIN_MAG_LINEAR_MIP_POINT: D3D11_FILTER = 276i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_MINIMUM_MIN_MAG_MIP_LINEAR: D3D11_FILTER = 277i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_MINIMUM_ANISOTROPIC: D3D11_FILTER = 341i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_MAXIMUM_MIN_MAG_MIP_POINT: D3D11_FILTER = 384i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_MAXIMUM_MIN_MAG_POINT_MIP_LINEAR: D3D11_FILTER = 385i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_MAXIMUM_MIN_POINT_MAG_LINEAR_MIP_POINT: D3D11_FILTER = 388i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_MAXIMUM_MIN_POINT_MAG_MIP_LINEAR: D3D11_FILTER = 389i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_MAXIMUM_MIN_LINEAR_MAG_MIP_POINT: D3D11_FILTER = 400i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_MAXIMUM_MIN_LINEAR_MAG_POINT_MIP_LINEAR: D3D11_FILTER = 401i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_MAXIMUM_MIN_MAG_LINEAR_MIP_POINT: D3D11_FILTER = 404i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_MAXIMUM_MIN_MAG_MIP_LINEAR: D3D11_FILTER = 405i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_MAXIMUM_ANISOTROPIC: D3D11_FILTER = 469i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_FILTER_REDUCTION_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_REDUCTION_TYPE_STANDARD: D3D11_FILTER_REDUCTION_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_REDUCTION_TYPE_COMPARISON: D3D11_FILTER_REDUCTION_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_REDUCTION_TYPE_MINIMUM: D3D11_FILTER_REDUCTION_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_REDUCTION_TYPE_MAXIMUM: D3D11_FILTER_REDUCTION_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_FILTER_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_TYPE_POINT: D3D11_FILTER_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FILTER_TYPE_LINEAR: D3D11_FILTER_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_FORMAT_SUPPORT = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_BUFFER: D3D11_FORMAT_SUPPORT = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_IA_VERTEX_BUFFER: D3D11_FORMAT_SUPPORT = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_IA_INDEX_BUFFER: D3D11_FORMAT_SUPPORT = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_SO_BUFFER: D3D11_FORMAT_SUPPORT = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_TEXTURE1D: D3D11_FORMAT_SUPPORT = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_TEXTURE2D: D3D11_FORMAT_SUPPORT = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_TEXTURE3D: D3D11_FORMAT_SUPPORT = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_TEXTURECUBE: D3D11_FORMAT_SUPPORT = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_SHADER_LOAD: D3D11_FORMAT_SUPPORT = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_SHADER_SAMPLE: D3D11_FORMAT_SUPPORT = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_SHADER_SAMPLE_COMPARISON: D3D11_FORMAT_SUPPORT = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_SHADER_SAMPLE_MONO_TEXT: D3D11_FORMAT_SUPPORT = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_MIP: D3D11_FORMAT_SUPPORT = 4096i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_MIP_AUTOGEN: D3D11_FORMAT_SUPPORT = 8192i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_RENDER_TARGET: D3D11_FORMAT_SUPPORT = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_BLENDABLE: D3D11_FORMAT_SUPPORT = 32768i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_DEPTH_STENCIL: D3D11_FORMAT_SUPPORT = 65536i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_CPU_LOCKABLE: D3D11_FORMAT_SUPPORT = 131072i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_MULTISAMPLE_RESOLVE: D3D11_FORMAT_SUPPORT = 262144i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_DISPLAY: D3D11_FORMAT_SUPPORT = 524288i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_CAST_WITHIN_BIT_LAYOUT: D3D11_FORMAT_SUPPORT = 1048576i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_MULTISAMPLE_RENDERTARGET: D3D11_FORMAT_SUPPORT = 2097152i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_MULTISAMPLE_LOAD: D3D11_FORMAT_SUPPORT = 4194304i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_SHADER_GATHER: D3D11_FORMAT_SUPPORT = 8388608i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_BACK_BUFFER_CAST: D3D11_FORMAT_SUPPORT = 16777216i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_TYPED_UNORDERED_ACCESS_VIEW: D3D11_FORMAT_SUPPORT = 33554432i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_SHADER_GATHER_COMPARISON: D3D11_FORMAT_SUPPORT = 67108864i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_DECODER_OUTPUT: D3D11_FORMAT_SUPPORT = 134217728i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_VIDEO_PROCESSOR_OUTPUT: D3D11_FORMAT_SUPPORT = 268435456i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_VIDEO_PROCESSOR_INPUT: D3D11_FORMAT_SUPPORT = 536870912i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT_VIDEO_ENCODER: D3D11_FORMAT_SUPPORT = 1073741824i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_FORMAT_SUPPORT2 = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT2_UAV_ATOMIC_ADD: D3D11_FORMAT_SUPPORT2 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT2_UAV_ATOMIC_BITWISE_OPS: D3D11_FORMAT_SUPPORT2 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT2_UAV_ATOMIC_COMPARE_STORE_OR_COMPARE_EXCHANGE: D3D11_FORMAT_SUPPORT2 = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT2_UAV_ATOMIC_EXCHANGE: D3D11_FORMAT_SUPPORT2 = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT2_UAV_ATOMIC_SIGNED_MIN_OR_MAX: D3D11_FORMAT_SUPPORT2 = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT2_UAV_ATOMIC_UNSIGNED_MIN_OR_MAX: D3D11_FORMAT_SUPPORT2 = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT2_UAV_TYPED_LOAD: D3D11_FORMAT_SUPPORT2 = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT2_UAV_TYPED_STORE: D3D11_FORMAT_SUPPORT2 = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT2_OUTPUT_MERGER_LOGIC_OP: D3D11_FORMAT_SUPPORT2 = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT2_TILED: D3D11_FORMAT_SUPPORT2 = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT2_SHAREABLE: D3D11_FORMAT_SUPPORT2 = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_FORMAT_SUPPORT2_MULTIPLANE_OVERLAY: D3D11_FORMAT_SUPPORT2 = 16384i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_INPUT_CLASSIFICATION = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_INPUT_PER_VERTEX_DATA: D3D11_INPUT_CLASSIFICATION = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_INPUT_PER_INSTANCE_DATA: D3D11_INPUT_CLASSIFICATION = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_LOGIC_OP = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_LOGIC_OP_CLEAR: D3D11_LOGIC_OP = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_LOGIC_OP_SET: D3D11_LOGIC_OP = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_LOGIC_OP_COPY: D3D11_LOGIC_OP = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_LOGIC_OP_COPY_INVERTED: D3D11_LOGIC_OP = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_LOGIC_OP_NOOP: D3D11_LOGIC_OP = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_LOGIC_OP_INVERT: D3D11_LOGIC_OP = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_LOGIC_OP_AND: D3D11_LOGIC_OP = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_LOGIC_OP_NAND: D3D11_LOGIC_OP = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_LOGIC_OP_OR: D3D11_LOGIC_OP = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_LOGIC_OP_NOR: D3D11_LOGIC_OP = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_LOGIC_OP_XOR: D3D11_LOGIC_OP = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_LOGIC_OP_EQUIV: D3D11_LOGIC_OP = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_LOGIC_OP_AND_REVERSE: D3D11_LOGIC_OP = 12i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_LOGIC_OP_AND_INVERTED: D3D11_LOGIC_OP = 13i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_LOGIC_OP_OR_REVERSE: D3D11_LOGIC_OP = 14i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_LOGIC_OP_OR_INVERTED: D3D11_LOGIC_OP = 15i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_MAP = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MAP_READ: D3D11_MAP = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MAP_WRITE: D3D11_MAP = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MAP_READ_WRITE: D3D11_MAP = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MAP_WRITE_DISCARD: D3D11_MAP = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MAP_WRITE_NO_OVERWRITE: D3D11_MAP = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_MAP_FLAG = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MAP_FLAG_DO_NOT_WAIT: D3D11_MAP_FLAG = 1048576i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_MESSAGE_CATEGORY = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_CATEGORY_APPLICATION_DEFINED: D3D11_MESSAGE_CATEGORY = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_CATEGORY_MISCELLANEOUS: D3D11_MESSAGE_CATEGORY = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_CATEGORY_INITIALIZATION: D3D11_MESSAGE_CATEGORY = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_CATEGORY_CLEANUP: D3D11_MESSAGE_CATEGORY = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_CATEGORY_COMPILATION: D3D11_MESSAGE_CATEGORY = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_CATEGORY_STATE_CREATION: D3D11_MESSAGE_CATEGORY = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_CATEGORY_STATE_SETTING: D3D11_MESSAGE_CATEGORY = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_CATEGORY_STATE_GETTING: D3D11_MESSAGE_CATEGORY = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_CATEGORY_RESOURCE_MANIPULATION: D3D11_MESSAGE_CATEGORY = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_CATEGORY_EXECUTION: D3D11_MESSAGE_CATEGORY = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_CATEGORY_SHADER: D3D11_MESSAGE_CATEGORY = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_MESSAGE_ID = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_UNKNOWN: D3D11_MESSAGE_ID = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_IASETVERTEXBUFFERS_HAZARD: D3D11_MESSAGE_ID = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_IASETINDEXBUFFER_HAZARD: D3D11_MESSAGE_ID = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_VSSETSHADERRESOURCES_HAZARD: D3D11_MESSAGE_ID = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_VSSETCONSTANTBUFFERS_HAZARD: D3D11_MESSAGE_ID = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_GSSETSHADERRESOURCES_HAZARD: D3D11_MESSAGE_ID = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_GSSETCONSTANTBUFFERS_HAZARD: D3D11_MESSAGE_ID = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_PSSETSHADERRESOURCES_HAZARD: D3D11_MESSAGE_ID = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_PSSETCONSTANTBUFFERS_HAZARD: D3D11_MESSAGE_ID = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_OMSETRENDERTARGETS_HAZARD: D3D11_MESSAGE_ID = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_SOSETTARGETS_HAZARD: D3D11_MESSAGE_ID = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_STRING_FROM_APPLICATION: D3D11_MESSAGE_ID = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CORRUPTED_THIS: D3D11_MESSAGE_ID = 12i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CORRUPTED_PARAMETER1: D3D11_MESSAGE_ID = 13i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CORRUPTED_PARAMETER2: D3D11_MESSAGE_ID = 14i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CORRUPTED_PARAMETER3: D3D11_MESSAGE_ID = 15i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CORRUPTED_PARAMETER4: D3D11_MESSAGE_ID = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CORRUPTED_PARAMETER5: D3D11_MESSAGE_ID = 17i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CORRUPTED_PARAMETER6: D3D11_MESSAGE_ID = 18i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CORRUPTED_PARAMETER7: D3D11_MESSAGE_ID = 19i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CORRUPTED_PARAMETER8: D3D11_MESSAGE_ID = 20i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CORRUPTED_PARAMETER9: D3D11_MESSAGE_ID = 21i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CORRUPTED_PARAMETER10: D3D11_MESSAGE_ID = 22i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CORRUPTED_PARAMETER11: D3D11_MESSAGE_ID = 23i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CORRUPTED_PARAMETER12: D3D11_MESSAGE_ID = 24i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CORRUPTED_PARAMETER13: D3D11_MESSAGE_ID = 25i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CORRUPTED_PARAMETER14: D3D11_MESSAGE_ID = 26i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CORRUPTED_PARAMETER15: D3D11_MESSAGE_ID = 27i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CORRUPTED_MULTITHREADING: D3D11_MESSAGE_ID = 28i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_MESSAGE_REPORTING_OUTOFMEMORY: D3D11_MESSAGE_ID = 29i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_IASETINPUTLAYOUT_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 30i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_IASETVERTEXBUFFERS_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 31i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_IASETINDEXBUFFER_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VSSETSHADER_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 33i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VSSETSHADERRESOURCES_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 34i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VSSETCONSTANTBUFFERS_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 35i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VSSETSAMPLERS_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 36i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GSSETSHADER_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 37i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GSSETSHADERRESOURCES_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 38i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GSSETCONSTANTBUFFERS_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 39i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GSSETSAMPLERS_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 40i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_SOSETTARGETS_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 41i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_PSSETSHADER_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 42i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_PSSETSHADERRESOURCES_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 43i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_PSSETCONSTANTBUFFERS_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 44i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_PSSETSAMPLERS_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 45i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_RSSETSTATE_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 46i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_OMSETBLENDSTATE_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 47i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_OMSETDEPTHSTENCILSTATE_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 48i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_OMSETRENDERTARGETS_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 49i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_SETPREDICATION_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 50i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETPRIVATEDATA_MOREDATA: D3D11_MESSAGE_ID = 51i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_SETPRIVATEDATA_INVALIDFREEDATA: D3D11_MESSAGE_ID = 52i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_SETPRIVATEDATA_INVALIDIUNKNOWN: D3D11_MESSAGE_ID = 53i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_SETPRIVATEDATA_INVALIDFLAGS: D3D11_MESSAGE_ID = 54i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_SETPRIVATEDATA_CHANGINGPARAMS: D3D11_MESSAGE_ID = 55i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_SETPRIVATEDATA_OUTOFMEMORY: D3D11_MESSAGE_ID = 56i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBUFFER_UNRECOGNIZEDFORMAT: D3D11_MESSAGE_ID = 57i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBUFFER_INVALIDSAMPLES: D3D11_MESSAGE_ID = 58i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBUFFER_UNRECOGNIZEDUSAGE: D3D11_MESSAGE_ID = 59i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBUFFER_UNRECOGNIZEDBINDFLAGS: D3D11_MESSAGE_ID = 60i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBUFFER_UNRECOGNIZEDCPUACCESSFLAGS: D3D11_MESSAGE_ID = 61i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBUFFER_UNRECOGNIZEDMISCFLAGS: D3D11_MESSAGE_ID = 62i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBUFFER_INVALIDCPUACCESSFLAGS: D3D11_MESSAGE_ID = 63i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBUFFER_INVALIDBINDFLAGS: D3D11_MESSAGE_ID = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBUFFER_INVALIDINITIALDATA: D3D11_MESSAGE_ID = 65i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBUFFER_INVALIDDIMENSIONS: D3D11_MESSAGE_ID = 66i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBUFFER_INVALIDMIPLEVELS: D3D11_MESSAGE_ID = 67i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBUFFER_INVALIDMISCFLAGS: D3D11_MESSAGE_ID = 68i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBUFFER_INVALIDARG_RETURN: D3D11_MESSAGE_ID = 69i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBUFFER_OUTOFMEMORY_RETURN: D3D11_MESSAGE_ID = 70i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBUFFER_NULLDESC: D3D11_MESSAGE_ID = 71i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBUFFER_INVALIDCONSTANTBUFFERBINDINGS: D3D11_MESSAGE_ID = 72i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBUFFER_LARGEALLOCATION: D3D11_MESSAGE_ID = 73i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE1D_UNRECOGNIZEDFORMAT: D3D11_MESSAGE_ID = 74i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE1D_UNSUPPORTEDFORMAT: D3D11_MESSAGE_ID = 75i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE1D_INVALIDSAMPLES: D3D11_MESSAGE_ID = 76i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE1D_UNRECOGNIZEDUSAGE: D3D11_MESSAGE_ID = 77i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE1D_UNRECOGNIZEDBINDFLAGS: D3D11_MESSAGE_ID = 78i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE1D_UNRECOGNIZEDCPUACCESSFLAGS: D3D11_MESSAGE_ID = 79i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE1D_UNRECOGNIZEDMISCFLAGS: D3D11_MESSAGE_ID = 80i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE1D_INVALIDCPUACCESSFLAGS: D3D11_MESSAGE_ID = 81i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE1D_INVALIDBINDFLAGS: D3D11_MESSAGE_ID = 82i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE1D_INVALIDINITIALDATA: D3D11_MESSAGE_ID = 83i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE1D_INVALIDDIMENSIONS: D3D11_MESSAGE_ID = 84i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE1D_INVALIDMIPLEVELS: D3D11_MESSAGE_ID = 85i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE1D_INVALIDMISCFLAGS: D3D11_MESSAGE_ID = 86i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE1D_INVALIDARG_RETURN: D3D11_MESSAGE_ID = 87i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE1D_OUTOFMEMORY_RETURN: D3D11_MESSAGE_ID = 88i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE1D_NULLDESC: D3D11_MESSAGE_ID = 89i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE1D_LARGEALLOCATION: D3D11_MESSAGE_ID = 90i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE2D_UNRECOGNIZEDFORMAT: D3D11_MESSAGE_ID = 91i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE2D_UNSUPPORTEDFORMAT: D3D11_MESSAGE_ID = 92i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE2D_INVALIDSAMPLES: D3D11_MESSAGE_ID = 93i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE2D_UNRECOGNIZEDUSAGE: D3D11_MESSAGE_ID = 94i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE2D_UNRECOGNIZEDBINDFLAGS: D3D11_MESSAGE_ID = 95i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE2D_UNRECOGNIZEDCPUACCESSFLAGS: D3D11_MESSAGE_ID = 96i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE2D_UNRECOGNIZEDMISCFLAGS: D3D11_MESSAGE_ID = 97i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE2D_INVALIDCPUACCESSFLAGS: D3D11_MESSAGE_ID = 98i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE2D_INVALIDBINDFLAGS: D3D11_MESSAGE_ID = 99i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE2D_INVALIDINITIALDATA: D3D11_MESSAGE_ID = 100i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE2D_INVALIDDIMENSIONS: D3D11_MESSAGE_ID = 101i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE2D_INVALIDMIPLEVELS: D3D11_MESSAGE_ID = 102i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE2D_INVALIDMISCFLAGS: D3D11_MESSAGE_ID = 103i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE2D_INVALIDARG_RETURN: D3D11_MESSAGE_ID = 104i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE2D_OUTOFMEMORY_RETURN: D3D11_MESSAGE_ID = 105i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE2D_NULLDESC: D3D11_MESSAGE_ID = 106i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE2D_LARGEALLOCATION: D3D11_MESSAGE_ID = 107i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE3D_UNRECOGNIZEDFORMAT: D3D11_MESSAGE_ID = 108i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE3D_UNSUPPORTEDFORMAT: D3D11_MESSAGE_ID = 109i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE3D_INVALIDSAMPLES: D3D11_MESSAGE_ID = 110i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE3D_UNRECOGNIZEDUSAGE: D3D11_MESSAGE_ID = 111i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE3D_UNRECOGNIZEDBINDFLAGS: D3D11_MESSAGE_ID = 112i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE3D_UNRECOGNIZEDCPUACCESSFLAGS: D3D11_MESSAGE_ID = 113i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE3D_UNRECOGNIZEDMISCFLAGS: D3D11_MESSAGE_ID = 114i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE3D_INVALIDCPUACCESSFLAGS: D3D11_MESSAGE_ID = 115i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE3D_INVALIDBINDFLAGS: D3D11_MESSAGE_ID = 116i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE3D_INVALIDINITIALDATA: D3D11_MESSAGE_ID = 117i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE3D_INVALIDDIMENSIONS: D3D11_MESSAGE_ID = 118i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE3D_INVALIDMIPLEVELS: D3D11_MESSAGE_ID = 119i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE3D_INVALIDMISCFLAGS: D3D11_MESSAGE_ID = 120i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE3D_INVALIDARG_RETURN: D3D11_MESSAGE_ID = 121i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE3D_OUTOFMEMORY_RETURN: D3D11_MESSAGE_ID = 122i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE3D_NULLDESC: D3D11_MESSAGE_ID = 123i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE3D_LARGEALLOCATION: D3D11_MESSAGE_ID = 124i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESHADERRESOURCEVIEW_UNRECOGNIZEDFORMAT: D3D11_MESSAGE_ID = 125i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESHADERRESOURCEVIEW_INVALIDDESC: D3D11_MESSAGE_ID = 126i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESHADERRESOURCEVIEW_INVALIDFORMAT: D3D11_MESSAGE_ID = 127i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESHADERRESOURCEVIEW_INVALIDDIMENSIONS: D3D11_MESSAGE_ID = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESHADERRESOURCEVIEW_INVALIDRESOURCE: D3D11_MESSAGE_ID = 129i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESHADERRESOURCEVIEW_TOOMANYOBJECTS: D3D11_MESSAGE_ID = 130i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESHADERRESOURCEVIEW_INVALIDARG_RETURN: D3D11_MESSAGE_ID = 131i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESHADERRESOURCEVIEW_OUTOFMEMORY_RETURN: D3D11_MESSAGE_ID = 132i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERENDERTARGETVIEW_UNRECOGNIZEDFORMAT: D3D11_MESSAGE_ID = 133i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERENDERTARGETVIEW_UNSUPPORTEDFORMAT: D3D11_MESSAGE_ID = 134i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERENDERTARGETVIEW_INVALIDDESC: D3D11_MESSAGE_ID = 135i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERENDERTARGETVIEW_INVALIDFORMAT: D3D11_MESSAGE_ID = 136i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERENDERTARGETVIEW_INVALIDDIMENSIONS: D3D11_MESSAGE_ID = 137i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERENDERTARGETVIEW_INVALIDRESOURCE: D3D11_MESSAGE_ID = 138i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERENDERTARGETVIEW_TOOMANYOBJECTS: D3D11_MESSAGE_ID = 139i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERENDERTARGETVIEW_INVALIDARG_RETURN: D3D11_MESSAGE_ID = 140i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERENDERTARGETVIEW_OUTOFMEMORY_RETURN: D3D11_MESSAGE_ID = 141i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEPTHSTENCILVIEW_UNRECOGNIZEDFORMAT: D3D11_MESSAGE_ID = 142i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEPTHSTENCILVIEW_INVALIDDESC: D3D11_MESSAGE_ID = 143i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEPTHSTENCILVIEW_INVALIDFORMAT: D3D11_MESSAGE_ID = 144i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEPTHSTENCILVIEW_INVALIDDIMENSIONS: D3D11_MESSAGE_ID = 145i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEPTHSTENCILVIEW_INVALIDRESOURCE: D3D11_MESSAGE_ID = 146i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEPTHSTENCILVIEW_TOOMANYOBJECTS: D3D11_MESSAGE_ID = 147i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEPTHSTENCILVIEW_INVALIDARG_RETURN: D3D11_MESSAGE_ID = 148i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEPTHSTENCILVIEW_OUTOFMEMORY_RETURN: D3D11_MESSAGE_ID = 149i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEINPUTLAYOUT_OUTOFMEMORY: D3D11_MESSAGE_ID = 150i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEINPUTLAYOUT_TOOMANYELEMENTS: D3D11_MESSAGE_ID = 151i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEINPUTLAYOUT_INVALIDFORMAT: D3D11_MESSAGE_ID = 152i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEINPUTLAYOUT_INCOMPATIBLEFORMAT: D3D11_MESSAGE_ID = 153i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEINPUTLAYOUT_INVALIDSLOT: D3D11_MESSAGE_ID = 154i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEINPUTLAYOUT_INVALIDINPUTSLOTCLASS: D3D11_MESSAGE_ID = 155i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEINPUTLAYOUT_STEPRATESLOTCLASSMISMATCH: D3D11_MESSAGE_ID = 156i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEINPUTLAYOUT_INVALIDSLOTCLASSCHANGE: D3D11_MESSAGE_ID = 157i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEINPUTLAYOUT_INVALIDSTEPRATECHANGE: D3D11_MESSAGE_ID = 158i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEINPUTLAYOUT_INVALIDALIGNMENT: D3D11_MESSAGE_ID = 159i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEINPUTLAYOUT_DUPLICATESEMANTIC: D3D11_MESSAGE_ID = 160i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEINPUTLAYOUT_UNPARSEABLEINPUTSIGNATURE: D3D11_MESSAGE_ID = 161i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEINPUTLAYOUT_NULLSEMANTIC: D3D11_MESSAGE_ID = 162i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEINPUTLAYOUT_MISSINGELEMENT: D3D11_MESSAGE_ID = 163i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEINPUTLAYOUT_NULLDESC: D3D11_MESSAGE_ID = 164i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVERTEXSHADER_OUTOFMEMORY: D3D11_MESSAGE_ID = 165i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVERTEXSHADER_INVALIDSHADERBYTECODE: D3D11_MESSAGE_ID = 166i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVERTEXSHADER_INVALIDSHADERTYPE: D3D11_MESSAGE_ID = 167i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADER_OUTOFMEMORY: D3D11_MESSAGE_ID = 168i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADER_INVALIDSHADERBYTECODE: D3D11_MESSAGE_ID = 169i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADER_INVALIDSHADERTYPE: D3D11_MESSAGE_ID = 170i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_OUTOFMEMORY: D3D11_MESSAGE_ID = 171i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_INVALIDSHADERBYTECODE: D3D11_MESSAGE_ID = 172i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_INVALIDSHADERTYPE: D3D11_MESSAGE_ID = 173i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_INVALIDNUMENTRIES: D3D11_MESSAGE_ID = 174i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_OUTPUTSTREAMSTRIDEUNUSED: D3D11_MESSAGE_ID = 175i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_UNEXPECTEDDECL: D3D11_MESSAGE_ID = 176i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_EXPECTEDDECL: D3D11_MESSAGE_ID = 177i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_OUTPUTSLOT0EXPECTED: D3D11_MESSAGE_ID = 178i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_INVALIDOUTPUTSLOT: D3D11_MESSAGE_ID = 179i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_ONLYONEELEMENTPERSLOT: D3D11_MESSAGE_ID = 180i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_INVALIDCOMPONENTCOUNT: D3D11_MESSAGE_ID = 181i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_INVALIDSTARTCOMPONENTANDCOMPONENTCOUNT: D3D11_MESSAGE_ID = 182i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_INVALIDGAPDEFINITION: D3D11_MESSAGE_ID = 183i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_REPEATEDOUTPUT: D3D11_MESSAGE_ID = 184i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_INVALIDOUTPUTSTREAMSTRIDE: D3D11_MESSAGE_ID = 185i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_MISSINGSEMANTIC: D3D11_MESSAGE_ID = 186i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_MASKMISMATCH: D3D11_MESSAGE_ID = 187i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_CANTHAVEONLYGAPS: D3D11_MESSAGE_ID = 188i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_DECLTOOCOMPLEX: D3D11_MESSAGE_ID = 189i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_MISSINGOUTPUTSIGNATURE: D3D11_MESSAGE_ID = 190i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEPIXELSHADER_OUTOFMEMORY: D3D11_MESSAGE_ID = 191i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEPIXELSHADER_INVALIDSHADERBYTECODE: D3D11_MESSAGE_ID = 192i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEPIXELSHADER_INVALIDSHADERTYPE: D3D11_MESSAGE_ID = 193i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERASTERIZERSTATE_INVALIDFILLMODE: D3D11_MESSAGE_ID = 194i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERASTERIZERSTATE_INVALIDCULLMODE: D3D11_MESSAGE_ID = 195i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERASTERIZERSTATE_INVALIDDEPTHBIASCLAMP: D3D11_MESSAGE_ID = 196i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERASTERIZERSTATE_INVALIDSLOPESCALEDDEPTHBIAS: D3D11_MESSAGE_ID = 197i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERASTERIZERSTATE_TOOMANYOBJECTS: D3D11_MESSAGE_ID = 198i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERASTERIZERSTATE_NULLDESC: D3D11_MESSAGE_ID = 199i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEPTHSTENCILSTATE_INVALIDDEPTHWRITEMASK: D3D11_MESSAGE_ID = 200i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEPTHSTENCILSTATE_INVALIDDEPTHFUNC: D3D11_MESSAGE_ID = 201i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEPTHSTENCILSTATE_INVALIDFRONTFACESTENCILFAILOP: D3D11_MESSAGE_ID = 202i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEPTHSTENCILSTATE_INVALIDFRONTFACESTENCILZFAILOP: D3D11_MESSAGE_ID = 203i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEPTHSTENCILSTATE_INVALIDFRONTFACESTENCILPASSOP: D3D11_MESSAGE_ID = 204i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEPTHSTENCILSTATE_INVALIDFRONTFACESTENCILFUNC: D3D11_MESSAGE_ID = 205i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEPTHSTENCILSTATE_INVALIDBACKFACESTENCILFAILOP: D3D11_MESSAGE_ID = 206i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEPTHSTENCILSTATE_INVALIDBACKFACESTENCILZFAILOP: D3D11_MESSAGE_ID = 207i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEPTHSTENCILSTATE_INVALIDBACKFACESTENCILPASSOP: D3D11_MESSAGE_ID = 208i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEPTHSTENCILSTATE_INVALIDBACKFACESTENCILFUNC: D3D11_MESSAGE_ID = 209i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEPTHSTENCILSTATE_TOOMANYOBJECTS: D3D11_MESSAGE_ID = 210i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEPTHSTENCILSTATE_NULLDESC: D3D11_MESSAGE_ID = 211i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBLENDSTATE_INVALIDSRCBLEND: D3D11_MESSAGE_ID = 212i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBLENDSTATE_INVALIDDESTBLEND: D3D11_MESSAGE_ID = 213i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBLENDSTATE_INVALIDBLENDOP: D3D11_MESSAGE_ID = 214i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBLENDSTATE_INVALIDSRCBLENDALPHA: D3D11_MESSAGE_ID = 215i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBLENDSTATE_INVALIDDESTBLENDALPHA: D3D11_MESSAGE_ID = 216i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBLENDSTATE_INVALIDBLENDOPALPHA: D3D11_MESSAGE_ID = 217i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBLENDSTATE_INVALIDRENDERTARGETWRITEMASK: D3D11_MESSAGE_ID = 218i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBLENDSTATE_TOOMANYOBJECTS: D3D11_MESSAGE_ID = 219i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBLENDSTATE_NULLDESC: D3D11_MESSAGE_ID = 220i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESAMPLERSTATE_INVALIDFILTER: D3D11_MESSAGE_ID = 221i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESAMPLERSTATE_INVALIDADDRESSU: D3D11_MESSAGE_ID = 222i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESAMPLERSTATE_INVALIDADDRESSV: D3D11_MESSAGE_ID = 223i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESAMPLERSTATE_INVALIDADDRESSW: D3D11_MESSAGE_ID = 224i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESAMPLERSTATE_INVALIDMIPLODBIAS: D3D11_MESSAGE_ID = 225i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESAMPLERSTATE_INVALIDMAXANISOTROPY: D3D11_MESSAGE_ID = 226i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESAMPLERSTATE_INVALIDCOMPARISONFUNC: D3D11_MESSAGE_ID = 227i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESAMPLERSTATE_INVALIDMINLOD: D3D11_MESSAGE_ID = 228i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESAMPLERSTATE_INVALIDMAXLOD: D3D11_MESSAGE_ID = 229i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESAMPLERSTATE_TOOMANYOBJECTS: D3D11_MESSAGE_ID = 230i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESAMPLERSTATE_NULLDESC: D3D11_MESSAGE_ID = 231i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEQUERYORPREDICATE_INVALIDQUERY: D3D11_MESSAGE_ID = 232i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEQUERYORPREDICATE_INVALIDMISCFLAGS: D3D11_MESSAGE_ID = 233i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEQUERYORPREDICATE_UNEXPECTEDMISCFLAG: D3D11_MESSAGE_ID = 234i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEQUERYORPREDICATE_NULLDESC: D3D11_MESSAGE_ID = 235i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_IASETPRIMITIVETOPOLOGY_TOPOLOGY_UNRECOGNIZED: D3D11_MESSAGE_ID = 236i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_IASETPRIMITIVETOPOLOGY_TOPOLOGY_UNDEFINED: D3D11_MESSAGE_ID = 237i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_IASETVERTEXBUFFERS_INVALIDBUFFER: D3D11_MESSAGE_ID = 238i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_IASETVERTEXBUFFERS_OFFSET_TOO_LARGE: D3D11_MESSAGE_ID = 239i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_IASETVERTEXBUFFERS_BUFFERS_EMPTY: D3D11_MESSAGE_ID = 240i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_IASETINDEXBUFFER_INVALIDBUFFER: D3D11_MESSAGE_ID = 241i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_IASETINDEXBUFFER_FORMAT_INVALID: D3D11_MESSAGE_ID = 242i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_IASETINDEXBUFFER_OFFSET_TOO_LARGE: D3D11_MESSAGE_ID = 243i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_IASETINDEXBUFFER_OFFSET_UNALIGNED: D3D11_MESSAGE_ID = 244i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_VSSETSHADERRESOURCES_VIEWS_EMPTY: D3D11_MESSAGE_ID = 245i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VSSETCONSTANTBUFFERS_INVALIDBUFFER: D3D11_MESSAGE_ID = 246i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_VSSETCONSTANTBUFFERS_BUFFERS_EMPTY: D3D11_MESSAGE_ID = 247i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_VSSETSAMPLERS_SAMPLERS_EMPTY: D3D11_MESSAGE_ID = 248i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_GSSETSHADERRESOURCES_VIEWS_EMPTY: D3D11_MESSAGE_ID = 249i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GSSETCONSTANTBUFFERS_INVALIDBUFFER: D3D11_MESSAGE_ID = 250i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_GSSETCONSTANTBUFFERS_BUFFERS_EMPTY: D3D11_MESSAGE_ID = 251i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_GSSETSAMPLERS_SAMPLERS_EMPTY: D3D11_MESSAGE_ID = 252i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_SOSETTARGETS_INVALIDBUFFER: D3D11_MESSAGE_ID = 253i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_SOSETTARGETS_OFFSET_UNALIGNED: D3D11_MESSAGE_ID = 254i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_PSSETSHADERRESOURCES_VIEWS_EMPTY: D3D11_MESSAGE_ID = 255i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_PSSETCONSTANTBUFFERS_INVALIDBUFFER: D3D11_MESSAGE_ID = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_PSSETCONSTANTBUFFERS_BUFFERS_EMPTY: D3D11_MESSAGE_ID = 257i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_PSSETSAMPLERS_SAMPLERS_EMPTY: D3D11_MESSAGE_ID = 258i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_RSSETVIEWPORTS_INVALIDVIEWPORT: D3D11_MESSAGE_ID = 259i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_RSSETSCISSORRECTS_INVALIDSCISSOR: D3D11_MESSAGE_ID = 260i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CLEARRENDERTARGETVIEW_DENORMFLUSH: D3D11_MESSAGE_ID = 261i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CLEARDEPTHSTENCILVIEW_DENORMFLUSH: D3D11_MESSAGE_ID = 262i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CLEARDEPTHSTENCILVIEW_INVALID: D3D11_MESSAGE_ID = 263i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_IAGETVERTEXBUFFERS_BUFFERS_EMPTY: D3D11_MESSAGE_ID = 264i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_VSGETSHADERRESOURCES_VIEWS_EMPTY: D3D11_MESSAGE_ID = 265i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_VSGETCONSTANTBUFFERS_BUFFERS_EMPTY: D3D11_MESSAGE_ID = 266i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_VSGETSAMPLERS_SAMPLERS_EMPTY: D3D11_MESSAGE_ID = 267i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_GSGETSHADERRESOURCES_VIEWS_EMPTY: D3D11_MESSAGE_ID = 268i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_GSGETCONSTANTBUFFERS_BUFFERS_EMPTY: D3D11_MESSAGE_ID = 269i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_GSGETSAMPLERS_SAMPLERS_EMPTY: D3D11_MESSAGE_ID = 270i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_SOGETTARGETS_BUFFERS_EMPTY: D3D11_MESSAGE_ID = 271i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_PSGETSHADERRESOURCES_VIEWS_EMPTY: D3D11_MESSAGE_ID = 272i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_PSGETCONSTANTBUFFERS_BUFFERS_EMPTY: D3D11_MESSAGE_ID = 273i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_PSGETSAMPLERS_SAMPLERS_EMPTY: D3D11_MESSAGE_ID = 274i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_RSGETVIEWPORTS_VIEWPORTS_EMPTY: D3D11_MESSAGE_ID = 275i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_RSGETSCISSORRECTS_RECTS_EMPTY: D3D11_MESSAGE_ID = 276i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_GENERATEMIPS_RESOURCE_INVALID: D3D11_MESSAGE_ID = 277i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_COPYSUBRESOURCEREGION_INVALIDDESTINATIONSUBRESOURCE: D3D11_MESSAGE_ID = 278i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_COPYSUBRESOURCEREGION_INVALIDSOURCESUBRESOURCE: D3D11_MESSAGE_ID = 279i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_COPYSUBRESOURCEREGION_INVALIDSOURCEBOX: D3D11_MESSAGE_ID = 280i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_COPYSUBRESOURCEREGION_INVALIDSOURCE: D3D11_MESSAGE_ID = 281i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_COPYSUBRESOURCEREGION_INVALIDDESTINATIONSTATE: D3D11_MESSAGE_ID = 282i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_COPYSUBRESOURCEREGION_INVALIDSOURCESTATE: D3D11_MESSAGE_ID = 283i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_COPYRESOURCE_INVALIDSOURCE: D3D11_MESSAGE_ID = 284i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_COPYRESOURCE_INVALIDDESTINATIONSTATE: D3D11_MESSAGE_ID = 285i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_COPYRESOURCE_INVALIDSOURCESTATE: D3D11_MESSAGE_ID = 286i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_UPDATESUBRESOURCE_INVALIDDESTINATIONSUBRESOURCE: D3D11_MESSAGE_ID = 287i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_UPDATESUBRESOURCE_INVALIDDESTINATIONBOX: D3D11_MESSAGE_ID = 288i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_UPDATESUBRESOURCE_INVALIDDESTINATIONSTATE: D3D11_MESSAGE_ID = 289i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_RESOLVESUBRESOURCE_DESTINATION_INVALID: D3D11_MESSAGE_ID = 290i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_RESOLVESUBRESOURCE_DESTINATION_SUBRESOURCE_INVALID: D3D11_MESSAGE_ID = 291i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_RESOLVESUBRESOURCE_SOURCE_INVALID: D3D11_MESSAGE_ID = 292i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_RESOLVESUBRESOURCE_SOURCE_SUBRESOURCE_INVALID: D3D11_MESSAGE_ID = 293i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_RESOLVESUBRESOURCE_FORMAT_INVALID: D3D11_MESSAGE_ID = 294i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_BUFFER_MAP_INVALIDMAPTYPE: D3D11_MESSAGE_ID = 295i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_BUFFER_MAP_INVALIDFLAGS: D3D11_MESSAGE_ID = 296i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_BUFFER_MAP_ALREADYMAPPED: D3D11_MESSAGE_ID = 297i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_BUFFER_MAP_DEVICEREMOVED_RETURN: D3D11_MESSAGE_ID = 298i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_BUFFER_UNMAP_NOTMAPPED: D3D11_MESSAGE_ID = 299i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TEXTURE1D_MAP_INVALIDMAPTYPE: D3D11_MESSAGE_ID = 300i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TEXTURE1D_MAP_INVALIDSUBRESOURCE: D3D11_MESSAGE_ID = 301i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TEXTURE1D_MAP_INVALIDFLAGS: D3D11_MESSAGE_ID = 302i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TEXTURE1D_MAP_ALREADYMAPPED: D3D11_MESSAGE_ID = 303i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TEXTURE1D_MAP_DEVICEREMOVED_RETURN: D3D11_MESSAGE_ID = 304i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TEXTURE1D_UNMAP_INVALIDSUBRESOURCE: D3D11_MESSAGE_ID = 305i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TEXTURE1D_UNMAP_NOTMAPPED: D3D11_MESSAGE_ID = 306i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TEXTURE2D_MAP_INVALIDMAPTYPE: D3D11_MESSAGE_ID = 307i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TEXTURE2D_MAP_INVALIDSUBRESOURCE: D3D11_MESSAGE_ID = 308i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TEXTURE2D_MAP_INVALIDFLAGS: D3D11_MESSAGE_ID = 309i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TEXTURE2D_MAP_ALREADYMAPPED: D3D11_MESSAGE_ID = 310i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TEXTURE2D_MAP_DEVICEREMOVED_RETURN: D3D11_MESSAGE_ID = 311i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TEXTURE2D_UNMAP_INVALIDSUBRESOURCE: D3D11_MESSAGE_ID = 312i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TEXTURE2D_UNMAP_NOTMAPPED: D3D11_MESSAGE_ID = 313i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TEXTURE3D_MAP_INVALIDMAPTYPE: D3D11_MESSAGE_ID = 314i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TEXTURE3D_MAP_INVALIDSUBRESOURCE: D3D11_MESSAGE_ID = 315i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TEXTURE3D_MAP_INVALIDFLAGS: D3D11_MESSAGE_ID = 316i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TEXTURE3D_MAP_ALREADYMAPPED: D3D11_MESSAGE_ID = 317i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TEXTURE3D_MAP_DEVICEREMOVED_RETURN: D3D11_MESSAGE_ID = 318i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TEXTURE3D_UNMAP_INVALIDSUBRESOURCE: D3D11_MESSAGE_ID = 319i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TEXTURE3D_UNMAP_NOTMAPPED: D3D11_MESSAGE_ID = 320i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CHECKFORMATSUPPORT_FORMAT_DEPRECATED: D3D11_MESSAGE_ID = 321i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CHECKMULTISAMPLEQUALITYLEVELS_FORMAT_DEPRECATED: D3D11_MESSAGE_ID = 322i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_SETEXCEPTIONMODE_UNRECOGNIZEDFLAGS: D3D11_MESSAGE_ID = 323i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_SETEXCEPTIONMODE_INVALIDARG_RETURN: D3D11_MESSAGE_ID = 324i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_SETEXCEPTIONMODE_DEVICEREMOVED_RETURN: D3D11_MESSAGE_ID = 325i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_REF_SIMULATING_INFINITELY_FAST_HARDWARE: D3D11_MESSAGE_ID = 326i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_REF_THREADING_MODE: D3D11_MESSAGE_ID = 327i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_REF_UMDRIVER_EXCEPTION: D3D11_MESSAGE_ID = 328i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_REF_KMDRIVER_EXCEPTION: D3D11_MESSAGE_ID = 329i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_REF_HARDWARE_EXCEPTION: D3D11_MESSAGE_ID = 330i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_REF_ACCESSING_INDEXABLE_TEMP_OUT_OF_RANGE: D3D11_MESSAGE_ID = 331i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_REF_PROBLEM_PARSING_SHADER: D3D11_MESSAGE_ID = 332i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_REF_OUT_OF_MEMORY: D3D11_MESSAGE_ID = 333i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_REF_INFO: D3D11_MESSAGE_ID = 334i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_VERTEXPOS_OVERFLOW: D3D11_MESSAGE_ID = 335i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAWINDEXED_INDEXPOS_OVERFLOW: D3D11_MESSAGE_ID = 336i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAWINSTANCED_VERTEXPOS_OVERFLOW: D3D11_MESSAGE_ID = 337i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAWINSTANCED_INSTANCEPOS_OVERFLOW: D3D11_MESSAGE_ID = 338i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAWINDEXEDINSTANCED_INSTANCEPOS_OVERFLOW: D3D11_MESSAGE_ID = 339i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAWINDEXEDINSTANCED_INDEXPOS_OVERFLOW: D3D11_MESSAGE_ID = 340i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_VERTEX_SHADER_NOT_SET: D3D11_MESSAGE_ID = 341i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_SHADER_LINKAGE_SEMANTICNAME_NOT_FOUND: D3D11_MESSAGE_ID = 342i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_SHADER_LINKAGE_REGISTERINDEX: D3D11_MESSAGE_ID = 343i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_SHADER_LINKAGE_COMPONENTTYPE: D3D11_MESSAGE_ID = 344i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_SHADER_LINKAGE_REGISTERMASK: D3D11_MESSAGE_ID = 345i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_SHADER_LINKAGE_SYSTEMVALUE: D3D11_MESSAGE_ID = 346i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_SHADER_LINKAGE_NEVERWRITTEN_ALWAYSREADS: D3D11_MESSAGE_ID = 347i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_VERTEX_BUFFER_NOT_SET: D3D11_MESSAGE_ID = 348i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_INPUTLAYOUT_NOT_SET: D3D11_MESSAGE_ID = 349i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_CONSTANT_BUFFER_NOT_SET: D3D11_MESSAGE_ID = 350i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_CONSTANT_BUFFER_TOO_SMALL: D3D11_MESSAGE_ID = 351i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_SAMPLER_NOT_SET: D3D11_MESSAGE_ID = 352i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_SHADERRESOURCEVIEW_NOT_SET: D3D11_MESSAGE_ID = 353i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_VIEW_DIMENSION_MISMATCH: D3D11_MESSAGE_ID = 354i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_VERTEX_BUFFER_STRIDE_TOO_SMALL: D3D11_MESSAGE_ID = 355i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_VERTEX_BUFFER_TOO_SMALL: D3D11_MESSAGE_ID = 356i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_INDEX_BUFFER_NOT_SET: D3D11_MESSAGE_ID = 357i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_INDEX_BUFFER_FORMAT_INVALID: D3D11_MESSAGE_ID = 358i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_INDEX_BUFFER_TOO_SMALL: D3D11_MESSAGE_ID = 359i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_GS_INPUT_PRIMITIVE_MISMATCH: D3D11_MESSAGE_ID = 360i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_RESOURCE_RETURN_TYPE_MISMATCH: D3D11_MESSAGE_ID = 361i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_POSITION_NOT_PRESENT: D3D11_MESSAGE_ID = 362i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_OUTPUT_STREAM_NOT_SET: D3D11_MESSAGE_ID = 363i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_BOUND_RESOURCE_MAPPED: D3D11_MESSAGE_ID = 364i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_INVALID_PRIMITIVETOPOLOGY: D3D11_MESSAGE_ID = 365i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_VERTEX_OFFSET_UNALIGNED: D3D11_MESSAGE_ID = 366i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_VERTEX_STRIDE_UNALIGNED: D3D11_MESSAGE_ID = 367i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_INDEX_OFFSET_UNALIGNED: D3D11_MESSAGE_ID = 368i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_OUTPUT_STREAM_OFFSET_UNALIGNED: D3D11_MESSAGE_ID = 369i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_RESOURCE_FORMAT_LD_UNSUPPORTED: D3D11_MESSAGE_ID = 370i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_RESOURCE_FORMAT_SAMPLE_UNSUPPORTED: D3D11_MESSAGE_ID = 371i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_RESOURCE_FORMAT_SAMPLE_C_UNSUPPORTED: D3D11_MESSAGE_ID = 372i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_RESOURCE_MULTISAMPLE_UNSUPPORTED: D3D11_MESSAGE_ID = 373i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_SO_TARGETS_BOUND_WITHOUT_SOURCE: D3D11_MESSAGE_ID = 374i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_SO_STRIDE_LARGER_THAN_BUFFER: D3D11_MESSAGE_ID = 375i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_OM_RENDER_TARGET_DOES_NOT_SUPPORT_BLENDING: D3D11_MESSAGE_ID = 376i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_OM_DUAL_SOURCE_BLENDING_CAN_ONLY_HAVE_RENDER_TARGET_0: D3D11_MESSAGE_ID = 377i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_REMOVAL_PROCESS_AT_FAULT: D3D11_MESSAGE_ID = 378i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_REMOVAL_PROCESS_POSSIBLY_AT_FAULT: D3D11_MESSAGE_ID = 379i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_REMOVAL_PROCESS_NOT_AT_FAULT: D3D11_MESSAGE_ID = 380i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_OPEN_SHARED_RESOURCE_INVALIDARG_RETURN: D3D11_MESSAGE_ID = 381i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_OPEN_SHARED_RESOURCE_OUTOFMEMORY_RETURN: D3D11_MESSAGE_ID = 382i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_OPEN_SHARED_RESOURCE_BADINTERFACE_RETURN: D3D11_MESSAGE_ID = 383i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_VIEWPORT_NOT_SET: D3D11_MESSAGE_ID = 384i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEINPUTLAYOUT_TRAILING_DIGIT_IN_SEMANTIC: D3D11_MESSAGE_ID = 385i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_TRAILING_DIGIT_IN_SEMANTIC: D3D11_MESSAGE_ID = 386i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_RSSETVIEWPORTS_DENORMFLUSH: D3D11_MESSAGE_ID = 387i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_OMSETRENDERTARGETS_INVALIDVIEW: D3D11_MESSAGE_ID = 388i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_SETTEXTFILTERSIZE_INVALIDDIMENSIONS: D3D11_MESSAGE_ID = 389i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_SAMPLER_MISMATCH: D3D11_MESSAGE_ID = 390i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEINPUTLAYOUT_TYPE_MISMATCH: D3D11_MESSAGE_ID = 391i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_BLENDSTATE_GETDESC_LEGACY: D3D11_MESSAGE_ID = 392i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_SHADERRESOURCEVIEW_GETDESC_LEGACY: D3D11_MESSAGE_ID = 393i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEQUERY_OUTOFMEMORY_RETURN: D3D11_MESSAGE_ID = 394i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEPREDICATE_OUTOFMEMORY_RETURN: D3D11_MESSAGE_ID = 395i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATECOUNTER_OUTOFRANGE_COUNTER: D3D11_MESSAGE_ID = 396i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATECOUNTER_SIMULTANEOUS_ACTIVE_COUNTERS_EXHAUSTED: D3D11_MESSAGE_ID = 397i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATECOUNTER_UNSUPPORTED_WELLKNOWN_COUNTER: D3D11_MESSAGE_ID = 398i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATECOUNTER_OUTOFMEMORY_RETURN: D3D11_MESSAGE_ID = 399i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATECOUNTER_NONEXCLUSIVE_RETURN: D3D11_MESSAGE_ID = 400i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATECOUNTER_NULLDESC: D3D11_MESSAGE_ID = 401i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CHECKCOUNTER_OUTOFRANGE_COUNTER: D3D11_MESSAGE_ID = 402i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CHECKCOUNTER_UNSUPPORTED_WELLKNOWN_COUNTER: D3D11_MESSAGE_ID = 403i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_SETPREDICATION_INVALID_PREDICATE_STATE: D3D11_MESSAGE_ID = 404i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_QUERY_BEGIN_UNSUPPORTED: D3D11_MESSAGE_ID = 405i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_PREDICATE_BEGIN_DURING_PREDICATION: D3D11_MESSAGE_ID = 406i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_QUERY_BEGIN_DUPLICATE: D3D11_MESSAGE_ID = 407i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_QUERY_BEGIN_ABANDONING_PREVIOUS_RESULTS: D3D11_MESSAGE_ID = 408i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_PREDICATE_END_DURING_PREDICATION: D3D11_MESSAGE_ID = 409i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_QUERY_END_ABANDONING_PREVIOUS_RESULTS: D3D11_MESSAGE_ID = 410i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_QUERY_END_WITHOUT_BEGIN: D3D11_MESSAGE_ID = 411i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_QUERY_GETDATA_INVALID_DATASIZE: D3D11_MESSAGE_ID = 412i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_QUERY_GETDATA_INVALID_FLAGS: D3D11_MESSAGE_ID = 413i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_QUERY_GETDATA_INVALID_CALL: D3D11_MESSAGE_ID = 414i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_PS_OUTPUT_TYPE_MISMATCH: D3D11_MESSAGE_ID = 415i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_RESOURCE_FORMAT_GATHER_UNSUPPORTED: D3D11_MESSAGE_ID = 416i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_INVALID_USE_OF_CENTER_MULTISAMPLE_PATTERN: D3D11_MESSAGE_ID = 417i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_IASETVERTEXBUFFERS_STRIDE_TOO_LARGE: D3D11_MESSAGE_ID = 418i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_IASETVERTEXBUFFERS_INVALIDRANGE: D3D11_MESSAGE_ID = 419i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEINPUTLAYOUT_EMPTY_LAYOUT: D3D11_MESSAGE_ID = 420i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_RESOURCE_SAMPLE_COUNT_MISMATCH: D3D11_MESSAGE_ID = 421i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_OBJECT_SUMMARY: D3D11_MESSAGE_ID = 422i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_BUFFER: D3D11_MESSAGE_ID = 423i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_TEXTURE1D: D3D11_MESSAGE_ID = 424i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_TEXTURE2D: D3D11_MESSAGE_ID = 425i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_TEXTURE3D: D3D11_MESSAGE_ID = 426i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_SHADERRESOURCEVIEW: D3D11_MESSAGE_ID = 427i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_RENDERTARGETVIEW: D3D11_MESSAGE_ID = 428i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_DEPTHSTENCILVIEW: D3D11_MESSAGE_ID = 429i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_VERTEXSHADER: D3D11_MESSAGE_ID = 430i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_GEOMETRYSHADER: D3D11_MESSAGE_ID = 431i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_PIXELSHADER: D3D11_MESSAGE_ID = 432i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_INPUTLAYOUT: D3D11_MESSAGE_ID = 433i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_SAMPLER: D3D11_MESSAGE_ID = 434i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_BLENDSTATE: D3D11_MESSAGE_ID = 435i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_DEPTHSTENCILSTATE: D3D11_MESSAGE_ID = 436i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_RASTERIZERSTATE: D3D11_MESSAGE_ID = 437i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_QUERY: D3D11_MESSAGE_ID = 438i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_PREDICATE: D3D11_MESSAGE_ID = 439i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_COUNTER: D3D11_MESSAGE_ID = 440i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_DEVICE: D3D11_MESSAGE_ID = 441i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_SWAPCHAIN: D3D11_MESSAGE_ID = 442i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_D3D10_MESSAGES_END: D3D11_MESSAGE_ID = 443i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_D3D10L9_MESSAGES_START: D3D11_MESSAGE_ID = 1048576i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEPTHSTENCILSTATE_STENCIL_NO_TWO_SIDED: D3D11_MESSAGE_ID = 1048577i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERASTERIZERSTATE_DepthBiasClamp_NOT_SUPPORTED: D3D11_MESSAGE_ID = 1048578i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESAMPLERSTATE_NO_COMPARISON_SUPPORT: D3D11_MESSAGE_ID = 1048579i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESAMPLERSTATE_EXCESSIVE_ANISOTROPY: D3D11_MESSAGE_ID = 1048580i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESAMPLERSTATE_BORDER_OUT_OF_RANGE: D3D11_MESSAGE_ID = 1048581i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VSSETSAMPLERS_NOT_SUPPORTED: D3D11_MESSAGE_ID = 1048582i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VSSETSAMPLERS_TOO_MANY_SAMPLERS: D3D11_MESSAGE_ID = 1048583i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_PSSETSAMPLERS_TOO_MANY_SAMPLERS: D3D11_MESSAGE_ID = 1048584i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERESOURCE_NO_ARRAYS: D3D11_MESSAGE_ID = 1048585i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERESOURCE_NO_VB_AND_IB_BIND: D3D11_MESSAGE_ID = 1048586i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERESOURCE_NO_TEXTURE_1D: D3D11_MESSAGE_ID = 1048587i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERESOURCE_DIMENSION_OUT_OF_RANGE: D3D11_MESSAGE_ID = 1048588i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERESOURCE_NOT_BINDABLE_AS_SHADER_RESOURCE: D3D11_MESSAGE_ID = 1048589i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_OMSETRENDERTARGETS_TOO_MANY_RENDER_TARGETS: D3D11_MESSAGE_ID = 1048590i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_OMSETRENDERTARGETS_NO_DIFFERING_BIT_DEPTHS: D3D11_MESSAGE_ID = 1048591i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_IASETVERTEXBUFFERS_BAD_BUFFER_INDEX: D3D11_MESSAGE_ID = 1048592i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_RSSETVIEWPORTS_TOO_MANY_VIEWPORTS: D3D11_MESSAGE_ID = 1048593i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_IASETPRIMITIVETOPOLOGY_ADJACENCY_UNSUPPORTED: D3D11_MESSAGE_ID = 1048594i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_RSSETSCISSORRECTS_TOO_MANY_SCISSORS: D3D11_MESSAGE_ID = 1048595i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_COPYRESOURCE_ONLY_TEXTURE_2D_WITHIN_GPU_MEMORY: D3D11_MESSAGE_ID = 1048596i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_COPYRESOURCE_NO_TEXTURE_3D_READBACK: D3D11_MESSAGE_ID = 1048597i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_COPYRESOURCE_NO_TEXTURE_ONLY_READBACK: D3D11_MESSAGE_ID = 1048598i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEINPUTLAYOUT_UNSUPPORTED_FORMAT: D3D11_MESSAGE_ID = 1048599i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBLENDSTATE_NO_ALPHA_TO_COVERAGE: D3D11_MESSAGE_ID = 1048600i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERASTERIZERSTATE_DepthClipEnable_MUST_BE_TRUE: D3D11_MESSAGE_ID = 1048601i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DRAWINDEXED_STARTINDEXLOCATION_MUST_BE_POSITIVE: D3D11_MESSAGE_ID = 1048602i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESHADERRESOURCEVIEW_MUST_USE_LOWEST_LOD: D3D11_MESSAGE_ID = 1048603i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESAMPLERSTATE_MINLOD_MUST_NOT_BE_FRACTIONAL: D3D11_MESSAGE_ID = 1048604i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESAMPLERSTATE_MAXLOD_MUST_BE_FLT_MAX: D3D11_MESSAGE_ID = 1048605i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESHADERRESOURCEVIEW_FIRSTARRAYSLICE_MUST_BE_ZERO: D3D11_MESSAGE_ID = 1048606i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESHADERRESOURCEVIEW_CUBES_MUST_HAVE_6_SIDES: D3D11_MESSAGE_ID = 1048607i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERESOURCE_NOT_BINDABLE_AS_RENDER_TARGET: D3D11_MESSAGE_ID = 1048608i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERESOURCE_NO_DWORD_INDEX_BUFFER: D3D11_MESSAGE_ID = 1048609i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERESOURCE_MSAA_PRECLUDES_SHADER_RESOURCE: D3D11_MESSAGE_ID = 1048610i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERESOURCE_PRESENTATION_PRECLUDES_SHADER_RESOURCE: D3D11_MESSAGE_ID = 1048611i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBLENDSTATE_NO_INDEPENDENT_BLEND_ENABLE: D3D11_MESSAGE_ID = 1048612i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBLENDSTATE_NO_INDEPENDENT_WRITE_MASKS: D3D11_MESSAGE_ID = 1048613i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERESOURCE_NO_STREAM_OUT: D3D11_MESSAGE_ID = 1048614i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERESOURCE_ONLY_VB_IB_FOR_BUFFERS: D3D11_MESSAGE_ID = 1048615i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERESOURCE_NO_AUTOGEN_FOR_VOLUMES: D3D11_MESSAGE_ID = 1048616i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERESOURCE_DXGI_FORMAT_R8G8B8A8_CANNOT_BE_SHARED: D3D11_MESSAGE_ID = 1048617i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VSSHADERRESOURCES_NOT_SUPPORTED: D3D11_MESSAGE_ID = 1048618i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GEOMETRY_SHADER_NOT_SUPPORTED: D3D11_MESSAGE_ID = 1048619i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_STREAM_OUT_NOT_SUPPORTED: D3D11_MESSAGE_ID = 1048620i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TEXT_FILTER_NOT_SUPPORTED: D3D11_MESSAGE_ID = 1048621i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBLENDSTATE_NO_SEPARATE_ALPHA_BLEND: D3D11_MESSAGE_ID = 1048622i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBLENDSTATE_NO_MRT_BLEND: D3D11_MESSAGE_ID = 1048623i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBLENDSTATE_OPERATION_NOT_SUPPORTED: D3D11_MESSAGE_ID = 1048624i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESAMPLERSTATE_NO_MIRRORONCE: D3D11_MESSAGE_ID = 1048625i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DRAWINSTANCED_NOT_SUPPORTED: D3D11_MESSAGE_ID = 1048626i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DRAWINDEXEDINSTANCED_NOT_SUPPORTED_BELOW_9_3: D3D11_MESSAGE_ID = 1048627i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DRAWINDEXED_POINTLIST_UNSUPPORTED: D3D11_MESSAGE_ID = 1048628i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_SETBLENDSTATE_SAMPLE_MASK_CANNOT_BE_ZERO: D3D11_MESSAGE_ID = 1048629i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERESOURCE_DIMENSION_EXCEEDS_FEATURE_LEVEL_DEFINITION: D3D11_MESSAGE_ID = 1048630i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERESOURCE_ONLY_SINGLE_MIP_LEVEL_DEPTH_STENCIL_SUPPORTED: D3D11_MESSAGE_ID = 1048631i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_RSSETSCISSORRECTS_NEGATIVESCISSOR: D3D11_MESSAGE_ID = 1048632i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_SLOT_ZERO_MUST_BE_D3D10_INPUT_PER_VERTEX_DATA: D3D11_MESSAGE_ID = 1048633i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERESOURCE_NON_POW_2_MIPMAP: D3D11_MESSAGE_ID = 1048634i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESAMPLERSTATE_BORDER_NOT_SUPPORTED: D3D11_MESSAGE_ID = 1048635i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_OMSETRENDERTARGETS_NO_SRGB_MRT: D3D11_MESSAGE_ID = 1048636i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_COPYRESOURCE_NO_3D_MISMATCHED_UPDATES: D3D11_MESSAGE_ID = 1048637i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_D3D10L9_MESSAGES_END: D3D11_MESSAGE_ID = 1048638i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_D3D11_MESSAGES_START: D3D11_MESSAGE_ID = 2097152i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEPTHSTENCILVIEW_INVALIDFLAGS: D3D11_MESSAGE_ID = 2097153i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVERTEXSHADER_INVALIDCLASSLINKAGE: D3D11_MESSAGE_ID = 2097154i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADER_INVALIDCLASSLINKAGE: D3D11_MESSAGE_ID = 2097155i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_INVALIDNUMSTREAMS: D3D11_MESSAGE_ID = 2097156i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_INVALIDSTREAMTORASTERIZER: D3D11_MESSAGE_ID = 2097157i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_UNEXPECTEDSTREAMS: D3D11_MESSAGE_ID = 2097158i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_INVALIDCLASSLINKAGE: D3D11_MESSAGE_ID = 2097159i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEPIXELSHADER_INVALIDCLASSLINKAGE: D3D11_MESSAGE_ID = 2097160i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEFERREDCONTEXT_INVALID_COMMANDLISTFLAGS: D3D11_MESSAGE_ID = 2097161i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEFERREDCONTEXT_SINGLETHREADED: D3D11_MESSAGE_ID = 2097162i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEFERREDCONTEXT_INVALIDARG_RETURN: D3D11_MESSAGE_ID = 2097163i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEFERREDCONTEXT_INVALID_CALL_RETURN: D3D11_MESSAGE_ID = 2097164i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEFERREDCONTEXT_OUTOFMEMORY_RETURN: D3D11_MESSAGE_ID = 2097165i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_FINISHDISPLAYLIST_ONIMMEDIATECONTEXT: D3D11_MESSAGE_ID = 2097166i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_FINISHDISPLAYLIST_OUTOFMEMORY_RETURN: D3D11_MESSAGE_ID = 2097167i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_FINISHDISPLAYLIST_INVALID_CALL_RETURN: D3D11_MESSAGE_ID = 2097168i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_INVALIDSTREAM: D3D11_MESSAGE_ID = 2097169i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_UNEXPECTEDENTRIES: D3D11_MESSAGE_ID = 2097170i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_UNEXPECTEDSTRIDES: D3D11_MESSAGE_ID = 2097171i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_INVALIDNUMSTRIDES: D3D11_MESSAGE_ID = 2097172i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_HSSETSHADERRESOURCES_HAZARD: D3D11_MESSAGE_ID = 2097173i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_HSSETCONSTANTBUFFERS_HAZARD: D3D11_MESSAGE_ID = 2097174i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_HSSETSHADERRESOURCES_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 2097175i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_HSSETCONSTANTBUFFERS_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 2097176i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEHULLSHADER_INVALIDCALL: D3D11_MESSAGE_ID = 2097177i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEHULLSHADER_OUTOFMEMORY: D3D11_MESSAGE_ID = 2097178i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEHULLSHADER_INVALIDSHADERBYTECODE: D3D11_MESSAGE_ID = 2097179i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEHULLSHADER_INVALIDSHADERTYPE: D3D11_MESSAGE_ID = 2097180i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEHULLSHADER_INVALIDCLASSLINKAGE: D3D11_MESSAGE_ID = 2097181i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_HSSETSHADERRESOURCES_VIEWS_EMPTY: D3D11_MESSAGE_ID = 2097182i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_HSSETCONSTANTBUFFERS_INVALIDBUFFER: D3D11_MESSAGE_ID = 2097183i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_HSSETCONSTANTBUFFERS_BUFFERS_EMPTY: D3D11_MESSAGE_ID = 2097184i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_HSSETSAMPLERS_SAMPLERS_EMPTY: D3D11_MESSAGE_ID = 2097185i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_HSGETSHADERRESOURCES_VIEWS_EMPTY: D3D11_MESSAGE_ID = 2097186i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_HSGETCONSTANTBUFFERS_BUFFERS_EMPTY: D3D11_MESSAGE_ID = 2097187i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_HSGETSAMPLERS_SAMPLERS_EMPTY: D3D11_MESSAGE_ID = 2097188i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DSSETSHADERRESOURCES_HAZARD: D3D11_MESSAGE_ID = 2097189i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DSSETCONSTANTBUFFERS_HAZARD: D3D11_MESSAGE_ID = 2097190i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DSSETSHADERRESOURCES_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 2097191i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DSSETCONSTANTBUFFERS_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 2097192i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDOMAINSHADER_INVALIDCALL: D3D11_MESSAGE_ID = 2097193i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDOMAINSHADER_OUTOFMEMORY: D3D11_MESSAGE_ID = 2097194i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDOMAINSHADER_INVALIDSHADERBYTECODE: D3D11_MESSAGE_ID = 2097195i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDOMAINSHADER_INVALIDSHADERTYPE: D3D11_MESSAGE_ID = 2097196i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDOMAINSHADER_INVALIDCLASSLINKAGE: D3D11_MESSAGE_ID = 2097197i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DSSETSHADERRESOURCES_VIEWS_EMPTY: D3D11_MESSAGE_ID = 2097198i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DSSETCONSTANTBUFFERS_INVALIDBUFFER: D3D11_MESSAGE_ID = 2097199i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DSSETCONSTANTBUFFERS_BUFFERS_EMPTY: D3D11_MESSAGE_ID = 2097200i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DSSETSAMPLERS_SAMPLERS_EMPTY: D3D11_MESSAGE_ID = 2097201i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DSGETSHADERRESOURCES_VIEWS_EMPTY: D3D11_MESSAGE_ID = 2097202i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DSGETCONSTANTBUFFERS_BUFFERS_EMPTY: D3D11_MESSAGE_ID = 2097203i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DSGETSAMPLERS_SAMPLERS_EMPTY: D3D11_MESSAGE_ID = 2097204i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_HS_XOR_DS_MISMATCH: D3D11_MESSAGE_ID = 2097205i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEFERRED_CONTEXT_REMOVAL_PROCESS_AT_FAULT: D3D11_MESSAGE_ID = 2097206i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAWINDIRECT_INVALID_ARG_BUFFER: D3D11_MESSAGE_ID = 2097207i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAWINDIRECT_OFFSET_UNALIGNED: D3D11_MESSAGE_ID = 2097208i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAWINDIRECT_OFFSET_OVERFLOW: D3D11_MESSAGE_ID = 2097209i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_RESOURCE_MAP_INVALIDMAPTYPE: D3D11_MESSAGE_ID = 2097210i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_RESOURCE_MAP_INVALIDSUBRESOURCE: D3D11_MESSAGE_ID = 2097211i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_RESOURCE_MAP_INVALIDFLAGS: D3D11_MESSAGE_ID = 2097212i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_RESOURCE_MAP_ALREADYMAPPED: D3D11_MESSAGE_ID = 2097213i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_RESOURCE_MAP_DEVICEREMOVED_RETURN: D3D11_MESSAGE_ID = 2097214i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_RESOURCE_MAP_OUTOFMEMORY_RETURN: D3D11_MESSAGE_ID = 2097215i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_RESOURCE_MAP_WITHOUT_INITIAL_DISCARD: D3D11_MESSAGE_ID = 2097216i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_RESOURCE_UNMAP_INVALIDSUBRESOURCE: D3D11_MESSAGE_ID = 2097217i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_RESOURCE_UNMAP_NOTMAPPED: D3D11_MESSAGE_ID = 2097218i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_RASTERIZING_CONTROL_POINTS: D3D11_MESSAGE_ID = 2097219i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_IASETPRIMITIVETOPOLOGY_TOPOLOGY_UNSUPPORTED: D3D11_MESSAGE_ID = 2097220i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_HS_DS_SIGNATURE_MISMATCH: D3D11_MESSAGE_ID = 2097221i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_HULL_SHADER_INPUT_TOPOLOGY_MISMATCH: D3D11_MESSAGE_ID = 2097222i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_HS_DS_CONTROL_POINT_COUNT_MISMATCH: D3D11_MESSAGE_ID = 2097223i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_HS_DS_TESSELLATOR_DOMAIN_MISMATCH: D3D11_MESSAGE_ID = 2097224i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_CONTEXT: D3D11_MESSAGE_ID = 2097225i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_CONTEXT: D3D11_MESSAGE_ID = 2097226i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_CONTEXT: D3D11_MESSAGE_ID = 2097227i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_BUFFER: D3D11_MESSAGE_ID = 2097228i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_BUFFER_WIN7: D3D11_MESSAGE_ID = 2097229i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_BUFFER: D3D11_MESSAGE_ID = 2097230i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_TEXTURE1D: D3D11_MESSAGE_ID = 2097231i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_TEXTURE1D_WIN7: D3D11_MESSAGE_ID = 2097232i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_TEXTURE1D: D3D11_MESSAGE_ID = 2097233i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_TEXTURE2D: D3D11_MESSAGE_ID = 2097234i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_TEXTURE2D_WIN7: D3D11_MESSAGE_ID = 2097235i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_TEXTURE2D: D3D11_MESSAGE_ID = 2097236i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_TEXTURE3D: D3D11_MESSAGE_ID = 2097237i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_TEXTURE3D_WIN7: D3D11_MESSAGE_ID = 2097238i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_TEXTURE3D: D3D11_MESSAGE_ID = 2097239i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_SHADERRESOURCEVIEW: D3D11_MESSAGE_ID = 2097240i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_SHADERRESOURCEVIEW_WIN7: D3D11_MESSAGE_ID = 2097241i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_SHADERRESOURCEVIEW: D3D11_MESSAGE_ID = 2097242i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_RENDERTARGETVIEW: D3D11_MESSAGE_ID = 2097243i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_RENDERTARGETVIEW_WIN7: D3D11_MESSAGE_ID = 2097244i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_RENDERTARGETVIEW: D3D11_MESSAGE_ID = 2097245i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_DEPTHSTENCILVIEW: D3D11_MESSAGE_ID = 2097246i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_DEPTHSTENCILVIEW_WIN7: D3D11_MESSAGE_ID = 2097247i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_DEPTHSTENCILVIEW: D3D11_MESSAGE_ID = 2097248i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_VERTEXSHADER: D3D11_MESSAGE_ID = 2097249i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_VERTEXSHADER_WIN7: D3D11_MESSAGE_ID = 2097250i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_VERTEXSHADER: D3D11_MESSAGE_ID = 2097251i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_HULLSHADER: D3D11_MESSAGE_ID = 2097252i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_HULLSHADER: D3D11_MESSAGE_ID = 2097253i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_HULLSHADER: D3D11_MESSAGE_ID = 2097254i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_DOMAINSHADER: D3D11_MESSAGE_ID = 2097255i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_DOMAINSHADER: D3D11_MESSAGE_ID = 2097256i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_DOMAINSHADER: D3D11_MESSAGE_ID = 2097257i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_GEOMETRYSHADER: D3D11_MESSAGE_ID = 2097258i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_GEOMETRYSHADER_WIN7: D3D11_MESSAGE_ID = 2097259i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_GEOMETRYSHADER: D3D11_MESSAGE_ID = 2097260i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_PIXELSHADER: D3D11_MESSAGE_ID = 2097261i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_PIXELSHADER_WIN7: D3D11_MESSAGE_ID = 2097262i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_PIXELSHADER: D3D11_MESSAGE_ID = 2097263i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_INPUTLAYOUT: D3D11_MESSAGE_ID = 2097264i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_INPUTLAYOUT_WIN7: D3D11_MESSAGE_ID = 2097265i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_INPUTLAYOUT: D3D11_MESSAGE_ID = 2097266i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_SAMPLER: D3D11_MESSAGE_ID = 2097267i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_SAMPLER_WIN7: D3D11_MESSAGE_ID = 2097268i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_SAMPLER: D3D11_MESSAGE_ID = 2097269i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_BLENDSTATE: D3D11_MESSAGE_ID = 2097270i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_BLENDSTATE_WIN7: D3D11_MESSAGE_ID = 2097271i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_BLENDSTATE: D3D11_MESSAGE_ID = 2097272i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_DEPTHSTENCILSTATE: D3D11_MESSAGE_ID = 2097273i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_DEPTHSTENCILSTATE_WIN7: D3D11_MESSAGE_ID = 2097274i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_DEPTHSTENCILSTATE: D3D11_MESSAGE_ID = 2097275i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_RASTERIZERSTATE: D3D11_MESSAGE_ID = 2097276i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_RASTERIZERSTATE_WIN7: D3D11_MESSAGE_ID = 2097277i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_RASTERIZERSTATE: D3D11_MESSAGE_ID = 2097278i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_QUERY: D3D11_MESSAGE_ID = 2097279i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_QUERY_WIN7: D3D11_MESSAGE_ID = 2097280i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_QUERY: D3D11_MESSAGE_ID = 2097281i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_PREDICATE: D3D11_MESSAGE_ID = 2097282i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_PREDICATE_WIN7: D3D11_MESSAGE_ID = 2097283i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_PREDICATE: D3D11_MESSAGE_ID = 2097284i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_COUNTER: D3D11_MESSAGE_ID = 2097285i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_COUNTER: D3D11_MESSAGE_ID = 2097286i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_COMMANDLIST: D3D11_MESSAGE_ID = 2097287i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_COMMANDLIST: D3D11_MESSAGE_ID = 2097288i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_COMMANDLIST: D3D11_MESSAGE_ID = 2097289i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_CLASSINSTANCE: D3D11_MESSAGE_ID = 2097290i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_CLASSINSTANCE: D3D11_MESSAGE_ID = 2097291i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_CLASSINSTANCE: D3D11_MESSAGE_ID = 2097292i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_CLASSLINKAGE: D3D11_MESSAGE_ID = 2097293i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_CLASSLINKAGE: D3D11_MESSAGE_ID = 2097294i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_CLASSLINKAGE: D3D11_MESSAGE_ID = 2097295i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_DEVICE_WIN7: D3D11_MESSAGE_ID = 2097296i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_OBJECT_SUMMARY_WIN7: D3D11_MESSAGE_ID = 2097297i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_COMPUTESHADER: D3D11_MESSAGE_ID = 2097298i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_COMPUTESHADER: D3D11_MESSAGE_ID = 2097299i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_COMPUTESHADER: D3D11_MESSAGE_ID = 2097300i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_UNORDEREDACCESSVIEW: D3D11_MESSAGE_ID = 2097301i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_UNORDEREDACCESSVIEW: D3D11_MESSAGE_ID = 2097302i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_UNORDEREDACCESSVIEW: D3D11_MESSAGE_ID = 2097303i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_SETSHADER_INTERFACES_FEATURELEVEL: D3D11_MESSAGE_ID = 2097304i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_SETSHADER_INTERFACE_COUNT_MISMATCH: D3D11_MESSAGE_ID = 2097305i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_SETSHADER_INVALID_INSTANCE: D3D11_MESSAGE_ID = 2097306i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_SETSHADER_INVALID_INSTANCE_INDEX: D3D11_MESSAGE_ID = 2097307i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_SETSHADER_INVALID_INSTANCE_TYPE: D3D11_MESSAGE_ID = 2097308i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_SETSHADER_INVALID_INSTANCE_DATA: D3D11_MESSAGE_ID = 2097309i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_SETSHADER_UNBOUND_INSTANCE_DATA: D3D11_MESSAGE_ID = 2097310i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_SETSHADER_INSTANCE_DATA_BINDINGS: D3D11_MESSAGE_ID = 2097311i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CREATESHADER_CLASSLINKAGE_FULL: D3D11_MESSAGE_ID = 2097312i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CHECKFEATURESUPPORT_UNRECOGNIZED_FEATURE: D3D11_MESSAGE_ID = 2097313i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CHECKFEATURESUPPORT_MISMATCHED_DATA_SIZE: D3D11_MESSAGE_ID = 2097314i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CHECKFEATURESUPPORT_INVALIDARG_RETURN: D3D11_MESSAGE_ID = 2097315i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CSSETSHADERRESOURCES_HAZARD: D3D11_MESSAGE_ID = 2097316i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CSSETCONSTANTBUFFERS_HAZARD: D3D11_MESSAGE_ID = 2097317i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CSSETSHADERRESOURCES_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 2097318i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CSSETCONSTANTBUFFERS_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 2097319i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATECOMPUTESHADER_INVALIDCALL: D3D11_MESSAGE_ID = 2097320i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATECOMPUTESHADER_OUTOFMEMORY: D3D11_MESSAGE_ID = 2097321i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATECOMPUTESHADER_INVALIDSHADERBYTECODE: D3D11_MESSAGE_ID = 2097322i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATECOMPUTESHADER_INVALIDSHADERTYPE: D3D11_MESSAGE_ID = 2097323i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATECOMPUTESHADER_INVALIDCLASSLINKAGE: D3D11_MESSAGE_ID = 2097324i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CSSETSHADERRESOURCES_VIEWS_EMPTY: D3D11_MESSAGE_ID = 2097325i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CSSETCONSTANTBUFFERS_INVALIDBUFFER: D3D11_MESSAGE_ID = 2097326i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CSSETCONSTANTBUFFERS_BUFFERS_EMPTY: D3D11_MESSAGE_ID = 2097327i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CSSETSAMPLERS_SAMPLERS_EMPTY: D3D11_MESSAGE_ID = 2097328i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CSGETSHADERRESOURCES_VIEWS_EMPTY: D3D11_MESSAGE_ID = 2097329i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CSGETCONSTANTBUFFERS_BUFFERS_EMPTY: D3D11_MESSAGE_ID = 2097330i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CSGETSAMPLERS_SAMPLERS_EMPTY: D3D11_MESSAGE_ID = 2097331i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CREATEVERTEXSHADER_DOUBLEFLOATOPSNOTSUPPORTED: D3D11_MESSAGE_ID = 2097332i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CREATEHULLSHADER_DOUBLEFLOATOPSNOTSUPPORTED: D3D11_MESSAGE_ID = 2097333i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CREATEDOMAINSHADER_DOUBLEFLOATOPSNOTSUPPORTED: D3D11_MESSAGE_ID = 2097334i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CREATEGEOMETRYSHADER_DOUBLEFLOATOPSNOTSUPPORTED: D3D11_MESSAGE_ID = 2097335i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_DOUBLEFLOATOPSNOTSUPPORTED: D3D11_MESSAGE_ID = 2097336i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CREATEPIXELSHADER_DOUBLEFLOATOPSNOTSUPPORTED: D3D11_MESSAGE_ID = 2097337i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CREATECOMPUTESHADER_DOUBLEFLOATOPSNOTSUPPORTED: D3D11_MESSAGE_ID = 2097338i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBUFFER_INVALIDSTRUCTURESTRIDE: D3D11_MESSAGE_ID = 2097339i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESHADERRESOURCEVIEW_INVALIDFLAGS: D3D11_MESSAGE_ID = 2097340i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEUNORDEREDACCESSVIEW_INVALIDRESOURCE: D3D11_MESSAGE_ID = 2097341i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEUNORDEREDACCESSVIEW_INVALIDDESC: D3D11_MESSAGE_ID = 2097342i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEUNORDEREDACCESSVIEW_INVALIDFORMAT: D3D11_MESSAGE_ID = 2097343i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEUNORDEREDACCESSVIEW_INVALIDDIMENSIONS: D3D11_MESSAGE_ID = 2097344i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEUNORDEREDACCESSVIEW_UNRECOGNIZEDFORMAT: D3D11_MESSAGE_ID = 2097345i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_OMSETRENDERTARGETSANDUNORDEREDACCESSVIEWS_HAZARD: D3D11_MESSAGE_ID = 2097346i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_OMSETRENDERTARGETSANDUNORDEREDACCESSVIEWS_OVERLAPPING_OLD_SLOTS: D3D11_MESSAGE_ID = 2097347i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_OMSETRENDERTARGETSANDUNORDEREDACCESSVIEWS_NO_OP: D3D11_MESSAGE_ID = 2097348i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CSSETUNORDEREDACCESSVIEWS_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 2097349i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_PSSETUNORDEREDACCESSVIEWS_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 2097350i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEUNORDEREDACCESSVIEW_INVALIDARG_RETURN: D3D11_MESSAGE_ID = 2097351i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEUNORDEREDACCESSVIEW_OUTOFMEMORY_RETURN: D3D11_MESSAGE_ID = 2097352i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEUNORDEREDACCESSVIEW_TOOMANYOBJECTS: D3D11_MESSAGE_ID = 2097353i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CSSETUNORDEREDACCESSVIEWS_HAZARD: D3D11_MESSAGE_ID = 2097354i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CLEARUNORDEREDACCESSVIEW_DENORMFLUSH: D3D11_MESSAGE_ID = 2097355i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CSSETUNORDEREDACCESSS_VIEWS_EMPTY: D3D11_MESSAGE_ID = 2097356i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CSGETUNORDEREDACCESSS_VIEWS_EMPTY: D3D11_MESSAGE_ID = 2097357i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEUNORDEREDACCESSVIEW_INVALIDFLAGS: D3D11_MESSAGE_ID = 2097358i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESHADERRESESOURCEVIEW_TOOMANYOBJECTS: D3D11_MESSAGE_ID = 2097359i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DISPATCHINDIRECT_INVALID_ARG_BUFFER: D3D11_MESSAGE_ID = 2097360i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DISPATCHINDIRECT_OFFSET_UNALIGNED: D3D11_MESSAGE_ID = 2097361i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DISPATCHINDIRECT_OFFSET_OVERFLOW: D3D11_MESSAGE_ID = 2097362i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_SETRESOURCEMINLOD_INVALIDCONTEXT: D3D11_MESSAGE_ID = 2097363i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_SETRESOURCEMINLOD_INVALIDRESOURCE: D3D11_MESSAGE_ID = 2097364i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_SETRESOURCEMINLOD_INVALIDMINLOD: D3D11_MESSAGE_ID = 2097365i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_GETRESOURCEMINLOD_INVALIDCONTEXT: D3D11_MESSAGE_ID = 2097366i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_GETRESOURCEMINLOD_INVALIDRESOURCE: D3D11_MESSAGE_ID = 2097367i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_OMSETDEPTHSTENCIL_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 2097368i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CLEARDEPTHSTENCILVIEW_DEPTH_READONLY: D3D11_MESSAGE_ID = 2097369i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CLEARDEPTHSTENCILVIEW_STENCIL_READONLY: D3D11_MESSAGE_ID = 2097370i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CHECKFEATURESUPPORT_FORMAT_DEPRECATED: D3D11_MESSAGE_ID = 2097371i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_UNORDEREDACCESSVIEW_RETURN_TYPE_MISMATCH: D3D11_MESSAGE_ID = 2097372i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_UNORDEREDACCESSVIEW_NOT_SET: D3D11_MESSAGE_ID = 2097373i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_UNORDEREDACCESSVIEW_RENDERTARGETVIEW_OVERLAP: D3D11_MESSAGE_ID = 2097374i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_UNORDEREDACCESSVIEW_DIMENSION_MISMATCH: D3D11_MESSAGE_ID = 2097375i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_UNORDEREDACCESSVIEW_APPEND_UNSUPPORTED: D3D11_MESSAGE_ID = 2097376i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_UNORDEREDACCESSVIEW_ATOMICS_UNSUPPORTED: D3D11_MESSAGE_ID = 2097377i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_UNORDEREDACCESSVIEW_STRUCTURE_STRIDE_MISMATCH: D3D11_MESSAGE_ID = 2097378i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_UNORDEREDACCESSVIEW_BUFFER_TYPE_MISMATCH: D3D11_MESSAGE_ID = 2097379i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_UNORDEREDACCESSVIEW_RAW_UNSUPPORTED: D3D11_MESSAGE_ID = 2097380i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_UNORDEREDACCESSVIEW_FORMAT_LD_UNSUPPORTED: D3D11_MESSAGE_ID = 2097381i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_UNORDEREDACCESSVIEW_FORMAT_STORE_UNSUPPORTED: D3D11_MESSAGE_ID = 2097382i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_UNORDEREDACCESSVIEW_ATOMIC_ADD_UNSUPPORTED: D3D11_MESSAGE_ID = 2097383i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_UNORDEREDACCESSVIEW_ATOMIC_BITWISE_OPS_UNSUPPORTED: D3D11_MESSAGE_ID = 2097384i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_UNORDEREDACCESSVIEW_ATOMIC_CMPSTORE_CMPEXCHANGE_UNSUPPORTED: D3D11_MESSAGE_ID = 2097385i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_UNORDEREDACCESSVIEW_ATOMIC_EXCHANGE_UNSUPPORTED: D3D11_MESSAGE_ID = 2097386i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_UNORDEREDACCESSVIEW_ATOMIC_SIGNED_MINMAX_UNSUPPORTED: D3D11_MESSAGE_ID = 2097387i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_UNORDEREDACCESSVIEW_ATOMIC_UNSIGNED_MINMAX_UNSUPPORTED: D3D11_MESSAGE_ID = 2097388i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DISPATCH_BOUND_RESOURCE_MAPPED: D3D11_MESSAGE_ID = 2097389i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DISPATCH_THREADGROUPCOUNT_OVERFLOW: D3D11_MESSAGE_ID = 2097390i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DISPATCH_THREADGROUPCOUNT_ZERO: D3D11_MESSAGE_ID = 2097391i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_SHADERRESOURCEVIEW_STRUCTURE_STRIDE_MISMATCH: D3D11_MESSAGE_ID = 2097392i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_SHADERRESOURCEVIEW_BUFFER_TYPE_MISMATCH: D3D11_MESSAGE_ID = 2097393i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_SHADERRESOURCEVIEW_RAW_UNSUPPORTED: D3D11_MESSAGE_ID = 2097394i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DISPATCH_UNSUPPORTED: D3D11_MESSAGE_ID = 2097395i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DISPATCHINDIRECT_UNSUPPORTED: D3D11_MESSAGE_ID = 2097396i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_COPYSTRUCTURECOUNT_INVALIDOFFSET: D3D11_MESSAGE_ID = 2097397i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_COPYSTRUCTURECOUNT_LARGEOFFSET: D3D11_MESSAGE_ID = 2097398i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_COPYSTRUCTURECOUNT_INVALIDDESTINATIONSTATE: D3D11_MESSAGE_ID = 2097399i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_COPYSTRUCTURECOUNT_INVALIDSOURCESTATE: D3D11_MESSAGE_ID = 2097400i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CHECKFORMATSUPPORT_FORMAT_NOT_SUPPORTED: D3D11_MESSAGE_ID = 2097401i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CSSETUNORDEREDACCESSVIEWS_INVALIDVIEW: D3D11_MESSAGE_ID = 2097402i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CSSETUNORDEREDACCESSVIEWS_INVALIDOFFSET: D3D11_MESSAGE_ID = 2097403i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CSSETUNORDEREDACCESSVIEWS_TOOMANYVIEWS: D3D11_MESSAGE_ID = 2097404i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CLEARUNORDEREDACCESSVIEWFLOAT_INVALIDFORMAT: D3D11_MESSAGE_ID = 2097405i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_UNORDEREDACCESSVIEW_COUNTER_UNSUPPORTED: D3D11_MESSAGE_ID = 2097406i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_REF_WARNING: D3D11_MESSAGE_ID = 2097407i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_PIXEL_SHADER_WITHOUT_RTV_OR_DSV: D3D11_MESSAGE_ID = 2097408i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_SHADER_ABORT: D3D11_MESSAGE_ID = 2097409i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_SHADER_MESSAGE: D3D11_MESSAGE_ID = 2097410i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_SHADER_ERROR: D3D11_MESSAGE_ID = 2097411i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_OFFERRESOURCES_INVALIDRESOURCE: D3D11_MESSAGE_ID = 2097412i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_HSSETSAMPLERS_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 2097413i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DSSETSAMPLERS_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 2097414i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CSSETSAMPLERS_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 2097415i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_HSSETSHADER_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 2097416i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DSSETSHADER_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 2097417i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CSSETSHADER_UNBINDDELETINGOBJECT: D3D11_MESSAGE_ID = 2097418i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_ENQUEUESETEVENT_INVALIDARG_RETURN: D3D11_MESSAGE_ID = 2097419i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_ENQUEUESETEVENT_OUTOFMEMORY_RETURN: D3D11_MESSAGE_ID = 2097420i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_ENQUEUESETEVENT_ACCESSDENIED_RETURN: D3D11_MESSAGE_ID = 2097421i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_OMSETRENDERTARGETSANDUNORDEREDACCESSVIEWS_NUMUAVS_INVALIDRANGE: D3D11_MESSAGE_ID = 2097422i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_USE_OF_ZERO_REFCOUNT_OBJECT: D3D11_MESSAGE_ID = 2097423i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_D3D11_MESSAGES_END: D3D11_MESSAGE_ID = 2097424i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_D3D11_1_MESSAGES_START: D3D11_MESSAGE_ID = 3145728i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_VIDEODECODER: D3D11_MESSAGE_ID = 3145729i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_VIDEOPROCESSORENUM: D3D11_MESSAGE_ID = 3145730i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_VIDEOPROCESSOR: D3D11_MESSAGE_ID = 3145731i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_DECODEROUTPUTVIEW: D3D11_MESSAGE_ID = 3145732i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_PROCESSORINPUTVIEW: D3D11_MESSAGE_ID = 3145733i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_PROCESSOROUTPUTVIEW: D3D11_MESSAGE_ID = 3145734i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_DEVICECONTEXTSTATE: D3D11_MESSAGE_ID = 3145735i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_VIDEODECODER: D3D11_MESSAGE_ID = 3145736i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_VIDEOPROCESSORENUM: D3D11_MESSAGE_ID = 3145737i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_VIDEOPROCESSOR: D3D11_MESSAGE_ID = 3145738i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_DECODEROUTPUTVIEW: D3D11_MESSAGE_ID = 3145739i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_PROCESSORINPUTVIEW: D3D11_MESSAGE_ID = 3145740i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_PROCESSOROUTPUTVIEW: D3D11_MESSAGE_ID = 3145741i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_DEVICECONTEXTSTATE: D3D11_MESSAGE_ID = 3145742i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_VIDEODECODER: D3D11_MESSAGE_ID = 3145743i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_VIDEOPROCESSORENUM: D3D11_MESSAGE_ID = 3145744i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_VIDEOPROCESSOR: D3D11_MESSAGE_ID = 3145745i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_DECODEROUTPUTVIEW: D3D11_MESSAGE_ID = 3145746i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_PROCESSORINPUTVIEW: D3D11_MESSAGE_ID = 3145747i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_PROCESSOROUTPUTVIEW: D3D11_MESSAGE_ID = 3145748i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_DEVICECONTEXTSTATE: D3D11_MESSAGE_ID = 3145749i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEVICECONTEXTSTATE_INVALIDFLAGS: D3D11_MESSAGE_ID = 3145750i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEVICECONTEXTSTATE_INVALIDFEATURELEVEL: D3D11_MESSAGE_ID = 3145751i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEVICECONTEXTSTATE_FEATURELEVELS_NOT_SUPPORTED: D3D11_MESSAGE_ID = 3145752i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEVICECONTEXTSTATE_INVALIDREFIID: D3D11_MESSAGE_ID = 3145753i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DISCARDVIEW_INVALIDVIEW: D3D11_MESSAGE_ID = 3145754i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_COPYSUBRESOURCEREGION1_INVALIDCOPYFLAGS: D3D11_MESSAGE_ID = 3145755i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_UPDATESUBRESOURCE1_INVALIDCOPYFLAGS: D3D11_MESSAGE_ID = 3145756i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERASTERIZERSTATE_INVALIDFORCEDSAMPLECOUNT: D3D11_MESSAGE_ID = 3145757i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEODECODER_OUTOFMEMORY_RETURN: D3D11_MESSAGE_ID = 3145758i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEODECODER_NULLPARAM: D3D11_MESSAGE_ID = 3145759i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEODECODER_INVALIDFORMAT: D3D11_MESSAGE_ID = 3145760i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEODECODER_ZEROWIDTHHEIGHT: D3D11_MESSAGE_ID = 3145761i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEODECODER_DRIVER_INVALIDBUFFERSIZE: D3D11_MESSAGE_ID = 3145762i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEODECODER_DRIVER_INVALIDBUFFERUSAGE: D3D11_MESSAGE_ID = 3145763i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETVIDEODECODERPROFILECOUNT_OUTOFMEMORY: D3D11_MESSAGE_ID = 3145764i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETVIDEODECODERPROFILE_NULLPARAM: D3D11_MESSAGE_ID = 3145765i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETVIDEODECODERPROFILE_INVALIDINDEX: D3D11_MESSAGE_ID = 3145766i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETVIDEODECODERPROFILE_OUTOFMEMORY_RETURN: D3D11_MESSAGE_ID = 3145767i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CHECKVIDEODECODERFORMAT_NULLPARAM: D3D11_MESSAGE_ID = 3145768i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CHECKVIDEODECODERFORMAT_OUTOFMEMORY_RETURN: D3D11_MESSAGE_ID = 3145769i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETVIDEODECODERCONFIGCOUNT_NULLPARAM: D3D11_MESSAGE_ID = 3145770i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETVIDEODECODERCONFIGCOUNT_OUTOFMEMORY_RETURN: D3D11_MESSAGE_ID = 3145771i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETVIDEODECODERCONFIG_NULLPARAM: D3D11_MESSAGE_ID = 3145772i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETVIDEODECODERCONFIG_INVALIDINDEX: D3D11_MESSAGE_ID = 3145773i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETVIDEODECODERCONFIG_OUTOFMEMORY_RETURN: D3D11_MESSAGE_ID = 3145774i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETDECODERCREATIONPARAMS_NULLPARAM: D3D11_MESSAGE_ID = 3145775i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETDECODERDRIVERHANDLE_NULLPARAM: D3D11_MESSAGE_ID = 3145776i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETDECODERBUFFER_NULLPARAM: D3D11_MESSAGE_ID = 3145777i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETDECODERBUFFER_INVALIDBUFFER: D3D11_MESSAGE_ID = 3145778i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETDECODERBUFFER_INVALIDTYPE: D3D11_MESSAGE_ID = 3145779i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETDECODERBUFFER_LOCKED: D3D11_MESSAGE_ID = 3145780i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_RELEASEDECODERBUFFER_NULLPARAM: D3D11_MESSAGE_ID = 3145781i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_RELEASEDECODERBUFFER_INVALIDTYPE: D3D11_MESSAGE_ID = 3145782i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_RELEASEDECODERBUFFER_NOTLOCKED: D3D11_MESSAGE_ID = 3145783i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DECODERBEGINFRAME_NULLPARAM: D3D11_MESSAGE_ID = 3145784i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DECODERBEGINFRAME_HAZARD: D3D11_MESSAGE_ID = 3145785i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DECODERENDFRAME_NULLPARAM: D3D11_MESSAGE_ID = 3145786i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_SUBMITDECODERBUFFERS_NULLPARAM: D3D11_MESSAGE_ID = 3145787i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_SUBMITDECODERBUFFERS_INVALIDTYPE: D3D11_MESSAGE_ID = 3145788i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DECODEREXTENSION_NULLPARAM: D3D11_MESSAGE_ID = 3145789i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DECODEREXTENSION_INVALIDRESOURCE: D3D11_MESSAGE_ID = 3145790i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSORENUMERATOR_OUTOFMEMORY_RETURN: D3D11_MESSAGE_ID = 3145791i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSORENUMERATOR_NULLPARAM: D3D11_MESSAGE_ID = 3145792i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSORENUMERATOR_INVALIDFRAMEFORMAT: D3D11_MESSAGE_ID = 3145793i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSORENUMERATOR_INVALIDUSAGE: D3D11_MESSAGE_ID = 3145794i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSORENUMERATOR_INVALIDINPUTFRAMERATE: D3D11_MESSAGE_ID = 3145795i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSORENUMERATOR_INVALIDOUTPUTFRAMERATE: D3D11_MESSAGE_ID = 3145796i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSORENUMERATOR_INVALIDWIDTHHEIGHT: D3D11_MESSAGE_ID = 3145797i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETVIDEOPROCESSORCONTENTDESC_NULLPARAM: D3D11_MESSAGE_ID = 3145798i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CHECKVIDEOPROCESSORFORMAT_NULLPARAM: D3D11_MESSAGE_ID = 3145799i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETVIDEOPROCESSORCAPS_NULLPARAM: D3D11_MESSAGE_ID = 3145800i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETVIDEOPROCESSORRATECONVERSIONCAPS_NULLPARAM: D3D11_MESSAGE_ID = 3145801i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETVIDEOPROCESSORRATECONVERSIONCAPS_INVALIDINDEX: D3D11_MESSAGE_ID = 3145802i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETVIDEOPROCESSORCUSTOMRATE_NULLPARAM: D3D11_MESSAGE_ID = 3145803i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETVIDEOPROCESSORCUSTOMRATE_INVALIDINDEX: D3D11_MESSAGE_ID = 3145804i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETVIDEOPROCESSORFILTERRANGE_NULLPARAM: D3D11_MESSAGE_ID = 3145805i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETVIDEOPROCESSORFILTERRANGE_UNSUPPORTED: D3D11_MESSAGE_ID = 3145806i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSOR_OUTOFMEMORY_RETURN: D3D11_MESSAGE_ID = 3145807i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSOR_NULLPARAM: D3D11_MESSAGE_ID = 3145808i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETOUTPUTTARGETRECT_NULLPARAM: D3D11_MESSAGE_ID = 3145809i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETOUTPUTBACKGROUNDCOLOR_NULLPARAM: D3D11_MESSAGE_ID = 3145810i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETOUTPUTBACKGROUNDCOLOR_INVALIDALPHA: D3D11_MESSAGE_ID = 3145811i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETOUTPUTCOLORSPACE_NULLPARAM: D3D11_MESSAGE_ID = 3145812i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETOUTPUTALPHAFILLMODE_NULLPARAM: D3D11_MESSAGE_ID = 3145813i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETOUTPUTALPHAFILLMODE_UNSUPPORTED: D3D11_MESSAGE_ID = 3145814i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETOUTPUTALPHAFILLMODE_INVALIDSTREAM: D3D11_MESSAGE_ID = 3145815i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETOUTPUTALPHAFILLMODE_INVALIDFILLMODE: D3D11_MESSAGE_ID = 3145816i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETOUTPUTCONSTRICTION_NULLPARAM: D3D11_MESSAGE_ID = 3145817i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETOUTPUTSTEREOMODE_NULLPARAM: D3D11_MESSAGE_ID = 3145818i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETOUTPUTSTEREOMODE_UNSUPPORTED: D3D11_MESSAGE_ID = 3145819i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETOUTPUTEXTENSION_NULLPARAM: D3D11_MESSAGE_ID = 3145820i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETOUTPUTTARGETRECT_NULLPARAM: D3D11_MESSAGE_ID = 3145821i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETOUTPUTBACKGROUNDCOLOR_NULLPARAM: D3D11_MESSAGE_ID = 3145822i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETOUTPUTCOLORSPACE_NULLPARAM: D3D11_MESSAGE_ID = 3145823i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETOUTPUTALPHAFILLMODE_NULLPARAM: D3D11_MESSAGE_ID = 3145824i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETOUTPUTCONSTRICTION_NULLPARAM: D3D11_MESSAGE_ID = 3145825i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETOUTPUTCONSTRICTION_UNSUPPORTED: D3D11_MESSAGE_ID = 3145826i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETOUTPUTCONSTRICTION_INVALIDSIZE: D3D11_MESSAGE_ID = 3145827i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETOUTPUTSTEREOMODE_NULLPARAM: D3D11_MESSAGE_ID = 3145828i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETOUTPUTEXTENSION_NULLPARAM: D3D11_MESSAGE_ID = 3145829i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMFRAMEFORMAT_NULLPARAM: D3D11_MESSAGE_ID = 3145830i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMFRAMEFORMAT_INVALIDFORMAT: D3D11_MESSAGE_ID = 3145831i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMFRAMEFORMAT_INVALIDSTREAM: D3D11_MESSAGE_ID = 3145832i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMCOLORSPACE_NULLPARAM: D3D11_MESSAGE_ID = 3145833i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMCOLORSPACE_INVALIDSTREAM: D3D11_MESSAGE_ID = 3145834i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMOUTPUTRATE_NULLPARAM: D3D11_MESSAGE_ID = 3145835i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMOUTPUTRATE_INVALIDRATE: D3D11_MESSAGE_ID = 3145836i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMOUTPUTRATE_INVALIDFLAG: D3D11_MESSAGE_ID = 3145837i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMOUTPUTRATE_INVALIDSTREAM: D3D11_MESSAGE_ID = 3145838i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMSOURCERECT_NULLPARAM: D3D11_MESSAGE_ID = 3145839i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMSOURCERECT_INVALIDSTREAM: D3D11_MESSAGE_ID = 3145840i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMSOURCERECT_INVALIDRECT: D3D11_MESSAGE_ID = 3145841i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMDESTRECT_NULLPARAM: D3D11_MESSAGE_ID = 3145842i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMDESTRECT_INVALIDSTREAM: D3D11_MESSAGE_ID = 3145843i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMDESTRECT_INVALIDRECT: D3D11_MESSAGE_ID = 3145844i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMALPHA_NULLPARAM: D3D11_MESSAGE_ID = 3145845i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMALPHA_INVALIDSTREAM: D3D11_MESSAGE_ID = 3145846i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMALPHA_INVALIDALPHA: D3D11_MESSAGE_ID = 3145847i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMPALETTE_NULLPARAM: D3D11_MESSAGE_ID = 3145848i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMPALETTE_INVALIDSTREAM: D3D11_MESSAGE_ID = 3145849i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMPALETTE_INVALIDCOUNT: D3D11_MESSAGE_ID = 3145850i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMPALETTE_INVALIDALPHA: D3D11_MESSAGE_ID = 3145851i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMPIXELASPECTRATIO_NULLPARAM: D3D11_MESSAGE_ID = 3145852i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMPIXELASPECTRATIO_INVALIDSTREAM: D3D11_MESSAGE_ID = 3145853i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMPIXELASPECTRATIO_INVALIDRATIO: D3D11_MESSAGE_ID = 3145854i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMLUMAKEY_NULLPARAM: D3D11_MESSAGE_ID = 3145855i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMLUMAKEY_INVALIDSTREAM: D3D11_MESSAGE_ID = 3145856i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMLUMAKEY_INVALIDRANGE: D3D11_MESSAGE_ID = 3145857i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMLUMAKEY_UNSUPPORTED: D3D11_MESSAGE_ID = 3145858i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMSTEREOFORMAT_NULLPARAM: D3D11_MESSAGE_ID = 3145859i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMSTEREOFORMAT_INVALIDSTREAM: D3D11_MESSAGE_ID = 3145860i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMSTEREOFORMAT_UNSUPPORTED: D3D11_MESSAGE_ID = 3145861i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMSTEREOFORMAT_FLIPUNSUPPORTED: D3D11_MESSAGE_ID = 3145862i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMSTEREOFORMAT_MONOOFFSETUNSUPPORTED: D3D11_MESSAGE_ID = 3145863i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMSTEREOFORMAT_FORMATUNSUPPORTED: D3D11_MESSAGE_ID = 3145864i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMSTEREOFORMAT_INVALIDFORMAT: D3D11_MESSAGE_ID = 3145865i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMAUTOPROCESSINGMODE_NULLPARAM: D3D11_MESSAGE_ID = 3145866i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMAUTOPROCESSINGMODE_INVALIDSTREAM: D3D11_MESSAGE_ID = 3145867i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMFILTER_NULLPARAM: D3D11_MESSAGE_ID = 3145868i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMFILTER_INVALIDSTREAM: D3D11_MESSAGE_ID = 3145869i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMFILTER_INVALIDFILTER: D3D11_MESSAGE_ID = 3145870i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMFILTER_UNSUPPORTED: D3D11_MESSAGE_ID = 3145871i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMFILTER_INVALIDLEVEL: D3D11_MESSAGE_ID = 3145872i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMEXTENSION_NULLPARAM: D3D11_MESSAGE_ID = 3145873i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMEXTENSION_INVALIDSTREAM: D3D11_MESSAGE_ID = 3145874i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMFRAMEFORMAT_NULLPARAM: D3D11_MESSAGE_ID = 3145875i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMCOLORSPACE_NULLPARAM: D3D11_MESSAGE_ID = 3145876i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMOUTPUTRATE_NULLPARAM: D3D11_MESSAGE_ID = 3145877i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMSOURCERECT_NULLPARAM: D3D11_MESSAGE_ID = 3145878i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMDESTRECT_NULLPARAM: D3D11_MESSAGE_ID = 3145879i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMALPHA_NULLPARAM: D3D11_MESSAGE_ID = 3145880i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMPALETTE_NULLPARAM: D3D11_MESSAGE_ID = 3145881i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMPIXELASPECTRATIO_NULLPARAM: D3D11_MESSAGE_ID = 3145882i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMLUMAKEY_NULLPARAM: D3D11_MESSAGE_ID = 3145883i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMSTEREOFORMAT_NULLPARAM: D3D11_MESSAGE_ID = 3145884i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMAUTOPROCESSINGMODE_NULLPARAM: D3D11_MESSAGE_ID = 3145885i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMFILTER_NULLPARAM: D3D11_MESSAGE_ID = 3145886i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMEXTENSION_NULLPARAM: D3D11_MESSAGE_ID = 3145887i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMEXTENSION_INVALIDSTREAM: D3D11_MESSAGE_ID = 3145888i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORBLT_NULLPARAM: D3D11_MESSAGE_ID = 3145889i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORBLT_INVALIDSTREAMCOUNT: D3D11_MESSAGE_ID = 3145890i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORBLT_TARGETRECT: D3D11_MESSAGE_ID = 3145891i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORBLT_INVALIDOUTPUT: D3D11_MESSAGE_ID = 3145892i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORBLT_INVALIDPASTFRAMES: D3D11_MESSAGE_ID = 3145893i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORBLT_INVALIDFUTUREFRAMES: D3D11_MESSAGE_ID = 3145894i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORBLT_INVALIDSOURCERECT: D3D11_MESSAGE_ID = 3145895i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORBLT_INVALIDDESTRECT: D3D11_MESSAGE_ID = 3145896i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORBLT_INVALIDINPUTRESOURCE: D3D11_MESSAGE_ID = 3145897i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORBLT_INVALIDARRAYSIZE: D3D11_MESSAGE_ID = 3145898i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORBLT_INVALIDARRAY: D3D11_MESSAGE_ID = 3145899i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORBLT_RIGHTEXPECTED: D3D11_MESSAGE_ID = 3145900i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORBLT_RIGHTNOTEXPECTED: D3D11_MESSAGE_ID = 3145901i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORBLT_STEREONOTENABLED: D3D11_MESSAGE_ID = 3145902i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORBLT_INVALIDRIGHTRESOURCE: D3D11_MESSAGE_ID = 3145903i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORBLT_NOSTEREOSTREAMS: D3D11_MESSAGE_ID = 3145904i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORBLT_INPUTHAZARD: D3D11_MESSAGE_ID = 3145905i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORBLT_OUTPUTHAZARD: D3D11_MESSAGE_ID = 3145906i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEODECODEROUTPUTVIEW_OUTOFMEMORY_RETURN: D3D11_MESSAGE_ID = 3145907i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEODECODEROUTPUTVIEW_NULLPARAM: D3D11_MESSAGE_ID = 3145908i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEODECODEROUTPUTVIEW_INVALIDTYPE: D3D11_MESSAGE_ID = 3145909i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEODECODEROUTPUTVIEW_INVALIDBIND: D3D11_MESSAGE_ID = 3145910i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEODECODEROUTPUTVIEW_UNSUPPORTEDFORMAT: D3D11_MESSAGE_ID = 3145911i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEODECODEROUTPUTVIEW_INVALIDMIP: D3D11_MESSAGE_ID = 3145912i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEODECODEROUTPUTVIEW_UNSUPPORTEMIP: D3D11_MESSAGE_ID = 3145913i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEODECODEROUTPUTVIEW_INVALIDARRAYSIZE: D3D11_MESSAGE_ID = 3145914i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEODECODEROUTPUTVIEW_INVALIDARRAY: D3D11_MESSAGE_ID = 3145915i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEODECODEROUTPUTVIEW_INVALIDDIMENSION: D3D11_MESSAGE_ID = 3145916i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSORINPUTVIEW_OUTOFMEMORY_RETURN: D3D11_MESSAGE_ID = 3145917i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSORINPUTVIEW_NULLPARAM: D3D11_MESSAGE_ID = 3145918i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSORINPUTVIEW_INVALIDTYPE: D3D11_MESSAGE_ID = 3145919i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSORINPUTVIEW_INVALIDBIND: D3D11_MESSAGE_ID = 3145920i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSORINPUTVIEW_INVALIDMISC: D3D11_MESSAGE_ID = 3145921i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSORINPUTVIEW_INVALIDUSAGE: D3D11_MESSAGE_ID = 3145922i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSORINPUTVIEW_INVALIDFORMAT: D3D11_MESSAGE_ID = 3145923i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSORINPUTVIEW_INVALIDFOURCC: D3D11_MESSAGE_ID = 3145924i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSORINPUTVIEW_INVALIDMIP: D3D11_MESSAGE_ID = 3145925i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSORINPUTVIEW_UNSUPPORTEDMIP: D3D11_MESSAGE_ID = 3145926i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSORINPUTVIEW_INVALIDARRAYSIZE: D3D11_MESSAGE_ID = 3145927i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSORINPUTVIEW_INVALIDARRAY: D3D11_MESSAGE_ID = 3145928i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSORINPUTVIEW_INVALIDDIMENSION: D3D11_MESSAGE_ID = 3145929i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSOROUTPUTVIEW_OUTOFMEMORY_RETURN: D3D11_MESSAGE_ID = 3145930i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSOROUTPUTVIEW_NULLPARAM: D3D11_MESSAGE_ID = 3145931i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSOROUTPUTVIEW_INVALIDTYPE: D3D11_MESSAGE_ID = 3145932i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSOROUTPUTVIEW_INVALIDBIND: D3D11_MESSAGE_ID = 3145933i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSOROUTPUTVIEW_INVALIDFORMAT: D3D11_MESSAGE_ID = 3145934i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSOROUTPUTVIEW_INVALIDMIP: D3D11_MESSAGE_ID = 3145935i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSOROUTPUTVIEW_UNSUPPORTEDMIP: D3D11_MESSAGE_ID = 3145936i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSOROUTPUTVIEW_UNSUPPORTEDARRAY: D3D11_MESSAGE_ID = 3145937i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSOROUTPUTVIEW_INVALIDARRAY: D3D11_MESSAGE_ID = 3145938i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSOROUTPUTVIEW_INVALIDDIMENSION: D3D11_MESSAGE_ID = 3145939i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_INVALID_USE_OF_FORCED_SAMPLE_COUNT: D3D11_MESSAGE_ID = 3145940i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBLENDSTATE_INVALIDLOGICOPS: D3D11_MESSAGE_ID = 3145941i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESHADERRESOURCEVIEW_INVALIDDARRAYWITHDECODER: D3D11_MESSAGE_ID = 3145942i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEUNORDEREDACCESSVIEW_INVALIDDARRAYWITHDECODER: D3D11_MESSAGE_ID = 3145943i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERENDERTARGETVIEW_INVALIDDARRAYWITHDECODER: D3D11_MESSAGE_ID = 3145944i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_LOCKEDOUT_INTERFACE: D3D11_MESSAGE_ID = 3145945i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_REF_WARNING_ATOMIC_INCONSISTENT: D3D11_MESSAGE_ID = 3145946i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_REF_WARNING_READING_UNINITIALIZED_RESOURCE: D3D11_MESSAGE_ID = 3145947i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_REF_WARNING_RAW_HAZARD: D3D11_MESSAGE_ID = 3145948i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_REF_WARNING_WAR_HAZARD: D3D11_MESSAGE_ID = 3145949i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_REF_WARNING_WAW_HAZARD: D3D11_MESSAGE_ID = 3145950i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATECRYPTOSESSION_NULLPARAM: D3D11_MESSAGE_ID = 3145951i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATECRYPTOSESSION_OUTOFMEMORY_RETURN: D3D11_MESSAGE_ID = 3145952i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETCRYPTOTYPE_NULLPARAM: D3D11_MESSAGE_ID = 3145953i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETDECODERPROFILE_NULLPARAM: D3D11_MESSAGE_ID = 3145954i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETCRYPTOSESSIONCERTIFICATESIZE_NULLPARAM: D3D11_MESSAGE_ID = 3145955i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETCRYPTOSESSIONCERTIFICATE_NULLPARAM: D3D11_MESSAGE_ID = 3145956i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETCRYPTOSESSIONCERTIFICATE_WRONGSIZE: D3D11_MESSAGE_ID = 3145957i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETCRYPTOSESSIONHANDLE_WRONGSIZE: D3D11_MESSAGE_ID = 3145958i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_NEGOTIATECRPYTOSESSIONKEYEXCHANGE_NULLPARAM: D3D11_MESSAGE_ID = 3145959i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_ENCRYPTIONBLT_UNSUPPORTED: D3D11_MESSAGE_ID = 3145960i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_ENCRYPTIONBLT_NULLPARAM: D3D11_MESSAGE_ID = 3145961i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_ENCRYPTIONBLT_SRC_WRONGDEVICE: D3D11_MESSAGE_ID = 3145962i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_ENCRYPTIONBLT_DST_WRONGDEVICE: D3D11_MESSAGE_ID = 3145963i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_ENCRYPTIONBLT_FORMAT_MISMATCH: D3D11_MESSAGE_ID = 3145964i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_ENCRYPTIONBLT_SIZE_MISMATCH: D3D11_MESSAGE_ID = 3145965i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_ENCRYPTIONBLT_SRC_MULTISAMPLED: D3D11_MESSAGE_ID = 3145966i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_ENCRYPTIONBLT_DST_NOT_STAGING: D3D11_MESSAGE_ID = 3145967i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_ENCRYPTIONBLT_SRC_MAPPED: D3D11_MESSAGE_ID = 3145968i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_ENCRYPTIONBLT_DST_MAPPED: D3D11_MESSAGE_ID = 3145969i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_ENCRYPTIONBLT_SRC_OFFERED: D3D11_MESSAGE_ID = 3145970i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_ENCRYPTIONBLT_DST_OFFERED: D3D11_MESSAGE_ID = 3145971i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_ENCRYPTIONBLT_SRC_CONTENT_UNDEFINED: D3D11_MESSAGE_ID = 3145972i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DECRYPTIONBLT_UNSUPPORTED: D3D11_MESSAGE_ID = 3145973i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DECRYPTIONBLT_NULLPARAM: D3D11_MESSAGE_ID = 3145974i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DECRYPTIONBLT_SRC_WRONGDEVICE: D3D11_MESSAGE_ID = 3145975i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DECRYPTIONBLT_DST_WRONGDEVICE: D3D11_MESSAGE_ID = 3145976i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DECRYPTIONBLT_FORMAT_MISMATCH: D3D11_MESSAGE_ID = 3145977i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DECRYPTIONBLT_SIZE_MISMATCH: D3D11_MESSAGE_ID = 3145978i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DECRYPTIONBLT_DST_MULTISAMPLED: D3D11_MESSAGE_ID = 3145979i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DECRYPTIONBLT_SRC_NOT_STAGING: D3D11_MESSAGE_ID = 3145980i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DECRYPTIONBLT_DST_NOT_RENDER_TARGET: D3D11_MESSAGE_ID = 3145981i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DECRYPTIONBLT_SRC_MAPPED: D3D11_MESSAGE_ID = 3145982i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DECRYPTIONBLT_DST_MAPPED: D3D11_MESSAGE_ID = 3145983i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DECRYPTIONBLT_SRC_OFFERED: D3D11_MESSAGE_ID = 3145984i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DECRYPTIONBLT_DST_OFFERED: D3D11_MESSAGE_ID = 3145985i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DECRYPTIONBLT_SRC_CONTENT_UNDEFINED: D3D11_MESSAGE_ID = 3145986i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_STARTSESSIONKEYREFRESH_NULLPARAM: D3D11_MESSAGE_ID = 3145987i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_STARTSESSIONKEYREFRESH_INVALIDSIZE: D3D11_MESSAGE_ID = 3145988i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_FINISHSESSIONKEYREFRESH_NULLPARAM: D3D11_MESSAGE_ID = 3145989i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETENCRYPTIONBLTKEY_NULLPARAM: D3D11_MESSAGE_ID = 3145990i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETENCRYPTIONBLTKEY_INVALIDSIZE: D3D11_MESSAGE_ID = 3145991i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETCONTENTPROTECTIONCAPS_NULLPARAM: D3D11_MESSAGE_ID = 3145992i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CHECKCRYPTOKEYEXCHANGE_NULLPARAM: D3D11_MESSAGE_ID = 3145993i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CHECKCRYPTOKEYEXCHANGE_INVALIDINDEX: D3D11_MESSAGE_ID = 3145994i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEAUTHENTICATEDCHANNEL_NULLPARAM: D3D11_MESSAGE_ID = 3145995i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEAUTHENTICATEDCHANNEL_UNSUPPORTED: D3D11_MESSAGE_ID = 3145996i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEAUTHENTICATEDCHANNEL_INVALIDTYPE: D3D11_MESSAGE_ID = 3145997i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEAUTHENTICATEDCHANNEL_OUTOFMEMORY_RETURN: D3D11_MESSAGE_ID = 3145998i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETAUTHENTICATEDCHANNELCERTIFICATESIZE_INVALIDCHANNEL: D3D11_MESSAGE_ID = 3145999i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETAUTHENTICATEDCHANNELCERTIFICATESIZE_NULLPARAM: D3D11_MESSAGE_ID = 3146000i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETAUTHENTICATEDCHANNELCERTIFICATE_INVALIDCHANNEL: D3D11_MESSAGE_ID = 3146001i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETAUTHENTICATEDCHANNELCERTIFICATE_NULLPARAM: D3D11_MESSAGE_ID = 3146002i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETAUTHENTICATEDCHANNELCERTIFICATE_WRONGSIZE: D3D11_MESSAGE_ID = 3146003i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_NEGOTIATEAUTHENTICATEDCHANNELKEYEXCHANGE_INVALIDCHANNEL: D3D11_MESSAGE_ID = 3146004i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_NEGOTIATEAUTHENTICATEDCHANNELKEYEXCHANGE_NULLPARAM: D3D11_MESSAGE_ID = 3146005i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_QUERYAUTHENTICATEDCHANNEL_NULLPARAM: D3D11_MESSAGE_ID = 3146006i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_QUERYAUTHENTICATEDCHANNEL_WRONGCHANNEL: D3D11_MESSAGE_ID = 3146007i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_QUERYAUTHENTICATEDCHANNEL_UNSUPPORTEDQUERY: D3D11_MESSAGE_ID = 3146008i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_QUERYAUTHENTICATEDCHANNEL_WRONGSIZE: D3D11_MESSAGE_ID = 3146009i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_QUERYAUTHENTICATEDCHANNEL_INVALIDPROCESSINDEX: D3D11_MESSAGE_ID = 3146010i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CONFIGUREAUTHENTICATEDCHANNEL_NULLPARAM: D3D11_MESSAGE_ID = 3146011i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CONFIGUREAUTHENTICATEDCHANNEL_WRONGCHANNEL: D3D11_MESSAGE_ID = 3146012i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CONFIGUREAUTHENTICATEDCHANNEL_UNSUPPORTEDCONFIGURE: D3D11_MESSAGE_ID = 3146013i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CONFIGUREAUTHENTICATEDCHANNEL_WRONGSIZE: D3D11_MESSAGE_ID = 3146014i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CONFIGUREAUTHENTICATEDCHANNEL_INVALIDPROCESSIDTYPE: D3D11_MESSAGE_ID = 3146015i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VSSETCONSTANTBUFFERS_INVALIDBUFFEROFFSETORCOUNT: D3D11_MESSAGE_ID = 3146016i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DSSETCONSTANTBUFFERS_INVALIDBUFFEROFFSETORCOUNT: D3D11_MESSAGE_ID = 3146017i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_HSSETCONSTANTBUFFERS_INVALIDBUFFEROFFSETORCOUNT: D3D11_MESSAGE_ID = 3146018i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GSSETCONSTANTBUFFERS_INVALIDBUFFEROFFSETORCOUNT: D3D11_MESSAGE_ID = 3146019i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_PSSETCONSTANTBUFFERS_INVALIDBUFFEROFFSETORCOUNT: D3D11_MESSAGE_ID = 3146020i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CSSETCONSTANTBUFFERS_INVALIDBUFFEROFFSETORCOUNT: D3D11_MESSAGE_ID = 3146021i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_NEGOTIATECRPYTOSESSIONKEYEXCHANGE_INVALIDSIZE: D3D11_MESSAGE_ID = 3146022i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_NEGOTIATEAUTHENTICATEDCHANNELKEYEXCHANGE_INVALIDSIZE: D3D11_MESSAGE_ID = 3146023i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_OFFERRESOURCES_INVALIDPRIORITY: D3D11_MESSAGE_ID = 3146024i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETCRYPTOSESSIONHANDLE_OUTOFMEMORY: D3D11_MESSAGE_ID = 3146025i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_ACQUIREHANDLEFORCAPTURE_NULLPARAM: D3D11_MESSAGE_ID = 3146026i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_ACQUIREHANDLEFORCAPTURE_INVALIDTYPE: D3D11_MESSAGE_ID = 3146027i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_ACQUIREHANDLEFORCAPTURE_INVALIDBIND: D3D11_MESSAGE_ID = 3146028i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_ACQUIREHANDLEFORCAPTURE_INVALIDARRAY: D3D11_MESSAGE_ID = 3146029i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMROTATION_NULLPARAM: D3D11_MESSAGE_ID = 3146030i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMROTATION_INVALIDSTREAM: D3D11_MESSAGE_ID = 3146031i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMROTATION_INVALID: D3D11_MESSAGE_ID = 3146032i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMROTATION_UNSUPPORTED: D3D11_MESSAGE_ID = 3146033i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMROTATION_NULLPARAM: D3D11_MESSAGE_ID = 3146034i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CLEARVIEW_INVALIDVIEW: D3D11_MESSAGE_ID = 3146035i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CREATEVERTEXSHADER_DOUBLEEXTENSIONSNOTSUPPORTED: D3D11_MESSAGE_ID = 3146036i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CREATEVERTEXSHADER_SHADEREXTENSIONSNOTSUPPORTED: D3D11_MESSAGE_ID = 3146037i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CREATEHULLSHADER_DOUBLEEXTENSIONSNOTSUPPORTED: D3D11_MESSAGE_ID = 3146038i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CREATEHULLSHADER_SHADEREXTENSIONSNOTSUPPORTED: D3D11_MESSAGE_ID = 3146039i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CREATEDOMAINSHADER_DOUBLEEXTENSIONSNOTSUPPORTED: D3D11_MESSAGE_ID = 3146040i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CREATEDOMAINSHADER_SHADEREXTENSIONSNOTSUPPORTED: D3D11_MESSAGE_ID = 3146041i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CREATEGEOMETRYSHADER_DOUBLEEXTENSIONSNOTSUPPORTED: D3D11_MESSAGE_ID = 3146042i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CREATEGEOMETRYSHADER_SHADEREXTENSIONSNOTSUPPORTED: D3D11_MESSAGE_ID = 3146043i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_DOUBLEEXTENSIONSNOTSUPPORTED: D3D11_MESSAGE_ID = 3146044i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_SHADEREXTENSIONSNOTSUPPORTED: D3D11_MESSAGE_ID = 3146045i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CREATEPIXELSHADER_DOUBLEEXTENSIONSNOTSUPPORTED: D3D11_MESSAGE_ID = 3146046i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CREATEPIXELSHADER_SHADEREXTENSIONSNOTSUPPORTED: D3D11_MESSAGE_ID = 3146047i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CREATECOMPUTESHADER_DOUBLEEXTENSIONSNOTSUPPORTED: D3D11_MESSAGE_ID = 3146048i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CREATECOMPUTESHADER_SHADEREXTENSIONSNOTSUPPORTED: D3D11_MESSAGE_ID = 3146049i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_SHADER_LINKAGE_MINPRECISION: D3D11_MESSAGE_ID = 3146050i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMALPHA_UNSUPPORTED: D3D11_MESSAGE_ID = 3146051i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMPIXELASPECTRATIO_UNSUPPORTED: D3D11_MESSAGE_ID = 3146052i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CREATEVERTEXSHADER_UAVSNOTSUPPORTED: D3D11_MESSAGE_ID = 3146053i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CREATEHULLSHADER_UAVSNOTSUPPORTED: D3D11_MESSAGE_ID = 3146054i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CREATEDOMAINSHADER_UAVSNOTSUPPORTED: D3D11_MESSAGE_ID = 3146055i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CREATEGEOMETRYSHADER_UAVSNOTSUPPORTED: D3D11_MESSAGE_ID = 3146056i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CREATEGEOMETRYSHADERWITHSTREAMOUTPUT_UAVSNOTSUPPORTED: D3D11_MESSAGE_ID = 3146057i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CREATEPIXELSHADER_UAVSNOTSUPPORTED: D3D11_MESSAGE_ID = 3146058i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CREATECOMPUTESHADER_UAVSNOTSUPPORTED: D3D11_MESSAGE_ID = 3146059i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_OMSETRENDERTARGETSANDUNORDEREDACCESSVIEWS_INVALIDOFFSET: D3D11_MESSAGE_ID = 3146060i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_OMSETRENDERTARGETSANDUNORDEREDACCESSVIEWS_TOOMANYVIEWS: D3D11_MESSAGE_ID = 3146061i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CLEARVIEW_NOTSUPPORTED: D3D11_MESSAGE_ID = 3146062i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_SWAPDEVICECONTEXTSTATE_NOTSUPPORTED: D3D11_MESSAGE_ID = 3146063i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_UPDATESUBRESOURCE_PREFERUPDATESUBRESOURCE1: D3D11_MESSAGE_ID = 3146064i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETDC_INACCESSIBLE: D3D11_MESSAGE_ID = 3146065i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CLEARVIEW_INVALIDRECT: D3D11_MESSAGE_ID = 3146066i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_SAMPLE_MASK_IGNORED_ON_FL9: D3D11_MESSAGE_ID = 3146067i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_OPEN_SHARED_RESOURCE1_NOT_SUPPORTED: D3D11_MESSAGE_ID = 3146068i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_OPEN_SHARED_RESOURCE_BY_NAME_NOT_SUPPORTED: D3D11_MESSAGE_ID = 3146069i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_ENQUEUESETEVENT_NOT_SUPPORTED: D3D11_MESSAGE_ID = 3146070i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_OFFERRELEASE_NOT_SUPPORTED: D3D11_MESSAGE_ID = 3146071i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_OFFERRESOURCES_INACCESSIBLE: D3D11_MESSAGE_ID = 3146072i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSORINPUTVIEW_INVALIDMSAA: D3D11_MESSAGE_ID = 3146073i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEVIDEOPROCESSOROUTPUTVIEW_INVALIDMSAA: D3D11_MESSAGE_ID = 3146074i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CLEARVIEW_INVALIDSOURCERECT: D3D11_MESSAGE_ID = 3146075i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_CLEARVIEW_EMPTYRECT: D3D11_MESSAGE_ID = 3146076i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_UPDATESUBRESOURCE_EMPTYDESTBOX: D3D11_MESSAGE_ID = 3146077i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_COPYSUBRESOURCEREGION_EMPTYSOURCEBOX: D3D11_MESSAGE_ID = 3146078i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_OM_RENDER_TARGET_DOES_NOT_SUPPORT_LOGIC_OPS: D3D11_MESSAGE_ID = 3146079i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_DEPTHSTENCILVIEW_NOT_SET: D3D11_MESSAGE_ID = 3146080i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_RENDERTARGETVIEW_NOT_SET: D3D11_MESSAGE_ID = 3146081i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_RENDERTARGETVIEW_NOT_SET_DUE_TO_FLIP_PRESENT: D3D11_MESSAGE_ID = 3146082i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_UNORDEREDACCESSVIEW_NOT_SET_DUE_TO_FLIP_PRESENT: D3D11_MESSAGE_ID = 3146083i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETDATAFORNEWHARDWAREKEY_NULLPARAM: D3D11_MESSAGE_ID = 3146084i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CHECKCRYPTOSESSIONSTATUS_NULLPARAM: D3D11_MESSAGE_ID = 3146085i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETCRYPTOSESSIONPRIVATEDATASIZE_NULLPARAM: D3D11_MESSAGE_ID = 3146086i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETVIDEODECODERCAPS_NULLPARAM: D3D11_MESSAGE_ID = 3146087i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETVIDEODECODERCAPS_ZEROWIDTHHEIGHT: D3D11_MESSAGE_ID = 3146088i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CHECKVIDEODECODERDOWNSAMPLING_NULLPARAM: D3D11_MESSAGE_ID = 3146089i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CHECKVIDEODECODERDOWNSAMPLING_INVALIDCOLORSPACE: D3D11_MESSAGE_ID = 3146090i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CHECKVIDEODECODERDOWNSAMPLING_ZEROWIDTHHEIGHT: D3D11_MESSAGE_ID = 3146091i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEODECODERENABLEDOWNSAMPLING_NULLPARAM: D3D11_MESSAGE_ID = 3146092i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEODECODERENABLEDOWNSAMPLING_UNSUPPORTED: D3D11_MESSAGE_ID = 3146093i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEODECODERUPDATEDOWNSAMPLING_NULLPARAM: D3D11_MESSAGE_ID = 3146094i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEODECODERUPDATEDOWNSAMPLING_UNSUPPORTED: D3D11_MESSAGE_ID = 3146095i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CHECKVIDEOPROCESSORFORMATCONVERSION_NULLPARAM: D3D11_MESSAGE_ID = 3146096i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETOUTPUTCOLORSPACE1_NULLPARAM: D3D11_MESSAGE_ID = 3146097i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETOUTPUTCOLORSPACE1_NULLPARAM: D3D11_MESSAGE_ID = 3146098i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMCOLORSPACE1_NULLPARAM: D3D11_MESSAGE_ID = 3146099i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMCOLORSPACE1_INVALIDSTREAM: D3D11_MESSAGE_ID = 3146100i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMMIRROR_NULLPARAM: D3D11_MESSAGE_ID = 3146101i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMMIRROR_INVALIDSTREAM: D3D11_MESSAGE_ID = 3146102i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMMIRROR_UNSUPPORTED: D3D11_MESSAGE_ID = 3146103i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMCOLORSPACE1_NULLPARAM: D3D11_MESSAGE_ID = 3146104i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMMIRROR_NULLPARAM: D3D11_MESSAGE_ID = 3146105i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_RECOMMENDVIDEODECODERDOWNSAMPLING_NULLPARAM: D3D11_MESSAGE_ID = 3146106i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_RECOMMENDVIDEODECODERDOWNSAMPLING_INVALIDCOLORSPACE: D3D11_MESSAGE_ID = 3146107i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_RECOMMENDVIDEODECODERDOWNSAMPLING_ZEROWIDTHHEIGHT: D3D11_MESSAGE_ID = 3146108i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETOUTPUTSHADERUSAGE_NULLPARAM: D3D11_MESSAGE_ID = 3146109i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETOUTPUTSHADERUSAGE_NULLPARAM: D3D11_MESSAGE_ID = 3146110i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETBEHAVIORHINTS_NULLPARAM: D3D11_MESSAGE_ID = 3146111i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETBEHAVIORHINTS_INVALIDSTREAMCOUNT: D3D11_MESSAGE_ID = 3146112i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETBEHAVIORHINTS_TARGETRECT: D3D11_MESSAGE_ID = 3146113i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETBEHAVIORHINTS_INVALIDSOURCERECT: D3D11_MESSAGE_ID = 3146114i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETBEHAVIORHINTS_INVALIDDESTRECT: D3D11_MESSAGE_ID = 3146115i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETCRYPTOSESSIONPRIVATEDATASIZE_INVALID_KEY_EXCHANGE_TYPE: D3D11_MESSAGE_ID = 3146116i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_D3D11_1_MESSAGES_END: D3D11_MESSAGE_ID = 3146117i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_D3D11_2_MESSAGES_START: D3D11_MESSAGE_ID = 3146118i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEBUFFER_INVALIDUSAGE: D3D11_MESSAGE_ID = 3146119i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE1D_INVALIDUSAGE: D3D11_MESSAGE_ID = 3146120i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATETEXTURE2D_INVALIDUSAGE: D3D11_MESSAGE_ID = 3146121i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEINPUTLAYOUT_LEVEL9_STEPRATE_NOT_1: D3D11_MESSAGE_ID = 3146122i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEINPUTLAYOUT_LEVEL9_INSTANCING_NOT_SUPPORTED: D3D11_MESSAGE_ID = 3146123i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_UPDATETILEMAPPINGS_INVALID_PARAMETER: D3D11_MESSAGE_ID = 3146124i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_COPYTILEMAPPINGS_INVALID_PARAMETER: D3D11_MESSAGE_ID = 3146125i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_COPYTILES_INVALID_PARAMETER: D3D11_MESSAGE_ID = 3146126i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_UPDATETILES_INVALID_PARAMETER: D3D11_MESSAGE_ID = 3146127i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_RESIZETILEPOOL_INVALID_PARAMETER: D3D11_MESSAGE_ID = 3146128i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TILEDRESOURCEBARRIER_INVALID_PARAMETER: D3D11_MESSAGE_ID = 3146129i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_NULL_TILE_MAPPING_ACCESS_WARNING: D3D11_MESSAGE_ID = 3146130i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_NULL_TILE_MAPPING_ACCESS_ERROR: D3D11_MESSAGE_ID = 3146131i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DIRTY_TILE_MAPPING_ACCESS: D3D11_MESSAGE_ID = 3146132i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DUPLICATE_TILE_MAPPINGS_IN_COVERED_AREA: D3D11_MESSAGE_ID = 3146133i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TILE_MAPPINGS_IN_COVERED_AREA_DUPLICATED_OUTSIDE: D3D11_MESSAGE_ID = 3146134i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TILE_MAPPINGS_SHARED_BETWEEN_INCOMPATIBLE_RESOURCES: D3D11_MESSAGE_ID = 3146135i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TILE_MAPPINGS_SHARED_BETWEEN_INPUT_AND_OUTPUT: D3D11_MESSAGE_ID = 3146136i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CHECKMULTISAMPLEQUALITYLEVELS_INVALIDFLAGS: D3D11_MESSAGE_ID = 3146137i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_GETRESOURCETILING_NONTILED_RESOURCE: D3D11_MESSAGE_ID = 3146138i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_RESIZETILEPOOL_SHRINK_WITH_MAPPINGS_STILL_DEFINED_PAST_END: D3D11_MESSAGE_ID = 3146139i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_NEED_TO_CALL_TILEDRESOURCEBARRIER: D3D11_MESSAGE_ID = 3146140i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEVICE_INVALIDARGS: D3D11_MESSAGE_ID = 3146141i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEDEVICE_WARNING: D3D11_MESSAGE_ID = 3146142i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CLEARUNORDEREDACCESSVIEWUINT_HAZARD: D3D11_MESSAGE_ID = 3146143i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CLEARUNORDEREDACCESSVIEWFLOAT_HAZARD: D3D11_MESSAGE_ID = 3146144i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TILED_RESOURCE_TIER_1_BUFFER_TEXTURE_MISMATCH: D3D11_MESSAGE_ID = 3146145i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_CRYPTOSESSION: D3D11_MESSAGE_ID = 3146146i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_AUTHENTICATEDCHANNEL: D3D11_MESSAGE_ID = 3146147i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_CRYPTOSESSION: D3D11_MESSAGE_ID = 3146148i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_AUTHENTICATEDCHANNEL: D3D11_MESSAGE_ID = 3146149i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_CRYPTOSESSION: D3D11_MESSAGE_ID = 3146150i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_AUTHENTICATEDCHANNEL: D3D11_MESSAGE_ID = 3146151i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_D3D11_2_MESSAGES_END: D3D11_MESSAGE_ID = 3146152i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_D3D11_3_MESSAGES_START: D3D11_MESSAGE_ID = 3146153i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERASTERIZERSTATE_INVALID_CONSERVATIVERASTERMODE: D3D11_MESSAGE_ID = 3146154i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_DRAW_INVALID_SYSTEMVALUE: D3D11_MESSAGE_ID = 3146155i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEQUERYORPREDICATE_INVALIDCONTEXTTYPE: D3D11_MESSAGE_ID = 3146156i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEQUERYORPREDICATE_DECODENOTSUPPORTED: D3D11_MESSAGE_ID = 3146157i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEQUERYORPREDICATE_ENCODENOTSUPPORTED: D3D11_MESSAGE_ID = 3146158i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESHADERRESOURCEVIEW_INVALIDPLANEINDEX: D3D11_MESSAGE_ID = 3146159i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESHADERRESOURCEVIEW_INVALIDVIDEOPLANEINDEX: D3D11_MESSAGE_ID = 3146160i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATESHADERRESOURCEVIEW_AMBIGUOUSVIDEOPLANEINDEX: D3D11_MESSAGE_ID = 3146161i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERENDERTARGETVIEW_INVALIDPLANEINDEX: D3D11_MESSAGE_ID = 3146162i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERENDERTARGETVIEW_INVALIDVIDEOPLANEINDEX: D3D11_MESSAGE_ID = 3146163i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATERENDERTARGETVIEW_AMBIGUOUSVIDEOPLANEINDEX: D3D11_MESSAGE_ID = 3146164i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEUNORDEREDACCESSVIEW_INVALIDPLANEINDEX: D3D11_MESSAGE_ID = 3146165i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEUNORDEREDACCESSVIEW_INVALIDVIDEOPLANEINDEX: D3D11_MESSAGE_ID = 3146166i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEUNORDEREDACCESSVIEW_AMBIGUOUSVIDEOPLANEINDEX: D3D11_MESSAGE_ID = 3146167i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_INVALIDSCANDATAOFFSET: D3D11_MESSAGE_ID = 3146168i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_NOTSUPPORTED: D3D11_MESSAGE_ID = 3146169i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_DIMENSIONSTOOLARGE: D3D11_MESSAGE_ID = 3146170i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_INVALIDCOMPONENTS: D3D11_MESSAGE_ID = 3146171i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_DESTINATIONNOT2D: D3D11_MESSAGE_ID = 3146172i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_TILEDRESOURCESUNSUPPORTED: D3D11_MESSAGE_ID = 3146173i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_GUARDRECTSUNSUPPORTED: D3D11_MESSAGE_ID = 3146174i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_FORMATUNSUPPORTED: D3D11_MESSAGE_ID = 3146175i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_INVALIDSUBRESOURCE: D3D11_MESSAGE_ID = 3146176i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_INVALIDMIPLEVEL: D3D11_MESSAGE_ID = 3146177i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_EMPTYDESTBOX: D3D11_MESSAGE_ID = 3146178i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_DESTBOXNOT2D: D3D11_MESSAGE_ID = 3146179i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_DESTBOXNOTSUB: D3D11_MESSAGE_ID = 3146180i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_DESTBOXESINTERSECT: D3D11_MESSAGE_ID = 3146181i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_XSUBSAMPLEMISMATCH: D3D11_MESSAGE_ID = 3146182i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_YSUBSAMPLEMISMATCH: D3D11_MESSAGE_ID = 3146183i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_XSUBSAMPLEODD: D3D11_MESSAGE_ID = 3146184i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_YSUBSAMPLEODD: D3D11_MESSAGE_ID = 3146185i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_OUTPUTDIMENSIONSTOOLARGE: D3D11_MESSAGE_ID = 3146186i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_NONPOW2SCALEUNSUPPORTED: D3D11_MESSAGE_ID = 3146187i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_FRACTIONALDOWNSCALETOLARGE: D3D11_MESSAGE_ID = 3146188i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_CHROMASIZEMISMATCH: D3D11_MESSAGE_ID = 3146189i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_LUMACHROMASIZEMISMATCH: D3D11_MESSAGE_ID = 3146190i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_INVALIDNUMDESTINATIONS: D3D11_MESSAGE_ID = 3146191i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_SUBBOXUNSUPPORTED: D3D11_MESSAGE_ID = 3146192i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_1DESTUNSUPPORTEDFORMAT: D3D11_MESSAGE_ID = 3146193i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_3DESTUNSUPPORTEDFORMAT: D3D11_MESSAGE_ID = 3146194i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_SCALEUNSUPPORTED: D3D11_MESSAGE_ID = 3146195i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_INVALIDSOURCESIZE: D3D11_MESSAGE_ID = 3146196i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_INVALIDCOPYFLAGS: D3D11_MESSAGE_ID = 3146197i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_HAZARD: D3D11_MESSAGE_ID = 3146198i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_UNSUPPORTEDSRCBUFFERUSAGE: D3D11_MESSAGE_ID = 3146199i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_UNSUPPORTEDSRCBUFFERMISCFLAGS: D3D11_MESSAGE_ID = 3146200i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_UNSUPPORTEDDSTTEXTUREUSAGE: D3D11_MESSAGE_ID = 3146201i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_BACKBUFFERNOTSUPPORTED: D3D11_MESSAGE_ID = 3146202i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGDECODE_UNSUPPRTEDCOPYFLAGS: D3D11_MESSAGE_ID = 3146203i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGENCODE_NOTSUPPORTED: D3D11_MESSAGE_ID = 3146204i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGENCODE_INVALIDSCANDATAOFFSET: D3D11_MESSAGE_ID = 3146205i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGENCODE_INVALIDCOMPONENTS: D3D11_MESSAGE_ID = 3146206i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGENCODE_SOURCENOT2D: D3D11_MESSAGE_ID = 3146207i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGENCODE_TILEDRESOURCESUNSUPPORTED: D3D11_MESSAGE_ID = 3146208i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGENCODE_GUARDRECTSUNSUPPORTED: D3D11_MESSAGE_ID = 3146209i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGENCODE_XSUBSAMPLEMISMATCH: D3D11_MESSAGE_ID = 3146210i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGENCODE_YSUBSAMPLEMISMATCH: D3D11_MESSAGE_ID = 3146211i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGENCODE_FORMATUNSUPPORTED: D3D11_MESSAGE_ID = 3146212i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGENCODE_INVALIDSUBRESOURCE: D3D11_MESSAGE_ID = 3146213i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGENCODE_INVALIDMIPLEVEL: D3D11_MESSAGE_ID = 3146214i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGENCODE_DIMENSIONSTOOLARGE: D3D11_MESSAGE_ID = 3146215i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGENCODE_HAZARD: D3D11_MESSAGE_ID = 3146216i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGENCODE_UNSUPPORTEDDSTBUFFERUSAGE: D3D11_MESSAGE_ID = 3146217i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGENCODE_UNSUPPORTEDDSTBUFFERMISCFLAGS: D3D11_MESSAGE_ID = 3146218i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGENCODE_UNSUPPORTEDSRCTEXTUREUSAGE: D3D11_MESSAGE_ID = 3146219i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_JPEGENCODE_BACKBUFFERNOTSUPPORTED: D3D11_MESSAGE_ID = 3146220i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEQUERYORPREDICATE_UNSUPPORTEDCONTEXTTTYPEFORQUERY: D3D11_MESSAGE_ID = 3146221i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_FLUSH1_INVALIDCONTEXTTYPE: D3D11_MESSAGE_ID = 3146222i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DEVICE_SETHARDWAREPROTECTION_INVALIDCONTEXT: D3D11_MESSAGE_ID = 3146223i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETOUTPUTHDRMETADATA_NULLPARAM: D3D11_MESSAGE_ID = 3146224i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETOUTPUTHDRMETADATA_INVALIDSIZE: D3D11_MESSAGE_ID = 3146225i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETOUTPUTHDRMETADATA_NULLPARAM: D3D11_MESSAGE_ID = 3146226i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETOUTPUTHDRMETADATA_INVALIDSIZE: D3D11_MESSAGE_ID = 3146227i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMHDRMETADATA_NULLPARAM: D3D11_MESSAGE_ID = 3146228i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMHDRMETADATA_INVALIDSTREAM: D3D11_MESSAGE_ID = 3146229i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORSETSTREAMHDRMETADATA_INVALIDSIZE: D3D11_MESSAGE_ID = 3146230i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMHDRMETADATA_NULLPARAM: D3D11_MESSAGE_ID = 3146231i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMHDRMETADATA_INVALIDSTREAM: D3D11_MESSAGE_ID = 3146232i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMHDRMETADATA_INVALIDSIZE: D3D11_MESSAGE_ID = 3146233i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMFRAMEFORMAT_INVALIDSTREAM: D3D11_MESSAGE_ID = 3146234i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMCOLORSPACE_INVALIDSTREAM: D3D11_MESSAGE_ID = 3146235i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMOUTPUTRATE_INVALIDSTREAM: D3D11_MESSAGE_ID = 3146236i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMSOURCERECT_INVALIDSTREAM: D3D11_MESSAGE_ID = 3146237i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMDESTRECT_INVALIDSTREAM: D3D11_MESSAGE_ID = 3146238i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMALPHA_INVALIDSTREAM: D3D11_MESSAGE_ID = 3146239i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMPALETTE_INVALIDSTREAM: D3D11_MESSAGE_ID = 3146240i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMPIXELASPECTRATIO_INVALIDSTREAM: D3D11_MESSAGE_ID = 3146241i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMLUMAKEY_INVALIDSTREAM: D3D11_MESSAGE_ID = 3146242i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMSTEREOFORMAT_INVALIDSTREAM: D3D11_MESSAGE_ID = 3146243i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMAUTOPROCESSINGMODE_INVALIDSTREAM: D3D11_MESSAGE_ID = 3146244i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMFILTER_INVALIDSTREAM: D3D11_MESSAGE_ID = 3146245i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMROTATION_INVALIDSTREAM: D3D11_MESSAGE_ID = 3146246i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMCOLORSPACE1_INVALIDSTREAM: D3D11_MESSAGE_ID = 3146247i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_VIDEOPROCESSORGETSTREAMMIRROR_INVALIDSTREAM: D3D11_MESSAGE_ID = 3146248i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_FENCE: D3D11_MESSAGE_ID = 3146249i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_FENCE: D3D11_MESSAGE_ID = 3146250i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_FENCE: D3D11_MESSAGE_ID = 3146251i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_SYNCHRONIZEDCHANNEL: D3D11_MESSAGE_ID = 3146252i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_SYNCHRONIZEDCHANNEL: D3D11_MESSAGE_ID = 3146253i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_SYNCHRONIZEDCHANNEL: D3D11_MESSAGE_ID = 3146254i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATEFENCE_INVALIDFLAGS: D3D11_MESSAGE_ID = 3146255i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_D3D11_3_MESSAGES_END: D3D11_MESSAGE_ID = 3146256i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_D3D11_5_MESSAGES_START: D3D11_MESSAGE_ID = 3146257i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_NEGOTIATECRYPTOSESSIONKEYEXCHANGEMT_INVALIDKEYEXCHANGETYPE: D3D11_MESSAGE_ID = 3146258i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_NEGOTIATECRYPTOSESSIONKEYEXCHANGEMT_NOT_SUPPORTED: D3D11_MESSAGE_ID = 3146259i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DECODERBEGINFRAME_INVALID_HISTOGRAM_COMPONENT_COUNT: D3D11_MESSAGE_ID = 3146260i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DECODERBEGINFRAME_INVALID_HISTOGRAM_COMPONENT: D3D11_MESSAGE_ID = 3146261i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DECODERBEGINFRAME_INVALID_HISTOGRAM_BUFFER_SIZE: D3D11_MESSAGE_ID = 3146262i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DECODERBEGINFRAME_INVALID_HISTOGRAM_BUFFER_USAGE: D3D11_MESSAGE_ID = 3146263i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DECODERBEGINFRAME_INVALID_HISTOGRAM_BUFFER_MISC_FLAGS: D3D11_MESSAGE_ID = 3146264i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DECODERBEGINFRAME_INVALID_HISTOGRAM_BUFFER_OFFSET: D3D11_MESSAGE_ID = 3146265i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_TRACKEDWORKLOAD: D3D11_MESSAGE_ID = 3146266i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_LIVE_TRACKEDWORKLOAD: D3D11_MESSAGE_ID = 3146267i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_DESTROY_TRACKEDWORKLOAD: D3D11_MESSAGE_ID = 3146268i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_TRACKED_WORKLOAD_NULLPARAM: D3D11_MESSAGE_ID = 3146269i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_TRACKED_WORKLOAD_INVALID_MAX_INSTANCES: D3D11_MESSAGE_ID = 3146270i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_TRACKED_WORKLOAD_INVALID_DEADLINE_TYPE: D3D11_MESSAGE_ID = 3146271i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CREATE_TRACKED_WORKLOAD_INVALID_ENGINE_TYPE: D3D11_MESSAGE_ID = 3146272i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_MULTIPLE_TRACKED_WORKLOADS: D3D11_MESSAGE_ID = 3146273i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_MULTIPLE_TRACKED_WORKLOAD_PAIRS: D3D11_MESSAGE_ID = 3146274i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_INCOMPLETE_TRACKED_WORKLOAD_PAIR: D3D11_MESSAGE_ID = 3146275i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_OUT_OF_ORDER_TRACKED_WORKLOAD_PAIR: D3D11_MESSAGE_ID = 3146276i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_CANNOT_ADD_TRACKED_WORKLOAD: D3D11_MESSAGE_ID = 3146277i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TRACKED_WORKLOAD_NOT_SUPPORTED: D3D11_MESSAGE_ID = 3146278i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TRACKED_WORKLOAD_ENGINE_TYPE_NOT_FOUND: D3D11_MESSAGE_ID = 3146279i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_NO_TRACKED_WORKLOAD_SLOT_AVAILABLE: D3D11_MESSAGE_ID = 3146280i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_END_TRACKED_WORKLOAD_INVALID_ARG: D3D11_MESSAGE_ID = 3146281i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_TRACKED_WORKLOAD_DISJOINT_FAILURE: D3D11_MESSAGE_ID = 3146282i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_ID_D3D11_5_MESSAGES_END: D3D11_MESSAGE_ID = 3146283i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_MESSAGE_SEVERITY = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_SEVERITY_CORRUPTION: D3D11_MESSAGE_SEVERITY = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_SEVERITY_ERROR: D3D11_MESSAGE_SEVERITY = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_SEVERITY_WARNING: D3D11_MESSAGE_SEVERITY = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_SEVERITY_INFO: D3D11_MESSAGE_SEVERITY = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_MESSAGE_SEVERITY_MESSAGE: D3D11_MESSAGE_SEVERITY = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_QUERY = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_QUERY_EVENT: D3D11_QUERY = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_QUERY_OCCLUSION: D3D11_QUERY = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_QUERY_TIMESTAMP: D3D11_QUERY = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_QUERY_TIMESTAMP_DISJOINT: D3D11_QUERY = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_QUERY_PIPELINE_STATISTICS: D3D11_QUERY = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_QUERY_OCCLUSION_PREDICATE: D3D11_QUERY = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_QUERY_SO_STATISTICS: D3D11_QUERY = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_QUERY_SO_OVERFLOW_PREDICATE: D3D11_QUERY = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_QUERY_SO_STATISTICS_STREAM0: D3D11_QUERY = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_QUERY_SO_OVERFLOW_PREDICATE_STREAM0: D3D11_QUERY = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_QUERY_SO_STATISTICS_STREAM1: D3D11_QUERY = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_QUERY_SO_OVERFLOW_PREDICATE_STREAM1: D3D11_QUERY = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_QUERY_SO_STATISTICS_STREAM2: D3D11_QUERY = 12i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_QUERY_SO_OVERFLOW_PREDICATE_STREAM2: D3D11_QUERY = 13i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_QUERY_SO_STATISTICS_STREAM3: D3D11_QUERY = 14i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_QUERY_SO_OVERFLOW_PREDICATE_STREAM3: D3D11_QUERY = 15i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_QUERY_MISC_FLAG = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_QUERY_MISC_PREDICATEHINT: D3D11_QUERY_MISC_FLAG = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_RAISE_FLAG = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RAISE_FLAG_DRIVER_INTERNAL_ERROR: D3D11_RAISE_FLAG = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_RESOURCE_DIMENSION = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RESOURCE_DIMENSION_UNKNOWN: D3D11_RESOURCE_DIMENSION = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RESOURCE_DIMENSION_BUFFER: D3D11_RESOURCE_DIMENSION = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RESOURCE_DIMENSION_TEXTURE1D: D3D11_RESOURCE_DIMENSION = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RESOURCE_DIMENSION_TEXTURE2D: D3D11_RESOURCE_DIMENSION = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RESOURCE_DIMENSION_TEXTURE3D: D3D11_RESOURCE_DIMENSION = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_RESOURCE_MISC_FLAG = u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RESOURCE_MISC_GENERATE_MIPS: D3D11_RESOURCE_MISC_FLAG = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RESOURCE_MISC_SHARED: D3D11_RESOURCE_MISC_FLAG = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RESOURCE_MISC_TEXTURECUBE: D3D11_RESOURCE_MISC_FLAG = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RESOURCE_MISC_DRAWINDIRECT_ARGS: D3D11_RESOURCE_MISC_FLAG = 16u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RESOURCE_MISC_BUFFER_ALLOW_RAW_VIEWS: D3D11_RESOURCE_MISC_FLAG = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RESOURCE_MISC_BUFFER_STRUCTURED: D3D11_RESOURCE_MISC_FLAG = 64u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RESOURCE_MISC_RESOURCE_CLAMP: D3D11_RESOURCE_MISC_FLAG = 128u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RESOURCE_MISC_SHARED_KEYEDMUTEX: D3D11_RESOURCE_MISC_FLAG = 256u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RESOURCE_MISC_GDI_COMPATIBLE: D3D11_RESOURCE_MISC_FLAG = 512u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RESOURCE_MISC_SHARED_NTHANDLE: D3D11_RESOURCE_MISC_FLAG = 2048u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RESOURCE_MISC_RESTRICTED_CONTENT: D3D11_RESOURCE_MISC_FLAG = 4096u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RESOURCE_MISC_RESTRICT_SHARED_RESOURCE: D3D11_RESOURCE_MISC_FLAG = 8192u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RESOURCE_MISC_RESTRICT_SHARED_RESOURCE_DRIVER: D3D11_RESOURCE_MISC_FLAG = 16384u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RESOURCE_MISC_GUARDED: D3D11_RESOURCE_MISC_FLAG = 32768u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RESOURCE_MISC_TILE_POOL: D3D11_RESOURCE_MISC_FLAG = 131072u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RESOURCE_MISC_TILED: D3D11_RESOURCE_MISC_FLAG = 262144u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RESOURCE_MISC_HW_PROTECTED: D3D11_RESOURCE_MISC_FLAG = 524288u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RESOURCE_MISC_SHARED_DISPLAYABLE: D3D11_RESOURCE_MISC_FLAG = 1048576u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RESOURCE_MISC_SHARED_EXCLUSIVE_WRITER: D3D11_RESOURCE_MISC_FLAG = 2097152u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_RLDO_FLAGS = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RLDO_SUMMARY: D3D11_RLDO_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RLDO_DETAIL: D3D11_RLDO_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RLDO_IGNORE_INTERNAL: D3D11_RLDO_FLAGS = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_RTV_DIMENSION = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RTV_DIMENSION_UNKNOWN: D3D11_RTV_DIMENSION = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RTV_DIMENSION_BUFFER: D3D11_RTV_DIMENSION = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RTV_DIMENSION_TEXTURE1D: D3D11_RTV_DIMENSION = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RTV_DIMENSION_TEXTURE1DARRAY: D3D11_RTV_DIMENSION = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RTV_DIMENSION_TEXTURE2D: D3D11_RTV_DIMENSION = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RTV_DIMENSION_TEXTURE2DARRAY: D3D11_RTV_DIMENSION = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RTV_DIMENSION_TEXTURE2DMS: D3D11_RTV_DIMENSION = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RTV_DIMENSION_TEXTURE2DMSARRAY: D3D11_RTV_DIMENSION = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_RTV_DIMENSION_TEXTURE3D: D3D11_RTV_DIMENSION = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_SHADER_CACHE_SUPPORT_FLAGS = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_CACHE_SUPPORT_NONE: D3D11_SHADER_CACHE_SUPPORT_FLAGS = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_CACHE_SUPPORT_AUTOMATIC_INPROC_CACHE: D3D11_SHADER_CACHE_SUPPORT_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_CACHE_SUPPORT_AUTOMATIC_DISK_CACHE: D3D11_SHADER_CACHE_SUPPORT_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_SHADER_MIN_PRECISION_SUPPORT = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_MIN_PRECISION_10_BIT: D3D11_SHADER_MIN_PRECISION_SUPPORT = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_MIN_PRECISION_16_BIT: D3D11_SHADER_MIN_PRECISION_SUPPORT = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_SHADER_TRACKING_OPTIONS = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_TRACKING_OPTION_IGNORE: D3D11_SHADER_TRACKING_OPTIONS = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_TRACKING_OPTION_TRACK_UNINITIALIZED: D3D11_SHADER_TRACKING_OPTIONS = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_TRACKING_OPTION_TRACK_RAW: D3D11_SHADER_TRACKING_OPTIONS = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_TRACKING_OPTION_TRACK_WAR: D3D11_SHADER_TRACKING_OPTIONS = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_TRACKING_OPTION_TRACK_WAW: D3D11_SHADER_TRACKING_OPTIONS = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_TRACKING_OPTION_ALLOW_SAME: D3D11_SHADER_TRACKING_OPTIONS = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_TRACKING_OPTION_TRACK_ATOMIC_CONSISTENCY: D3D11_SHADER_TRACKING_OPTIONS = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_TRACKING_OPTION_TRACK_RAW_ACROSS_THREADGROUPS: D3D11_SHADER_TRACKING_OPTIONS = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_TRACKING_OPTION_TRACK_WAR_ACROSS_THREADGROUPS: D3D11_SHADER_TRACKING_OPTIONS = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_TRACKING_OPTION_TRACK_WAW_ACROSS_THREADGROUPS: D3D11_SHADER_TRACKING_OPTIONS = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_TRACKING_OPTION_TRACK_ATOMIC_CONSISTENCY_ACROSS_THREADGROUPS: D3D11_SHADER_TRACKING_OPTIONS = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_TRACKING_OPTION_UAV_SPECIFIC_FLAGS: D3D11_SHADER_TRACKING_OPTIONS = 960i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_TRACKING_OPTION_ALL_HAZARDS: D3D11_SHADER_TRACKING_OPTIONS = 1006i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_TRACKING_OPTION_ALL_HAZARDS_ALLOWING_SAME: D3D11_SHADER_TRACKING_OPTIONS = 1022i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_TRACKING_OPTION_ALL_OPTIONS: D3D11_SHADER_TRACKING_OPTIONS = 1023i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_SHADER_TRACKING_RESOURCE_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_TRACKING_RESOURCE_TYPE_NONE: D3D11_SHADER_TRACKING_RESOURCE_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_TRACKING_RESOURCE_TYPE_UAV_DEVICEMEMORY: D3D11_SHADER_TRACKING_RESOURCE_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_TRACKING_RESOURCE_TYPE_NON_UAV_DEVICEMEMORY: D3D11_SHADER_TRACKING_RESOURCE_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_TRACKING_RESOURCE_TYPE_ALL_DEVICEMEMORY: D3D11_SHADER_TRACKING_RESOURCE_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_TRACKING_RESOURCE_TYPE_GROUPSHARED_MEMORY: D3D11_SHADER_TRACKING_RESOURCE_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_TRACKING_RESOURCE_TYPE_ALL_SHARED_MEMORY: D3D11_SHADER_TRACKING_RESOURCE_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_TRACKING_RESOURCE_TYPE_GROUPSHARED_NON_UAV: D3D11_SHADER_TRACKING_RESOURCE_TYPE = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHADER_TRACKING_RESOURCE_TYPE_ALL: D3D11_SHADER_TRACKING_RESOURCE_TYPE = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_SHADER_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VERTEX_SHADER: D3D11_SHADER_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_HULL_SHADER: D3D11_SHADER_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_DOMAIN_SHADER: D3D11_SHADER_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_GEOMETRY_SHADER: D3D11_SHADER_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_PIXEL_SHADER: D3D11_SHADER_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_COMPUTE_SHADER: D3D11_SHADER_TYPE = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_SHADER_VERSION_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHVER_PIXEL_SHADER: D3D11_SHADER_VERSION_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHVER_VERTEX_SHADER: D3D11_SHADER_VERSION_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHVER_GEOMETRY_SHADER: D3D11_SHADER_VERSION_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHVER_HULL_SHADER: D3D11_SHADER_VERSION_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHVER_DOMAIN_SHADER: D3D11_SHADER_VERSION_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHVER_COMPUTE_SHADER: D3D11_SHADER_VERSION_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHVER_RESERVED0: D3D11_SHADER_VERSION_TYPE = 65520i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_SHARED_RESOURCE_TIER = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHARED_RESOURCE_TIER_0: D3D11_SHARED_RESOURCE_TIER = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHARED_RESOURCE_TIER_1: D3D11_SHARED_RESOURCE_TIER = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHARED_RESOURCE_TIER_2: D3D11_SHARED_RESOURCE_TIER = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_SHARED_RESOURCE_TIER_3: D3D11_SHARED_RESOURCE_TIER = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_STANDARD_MULTISAMPLE_QUALITY_LEVELS = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_STANDARD_MULTISAMPLE_PATTERN: D3D11_STANDARD_MULTISAMPLE_QUALITY_LEVELS = -1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_CENTER_MULTISAMPLE_PATTERN: D3D11_STANDARD_MULTISAMPLE_QUALITY_LEVELS = -2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_STENCIL_OP = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_STENCIL_OP_KEEP: D3D11_STENCIL_OP = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_STENCIL_OP_ZERO: D3D11_STENCIL_OP = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_STENCIL_OP_REPLACE: D3D11_STENCIL_OP = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_STENCIL_OP_INCR_SAT: D3D11_STENCIL_OP = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_STENCIL_OP_DECR_SAT: D3D11_STENCIL_OP = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_STENCIL_OP_INVERT: D3D11_STENCIL_OP = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_STENCIL_OP_INCR: D3D11_STENCIL_OP = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_STENCIL_OP_DECR: D3D11_STENCIL_OP = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_TEXTURECUBE_FACE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TEXTURECUBE_FACE_POSITIVE_X: D3D11_TEXTURECUBE_FACE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TEXTURECUBE_FACE_NEGATIVE_X: D3D11_TEXTURECUBE_FACE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TEXTURECUBE_FACE_POSITIVE_Y: D3D11_TEXTURECUBE_FACE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TEXTURECUBE_FACE_NEGATIVE_Y: D3D11_TEXTURECUBE_FACE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TEXTURECUBE_FACE_POSITIVE_Z: D3D11_TEXTURECUBE_FACE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TEXTURECUBE_FACE_NEGATIVE_Z: D3D11_TEXTURECUBE_FACE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_TEXTURE_ADDRESS_MODE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TEXTURE_ADDRESS_WRAP: D3D11_TEXTURE_ADDRESS_MODE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TEXTURE_ADDRESS_MIRROR: D3D11_TEXTURE_ADDRESS_MODE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TEXTURE_ADDRESS_CLAMP: D3D11_TEXTURE_ADDRESS_MODE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TEXTURE_ADDRESS_BORDER: D3D11_TEXTURE_ADDRESS_MODE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TEXTURE_ADDRESS_MIRROR_ONCE: D3D11_TEXTURE_ADDRESS_MODE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_TEXTURE_LAYOUT = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TEXTURE_LAYOUT_UNDEFINED: D3D11_TEXTURE_LAYOUT = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TEXTURE_LAYOUT_ROW_MAJOR: D3D11_TEXTURE_LAYOUT = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TEXTURE_LAYOUT_64K_STANDARD_SWIZZLE: D3D11_TEXTURE_LAYOUT = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_TILED_RESOURCES_TIER = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TILED_RESOURCES_NOT_SUPPORTED: D3D11_TILED_RESOURCES_TIER = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TILED_RESOURCES_TIER_1: D3D11_TILED_RESOURCES_TIER = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TILED_RESOURCES_TIER_2: D3D11_TILED_RESOURCES_TIER = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TILED_RESOURCES_TIER_3: D3D11_TILED_RESOURCES_TIER = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_TILE_COPY_FLAG = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TILE_COPY_NO_OVERWRITE: D3D11_TILE_COPY_FLAG = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TILE_COPY_LINEAR_BUFFER_TO_SWIZZLED_TILED_RESOURCE: D3D11_TILE_COPY_FLAG = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TILE_COPY_SWIZZLED_TILED_RESOURCE_TO_LINEAR_BUFFER: D3D11_TILE_COPY_FLAG = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_TILE_MAPPING_FLAG = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TILE_MAPPING_NO_OVERWRITE: D3D11_TILE_MAPPING_FLAG = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_TILE_RANGE_FLAG = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TILE_RANGE_NULL: D3D11_TILE_RANGE_FLAG = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TILE_RANGE_SKIP: D3D11_TILE_RANGE_FLAG = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TILE_RANGE_REUSE_SINGLE_TILE: D3D11_TILE_RANGE_FLAG = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_TRACE_GS_INPUT_PRIMITIVE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_GS_INPUT_PRIMITIVE_UNDEFINED: D3D11_TRACE_GS_INPUT_PRIMITIVE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_GS_INPUT_PRIMITIVE_POINT: D3D11_TRACE_GS_INPUT_PRIMITIVE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_GS_INPUT_PRIMITIVE_LINE: D3D11_TRACE_GS_INPUT_PRIMITIVE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_GS_INPUT_PRIMITIVE_TRIANGLE: D3D11_TRACE_GS_INPUT_PRIMITIVE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_GS_INPUT_PRIMITIVE_LINE_ADJ: D3D11_TRACE_GS_INPUT_PRIMITIVE = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_GS_INPUT_PRIMITIVE_TRIANGLE_ADJ: D3D11_TRACE_GS_INPUT_PRIMITIVE = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_TRACE_REGISTER_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_OUTPUT_NULL_REGISTER: D3D11_TRACE_REGISTER_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_INPUT_REGISTER: D3D11_TRACE_REGISTER_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_INPUT_PRIMITIVE_ID_REGISTER: D3D11_TRACE_REGISTER_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_IMMEDIATE_CONSTANT_BUFFER: D3D11_TRACE_REGISTER_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_TEMP_REGISTER: D3D11_TRACE_REGISTER_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_INDEXABLE_TEMP_REGISTER: D3D11_TRACE_REGISTER_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_OUTPUT_REGISTER: D3D11_TRACE_REGISTER_TYPE = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_OUTPUT_DEPTH_REGISTER: D3D11_TRACE_REGISTER_TYPE = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_CONSTANT_BUFFER: D3D11_TRACE_REGISTER_TYPE = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_IMMEDIATE32: D3D11_TRACE_REGISTER_TYPE = 9i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_SAMPLER: D3D11_TRACE_REGISTER_TYPE = 10i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_RESOURCE: D3D11_TRACE_REGISTER_TYPE = 11i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_RASTERIZER: D3D11_TRACE_REGISTER_TYPE = 12i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_OUTPUT_COVERAGE_MASK: D3D11_TRACE_REGISTER_TYPE = 13i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_STREAM: D3D11_TRACE_REGISTER_TYPE = 14i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_THIS_POINTER: D3D11_TRACE_REGISTER_TYPE = 15i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_OUTPUT_CONTROL_POINT_ID_REGISTER: D3D11_TRACE_REGISTER_TYPE = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_INPUT_FORK_INSTANCE_ID_REGISTER: D3D11_TRACE_REGISTER_TYPE = 17i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_INPUT_JOIN_INSTANCE_ID_REGISTER: D3D11_TRACE_REGISTER_TYPE = 18i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_INPUT_CONTROL_POINT_REGISTER: D3D11_TRACE_REGISTER_TYPE = 19i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_OUTPUT_CONTROL_POINT_REGISTER: D3D11_TRACE_REGISTER_TYPE = 20i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_INPUT_PATCH_CONSTANT_REGISTER: D3D11_TRACE_REGISTER_TYPE = 21i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_INPUT_DOMAIN_POINT_REGISTER: D3D11_TRACE_REGISTER_TYPE = 22i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_UNORDERED_ACCESS_VIEW: D3D11_TRACE_REGISTER_TYPE = 23i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_THREAD_GROUP_SHARED_MEMORY: D3D11_TRACE_REGISTER_TYPE = 24i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_INPUT_THREAD_ID_REGISTER: D3D11_TRACE_REGISTER_TYPE = 25i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_INPUT_THREAD_GROUP_ID_REGISTER: D3D11_TRACE_REGISTER_TYPE = 26i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_INPUT_THREAD_ID_IN_GROUP_REGISTER: D3D11_TRACE_REGISTER_TYPE = 27i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_INPUT_COVERAGE_MASK_REGISTER: D3D11_TRACE_REGISTER_TYPE = 28i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_INPUT_THREAD_ID_IN_GROUP_FLATTENED_REGISTER: D3D11_TRACE_REGISTER_TYPE = 29i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_INPUT_GS_INSTANCE_ID_REGISTER: D3D11_TRACE_REGISTER_TYPE = 30i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_OUTPUT_DEPTH_GREATER_EQUAL_REGISTER: D3D11_TRACE_REGISTER_TYPE = 31i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_OUTPUT_DEPTH_LESS_EQUAL_REGISTER: D3D11_TRACE_REGISTER_TYPE = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_IMMEDIATE64: D3D11_TRACE_REGISTER_TYPE = 33i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_INPUT_CYCLE_COUNTER_REGISTER: D3D11_TRACE_REGISTER_TYPE = 34i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_TRACE_INTERFACE_POINTER: D3D11_TRACE_REGISTER_TYPE = 35i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_UAV_DIMENSION = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_UAV_DIMENSION_UNKNOWN: D3D11_UAV_DIMENSION = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_UAV_DIMENSION_BUFFER: D3D11_UAV_DIMENSION = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_UAV_DIMENSION_TEXTURE1D: D3D11_UAV_DIMENSION = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_UAV_DIMENSION_TEXTURE1DARRAY: D3D11_UAV_DIMENSION = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_UAV_DIMENSION_TEXTURE2D: D3D11_UAV_DIMENSION = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_UAV_DIMENSION_TEXTURE2DARRAY: D3D11_UAV_DIMENSION = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_UAV_DIMENSION_TEXTURE3D: D3D11_UAV_DIMENSION = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_USAGE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_USAGE_DEFAULT: D3D11_USAGE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_USAGE_IMMUTABLE: D3D11_USAGE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_USAGE_DYNAMIC: D3D11_USAGE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_USAGE_STAGING: D3D11_USAGE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_VDOV_DIMENSION = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VDOV_DIMENSION_UNKNOWN: D3D11_VDOV_DIMENSION = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VDOV_DIMENSION_TEXTURE2D: D3D11_VDOV_DIMENSION = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_VIDEO_DECODER_BUFFER_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_DECODER_BUFFER_PICTURE_PARAMETERS: D3D11_VIDEO_DECODER_BUFFER_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_DECODER_BUFFER_MACROBLOCK_CONTROL: D3D11_VIDEO_DECODER_BUFFER_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_DECODER_BUFFER_RESIDUAL_DIFFERENCE: D3D11_VIDEO_DECODER_BUFFER_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_DECODER_BUFFER_DEBLOCKING_CONTROL: D3D11_VIDEO_DECODER_BUFFER_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_DECODER_BUFFER_INVERSE_QUANTIZATION_MATRIX: D3D11_VIDEO_DECODER_BUFFER_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_DECODER_BUFFER_SLICE_CONTROL: D3D11_VIDEO_DECODER_BUFFER_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_DECODER_BUFFER_BITSTREAM: D3D11_VIDEO_DECODER_BUFFER_TYPE = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_DECODER_BUFFER_MOTION_VECTOR: D3D11_VIDEO_DECODER_BUFFER_TYPE = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_DECODER_BUFFER_FILM_GRAIN: D3D11_VIDEO_DECODER_BUFFER_TYPE = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_VIDEO_DECODER_CAPS = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_DECODER_CAPS_DOWNSAMPLE: D3D11_VIDEO_DECODER_CAPS = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_DECODER_CAPS_NON_REAL_TIME: D3D11_VIDEO_DECODER_CAPS = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_DECODER_CAPS_DOWNSAMPLE_DYNAMIC: D3D11_VIDEO_DECODER_CAPS = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_DECODER_CAPS_DOWNSAMPLE_REQUIRED: D3D11_VIDEO_DECODER_CAPS = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_DECODER_CAPS_UNSUPPORTED: D3D11_VIDEO_DECODER_CAPS = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT_Y: D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT_U: D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT_V: D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT_R: D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT_G: D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT_B: D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT_A: D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT_FLAGS = u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT_FLAG_NONE: D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT_FLAG_Y: D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT_FLAG_U: D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT_FLAG_V: D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT_FLAG_R: D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT_FLAG_G: D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT_FLAG_B: D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT_FLAG_A: D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_VIDEO_FRAME_FORMAT = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_FRAME_FORMAT_PROGRESSIVE: D3D11_VIDEO_FRAME_FORMAT = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_FRAME_FORMAT_INTERLACED_TOP_FIELD_FIRST: D3D11_VIDEO_FRAME_FORMAT = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_FRAME_FORMAT_INTERLACED_BOTTOM_FIELD_FIRST: D3D11_VIDEO_FRAME_FORMAT = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_VIDEO_PROCESSOR_ALPHA_FILL_MODE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_ALPHA_FILL_MODE_OPAQUE: D3D11_VIDEO_PROCESSOR_ALPHA_FILL_MODE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_ALPHA_FILL_MODE_BACKGROUND: D3D11_VIDEO_PROCESSOR_ALPHA_FILL_MODE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_ALPHA_FILL_MODE_DESTINATION: D3D11_VIDEO_PROCESSOR_ALPHA_FILL_MODE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_ALPHA_FILL_MODE_SOURCE_STREAM: D3D11_VIDEO_PROCESSOR_ALPHA_FILL_MODE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_VIDEO_PROCESSOR_AUTO_STREAM_CAPS = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_AUTO_STREAM_CAPS_DENOISE: D3D11_VIDEO_PROCESSOR_AUTO_STREAM_CAPS = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_AUTO_STREAM_CAPS_DERINGING: D3D11_VIDEO_PROCESSOR_AUTO_STREAM_CAPS = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_AUTO_STREAM_CAPS_EDGE_ENHANCEMENT: D3D11_VIDEO_PROCESSOR_AUTO_STREAM_CAPS = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_AUTO_STREAM_CAPS_COLOR_CORRECTION: D3D11_VIDEO_PROCESSOR_AUTO_STREAM_CAPS = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_AUTO_STREAM_CAPS_FLESH_TONE_MAPPING: D3D11_VIDEO_PROCESSOR_AUTO_STREAM_CAPS = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_AUTO_STREAM_CAPS_IMAGE_STABILIZATION: D3D11_VIDEO_PROCESSOR_AUTO_STREAM_CAPS = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_AUTO_STREAM_CAPS_SUPER_RESOLUTION: D3D11_VIDEO_PROCESSOR_AUTO_STREAM_CAPS = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_AUTO_STREAM_CAPS_ANAMORPHIC_SCALING: D3D11_VIDEO_PROCESSOR_AUTO_STREAM_CAPS = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_VIDEO_PROCESSOR_BEHAVIOR_HINTS = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_BEHAVIOR_HINT_MULTIPLANE_OVERLAY_ROTATION: D3D11_VIDEO_PROCESSOR_BEHAVIOR_HINTS = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_BEHAVIOR_HINT_MULTIPLANE_OVERLAY_RESIZE: D3D11_VIDEO_PROCESSOR_BEHAVIOR_HINTS = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_BEHAVIOR_HINT_MULTIPLANE_OVERLAY_COLOR_SPACE_CONVERSION: D3D11_VIDEO_PROCESSOR_BEHAVIOR_HINTS = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_BEHAVIOR_HINT_TRIPLE_BUFFER_OUTPUT: D3D11_VIDEO_PROCESSOR_BEHAVIOR_HINTS = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_VIDEO_PROCESSOR_DEVICE_CAPS = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_DEVICE_CAPS_LINEAR_SPACE: D3D11_VIDEO_PROCESSOR_DEVICE_CAPS = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_DEVICE_CAPS_xvYCC: D3D11_VIDEO_PROCESSOR_DEVICE_CAPS = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_DEVICE_CAPS_RGB_RANGE_CONVERSION: D3D11_VIDEO_PROCESSOR_DEVICE_CAPS = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_DEVICE_CAPS_YCbCr_MATRIX_CONVERSION: D3D11_VIDEO_PROCESSOR_DEVICE_CAPS = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_DEVICE_CAPS_NOMINAL_RANGE: D3D11_VIDEO_PROCESSOR_DEVICE_CAPS = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_VIDEO_PROCESSOR_FEATURE_CAPS = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FEATURE_CAPS_ALPHA_FILL: D3D11_VIDEO_PROCESSOR_FEATURE_CAPS = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FEATURE_CAPS_CONSTRICTION: D3D11_VIDEO_PROCESSOR_FEATURE_CAPS = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FEATURE_CAPS_LUMA_KEY: D3D11_VIDEO_PROCESSOR_FEATURE_CAPS = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FEATURE_CAPS_ALPHA_PALETTE: D3D11_VIDEO_PROCESSOR_FEATURE_CAPS = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FEATURE_CAPS_LEGACY: D3D11_VIDEO_PROCESSOR_FEATURE_CAPS = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FEATURE_CAPS_STEREO: D3D11_VIDEO_PROCESSOR_FEATURE_CAPS = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FEATURE_CAPS_ROTATION: D3D11_VIDEO_PROCESSOR_FEATURE_CAPS = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FEATURE_CAPS_ALPHA_STREAM: D3D11_VIDEO_PROCESSOR_FEATURE_CAPS = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FEATURE_CAPS_PIXEL_ASPECT_RATIO: D3D11_VIDEO_PROCESSOR_FEATURE_CAPS = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FEATURE_CAPS_MIRROR: D3D11_VIDEO_PROCESSOR_FEATURE_CAPS = 512i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FEATURE_CAPS_SHADER_USAGE: D3D11_VIDEO_PROCESSOR_FEATURE_CAPS = 1024i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FEATURE_CAPS_METADATA_HDR10: D3D11_VIDEO_PROCESSOR_FEATURE_CAPS = 2048i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_VIDEO_PROCESSOR_FILTER = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FILTER_BRIGHTNESS: D3D11_VIDEO_PROCESSOR_FILTER = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FILTER_CONTRAST: D3D11_VIDEO_PROCESSOR_FILTER = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FILTER_HUE: D3D11_VIDEO_PROCESSOR_FILTER = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FILTER_SATURATION: D3D11_VIDEO_PROCESSOR_FILTER = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FILTER_NOISE_REDUCTION: D3D11_VIDEO_PROCESSOR_FILTER = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FILTER_EDGE_ENHANCEMENT: D3D11_VIDEO_PROCESSOR_FILTER = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FILTER_ANAMORPHIC_SCALING: D3D11_VIDEO_PROCESSOR_FILTER = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FILTER_STEREO_ADJUSTMENT: D3D11_VIDEO_PROCESSOR_FILTER = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_VIDEO_PROCESSOR_FILTER_CAPS = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FILTER_CAPS_BRIGHTNESS: D3D11_VIDEO_PROCESSOR_FILTER_CAPS = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FILTER_CAPS_CONTRAST: D3D11_VIDEO_PROCESSOR_FILTER_CAPS = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FILTER_CAPS_HUE: D3D11_VIDEO_PROCESSOR_FILTER_CAPS = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FILTER_CAPS_SATURATION: D3D11_VIDEO_PROCESSOR_FILTER_CAPS = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FILTER_CAPS_NOISE_REDUCTION: D3D11_VIDEO_PROCESSOR_FILTER_CAPS = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FILTER_CAPS_EDGE_ENHANCEMENT: D3D11_VIDEO_PROCESSOR_FILTER_CAPS = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FILTER_CAPS_ANAMORPHIC_SCALING: D3D11_VIDEO_PROCESSOR_FILTER_CAPS = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FILTER_CAPS_STEREO_ADJUSTMENT: D3D11_VIDEO_PROCESSOR_FILTER_CAPS = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_VIDEO_PROCESSOR_FORMAT_CAPS = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FORMAT_CAPS_RGB_INTERLACED: D3D11_VIDEO_PROCESSOR_FORMAT_CAPS = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FORMAT_CAPS_RGB_PROCAMP: D3D11_VIDEO_PROCESSOR_FORMAT_CAPS = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FORMAT_CAPS_RGB_LUMA_KEY: D3D11_VIDEO_PROCESSOR_FORMAT_CAPS = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FORMAT_CAPS_PALETTE_INTERLACED: D3D11_VIDEO_PROCESSOR_FORMAT_CAPS = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_VIDEO_PROCESSOR_FORMAT_SUPPORT = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FORMAT_SUPPORT_INPUT: D3D11_VIDEO_PROCESSOR_FORMAT_SUPPORT = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_FORMAT_SUPPORT_OUTPUT: D3D11_VIDEO_PROCESSOR_FORMAT_SUPPORT = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_VIDEO_PROCESSOR_ITELECINE_CAPS = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_ITELECINE_CAPS_32: D3D11_VIDEO_PROCESSOR_ITELECINE_CAPS = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_ITELECINE_CAPS_22: D3D11_VIDEO_PROCESSOR_ITELECINE_CAPS = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_ITELECINE_CAPS_2224: D3D11_VIDEO_PROCESSOR_ITELECINE_CAPS = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_ITELECINE_CAPS_2332: D3D11_VIDEO_PROCESSOR_ITELECINE_CAPS = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_ITELECINE_CAPS_32322: D3D11_VIDEO_PROCESSOR_ITELECINE_CAPS = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_ITELECINE_CAPS_55: D3D11_VIDEO_PROCESSOR_ITELECINE_CAPS = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_ITELECINE_CAPS_64: D3D11_VIDEO_PROCESSOR_ITELECINE_CAPS = 64i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_ITELECINE_CAPS_87: D3D11_VIDEO_PROCESSOR_ITELECINE_CAPS = 128i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_ITELECINE_CAPS_222222222223: D3D11_VIDEO_PROCESSOR_ITELECINE_CAPS = 256i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_ITELECINE_CAPS_OTHER: D3D11_VIDEO_PROCESSOR_ITELECINE_CAPS = -2147483648i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_VIDEO_PROCESSOR_NOMINAL_RANGE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_NOMINAL_RANGE_UNDEFINED: D3D11_VIDEO_PROCESSOR_NOMINAL_RANGE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_NOMINAL_RANGE_16_235: D3D11_VIDEO_PROCESSOR_NOMINAL_RANGE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_NOMINAL_RANGE_0_255: D3D11_VIDEO_PROCESSOR_NOMINAL_RANGE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_VIDEO_PROCESSOR_OUTPUT_RATE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_OUTPUT_RATE_NORMAL: D3D11_VIDEO_PROCESSOR_OUTPUT_RATE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_OUTPUT_RATE_HALF: D3D11_VIDEO_PROCESSOR_OUTPUT_RATE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_OUTPUT_RATE_CUSTOM: D3D11_VIDEO_PROCESSOR_OUTPUT_RATE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_VIDEO_PROCESSOR_PROCESSOR_CAPS = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_PROCESSOR_CAPS_DEINTERLACE_BLEND: D3D11_VIDEO_PROCESSOR_PROCESSOR_CAPS = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_PROCESSOR_CAPS_DEINTERLACE_BOB: D3D11_VIDEO_PROCESSOR_PROCESSOR_CAPS = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_PROCESSOR_CAPS_DEINTERLACE_ADAPTIVE: D3D11_VIDEO_PROCESSOR_PROCESSOR_CAPS = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_PROCESSOR_CAPS_DEINTERLACE_MOTION_COMPENSATION: D3D11_VIDEO_PROCESSOR_PROCESSOR_CAPS = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_PROCESSOR_CAPS_INVERSE_TELECINE: D3D11_VIDEO_PROCESSOR_PROCESSOR_CAPS = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_PROCESSOR_CAPS_FRAME_RATE_CONVERSION: D3D11_VIDEO_PROCESSOR_PROCESSOR_CAPS = 32i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_VIDEO_PROCESSOR_ROTATION = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_ROTATION_IDENTITY: D3D11_VIDEO_PROCESSOR_ROTATION = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_ROTATION_90: D3D11_VIDEO_PROCESSOR_ROTATION = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_ROTATION_180: D3D11_VIDEO_PROCESSOR_ROTATION = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_ROTATION_270: D3D11_VIDEO_PROCESSOR_ROTATION = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_VIDEO_PROCESSOR_STEREO_CAPS = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_STEREO_CAPS_MONO_OFFSET: D3D11_VIDEO_PROCESSOR_STEREO_CAPS = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_STEREO_CAPS_ROW_INTERLEAVED: D3D11_VIDEO_PROCESSOR_STEREO_CAPS = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_STEREO_CAPS_COLUMN_INTERLEAVED: D3D11_VIDEO_PROCESSOR_STEREO_CAPS = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_STEREO_CAPS_CHECKERBOARD: D3D11_VIDEO_PROCESSOR_STEREO_CAPS = 8i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_STEREO_CAPS_FLIP_MODE: D3D11_VIDEO_PROCESSOR_STEREO_CAPS = 16i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_VIDEO_PROCESSOR_STEREO_FLIP_MODE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_STEREO_FLIP_NONE: D3D11_VIDEO_PROCESSOR_STEREO_FLIP_MODE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_STEREO_FLIP_FRAME0: D3D11_VIDEO_PROCESSOR_STEREO_FLIP_MODE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_STEREO_FLIP_FRAME1: D3D11_VIDEO_PROCESSOR_STEREO_FLIP_MODE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_VIDEO_PROCESSOR_STEREO_FORMAT = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_STEREO_FORMAT_MONO: D3D11_VIDEO_PROCESSOR_STEREO_FORMAT = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_STEREO_FORMAT_HORIZONTAL: D3D11_VIDEO_PROCESSOR_STEREO_FORMAT = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_STEREO_FORMAT_VERTICAL: D3D11_VIDEO_PROCESSOR_STEREO_FORMAT = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_STEREO_FORMAT_SEPARATE: D3D11_VIDEO_PROCESSOR_STEREO_FORMAT = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_STEREO_FORMAT_MONO_OFFSET: D3D11_VIDEO_PROCESSOR_STEREO_FORMAT = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_STEREO_FORMAT_ROW_INTERLEAVED: D3D11_VIDEO_PROCESSOR_STEREO_FORMAT = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_STEREO_FORMAT_COLUMN_INTERLEAVED: D3D11_VIDEO_PROCESSOR_STEREO_FORMAT = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_PROCESSOR_STEREO_FORMAT_CHECKERBOARD: D3D11_VIDEO_PROCESSOR_STEREO_FORMAT = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_VIDEO_USAGE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_USAGE_PLAYBACK_NORMAL: D3D11_VIDEO_USAGE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_USAGE_OPTIMAL_SPEED: D3D11_VIDEO_USAGE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VIDEO_USAGE_OPTIMAL_QUALITY: D3D11_VIDEO_USAGE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_VPIV_DIMENSION = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VPIV_DIMENSION_UNKNOWN: D3D11_VPIV_DIMENSION = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VPIV_DIMENSION_TEXTURE2D: D3D11_VPIV_DIMENSION = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3D11_VPOV_DIMENSION = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VPOV_DIMENSION_UNKNOWN: D3D11_VPOV_DIMENSION = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VPOV_DIMENSION_TEXTURE2D: D3D11_VPOV_DIMENSION = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3D11_VPOV_DIMENSION_TEXTURE2DARRAY: D3D11_VPOV_DIMENSION = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3DX11_FFT_CREATE_FLAG = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3DX11_FFT_CREATE_FLAG_NO_PRECOMPUTE_BUFFERS: D3DX11_FFT_CREATE_FLAG = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3DX11_FFT_DATA_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3DX11_FFT_DATA_TYPE_REAL: D3DX11_FFT_DATA_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3DX11_FFT_DATA_TYPE_COMPLEX: D3DX11_FFT_DATA_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3DX11_FFT_DIM_MASK = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3DX11_FFT_DIM_MASK_1D: D3DX11_FFT_DIM_MASK = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3DX11_FFT_DIM_MASK_2D: D3DX11_FFT_DIM_MASK = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3DX11_FFT_DIM_MASK_3D: D3DX11_FFT_DIM_MASK = 7i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3DX11_SCAN_DATA_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3DX11_SCAN_DATA_TYPE_FLOAT: D3DX11_SCAN_DATA_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3DX11_SCAN_DATA_TYPE_INT: D3DX11_SCAN_DATA_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3DX11_SCAN_DATA_TYPE_UINT: D3DX11_SCAN_DATA_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3DX11_SCAN_DIRECTION = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3DX11_SCAN_DIRECTION_FORWARD: D3DX11_SCAN_DIRECTION = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3DX11_SCAN_DIRECTION_BACKWARD: D3DX11_SCAN_DIRECTION = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub type D3DX11_SCAN_OPCODE = i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3DX11_SCAN_OPCODE_ADD: D3DX11_SCAN_OPCODE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3DX11_SCAN_OPCODE_MIN: D3DX11_SCAN_OPCODE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3DX11_SCAN_OPCODE_MAX: D3DX11_SCAN_OPCODE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3DX11_SCAN_OPCODE_MUL: D3DX11_SCAN_OPCODE = 4i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3DX11_SCAN_OPCODE_AND: D3DX11_SCAN_OPCODE = 5i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3DX11_SCAN_OPCODE_OR: D3DX11_SCAN_OPCODE = 6i32;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub const D3DX11_SCAN_OPCODE_XOR: D3DX11_SCAN_OPCODE = 7i32;
#[repr(C)]
pub struct CD3D11_VIDEO_DEFAULT(pub u8);
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_AES_CTR_IV {
    pub IV: u64,
    pub Count: u64,
}
impl ::core::marker::Copy for D3D11_AES_CTR_IV {}
impl ::core::clone::Clone for D3D11_AES_CTR_IV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_AUTHENTICATED_CONFIGURE_ACCESSIBLE_ENCRYPTION_INPUT {
    pub Parameters: D3D11_AUTHENTICATED_CONFIGURE_INPUT,
    pub EncryptionGuid: ::windows_sys::core::GUID,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_AUTHENTICATED_CONFIGURE_ACCESSIBLE_ENCRYPTION_INPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_AUTHENTICATED_CONFIGURE_ACCESSIBLE_ENCRYPTION_INPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_AUTHENTICATED_CONFIGURE_CRYPTO_SESSION_INPUT {
    pub Parameters: D3D11_AUTHENTICATED_CONFIGURE_INPUT,
    pub DecoderHandle: super::super::Foundation::HANDLE,
    pub CryptoSessionHandle: super::super::Foundation::HANDLE,
    pub DeviceHandle: super::super::Foundation::HANDLE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_AUTHENTICATED_CONFIGURE_CRYPTO_SESSION_INPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_AUTHENTICATED_CONFIGURE_CRYPTO_SESSION_INPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_AUTHENTICATED_CONFIGURE_INITIALIZE_INPUT {
    pub Parameters: D3D11_AUTHENTICATED_CONFIGURE_INPUT,
    pub StartSequenceQuery: u32,
    pub StartSequenceConfigure: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_AUTHENTICATED_CONFIGURE_INITIALIZE_INPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_AUTHENTICATED_CONFIGURE_INITIALIZE_INPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_AUTHENTICATED_CONFIGURE_INPUT {
    pub omac: D3D11_OMAC,
    pub ConfigureType: ::windows_sys::core::GUID,
    pub hChannel: super::super::Foundation::HANDLE,
    pub SequenceNumber: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_AUTHENTICATED_CONFIGURE_INPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_AUTHENTICATED_CONFIGURE_INPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_AUTHENTICATED_CONFIGURE_OUTPUT {
    pub omac: D3D11_OMAC,
    pub ConfigureType: ::windows_sys::core::GUID,
    pub hChannel: super::super::Foundation::HANDLE,
    pub SequenceNumber: u32,
    pub ReturnCode: ::windows_sys::core::HRESULT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_AUTHENTICATED_CONFIGURE_OUTPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_AUTHENTICATED_CONFIGURE_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_AUTHENTICATED_CONFIGURE_PROTECTION_INPUT {
    pub Parameters: D3D11_AUTHENTICATED_CONFIGURE_INPUT,
    pub Protections: D3D11_AUTHENTICATED_PROTECTION_FLAGS,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_AUTHENTICATED_CONFIGURE_PROTECTION_INPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_AUTHENTICATED_CONFIGURE_PROTECTION_INPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_AUTHENTICATED_CONFIGURE_SHARED_RESOURCE_INPUT {
    pub Parameters: D3D11_AUTHENTICATED_CONFIGURE_INPUT,
    pub ProcessType: D3D11_AUTHENTICATED_PROCESS_IDENTIFIER_TYPE,
    pub ProcessHandle: super::super::Foundation::HANDLE,
    pub AllowAccess: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_AUTHENTICATED_CONFIGURE_SHARED_RESOURCE_INPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_AUTHENTICATED_CONFIGURE_SHARED_RESOURCE_INPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub union D3D11_AUTHENTICATED_PROTECTION_FLAGS {
    pub Flags: D3D11_AUTHENTICATED_PROTECTION_FLAGS_0,
    pub Value: u32,
}
impl ::core::marker::Copy for D3D11_AUTHENTICATED_PROTECTION_FLAGS {}
impl ::core::clone::Clone for D3D11_AUTHENTICATED_PROTECTION_FLAGS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_AUTHENTICATED_PROTECTION_FLAGS_0 {
    pub _bitfield: u32,
}
impl ::core::marker::Copy for D3D11_AUTHENTICATED_PROTECTION_FLAGS_0 {}
impl ::core::clone::Clone for D3D11_AUTHENTICATED_PROTECTION_FLAGS_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_AUTHENTICATED_QUERY_ACCESSIBILITY_ENCRYPTION_GUID_COUNT_OUTPUT {
    pub Output: D3D11_AUTHENTICATED_QUERY_OUTPUT,
    pub EncryptionGuidCount: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_AUTHENTICATED_QUERY_ACCESSIBILITY_ENCRYPTION_GUID_COUNT_OUTPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_AUTHENTICATED_QUERY_ACCESSIBILITY_ENCRYPTION_GUID_COUNT_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_AUTHENTICATED_QUERY_ACCESSIBILITY_ENCRYPTION_GUID_INPUT {
    pub Input: D3D11_AUTHENTICATED_QUERY_INPUT,
    pub EncryptionGuidIndex: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_AUTHENTICATED_QUERY_ACCESSIBILITY_ENCRYPTION_GUID_INPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_AUTHENTICATED_QUERY_ACCESSIBILITY_ENCRYPTION_GUID_INPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_AUTHENTICATED_QUERY_ACCESSIBILITY_ENCRYPTION_GUID_OUTPUT {
    pub Output: D3D11_AUTHENTICATED_QUERY_OUTPUT,
    pub EncryptionGuidIndex: u32,
    pub EncryptionGuid: ::windows_sys::core::GUID,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_AUTHENTICATED_QUERY_ACCESSIBILITY_ENCRYPTION_GUID_OUTPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_AUTHENTICATED_QUERY_ACCESSIBILITY_ENCRYPTION_GUID_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_AUTHENTICATED_QUERY_ACCESSIBILITY_OUTPUT {
    pub Output: D3D11_AUTHENTICATED_QUERY_OUTPUT,
    pub BusType: D3D11_BUS_TYPE,
    pub AccessibleInContiguousBlocks: super::super::Foundation::BOOL,
    pub AccessibleInNonContiguousBlocks: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_AUTHENTICATED_QUERY_ACCESSIBILITY_OUTPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_AUTHENTICATED_QUERY_ACCESSIBILITY_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_AUTHENTICATED_QUERY_CHANNEL_TYPE_OUTPUT {
    pub Output: D3D11_AUTHENTICATED_QUERY_OUTPUT,
    pub ChannelType: D3D11_AUTHENTICATED_CHANNEL_TYPE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_AUTHENTICATED_QUERY_CHANNEL_TYPE_OUTPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_AUTHENTICATED_QUERY_CHANNEL_TYPE_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_AUTHENTICATED_QUERY_CRYPTO_SESSION_INPUT {
    pub Input: D3D11_AUTHENTICATED_QUERY_INPUT,
    pub DecoderHandle: super::super::Foundation::HANDLE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_AUTHENTICATED_QUERY_CRYPTO_SESSION_INPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_AUTHENTICATED_QUERY_CRYPTO_SESSION_INPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_AUTHENTICATED_QUERY_CRYPTO_SESSION_OUTPUT {
    pub Output: D3D11_AUTHENTICATED_QUERY_OUTPUT,
    pub DecoderHandle: super::super::Foundation::HANDLE,
    pub CryptoSessionHandle: super::super::Foundation::HANDLE,
    pub DeviceHandle: super::super::Foundation::HANDLE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_AUTHENTICATED_QUERY_CRYPTO_SESSION_OUTPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_AUTHENTICATED_QUERY_CRYPTO_SESSION_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_AUTHENTICATED_QUERY_CURRENT_ACCESSIBILITY_ENCRYPTION_OUTPUT {
    pub Output: D3D11_AUTHENTICATED_QUERY_OUTPUT,
    pub EncryptionGuid: ::windows_sys::core::GUID,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_AUTHENTICATED_QUERY_CURRENT_ACCESSIBILITY_ENCRYPTION_OUTPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_AUTHENTICATED_QUERY_CURRENT_ACCESSIBILITY_ENCRYPTION_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_AUTHENTICATED_QUERY_DEVICE_HANDLE_OUTPUT {
    pub Output: D3D11_AUTHENTICATED_QUERY_OUTPUT,
    pub DeviceHandle: super::super::Foundation::HANDLE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_AUTHENTICATED_QUERY_DEVICE_HANDLE_OUTPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_AUTHENTICATED_QUERY_DEVICE_HANDLE_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_AUTHENTICATED_QUERY_INPUT {
    pub QueryType: ::windows_sys::core::GUID,
    pub hChannel: super::super::Foundation::HANDLE,
    pub SequenceNumber: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_AUTHENTICATED_QUERY_INPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_AUTHENTICATED_QUERY_INPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_AUTHENTICATED_QUERY_OUTPUT {
    pub omac: D3D11_OMAC,
    pub QueryType: ::windows_sys::core::GUID,
    pub hChannel: super::super::Foundation::HANDLE,
    pub SequenceNumber: u32,
    pub ReturnCode: ::windows_sys::core::HRESULT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_AUTHENTICATED_QUERY_OUTPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_AUTHENTICATED_QUERY_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_AUTHENTICATED_QUERY_OUTPUT_ID_COUNT_INPUT {
    pub Input: D3D11_AUTHENTICATED_QUERY_INPUT,
    pub DeviceHandle: super::super::Foundation::HANDLE,
    pub CryptoSessionHandle: super::super::Foundation::HANDLE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_AUTHENTICATED_QUERY_OUTPUT_ID_COUNT_INPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_AUTHENTICATED_QUERY_OUTPUT_ID_COUNT_INPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_AUTHENTICATED_QUERY_OUTPUT_ID_COUNT_OUTPUT {
    pub Output: D3D11_AUTHENTICATED_QUERY_OUTPUT,
    pub DeviceHandle: super::super::Foundation::HANDLE,
    pub CryptoSessionHandle: super::super::Foundation::HANDLE,
    pub OutputIDCount: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_AUTHENTICATED_QUERY_OUTPUT_ID_COUNT_OUTPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_AUTHENTICATED_QUERY_OUTPUT_ID_COUNT_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_AUTHENTICATED_QUERY_OUTPUT_ID_INPUT {
    pub Input: D3D11_AUTHENTICATED_QUERY_INPUT,
    pub DeviceHandle: super::super::Foundation::HANDLE,
    pub CryptoSessionHandle: super::super::Foundation::HANDLE,
    pub OutputIDIndex: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_AUTHENTICATED_QUERY_OUTPUT_ID_INPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_AUTHENTICATED_QUERY_OUTPUT_ID_INPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_AUTHENTICATED_QUERY_OUTPUT_ID_OUTPUT {
    pub Output: D3D11_AUTHENTICATED_QUERY_OUTPUT,
    pub DeviceHandle: super::super::Foundation::HANDLE,
    pub CryptoSessionHandle: super::super::Foundation::HANDLE,
    pub OutputIDIndex: u32,
    pub OutputID: u64,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_AUTHENTICATED_QUERY_OUTPUT_ID_OUTPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_AUTHENTICATED_QUERY_OUTPUT_ID_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_AUTHENTICATED_QUERY_PROTECTION_OUTPUT {
    pub Output: D3D11_AUTHENTICATED_QUERY_OUTPUT,
    pub ProtectionFlags: D3D11_AUTHENTICATED_PROTECTION_FLAGS,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_AUTHENTICATED_QUERY_PROTECTION_OUTPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_AUTHENTICATED_QUERY_PROTECTION_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_AUTHENTICATED_QUERY_RESTRICTED_SHARED_RESOURCE_PROCESS_COUNT_OUTPUT {
    pub Output: D3D11_AUTHENTICATED_QUERY_OUTPUT,
    pub RestrictedSharedResourceProcessCount: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_AUTHENTICATED_QUERY_RESTRICTED_SHARED_RESOURCE_PROCESS_COUNT_OUTPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_AUTHENTICATED_QUERY_RESTRICTED_SHARED_RESOURCE_PROCESS_COUNT_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_AUTHENTICATED_QUERY_RESTRICTED_SHARED_RESOURCE_PROCESS_INPUT {
    pub Input: D3D11_AUTHENTICATED_QUERY_INPUT,
    pub ProcessIndex: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_AUTHENTICATED_QUERY_RESTRICTED_SHARED_RESOURCE_PROCESS_INPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_AUTHENTICATED_QUERY_RESTRICTED_SHARED_RESOURCE_PROCESS_INPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_AUTHENTICATED_QUERY_RESTRICTED_SHARED_RESOURCE_PROCESS_OUTPUT {
    pub Output: D3D11_AUTHENTICATED_QUERY_OUTPUT,
    pub ProcessIndex: u32,
    pub ProcessIdentifier: D3D11_AUTHENTICATED_PROCESS_IDENTIFIER_TYPE,
    pub ProcessHandle: super::super::Foundation::HANDLE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_AUTHENTICATED_QUERY_RESTRICTED_SHARED_RESOURCE_PROCESS_OUTPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_AUTHENTICATED_QUERY_RESTRICTED_SHARED_RESOURCE_PROCESS_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_AUTHENTICATED_QUERY_UNRESTRICTED_PROTECTED_SHARED_RESOURCE_COUNT_OUTPUT {
    pub Output: D3D11_AUTHENTICATED_QUERY_OUTPUT,
    pub UnrestrictedProtectedSharedResourceCount: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_AUTHENTICATED_QUERY_UNRESTRICTED_PROTECTED_SHARED_RESOURCE_COUNT_OUTPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_AUTHENTICATED_QUERY_UNRESTRICTED_PROTECTED_SHARED_RESOURCE_COUNT_OUTPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_BLEND_DESC {
    pub AlphaToCoverageEnable: super::super::Foundation::BOOL,
    pub IndependentBlendEnable: super::super::Foundation::BOOL,
    pub RenderTarget: [D3D11_RENDER_TARGET_BLEND_DESC; 8],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_BLEND_DESC {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_BLEND_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_BLEND_DESC1 {
    pub AlphaToCoverageEnable: super::super::Foundation::BOOL,
    pub IndependentBlendEnable: super::super::Foundation::BOOL,
    pub RenderTarget: [D3D11_RENDER_TARGET_BLEND_DESC1; 8],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_BLEND_DESC1 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_BLEND_DESC1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_BOX {
    pub left: u32,
    pub top: u32,
    pub front: u32,
    pub right: u32,
    pub bottom: u32,
    pub back: u32,
}
impl ::core::marker::Copy for D3D11_BOX {}
impl ::core::clone::Clone for D3D11_BOX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_BUFFEREX_SRV {
    pub FirstElement: u32,
    pub NumElements: u32,
    pub Flags: u32,
}
impl ::core::marker::Copy for D3D11_BUFFEREX_SRV {}
impl ::core::clone::Clone for D3D11_BUFFEREX_SRV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_BUFFER_DESC {
    pub ByteWidth: u32,
    pub Usage: D3D11_USAGE,
    pub BindFlags: D3D11_BIND_FLAG,
    pub CPUAccessFlags: D3D11_CPU_ACCESS_FLAG,
    pub MiscFlags: D3D11_RESOURCE_MISC_FLAG,
    pub StructureByteStride: u32,
}
impl ::core::marker::Copy for D3D11_BUFFER_DESC {}
impl ::core::clone::Clone for D3D11_BUFFER_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_BUFFER_RTV {
    pub Anonymous1: D3D11_BUFFER_RTV_0,
    pub Anonymous2: D3D11_BUFFER_RTV_1,
}
impl ::core::marker::Copy for D3D11_BUFFER_RTV {}
impl ::core::clone::Clone for D3D11_BUFFER_RTV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub union D3D11_BUFFER_RTV_0 {
    pub FirstElement: u32,
    pub ElementOffset: u32,
}
impl ::core::marker::Copy for D3D11_BUFFER_RTV_0 {}
impl ::core::clone::Clone for D3D11_BUFFER_RTV_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub union D3D11_BUFFER_RTV_1 {
    pub NumElements: u32,
    pub ElementWidth: u32,
}
impl ::core::marker::Copy for D3D11_BUFFER_RTV_1 {}
impl ::core::clone::Clone for D3D11_BUFFER_RTV_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_BUFFER_SRV {
    pub Anonymous1: D3D11_BUFFER_SRV_0,
    pub Anonymous2: D3D11_BUFFER_SRV_1,
}
impl ::core::marker::Copy for D3D11_BUFFER_SRV {}
impl ::core::clone::Clone for D3D11_BUFFER_SRV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub union D3D11_BUFFER_SRV_0 {
    pub FirstElement: u32,
    pub ElementOffset: u32,
}
impl ::core::marker::Copy for D3D11_BUFFER_SRV_0 {}
impl ::core::clone::Clone for D3D11_BUFFER_SRV_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub union D3D11_BUFFER_SRV_1 {
    pub NumElements: u32,
    pub ElementWidth: u32,
}
impl ::core::marker::Copy for D3D11_BUFFER_SRV_1 {}
impl ::core::clone::Clone for D3D11_BUFFER_SRV_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_BUFFER_UAV {
    pub FirstElement: u32,
    pub NumElements: u32,
    pub Flags: u32,
}
impl ::core::marker::Copy for D3D11_BUFFER_UAV {}
impl ::core::clone::Clone for D3D11_BUFFER_UAV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_CLASS_INSTANCE_DESC {
    pub InstanceId: u32,
    pub InstanceIndex: u32,
    pub TypeId: u32,
    pub ConstantBuffer: u32,
    pub BaseConstantBufferOffset: u32,
    pub BaseTexture: u32,
    pub BaseSampler: u32,
    pub Created: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_CLASS_INSTANCE_DESC {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_CLASS_INSTANCE_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_COMPUTE_SHADER_TRACE_DESC {
    pub Invocation: u64,
    pub ThreadIDInGroup: [u32; 3],
    pub ThreadGroupID: [u32; 3],
}
impl ::core::marker::Copy for D3D11_COMPUTE_SHADER_TRACE_DESC {}
impl ::core::clone::Clone for D3D11_COMPUTE_SHADER_TRACE_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_COUNTER_DESC {
    pub Counter: D3D11_COUNTER,
    pub MiscFlags: u32,
}
impl ::core::marker::Copy for D3D11_COUNTER_DESC {}
impl ::core::clone::Clone for D3D11_COUNTER_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_COUNTER_INFO {
    pub LastDeviceDependentCounter: D3D11_COUNTER,
    pub NumSimultaneousCounters: u32,
    pub NumDetectableParallelUnits: u8,
}
impl ::core::marker::Copy for D3D11_COUNTER_INFO {}
impl ::core::clone::Clone for D3D11_COUNTER_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_DEPTH_STENCILOP_DESC {
    pub StencilFailOp: D3D11_STENCIL_OP,
    pub StencilDepthFailOp: D3D11_STENCIL_OP,
    pub StencilPassOp: D3D11_STENCIL_OP,
    pub StencilFunc: D3D11_COMPARISON_FUNC,
}
impl ::core::marker::Copy for D3D11_DEPTH_STENCILOP_DESC {}
impl ::core::clone::Clone for D3D11_DEPTH_STENCILOP_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_DEPTH_STENCIL_DESC {
    pub DepthEnable: super::super::Foundation::BOOL,
    pub DepthWriteMask: D3D11_DEPTH_WRITE_MASK,
    pub DepthFunc: D3D11_COMPARISON_FUNC,
    pub StencilEnable: super::super::Foundation::BOOL,
    pub StencilReadMask: u8,
    pub StencilWriteMask: u8,
    pub FrontFace: D3D11_DEPTH_STENCILOP_DESC,
    pub BackFace: D3D11_DEPTH_STENCILOP_DESC,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_DEPTH_STENCIL_DESC {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_DEPTH_STENCIL_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub struct D3D11_DEPTH_STENCIL_VIEW_DESC {
    pub Format: super::Dxgi::Common::DXGI_FORMAT,
    pub ViewDimension: D3D11_DSV_DIMENSION,
    pub Flags: u32,
    pub Anonymous: D3D11_DEPTH_STENCIL_VIEW_DESC_0,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D11_DEPTH_STENCIL_VIEW_DESC {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D11_DEPTH_STENCIL_VIEW_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub union D3D11_DEPTH_STENCIL_VIEW_DESC_0 {
    pub Texture1D: D3D11_TEX1D_DSV,
    pub Texture1DArray: D3D11_TEX1D_ARRAY_DSV,
    pub Texture2D: D3D11_TEX2D_DSV,
    pub Texture2DArray: D3D11_TEX2D_ARRAY_DSV,
    pub Texture2DMS: D3D11_TEX2DMS_DSV,
    pub Texture2DMSArray: D3D11_TEX2DMS_ARRAY_DSV,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D11_DEPTH_STENCIL_VIEW_DESC_0 {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D11_DEPTH_STENCIL_VIEW_DESC_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_DOMAIN_SHADER_TRACE_DESC {
    pub Invocation: u64,
}
impl ::core::marker::Copy for D3D11_DOMAIN_SHADER_TRACE_DESC {}
impl ::core::clone::Clone for D3D11_DOMAIN_SHADER_TRACE_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_DRAW_INDEXED_INSTANCED_INDIRECT_ARGS {
    pub IndexCountPerInstance: u32,
    pub InstanceCount: u32,
    pub StartIndexLocation: u32,
    pub BaseVertexLocation: i32,
    pub StartInstanceLocation: u32,
}
impl ::core::marker::Copy for D3D11_DRAW_INDEXED_INSTANCED_INDIRECT_ARGS {}
impl ::core::clone::Clone for D3D11_DRAW_INDEXED_INSTANCED_INDIRECT_ARGS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_DRAW_INSTANCED_INDIRECT_ARGS {
    pub VertexCountPerInstance: u32,
    pub InstanceCount: u32,
    pub StartVertexLocation: u32,
    pub StartInstanceLocation: u32,
}
impl ::core::marker::Copy for D3D11_DRAW_INSTANCED_INDIRECT_ARGS {}
impl ::core::clone::Clone for D3D11_DRAW_INSTANCED_INDIRECT_ARGS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_ENCRYPTED_BLOCK_INFO {
    pub NumEncryptedBytesAtBeginning: u32,
    pub NumBytesInSkipPattern: u32,
    pub NumBytesInEncryptPattern: u32,
}
impl ::core::marker::Copy for D3D11_ENCRYPTED_BLOCK_INFO {}
impl ::core::clone::Clone for D3D11_ENCRYPTED_BLOCK_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_FEATURE_DATA_ARCHITECTURE_INFO {
    pub TileBasedDeferredRenderer: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_FEATURE_DATA_ARCHITECTURE_INFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_FEATURE_DATA_ARCHITECTURE_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_FEATURE_DATA_D3D10_X_HARDWARE_OPTIONS {
    pub ComputeShaders_Plus_RawAndStructuredBuffers_Via_Shader_4_x: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_FEATURE_DATA_D3D10_X_HARDWARE_OPTIONS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_FEATURE_DATA_D3D10_X_HARDWARE_OPTIONS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_FEATURE_DATA_D3D11_OPTIONS {
    pub OutputMergerLogicOp: super::super::Foundation::BOOL,
    pub UAVOnlyRenderingForcedSampleCount: super::super::Foundation::BOOL,
    pub DiscardAPIsSeenByDriver: super::super::Foundation::BOOL,
    pub FlagsForUpdateAndCopySeenByDriver: super::super::Foundation::BOOL,
    pub ClearView: super::super::Foundation::BOOL,
    pub CopyWithOverlap: super::super::Foundation::BOOL,
    pub ConstantBufferPartialUpdate: super::super::Foundation::BOOL,
    pub ConstantBufferOffsetting: super::super::Foundation::BOOL,
    pub MapNoOverwriteOnDynamicConstantBuffer: super::super::Foundation::BOOL,
    pub MapNoOverwriteOnDynamicBufferSRV: super::super::Foundation::BOOL,
    pub MultisampleRTVWithForcedSampleCountOne: super::super::Foundation::BOOL,
    pub SAD4ShaderInstructions: super::super::Foundation::BOOL,
    pub ExtendedDoublesShaderInstructions: super::super::Foundation::BOOL,
    pub ExtendedResourceSharing: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_FEATURE_DATA_D3D11_OPTIONS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_FEATURE_DATA_D3D11_OPTIONS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_FEATURE_DATA_D3D11_OPTIONS1 {
    pub TiledResourcesTier: D3D11_TILED_RESOURCES_TIER,
    pub MinMaxFiltering: super::super::Foundation::BOOL,
    pub ClearViewAlsoSupportsDepthOnlyFormats: super::super::Foundation::BOOL,
    pub MapOnDefaultBuffers: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_FEATURE_DATA_D3D11_OPTIONS1 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_FEATURE_DATA_D3D11_OPTIONS1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_FEATURE_DATA_D3D11_OPTIONS2 {
    pub PSSpecifiedStencilRefSupported: super::super::Foundation::BOOL,
    pub TypedUAVLoadAdditionalFormats: super::super::Foundation::BOOL,
    pub ROVsSupported: super::super::Foundation::BOOL,
    pub ConservativeRasterizationTier: D3D11_CONSERVATIVE_RASTERIZATION_TIER,
    pub TiledResourcesTier: D3D11_TILED_RESOURCES_TIER,
    pub MapOnDefaultTextures: super::super::Foundation::BOOL,
    pub StandardSwizzle: super::super::Foundation::BOOL,
    pub UnifiedMemoryArchitecture: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_FEATURE_DATA_D3D11_OPTIONS2 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_FEATURE_DATA_D3D11_OPTIONS2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_FEATURE_DATA_D3D11_OPTIONS3 {
    pub VPAndRTArrayIndexFromAnyShaderFeedingRasterizer: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_FEATURE_DATA_D3D11_OPTIONS3 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_FEATURE_DATA_D3D11_OPTIONS3 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_FEATURE_DATA_D3D11_OPTIONS4 {
    pub ExtendedNV12SharedTextureSupported: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_FEATURE_DATA_D3D11_OPTIONS4 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_FEATURE_DATA_D3D11_OPTIONS4 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_FEATURE_DATA_D3D11_OPTIONS5 {
    pub SharedResourceTier: D3D11_SHARED_RESOURCE_TIER,
}
impl ::core::marker::Copy for D3D11_FEATURE_DATA_D3D11_OPTIONS5 {}
impl ::core::clone::Clone for D3D11_FEATURE_DATA_D3D11_OPTIONS5 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_FEATURE_DATA_D3D9_OPTIONS {
    pub FullNonPow2TextureSupport: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_FEATURE_DATA_D3D9_OPTIONS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_FEATURE_DATA_D3D9_OPTIONS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_FEATURE_DATA_D3D9_OPTIONS1 {
    pub FullNonPow2TextureSupported: super::super::Foundation::BOOL,
    pub DepthAsTextureWithLessEqualComparisonFilterSupported: super::super::Foundation::BOOL,
    pub SimpleInstancingSupported: super::super::Foundation::BOOL,
    pub TextureCubeFaceRenderTargetWithNonCubeDepthStencilSupported: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_FEATURE_DATA_D3D9_OPTIONS1 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_FEATURE_DATA_D3D9_OPTIONS1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_FEATURE_DATA_D3D9_SHADOW_SUPPORT {
    pub SupportsDepthAsTextureWithLessEqualComparisonFilter: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_FEATURE_DATA_D3D9_SHADOW_SUPPORT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_FEATURE_DATA_D3D9_SHADOW_SUPPORT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_FEATURE_DATA_D3D9_SIMPLE_INSTANCING_SUPPORT {
    pub SimpleInstancingSupported: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_FEATURE_DATA_D3D9_SIMPLE_INSTANCING_SUPPORT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_FEATURE_DATA_D3D9_SIMPLE_INSTANCING_SUPPORT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_FEATURE_DATA_DISPLAYABLE {
    pub DisplayableTexture: super::super::Foundation::BOOL,
    pub SharedResourceTier: D3D11_SHARED_RESOURCE_TIER,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_FEATURE_DATA_DISPLAYABLE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_FEATURE_DATA_DISPLAYABLE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_FEATURE_DATA_DOUBLES {
    pub DoublePrecisionFloatShaderOps: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_FEATURE_DATA_DOUBLES {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_FEATURE_DATA_DOUBLES {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub struct D3D11_FEATURE_DATA_FORMAT_SUPPORT {
    pub InFormat: super::Dxgi::Common::DXGI_FORMAT,
    pub OutFormatSupport: u32,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D11_FEATURE_DATA_FORMAT_SUPPORT {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D11_FEATURE_DATA_FORMAT_SUPPORT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub struct D3D11_FEATURE_DATA_FORMAT_SUPPORT2 {
    pub InFormat: super::Dxgi::Common::DXGI_FORMAT,
    pub OutFormatSupport2: u32,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D11_FEATURE_DATA_FORMAT_SUPPORT2 {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D11_FEATURE_DATA_FORMAT_SUPPORT2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_FEATURE_DATA_GPU_VIRTUAL_ADDRESS_SUPPORT {
    pub MaxGPUVirtualAddressBitsPerResource: u32,
    pub MaxGPUVirtualAddressBitsPerProcess: u32,
}
impl ::core::marker::Copy for D3D11_FEATURE_DATA_GPU_VIRTUAL_ADDRESS_SUPPORT {}
impl ::core::clone::Clone for D3D11_FEATURE_DATA_GPU_VIRTUAL_ADDRESS_SUPPORT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_FEATURE_DATA_MARKER_SUPPORT {
    pub Profile: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_FEATURE_DATA_MARKER_SUPPORT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_FEATURE_DATA_MARKER_SUPPORT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_FEATURE_DATA_SHADER_CACHE {
    pub SupportFlags: u32,
}
impl ::core::marker::Copy for D3D11_FEATURE_DATA_SHADER_CACHE {}
impl ::core::clone::Clone for D3D11_FEATURE_DATA_SHADER_CACHE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_FEATURE_DATA_SHADER_MIN_PRECISION_SUPPORT {
    pub PixelShaderMinPrecision: u32,
    pub AllOtherShaderStagesMinPrecision: u32,
}
impl ::core::marker::Copy for D3D11_FEATURE_DATA_SHADER_MIN_PRECISION_SUPPORT {}
impl ::core::clone::Clone for D3D11_FEATURE_DATA_SHADER_MIN_PRECISION_SUPPORT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_FEATURE_DATA_THREADING {
    pub DriverConcurrentCreates: super::super::Foundation::BOOL,
    pub DriverCommandLists: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_FEATURE_DATA_THREADING {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_FEATURE_DATA_THREADING {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub struct D3D11_FEATURE_DATA_VIDEO_DECODER_HISTOGRAM {
    pub DecoderDesc: D3D11_VIDEO_DECODER_DESC,
    pub Components: D3D11_VIDEO_DECODER_HISTOGRAM_COMPONENT_FLAGS,
    pub BinCount: u32,
    pub CounterBitDepth: u32,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D11_FEATURE_DATA_VIDEO_DECODER_HISTOGRAM {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D11_FEATURE_DATA_VIDEO_DECODER_HISTOGRAM {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Direct3D\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct3D"))]
pub struct D3D11_FUNCTION_DESC {
    pub Version: u32,
    pub Creator: ::windows_sys::core::PCSTR,
    pub Flags: u32,
    pub ConstantBuffers: u32,
    pub BoundResources: u32,
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
    pub MovInstructionCount: u32,
    pub MovcInstructionCount: u32,
    pub ConversionInstructionCount: u32,
    pub BitwiseInstructionCount: u32,
    pub MinFeatureLevel: super::Direct3D::D3D_FEATURE_LEVEL,
    pub RequiredFeatureFlags: u64,
    pub Name: ::windows_sys::core::PCSTR,
    pub FunctionParameterCount: i32,
    pub HasReturn: super::super::Foundation::BOOL,
    pub Has10Level9VertexShader: super::super::Foundation::BOOL,
    pub Has10Level9PixelShader: super::super::Foundation::BOOL,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct3D"))]
impl ::core::marker::Copy for D3D11_FUNCTION_DESC {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct3D"))]
impl ::core::clone::Clone for D3D11_FUNCTION_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_GEOMETRY_SHADER_TRACE_DESC {
    pub Invocation: u64,
}
impl ::core::marker::Copy for D3D11_GEOMETRY_SHADER_TRACE_DESC {}
impl ::core::clone::Clone for D3D11_GEOMETRY_SHADER_TRACE_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_HULL_SHADER_TRACE_DESC {
    pub Invocation: u64,
}
impl ::core::marker::Copy for D3D11_HULL_SHADER_TRACE_DESC {}
impl ::core::clone::Clone for D3D11_HULL_SHADER_TRACE_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_INFO_QUEUE_FILTER {
    pub AllowList: D3D11_INFO_QUEUE_FILTER_DESC,
    pub DenyList: D3D11_INFO_QUEUE_FILTER_DESC,
}
impl ::core::marker::Copy for D3D11_INFO_QUEUE_FILTER {}
impl ::core::clone::Clone for D3D11_INFO_QUEUE_FILTER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_INFO_QUEUE_FILTER_DESC {
    pub NumCategories: u32,
    pub pCategoryList: *mut D3D11_MESSAGE_CATEGORY,
    pub NumSeverities: u32,
    pub pSeverityList: *mut D3D11_MESSAGE_SEVERITY,
    pub NumIDs: u32,
    pub pIDList: *mut D3D11_MESSAGE_ID,
}
impl ::core::marker::Copy for D3D11_INFO_QUEUE_FILTER_DESC {}
impl ::core::clone::Clone for D3D11_INFO_QUEUE_FILTER_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub struct D3D11_INPUT_ELEMENT_DESC {
    pub SemanticName: ::windows_sys::core::PCSTR,
    pub SemanticIndex: u32,
    pub Format: super::Dxgi::Common::DXGI_FORMAT,
    pub InputSlot: u32,
    pub AlignedByteOffset: u32,
    pub InputSlotClass: D3D11_INPUT_CLASSIFICATION,
    pub InstanceDataStepRate: u32,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D11_INPUT_ELEMENT_DESC {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D11_INPUT_ELEMENT_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_KEY_EXCHANGE_HW_PROTECTION_DATA {
    pub HWProtectionFunctionID: u32,
    pub pInputData: *mut D3D11_KEY_EXCHANGE_HW_PROTECTION_INPUT_DATA,
    pub pOutputData: *mut D3D11_KEY_EXCHANGE_HW_PROTECTION_OUTPUT_DATA,
    pub Status: ::windows_sys::core::HRESULT,
}
impl ::core::marker::Copy for D3D11_KEY_EXCHANGE_HW_PROTECTION_DATA {}
impl ::core::clone::Clone for D3D11_KEY_EXCHANGE_HW_PROTECTION_DATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_KEY_EXCHANGE_HW_PROTECTION_INPUT_DATA {
    pub PrivateDataSize: u32,
    pub HWProtectionDataSize: u32,
    pub pbInput: [u8; 4],
}
impl ::core::marker::Copy for D3D11_KEY_EXCHANGE_HW_PROTECTION_INPUT_DATA {}
impl ::core::clone::Clone for D3D11_KEY_EXCHANGE_HW_PROTECTION_INPUT_DATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_KEY_EXCHANGE_HW_PROTECTION_OUTPUT_DATA {
    pub PrivateDataSize: u32,
    pub MaxHWProtectionDataSize: u32,
    pub HWProtectionDataSize: u32,
    pub TransportTime: u64,
    pub ExecutionTime: u64,
    pub pbOutput: [u8; 4],
}
impl ::core::marker::Copy for D3D11_KEY_EXCHANGE_HW_PROTECTION_OUTPUT_DATA {}
impl ::core::clone::Clone for D3D11_KEY_EXCHANGE_HW_PROTECTION_OUTPUT_DATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_LIBRARY_DESC {
    pub Creator: ::windows_sys::core::PCSTR,
    pub Flags: u32,
    pub FunctionCount: u32,
}
impl ::core::marker::Copy for D3D11_LIBRARY_DESC {}
impl ::core::clone::Clone for D3D11_LIBRARY_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_MAPPED_SUBRESOURCE {
    pub pData: *mut ::core::ffi::c_void,
    pub RowPitch: u32,
    pub DepthPitch: u32,
}
impl ::core::marker::Copy for D3D11_MAPPED_SUBRESOURCE {}
impl ::core::clone::Clone for D3D11_MAPPED_SUBRESOURCE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_MESSAGE {
    pub Category: D3D11_MESSAGE_CATEGORY,
    pub Severity: D3D11_MESSAGE_SEVERITY,
    pub ID: D3D11_MESSAGE_ID,
    pub pDescription: *const u8,
    pub DescriptionByteLength: usize,
}
impl ::core::marker::Copy for D3D11_MESSAGE {}
impl ::core::clone::Clone for D3D11_MESSAGE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_OMAC {
    pub Omac: [u8; 16],
}
impl ::core::marker::Copy for D3D11_OMAC {}
impl ::core::clone::Clone for D3D11_OMAC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_PACKED_MIP_DESC {
    pub NumStandardMips: u8,
    pub NumPackedMips: u8,
    pub NumTilesForPackedMips: u32,
    pub StartTileIndexInOverallResource: u32,
}
impl ::core::marker::Copy for D3D11_PACKED_MIP_DESC {}
impl ::core::clone::Clone for D3D11_PACKED_MIP_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Direct3D\"`*"]
#[cfg(feature = "Win32_Graphics_Direct3D")]
pub struct D3D11_PARAMETER_DESC {
    pub Name: ::windows_sys::core::PCSTR,
    pub SemanticName: ::windows_sys::core::PCSTR,
    pub Type: super::Direct3D::D3D_SHADER_VARIABLE_TYPE,
    pub Class: super::Direct3D::D3D_SHADER_VARIABLE_CLASS,
    pub Rows: u32,
    pub Columns: u32,
    pub InterpolationMode: super::Direct3D::D3D_INTERPOLATION_MODE,
    pub Flags: super::Direct3D::D3D_PARAMETER_FLAGS,
    pub FirstInRegister: u32,
    pub FirstInComponent: u32,
    pub FirstOutRegister: u32,
    pub FirstOutComponent: u32,
}
#[cfg(feature = "Win32_Graphics_Direct3D")]
impl ::core::marker::Copy for D3D11_PARAMETER_DESC {}
#[cfg(feature = "Win32_Graphics_Direct3D")]
impl ::core::clone::Clone for D3D11_PARAMETER_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_PIXEL_SHADER_TRACE_DESC {
    pub Invocation: u64,
    pub X: i32,
    pub Y: i32,
    pub SampleMask: u64,
}
impl ::core::marker::Copy for D3D11_PIXEL_SHADER_TRACE_DESC {}
impl ::core::clone::Clone for D3D11_PIXEL_SHADER_TRACE_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_QUERY_DATA_PIPELINE_STATISTICS {
    pub IAVertices: u64,
    pub IAPrimitives: u64,
    pub VSInvocations: u64,
    pub GSInvocations: u64,
    pub GSPrimitives: u64,
    pub CInvocations: u64,
    pub CPrimitives: u64,
    pub PSInvocations: u64,
    pub HSInvocations: u64,
    pub DSInvocations: u64,
    pub CSInvocations: u64,
}
impl ::core::marker::Copy for D3D11_QUERY_DATA_PIPELINE_STATISTICS {}
impl ::core::clone::Clone for D3D11_QUERY_DATA_PIPELINE_STATISTICS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_QUERY_DATA_SO_STATISTICS {
    pub NumPrimitivesWritten: u64,
    pub PrimitivesStorageNeeded: u64,
}
impl ::core::marker::Copy for D3D11_QUERY_DATA_SO_STATISTICS {}
impl ::core::clone::Clone for D3D11_QUERY_DATA_SO_STATISTICS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_QUERY_DATA_TIMESTAMP_DISJOINT {
    pub Frequency: u64,
    pub Disjoint: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_QUERY_DATA_TIMESTAMP_DISJOINT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_QUERY_DATA_TIMESTAMP_DISJOINT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_QUERY_DESC {
    pub Query: D3D11_QUERY,
    pub MiscFlags: u32,
}
impl ::core::marker::Copy for D3D11_QUERY_DESC {}
impl ::core::clone::Clone for D3D11_QUERY_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_QUERY_DESC1 {
    pub Query: D3D11_QUERY,
    pub MiscFlags: u32,
    pub ContextType: D3D11_CONTEXT_TYPE,
}
impl ::core::marker::Copy for D3D11_QUERY_DESC1 {}
impl ::core::clone::Clone for D3D11_QUERY_DESC1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_RASTERIZER_DESC {
    pub FillMode: D3D11_FILL_MODE,
    pub CullMode: D3D11_CULL_MODE,
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
impl ::core::marker::Copy for D3D11_RASTERIZER_DESC {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_RASTERIZER_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_RASTERIZER_DESC1 {
    pub FillMode: D3D11_FILL_MODE,
    pub CullMode: D3D11_CULL_MODE,
    pub FrontCounterClockwise: super::super::Foundation::BOOL,
    pub DepthBias: i32,
    pub DepthBiasClamp: f32,
    pub SlopeScaledDepthBias: f32,
    pub DepthClipEnable: super::super::Foundation::BOOL,
    pub ScissorEnable: super::super::Foundation::BOOL,
    pub MultisampleEnable: super::super::Foundation::BOOL,
    pub AntialiasedLineEnable: super::super::Foundation::BOOL,
    pub ForcedSampleCount: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_RASTERIZER_DESC1 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_RASTERIZER_DESC1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_RASTERIZER_DESC2 {
    pub FillMode: D3D11_FILL_MODE,
    pub CullMode: D3D11_CULL_MODE,
    pub FrontCounterClockwise: super::super::Foundation::BOOL,
    pub DepthBias: i32,
    pub DepthBiasClamp: f32,
    pub SlopeScaledDepthBias: f32,
    pub DepthClipEnable: super::super::Foundation::BOOL,
    pub ScissorEnable: super::super::Foundation::BOOL,
    pub MultisampleEnable: super::super::Foundation::BOOL,
    pub AntialiasedLineEnable: super::super::Foundation::BOOL,
    pub ForcedSampleCount: u32,
    pub ConservativeRaster: D3D11_CONSERVATIVE_RASTERIZATION_MODE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_RASTERIZER_DESC2 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_RASTERIZER_DESC2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_RENDER_TARGET_BLEND_DESC {
    pub BlendEnable: super::super::Foundation::BOOL,
    pub SrcBlend: D3D11_BLEND,
    pub DestBlend: D3D11_BLEND,
    pub BlendOp: D3D11_BLEND_OP,
    pub SrcBlendAlpha: D3D11_BLEND,
    pub DestBlendAlpha: D3D11_BLEND,
    pub BlendOpAlpha: D3D11_BLEND_OP,
    pub RenderTargetWriteMask: u8,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_RENDER_TARGET_BLEND_DESC {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_RENDER_TARGET_BLEND_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_RENDER_TARGET_BLEND_DESC1 {
    pub BlendEnable: super::super::Foundation::BOOL,
    pub LogicOpEnable: super::super::Foundation::BOOL,
    pub SrcBlend: D3D11_BLEND,
    pub DestBlend: D3D11_BLEND,
    pub BlendOp: D3D11_BLEND_OP,
    pub SrcBlendAlpha: D3D11_BLEND,
    pub DestBlendAlpha: D3D11_BLEND,
    pub BlendOpAlpha: D3D11_BLEND_OP,
    pub LogicOp: D3D11_LOGIC_OP,
    pub RenderTargetWriteMask: u8,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_RENDER_TARGET_BLEND_DESC1 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_RENDER_TARGET_BLEND_DESC1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub struct D3D11_RENDER_TARGET_VIEW_DESC {
    pub Format: super::Dxgi::Common::DXGI_FORMAT,
    pub ViewDimension: D3D11_RTV_DIMENSION,
    pub Anonymous: D3D11_RENDER_TARGET_VIEW_DESC_0,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D11_RENDER_TARGET_VIEW_DESC {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D11_RENDER_TARGET_VIEW_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub union D3D11_RENDER_TARGET_VIEW_DESC_0 {
    pub Buffer: D3D11_BUFFER_RTV,
    pub Texture1D: D3D11_TEX1D_RTV,
    pub Texture1DArray: D3D11_TEX1D_ARRAY_RTV,
    pub Texture2D: D3D11_TEX2D_RTV,
    pub Texture2DArray: D3D11_TEX2D_ARRAY_RTV,
    pub Texture2DMS: D3D11_TEX2DMS_RTV,
    pub Texture2DMSArray: D3D11_TEX2DMS_ARRAY_RTV,
    pub Texture3D: D3D11_TEX3D_RTV,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D11_RENDER_TARGET_VIEW_DESC_0 {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D11_RENDER_TARGET_VIEW_DESC_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub struct D3D11_RENDER_TARGET_VIEW_DESC1 {
    pub Format: super::Dxgi::Common::DXGI_FORMAT,
    pub ViewDimension: D3D11_RTV_DIMENSION,
    pub Anonymous: D3D11_RENDER_TARGET_VIEW_DESC1_0,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D11_RENDER_TARGET_VIEW_DESC1 {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D11_RENDER_TARGET_VIEW_DESC1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub union D3D11_RENDER_TARGET_VIEW_DESC1_0 {
    pub Buffer: D3D11_BUFFER_RTV,
    pub Texture1D: D3D11_TEX1D_RTV,
    pub Texture1DArray: D3D11_TEX1D_ARRAY_RTV,
    pub Texture2D: D3D11_TEX2D_RTV1,
    pub Texture2DArray: D3D11_TEX2D_ARRAY_RTV1,
    pub Texture2DMS: D3D11_TEX2DMS_RTV,
    pub Texture2DMSArray: D3D11_TEX2DMS_ARRAY_RTV,
    pub Texture3D: D3D11_TEX3D_RTV,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D11_RENDER_TARGET_VIEW_DESC1_0 {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D11_RENDER_TARGET_VIEW_DESC1_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_SAMPLER_DESC {
    pub Filter: D3D11_FILTER,
    pub AddressU: D3D11_TEXTURE_ADDRESS_MODE,
    pub AddressV: D3D11_TEXTURE_ADDRESS_MODE,
    pub AddressW: D3D11_TEXTURE_ADDRESS_MODE,
    pub MipLODBias: f32,
    pub MaxAnisotropy: u32,
    pub ComparisonFunc: D3D11_COMPARISON_FUNC,
    pub BorderColor: [f32; 4],
    pub MinLOD: f32,
    pub MaxLOD: f32,
}
impl ::core::marker::Copy for D3D11_SAMPLER_DESC {}
impl ::core::clone::Clone for D3D11_SAMPLER_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Direct3D\"`*"]
#[cfg(feature = "Win32_Graphics_Direct3D")]
pub struct D3D11_SHADER_BUFFER_DESC {
    pub Name: ::windows_sys::core::PCSTR,
    pub Type: super::Direct3D::D3D_CBUFFER_TYPE,
    pub Variables: u32,
    pub Size: u32,
    pub uFlags: u32,
}
#[cfg(feature = "Win32_Graphics_Direct3D")]
impl ::core::marker::Copy for D3D11_SHADER_BUFFER_DESC {}
#[cfg(feature = "Win32_Graphics_Direct3D")]
impl ::core::clone::Clone for D3D11_SHADER_BUFFER_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Direct3D\"`*"]
#[cfg(feature = "Win32_Graphics_Direct3D")]
pub struct D3D11_SHADER_DESC {
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
    pub InputPrimitive: super::Direct3D::D3D_PRIMITIVE,
    pub PatchConstantParameters: u32,
    pub cGSInstanceCount: u32,
    pub cControlPoints: u32,
    pub HSOutputPrimitive: super::Direct3D::D3D_TESSELLATOR_OUTPUT_PRIMITIVE,
    pub HSPartitioning: super::Direct3D::D3D_TESSELLATOR_PARTITIONING,
    pub TessellatorDomain: super::Direct3D::D3D_TESSELLATOR_DOMAIN,
    pub cBarrierInstructions: u32,
    pub cInterlockedInstructions: u32,
    pub cTextureStoreInstructions: u32,
}
#[cfg(feature = "Win32_Graphics_Direct3D")]
impl ::core::marker::Copy for D3D11_SHADER_DESC {}
#[cfg(feature = "Win32_Graphics_Direct3D")]
impl ::core::clone::Clone for D3D11_SHADER_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Direct3D\"`*"]
#[cfg(feature = "Win32_Graphics_Direct3D")]
pub struct D3D11_SHADER_INPUT_BIND_DESC {
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
impl ::core::marker::Copy for D3D11_SHADER_INPUT_BIND_DESC {}
#[cfg(feature = "Win32_Graphics_Direct3D")]
impl ::core::clone::Clone for D3D11_SHADER_INPUT_BIND_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Direct3D\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(all(feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common"))]
pub struct D3D11_SHADER_RESOURCE_VIEW_DESC {
    pub Format: super::Dxgi::Common::DXGI_FORMAT,
    pub ViewDimension: super::Direct3D::D3D_SRV_DIMENSION,
    pub Anonymous: D3D11_SHADER_RESOURCE_VIEW_DESC_0,
}
#[cfg(all(feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common"))]
impl ::core::marker::Copy for D3D11_SHADER_RESOURCE_VIEW_DESC {}
#[cfg(all(feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common"))]
impl ::core::clone::Clone for D3D11_SHADER_RESOURCE_VIEW_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Direct3D\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(all(feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common"))]
pub union D3D11_SHADER_RESOURCE_VIEW_DESC_0 {
    pub Buffer: D3D11_BUFFER_SRV,
    pub Texture1D: D3D11_TEX1D_SRV,
    pub Texture1DArray: D3D11_TEX1D_ARRAY_SRV,
    pub Texture2D: D3D11_TEX2D_SRV,
    pub Texture2DArray: D3D11_TEX2D_ARRAY_SRV,
    pub Texture2DMS: D3D11_TEX2DMS_SRV,
    pub Texture2DMSArray: D3D11_TEX2DMS_ARRAY_SRV,
    pub Texture3D: D3D11_TEX3D_SRV,
    pub TextureCube: D3D11_TEXCUBE_SRV,
    pub TextureCubeArray: D3D11_TEXCUBE_ARRAY_SRV,
    pub BufferEx: D3D11_BUFFEREX_SRV,
}
#[cfg(all(feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common"))]
impl ::core::marker::Copy for D3D11_SHADER_RESOURCE_VIEW_DESC_0 {}
#[cfg(all(feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common"))]
impl ::core::clone::Clone for D3D11_SHADER_RESOURCE_VIEW_DESC_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Direct3D\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(all(feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common"))]
pub struct D3D11_SHADER_RESOURCE_VIEW_DESC1 {
    pub Format: super::Dxgi::Common::DXGI_FORMAT,
    pub ViewDimension: super::Direct3D::D3D_SRV_DIMENSION,
    pub Anonymous: D3D11_SHADER_RESOURCE_VIEW_DESC1_0,
}
#[cfg(all(feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common"))]
impl ::core::marker::Copy for D3D11_SHADER_RESOURCE_VIEW_DESC1 {}
#[cfg(all(feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common"))]
impl ::core::clone::Clone for D3D11_SHADER_RESOURCE_VIEW_DESC1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Direct3D\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(all(feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common"))]
pub union D3D11_SHADER_RESOURCE_VIEW_DESC1_0 {
    pub Buffer: D3D11_BUFFER_SRV,
    pub Texture1D: D3D11_TEX1D_SRV,
    pub Texture1DArray: D3D11_TEX1D_ARRAY_SRV,
    pub Texture2D: D3D11_TEX2D_SRV1,
    pub Texture2DArray: D3D11_TEX2D_ARRAY_SRV1,
    pub Texture2DMS: D3D11_TEX2DMS_SRV,
    pub Texture2DMSArray: D3D11_TEX2DMS_ARRAY_SRV,
    pub Texture3D: D3D11_TEX3D_SRV,
    pub TextureCube: D3D11_TEXCUBE_SRV,
    pub TextureCubeArray: D3D11_TEXCUBE_ARRAY_SRV,
    pub BufferEx: D3D11_BUFFEREX_SRV,
}
#[cfg(all(feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common"))]
impl ::core::marker::Copy for D3D11_SHADER_RESOURCE_VIEW_DESC1_0 {}
#[cfg(all(feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common"))]
impl ::core::clone::Clone for D3D11_SHADER_RESOURCE_VIEW_DESC1_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_SHADER_TRACE_DESC {
    pub Type: D3D11_SHADER_TYPE,
    pub Flags: u32,
    pub Anonymous: D3D11_SHADER_TRACE_DESC_0,
}
impl ::core::marker::Copy for D3D11_SHADER_TRACE_DESC {}
impl ::core::clone::Clone for D3D11_SHADER_TRACE_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub union D3D11_SHADER_TRACE_DESC_0 {
    pub VertexShaderTraceDesc: D3D11_VERTEX_SHADER_TRACE_DESC,
    pub HullShaderTraceDesc: D3D11_HULL_SHADER_TRACE_DESC,
    pub DomainShaderTraceDesc: D3D11_DOMAIN_SHADER_TRACE_DESC,
    pub GeometryShaderTraceDesc: D3D11_GEOMETRY_SHADER_TRACE_DESC,
    pub PixelShaderTraceDesc: D3D11_PIXEL_SHADER_TRACE_DESC,
    pub ComputeShaderTraceDesc: D3D11_COMPUTE_SHADER_TRACE_DESC,
}
impl ::core::marker::Copy for D3D11_SHADER_TRACE_DESC_0 {}
impl ::core::clone::Clone for D3D11_SHADER_TRACE_DESC_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Direct3D\"`*"]
#[cfg(feature = "Win32_Graphics_Direct3D")]
pub struct D3D11_SHADER_TYPE_DESC {
    pub Class: super::Direct3D::D3D_SHADER_VARIABLE_CLASS,
    pub Type: super::Direct3D::D3D_SHADER_VARIABLE_TYPE,
    pub Rows: u32,
    pub Columns: u32,
    pub Elements: u32,
    pub Members: u32,
    pub Offset: u32,
    pub Name: ::windows_sys::core::PCSTR,
}
#[cfg(feature = "Win32_Graphics_Direct3D")]
impl ::core::marker::Copy for D3D11_SHADER_TYPE_DESC {}
#[cfg(feature = "Win32_Graphics_Direct3D")]
impl ::core::clone::Clone for D3D11_SHADER_TYPE_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_SHADER_VARIABLE_DESC {
    pub Name: ::windows_sys::core::PCSTR,
    pub StartOffset: u32,
    pub Size: u32,
    pub uFlags: u32,
    pub DefaultValue: *mut ::core::ffi::c_void,
    pub StartTexture: u32,
    pub TextureSize: u32,
    pub StartSampler: u32,
    pub SamplerSize: u32,
}
impl ::core::marker::Copy for D3D11_SHADER_VARIABLE_DESC {}
impl ::core::clone::Clone for D3D11_SHADER_VARIABLE_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Direct3D\"`*"]
#[cfg(feature = "Win32_Graphics_Direct3D")]
pub struct D3D11_SIGNATURE_PARAMETER_DESC {
    pub SemanticName: ::windows_sys::core::PCSTR,
    pub SemanticIndex: u32,
    pub Register: u32,
    pub SystemValueType: super::Direct3D::D3D_NAME,
    pub ComponentType: super::Direct3D::D3D_REGISTER_COMPONENT_TYPE,
    pub Mask: u8,
    pub ReadWriteMask: u8,
    pub Stream: u32,
    pub MinPrecision: super::Direct3D::D3D_MIN_PRECISION,
}
#[cfg(feature = "Win32_Graphics_Direct3D")]
impl ::core::marker::Copy for D3D11_SIGNATURE_PARAMETER_DESC {}
#[cfg(feature = "Win32_Graphics_Direct3D")]
impl ::core::clone::Clone for D3D11_SIGNATURE_PARAMETER_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_SO_DECLARATION_ENTRY {
    pub Stream: u32,
    pub SemanticName: ::windows_sys::core::PCSTR,
    pub SemanticIndex: u32,
    pub StartComponent: u8,
    pub ComponentCount: u8,
    pub OutputSlot: u8,
}
impl ::core::marker::Copy for D3D11_SO_DECLARATION_ENTRY {}
impl ::core::clone::Clone for D3D11_SO_DECLARATION_ENTRY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_SUBRESOURCE_DATA {
    pub pSysMem: *const ::core::ffi::c_void,
    pub SysMemPitch: u32,
    pub SysMemSlicePitch: u32,
}
impl ::core::marker::Copy for D3D11_SUBRESOURCE_DATA {}
impl ::core::clone::Clone for D3D11_SUBRESOURCE_DATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_SUBRESOURCE_TILING {
    pub WidthInTiles: u32,
    pub HeightInTiles: u16,
    pub DepthInTiles: u16,
    pub StartTileIndexInOverallResource: u32,
}
impl ::core::marker::Copy for D3D11_SUBRESOURCE_TILING {}
impl ::core::clone::Clone for D3D11_SUBRESOURCE_TILING {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX1D_ARRAY_DSV {
    pub MipSlice: u32,
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
}
impl ::core::marker::Copy for D3D11_TEX1D_ARRAY_DSV {}
impl ::core::clone::Clone for D3D11_TEX1D_ARRAY_DSV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX1D_ARRAY_RTV {
    pub MipSlice: u32,
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
}
impl ::core::marker::Copy for D3D11_TEX1D_ARRAY_RTV {}
impl ::core::clone::Clone for D3D11_TEX1D_ARRAY_RTV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX1D_ARRAY_SRV {
    pub MostDetailedMip: u32,
    pub MipLevels: u32,
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
}
impl ::core::marker::Copy for D3D11_TEX1D_ARRAY_SRV {}
impl ::core::clone::Clone for D3D11_TEX1D_ARRAY_SRV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX1D_ARRAY_UAV {
    pub MipSlice: u32,
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
}
impl ::core::marker::Copy for D3D11_TEX1D_ARRAY_UAV {}
impl ::core::clone::Clone for D3D11_TEX1D_ARRAY_UAV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX1D_DSV {
    pub MipSlice: u32,
}
impl ::core::marker::Copy for D3D11_TEX1D_DSV {}
impl ::core::clone::Clone for D3D11_TEX1D_DSV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX1D_RTV {
    pub MipSlice: u32,
}
impl ::core::marker::Copy for D3D11_TEX1D_RTV {}
impl ::core::clone::Clone for D3D11_TEX1D_RTV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX1D_SRV {
    pub MostDetailedMip: u32,
    pub MipLevels: u32,
}
impl ::core::marker::Copy for D3D11_TEX1D_SRV {}
impl ::core::clone::Clone for D3D11_TEX1D_SRV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX1D_UAV {
    pub MipSlice: u32,
}
impl ::core::marker::Copy for D3D11_TEX1D_UAV {}
impl ::core::clone::Clone for D3D11_TEX1D_UAV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX2DMS_ARRAY_DSV {
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
}
impl ::core::marker::Copy for D3D11_TEX2DMS_ARRAY_DSV {}
impl ::core::clone::Clone for D3D11_TEX2DMS_ARRAY_DSV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX2DMS_ARRAY_RTV {
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
}
impl ::core::marker::Copy for D3D11_TEX2DMS_ARRAY_RTV {}
impl ::core::clone::Clone for D3D11_TEX2DMS_ARRAY_RTV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX2DMS_ARRAY_SRV {
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
}
impl ::core::marker::Copy for D3D11_TEX2DMS_ARRAY_SRV {}
impl ::core::clone::Clone for D3D11_TEX2DMS_ARRAY_SRV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX2DMS_DSV {
    pub UnusedField_NothingToDefine: u32,
}
impl ::core::marker::Copy for D3D11_TEX2DMS_DSV {}
impl ::core::clone::Clone for D3D11_TEX2DMS_DSV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX2DMS_RTV {
    pub UnusedField_NothingToDefine: u32,
}
impl ::core::marker::Copy for D3D11_TEX2DMS_RTV {}
impl ::core::clone::Clone for D3D11_TEX2DMS_RTV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX2DMS_SRV {
    pub UnusedField_NothingToDefine: u32,
}
impl ::core::marker::Copy for D3D11_TEX2DMS_SRV {}
impl ::core::clone::Clone for D3D11_TEX2DMS_SRV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX2D_ARRAY_DSV {
    pub MipSlice: u32,
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
}
impl ::core::marker::Copy for D3D11_TEX2D_ARRAY_DSV {}
impl ::core::clone::Clone for D3D11_TEX2D_ARRAY_DSV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX2D_ARRAY_RTV {
    pub MipSlice: u32,
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
}
impl ::core::marker::Copy for D3D11_TEX2D_ARRAY_RTV {}
impl ::core::clone::Clone for D3D11_TEX2D_ARRAY_RTV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX2D_ARRAY_RTV1 {
    pub MipSlice: u32,
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
    pub PlaneSlice: u32,
}
impl ::core::marker::Copy for D3D11_TEX2D_ARRAY_RTV1 {}
impl ::core::clone::Clone for D3D11_TEX2D_ARRAY_RTV1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX2D_ARRAY_SRV {
    pub MostDetailedMip: u32,
    pub MipLevels: u32,
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
}
impl ::core::marker::Copy for D3D11_TEX2D_ARRAY_SRV {}
impl ::core::clone::Clone for D3D11_TEX2D_ARRAY_SRV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX2D_ARRAY_SRV1 {
    pub MostDetailedMip: u32,
    pub MipLevels: u32,
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
    pub PlaneSlice: u32,
}
impl ::core::marker::Copy for D3D11_TEX2D_ARRAY_SRV1 {}
impl ::core::clone::Clone for D3D11_TEX2D_ARRAY_SRV1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX2D_ARRAY_UAV {
    pub MipSlice: u32,
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
}
impl ::core::marker::Copy for D3D11_TEX2D_ARRAY_UAV {}
impl ::core::clone::Clone for D3D11_TEX2D_ARRAY_UAV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX2D_ARRAY_UAV1 {
    pub MipSlice: u32,
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
    pub PlaneSlice: u32,
}
impl ::core::marker::Copy for D3D11_TEX2D_ARRAY_UAV1 {}
impl ::core::clone::Clone for D3D11_TEX2D_ARRAY_UAV1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX2D_ARRAY_VPOV {
    pub MipSlice: u32,
    pub FirstArraySlice: u32,
    pub ArraySize: u32,
}
impl ::core::marker::Copy for D3D11_TEX2D_ARRAY_VPOV {}
impl ::core::clone::Clone for D3D11_TEX2D_ARRAY_VPOV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX2D_DSV {
    pub MipSlice: u32,
}
impl ::core::marker::Copy for D3D11_TEX2D_DSV {}
impl ::core::clone::Clone for D3D11_TEX2D_DSV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX2D_RTV {
    pub MipSlice: u32,
}
impl ::core::marker::Copy for D3D11_TEX2D_RTV {}
impl ::core::clone::Clone for D3D11_TEX2D_RTV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX2D_RTV1 {
    pub MipSlice: u32,
    pub PlaneSlice: u32,
}
impl ::core::marker::Copy for D3D11_TEX2D_RTV1 {}
impl ::core::clone::Clone for D3D11_TEX2D_RTV1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX2D_SRV {
    pub MostDetailedMip: u32,
    pub MipLevels: u32,
}
impl ::core::marker::Copy for D3D11_TEX2D_SRV {}
impl ::core::clone::Clone for D3D11_TEX2D_SRV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX2D_SRV1 {
    pub MostDetailedMip: u32,
    pub MipLevels: u32,
    pub PlaneSlice: u32,
}
impl ::core::marker::Copy for D3D11_TEX2D_SRV1 {}
impl ::core::clone::Clone for D3D11_TEX2D_SRV1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX2D_UAV {
    pub MipSlice: u32,
}
impl ::core::marker::Copy for D3D11_TEX2D_UAV {}
impl ::core::clone::Clone for D3D11_TEX2D_UAV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX2D_UAV1 {
    pub MipSlice: u32,
    pub PlaneSlice: u32,
}
impl ::core::marker::Copy for D3D11_TEX2D_UAV1 {}
impl ::core::clone::Clone for D3D11_TEX2D_UAV1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX2D_VDOV {
    pub ArraySlice: u32,
}
impl ::core::marker::Copy for D3D11_TEX2D_VDOV {}
impl ::core::clone::Clone for D3D11_TEX2D_VDOV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX2D_VPIV {
    pub MipSlice: u32,
    pub ArraySlice: u32,
}
impl ::core::marker::Copy for D3D11_TEX2D_VPIV {}
impl ::core::clone::Clone for D3D11_TEX2D_VPIV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX2D_VPOV {
    pub MipSlice: u32,
}
impl ::core::marker::Copy for D3D11_TEX2D_VPOV {}
impl ::core::clone::Clone for D3D11_TEX2D_VPOV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX3D_RTV {
    pub MipSlice: u32,
    pub FirstWSlice: u32,
    pub WSize: u32,
}
impl ::core::marker::Copy for D3D11_TEX3D_RTV {}
impl ::core::clone::Clone for D3D11_TEX3D_RTV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX3D_SRV {
    pub MostDetailedMip: u32,
    pub MipLevels: u32,
}
impl ::core::marker::Copy for D3D11_TEX3D_SRV {}
impl ::core::clone::Clone for D3D11_TEX3D_SRV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEX3D_UAV {
    pub MipSlice: u32,
    pub FirstWSlice: u32,
    pub WSize: u32,
}
impl ::core::marker::Copy for D3D11_TEX3D_UAV {}
impl ::core::clone::Clone for D3D11_TEX3D_UAV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEXCUBE_ARRAY_SRV {
    pub MostDetailedMip: u32,
    pub MipLevels: u32,
    pub First2DArrayFace: u32,
    pub NumCubes: u32,
}
impl ::core::marker::Copy for D3D11_TEXCUBE_ARRAY_SRV {}
impl ::core::clone::Clone for D3D11_TEXCUBE_ARRAY_SRV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TEXCUBE_SRV {
    pub MostDetailedMip: u32,
    pub MipLevels: u32,
}
impl ::core::marker::Copy for D3D11_TEXCUBE_SRV {}
impl ::core::clone::Clone for D3D11_TEXCUBE_SRV {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub struct D3D11_TEXTURE1D_DESC {
    pub Width: u32,
    pub MipLevels: u32,
    pub ArraySize: u32,
    pub Format: super::Dxgi::Common::DXGI_FORMAT,
    pub Usage: D3D11_USAGE,
    pub BindFlags: D3D11_BIND_FLAG,
    pub CPUAccessFlags: D3D11_CPU_ACCESS_FLAG,
    pub MiscFlags: D3D11_RESOURCE_MISC_FLAG,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D11_TEXTURE1D_DESC {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D11_TEXTURE1D_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub struct D3D11_TEXTURE2D_DESC {
    pub Width: u32,
    pub Height: u32,
    pub MipLevels: u32,
    pub ArraySize: u32,
    pub Format: super::Dxgi::Common::DXGI_FORMAT,
    pub SampleDesc: super::Dxgi::Common::DXGI_SAMPLE_DESC,
    pub Usage: D3D11_USAGE,
    pub BindFlags: D3D11_BIND_FLAG,
    pub CPUAccessFlags: D3D11_CPU_ACCESS_FLAG,
    pub MiscFlags: D3D11_RESOURCE_MISC_FLAG,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D11_TEXTURE2D_DESC {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D11_TEXTURE2D_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub struct D3D11_TEXTURE2D_DESC1 {
    pub Width: u32,
    pub Height: u32,
    pub MipLevels: u32,
    pub ArraySize: u32,
    pub Format: super::Dxgi::Common::DXGI_FORMAT,
    pub SampleDesc: super::Dxgi::Common::DXGI_SAMPLE_DESC,
    pub Usage: D3D11_USAGE,
    pub BindFlags: D3D11_BIND_FLAG,
    pub CPUAccessFlags: D3D11_CPU_ACCESS_FLAG,
    pub MiscFlags: D3D11_RESOURCE_MISC_FLAG,
    pub TextureLayout: D3D11_TEXTURE_LAYOUT,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D11_TEXTURE2D_DESC1 {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D11_TEXTURE2D_DESC1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub struct D3D11_TEXTURE3D_DESC {
    pub Width: u32,
    pub Height: u32,
    pub Depth: u32,
    pub MipLevels: u32,
    pub Format: super::Dxgi::Common::DXGI_FORMAT,
    pub Usage: D3D11_USAGE,
    pub BindFlags: D3D11_BIND_FLAG,
    pub CPUAccessFlags: D3D11_CPU_ACCESS_FLAG,
    pub MiscFlags: D3D11_RESOURCE_MISC_FLAG,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D11_TEXTURE3D_DESC {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D11_TEXTURE3D_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub struct D3D11_TEXTURE3D_DESC1 {
    pub Width: u32,
    pub Height: u32,
    pub Depth: u32,
    pub MipLevels: u32,
    pub Format: super::Dxgi::Common::DXGI_FORMAT,
    pub Usage: D3D11_USAGE,
    pub BindFlags: D3D11_BIND_FLAG,
    pub CPUAccessFlags: D3D11_CPU_ACCESS_FLAG,
    pub MiscFlags: D3D11_RESOURCE_MISC_FLAG,
    pub TextureLayout: D3D11_TEXTURE_LAYOUT,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D11_TEXTURE3D_DESC1 {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D11_TEXTURE3D_DESC1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TILED_RESOURCE_COORDINATE {
    pub X: u32,
    pub Y: u32,
    pub Z: u32,
    pub Subresource: u32,
}
impl ::core::marker::Copy for D3D11_TILED_RESOURCE_COORDINATE {}
impl ::core::clone::Clone for D3D11_TILED_RESOURCE_COORDINATE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_TILE_REGION_SIZE {
    pub NumTiles: u32,
    pub bUseBox: super::super::Foundation::BOOL,
    pub Width: u32,
    pub Height: u16,
    pub Depth: u16,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_TILE_REGION_SIZE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_TILE_REGION_SIZE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TILE_SHAPE {
    pub WidthInTexels: u32,
    pub HeightInTexels: u32,
    pub DepthInTexels: u32,
}
impl ::core::marker::Copy for D3D11_TILE_SHAPE {}
impl ::core::clone::Clone for D3D11_TILE_SHAPE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TRACE_REGISTER {
    pub RegType: D3D11_TRACE_REGISTER_TYPE,
    pub Anonymous: D3D11_TRACE_REGISTER_0,
    pub OperandIndex: u8,
    pub Flags: u8,
}
impl ::core::marker::Copy for D3D11_TRACE_REGISTER {}
impl ::core::clone::Clone for D3D11_TRACE_REGISTER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub union D3D11_TRACE_REGISTER_0 {
    pub Index1D: u16,
    pub Index2D: [u16; 2],
}
impl ::core::marker::Copy for D3D11_TRACE_REGISTER_0 {}
impl ::core::clone::Clone for D3D11_TRACE_REGISTER_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_TRACE_STATS {
    pub TraceDesc: D3D11_SHADER_TRACE_DESC,
    pub NumInvocationsInStamp: u8,
    pub TargetStampIndex: u8,
    pub NumTraceSteps: u32,
    pub InputMask: [u8; 32],
    pub OutputMask: [u8; 32],
    pub NumTemps: u16,
    pub MaxIndexableTempIndex: u16,
    pub IndexableTempSize: [u16; 4096],
    pub ImmediateConstantBufferSize: u16,
    pub PixelPosition: [u32; 8],
    pub PixelCoverageMask: [u64; 4],
    pub PixelDiscardedMask: [u64; 4],
    pub PixelCoverageMaskAfterShader: [u64; 4],
    pub PixelCoverageMaskAfterA2CSampleMask: [u64; 4],
    pub PixelCoverageMaskAfterA2CSampleMaskDepth: [u64; 4],
    pub PixelCoverageMaskAfterA2CSampleMaskDepthStencil: [u64; 4],
    pub PSOutputsDepth: super::super::Foundation::BOOL,
    pub PSOutputsMask: super::super::Foundation::BOOL,
    pub GSInputPrimitive: D3D11_TRACE_GS_INPUT_PRIMITIVE,
    pub GSInputsPrimitiveID: super::super::Foundation::BOOL,
    pub HSOutputPatchConstantMask: [u8; 32],
    pub DSInputPatchConstantMask: [u8; 32],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_TRACE_STATS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_TRACE_STATS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_TRACE_STEP {
    pub ID: u32,
    pub InstructionActive: super::super::Foundation::BOOL,
    pub NumRegistersWritten: u8,
    pub NumRegistersRead: u8,
    pub MiscOperations: u16,
    pub OpcodeType: u32,
    pub CurrentGlobalCycle: u64,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_TRACE_STEP {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_TRACE_STEP {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_TRACE_VALUE {
    pub Bits: [u32; 4],
    pub ValidMask: u8,
}
impl ::core::marker::Copy for D3D11_TRACE_VALUE {}
impl ::core::clone::Clone for D3D11_TRACE_VALUE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub struct D3D11_UNORDERED_ACCESS_VIEW_DESC {
    pub Format: super::Dxgi::Common::DXGI_FORMAT,
    pub ViewDimension: D3D11_UAV_DIMENSION,
    pub Anonymous: D3D11_UNORDERED_ACCESS_VIEW_DESC_0,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D11_UNORDERED_ACCESS_VIEW_DESC {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D11_UNORDERED_ACCESS_VIEW_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub union D3D11_UNORDERED_ACCESS_VIEW_DESC_0 {
    pub Buffer: D3D11_BUFFER_UAV,
    pub Texture1D: D3D11_TEX1D_UAV,
    pub Texture1DArray: D3D11_TEX1D_ARRAY_UAV,
    pub Texture2D: D3D11_TEX2D_UAV,
    pub Texture2DArray: D3D11_TEX2D_ARRAY_UAV,
    pub Texture3D: D3D11_TEX3D_UAV,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D11_UNORDERED_ACCESS_VIEW_DESC_0 {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D11_UNORDERED_ACCESS_VIEW_DESC_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub struct D3D11_UNORDERED_ACCESS_VIEW_DESC1 {
    pub Format: super::Dxgi::Common::DXGI_FORMAT,
    pub ViewDimension: D3D11_UAV_DIMENSION,
    pub Anonymous: D3D11_UNORDERED_ACCESS_VIEW_DESC1_0,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D11_UNORDERED_ACCESS_VIEW_DESC1 {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D11_UNORDERED_ACCESS_VIEW_DESC1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub union D3D11_UNORDERED_ACCESS_VIEW_DESC1_0 {
    pub Buffer: D3D11_BUFFER_UAV,
    pub Texture1D: D3D11_TEX1D_UAV,
    pub Texture1DArray: D3D11_TEX1D_ARRAY_UAV,
    pub Texture2D: D3D11_TEX2D_UAV1,
    pub Texture2DArray: D3D11_TEX2D_ARRAY_UAV1,
    pub Texture3D: D3D11_TEX3D_UAV,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D11_UNORDERED_ACCESS_VIEW_DESC1_0 {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D11_UNORDERED_ACCESS_VIEW_DESC1_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_VERTEX_SHADER_TRACE_DESC {
    pub Invocation: u64,
}
impl ::core::marker::Copy for D3D11_VERTEX_SHADER_TRACE_DESC {}
impl ::core::clone::Clone for D3D11_VERTEX_SHADER_TRACE_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_VIDEO_COLOR {
    pub Anonymous: D3D11_VIDEO_COLOR_0,
}
impl ::core::marker::Copy for D3D11_VIDEO_COLOR {}
impl ::core::clone::Clone for D3D11_VIDEO_COLOR {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub union D3D11_VIDEO_COLOR_0 {
    pub YCbCr: D3D11_VIDEO_COLOR_YCbCrA,
    pub RGBA: D3D11_VIDEO_COLOR_RGBA,
}
impl ::core::marker::Copy for D3D11_VIDEO_COLOR_0 {}
impl ::core::clone::Clone for D3D11_VIDEO_COLOR_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_VIDEO_COLOR_RGBA {
    pub R: f32,
    pub G: f32,
    pub B: f32,
    pub A: f32,
}
impl ::core::marker::Copy for D3D11_VIDEO_COLOR_RGBA {}
impl ::core::clone::Clone for D3D11_VIDEO_COLOR_RGBA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_VIDEO_COLOR_YCbCrA {
    pub Y: f32,
    pub Cb: f32,
    pub Cr: f32,
    pub A: f32,
}
impl ::core::marker::Copy for D3D11_VIDEO_COLOR_YCbCrA {}
impl ::core::clone::Clone for D3D11_VIDEO_COLOR_YCbCrA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_VIDEO_CONTENT_PROTECTION_CAPS {
    pub Caps: u32,
    pub KeyExchangeTypeCount: u32,
    pub BlockAlignmentSize: u32,
    pub ProtectedMemorySize: u64,
}
impl ::core::marker::Copy for D3D11_VIDEO_CONTENT_PROTECTION_CAPS {}
impl ::core::clone::Clone for D3D11_VIDEO_CONTENT_PROTECTION_CAPS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_VIDEO_DECODER_BEGIN_FRAME_CRYPTO_SESSION {
    pub pCryptoSession: ID3D11CryptoSession,
    pub BlobSize: u32,
    pub pBlob: *mut ::core::ffi::c_void,
    pub pKeyInfoId: *mut ::windows_sys::core::GUID,
    pub PrivateDataSize: u32,
    pub pPrivateData: *mut ::core::ffi::c_void,
}
impl ::core::marker::Copy for D3D11_VIDEO_DECODER_BEGIN_FRAME_CRYPTO_SESSION {}
impl ::core::clone::Clone for D3D11_VIDEO_DECODER_BEGIN_FRAME_CRYPTO_SESSION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_VIDEO_DECODER_BUFFER_DESC {
    pub BufferType: D3D11_VIDEO_DECODER_BUFFER_TYPE,
    pub BufferIndex: u32,
    pub DataOffset: u32,
    pub DataSize: u32,
    pub FirstMBaddress: u32,
    pub NumMBsInBuffer: u32,
    pub Width: u32,
    pub Height: u32,
    pub Stride: u32,
    pub ReservedBits: u32,
    pub pIV: *mut ::core::ffi::c_void,
    pub IVSize: u32,
    pub PartialEncryption: super::super::Foundation::BOOL,
    pub EncryptedBlockInfo: D3D11_ENCRYPTED_BLOCK_INFO,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_VIDEO_DECODER_BUFFER_DESC {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_VIDEO_DECODER_BUFFER_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_VIDEO_DECODER_BUFFER_DESC1 {
    pub BufferType: D3D11_VIDEO_DECODER_BUFFER_TYPE,
    pub DataOffset: u32,
    pub DataSize: u32,
    pub pIV: *mut ::core::ffi::c_void,
    pub IVSize: u32,
    pub pSubSampleMappingBlock: *mut D3D11_VIDEO_DECODER_SUB_SAMPLE_MAPPING_BLOCK,
    pub SubSampleMappingCount: u32,
}
impl ::core::marker::Copy for D3D11_VIDEO_DECODER_BUFFER_DESC1 {}
impl ::core::clone::Clone for D3D11_VIDEO_DECODER_BUFFER_DESC1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_VIDEO_DECODER_BUFFER_DESC2 {
    pub BufferType: D3D11_VIDEO_DECODER_BUFFER_TYPE,
    pub DataOffset: u32,
    pub DataSize: u32,
    pub pIV: *mut ::core::ffi::c_void,
    pub IVSize: u32,
    pub pSubSampleMappingBlock: *mut D3D11_VIDEO_DECODER_SUB_SAMPLE_MAPPING_BLOCK,
    pub SubSampleMappingCount: u32,
    pub cBlocksStripeEncrypted: u32,
    pub cBlocksStripeClear: u32,
}
impl ::core::marker::Copy for D3D11_VIDEO_DECODER_BUFFER_DESC2 {}
impl ::core::clone::Clone for D3D11_VIDEO_DECODER_BUFFER_DESC2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_VIDEO_DECODER_CONFIG {
    pub guidConfigBitstreamEncryption: ::windows_sys::core::GUID,
    pub guidConfigMBcontrolEncryption: ::windows_sys::core::GUID,
    pub guidConfigResidDiffEncryption: ::windows_sys::core::GUID,
    pub ConfigBitstreamRaw: u32,
    pub ConfigMBcontrolRasterOrder: u32,
    pub ConfigResidDiffHost: u32,
    pub ConfigSpatialResid8: u32,
    pub ConfigResid8Subtraction: u32,
    pub ConfigSpatialHost8or9Clipping: u32,
    pub ConfigSpatialResidInterleaved: u32,
    pub ConfigIntraResidUnsigned: u32,
    pub ConfigResidDiffAccelerator: u32,
    pub ConfigHostInverseScan: u32,
    pub ConfigSpecificIDCT: u32,
    pub Config4GroupedCoefs: u32,
    pub ConfigMinRenderTargetBuffCount: u16,
    pub ConfigDecoderSpecific: u16,
}
impl ::core::marker::Copy for D3D11_VIDEO_DECODER_CONFIG {}
impl ::core::clone::Clone for D3D11_VIDEO_DECODER_CONFIG {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub struct D3D11_VIDEO_DECODER_DESC {
    pub Guid: ::windows_sys::core::GUID,
    pub SampleWidth: u32,
    pub SampleHeight: u32,
    pub OutputFormat: super::Dxgi::Common::DXGI_FORMAT,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D11_VIDEO_DECODER_DESC {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D11_VIDEO_DECODER_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_VIDEO_DECODER_EXTENSION {
    pub Function: u32,
    pub pPrivateInputData: *mut ::core::ffi::c_void,
    pub PrivateInputDataSize: u32,
    pub pPrivateOutputData: *mut ::core::ffi::c_void,
    pub PrivateOutputDataSize: u32,
    pub ResourceCount: u32,
    pub ppResourceList: *mut ID3D11Resource,
}
impl ::core::marker::Copy for D3D11_VIDEO_DECODER_EXTENSION {}
impl ::core::clone::Clone for D3D11_VIDEO_DECODER_EXTENSION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_VIDEO_DECODER_OUTPUT_VIEW_DESC {
    pub DecodeProfile: ::windows_sys::core::GUID,
    pub ViewDimension: D3D11_VDOV_DIMENSION,
    pub Anonymous: D3D11_VIDEO_DECODER_OUTPUT_VIEW_DESC_0,
}
impl ::core::marker::Copy for D3D11_VIDEO_DECODER_OUTPUT_VIEW_DESC {}
impl ::core::clone::Clone for D3D11_VIDEO_DECODER_OUTPUT_VIEW_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub union D3D11_VIDEO_DECODER_OUTPUT_VIEW_DESC_0 {
    pub Texture2D: D3D11_TEX2D_VDOV,
}
impl ::core::marker::Copy for D3D11_VIDEO_DECODER_OUTPUT_VIEW_DESC_0 {}
impl ::core::clone::Clone for D3D11_VIDEO_DECODER_OUTPUT_VIEW_DESC_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_VIDEO_DECODER_SUB_SAMPLE_MAPPING_BLOCK {
    pub ClearSize: u32,
    pub EncryptedSize: u32,
}
impl ::core::marker::Copy for D3D11_VIDEO_DECODER_SUB_SAMPLE_MAPPING_BLOCK {}
impl ::core::clone::Clone for D3D11_VIDEO_DECODER_SUB_SAMPLE_MAPPING_BLOCK {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_VIDEO_PROCESSOR_CAPS {
    pub DeviceCaps: u32,
    pub FeatureCaps: u32,
    pub FilterCaps: u32,
    pub InputFormatCaps: u32,
    pub AutoStreamCaps: u32,
    pub StereoCaps: u32,
    pub RateConversionCapsCount: u32,
    pub MaxInputStreams: u32,
    pub MaxStreamStates: u32,
}
impl ::core::marker::Copy for D3D11_VIDEO_PROCESSOR_CAPS {}
impl ::core::clone::Clone for D3D11_VIDEO_PROCESSOR_CAPS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_VIDEO_PROCESSOR_COLOR_SPACE {
    pub _bitfield: u32,
}
impl ::core::marker::Copy for D3D11_VIDEO_PROCESSOR_COLOR_SPACE {}
impl ::core::clone::Clone for D3D11_VIDEO_PROCESSOR_COLOR_SPACE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub struct D3D11_VIDEO_PROCESSOR_CONTENT_DESC {
    pub InputFrameFormat: D3D11_VIDEO_FRAME_FORMAT,
    pub InputFrameRate: super::Dxgi::Common::DXGI_RATIONAL,
    pub InputWidth: u32,
    pub InputHeight: u32,
    pub OutputFrameRate: super::Dxgi::Common::DXGI_RATIONAL,
    pub OutputWidth: u32,
    pub OutputHeight: u32,
    pub Usage: D3D11_VIDEO_USAGE,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D11_VIDEO_PROCESSOR_CONTENT_DESC {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D11_VIDEO_PROCESSOR_CONTENT_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Dxgi_Common"))]
pub struct D3D11_VIDEO_PROCESSOR_CUSTOM_RATE {
    pub CustomRate: super::Dxgi::Common::DXGI_RATIONAL,
    pub OutputFrames: u32,
    pub InputInterlaced: super::super::Foundation::BOOL,
    pub InputFramesOrFields: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Dxgi_Common"))]
impl ::core::marker::Copy for D3D11_VIDEO_PROCESSOR_CUSTOM_RATE {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Dxgi_Common"))]
impl ::core::clone::Clone for D3D11_VIDEO_PROCESSOR_CUSTOM_RATE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_VIDEO_PROCESSOR_FILTER_RANGE {
    pub Minimum: i32,
    pub Maximum: i32,
    pub Default: i32,
    pub Multiplier: f32,
}
impl ::core::marker::Copy for D3D11_VIDEO_PROCESSOR_FILTER_RANGE {}
impl ::core::clone::Clone for D3D11_VIDEO_PROCESSOR_FILTER_RANGE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC {
    pub FourCC: u32,
    pub ViewDimension: D3D11_VPIV_DIMENSION,
    pub Anonymous: D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC_0,
}
impl ::core::marker::Copy for D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC {}
impl ::core::clone::Clone for D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub union D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC_0 {
    pub Texture2D: D3D11_TEX2D_VPIV,
}
impl ::core::marker::Copy for D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC_0 {}
impl ::core::clone::Clone for D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC {
    pub ViewDimension: D3D11_VPOV_DIMENSION,
    pub Anonymous: D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC_0,
}
impl ::core::marker::Copy for D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC {}
impl ::core::clone::Clone for D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub union D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC_0 {
    pub Texture2D: D3D11_TEX2D_VPOV,
    pub Texture2DArray: D3D11_TEX2D_ARRAY_VPOV,
}
impl ::core::marker::Copy for D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC_0 {}
impl ::core::clone::Clone for D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_VIDEO_PROCESSOR_RATE_CONVERSION_CAPS {
    pub PastFrames: u32,
    pub FutureFrames: u32,
    pub ProcessorCaps: u32,
    pub ITelecineCaps: u32,
    pub CustomRateCount: u32,
}
impl ::core::marker::Copy for D3D11_VIDEO_PROCESSOR_RATE_CONVERSION_CAPS {}
impl ::core::clone::Clone for D3D11_VIDEO_PROCESSOR_RATE_CONVERSION_CAPS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct D3D11_VIDEO_PROCESSOR_STREAM {
    pub Enable: super::super::Foundation::BOOL,
    pub OutputIndex: u32,
    pub InputFrameOrField: u32,
    pub PastFrames: u32,
    pub FutureFrames: u32,
    pub ppPastSurfaces: *mut ID3D11VideoProcessorInputView,
    pub pInputSurface: ID3D11VideoProcessorInputView,
    pub ppFutureSurfaces: *mut ID3D11VideoProcessorInputView,
    pub ppPastSurfacesRight: *mut ID3D11VideoProcessorInputView,
    pub pInputSurfaceRight: ID3D11VideoProcessorInputView,
    pub ppFutureSurfacesRight: *mut ID3D11VideoProcessorInputView,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for D3D11_VIDEO_PROCESSOR_STREAM {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for D3D11_VIDEO_PROCESSOR_STREAM {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Dxgi_Common"))]
pub struct D3D11_VIDEO_PROCESSOR_STREAM_BEHAVIOR_HINT {
    pub Enable: super::super::Foundation::BOOL,
    pub Width: u32,
    pub Height: u32,
    pub Format: super::Dxgi::Common::DXGI_FORMAT,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Dxgi_Common"))]
impl ::core::marker::Copy for D3D11_VIDEO_PROCESSOR_STREAM_BEHAVIOR_HINT {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Dxgi_Common"))]
impl ::core::clone::Clone for D3D11_VIDEO_PROCESSOR_STREAM_BEHAVIOR_HINT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub struct D3D11_VIDEO_SAMPLE_DESC {
    pub Width: u32,
    pub Height: u32,
    pub Format: super::Dxgi::Common::DXGI_FORMAT,
    pub ColorSpace: super::Dxgi::Common::DXGI_COLOR_SPACE_TYPE,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for D3D11_VIDEO_SAMPLE_DESC {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for D3D11_VIDEO_SAMPLE_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3D11_VIEWPORT {
    pub TopLeftX: f32,
    pub TopLeftY: f32,
    pub Width: f32,
    pub Height: f32,
    pub MinDepth: f32,
    pub MaxDepth: f32,
}
impl ::core::marker::Copy for D3D11_VIEWPORT {}
impl ::core::clone::Clone for D3D11_VIEWPORT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3DX11_FFT_BUFFER_INFO {
    pub NumTempBufferSizes: u32,
    pub TempBufferFloatSizes: [u32; 4],
    pub NumPrecomputeBufferSizes: u32,
    pub PrecomputeBufferFloatSizes: [u32; 4],
}
impl ::core::marker::Copy for D3DX11_FFT_BUFFER_INFO {}
impl ::core::clone::Clone for D3DX11_FFT_BUFFER_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`*"]
pub struct D3DX11_FFT_DESC {
    pub NumDimensions: u32,
    pub ElementLengths: [u32; 32],
    pub DimensionMask: u32,
    pub Type: D3DX11_FFT_DATA_TYPE,
}
impl ::core::marker::Copy for D3DX11_FFT_DESC {}
impl ::core::clone::Clone for D3DX11_FFT_DESC {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Direct3D\"`, `\"Win32_Graphics_Dxgi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi"))]
pub type PFN_D3D11_CREATE_DEVICE = ::core::option::Option<unsafe extern "system" fn(param0: super::Dxgi::IDXGIAdapter, param1: super::Direct3D::D3D_DRIVER_TYPE, param2: super::super::Foundation::HINSTANCE, param3: u32, param4: *const super::Direct3D::D3D_FEATURE_LEVEL, featurelevels: u32, param6: u32, param7: *mut ID3D11Device, param8: *mut super::Direct3D::D3D_FEATURE_LEVEL, param9: *mut ID3D11DeviceContext) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Direct3D\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Dxgi_Common"))]
pub type PFN_D3D11_CREATE_DEVICE_AND_SWAP_CHAIN = ::core::option::Option<unsafe extern "system" fn(param0: super::Dxgi::IDXGIAdapter, param1: super::Direct3D::D3D_DRIVER_TYPE, param2: super::super::Foundation::HINSTANCE, param3: u32, param4: *const super::Direct3D::D3D_FEATURE_LEVEL, featurelevels: u32, param6: u32, param7: *const super::Dxgi::DXGI_SWAP_CHAIN_DESC, param8: *mut super::Dxgi::IDXGISwapChain, param9: *mut ID3D11Device, param10: *mut super::Direct3D::D3D_FEATURE_LEVEL, param11: *mut ID3D11DeviceContext) -> ::windows_sys::core::HRESULT>;

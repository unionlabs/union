#[link(name = "windows")]
extern "system" {
    #[doc = "*Required features: `\"Win32_System_WinRT_Direct3D11\"`, `\"Win32_Graphics_Dxgi\"`*"]
    #[cfg(feature = "Win32_Graphics_Dxgi")]
    pub fn CreateDirect3D11DeviceFromDXGIDevice(dxgidevice: super::super::super::Graphics::Dxgi::IDXGIDevice, graphicsdevice: *mut ::windows_sys::core::IInspectable) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT_Direct3D11\"`, `\"Win32_Graphics_Dxgi\"`*"]
    #[cfg(feature = "Win32_Graphics_Dxgi")]
    pub fn CreateDirect3D11SurfaceFromDXGISurface(dgxisurface: super::super::super::Graphics::Dxgi::IDXGISurface, graphicssurface: *mut ::windows_sys::core::IInspectable) -> ::windows_sys::core::HRESULT;
}
pub type IDirect3DDxgiInterfaceAccess = *mut ::core::ffi::c_void;

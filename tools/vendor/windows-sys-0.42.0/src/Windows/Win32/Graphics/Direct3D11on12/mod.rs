#[cfg_attr(windows, link(name = "windows"))]
extern "system" {
    #[doc = "*Required features: `\"Win32_Graphics_Direct3D11on12\"`, `\"Win32_Graphics_Direct3D\"`, `\"Win32_Graphics_Direct3D11\"`*"]
    #[cfg(all(feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Direct3D11"))]
    pub fn D3D11On12CreateDevice(pdevice: ::windows_sys::core::IUnknown, flags: u32, pfeaturelevels: *const super::Direct3D::D3D_FEATURE_LEVEL, featurelevels: u32, ppcommandqueues: *const ::windows_sys::core::IUnknown, numqueues: u32, nodemask: u32, ppdevice: *mut super::Direct3D11::ID3D11Device, ppimmediatecontext: *mut super::Direct3D11::ID3D11DeviceContext, pchosenfeaturelevel: *mut super::Direct3D::D3D_FEATURE_LEVEL) -> ::windows_sys::core::HRESULT;
}
pub type ID3D11On12Device = *mut ::core::ffi::c_void;
pub type ID3D11On12Device1 = *mut ::core::ffi::c_void;
pub type ID3D11On12Device2 = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11on12\"`*"]
pub struct D3D11_RESOURCE_FLAGS {
    pub BindFlags: u32,
    pub MiscFlags: u32,
    pub CPUAccessFlags: u32,
    pub StructureByteStride: u32,
}
impl ::core::marker::Copy for D3D11_RESOURCE_FLAGS {}
impl ::core::clone::Clone for D3D11_RESOURCE_FLAGS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_Direct3D11on12\"`, `\"Win32_Graphics_Direct3D\"`, `\"Win32_Graphics_Direct3D11\"`*"]
#[cfg(all(feature = "Win32_Graphics_Direct3D", feature = "Win32_Graphics_Direct3D11"))]
pub type PFN_D3D11ON12_CREATE_DEVICE = ::core::option::Option<unsafe extern "system" fn(param0: ::windows_sys::core::IUnknown, param1: u32, param2: *const super::Direct3D::D3D_FEATURE_LEVEL, featurelevels: u32, param4: *const ::windows_sys::core::IUnknown, numqueues: u32, param6: u32, param7: *mut super::Direct3D11::ID3D11Device, param8: *mut super::Direct3D11::ID3D11DeviceContext, param9: *mut super::Direct3D::D3D_FEATURE_LEVEL) -> ::windows_sys::core::HRESULT>;

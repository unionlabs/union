#[doc = "*Required features: `\"Graphics_DirectX_Direct3D11\"`*"]
#[repr(transparent)]
pub struct Direct3DBindings(pub u32);
impl Direct3DBindings {
    pub const VertexBuffer: Self = Self(1u32);
    pub const IndexBuffer: Self = Self(2u32);
    pub const ConstantBuffer: Self = Self(4u32);
    pub const ShaderResource: Self = Self(8u32);
    pub const StreamOutput: Self = Self(16u32);
    pub const RenderTarget: Self = Self(32u32);
    pub const DepthStencil: Self = Self(64u32);
    pub const UnorderedAccess: Self = Self(128u32);
    pub const Decoder: Self = Self(512u32);
    pub const VideoEncoder: Self = Self(1024u32);
}
impl ::core::marker::Copy for Direct3DBindings {}
impl ::core::clone::Clone for Direct3DBindings {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Graphics_DirectX_Direct3D11\"`*"]
pub struct Direct3DMultisampleDescription {
    pub Count: i32,
    pub Quality: i32,
}
impl ::core::marker::Copy for Direct3DMultisampleDescription {}
impl ::core::clone::Clone for Direct3DMultisampleDescription {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Graphics_DirectX_Direct3D11\"`*"]
pub struct Direct3DSurfaceDescription {
    pub Width: i32,
    pub Height: i32,
    pub Format: super::DirectXPixelFormat,
    pub MultisampleDescription: Direct3DMultisampleDescription,
}
impl ::core::marker::Copy for Direct3DSurfaceDescription {}
impl ::core::clone::Clone for Direct3DSurfaceDescription {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Graphics_DirectX_Direct3D11\"`*"]
#[repr(transparent)]
pub struct Direct3DUsage(pub i32);
impl Direct3DUsage {
    pub const Default: Self = Self(0i32);
    pub const Immutable: Self = Self(1i32);
    pub const Dynamic: Self = Self(2i32);
    pub const Staging: Self = Self(3i32);
}
impl ::core::marker::Copy for Direct3DUsage {}
impl ::core::clone::Clone for Direct3DUsage {
    fn clone(&self) -> Self {
        *self
    }
}
pub type IDirect3DDevice = *mut ::core::ffi::c_void;
pub type IDirect3DSurface = *mut ::core::ffi::c_void;

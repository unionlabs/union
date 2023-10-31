#[doc = "*Required features: `\"UI_Composition_Scenes\"`*"]
#[repr(transparent)]
pub struct SceneAlphaMode(pub i32);
impl SceneAlphaMode {
    pub const Opaque: Self = Self(0i32);
    pub const AlphaTest: Self = Self(1i32);
    pub const Blend: Self = Self(2i32);
}
impl ::core::marker::Copy for SceneAlphaMode {}
impl ::core::clone::Clone for SceneAlphaMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Composition_Scenes\"`*"]
#[repr(transparent)]
pub struct SceneAttributeSemantic(pub i32);
impl SceneAttributeSemantic {
    pub const Index: Self = Self(0i32);
    pub const Vertex: Self = Self(1i32);
    pub const Normal: Self = Self(2i32);
    pub const TexCoord0: Self = Self(3i32);
    pub const TexCoord1: Self = Self(4i32);
    pub const Color: Self = Self(5i32);
    pub const Tangent: Self = Self(6i32);
}
impl ::core::marker::Copy for SceneAttributeSemantic {}
impl ::core::clone::Clone for SceneAttributeSemantic {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SceneBoundingBox = *mut ::core::ffi::c_void;
pub type SceneComponent = *mut ::core::ffi::c_void;
pub type SceneComponentCollection = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Composition_Scenes\"`*"]
#[repr(transparent)]
pub struct SceneComponentType(pub i32);
impl SceneComponentType {
    pub const MeshRendererComponent: Self = Self(0i32);
}
impl ::core::marker::Copy for SceneComponentType {}
impl ::core::clone::Clone for SceneComponentType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SceneMaterial = *mut ::core::ffi::c_void;
pub type SceneMaterialInput = *mut ::core::ffi::c_void;
pub type SceneMesh = *mut ::core::ffi::c_void;
pub type SceneMeshMaterialAttributeMap = *mut ::core::ffi::c_void;
pub type SceneMeshRendererComponent = *mut ::core::ffi::c_void;
pub type SceneMetallicRoughnessMaterial = *mut ::core::ffi::c_void;
pub type SceneModelTransform = *mut ::core::ffi::c_void;
pub type SceneNode = *mut ::core::ffi::c_void;
pub type SceneNodeCollection = *mut ::core::ffi::c_void;
pub type SceneObject = *mut ::core::ffi::c_void;
pub type ScenePbrMaterial = *mut ::core::ffi::c_void;
pub type SceneRendererComponent = *mut ::core::ffi::c_void;
pub type SceneSurfaceMaterialInput = *mut ::core::ffi::c_void;
pub type SceneVisual = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Composition_Scenes\"`*"]
#[repr(transparent)]
pub struct SceneWrappingMode(pub i32);
impl SceneWrappingMode {
    pub const ClampToEdge: Self = Self(0i32);
    pub const MirroredRepeat: Self = Self(1i32);
    pub const Repeat: Self = Self(2i32);
}
impl ::core::marker::Copy for SceneWrappingMode {}
impl ::core::clone::Clone for SceneWrappingMode {
    fn clone(&self) -> Self {
        *self
    }
}

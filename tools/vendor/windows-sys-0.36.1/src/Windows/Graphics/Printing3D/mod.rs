pub type Print3DManager = *mut ::core::ffi::c_void;
pub type Print3DTask = *mut ::core::ffi::c_void;
pub type Print3DTaskCompletedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Graphics_Printing3D\"`*"]
#[repr(transparent)]
pub struct Print3DTaskCompletion(pub i32);
impl Print3DTaskCompletion {
    pub const Abandoned: Self = Self(0i32);
    pub const Canceled: Self = Self(1i32);
    pub const Failed: Self = Self(2i32);
    pub const Slicing: Self = Self(3i32);
    pub const Submitted: Self = Self(4i32);
}
impl ::core::marker::Copy for Print3DTaskCompletion {}
impl ::core::clone::Clone for Print3DTaskCompletion {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Graphics_Printing3D\"`*"]
#[repr(transparent)]
pub struct Print3DTaskDetail(pub i32);
impl Print3DTaskDetail {
    pub const Unknown: Self = Self(0i32);
    pub const ModelExceedsPrintBed: Self = Self(1i32);
    pub const UploadFailed: Self = Self(2i32);
    pub const InvalidMaterialSelection: Self = Self(3i32);
    pub const InvalidModel: Self = Self(4i32);
    pub const ModelNotManifold: Self = Self(5i32);
    pub const InvalidPrintTicket: Self = Self(6i32);
}
impl ::core::marker::Copy for Print3DTaskDetail {}
impl ::core::clone::Clone for Print3DTaskDetail {
    fn clone(&self) -> Self {
        *self
    }
}
pub type Print3DTaskRequest = *mut ::core::ffi::c_void;
pub type Print3DTaskRequestedEventArgs = *mut ::core::ffi::c_void;
pub type Print3DTaskSourceChangedEventArgs = *mut ::core::ffi::c_void;
pub type Print3DTaskSourceRequestedArgs = *mut ::core::ffi::c_void;
pub type Print3DTaskSourceRequestedHandler = *mut ::core::ffi::c_void;
pub type Printing3D3MFPackage = *mut ::core::ffi::c_void;
pub type Printing3DBaseMaterial = *mut ::core::ffi::c_void;
pub type Printing3DBaseMaterialGroup = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"Graphics_Printing3D\"`*"]
pub struct Printing3DBufferDescription {
    pub Format: Printing3DBufferFormat,
    pub Stride: u32,
}
impl ::core::marker::Copy for Printing3DBufferDescription {}
impl ::core::clone::Clone for Printing3DBufferDescription {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Graphics_Printing3D\"`*"]
#[repr(transparent)]
pub struct Printing3DBufferFormat(pub i32);
impl Printing3DBufferFormat {
    pub const Unknown: Self = Self(0i32);
    pub const R32G32B32A32Float: Self = Self(2i32);
    pub const R32G32B32A32UInt: Self = Self(3i32);
    pub const R32G32B32Float: Self = Self(6i32);
    pub const R32G32B32UInt: Self = Self(7i32);
    pub const Printing3DDouble: Self = Self(500i32);
    pub const Printing3DUInt: Self = Self(501i32);
}
impl ::core::marker::Copy for Printing3DBufferFormat {}
impl ::core::clone::Clone for Printing3DBufferFormat {
    fn clone(&self) -> Self {
        *self
    }
}
pub type Printing3DColorMaterial = *mut ::core::ffi::c_void;
pub type Printing3DColorMaterialGroup = *mut ::core::ffi::c_void;
pub type Printing3DComponent = *mut ::core::ffi::c_void;
pub type Printing3DComponentWithMatrix = *mut ::core::ffi::c_void;
pub type Printing3DCompositeMaterial = *mut ::core::ffi::c_void;
pub type Printing3DCompositeMaterialGroup = *mut ::core::ffi::c_void;
pub type Printing3DFaceReductionOptions = *mut ::core::ffi::c_void;
pub type Printing3DMaterial = *mut ::core::ffi::c_void;
pub type Printing3DMesh = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Graphics_Printing3D\"`*"]
#[repr(transparent)]
pub struct Printing3DMeshVerificationMode(pub i32);
impl Printing3DMeshVerificationMode {
    pub const FindFirstError: Self = Self(0i32);
    pub const FindAllErrors: Self = Self(1i32);
}
impl ::core::marker::Copy for Printing3DMeshVerificationMode {}
impl ::core::clone::Clone for Printing3DMeshVerificationMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type Printing3DMeshVerificationResult = *mut ::core::ffi::c_void;
pub type Printing3DModel = *mut ::core::ffi::c_void;
pub type Printing3DModelTexture = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Graphics_Printing3D\"`*"]
#[repr(transparent)]
pub struct Printing3DModelUnit(pub i32);
impl Printing3DModelUnit {
    pub const Meter: Self = Self(0i32);
    pub const Micron: Self = Self(1i32);
    pub const Millimeter: Self = Self(2i32);
    pub const Centimeter: Self = Self(3i32);
    pub const Inch: Self = Self(4i32);
    pub const Foot: Self = Self(5i32);
}
impl ::core::marker::Copy for Printing3DModelUnit {}
impl ::core::clone::Clone for Printing3DModelUnit {
    fn clone(&self) -> Self {
        *self
    }
}
pub type Printing3DMultiplePropertyMaterial = *mut ::core::ffi::c_void;
pub type Printing3DMultiplePropertyMaterialGroup = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Graphics_Printing3D\"`*"]
#[repr(transparent)]
pub struct Printing3DObjectType(pub i32);
impl Printing3DObjectType {
    pub const Model: Self = Self(0i32);
    pub const Support: Self = Self(1i32);
    pub const Others: Self = Self(2i32);
}
impl ::core::marker::Copy for Printing3DObjectType {}
impl ::core::clone::Clone for Printing3DObjectType {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Graphics_Printing3D\"`*"]
#[repr(transparent)]
pub struct Printing3DPackageCompression(pub i32);
impl Printing3DPackageCompression {
    pub const Low: Self = Self(0i32);
    pub const Medium: Self = Self(1i32);
    pub const High: Self = Self(2i32);
}
impl ::core::marker::Copy for Printing3DPackageCompression {}
impl ::core::clone::Clone for Printing3DPackageCompression {
    fn clone(&self) -> Self {
        *self
    }
}
pub type Printing3DTexture2CoordMaterial = *mut ::core::ffi::c_void;
pub type Printing3DTexture2CoordMaterialGroup = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Graphics_Printing3D\"`*"]
#[repr(transparent)]
pub struct Printing3DTextureEdgeBehavior(pub i32);
impl Printing3DTextureEdgeBehavior {
    pub const None: Self = Self(0i32);
    pub const Wrap: Self = Self(1i32);
    pub const Mirror: Self = Self(2i32);
    pub const Clamp: Self = Self(3i32);
}
impl ::core::marker::Copy for Printing3DTextureEdgeBehavior {}
impl ::core::clone::Clone for Printing3DTextureEdgeBehavior {
    fn clone(&self) -> Self {
        *self
    }
}
pub type Printing3DTextureResource = *mut ::core::ffi::c_void;

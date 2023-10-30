#[doc = "*Required features: `\"AI_MachineLearning_Preview\"`, `\"deprecated\"`*"]
#[cfg(feature = "deprecated")]
#[repr(transparent)]
pub struct FeatureElementKindPreview(pub i32);
#[cfg(feature = "deprecated")]
impl FeatureElementKindPreview {
    pub const Undefined: Self = Self(0i32);
    pub const Float: Self = Self(1i32);
    pub const UInt8: Self = Self(2i32);
    pub const Int8: Self = Self(3i32);
    pub const UInt16: Self = Self(4i32);
    pub const Int16: Self = Self(5i32);
    pub const Int32: Self = Self(6i32);
    pub const Int64: Self = Self(7i32);
    pub const String: Self = Self(8i32);
    pub const Boolean: Self = Self(9i32);
    pub const Float16: Self = Self(10i32);
    pub const Double: Self = Self(11i32);
    pub const UInt32: Self = Self(12i32);
    pub const UInt64: Self = Self(13i32);
    pub const Complex64: Self = Self(14i32);
    pub const Complex128: Self = Self(15i32);
}
#[cfg(feature = "deprecated")]
impl ::core::marker::Copy for FeatureElementKindPreview {}
#[cfg(feature = "deprecated")]
impl ::core::clone::Clone for FeatureElementKindPreview {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ILearningModelVariableDescriptorPreview = *mut ::core::ffi::c_void;
pub type ImageVariableDescriptorPreview = *mut ::core::ffi::c_void;
pub type InferencingOptionsPreview = *mut ::core::ffi::c_void;
pub type LearningModelBindingPreview = *mut ::core::ffi::c_void;
pub type LearningModelDescriptionPreview = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"AI_MachineLearning_Preview\"`, `\"deprecated\"`*"]
#[cfg(feature = "deprecated")]
#[repr(transparent)]
pub struct LearningModelDeviceKindPreview(pub i32);
#[cfg(feature = "deprecated")]
impl LearningModelDeviceKindPreview {
    pub const LearningDeviceAny: Self = Self(0i32);
    pub const LearningDeviceCpu: Self = Self(1i32);
    pub const LearningDeviceGpu: Self = Self(2i32);
    pub const LearningDeviceNpu: Self = Self(3i32);
    pub const LearningDeviceDsp: Self = Self(4i32);
    pub const LearningDeviceFpga: Self = Self(5i32);
}
#[cfg(feature = "deprecated")]
impl ::core::marker::Copy for LearningModelDeviceKindPreview {}
#[cfg(feature = "deprecated")]
impl ::core::clone::Clone for LearningModelDeviceKindPreview {
    fn clone(&self) -> Self {
        *self
    }
}
pub type LearningModelEvaluationResultPreview = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"AI_MachineLearning_Preview\"`, `\"deprecated\"`*"]
#[cfg(feature = "deprecated")]
#[repr(transparent)]
pub struct LearningModelFeatureKindPreview(pub i32);
#[cfg(feature = "deprecated")]
impl LearningModelFeatureKindPreview {
    pub const Undefined: Self = Self(0i32);
    pub const Tensor: Self = Self(1i32);
    pub const Sequence: Self = Self(2i32);
    pub const Map: Self = Self(3i32);
    pub const Image: Self = Self(4i32);
}
#[cfg(feature = "deprecated")]
impl ::core::marker::Copy for LearningModelFeatureKindPreview {}
#[cfg(feature = "deprecated")]
impl ::core::clone::Clone for LearningModelFeatureKindPreview {
    fn clone(&self) -> Self {
        *self
    }
}
pub type LearningModelPreview = *mut ::core::ffi::c_void;
pub type LearningModelVariableDescriptorPreview = *mut ::core::ffi::c_void;
pub type MapVariableDescriptorPreview = *mut ::core::ffi::c_void;
pub type SequenceVariableDescriptorPreview = *mut ::core::ffi::c_void;
pub type TensorVariableDescriptorPreview = *mut ::core::ffi::c_void;

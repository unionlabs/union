#[cfg(feature = "AI_MachineLearning_Preview")]
pub mod Preview;
pub type ILearningModelFeatureDescriptor = *mut ::core::ffi::c_void;
pub type ILearningModelFeatureValue = *mut ::core::ffi::c_void;
pub type ILearningModelOperatorProvider = *mut ::core::ffi::c_void;
pub type ITensor = *mut ::core::ffi::c_void;
pub type ImageFeatureDescriptor = *mut ::core::ffi::c_void;
pub type ImageFeatureValue = *mut ::core::ffi::c_void;
pub type LearningModel = *mut ::core::ffi::c_void;
pub type LearningModelBinding = *mut ::core::ffi::c_void;
pub type LearningModelDevice = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"AI_MachineLearning\"`*"]
#[repr(transparent)]
pub struct LearningModelDeviceKind(pub i32);
impl LearningModelDeviceKind {
    pub const Default: Self = Self(0i32);
    pub const Cpu: Self = Self(1i32);
    pub const DirectX: Self = Self(2i32);
    pub const DirectXHighPerformance: Self = Self(3i32);
    pub const DirectXMinPower: Self = Self(4i32);
}
impl ::core::marker::Copy for LearningModelDeviceKind {}
impl ::core::clone::Clone for LearningModelDeviceKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type LearningModelEvaluationResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"AI_MachineLearning\"`*"]
#[repr(transparent)]
pub struct LearningModelFeatureKind(pub i32);
impl LearningModelFeatureKind {
    pub const Tensor: Self = Self(0i32);
    pub const Sequence: Self = Self(1i32);
    pub const Map: Self = Self(2i32);
    pub const Image: Self = Self(3i32);
}
impl ::core::marker::Copy for LearningModelFeatureKind {}
impl ::core::clone::Clone for LearningModelFeatureKind {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"AI_MachineLearning\"`*"]
#[repr(transparent)]
pub struct LearningModelPixelRange(pub i32);
impl LearningModelPixelRange {
    pub const ZeroTo255: Self = Self(0i32);
    pub const ZeroToOne: Self = Self(1i32);
    pub const MinusOneToOne: Self = Self(2i32);
}
impl ::core::marker::Copy for LearningModelPixelRange {}
impl ::core::clone::Clone for LearningModelPixelRange {
    fn clone(&self) -> Self {
        *self
    }
}
pub type LearningModelSession = *mut ::core::ffi::c_void;
pub type LearningModelSessionOptions = *mut ::core::ffi::c_void;
pub type MapFeatureDescriptor = *mut ::core::ffi::c_void;
pub type SequenceFeatureDescriptor = *mut ::core::ffi::c_void;
pub type TensorBoolean = *mut ::core::ffi::c_void;
pub type TensorDouble = *mut ::core::ffi::c_void;
pub type TensorFeatureDescriptor = *mut ::core::ffi::c_void;
pub type TensorFloat = *mut ::core::ffi::c_void;
pub type TensorFloat16Bit = *mut ::core::ffi::c_void;
pub type TensorInt16Bit = *mut ::core::ffi::c_void;
pub type TensorInt32Bit = *mut ::core::ffi::c_void;
pub type TensorInt64Bit = *mut ::core::ffi::c_void;
pub type TensorInt8Bit = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"AI_MachineLearning\"`*"]
#[repr(transparent)]
pub struct TensorKind(pub i32);
impl TensorKind {
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
impl ::core::marker::Copy for TensorKind {}
impl ::core::clone::Clone for TensorKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type TensorString = *mut ::core::ffi::c_void;
pub type TensorUInt16Bit = *mut ::core::ffi::c_void;
pub type TensorUInt32Bit = *mut ::core::ffi::c_void;
pub type TensorUInt64Bit = *mut ::core::ffi::c_void;
pub type TensorUInt8Bit = *mut ::core::ffi::c_void;

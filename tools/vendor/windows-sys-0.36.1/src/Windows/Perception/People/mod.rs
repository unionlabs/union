pub type EyesPose = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Perception_People\"`*"]
#[repr(transparent)]
pub struct HandJointKind(pub i32);
impl HandJointKind {
    pub const Palm: Self = Self(0i32);
    pub const Wrist: Self = Self(1i32);
    pub const ThumbMetacarpal: Self = Self(2i32);
    pub const ThumbProximal: Self = Self(3i32);
    pub const ThumbDistal: Self = Self(4i32);
    pub const ThumbTip: Self = Self(5i32);
    pub const IndexMetacarpal: Self = Self(6i32);
    pub const IndexProximal: Self = Self(7i32);
    pub const IndexIntermediate: Self = Self(8i32);
    pub const IndexDistal: Self = Self(9i32);
    pub const IndexTip: Self = Self(10i32);
    pub const MiddleMetacarpal: Self = Self(11i32);
    pub const MiddleProximal: Self = Self(12i32);
    pub const MiddleIntermediate: Self = Self(13i32);
    pub const MiddleDistal: Self = Self(14i32);
    pub const MiddleTip: Self = Self(15i32);
    pub const RingMetacarpal: Self = Self(16i32);
    pub const RingProximal: Self = Self(17i32);
    pub const RingIntermediate: Self = Self(18i32);
    pub const RingDistal: Self = Self(19i32);
    pub const RingTip: Self = Self(20i32);
    pub const LittleMetacarpal: Self = Self(21i32);
    pub const LittleProximal: Self = Self(22i32);
    pub const LittleIntermediate: Self = Self(23i32);
    pub const LittleDistal: Self = Self(24i32);
    pub const LittleTip: Self = Self(25i32);
}
impl ::core::marker::Copy for HandJointKind {}
impl ::core::clone::Clone for HandJointKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type HandMeshObserver = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"Perception_People\"`, `\"Foundation_Numerics\"`*"]
#[cfg(feature = "Foundation_Numerics")]
pub struct HandMeshVertex {
    pub Position: super::super::Foundation::Numerics::Vector3,
    pub Normal: super::super::Foundation::Numerics::Vector3,
}
#[cfg(feature = "Foundation_Numerics")]
impl ::core::marker::Copy for HandMeshVertex {}
#[cfg(feature = "Foundation_Numerics")]
impl ::core::clone::Clone for HandMeshVertex {
    fn clone(&self) -> Self {
        *self
    }
}
pub type HandMeshVertexState = *mut ::core::ffi::c_void;
pub type HandPose = *mut ::core::ffi::c_void;
pub type HeadPose = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"Perception_People\"`, `\"Foundation_Numerics\"`*"]
#[cfg(feature = "Foundation_Numerics")]
pub struct JointPose {
    pub Orientation: super::super::Foundation::Numerics::Quaternion,
    pub Position: super::super::Foundation::Numerics::Vector3,
    pub Radius: f32,
    pub Accuracy: JointPoseAccuracy,
}
#[cfg(feature = "Foundation_Numerics")]
impl ::core::marker::Copy for JointPose {}
#[cfg(feature = "Foundation_Numerics")]
impl ::core::clone::Clone for JointPose {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Perception_People\"`*"]
#[repr(transparent)]
pub struct JointPoseAccuracy(pub i32);
impl JointPoseAccuracy {
    pub const High: Self = Self(0i32);
    pub const Approximate: Self = Self(1i32);
}
impl ::core::marker::Copy for JointPoseAccuracy {}
impl ::core::clone::Clone for JointPoseAccuracy {
    fn clone(&self) -> Self {
        *self
    }
}

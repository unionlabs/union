pub type SpatialGestureRecognizer = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Input_Spatial\"`*"]
#[repr(transparent)]
pub struct SpatialGestureSettings(pub u32);
impl SpatialGestureSettings {
    pub const None: Self = Self(0u32);
    pub const Tap: Self = Self(1u32);
    pub const DoubleTap: Self = Self(2u32);
    pub const Hold: Self = Self(4u32);
    pub const ManipulationTranslate: Self = Self(8u32);
    pub const NavigationX: Self = Self(16u32);
    pub const NavigationY: Self = Self(32u32);
    pub const NavigationZ: Self = Self(64u32);
    pub const NavigationRailsX: Self = Self(128u32);
    pub const NavigationRailsY: Self = Self(256u32);
    pub const NavigationRailsZ: Self = Self(512u32);
}
impl ::core::marker::Copy for SpatialGestureSettings {}
impl ::core::clone::Clone for SpatialGestureSettings {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SpatialHoldCanceledEventArgs = *mut ::core::ffi::c_void;
pub type SpatialHoldCompletedEventArgs = *mut ::core::ffi::c_void;
pub type SpatialHoldStartedEventArgs = *mut ::core::ffi::c_void;
pub type SpatialInteraction = *mut ::core::ffi::c_void;
pub type SpatialInteractionController = *mut ::core::ffi::c_void;
pub type SpatialInteractionControllerProperties = *mut ::core::ffi::c_void;
pub type SpatialInteractionDetectedEventArgs = *mut ::core::ffi::c_void;
pub type SpatialInteractionManager = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Input_Spatial\"`*"]
#[repr(transparent)]
pub struct SpatialInteractionPressKind(pub i32);
impl SpatialInteractionPressKind {
    pub const None: Self = Self(0i32);
    pub const Select: Self = Self(1i32);
    pub const Menu: Self = Self(2i32);
    pub const Grasp: Self = Self(3i32);
    pub const Touchpad: Self = Self(4i32);
    pub const Thumbstick: Self = Self(5i32);
}
impl ::core::marker::Copy for SpatialInteractionPressKind {}
impl ::core::clone::Clone for SpatialInteractionPressKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SpatialInteractionSource = *mut ::core::ffi::c_void;
pub type SpatialInteractionSourceEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Input_Spatial\"`*"]
#[repr(transparent)]
pub struct SpatialInteractionSourceHandedness(pub i32);
impl SpatialInteractionSourceHandedness {
    pub const Unspecified: Self = Self(0i32);
    pub const Left: Self = Self(1i32);
    pub const Right: Self = Self(2i32);
}
impl ::core::marker::Copy for SpatialInteractionSourceHandedness {}
impl ::core::clone::Clone for SpatialInteractionSourceHandedness {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Input_Spatial\"`*"]
#[repr(transparent)]
pub struct SpatialInteractionSourceKind(pub i32);
impl SpatialInteractionSourceKind {
    pub const Other: Self = Self(0i32);
    pub const Hand: Self = Self(1i32);
    pub const Voice: Self = Self(2i32);
    pub const Controller: Self = Self(3i32);
}
impl ::core::marker::Copy for SpatialInteractionSourceKind {}
impl ::core::clone::Clone for SpatialInteractionSourceKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SpatialInteractionSourceLocation = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Input_Spatial\"`*"]
#[repr(transparent)]
pub struct SpatialInteractionSourcePositionAccuracy(pub i32);
impl SpatialInteractionSourcePositionAccuracy {
    pub const High: Self = Self(0i32);
    pub const Approximate: Self = Self(1i32);
}
impl ::core::marker::Copy for SpatialInteractionSourcePositionAccuracy {}
impl ::core::clone::Clone for SpatialInteractionSourcePositionAccuracy {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SpatialInteractionSourceProperties = *mut ::core::ffi::c_void;
pub type SpatialInteractionSourceState = *mut ::core::ffi::c_void;
pub type SpatialManipulationCanceledEventArgs = *mut ::core::ffi::c_void;
pub type SpatialManipulationCompletedEventArgs = *mut ::core::ffi::c_void;
pub type SpatialManipulationDelta = *mut ::core::ffi::c_void;
pub type SpatialManipulationStartedEventArgs = *mut ::core::ffi::c_void;
pub type SpatialManipulationUpdatedEventArgs = *mut ::core::ffi::c_void;
pub type SpatialNavigationCanceledEventArgs = *mut ::core::ffi::c_void;
pub type SpatialNavigationCompletedEventArgs = *mut ::core::ffi::c_void;
pub type SpatialNavigationStartedEventArgs = *mut ::core::ffi::c_void;
pub type SpatialNavigationUpdatedEventArgs = *mut ::core::ffi::c_void;
pub type SpatialPointerInteractionSourcePose = *mut ::core::ffi::c_void;
pub type SpatialPointerPose = *mut ::core::ffi::c_void;
pub type SpatialRecognitionEndedEventArgs = *mut ::core::ffi::c_void;
pub type SpatialRecognitionStartedEventArgs = *mut ::core::ffi::c_void;
pub type SpatialTappedEventArgs = *mut ::core::ffi::c_void;

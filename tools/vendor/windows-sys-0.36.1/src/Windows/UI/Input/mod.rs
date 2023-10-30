#[cfg(feature = "UI_Input_Core")]
pub mod Core;
#[cfg(feature = "UI_Input_Inking")]
pub mod Inking;
#[cfg(feature = "UI_Input_Preview")]
pub mod Preview;
#[cfg(feature = "UI_Input_Spatial")]
pub mod Spatial;
pub type AttachableInputObject = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"UI_Input\"`*"]
pub struct CrossSlideThresholds {
    pub SelectionStart: f32,
    pub SpeedBumpStart: f32,
    pub SpeedBumpEnd: f32,
    pub RearrangeStart: f32,
}
impl ::core::marker::Copy for CrossSlideThresholds {}
impl ::core::clone::Clone for CrossSlideThresholds {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CrossSlidingEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Input\"`*"]
#[repr(transparent)]
pub struct CrossSlidingState(pub i32);
impl CrossSlidingState {
    pub const Started: Self = Self(0i32);
    pub const Dragging: Self = Self(1i32);
    pub const Selecting: Self = Self(2i32);
    pub const SelectSpeedBumping: Self = Self(3i32);
    pub const SpeedBumping: Self = Self(4i32);
    pub const Rearranging: Self = Self(5i32);
    pub const Completed: Self = Self(6i32);
}
impl ::core::marker::Copy for CrossSlidingState {}
impl ::core::clone::Clone for CrossSlidingState {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DraggingEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Input\"`*"]
#[repr(transparent)]
pub struct DraggingState(pub i32);
impl DraggingState {
    pub const Started: Self = Self(0i32);
    pub const Continuing: Self = Self(1i32);
    pub const Completed: Self = Self(2i32);
}
impl ::core::marker::Copy for DraggingState {}
impl ::core::clone::Clone for DraggingState {
    fn clone(&self) -> Self {
        *self
    }
}
pub type EdgeGesture = *mut ::core::ffi::c_void;
pub type EdgeGestureEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Input\"`*"]
#[repr(transparent)]
pub struct EdgeGestureKind(pub i32);
impl EdgeGestureKind {
    pub const Touch: Self = Self(0i32);
    pub const Keyboard: Self = Self(1i32);
    pub const Mouse: Self = Self(2i32);
}
impl ::core::marker::Copy for EdgeGestureKind {}
impl ::core::clone::Clone for EdgeGestureKind {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Input\"`*"]
#[repr(transparent)]
pub struct GazeInputAccessStatus(pub i32);
impl GazeInputAccessStatus {
    pub const Unspecified: Self = Self(0i32);
    pub const Allowed: Self = Self(1i32);
    pub const DeniedByUser: Self = Self(2i32);
    pub const DeniedBySystem: Self = Self(3i32);
}
impl ::core::marker::Copy for GazeInputAccessStatus {}
impl ::core::clone::Clone for GazeInputAccessStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type GestureRecognizer = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Input\"`*"]
#[repr(transparent)]
pub struct GestureSettings(pub u32);
impl GestureSettings {
    pub const None: Self = Self(0u32);
    pub const Tap: Self = Self(1u32);
    pub const DoubleTap: Self = Self(2u32);
    pub const Hold: Self = Self(4u32);
    pub const HoldWithMouse: Self = Self(8u32);
    pub const RightTap: Self = Self(16u32);
    pub const Drag: Self = Self(32u32);
    pub const ManipulationTranslateX: Self = Self(64u32);
    pub const ManipulationTranslateY: Self = Self(128u32);
    pub const ManipulationTranslateRailsX: Self = Self(256u32);
    pub const ManipulationTranslateRailsY: Self = Self(512u32);
    pub const ManipulationRotate: Self = Self(1024u32);
    pub const ManipulationScale: Self = Self(2048u32);
    pub const ManipulationTranslateInertia: Self = Self(4096u32);
    pub const ManipulationRotateInertia: Self = Self(8192u32);
    pub const ManipulationScaleInertia: Self = Self(16384u32);
    pub const CrossSlide: Self = Self(32768u32);
    pub const ManipulationMultipleFingerPanning: Self = Self(65536u32);
}
impl ::core::marker::Copy for GestureSettings {}
impl ::core::clone::Clone for GestureSettings {
    fn clone(&self) -> Self {
        *self
    }
}
pub type HoldingEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Input\"`*"]
#[repr(transparent)]
pub struct HoldingState(pub i32);
impl HoldingState {
    pub const Started: Self = Self(0i32);
    pub const Completed: Self = Self(1i32);
    pub const Canceled: Self = Self(2i32);
}
impl ::core::marker::Copy for HoldingState {}
impl ::core::clone::Clone for HoldingState {
    fn clone(&self) -> Self {
        *self
    }
}
pub type IPointerPointTransform = *mut ::core::ffi::c_void;
pub type InputActivationListener = *mut ::core::ffi::c_void;
pub type InputActivationListenerActivationChangedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Input\"`*"]
#[repr(transparent)]
pub struct InputActivationState(pub i32);
impl InputActivationState {
    pub const None: Self = Self(0i32);
    pub const Deactivated: Self = Self(1i32);
    pub const ActivatedNotForeground: Self = Self(2i32);
    pub const ActivatedInForeground: Self = Self(3i32);
}
impl ::core::marker::Copy for InputActivationState {}
impl ::core::clone::Clone for InputActivationState {
    fn clone(&self) -> Self {
        *self
    }
}
pub type KeyboardDeliveryInterceptor = *mut ::core::ffi::c_void;
pub type ManipulationCompletedEventArgs = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"UI_Input\"`, `\"Foundation\"`*"]
#[cfg(feature = "Foundation")]
pub struct ManipulationDelta {
    pub Translation: super::super::Foundation::Point,
    pub Scale: f32,
    pub Rotation: f32,
    pub Expansion: f32,
}
#[cfg(feature = "Foundation")]
impl ::core::marker::Copy for ManipulationDelta {}
#[cfg(feature = "Foundation")]
impl ::core::clone::Clone for ManipulationDelta {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ManipulationInertiaStartingEventArgs = *mut ::core::ffi::c_void;
pub type ManipulationStartedEventArgs = *mut ::core::ffi::c_void;
pub type ManipulationUpdatedEventArgs = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"UI_Input\"`, `\"Foundation\"`*"]
#[cfg(feature = "Foundation")]
pub struct ManipulationVelocities {
    pub Linear: super::super::Foundation::Point,
    pub Angular: f32,
    pub Expansion: f32,
}
#[cfg(feature = "Foundation")]
impl ::core::marker::Copy for ManipulationVelocities {}
#[cfg(feature = "Foundation")]
impl ::core::clone::Clone for ManipulationVelocities {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MouseWheelParameters = *mut ::core::ffi::c_void;
pub type PointerPoint = *mut ::core::ffi::c_void;
pub type PointerPointProperties = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Input\"`*"]
#[repr(transparent)]
pub struct PointerUpdateKind(pub i32);
impl PointerUpdateKind {
    pub const Other: Self = Self(0i32);
    pub const LeftButtonPressed: Self = Self(1i32);
    pub const LeftButtonReleased: Self = Self(2i32);
    pub const RightButtonPressed: Self = Self(3i32);
    pub const RightButtonReleased: Self = Self(4i32);
    pub const MiddleButtonPressed: Self = Self(5i32);
    pub const MiddleButtonReleased: Self = Self(6i32);
    pub const XButton1Pressed: Self = Self(7i32);
    pub const XButton1Released: Self = Self(8i32);
    pub const XButton2Pressed: Self = Self(9i32);
    pub const XButton2Released: Self = Self(10i32);
}
impl ::core::marker::Copy for PointerUpdateKind {}
impl ::core::clone::Clone for PointerUpdateKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PointerVisualizationSettings = *mut ::core::ffi::c_void;
pub type RadialController = *mut ::core::ffi::c_void;
pub type RadialControllerButtonClickedEventArgs = *mut ::core::ffi::c_void;
pub type RadialControllerButtonHoldingEventArgs = *mut ::core::ffi::c_void;
pub type RadialControllerButtonPressedEventArgs = *mut ::core::ffi::c_void;
pub type RadialControllerButtonReleasedEventArgs = *mut ::core::ffi::c_void;
pub type RadialControllerConfiguration = *mut ::core::ffi::c_void;
pub type RadialControllerControlAcquiredEventArgs = *mut ::core::ffi::c_void;
pub type RadialControllerMenu = *mut ::core::ffi::c_void;
pub type RadialControllerMenuItem = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Input\"`*"]
#[repr(transparent)]
pub struct RadialControllerMenuKnownIcon(pub i32);
impl RadialControllerMenuKnownIcon {
    pub const Scroll: Self = Self(0i32);
    pub const Zoom: Self = Self(1i32);
    pub const UndoRedo: Self = Self(2i32);
    pub const Volume: Self = Self(3i32);
    pub const NextPreviousTrack: Self = Self(4i32);
    pub const Ruler: Self = Self(5i32);
    pub const InkColor: Self = Self(6i32);
    pub const InkThickness: Self = Self(7i32);
    pub const PenType: Self = Self(8i32);
}
impl ::core::marker::Copy for RadialControllerMenuKnownIcon {}
impl ::core::clone::Clone for RadialControllerMenuKnownIcon {
    fn clone(&self) -> Self {
        *self
    }
}
pub type RadialControllerRotationChangedEventArgs = *mut ::core::ffi::c_void;
pub type RadialControllerScreenContact = *mut ::core::ffi::c_void;
pub type RadialControllerScreenContactContinuedEventArgs = *mut ::core::ffi::c_void;
pub type RadialControllerScreenContactEndedEventArgs = *mut ::core::ffi::c_void;
pub type RadialControllerScreenContactStartedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Input\"`*"]
#[repr(transparent)]
pub struct RadialControllerSystemMenuItemKind(pub i32);
impl RadialControllerSystemMenuItemKind {
    pub const Scroll: Self = Self(0i32);
    pub const Zoom: Self = Self(1i32);
    pub const UndoRedo: Self = Self(2i32);
    pub const Volume: Self = Self(3i32);
    pub const NextPreviousTrack: Self = Self(4i32);
}
impl ::core::marker::Copy for RadialControllerSystemMenuItemKind {}
impl ::core::clone::Clone for RadialControllerSystemMenuItemKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type RightTappedEventArgs = *mut ::core::ffi::c_void;
pub type SystemButtonEventController = *mut ::core::ffi::c_void;
pub type SystemFunctionButtonEventArgs = *mut ::core::ffi::c_void;
pub type SystemFunctionLockChangedEventArgs = *mut ::core::ffi::c_void;
pub type SystemFunctionLockIndicatorChangedEventArgs = *mut ::core::ffi::c_void;
pub type TappedEventArgs = *mut ::core::ffi::c_void;

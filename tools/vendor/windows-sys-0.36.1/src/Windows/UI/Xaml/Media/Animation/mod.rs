pub type AddDeleteThemeTransition = *mut ::core::ffi::c_void;
pub type BackEase = *mut ::core::ffi::c_void;
pub type BasicConnectedAnimationConfiguration = *mut ::core::ffi::c_void;
pub type BeginStoryboard = *mut ::core::ffi::c_void;
pub type BounceEase = *mut ::core::ffi::c_void;
pub type CircleEase = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Media_Animation\"`*"]
#[repr(transparent)]
pub struct ClockState(pub i32);
impl ClockState {
    pub const Active: Self = Self(0i32);
    pub const Filling: Self = Self(1i32);
    pub const Stopped: Self = Self(2i32);
}
impl ::core::marker::Copy for ClockState {}
impl ::core::clone::Clone for ClockState {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ColorAnimation = *mut ::core::ffi::c_void;
pub type ColorAnimationUsingKeyFrames = *mut ::core::ffi::c_void;
pub type ColorKeyFrame = *mut ::core::ffi::c_void;
pub type ColorKeyFrameCollection = *mut ::core::ffi::c_void;
pub type CommonNavigationTransitionInfo = *mut ::core::ffi::c_void;
pub type ConnectedAnimation = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Media_Animation\"`*"]
#[repr(transparent)]
pub struct ConnectedAnimationComponent(pub i32);
impl ConnectedAnimationComponent {
    pub const OffsetX: Self = Self(0i32);
    pub const OffsetY: Self = Self(1i32);
    pub const CrossFade: Self = Self(2i32);
    pub const Scale: Self = Self(3i32);
}
impl ::core::marker::Copy for ConnectedAnimationComponent {}
impl ::core::clone::Clone for ConnectedAnimationComponent {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ConnectedAnimationConfiguration = *mut ::core::ffi::c_void;
pub type ConnectedAnimationService = *mut ::core::ffi::c_void;
pub type ContentThemeTransition = *mut ::core::ffi::c_void;
pub type ContinuumNavigationTransitionInfo = *mut ::core::ffi::c_void;
pub type CubicEase = *mut ::core::ffi::c_void;
pub type DirectConnectedAnimationConfiguration = *mut ::core::ffi::c_void;
pub type DiscreteColorKeyFrame = *mut ::core::ffi::c_void;
pub type DiscreteDoubleKeyFrame = *mut ::core::ffi::c_void;
pub type DiscreteObjectKeyFrame = *mut ::core::ffi::c_void;
pub type DiscretePointKeyFrame = *mut ::core::ffi::c_void;
pub type DoubleAnimation = *mut ::core::ffi::c_void;
pub type DoubleAnimationUsingKeyFrames = *mut ::core::ffi::c_void;
pub type DoubleKeyFrame = *mut ::core::ffi::c_void;
pub type DoubleKeyFrameCollection = *mut ::core::ffi::c_void;
pub type DragItemThemeAnimation = *mut ::core::ffi::c_void;
pub type DragOverThemeAnimation = *mut ::core::ffi::c_void;
pub type DrillInNavigationTransitionInfo = *mut ::core::ffi::c_void;
pub type DrillInThemeAnimation = *mut ::core::ffi::c_void;
pub type DrillOutThemeAnimation = *mut ::core::ffi::c_void;
pub type DropTargetItemThemeAnimation = *mut ::core::ffi::c_void;
pub type EasingColorKeyFrame = *mut ::core::ffi::c_void;
pub type EasingDoubleKeyFrame = *mut ::core::ffi::c_void;
pub type EasingFunctionBase = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Media_Animation\"`*"]
#[repr(transparent)]
pub struct EasingMode(pub i32);
impl EasingMode {
    pub const EaseOut: Self = Self(0i32);
    pub const EaseIn: Self = Self(1i32);
    pub const EaseInOut: Self = Self(2i32);
}
impl ::core::marker::Copy for EasingMode {}
impl ::core::clone::Clone for EasingMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type EasingPointKeyFrame = *mut ::core::ffi::c_void;
pub type EdgeUIThemeTransition = *mut ::core::ffi::c_void;
pub type ElasticEase = *mut ::core::ffi::c_void;
pub type EntranceNavigationTransitionInfo = *mut ::core::ffi::c_void;
pub type EntranceThemeTransition = *mut ::core::ffi::c_void;
pub type ExponentialEase = *mut ::core::ffi::c_void;
pub type FadeInThemeAnimation = *mut ::core::ffi::c_void;
pub type FadeOutThemeAnimation = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Media_Animation\"`*"]
#[repr(transparent)]
pub struct FillBehavior(pub i32);
impl FillBehavior {
    pub const HoldEnd: Self = Self(0i32);
    pub const Stop: Self = Self(1i32);
}
impl ::core::marker::Copy for FillBehavior {}
impl ::core::clone::Clone for FillBehavior {
    fn clone(&self) -> Self {
        *self
    }
}
pub type GravityConnectedAnimationConfiguration = *mut ::core::ffi::c_void;
pub type KeySpline = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"UI_Xaml_Media_Animation\"`, `\"Foundation\"`*"]
#[cfg(feature = "Foundation")]
pub struct KeyTime {
    pub TimeSpan: super::super::super::super::Foundation::TimeSpan,
}
#[cfg(feature = "Foundation")]
impl ::core::marker::Copy for KeyTime {}
#[cfg(feature = "Foundation")]
impl ::core::clone::Clone for KeyTime {
    fn clone(&self) -> Self {
        *self
    }
}
pub type KeyTimeHelper = *mut ::core::ffi::c_void;
pub type LinearColorKeyFrame = *mut ::core::ffi::c_void;
pub type LinearDoubleKeyFrame = *mut ::core::ffi::c_void;
pub type LinearPointKeyFrame = *mut ::core::ffi::c_void;
pub type NavigationThemeTransition = *mut ::core::ffi::c_void;
pub type NavigationTransitionInfo = *mut ::core::ffi::c_void;
pub type ObjectAnimationUsingKeyFrames = *mut ::core::ffi::c_void;
pub type ObjectKeyFrame = *mut ::core::ffi::c_void;
pub type ObjectKeyFrameCollection = *mut ::core::ffi::c_void;
pub type PaneThemeTransition = *mut ::core::ffi::c_void;
pub type PointAnimation = *mut ::core::ffi::c_void;
pub type PointAnimationUsingKeyFrames = *mut ::core::ffi::c_void;
pub type PointKeyFrame = *mut ::core::ffi::c_void;
pub type PointKeyFrameCollection = *mut ::core::ffi::c_void;
pub type PointerDownThemeAnimation = *mut ::core::ffi::c_void;
pub type PointerUpThemeAnimation = *mut ::core::ffi::c_void;
pub type PopInThemeAnimation = *mut ::core::ffi::c_void;
pub type PopOutThemeAnimation = *mut ::core::ffi::c_void;
pub type PopupThemeTransition = *mut ::core::ffi::c_void;
pub type PowerEase = *mut ::core::ffi::c_void;
pub type QuadraticEase = *mut ::core::ffi::c_void;
pub type QuarticEase = *mut ::core::ffi::c_void;
pub type QuinticEase = *mut ::core::ffi::c_void;
pub type ReorderThemeTransition = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"UI_Xaml_Media_Animation\"`, `\"Foundation\"`*"]
#[cfg(feature = "Foundation")]
pub struct RepeatBehavior {
    pub Count: f64,
    pub Duration: super::super::super::super::Foundation::TimeSpan,
    pub Type: RepeatBehaviorType,
}
#[cfg(feature = "Foundation")]
impl ::core::marker::Copy for RepeatBehavior {}
#[cfg(feature = "Foundation")]
impl ::core::clone::Clone for RepeatBehavior {
    fn clone(&self) -> Self {
        *self
    }
}
pub type RepeatBehaviorHelper = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Media_Animation\"`*"]
#[repr(transparent)]
pub struct RepeatBehaviorType(pub i32);
impl RepeatBehaviorType {
    pub const Count: Self = Self(0i32);
    pub const Duration: Self = Self(1i32);
    pub const Forever: Self = Self(2i32);
}
impl ::core::marker::Copy for RepeatBehaviorType {}
impl ::core::clone::Clone for RepeatBehaviorType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type RepositionThemeAnimation = *mut ::core::ffi::c_void;
pub type RepositionThemeTransition = *mut ::core::ffi::c_void;
pub type SineEase = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Media_Animation\"`*"]
#[repr(transparent)]
pub struct SlideNavigationTransitionEffect(pub i32);
impl SlideNavigationTransitionEffect {
    pub const FromBottom: Self = Self(0i32);
    pub const FromLeft: Self = Self(1i32);
    pub const FromRight: Self = Self(2i32);
}
impl ::core::marker::Copy for SlideNavigationTransitionEffect {}
impl ::core::clone::Clone for SlideNavigationTransitionEffect {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SlideNavigationTransitionInfo = *mut ::core::ffi::c_void;
pub type SplineColorKeyFrame = *mut ::core::ffi::c_void;
pub type SplineDoubleKeyFrame = *mut ::core::ffi::c_void;
pub type SplinePointKeyFrame = *mut ::core::ffi::c_void;
pub type SplitCloseThemeAnimation = *mut ::core::ffi::c_void;
pub type SplitOpenThemeAnimation = *mut ::core::ffi::c_void;
pub type Storyboard = *mut ::core::ffi::c_void;
pub type SuppressNavigationTransitionInfo = *mut ::core::ffi::c_void;
pub type SwipeBackThemeAnimation = *mut ::core::ffi::c_void;
pub type SwipeHintThemeAnimation = *mut ::core::ffi::c_void;
pub type Timeline = *mut ::core::ffi::c_void;
pub type TimelineCollection = *mut ::core::ffi::c_void;
pub type Transition = *mut ::core::ffi::c_void;
pub type TransitionCollection = *mut ::core::ffi::c_void;

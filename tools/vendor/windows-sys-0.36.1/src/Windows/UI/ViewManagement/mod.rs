#[cfg(feature = "UI_ViewManagement_Core")]
pub mod Core;
pub type AccessibilitySettings = *mut ::core::ffi::c_void;
pub type ActivationViewSwitcher = *mut ::core::ffi::c_void;
pub type ApplicationView = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_ViewManagement\"`*"]
#[repr(transparent)]
pub struct ApplicationViewBoundsMode(pub i32);
impl ApplicationViewBoundsMode {
    pub const UseVisible: Self = Self(0i32);
    pub const UseCoreWindow: Self = Self(1i32);
}
impl ::core::marker::Copy for ApplicationViewBoundsMode {}
impl ::core::clone::Clone for ApplicationViewBoundsMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ApplicationViewConsolidatedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_ViewManagement\"`*"]
#[repr(transparent)]
pub struct ApplicationViewMode(pub i32);
impl ApplicationViewMode {
    pub const Default: Self = Self(0i32);
    pub const CompactOverlay: Self = Self(1i32);
}
impl ::core::marker::Copy for ApplicationViewMode {}
impl ::core::clone::Clone for ApplicationViewMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_ViewManagement\"`*"]
#[repr(transparent)]
pub struct ApplicationViewOrientation(pub i32);
impl ApplicationViewOrientation {
    pub const Landscape: Self = Self(0i32);
    pub const Portrait: Self = Self(1i32);
}
impl ::core::marker::Copy for ApplicationViewOrientation {}
impl ::core::clone::Clone for ApplicationViewOrientation {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ApplicationViewScaling = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_ViewManagement\"`, `\"deprecated\"`*"]
#[cfg(feature = "deprecated")]
#[repr(transparent)]
pub struct ApplicationViewState(pub i32);
#[cfg(feature = "deprecated")]
impl ApplicationViewState {
    pub const FullScreenLandscape: Self = Self(0i32);
    pub const Filled: Self = Self(1i32);
    pub const Snapped: Self = Self(2i32);
    pub const FullScreenPortrait: Self = Self(3i32);
}
#[cfg(feature = "deprecated")]
impl ::core::marker::Copy for ApplicationViewState {}
#[cfg(feature = "deprecated")]
impl ::core::clone::Clone for ApplicationViewState {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_ViewManagement\"`*"]
#[repr(transparent)]
pub struct ApplicationViewSwitchingOptions(pub u32);
impl ApplicationViewSwitchingOptions {
    pub const Default: Self = Self(0u32);
    pub const SkipAnimation: Self = Self(1u32);
    pub const ConsolidateViews: Self = Self(2u32);
}
impl ::core::marker::Copy for ApplicationViewSwitchingOptions {}
impl ::core::clone::Clone for ApplicationViewSwitchingOptions {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ApplicationViewTitleBar = *mut ::core::ffi::c_void;
pub type ApplicationViewTransferContext = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_ViewManagement\"`*"]
#[repr(transparent)]
pub struct ApplicationViewWindowingMode(pub i32);
impl ApplicationViewWindowingMode {
    pub const Auto: Self = Self(0i32);
    pub const PreferredLaunchViewSize: Self = Self(1i32);
    pub const FullScreen: Self = Self(2i32);
    pub const CompactOverlay: Self = Self(3i32);
    pub const Maximized: Self = Self(4i32);
}
impl ::core::marker::Copy for ApplicationViewWindowingMode {}
impl ::core::clone::Clone for ApplicationViewWindowingMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_ViewManagement\"`*"]
#[repr(transparent)]
pub struct FullScreenSystemOverlayMode(pub i32);
impl FullScreenSystemOverlayMode {
    pub const Standard: Self = Self(0i32);
    pub const Minimal: Self = Self(1i32);
}
impl ::core::marker::Copy for FullScreenSystemOverlayMode {}
impl ::core::clone::Clone for FullScreenSystemOverlayMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_ViewManagement\"`*"]
#[repr(transparent)]
pub struct HandPreference(pub i32);
impl HandPreference {
    pub const LeftHanded: Self = Self(0i32);
    pub const RightHanded: Self = Self(1i32);
}
impl ::core::marker::Copy for HandPreference {}
impl ::core::clone::Clone for HandPreference {
    fn clone(&self) -> Self {
        *self
    }
}
pub type InputPane = *mut ::core::ffi::c_void;
pub type InputPaneVisibilityEventArgs = *mut ::core::ffi::c_void;
pub type StatusBar = *mut ::core::ffi::c_void;
pub type StatusBarProgressIndicator = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_ViewManagement\"`*"]
#[repr(transparent)]
pub struct UIColorType(pub i32);
impl UIColorType {
    pub const Background: Self = Self(0i32);
    pub const Foreground: Self = Self(1i32);
    pub const AccentDark3: Self = Self(2i32);
    pub const AccentDark2: Self = Self(3i32);
    pub const AccentDark1: Self = Self(4i32);
    pub const Accent: Self = Self(5i32);
    pub const AccentLight1: Self = Self(6i32);
    pub const AccentLight2: Self = Self(7i32);
    pub const AccentLight3: Self = Self(8i32);
    pub const Complement: Self = Self(9i32);
}
impl ::core::marker::Copy for UIColorType {}
impl ::core::clone::Clone for UIColorType {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_ViewManagement\"`*"]
#[repr(transparent)]
pub struct UIElementType(pub i32);
impl UIElementType {
    pub const ActiveCaption: Self = Self(0i32);
    pub const Background: Self = Self(1i32);
    pub const ButtonFace: Self = Self(2i32);
    pub const ButtonText: Self = Self(3i32);
    pub const CaptionText: Self = Self(4i32);
    pub const GrayText: Self = Self(5i32);
    pub const Highlight: Self = Self(6i32);
    pub const HighlightText: Self = Self(7i32);
    pub const Hotlight: Self = Self(8i32);
    pub const InactiveCaption: Self = Self(9i32);
    pub const InactiveCaptionText: Self = Self(10i32);
    pub const Window: Self = Self(11i32);
    pub const WindowText: Self = Self(12i32);
    pub const AccentColor: Self = Self(1000i32);
    pub const TextHigh: Self = Self(1001i32);
    pub const TextMedium: Self = Self(1002i32);
    pub const TextLow: Self = Self(1003i32);
    pub const TextContrastWithHigh: Self = Self(1004i32);
    pub const NonTextHigh: Self = Self(1005i32);
    pub const NonTextMediumHigh: Self = Self(1006i32);
    pub const NonTextMedium: Self = Self(1007i32);
    pub const NonTextMediumLow: Self = Self(1008i32);
    pub const NonTextLow: Self = Self(1009i32);
    pub const PageBackground: Self = Self(1010i32);
    pub const PopupBackground: Self = Self(1011i32);
    pub const OverlayOutsidePopup: Self = Self(1012i32);
}
impl ::core::marker::Copy for UIElementType {}
impl ::core::clone::Clone for UIElementType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type UISettings = *mut ::core::ffi::c_void;
pub type UISettingsAnimationsEnabledChangedEventArgs = *mut ::core::ffi::c_void;
pub type UISettingsAutoHideScrollBarsChangedEventArgs = *mut ::core::ffi::c_void;
pub type UISettingsMessageDurationChangedEventArgs = *mut ::core::ffi::c_void;
pub type UIViewSettings = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_ViewManagement\"`*"]
#[repr(transparent)]
pub struct UserInteractionMode(pub i32);
impl UserInteractionMode {
    pub const Mouse: Self = Self(0i32);
    pub const Touch: Self = Self(1i32);
}
impl ::core::marker::Copy for UserInteractionMode {}
impl ::core::clone::Clone for UserInteractionMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ViewModePreferences = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_ViewManagement\"`*"]
#[repr(transparent)]
pub struct ViewSizePreference(pub i32);
impl ViewSizePreference {
    pub const Default: Self = Self(0i32);
    pub const UseLess: Self = Self(1i32);
    pub const UseHalf: Self = Self(2i32);
    pub const UseMore: Self = Self(3i32);
    pub const UseMinimum: Self = Self(4i32);
    pub const UseNone: Self = Self(5i32);
    pub const Custom: Self = Self(6i32);
}
impl ::core::marker::Copy for ViewSizePreference {}
impl ::core::clone::Clone for ViewSizePreference {
    fn clone(&self) -> Self {
        *self
    }
}

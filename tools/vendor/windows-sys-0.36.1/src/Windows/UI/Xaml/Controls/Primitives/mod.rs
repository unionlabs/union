#[doc = "*Required features: `\"UI_Xaml_Controls_Primitives\"`*"]
#[repr(transparent)]
pub struct AnimationDirection(pub i32);
impl AnimationDirection {
    pub const Left: Self = Self(0i32);
    pub const Top: Self = Self(1i32);
    pub const Right: Self = Self(2i32);
    pub const Bottom: Self = Self(3i32);
}
impl ::core::marker::Copy for AnimationDirection {}
impl ::core::clone::Clone for AnimationDirection {
    fn clone(&self) -> Self {
        *self
    }
}
pub type AppBarButtonTemplateSettings = *mut ::core::ffi::c_void;
pub type AppBarTemplateSettings = *mut ::core::ffi::c_void;
pub type AppBarToggleButtonTemplateSettings = *mut ::core::ffi::c_void;
pub type ButtonBase = *mut ::core::ffi::c_void;
pub type CalendarPanel = *mut ::core::ffi::c_void;
pub type CalendarViewTemplateSettings = *mut ::core::ffi::c_void;
pub type CarouselPanel = *mut ::core::ffi::c_void;
pub type ColorPickerSlider = *mut ::core::ffi::c_void;
pub type ColorSpectrum = *mut ::core::ffi::c_void;
pub type ComboBoxTemplateSettings = *mut ::core::ffi::c_void;
pub type CommandBarFlyoutCommandBar = *mut ::core::ffi::c_void;
pub type CommandBarFlyoutCommandBarTemplateSettings = *mut ::core::ffi::c_void;
pub type CommandBarTemplateSettings = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls_Primitives\"`*"]
#[repr(transparent)]
pub struct ComponentResourceLocation(pub i32);
impl ComponentResourceLocation {
    pub const Application: Self = Self(0i32);
    pub const Nested: Self = Self(1i32);
}
impl ::core::marker::Copy for ComponentResourceLocation {}
impl ::core::clone::Clone for ComponentResourceLocation {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DragCompletedEventArgs = *mut ::core::ffi::c_void;
pub type DragCompletedEventHandler = *mut ::core::ffi::c_void;
pub type DragDeltaEventArgs = *mut ::core::ffi::c_void;
pub type DragDeltaEventHandler = *mut ::core::ffi::c_void;
pub type DragStartedEventArgs = *mut ::core::ffi::c_void;
pub type DragStartedEventHandler = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls_Primitives\"`*"]
#[repr(transparent)]
pub struct EdgeTransitionLocation(pub i32);
impl EdgeTransitionLocation {
    pub const Left: Self = Self(0i32);
    pub const Top: Self = Self(1i32);
    pub const Right: Self = Self(2i32);
    pub const Bottom: Self = Self(3i32);
}
impl ::core::marker::Copy for EdgeTransitionLocation {}
impl ::core::clone::Clone for EdgeTransitionLocation {
    fn clone(&self) -> Self {
        *self
    }
}
pub type FlyoutBase = *mut ::core::ffi::c_void;
pub type FlyoutBaseClosingEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls_Primitives\"`*"]
#[repr(transparent)]
pub struct FlyoutPlacementMode(pub i32);
impl FlyoutPlacementMode {
    pub const Top: Self = Self(0i32);
    pub const Bottom: Self = Self(1i32);
    pub const Left: Self = Self(2i32);
    pub const Right: Self = Self(3i32);
    pub const Full: Self = Self(4i32);
    pub const TopEdgeAlignedLeft: Self = Self(5i32);
    pub const TopEdgeAlignedRight: Self = Self(6i32);
    pub const BottomEdgeAlignedLeft: Self = Self(7i32);
    pub const BottomEdgeAlignedRight: Self = Self(8i32);
    pub const LeftEdgeAlignedTop: Self = Self(9i32);
    pub const LeftEdgeAlignedBottom: Self = Self(10i32);
    pub const RightEdgeAlignedTop: Self = Self(11i32);
    pub const RightEdgeAlignedBottom: Self = Self(12i32);
    pub const Auto: Self = Self(13i32);
}
impl ::core::marker::Copy for FlyoutPlacementMode {}
impl ::core::clone::Clone for FlyoutPlacementMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Controls_Primitives\"`*"]
#[repr(transparent)]
pub struct FlyoutShowMode(pub i32);
impl FlyoutShowMode {
    pub const Auto: Self = Self(0i32);
    pub const Standard: Self = Self(1i32);
    pub const Transient: Self = Self(2i32);
    pub const TransientWithDismissOnPointerMoveAway: Self = Self(3i32);
}
impl ::core::marker::Copy for FlyoutShowMode {}
impl ::core::clone::Clone for FlyoutShowMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type FlyoutShowOptions = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls_Primitives\"`*"]
#[repr(transparent)]
pub struct GeneratorDirection(pub i32);
impl GeneratorDirection {
    pub const Forward: Self = Self(0i32);
    pub const Backward: Self = Self(1i32);
}
impl ::core::marker::Copy for GeneratorDirection {}
impl ::core::clone::Clone for GeneratorDirection {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"UI_Xaml_Controls_Primitives\"`*"]
pub struct GeneratorPosition {
    pub Index: i32,
    pub Offset: i32,
}
impl ::core::marker::Copy for GeneratorPosition {}
impl ::core::clone::Clone for GeneratorPosition {
    fn clone(&self) -> Self {
        *self
    }
}
pub type GeneratorPositionHelper = *mut ::core::ffi::c_void;
pub type GridViewItemPresenter = *mut ::core::ffi::c_void;
pub type GridViewItemTemplateSettings = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls_Primitives\"`*"]
#[repr(transparent)]
pub struct GroupHeaderPlacement(pub i32);
impl GroupHeaderPlacement {
    pub const Top: Self = Self(0i32);
    pub const Left: Self = Self(1i32);
}
impl ::core::marker::Copy for GroupHeaderPlacement {}
impl ::core::clone::Clone for GroupHeaderPlacement {
    fn clone(&self) -> Self {
        *self
    }
}
pub type IScrollSnapPointsInfo = *mut ::core::ffi::c_void;
pub type ItemsChangedEventArgs = *mut ::core::ffi::c_void;
pub type ItemsChangedEventHandler = *mut ::core::ffi::c_void;
pub type JumpListItemBackgroundConverter = *mut ::core::ffi::c_void;
pub type JumpListItemForegroundConverter = *mut ::core::ffi::c_void;
pub type LayoutInformation = *mut ::core::ffi::c_void;
pub type ListViewItemPresenter = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls_Primitives\"`*"]
#[repr(transparent)]
pub struct ListViewItemPresenterCheckMode(pub i32);
impl ListViewItemPresenterCheckMode {
    pub const Inline: Self = Self(0i32);
    pub const Overlay: Self = Self(1i32);
}
impl ::core::marker::Copy for ListViewItemPresenterCheckMode {}
impl ::core::clone::Clone for ListViewItemPresenterCheckMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Controls_Primitives\"`*"]
#[repr(transparent)]
pub struct ListViewItemPresenterSelectionIndicatorMode(pub i32);
impl ListViewItemPresenterSelectionIndicatorMode {
    pub const Inline: Self = Self(0i32);
    pub const Overlay: Self = Self(1i32);
}
impl ::core::marker::Copy for ListViewItemPresenterSelectionIndicatorMode {}
impl ::core::clone::Clone for ListViewItemPresenterSelectionIndicatorMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ListViewItemTemplateSettings = *mut ::core::ffi::c_void;
pub type LoopingSelector = *mut ::core::ffi::c_void;
pub type LoopingSelectorItem = *mut ::core::ffi::c_void;
pub type LoopingSelectorPanel = *mut ::core::ffi::c_void;
pub type MenuFlyoutItemTemplateSettings = *mut ::core::ffi::c_void;
pub type MenuFlyoutPresenterTemplateSettings = *mut ::core::ffi::c_void;
pub type NavigationViewItemPresenter = *mut ::core::ffi::c_void;
pub type OrientedVirtualizingPanel = *mut ::core::ffi::c_void;
pub type PickerFlyoutBase = *mut ::core::ffi::c_void;
pub type PivotHeaderItem = *mut ::core::ffi::c_void;
pub type PivotHeaderPanel = *mut ::core::ffi::c_void;
pub type PivotPanel = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls_Primitives\"`*"]
#[repr(transparent)]
pub struct PlacementMode(pub i32);
impl PlacementMode {
    pub const Bottom: Self = Self(2i32);
    pub const Left: Self = Self(9i32);
    pub const Mouse: Self = Self(7i32);
    pub const Right: Self = Self(4i32);
    pub const Top: Self = Self(10i32);
}
impl ::core::marker::Copy for PlacementMode {}
impl ::core::clone::Clone for PlacementMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type Popup = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls_Primitives\"`*"]
#[repr(transparent)]
pub struct PopupPlacementMode(pub i32);
impl PopupPlacementMode {
    pub const Auto: Self = Self(0i32);
    pub const Top: Self = Self(1i32);
    pub const Bottom: Self = Self(2i32);
    pub const Left: Self = Self(3i32);
    pub const Right: Self = Self(4i32);
    pub const TopEdgeAlignedLeft: Self = Self(5i32);
    pub const TopEdgeAlignedRight: Self = Self(6i32);
    pub const BottomEdgeAlignedLeft: Self = Self(7i32);
    pub const BottomEdgeAlignedRight: Self = Self(8i32);
    pub const LeftEdgeAlignedTop: Self = Self(9i32);
    pub const LeftEdgeAlignedBottom: Self = Self(10i32);
    pub const RightEdgeAlignedTop: Self = Self(11i32);
    pub const RightEdgeAlignedBottom: Self = Self(12i32);
}
impl ::core::marker::Copy for PopupPlacementMode {}
impl ::core::clone::Clone for PopupPlacementMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ProgressBarTemplateSettings = *mut ::core::ffi::c_void;
pub type ProgressRingTemplateSettings = *mut ::core::ffi::c_void;
pub type RangeBase = *mut ::core::ffi::c_void;
pub type RangeBaseValueChangedEventArgs = *mut ::core::ffi::c_void;
pub type RangeBaseValueChangedEventHandler = *mut ::core::ffi::c_void;
pub type RepeatButton = *mut ::core::ffi::c_void;
pub type ScrollBar = *mut ::core::ffi::c_void;
pub type ScrollEventArgs = *mut ::core::ffi::c_void;
pub type ScrollEventHandler = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls_Primitives\"`*"]
#[repr(transparent)]
pub struct ScrollEventType(pub i32);
impl ScrollEventType {
    pub const SmallDecrement: Self = Self(0i32);
    pub const SmallIncrement: Self = Self(1i32);
    pub const LargeDecrement: Self = Self(2i32);
    pub const LargeIncrement: Self = Self(3i32);
    pub const ThumbPosition: Self = Self(4i32);
    pub const ThumbTrack: Self = Self(5i32);
    pub const First: Self = Self(6i32);
    pub const Last: Self = Self(7i32);
    pub const EndScroll: Self = Self(8i32);
}
impl ::core::marker::Copy for ScrollEventType {}
impl ::core::clone::Clone for ScrollEventType {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Controls_Primitives\"`*"]
#[repr(transparent)]
pub struct ScrollingIndicatorMode(pub i32);
impl ScrollingIndicatorMode {
    pub const None: Self = Self(0i32);
    pub const TouchIndicator: Self = Self(1i32);
    pub const MouseIndicator: Self = Self(2i32);
}
impl ::core::marker::Copy for ScrollingIndicatorMode {}
impl ::core::clone::Clone for ScrollingIndicatorMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type Selector = *mut ::core::ffi::c_void;
pub type SelectorItem = *mut ::core::ffi::c_void;
pub type SettingsFlyoutTemplateSettings = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls_Primitives\"`*"]
#[repr(transparent)]
pub struct SliderSnapsTo(pub i32);
impl SliderSnapsTo {
    pub const StepValues: Self = Self(0i32);
    pub const Ticks: Self = Self(1i32);
}
impl ::core::marker::Copy for SliderSnapsTo {}
impl ::core::clone::Clone for SliderSnapsTo {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Controls_Primitives\"`*"]
#[repr(transparent)]
pub struct SnapPointsAlignment(pub i32);
impl SnapPointsAlignment {
    pub const Near: Self = Self(0i32);
    pub const Center: Self = Self(1i32);
    pub const Far: Self = Self(2i32);
}
impl ::core::marker::Copy for SnapPointsAlignment {}
impl ::core::clone::Clone for SnapPointsAlignment {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SplitViewTemplateSettings = *mut ::core::ffi::c_void;
pub type Thumb = *mut ::core::ffi::c_void;
pub type TickBar = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls_Primitives\"`*"]
#[repr(transparent)]
pub struct TickPlacement(pub i32);
impl TickPlacement {
    pub const None: Self = Self(0i32);
    pub const TopLeft: Self = Self(1i32);
    pub const BottomRight: Self = Self(2i32);
    pub const Outside: Self = Self(3i32);
    pub const Inline: Self = Self(4i32);
}
impl ::core::marker::Copy for TickPlacement {}
impl ::core::clone::Clone for TickPlacement {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ToggleButton = *mut ::core::ffi::c_void;
pub type ToggleSwitchTemplateSettings = *mut ::core::ffi::c_void;
pub type ToolTipTemplateSettings = *mut ::core::ffi::c_void;

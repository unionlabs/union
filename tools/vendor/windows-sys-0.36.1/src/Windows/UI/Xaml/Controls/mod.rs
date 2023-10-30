#[cfg(feature = "UI_Xaml_Controls_Maps")]
pub mod Maps;
#[cfg(feature = "UI_Xaml_Controls_Primitives")]
pub mod Primitives;
pub type AnchorRequestedEventArgs = *mut ::core::ffi::c_void;
pub type AppBar = *mut ::core::ffi::c_void;
pub type AppBarButton = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct AppBarClosedDisplayMode(pub i32);
impl AppBarClosedDisplayMode {
    pub const Compact: Self = Self(0i32);
    pub const Minimal: Self = Self(1i32);
    pub const Hidden: Self = Self(2i32);
}
impl ::core::marker::Copy for AppBarClosedDisplayMode {}
impl ::core::clone::Clone for AppBarClosedDisplayMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type AppBarElementContainer = *mut ::core::ffi::c_void;
pub type AppBarSeparator = *mut ::core::ffi::c_void;
pub type AppBarToggleButton = *mut ::core::ffi::c_void;
pub type AutoSuggestBox = *mut ::core::ffi::c_void;
pub type AutoSuggestBoxQuerySubmittedEventArgs = *mut ::core::ffi::c_void;
pub type AutoSuggestBoxSuggestionChosenEventArgs = *mut ::core::ffi::c_void;
pub type AutoSuggestBoxTextChangedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct AutoSuggestionBoxTextChangeReason(pub i32);
impl AutoSuggestionBoxTextChangeReason {
    pub const UserInput: Self = Self(0i32);
    pub const ProgrammaticChange: Self = Self(1i32);
    pub const SuggestionChosen: Self = Self(2i32);
}
impl ::core::marker::Copy for AutoSuggestionBoxTextChangeReason {}
impl ::core::clone::Clone for AutoSuggestionBoxTextChangeReason {
    fn clone(&self) -> Self {
        *self
    }
}
pub type BackClickEventArgs = *mut ::core::ffi::c_void;
pub type BackClickEventHandler = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct BackgroundSizing(pub i32);
impl BackgroundSizing {
    pub const InnerBorderEdge: Self = Self(0i32);
    pub const OuterBorderEdge: Self = Self(1i32);
}
impl ::core::marker::Copy for BackgroundSizing {}
impl ::core::clone::Clone for BackgroundSizing {
    fn clone(&self) -> Self {
        *self
    }
}
pub type BitmapIcon = *mut ::core::ffi::c_void;
pub type BitmapIconSource = *mut ::core::ffi::c_void;
pub type Border = *mut ::core::ffi::c_void;
pub type Button = *mut ::core::ffi::c_void;
pub type CalendarDatePicker = *mut ::core::ffi::c_void;
pub type CalendarDatePickerDateChangedEventArgs = *mut ::core::ffi::c_void;
pub type CalendarView = *mut ::core::ffi::c_void;
pub type CalendarViewDayItem = *mut ::core::ffi::c_void;
pub type CalendarViewDayItemChangingEventArgs = *mut ::core::ffi::c_void;
pub type CalendarViewDayItemChangingEventHandler = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct CalendarViewDisplayMode(pub i32);
impl CalendarViewDisplayMode {
    pub const Month: Self = Self(0i32);
    pub const Year: Self = Self(1i32);
    pub const Decade: Self = Self(2i32);
}
impl ::core::marker::Copy for CalendarViewDisplayMode {}
impl ::core::clone::Clone for CalendarViewDisplayMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CalendarViewSelectedDatesChangedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct CalendarViewSelectionMode(pub i32);
impl CalendarViewSelectionMode {
    pub const None: Self = Self(0i32);
    pub const Single: Self = Self(1i32);
    pub const Multiple: Self = Self(2i32);
}
impl ::core::marker::Copy for CalendarViewSelectionMode {}
impl ::core::clone::Clone for CalendarViewSelectionMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct CandidateWindowAlignment(pub i32);
impl CandidateWindowAlignment {
    pub const Default: Self = Self(0i32);
    pub const BottomEdge: Self = Self(1i32);
}
impl ::core::marker::Copy for CandidateWindowAlignment {}
impl ::core::clone::Clone for CandidateWindowAlignment {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CandidateWindowBoundsChangedEventArgs = *mut ::core::ffi::c_void;
pub type Canvas = *mut ::core::ffi::c_void;
pub type CaptureElement = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct CharacterCasing(pub i32);
impl CharacterCasing {
    pub const Normal: Self = Self(0i32);
    pub const Lower: Self = Self(1i32);
    pub const Upper: Self = Self(2i32);
}
impl ::core::marker::Copy for CharacterCasing {}
impl ::core::clone::Clone for CharacterCasing {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CheckBox = *mut ::core::ffi::c_void;
pub type ChoosingGroupHeaderContainerEventArgs = *mut ::core::ffi::c_void;
pub type ChoosingItemContainerEventArgs = *mut ::core::ffi::c_void;
pub type CleanUpVirtualizedItemEventArgs = *mut ::core::ffi::c_void;
pub type CleanUpVirtualizedItemEventHandler = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct ClickMode(pub i32);
impl ClickMode {
    pub const Release: Self = Self(0i32);
    pub const Press: Self = Self(1i32);
    pub const Hover: Self = Self(2i32);
}
impl ::core::marker::Copy for ClickMode {}
impl ::core::clone::Clone for ClickMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ColorChangedEventArgs = *mut ::core::ffi::c_void;
pub type ColorPicker = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct ColorPickerHsvChannel(pub i32);
impl ColorPickerHsvChannel {
    pub const Hue: Self = Self(0i32);
    pub const Saturation: Self = Self(1i32);
    pub const Value: Self = Self(2i32);
    pub const Alpha: Self = Self(3i32);
}
impl ::core::marker::Copy for ColorPickerHsvChannel {}
impl ::core::clone::Clone for ColorPickerHsvChannel {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct ColorSpectrumComponents(pub i32);
impl ColorSpectrumComponents {
    pub const HueValue: Self = Self(0i32);
    pub const ValueHue: Self = Self(1i32);
    pub const HueSaturation: Self = Self(2i32);
    pub const SaturationHue: Self = Self(3i32);
    pub const SaturationValue: Self = Self(4i32);
    pub const ValueSaturation: Self = Self(5i32);
}
impl ::core::marker::Copy for ColorSpectrumComponents {}
impl ::core::clone::Clone for ColorSpectrumComponents {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct ColorSpectrumShape(pub i32);
impl ColorSpectrumShape {
    pub const Box: Self = Self(0i32);
    pub const Ring: Self = Self(1i32);
}
impl ::core::marker::Copy for ColorSpectrumShape {}
impl ::core::clone::Clone for ColorSpectrumShape {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ColumnDefinition = *mut ::core::ffi::c_void;
pub type ColumnDefinitionCollection = *mut ::core::ffi::c_void;
pub type ComboBox = *mut ::core::ffi::c_void;
pub type ComboBoxItem = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct ComboBoxSelectionChangedTrigger(pub i32);
impl ComboBoxSelectionChangedTrigger {
    pub const Committed: Self = Self(0i32);
    pub const Always: Self = Self(1i32);
}
impl ::core::marker::Copy for ComboBoxSelectionChangedTrigger {}
impl ::core::clone::Clone for ComboBoxSelectionChangedTrigger {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ComboBoxTextSubmittedEventArgs = *mut ::core::ffi::c_void;
pub type CommandBar = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct CommandBarDefaultLabelPosition(pub i32);
impl CommandBarDefaultLabelPosition {
    pub const Bottom: Self = Self(0i32);
    pub const Right: Self = Self(1i32);
    pub const Collapsed: Self = Self(2i32);
}
impl ::core::marker::Copy for CommandBarDefaultLabelPosition {}
impl ::core::clone::Clone for CommandBarDefaultLabelPosition {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct CommandBarDynamicOverflowAction(pub i32);
impl CommandBarDynamicOverflowAction {
    pub const AddingToOverflow: Self = Self(0i32);
    pub const RemovingFromOverflow: Self = Self(1i32);
}
impl ::core::marker::Copy for CommandBarDynamicOverflowAction {}
impl ::core::clone::Clone for CommandBarDynamicOverflowAction {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CommandBarFlyout = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct CommandBarLabelPosition(pub i32);
impl CommandBarLabelPosition {
    pub const Default: Self = Self(0i32);
    pub const Collapsed: Self = Self(1i32);
}
impl ::core::marker::Copy for CommandBarLabelPosition {}
impl ::core::clone::Clone for CommandBarLabelPosition {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct CommandBarOverflowButtonVisibility(pub i32);
impl CommandBarOverflowButtonVisibility {
    pub const Auto: Self = Self(0i32);
    pub const Visible: Self = Self(1i32);
    pub const Collapsed: Self = Self(2i32);
}
impl ::core::marker::Copy for CommandBarOverflowButtonVisibility {}
impl ::core::clone::Clone for CommandBarOverflowButtonVisibility {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CommandBarOverflowPresenter = *mut ::core::ffi::c_void;
pub type ContainerContentChangingEventArgs = *mut ::core::ffi::c_void;
pub type ContentControl = *mut ::core::ffi::c_void;
pub type ContentDialog = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct ContentDialogButton(pub i32);
impl ContentDialogButton {
    pub const None: Self = Self(0i32);
    pub const Primary: Self = Self(1i32);
    pub const Secondary: Self = Self(2i32);
    pub const Close: Self = Self(3i32);
}
impl ::core::marker::Copy for ContentDialogButton {}
impl ::core::clone::Clone for ContentDialogButton {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ContentDialogButtonClickDeferral = *mut ::core::ffi::c_void;
pub type ContentDialogButtonClickEventArgs = *mut ::core::ffi::c_void;
pub type ContentDialogClosedEventArgs = *mut ::core::ffi::c_void;
pub type ContentDialogClosingDeferral = *mut ::core::ffi::c_void;
pub type ContentDialogClosingEventArgs = *mut ::core::ffi::c_void;
pub type ContentDialogOpenedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct ContentDialogPlacement(pub i32);
impl ContentDialogPlacement {
    pub const Popup: Self = Self(0i32);
    pub const InPlace: Self = Self(1i32);
}
impl ::core::marker::Copy for ContentDialogPlacement {}
impl ::core::clone::Clone for ContentDialogPlacement {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct ContentDialogResult(pub i32);
impl ContentDialogResult {
    pub const None: Self = Self(0i32);
    pub const Primary: Self = Self(1i32);
    pub const Secondary: Self = Self(2i32);
}
impl ::core::marker::Copy for ContentDialogResult {}
impl ::core::clone::Clone for ContentDialogResult {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct ContentLinkChangeKind(pub i32);
impl ContentLinkChangeKind {
    pub const Inserted: Self = Self(0i32);
    pub const Removed: Self = Self(1i32);
    pub const Edited: Self = Self(2i32);
}
impl ::core::marker::Copy for ContentLinkChangeKind {}
impl ::core::clone::Clone for ContentLinkChangeKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ContentLinkChangedEventArgs = *mut ::core::ffi::c_void;
pub type ContentPresenter = *mut ::core::ffi::c_void;
pub type ContextMenuEventArgs = *mut ::core::ffi::c_void;
pub type ContextMenuOpeningEventHandler = *mut ::core::ffi::c_void;
pub type Control = *mut ::core::ffi::c_void;
pub type ControlTemplate = *mut ::core::ffi::c_void;
pub type DataTemplateSelector = *mut ::core::ffi::c_void;
pub type DatePickedEventArgs = *mut ::core::ffi::c_void;
pub type DatePicker = *mut ::core::ffi::c_void;
pub type DatePickerFlyout = *mut ::core::ffi::c_void;
pub type DatePickerFlyoutItem = *mut ::core::ffi::c_void;
pub type DatePickerFlyoutPresenter = *mut ::core::ffi::c_void;
pub type DatePickerSelectedValueChangedEventArgs = *mut ::core::ffi::c_void;
pub type DatePickerValueChangedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct DisabledFormattingAccelerators(pub u32);
impl DisabledFormattingAccelerators {
    pub const None: Self = Self(0u32);
    pub const Bold: Self = Self(1u32);
    pub const Italic: Self = Self(2u32);
    pub const Underline: Self = Self(4u32);
    pub const All: Self = Self(4294967295u32);
}
impl ::core::marker::Copy for DisabledFormattingAccelerators {}
impl ::core::clone::Clone for DisabledFormattingAccelerators {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DragItemsCompletedEventArgs = *mut ::core::ffi::c_void;
pub type DragItemsStartingEventArgs = *mut ::core::ffi::c_void;
pub type DragItemsStartingEventHandler = *mut ::core::ffi::c_void;
pub type DropDownButton = *mut ::core::ffi::c_void;
pub type DropDownButtonAutomationPeer = *mut ::core::ffi::c_void;
pub type DynamicOverflowItemsChangingEventArgs = *mut ::core::ffi::c_void;
pub type FlipView = *mut ::core::ffi::c_void;
pub type FlipViewItem = *mut ::core::ffi::c_void;
pub type Flyout = *mut ::core::ffi::c_void;
pub type FlyoutPresenter = *mut ::core::ffi::c_void;
pub type FocusDisengagedEventArgs = *mut ::core::ffi::c_void;
pub type FocusEngagedEventArgs = *mut ::core::ffi::c_void;
pub type FontIcon = *mut ::core::ffi::c_void;
pub type FontIconSource = *mut ::core::ffi::c_void;
pub type Frame = *mut ::core::ffi::c_void;
pub type Grid = *mut ::core::ffi::c_void;
pub type GridView = *mut ::core::ffi::c_void;
pub type GridViewHeaderItem = *mut ::core::ffi::c_void;
pub type GridViewItem = *mut ::core::ffi::c_void;
pub type GroupItem = *mut ::core::ffi::c_void;
pub type GroupStyle = *mut ::core::ffi::c_void;
pub type GroupStyleSelector = *mut ::core::ffi::c_void;
pub type HandwritingPanelClosedEventArgs = *mut ::core::ffi::c_void;
pub type HandwritingPanelOpenedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct HandwritingPanelPlacementAlignment(pub i32);
impl HandwritingPanelPlacementAlignment {
    pub const Auto: Self = Self(0i32);
    pub const TopLeft: Self = Self(1i32);
    pub const TopRight: Self = Self(2i32);
    pub const BottomLeft: Self = Self(3i32);
    pub const BottomRight: Self = Self(4i32);
}
impl ::core::marker::Copy for HandwritingPanelPlacementAlignment {}
impl ::core::clone::Clone for HandwritingPanelPlacementAlignment {
    fn clone(&self) -> Self {
        *self
    }
}
pub type HandwritingView = *mut ::core::ffi::c_void;
pub type HandwritingViewCandidatesChangedEventArgs = *mut ::core::ffi::c_void;
pub type HandwritingViewTextSubmittedEventArgs = *mut ::core::ffi::c_void;
pub type Hub = *mut ::core::ffi::c_void;
pub type HubSection = *mut ::core::ffi::c_void;
pub type HubSectionCollection = *mut ::core::ffi::c_void;
pub type HubSectionHeaderClickEventArgs = *mut ::core::ffi::c_void;
pub type HubSectionHeaderClickEventHandler = *mut ::core::ffi::c_void;
pub type HyperlinkButton = *mut ::core::ffi::c_void;
pub type ICommandBarElement = *mut ::core::ffi::c_void;
pub type ICommandBarElement2 = *mut ::core::ffi::c_void;
pub type IInsertionPanel = *mut ::core::ffi::c_void;
pub type IItemContainerMapping = *mut ::core::ffi::c_void;
pub type INavigate = *mut ::core::ffi::c_void;
pub type IScrollAnchorProvider = *mut ::core::ffi::c_void;
pub type ISemanticZoomInformation = *mut ::core::ffi::c_void;
pub type IconElement = *mut ::core::ffi::c_void;
pub type IconSource = *mut ::core::ffi::c_void;
pub type IconSourceElement = *mut ::core::ffi::c_void;
pub type Image = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct IncrementalLoadingTrigger(pub i32);
impl IncrementalLoadingTrigger {
    pub const None: Self = Self(0i32);
    pub const Edge: Self = Self(1i32);
}
impl ::core::marker::Copy for IncrementalLoadingTrigger {}
impl ::core::clone::Clone for IncrementalLoadingTrigger {
    fn clone(&self) -> Self {
        *self
    }
}
pub type InkCanvas = *mut ::core::ffi::c_void;
pub type InkToolbar = *mut ::core::ffi::c_void;
pub type InkToolbarBallpointPenButton = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct InkToolbarButtonFlyoutPlacement(pub i32);
impl InkToolbarButtonFlyoutPlacement {
    pub const Auto: Self = Self(0i32);
    pub const Top: Self = Self(1i32);
    pub const Bottom: Self = Self(2i32);
    pub const Left: Self = Self(3i32);
    pub const Right: Self = Self(4i32);
}
impl ::core::marker::Copy for InkToolbarButtonFlyoutPlacement {}
impl ::core::clone::Clone for InkToolbarButtonFlyoutPlacement {
    fn clone(&self) -> Self {
        *self
    }
}
pub type InkToolbarCustomPen = *mut ::core::ffi::c_void;
pub type InkToolbarCustomPenButton = *mut ::core::ffi::c_void;
pub type InkToolbarCustomToggleButton = *mut ::core::ffi::c_void;
pub type InkToolbarCustomToolButton = *mut ::core::ffi::c_void;
pub type InkToolbarEraserButton = *mut ::core::ffi::c_void;
pub type InkToolbarFlyoutItem = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct InkToolbarFlyoutItemKind(pub i32);
impl InkToolbarFlyoutItemKind {
    pub const Simple: Self = Self(0i32);
    pub const Radio: Self = Self(1i32);
    pub const Check: Self = Self(2i32);
    pub const RadioCheck: Self = Self(3i32);
}
impl ::core::marker::Copy for InkToolbarFlyoutItemKind {}
impl ::core::clone::Clone for InkToolbarFlyoutItemKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type InkToolbarHighlighterButton = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct InkToolbarInitialControls(pub i32);
impl InkToolbarInitialControls {
    pub const All: Self = Self(0i32);
    pub const None: Self = Self(1i32);
    pub const PensOnly: Self = Self(2i32);
    pub const AllExceptPens: Self = Self(3i32);
}
impl ::core::marker::Copy for InkToolbarInitialControls {}
impl ::core::clone::Clone for InkToolbarInitialControls {
    fn clone(&self) -> Self {
        *self
    }
}
pub type InkToolbarIsStencilButtonCheckedChangedEventArgs = *mut ::core::ffi::c_void;
pub type InkToolbarMenuButton = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct InkToolbarMenuKind(pub i32);
impl InkToolbarMenuKind {
    pub const Stencil: Self = Self(0i32);
}
impl ::core::marker::Copy for InkToolbarMenuKind {}
impl ::core::clone::Clone for InkToolbarMenuKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type InkToolbarPenButton = *mut ::core::ffi::c_void;
pub type InkToolbarPenConfigurationControl = *mut ::core::ffi::c_void;
pub type InkToolbarPencilButton = *mut ::core::ffi::c_void;
pub type InkToolbarRulerButton = *mut ::core::ffi::c_void;
pub type InkToolbarStencilButton = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct InkToolbarStencilKind(pub i32);
impl InkToolbarStencilKind {
    pub const Ruler: Self = Self(0i32);
    pub const Protractor: Self = Self(1i32);
}
impl ::core::marker::Copy for InkToolbarStencilKind {}
impl ::core::clone::Clone for InkToolbarStencilKind {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct InkToolbarToggle(pub i32);
impl InkToolbarToggle {
    pub const Ruler: Self = Self(0i32);
    pub const Custom: Self = Self(1i32);
}
impl ::core::marker::Copy for InkToolbarToggle {}
impl ::core::clone::Clone for InkToolbarToggle {
    fn clone(&self) -> Self {
        *self
    }
}
pub type InkToolbarToggleButton = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct InkToolbarTool(pub i32);
impl InkToolbarTool {
    pub const BallpointPen: Self = Self(0i32);
    pub const Pencil: Self = Self(1i32);
    pub const Highlighter: Self = Self(2i32);
    pub const Eraser: Self = Self(3i32);
    pub const CustomPen: Self = Self(4i32);
    pub const CustomTool: Self = Self(5i32);
}
impl ::core::marker::Copy for InkToolbarTool {}
impl ::core::clone::Clone for InkToolbarTool {
    fn clone(&self) -> Self {
        *self
    }
}
pub type InkToolbarToolButton = *mut ::core::ffi::c_void;
pub type IsTextTrimmedChangedEventArgs = *mut ::core::ffi::c_void;
pub type ItemClickEventArgs = *mut ::core::ffi::c_void;
pub type ItemClickEventHandler = *mut ::core::ffi::c_void;
pub type ItemCollection = *mut ::core::ffi::c_void;
pub type ItemContainerGenerator = *mut ::core::ffi::c_void;
pub type ItemsControl = *mut ::core::ffi::c_void;
pub type ItemsPanelTemplate = *mut ::core::ffi::c_void;
pub type ItemsPickedEventArgs = *mut ::core::ffi::c_void;
pub type ItemsPresenter = *mut ::core::ffi::c_void;
pub type ItemsStackPanel = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct ItemsUpdatingScrollMode(pub i32);
impl ItemsUpdatingScrollMode {
    pub const KeepItemsInView: Self = Self(0i32);
    pub const KeepScrollOffset: Self = Self(1i32);
    pub const KeepLastItemInView: Self = Self(2i32);
}
impl ::core::marker::Copy for ItemsUpdatingScrollMode {}
impl ::core::clone::Clone for ItemsUpdatingScrollMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ItemsWrapGrid = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct LightDismissOverlayMode(pub i32);
impl LightDismissOverlayMode {
    pub const Auto: Self = Self(0i32);
    pub const On: Self = Self(1i32);
    pub const Off: Self = Self(2i32);
}
impl ::core::marker::Copy for LightDismissOverlayMode {}
impl ::core::clone::Clone for LightDismissOverlayMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ListBox = *mut ::core::ffi::c_void;
pub type ListBoxItem = *mut ::core::ffi::c_void;
pub type ListPickerFlyout = *mut ::core::ffi::c_void;
pub type ListPickerFlyoutPresenter = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct ListPickerFlyoutSelectionMode(pub i32);
impl ListPickerFlyoutSelectionMode {
    pub const Single: Self = Self(0i32);
    pub const Multiple: Self = Self(1i32);
}
impl ::core::marker::Copy for ListPickerFlyoutSelectionMode {}
impl ::core::clone::Clone for ListPickerFlyoutSelectionMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ListView = *mut ::core::ffi::c_void;
pub type ListViewBase = *mut ::core::ffi::c_void;
pub type ListViewBaseHeaderItem = *mut ::core::ffi::c_void;
pub type ListViewHeaderItem = *mut ::core::ffi::c_void;
pub type ListViewItem = *mut ::core::ffi::c_void;
pub type ListViewItemToKeyHandler = *mut ::core::ffi::c_void;
pub type ListViewKeyToItemHandler = *mut ::core::ffi::c_void;
pub type ListViewPersistenceHelper = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct ListViewReorderMode(pub i32);
impl ListViewReorderMode {
    pub const Disabled: Self = Self(0i32);
    pub const Enabled: Self = Self(1i32);
}
impl ::core::marker::Copy for ListViewReorderMode {}
impl ::core::clone::Clone for ListViewReorderMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct ListViewSelectionMode(pub i32);
impl ListViewSelectionMode {
    pub const None: Self = Self(0i32);
    pub const Single: Self = Self(1i32);
    pub const Multiple: Self = Self(2i32);
    pub const Extended: Self = Self(3i32);
}
impl ::core::marker::Copy for ListViewSelectionMode {}
impl ::core::clone::Clone for ListViewSelectionMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MediaElement = *mut ::core::ffi::c_void;
pub type MediaPlayerElement = *mut ::core::ffi::c_void;
pub type MediaPlayerPresenter = *mut ::core::ffi::c_void;
pub type MediaTransportControls = *mut ::core::ffi::c_void;
pub type MediaTransportControlsHelper = *mut ::core::ffi::c_void;
pub type MenuBar = *mut ::core::ffi::c_void;
pub type MenuBarItem = *mut ::core::ffi::c_void;
pub type MenuBarItemFlyout = *mut ::core::ffi::c_void;
pub type MenuFlyout = *mut ::core::ffi::c_void;
pub type MenuFlyoutItem = *mut ::core::ffi::c_void;
pub type MenuFlyoutItemBase = *mut ::core::ffi::c_void;
pub type MenuFlyoutPresenter = *mut ::core::ffi::c_void;
pub type MenuFlyoutSeparator = *mut ::core::ffi::c_void;
pub type MenuFlyoutSubItem = *mut ::core::ffi::c_void;
pub type NavigationView = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct NavigationViewBackButtonVisible(pub i32);
impl NavigationViewBackButtonVisible {
    pub const Collapsed: Self = Self(0i32);
    pub const Visible: Self = Self(1i32);
    pub const Auto: Self = Self(2i32);
}
impl ::core::marker::Copy for NavigationViewBackButtonVisible {}
impl ::core::clone::Clone for NavigationViewBackButtonVisible {
    fn clone(&self) -> Self {
        *self
    }
}
pub type NavigationViewBackRequestedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct NavigationViewDisplayMode(pub i32);
impl NavigationViewDisplayMode {
    pub const Minimal: Self = Self(0i32);
    pub const Compact: Self = Self(1i32);
    pub const Expanded: Self = Self(2i32);
}
impl ::core::marker::Copy for NavigationViewDisplayMode {}
impl ::core::clone::Clone for NavigationViewDisplayMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type NavigationViewDisplayModeChangedEventArgs = *mut ::core::ffi::c_void;
pub type NavigationViewItem = *mut ::core::ffi::c_void;
pub type NavigationViewItemBase = *mut ::core::ffi::c_void;
pub type NavigationViewItemHeader = *mut ::core::ffi::c_void;
pub type NavigationViewItemInvokedEventArgs = *mut ::core::ffi::c_void;
pub type NavigationViewItemSeparator = *mut ::core::ffi::c_void;
pub type NavigationViewList = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct NavigationViewOverflowLabelMode(pub i32);
impl NavigationViewOverflowLabelMode {
    pub const MoreLabel: Self = Self(0i32);
    pub const NoLabel: Self = Self(1i32);
}
impl ::core::marker::Copy for NavigationViewOverflowLabelMode {}
impl ::core::clone::Clone for NavigationViewOverflowLabelMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type NavigationViewPaneClosingEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct NavigationViewPaneDisplayMode(pub i32);
impl NavigationViewPaneDisplayMode {
    pub const Auto: Self = Self(0i32);
    pub const Left: Self = Self(1i32);
    pub const Top: Self = Self(2i32);
    pub const LeftCompact: Self = Self(3i32);
    pub const LeftMinimal: Self = Self(4i32);
}
impl ::core::marker::Copy for NavigationViewPaneDisplayMode {}
impl ::core::clone::Clone for NavigationViewPaneDisplayMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type NavigationViewSelectionChangedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct NavigationViewSelectionFollowsFocus(pub i32);
impl NavigationViewSelectionFollowsFocus {
    pub const Disabled: Self = Self(0i32);
    pub const Enabled: Self = Self(1i32);
}
impl ::core::marker::Copy for NavigationViewSelectionFollowsFocus {}
impl ::core::clone::Clone for NavigationViewSelectionFollowsFocus {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct NavigationViewShoulderNavigationEnabled(pub i32);
impl NavigationViewShoulderNavigationEnabled {
    pub const WhenSelectionFollowsFocus: Self = Self(0i32);
    pub const Always: Self = Self(1i32);
    pub const Never: Self = Self(2i32);
}
impl ::core::marker::Copy for NavigationViewShoulderNavigationEnabled {}
impl ::core::clone::Clone for NavigationViewShoulderNavigationEnabled {
    fn clone(&self) -> Self {
        *self
    }
}
pub type NavigationViewTemplateSettings = *mut ::core::ffi::c_void;
pub type NotifyEventArgs = *mut ::core::ffi::c_void;
pub type NotifyEventHandler = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct Orientation(pub i32);
impl Orientation {
    pub const Vertical: Self = Self(0i32);
    pub const Horizontal: Self = Self(1i32);
}
impl ::core::marker::Copy for Orientation {}
impl ::core::clone::Clone for Orientation {
    fn clone(&self) -> Self {
        *self
    }
}
pub type Page = *mut ::core::ffi::c_void;
pub type Panel = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct PanelScrollingDirection(pub i32);
impl PanelScrollingDirection {
    pub const None: Self = Self(0i32);
    pub const Forward: Self = Self(1i32);
    pub const Backward: Self = Self(2i32);
}
impl ::core::marker::Copy for PanelScrollingDirection {}
impl ::core::clone::Clone for PanelScrollingDirection {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct ParallaxSourceOffsetKind(pub i32);
impl ParallaxSourceOffsetKind {
    pub const Absolute: Self = Self(0i32);
    pub const Relative: Self = Self(1i32);
}
impl ::core::marker::Copy for ParallaxSourceOffsetKind {}
impl ::core::clone::Clone for ParallaxSourceOffsetKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ParallaxView = *mut ::core::ffi::c_void;
pub type PasswordBox = *mut ::core::ffi::c_void;
pub type PasswordBoxPasswordChangingEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct PasswordRevealMode(pub i32);
impl PasswordRevealMode {
    pub const Peek: Self = Self(0i32);
    pub const Hidden: Self = Self(1i32);
    pub const Visible: Self = Self(2i32);
}
impl ::core::marker::Copy for PasswordRevealMode {}
impl ::core::clone::Clone for PasswordRevealMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PathIcon = *mut ::core::ffi::c_void;
pub type PathIconSource = *mut ::core::ffi::c_void;
pub type PersonPicture = *mut ::core::ffi::c_void;
pub type PickerConfirmedEventArgs = *mut ::core::ffi::c_void;
pub type PickerFlyout = *mut ::core::ffi::c_void;
pub type PickerFlyoutPresenter = *mut ::core::ffi::c_void;
pub type Pivot = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct PivotHeaderFocusVisualPlacement(pub i32);
impl PivotHeaderFocusVisualPlacement {
    pub const ItemHeaders: Self = Self(0i32);
    pub const SelectedItemHeader: Self = Self(1i32);
}
impl ::core::marker::Copy for PivotHeaderFocusVisualPlacement {}
impl ::core::clone::Clone for PivotHeaderFocusVisualPlacement {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PivotItem = *mut ::core::ffi::c_void;
pub type PivotItemEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct PivotSlideInAnimationGroup(pub i32);
impl PivotSlideInAnimationGroup {
    pub const Default: Self = Self(0i32);
    pub const GroupOne: Self = Self(1i32);
    pub const GroupTwo: Self = Self(2i32);
    pub const GroupThree: Self = Self(3i32);
}
impl ::core::marker::Copy for PivotSlideInAnimationGroup {}
impl ::core::clone::Clone for PivotSlideInAnimationGroup {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ProgressBar = *mut ::core::ffi::c_void;
pub type ProgressRing = *mut ::core::ffi::c_void;
pub type RadioButton = *mut ::core::ffi::c_void;
pub type RatingControl = *mut ::core::ffi::c_void;
pub type RatingItemFontInfo = *mut ::core::ffi::c_void;
pub type RatingItemImageInfo = *mut ::core::ffi::c_void;
pub type RatingItemInfo = *mut ::core::ffi::c_void;
pub type RefreshContainer = *mut ::core::ffi::c_void;
pub type RefreshInteractionRatioChangedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct RefreshPullDirection(pub i32);
impl RefreshPullDirection {
    pub const LeftToRight: Self = Self(0i32);
    pub const TopToBottom: Self = Self(1i32);
    pub const RightToLeft: Self = Self(2i32);
    pub const BottomToTop: Self = Self(3i32);
}
impl ::core::marker::Copy for RefreshPullDirection {}
impl ::core::clone::Clone for RefreshPullDirection {
    fn clone(&self) -> Self {
        *self
    }
}
pub type RefreshRequestedEventArgs = *mut ::core::ffi::c_void;
pub type RefreshStateChangedEventArgs = *mut ::core::ffi::c_void;
pub type RefreshVisualizer = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct RefreshVisualizerOrientation(pub i32);
impl RefreshVisualizerOrientation {
    pub const Auto: Self = Self(0i32);
    pub const Normal: Self = Self(1i32);
    pub const Rotate90DegreesCounterclockwise: Self = Self(2i32);
    pub const Rotate270DegreesCounterclockwise: Self = Self(3i32);
}
impl ::core::marker::Copy for RefreshVisualizerOrientation {}
impl ::core::clone::Clone for RefreshVisualizerOrientation {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct RefreshVisualizerState(pub i32);
impl RefreshVisualizerState {
    pub const Idle: Self = Self(0i32);
    pub const Peeking: Self = Self(1i32);
    pub const Interacting: Self = Self(2i32);
    pub const Pending: Self = Self(3i32);
    pub const Refreshing: Self = Self(4i32);
}
impl ::core::marker::Copy for RefreshVisualizerState {}
impl ::core::clone::Clone for RefreshVisualizerState {
    fn clone(&self) -> Self {
        *self
    }
}
pub type RelativePanel = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct RequiresPointer(pub i32);
impl RequiresPointer {
    pub const Never: Self = Self(0i32);
    pub const WhenEngaged: Self = Self(1i32);
    pub const WhenFocused: Self = Self(2i32);
}
impl ::core::marker::Copy for RequiresPointer {}
impl ::core::clone::Clone for RequiresPointer {
    fn clone(&self) -> Self {
        *self
    }
}
pub type RichEditBox = *mut ::core::ffi::c_void;
pub type RichEditBoxSelectionChangingEventArgs = *mut ::core::ffi::c_void;
pub type RichEditBoxTextChangingEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct RichEditClipboardFormat(pub i32);
impl RichEditClipboardFormat {
    pub const AllFormats: Self = Self(0i32);
    pub const PlainText: Self = Self(1i32);
}
impl ::core::marker::Copy for RichEditClipboardFormat {}
impl ::core::clone::Clone for RichEditClipboardFormat {
    fn clone(&self) -> Self {
        *self
    }
}
pub type RichTextBlock = *mut ::core::ffi::c_void;
pub type RichTextBlockOverflow = *mut ::core::ffi::c_void;
pub type RowDefinition = *mut ::core::ffi::c_void;
pub type RowDefinitionCollection = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct ScrollBarVisibility(pub i32);
impl ScrollBarVisibility {
    pub const Disabled: Self = Self(0i32);
    pub const Auto: Self = Self(1i32);
    pub const Hidden: Self = Self(2i32);
    pub const Visible: Self = Self(3i32);
}
impl ::core::marker::Copy for ScrollBarVisibility {}
impl ::core::clone::Clone for ScrollBarVisibility {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ScrollContentPresenter = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct ScrollIntoViewAlignment(pub i32);
impl ScrollIntoViewAlignment {
    pub const Default: Self = Self(0i32);
    pub const Leading: Self = Self(1i32);
}
impl ::core::marker::Copy for ScrollIntoViewAlignment {}
impl ::core::clone::Clone for ScrollIntoViewAlignment {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct ScrollMode(pub i32);
impl ScrollMode {
    pub const Disabled: Self = Self(0i32);
    pub const Enabled: Self = Self(1i32);
    pub const Auto: Self = Self(2i32);
}
impl ::core::marker::Copy for ScrollMode {}
impl ::core::clone::Clone for ScrollMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ScrollViewer = *mut ::core::ffi::c_void;
pub type ScrollViewerView = *mut ::core::ffi::c_void;
pub type ScrollViewerViewChangedEventArgs = *mut ::core::ffi::c_void;
pub type ScrollViewerViewChangingEventArgs = *mut ::core::ffi::c_void;
pub type SearchBox = *mut ::core::ffi::c_void;
pub type SearchBoxQueryChangedEventArgs = *mut ::core::ffi::c_void;
pub type SearchBoxQuerySubmittedEventArgs = *mut ::core::ffi::c_void;
pub type SearchBoxResultSuggestionChosenEventArgs = *mut ::core::ffi::c_void;
pub type SearchBoxSuggestionsRequestedEventArgs = *mut ::core::ffi::c_void;
pub type SectionsInViewChangedEventArgs = *mut ::core::ffi::c_void;
pub type SectionsInViewChangedEventHandler = *mut ::core::ffi::c_void;
pub type SelectionChangedEventArgs = *mut ::core::ffi::c_void;
pub type SelectionChangedEventHandler = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct SelectionMode(pub i32);
impl SelectionMode {
    pub const Single: Self = Self(0i32);
    pub const Multiple: Self = Self(1i32);
    pub const Extended: Self = Self(2i32);
}
impl ::core::marker::Copy for SelectionMode {}
impl ::core::clone::Clone for SelectionMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SemanticZoom = *mut ::core::ffi::c_void;
pub type SemanticZoomLocation = *mut ::core::ffi::c_void;
pub type SemanticZoomViewChangedEventArgs = *mut ::core::ffi::c_void;
pub type SemanticZoomViewChangedEventHandler = *mut ::core::ffi::c_void;
pub type SettingsFlyout = *mut ::core::ffi::c_void;
pub type Slider = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct SnapPointsType(pub i32);
impl SnapPointsType {
    pub const None: Self = Self(0i32);
    pub const Optional: Self = Self(1i32);
    pub const Mandatory: Self = Self(2i32);
    pub const OptionalSingle: Self = Self(3i32);
    pub const MandatorySingle: Self = Self(4i32);
}
impl ::core::marker::Copy for SnapPointsType {}
impl ::core::clone::Clone for SnapPointsType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SplitButton = *mut ::core::ffi::c_void;
pub type SplitButtonAutomationPeer = *mut ::core::ffi::c_void;
pub type SplitButtonClickEventArgs = *mut ::core::ffi::c_void;
pub type SplitView = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct SplitViewDisplayMode(pub i32);
impl SplitViewDisplayMode {
    pub const Overlay: Self = Self(0i32);
    pub const Inline: Self = Self(1i32);
    pub const CompactOverlay: Self = Self(2i32);
    pub const CompactInline: Self = Self(3i32);
}
impl ::core::marker::Copy for SplitViewDisplayMode {}
impl ::core::clone::Clone for SplitViewDisplayMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SplitViewPaneClosingEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct SplitViewPanePlacement(pub i32);
impl SplitViewPanePlacement {
    pub const Left: Self = Self(0i32);
    pub const Right: Self = Self(1i32);
}
impl ::core::marker::Copy for SplitViewPanePlacement {}
impl ::core::clone::Clone for SplitViewPanePlacement {
    fn clone(&self) -> Self {
        *self
    }
}
pub type StackPanel = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct StretchDirection(pub i32);
impl StretchDirection {
    pub const UpOnly: Self = Self(0i32);
    pub const DownOnly: Self = Self(1i32);
    pub const Both: Self = Self(2i32);
}
impl ::core::marker::Copy for StretchDirection {}
impl ::core::clone::Clone for StretchDirection {
    fn clone(&self) -> Self {
        *self
    }
}
pub type StyleSelector = *mut ::core::ffi::c_void;
pub type SwapChainBackgroundPanel = *mut ::core::ffi::c_void;
pub type SwapChainPanel = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct SwipeBehaviorOnInvoked(pub i32);
impl SwipeBehaviorOnInvoked {
    pub const Auto: Self = Self(0i32);
    pub const Close: Self = Self(1i32);
    pub const RemainOpen: Self = Self(2i32);
}
impl ::core::marker::Copy for SwipeBehaviorOnInvoked {}
impl ::core::clone::Clone for SwipeBehaviorOnInvoked {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SwipeControl = *mut ::core::ffi::c_void;
pub type SwipeItem = *mut ::core::ffi::c_void;
pub type SwipeItemInvokedEventArgs = *mut ::core::ffi::c_void;
pub type SwipeItems = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct SwipeMode(pub i32);
impl SwipeMode {
    pub const Reveal: Self = Self(0i32);
    pub const Execute: Self = Self(1i32);
}
impl ::core::marker::Copy for SwipeMode {}
impl ::core::clone::Clone for SwipeMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct Symbol(pub i32);
impl Symbol {
    pub const Previous: Self = Self(57600i32);
    pub const Next: Self = Self(57601i32);
    pub const Play: Self = Self(57602i32);
    pub const Pause: Self = Self(57603i32);
    pub const Edit: Self = Self(57604i32);
    pub const Save: Self = Self(57605i32);
    pub const Clear: Self = Self(57606i32);
    pub const Delete: Self = Self(57607i32);
    pub const Remove: Self = Self(57608i32);
    pub const Add: Self = Self(57609i32);
    pub const Cancel: Self = Self(57610i32);
    pub const Accept: Self = Self(57611i32);
    pub const More: Self = Self(57612i32);
    pub const Redo: Self = Self(57613i32);
    pub const Undo: Self = Self(57614i32);
    pub const Home: Self = Self(57615i32);
    pub const Up: Self = Self(57616i32);
    pub const Forward: Self = Self(57617i32);
    pub const Back: Self = Self(57618i32);
    pub const Favorite: Self = Self(57619i32);
    pub const Camera: Self = Self(57620i32);
    pub const Setting: Self = Self(57621i32);
    pub const Video: Self = Self(57622i32);
    pub const Sync: Self = Self(57623i32);
    pub const Download: Self = Self(57624i32);
    pub const Mail: Self = Self(57625i32);
    pub const Find: Self = Self(57626i32);
    pub const Help: Self = Self(57627i32);
    pub const Upload: Self = Self(57628i32);
    pub const Emoji: Self = Self(57629i32);
    pub const TwoPage: Self = Self(57630i32);
    pub const LeaveChat: Self = Self(57631i32);
    pub const MailForward: Self = Self(57632i32);
    pub const Clock: Self = Self(57633i32);
    pub const Send: Self = Self(57634i32);
    pub const Crop: Self = Self(57635i32);
    pub const RotateCamera: Self = Self(57636i32);
    pub const People: Self = Self(57637i32);
    pub const OpenPane: Self = Self(57638i32);
    pub const ClosePane: Self = Self(57639i32);
    pub const World: Self = Self(57640i32);
    pub const Flag: Self = Self(57641i32);
    pub const PreviewLink: Self = Self(57642i32);
    pub const Globe: Self = Self(57643i32);
    pub const Trim: Self = Self(57644i32);
    pub const AttachCamera: Self = Self(57645i32);
    pub const ZoomIn: Self = Self(57646i32);
    pub const Bookmarks: Self = Self(57647i32);
    pub const Document: Self = Self(57648i32);
    pub const ProtectedDocument: Self = Self(57649i32);
    pub const Page: Self = Self(57650i32);
    pub const Bullets: Self = Self(57651i32);
    pub const Comment: Self = Self(57652i32);
    pub const MailFilled: Self = Self(57653i32);
    pub const ContactInfo: Self = Self(57654i32);
    pub const HangUp: Self = Self(57655i32);
    pub const ViewAll: Self = Self(57656i32);
    pub const MapPin: Self = Self(57657i32);
    pub const Phone: Self = Self(57658i32);
    pub const VideoChat: Self = Self(57659i32);
    pub const Switch: Self = Self(57660i32);
    pub const Contact: Self = Self(57661i32);
    pub const Rename: Self = Self(57662i32);
    pub const Pin: Self = Self(57665i32);
    pub const MusicInfo: Self = Self(57666i32);
    pub const Go: Self = Self(57667i32);
    pub const Keyboard: Self = Self(57668i32);
    pub const DockLeft: Self = Self(57669i32);
    pub const DockRight: Self = Self(57670i32);
    pub const DockBottom: Self = Self(57671i32);
    pub const Remote: Self = Self(57672i32);
    pub const Refresh: Self = Self(57673i32);
    pub const Rotate: Self = Self(57674i32);
    pub const Shuffle: Self = Self(57675i32);
    pub const List: Self = Self(57676i32);
    pub const Shop: Self = Self(57677i32);
    pub const SelectAll: Self = Self(57678i32);
    pub const Orientation: Self = Self(57679i32);
    pub const Import: Self = Self(57680i32);
    pub const ImportAll: Self = Self(57681i32);
    pub const BrowsePhotos: Self = Self(57685i32);
    pub const WebCam: Self = Self(57686i32);
    pub const Pictures: Self = Self(57688i32);
    pub const SaveLocal: Self = Self(57689i32);
    pub const Caption: Self = Self(57690i32);
    pub const Stop: Self = Self(57691i32);
    pub const ShowResults: Self = Self(57692i32);
    pub const Volume: Self = Self(57693i32);
    pub const Repair: Self = Self(57694i32);
    pub const Message: Self = Self(57695i32);
    pub const Page2: Self = Self(57696i32);
    pub const CalendarDay: Self = Self(57697i32);
    pub const CalendarWeek: Self = Self(57698i32);
    pub const Calendar: Self = Self(57699i32);
    pub const Character: Self = Self(57700i32);
    pub const MailReplyAll: Self = Self(57701i32);
    pub const Read: Self = Self(57702i32);
    pub const Link: Self = Self(57703i32);
    pub const Account: Self = Self(57704i32);
    pub const ShowBcc: Self = Self(57705i32);
    pub const HideBcc: Self = Self(57706i32);
    pub const Cut: Self = Self(57707i32);
    pub const Attach: Self = Self(57708i32);
    pub const Paste: Self = Self(57709i32);
    pub const Filter: Self = Self(57710i32);
    pub const Copy: Self = Self(57711i32);
    pub const Emoji2: Self = Self(57712i32);
    pub const Important: Self = Self(57713i32);
    pub const MailReply: Self = Self(57714i32);
    pub const SlideShow: Self = Self(57715i32);
    pub const Sort: Self = Self(57716i32);
    pub const Manage: Self = Self(57720i32);
    pub const AllApps: Self = Self(57721i32);
    pub const DisconnectDrive: Self = Self(57722i32);
    pub const MapDrive: Self = Self(57723i32);
    pub const NewWindow: Self = Self(57724i32);
    pub const OpenWith: Self = Self(57725i32);
    pub const ContactPresence: Self = Self(57729i32);
    pub const Priority: Self = Self(57730i32);
    pub const GoToToday: Self = Self(57732i32);
    pub const Font: Self = Self(57733i32);
    pub const FontColor: Self = Self(57734i32);
    pub const Contact2: Self = Self(57735i32);
    pub const Folder: Self = Self(57736i32);
    pub const Audio: Self = Self(57737i32);
    pub const Placeholder: Self = Self(57738i32);
    pub const View: Self = Self(57739i32);
    pub const SetLockScreen: Self = Self(57740i32);
    pub const SetTile: Self = Self(57741i32);
    pub const ClosedCaption: Self = Self(57744i32);
    pub const StopSlideShow: Self = Self(57745i32);
    pub const Permissions: Self = Self(57746i32);
    pub const Highlight: Self = Self(57747i32);
    pub const DisableUpdates: Self = Self(57748i32);
    pub const UnFavorite: Self = Self(57749i32);
    pub const UnPin: Self = Self(57750i32);
    pub const OpenLocal: Self = Self(57751i32);
    pub const Mute: Self = Self(57752i32);
    pub const Italic: Self = Self(57753i32);
    pub const Underline: Self = Self(57754i32);
    pub const Bold: Self = Self(57755i32);
    pub const MoveToFolder: Self = Self(57756i32);
    pub const LikeDislike: Self = Self(57757i32);
    pub const Dislike: Self = Self(57758i32);
    pub const Like: Self = Self(57759i32);
    pub const AlignRight: Self = Self(57760i32);
    pub const AlignCenter: Self = Self(57761i32);
    pub const AlignLeft: Self = Self(57762i32);
    pub const Zoom: Self = Self(57763i32);
    pub const ZoomOut: Self = Self(57764i32);
    pub const OpenFile: Self = Self(57765i32);
    pub const OtherUser: Self = Self(57766i32);
    pub const Admin: Self = Self(57767i32);
    pub const Street: Self = Self(57795i32);
    pub const Map: Self = Self(57796i32);
    pub const ClearSelection: Self = Self(57797i32);
    pub const FontDecrease: Self = Self(57798i32);
    pub const FontIncrease: Self = Self(57799i32);
    pub const FontSize: Self = Self(57800i32);
    pub const CellPhone: Self = Self(57801i32);
    pub const ReShare: Self = Self(57802i32);
    pub const Tag: Self = Self(57803i32);
    pub const RepeatOne: Self = Self(57804i32);
    pub const RepeatAll: Self = Self(57805i32);
    pub const OutlineStar: Self = Self(57806i32);
    pub const SolidStar: Self = Self(57807i32);
    pub const Calculator: Self = Self(57808i32);
    pub const Directions: Self = Self(57809i32);
    pub const Target: Self = Self(57810i32);
    pub const Library: Self = Self(57811i32);
    pub const PhoneBook: Self = Self(57812i32);
    pub const Memo: Self = Self(57813i32);
    pub const Microphone: Self = Self(57814i32);
    pub const PostUpdate: Self = Self(57815i32);
    pub const BackToWindow: Self = Self(57816i32);
    pub const FullScreen: Self = Self(57817i32);
    pub const NewFolder: Self = Self(57818i32);
    pub const CalendarReply: Self = Self(57819i32);
    pub const UnSyncFolder: Self = Self(57821i32);
    pub const ReportHacked: Self = Self(57822i32);
    pub const SyncFolder: Self = Self(57823i32);
    pub const BlockContact: Self = Self(57824i32);
    pub const SwitchApps: Self = Self(57825i32);
    pub const AddFriend: Self = Self(57826i32);
    pub const TouchPointer: Self = Self(57827i32);
    pub const GoToStart: Self = Self(57828i32);
    pub const ZeroBars: Self = Self(57829i32);
    pub const OneBar: Self = Self(57830i32);
    pub const TwoBars: Self = Self(57831i32);
    pub const ThreeBars: Self = Self(57832i32);
    pub const FourBars: Self = Self(57833i32);
    pub const Scan: Self = Self(58004i32);
    pub const Preview: Self = Self(58005i32);
    pub const GlobalNavigationButton: Self = Self(59136i32);
    pub const Share: Self = Self(59181i32);
    pub const Print: Self = Self(59209i32);
    pub const XboxOneConsole: Self = Self(59792i32);
}
impl ::core::marker::Copy for Symbol {}
impl ::core::clone::Clone for Symbol {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SymbolIcon = *mut ::core::ffi::c_void;
pub type SymbolIconSource = *mut ::core::ffi::c_void;
pub type TextBlock = *mut ::core::ffi::c_void;
pub type TextBox = *mut ::core::ffi::c_void;
pub type TextBoxBeforeTextChangingEventArgs = *mut ::core::ffi::c_void;
pub type TextBoxSelectionChangingEventArgs = *mut ::core::ffi::c_void;
pub type TextBoxTextChangingEventArgs = *mut ::core::ffi::c_void;
pub type TextChangedEventArgs = *mut ::core::ffi::c_void;
pub type TextChangedEventHandler = *mut ::core::ffi::c_void;
pub type TextCommandBarFlyout = *mut ::core::ffi::c_void;
pub type TextCompositionChangedEventArgs = *mut ::core::ffi::c_void;
pub type TextCompositionEndedEventArgs = *mut ::core::ffi::c_void;
pub type TextCompositionStartedEventArgs = *mut ::core::ffi::c_void;
pub type TextControlCopyingToClipboardEventArgs = *mut ::core::ffi::c_void;
pub type TextControlCuttingToClipboardEventArgs = *mut ::core::ffi::c_void;
pub type TextControlPasteEventArgs = *mut ::core::ffi::c_void;
pub type TextControlPasteEventHandler = *mut ::core::ffi::c_void;
pub type TimePickedEventArgs = *mut ::core::ffi::c_void;
pub type TimePicker = *mut ::core::ffi::c_void;
pub type TimePickerFlyout = *mut ::core::ffi::c_void;
pub type TimePickerFlyoutPresenter = *mut ::core::ffi::c_void;
pub type TimePickerSelectedValueChangedEventArgs = *mut ::core::ffi::c_void;
pub type TimePickerValueChangedEventArgs = *mut ::core::ffi::c_void;
pub type ToggleMenuFlyoutItem = *mut ::core::ffi::c_void;
pub type ToggleSplitButton = *mut ::core::ffi::c_void;
pub type ToggleSplitButtonAutomationPeer = *mut ::core::ffi::c_void;
pub type ToggleSplitButtonIsCheckedChangedEventArgs = *mut ::core::ffi::c_void;
pub type ToggleSwitch = *mut ::core::ffi::c_void;
pub type ToolTip = *mut ::core::ffi::c_void;
pub type ToolTipService = *mut ::core::ffi::c_void;
pub type TreeView = *mut ::core::ffi::c_void;
pub type TreeViewCollapsedEventArgs = *mut ::core::ffi::c_void;
pub type TreeViewDragItemsCompletedEventArgs = *mut ::core::ffi::c_void;
pub type TreeViewDragItemsStartingEventArgs = *mut ::core::ffi::c_void;
pub type TreeViewExpandingEventArgs = *mut ::core::ffi::c_void;
pub type TreeViewItem = *mut ::core::ffi::c_void;
pub type TreeViewItemInvokedEventArgs = *mut ::core::ffi::c_void;
pub type TreeViewItemTemplateSettings = *mut ::core::ffi::c_void;
pub type TreeViewList = *mut ::core::ffi::c_void;
pub type TreeViewNode = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct TreeViewSelectionMode(pub i32);
impl TreeViewSelectionMode {
    pub const None: Self = Self(0i32);
    pub const Single: Self = Self(1i32);
    pub const Multiple: Self = Self(2i32);
}
impl ::core::marker::Copy for TreeViewSelectionMode {}
impl ::core::clone::Clone for TreeViewSelectionMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type TwoPaneView = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct TwoPaneViewMode(pub i32);
impl TwoPaneViewMode {
    pub const SinglePane: Self = Self(0i32);
    pub const Wide: Self = Self(1i32);
    pub const Tall: Self = Self(2i32);
}
impl ::core::marker::Copy for TwoPaneViewMode {}
impl ::core::clone::Clone for TwoPaneViewMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct TwoPaneViewPriority(pub i32);
impl TwoPaneViewPriority {
    pub const Pane1: Self = Self(0i32);
    pub const Pane2: Self = Self(1i32);
}
impl ::core::marker::Copy for TwoPaneViewPriority {}
impl ::core::clone::Clone for TwoPaneViewPriority {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct TwoPaneViewTallModeConfiguration(pub i32);
impl TwoPaneViewTallModeConfiguration {
    pub const SinglePane: Self = Self(0i32);
    pub const TopBottom: Self = Self(1i32);
    pub const BottomTop: Self = Self(2i32);
}
impl ::core::marker::Copy for TwoPaneViewTallModeConfiguration {}
impl ::core::clone::Clone for TwoPaneViewTallModeConfiguration {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct TwoPaneViewWideModeConfiguration(pub i32);
impl TwoPaneViewWideModeConfiguration {
    pub const SinglePane: Self = Self(0i32);
    pub const LeftRight: Self = Self(1i32);
    pub const RightLeft: Self = Self(2i32);
}
impl ::core::marker::Copy for TwoPaneViewWideModeConfiguration {}
impl ::core::clone::Clone for TwoPaneViewWideModeConfiguration {
    fn clone(&self) -> Self {
        *self
    }
}
pub type UIElementCollection = *mut ::core::ffi::c_void;
pub type UserControl = *mut ::core::ffi::c_void;
pub type VariableSizedWrapGrid = *mut ::core::ffi::c_void;
pub type Viewbox = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct VirtualizationMode(pub i32);
impl VirtualizationMode {
    pub const Standard: Self = Self(0i32);
    pub const Recycling: Self = Self(1i32);
}
impl ::core::marker::Copy for VirtualizationMode {}
impl ::core::clone::Clone for VirtualizationMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type VirtualizingPanel = *mut ::core::ffi::c_void;
pub type VirtualizingStackPanel = *mut ::core::ffi::c_void;
pub type WebView = *mut ::core::ffi::c_void;
pub type WebViewBrush = *mut ::core::ffi::c_void;
pub type WebViewContentLoadingEventArgs = *mut ::core::ffi::c_void;
pub type WebViewDOMContentLoadedEventArgs = *mut ::core::ffi::c_void;
pub type WebViewDeferredPermissionRequest = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct WebViewExecutionMode(pub i32);
impl WebViewExecutionMode {
    pub const SameThread: Self = Self(0i32);
    pub const SeparateThread: Self = Self(1i32);
    pub const SeparateProcess: Self = Self(2i32);
}
impl ::core::marker::Copy for WebViewExecutionMode {}
impl ::core::clone::Clone for WebViewExecutionMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type WebViewLongRunningScriptDetectedEventArgs = *mut ::core::ffi::c_void;
pub type WebViewNavigationCompletedEventArgs = *mut ::core::ffi::c_void;
pub type WebViewNavigationFailedEventArgs = *mut ::core::ffi::c_void;
pub type WebViewNavigationFailedEventHandler = *mut ::core::ffi::c_void;
pub type WebViewNavigationStartingEventArgs = *mut ::core::ffi::c_void;
pub type WebViewNewWindowRequestedEventArgs = *mut ::core::ffi::c_void;
pub type WebViewPermissionRequest = *mut ::core::ffi::c_void;
pub type WebViewPermissionRequestedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct WebViewPermissionState(pub i32);
impl WebViewPermissionState {
    pub const Unknown: Self = Self(0i32);
    pub const Defer: Self = Self(1i32);
    pub const Allow: Self = Self(2i32);
    pub const Deny: Self = Self(3i32);
}
impl ::core::marker::Copy for WebViewPermissionState {}
impl ::core::clone::Clone for WebViewPermissionState {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct WebViewPermissionType(pub i32);
impl WebViewPermissionType {
    pub const Geolocation: Self = Self(0i32);
    pub const UnlimitedIndexedDBQuota: Self = Self(1i32);
    pub const Media: Self = Self(2i32);
    pub const PointerLock: Self = Self(3i32);
    pub const WebNotifications: Self = Self(4i32);
    pub const Screen: Self = Self(5i32);
    pub const ImmersiveView: Self = Self(6i32);
}
impl ::core::marker::Copy for WebViewPermissionType {}
impl ::core::clone::Clone for WebViewPermissionType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type WebViewSeparateProcessLostEventArgs = *mut ::core::ffi::c_void;
pub type WebViewSettings = *mut ::core::ffi::c_void;
pub type WebViewUnsupportedUriSchemeIdentifiedEventArgs = *mut ::core::ffi::c_void;
pub type WebViewUnviewableContentIdentifiedEventArgs = *mut ::core::ffi::c_void;
pub type WebViewWebResourceRequestedEventArgs = *mut ::core::ffi::c_void;
pub type WrapGrid = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Controls\"`*"]
#[repr(transparent)]
pub struct ZoomMode(pub i32);
impl ZoomMode {
    pub const Disabled: Self = Self(0i32);
    pub const Enabled: Self = Self(1i32);
}
impl ::core::marker::Copy for ZoomMode {}
impl ::core::clone::Clone for ZoomMode {
    fn clone(&self) -> Self {
        *self
    }
}

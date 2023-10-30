#[doc = "*Required features: `\"UI_Xaml_Automation_Peers\"`*"]
#[repr(transparent)]
pub struct AccessibilityView(pub i32);
impl AccessibilityView {
    pub const Raw: Self = Self(0i32);
    pub const Control: Self = Self(1i32);
    pub const Content: Self = Self(2i32);
}
impl ::core::marker::Copy for AccessibilityView {}
impl ::core::clone::Clone for AccessibilityView {
    fn clone(&self) -> Self {
        *self
    }
}
pub type AppBarAutomationPeer = *mut ::core::ffi::c_void;
pub type AppBarButtonAutomationPeer = *mut ::core::ffi::c_void;
pub type AppBarToggleButtonAutomationPeer = *mut ::core::ffi::c_void;
pub type AutoSuggestBoxAutomationPeer = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Automation_Peers\"`*"]
#[repr(transparent)]
pub struct AutomationControlType(pub i32);
impl AutomationControlType {
    pub const Button: Self = Self(0i32);
    pub const Calendar: Self = Self(1i32);
    pub const CheckBox: Self = Self(2i32);
    pub const ComboBox: Self = Self(3i32);
    pub const Edit: Self = Self(4i32);
    pub const Hyperlink: Self = Self(5i32);
    pub const Image: Self = Self(6i32);
    pub const ListItem: Self = Self(7i32);
    pub const List: Self = Self(8i32);
    pub const Menu: Self = Self(9i32);
    pub const MenuBar: Self = Self(10i32);
    pub const MenuItem: Self = Self(11i32);
    pub const ProgressBar: Self = Self(12i32);
    pub const RadioButton: Self = Self(13i32);
    pub const ScrollBar: Self = Self(14i32);
    pub const Slider: Self = Self(15i32);
    pub const Spinner: Self = Self(16i32);
    pub const StatusBar: Self = Self(17i32);
    pub const Tab: Self = Self(18i32);
    pub const TabItem: Self = Self(19i32);
    pub const Text: Self = Self(20i32);
    pub const ToolBar: Self = Self(21i32);
    pub const ToolTip: Self = Self(22i32);
    pub const Tree: Self = Self(23i32);
    pub const TreeItem: Self = Self(24i32);
    pub const Custom: Self = Self(25i32);
    pub const Group: Self = Self(26i32);
    pub const Thumb: Self = Self(27i32);
    pub const DataGrid: Self = Self(28i32);
    pub const DataItem: Self = Self(29i32);
    pub const Document: Self = Self(30i32);
    pub const SplitButton: Self = Self(31i32);
    pub const Window: Self = Self(32i32);
    pub const Pane: Self = Self(33i32);
    pub const Header: Self = Self(34i32);
    pub const HeaderItem: Self = Self(35i32);
    pub const Table: Self = Self(36i32);
    pub const TitleBar: Self = Self(37i32);
    pub const Separator: Self = Self(38i32);
    pub const SemanticZoom: Self = Self(39i32);
    pub const AppBar: Self = Self(40i32);
}
impl ::core::marker::Copy for AutomationControlType {}
impl ::core::clone::Clone for AutomationControlType {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Automation_Peers\"`*"]
#[repr(transparent)]
pub struct AutomationEvents(pub i32);
impl AutomationEvents {
    pub const ToolTipOpened: Self = Self(0i32);
    pub const ToolTipClosed: Self = Self(1i32);
    pub const MenuOpened: Self = Self(2i32);
    pub const MenuClosed: Self = Self(3i32);
    pub const AutomationFocusChanged: Self = Self(4i32);
    pub const InvokePatternOnInvoked: Self = Self(5i32);
    pub const SelectionItemPatternOnElementAddedToSelection: Self = Self(6i32);
    pub const SelectionItemPatternOnElementRemovedFromSelection: Self = Self(7i32);
    pub const SelectionItemPatternOnElementSelected: Self = Self(8i32);
    pub const SelectionPatternOnInvalidated: Self = Self(9i32);
    pub const TextPatternOnTextSelectionChanged: Self = Self(10i32);
    pub const TextPatternOnTextChanged: Self = Self(11i32);
    pub const AsyncContentLoaded: Self = Self(12i32);
    pub const PropertyChanged: Self = Self(13i32);
    pub const StructureChanged: Self = Self(14i32);
    pub const DragStart: Self = Self(15i32);
    pub const DragCancel: Self = Self(16i32);
    pub const DragComplete: Self = Self(17i32);
    pub const DragEnter: Self = Self(18i32);
    pub const DragLeave: Self = Self(19i32);
    pub const Dropped: Self = Self(20i32);
    pub const LiveRegionChanged: Self = Self(21i32);
    pub const InputReachedTarget: Self = Self(22i32);
    pub const InputReachedOtherElement: Self = Self(23i32);
    pub const InputDiscarded: Self = Self(24i32);
    pub const WindowClosed: Self = Self(25i32);
    pub const WindowOpened: Self = Self(26i32);
    pub const ConversionTargetChanged: Self = Self(27i32);
    pub const TextEditTextChanged: Self = Self(28i32);
    pub const LayoutInvalidated: Self = Self(29i32);
}
impl ::core::marker::Copy for AutomationEvents {}
impl ::core::clone::Clone for AutomationEvents {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Automation_Peers\"`*"]
#[repr(transparent)]
pub struct AutomationHeadingLevel(pub i32);
impl AutomationHeadingLevel {
    pub const None: Self = Self(0i32);
    pub const Level1: Self = Self(1i32);
    pub const Level2: Self = Self(2i32);
    pub const Level3: Self = Self(3i32);
    pub const Level4: Self = Self(4i32);
    pub const Level5: Self = Self(5i32);
    pub const Level6: Self = Self(6i32);
    pub const Level7: Self = Self(7i32);
    pub const Level8: Self = Self(8i32);
    pub const Level9: Self = Self(9i32);
}
impl ::core::marker::Copy for AutomationHeadingLevel {}
impl ::core::clone::Clone for AutomationHeadingLevel {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Automation_Peers\"`*"]
#[repr(transparent)]
pub struct AutomationLandmarkType(pub i32);
impl AutomationLandmarkType {
    pub const None: Self = Self(0i32);
    pub const Custom: Self = Self(1i32);
    pub const Form: Self = Self(2i32);
    pub const Main: Self = Self(3i32);
    pub const Navigation: Self = Self(4i32);
    pub const Search: Self = Self(5i32);
}
impl ::core::marker::Copy for AutomationLandmarkType {}
impl ::core::clone::Clone for AutomationLandmarkType {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Automation_Peers\"`*"]
#[repr(transparent)]
pub struct AutomationLiveSetting(pub i32);
impl AutomationLiveSetting {
    pub const Off: Self = Self(0i32);
    pub const Polite: Self = Self(1i32);
    pub const Assertive: Self = Self(2i32);
}
impl ::core::marker::Copy for AutomationLiveSetting {}
impl ::core::clone::Clone for AutomationLiveSetting {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Automation_Peers\"`*"]
#[repr(transparent)]
pub struct AutomationNavigationDirection(pub i32);
impl AutomationNavigationDirection {
    pub const Parent: Self = Self(0i32);
    pub const NextSibling: Self = Self(1i32);
    pub const PreviousSibling: Self = Self(2i32);
    pub const FirstChild: Self = Self(3i32);
    pub const LastChild: Self = Self(4i32);
}
impl ::core::marker::Copy for AutomationNavigationDirection {}
impl ::core::clone::Clone for AutomationNavigationDirection {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Automation_Peers\"`*"]
#[repr(transparent)]
pub struct AutomationNotificationKind(pub i32);
impl AutomationNotificationKind {
    pub const ItemAdded: Self = Self(0i32);
    pub const ItemRemoved: Self = Self(1i32);
    pub const ActionCompleted: Self = Self(2i32);
    pub const ActionAborted: Self = Self(3i32);
    pub const Other: Self = Self(4i32);
}
impl ::core::marker::Copy for AutomationNotificationKind {}
impl ::core::clone::Clone for AutomationNotificationKind {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Automation_Peers\"`*"]
#[repr(transparent)]
pub struct AutomationNotificationProcessing(pub i32);
impl AutomationNotificationProcessing {
    pub const ImportantAll: Self = Self(0i32);
    pub const ImportantMostRecent: Self = Self(1i32);
    pub const All: Self = Self(2i32);
    pub const MostRecent: Self = Self(3i32);
    pub const CurrentThenMostRecent: Self = Self(4i32);
}
impl ::core::marker::Copy for AutomationNotificationProcessing {}
impl ::core::clone::Clone for AutomationNotificationProcessing {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Automation_Peers\"`*"]
#[repr(transparent)]
pub struct AutomationOrientation(pub i32);
impl AutomationOrientation {
    pub const None: Self = Self(0i32);
    pub const Horizontal: Self = Self(1i32);
    pub const Vertical: Self = Self(2i32);
}
impl ::core::marker::Copy for AutomationOrientation {}
impl ::core::clone::Clone for AutomationOrientation {
    fn clone(&self) -> Self {
        *self
    }
}
pub type AutomationPeer = *mut ::core::ffi::c_void;
pub type AutomationPeerAnnotation = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Automation_Peers\"`*"]
#[repr(transparent)]
pub struct AutomationStructureChangeType(pub i32);
impl AutomationStructureChangeType {
    pub const ChildAdded: Self = Self(0i32);
    pub const ChildRemoved: Self = Self(1i32);
    pub const ChildrenInvalidated: Self = Self(2i32);
    pub const ChildrenBulkAdded: Self = Self(3i32);
    pub const ChildrenBulkRemoved: Self = Self(4i32);
    pub const ChildrenReordered: Self = Self(5i32);
}
impl ::core::marker::Copy for AutomationStructureChangeType {}
impl ::core::clone::Clone for AutomationStructureChangeType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ButtonAutomationPeer = *mut ::core::ffi::c_void;
pub type ButtonBaseAutomationPeer = *mut ::core::ffi::c_void;
pub type CalendarDatePickerAutomationPeer = *mut ::core::ffi::c_void;
pub type CaptureElementAutomationPeer = *mut ::core::ffi::c_void;
pub type CheckBoxAutomationPeer = *mut ::core::ffi::c_void;
pub type ColorPickerSliderAutomationPeer = *mut ::core::ffi::c_void;
pub type ColorSpectrumAutomationPeer = *mut ::core::ffi::c_void;
pub type ComboBoxAutomationPeer = *mut ::core::ffi::c_void;
pub type ComboBoxItemAutomationPeer = *mut ::core::ffi::c_void;
pub type ComboBoxItemDataAutomationPeer = *mut ::core::ffi::c_void;
pub type DatePickerAutomationPeer = *mut ::core::ffi::c_void;
pub type DatePickerFlyoutPresenterAutomationPeer = *mut ::core::ffi::c_void;
pub type FlipViewAutomationPeer = *mut ::core::ffi::c_void;
pub type FlipViewItemAutomationPeer = *mut ::core::ffi::c_void;
pub type FlipViewItemDataAutomationPeer = *mut ::core::ffi::c_void;
pub type FlyoutPresenterAutomationPeer = *mut ::core::ffi::c_void;
pub type FrameworkElementAutomationPeer = *mut ::core::ffi::c_void;
pub type GridViewAutomationPeer = *mut ::core::ffi::c_void;
pub type GridViewHeaderItemAutomationPeer = *mut ::core::ffi::c_void;
pub type GridViewItemAutomationPeer = *mut ::core::ffi::c_void;
pub type GridViewItemDataAutomationPeer = *mut ::core::ffi::c_void;
pub type GroupItemAutomationPeer = *mut ::core::ffi::c_void;
pub type HubAutomationPeer = *mut ::core::ffi::c_void;
pub type HubSectionAutomationPeer = *mut ::core::ffi::c_void;
pub type HyperlinkButtonAutomationPeer = *mut ::core::ffi::c_void;
pub type ImageAutomationPeer = *mut ::core::ffi::c_void;
pub type InkToolbarAutomationPeer = *mut ::core::ffi::c_void;
pub type ItemAutomationPeer = *mut ::core::ffi::c_void;
pub type ItemsControlAutomationPeer = *mut ::core::ffi::c_void;
pub type ListBoxAutomationPeer = *mut ::core::ffi::c_void;
pub type ListBoxItemAutomationPeer = *mut ::core::ffi::c_void;
pub type ListBoxItemDataAutomationPeer = *mut ::core::ffi::c_void;
pub type ListPickerFlyoutPresenterAutomationPeer = *mut ::core::ffi::c_void;
pub type ListViewAutomationPeer = *mut ::core::ffi::c_void;
pub type ListViewBaseAutomationPeer = *mut ::core::ffi::c_void;
pub type ListViewBaseHeaderItemAutomationPeer = *mut ::core::ffi::c_void;
pub type ListViewHeaderItemAutomationPeer = *mut ::core::ffi::c_void;
pub type ListViewItemAutomationPeer = *mut ::core::ffi::c_void;
pub type ListViewItemDataAutomationPeer = *mut ::core::ffi::c_void;
pub type LoopingSelectorAutomationPeer = *mut ::core::ffi::c_void;
pub type LoopingSelectorItemAutomationPeer = *mut ::core::ffi::c_void;
pub type LoopingSelectorItemDataAutomationPeer = *mut ::core::ffi::c_void;
pub type MapControlAutomationPeer = *mut ::core::ffi::c_void;
pub type MediaElementAutomationPeer = *mut ::core::ffi::c_void;
pub type MediaPlayerElementAutomationPeer = *mut ::core::ffi::c_void;
pub type MediaTransportControlsAutomationPeer = *mut ::core::ffi::c_void;
pub type MenuBarAutomationPeer = *mut ::core::ffi::c_void;
pub type MenuBarItemAutomationPeer = *mut ::core::ffi::c_void;
pub type MenuFlyoutItemAutomationPeer = *mut ::core::ffi::c_void;
pub type MenuFlyoutPresenterAutomationPeer = *mut ::core::ffi::c_void;
pub type NavigationViewItemAutomationPeer = *mut ::core::ffi::c_void;
pub type PasswordBoxAutomationPeer = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Automation_Peers\"`*"]
#[repr(transparent)]
pub struct PatternInterface(pub i32);
impl PatternInterface {
    pub const Invoke: Self = Self(0i32);
    pub const Selection: Self = Self(1i32);
    pub const Value: Self = Self(2i32);
    pub const RangeValue: Self = Self(3i32);
    pub const Scroll: Self = Self(4i32);
    pub const ScrollItem: Self = Self(5i32);
    pub const ExpandCollapse: Self = Self(6i32);
    pub const Grid: Self = Self(7i32);
    pub const GridItem: Self = Self(8i32);
    pub const MultipleView: Self = Self(9i32);
    pub const Window: Self = Self(10i32);
    pub const SelectionItem: Self = Self(11i32);
    pub const Dock: Self = Self(12i32);
    pub const Table: Self = Self(13i32);
    pub const TableItem: Self = Self(14i32);
    pub const Toggle: Self = Self(15i32);
    pub const Transform: Self = Self(16i32);
    pub const Text: Self = Self(17i32);
    pub const ItemContainer: Self = Self(18i32);
    pub const VirtualizedItem: Self = Self(19i32);
    pub const Text2: Self = Self(20i32);
    pub const TextChild: Self = Self(21i32);
    pub const TextRange: Self = Self(22i32);
    pub const Annotation: Self = Self(23i32);
    pub const Drag: Self = Self(24i32);
    pub const DropTarget: Self = Self(25i32);
    pub const ObjectModel: Self = Self(26i32);
    pub const Spreadsheet: Self = Self(27i32);
    pub const SpreadsheetItem: Self = Self(28i32);
    pub const Styles: Self = Self(29i32);
    pub const Transform2: Self = Self(30i32);
    pub const SynchronizedInput: Self = Self(31i32);
    pub const TextEdit: Self = Self(32i32);
    pub const CustomNavigation: Self = Self(33i32);
}
impl ::core::marker::Copy for PatternInterface {}
impl ::core::clone::Clone for PatternInterface {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PersonPictureAutomationPeer = *mut ::core::ffi::c_void;
pub type PickerFlyoutPresenterAutomationPeer = *mut ::core::ffi::c_void;
pub type PivotAutomationPeer = *mut ::core::ffi::c_void;
pub type PivotItemAutomationPeer = *mut ::core::ffi::c_void;
pub type PivotItemDataAutomationPeer = *mut ::core::ffi::c_void;
pub type ProgressBarAutomationPeer = *mut ::core::ffi::c_void;
pub type ProgressRingAutomationPeer = *mut ::core::ffi::c_void;
pub type RadioButtonAutomationPeer = *mut ::core::ffi::c_void;
pub type RangeBaseAutomationPeer = *mut ::core::ffi::c_void;
pub type RatingControlAutomationPeer = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"UI_Xaml_Automation_Peers\"`*"]
pub struct RawElementProviderRuntimeId {
    pub Part1: u32,
    pub Part2: u32,
}
impl ::core::marker::Copy for RawElementProviderRuntimeId {}
impl ::core::clone::Clone for RawElementProviderRuntimeId {
    fn clone(&self) -> Self {
        *self
    }
}
pub type RepeatButtonAutomationPeer = *mut ::core::ffi::c_void;
pub type RichEditBoxAutomationPeer = *mut ::core::ffi::c_void;
pub type RichTextBlockAutomationPeer = *mut ::core::ffi::c_void;
pub type RichTextBlockOverflowAutomationPeer = *mut ::core::ffi::c_void;
pub type ScrollBarAutomationPeer = *mut ::core::ffi::c_void;
pub type ScrollViewerAutomationPeer = *mut ::core::ffi::c_void;
pub type SearchBoxAutomationPeer = *mut ::core::ffi::c_void;
pub type SelectorAutomationPeer = *mut ::core::ffi::c_void;
pub type SelectorItemAutomationPeer = *mut ::core::ffi::c_void;
pub type SemanticZoomAutomationPeer = *mut ::core::ffi::c_void;
pub type SettingsFlyoutAutomationPeer = *mut ::core::ffi::c_void;
pub type SliderAutomationPeer = *mut ::core::ffi::c_void;
pub type TextBlockAutomationPeer = *mut ::core::ffi::c_void;
pub type TextBoxAutomationPeer = *mut ::core::ffi::c_void;
pub type ThumbAutomationPeer = *mut ::core::ffi::c_void;
pub type TimePickerAutomationPeer = *mut ::core::ffi::c_void;
pub type TimePickerFlyoutPresenterAutomationPeer = *mut ::core::ffi::c_void;
pub type ToggleButtonAutomationPeer = *mut ::core::ffi::c_void;
pub type ToggleMenuFlyoutItemAutomationPeer = *mut ::core::ffi::c_void;
pub type ToggleSwitchAutomationPeer = *mut ::core::ffi::c_void;
pub type TreeViewItemAutomationPeer = *mut ::core::ffi::c_void;
pub type TreeViewListAutomationPeer = *mut ::core::ffi::c_void;

#[cfg(feature = "UI_Xaml_Automation")]
pub mod Automation;
#[cfg(feature = "UI_Xaml_Controls")]
pub mod Controls;
#[cfg(feature = "UI_Xaml_Core")]
pub mod Core;
#[cfg(feature = "UI_Xaml_Data")]
pub mod Data;
#[cfg(feature = "UI_Xaml_Documents")]
pub mod Documents;
#[cfg(feature = "UI_Xaml_Hosting")]
pub mod Hosting;
#[cfg(feature = "UI_Xaml_Input")]
pub mod Input;
#[cfg(feature = "UI_Xaml_Interop")]
pub mod Interop;
#[cfg(feature = "UI_Xaml_Markup")]
pub mod Markup;
#[cfg(feature = "UI_Xaml_Media")]
pub mod Media;
#[cfg(feature = "UI_Xaml_Navigation")]
pub mod Navigation;
#[cfg(feature = "UI_Xaml_Printing")]
pub mod Printing;
#[cfg(feature = "UI_Xaml_Resources")]
pub mod Resources;
#[cfg(feature = "UI_Xaml_Shapes")]
pub mod Shapes;
pub type AdaptiveTrigger = *mut ::core::ffi::c_void;
pub type Application = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct ApplicationHighContrastAdjustment(pub u32);
impl ApplicationHighContrastAdjustment {
    pub const None: Self = Self(0u32);
    pub const Auto: Self = Self(4294967295u32);
}
impl ::core::marker::Copy for ApplicationHighContrastAdjustment {}
impl ::core::clone::Clone for ApplicationHighContrastAdjustment {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ApplicationInitializationCallback = *mut ::core::ffi::c_void;
pub type ApplicationInitializationCallbackParams = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct ApplicationRequiresPointerMode(pub i32);
impl ApplicationRequiresPointerMode {
    pub const Auto: Self = Self(0i32);
    pub const WhenRequested: Self = Self(1i32);
}
impl ::core::marker::Copy for ApplicationRequiresPointerMode {}
impl ::core::clone::Clone for ApplicationRequiresPointerMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct ApplicationTheme(pub i32);
impl ApplicationTheme {
    pub const Light: Self = Self(0i32);
    pub const Dark: Self = Self(1i32);
}
impl ::core::marker::Copy for ApplicationTheme {}
impl ::core::clone::Clone for ApplicationTheme {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct AutomationTextAttributesEnum(pub i32);
impl AutomationTextAttributesEnum {
    pub const AnimationStyleAttribute: Self = Self(40000i32);
    pub const BackgroundColorAttribute: Self = Self(40001i32);
    pub const BulletStyleAttribute: Self = Self(40002i32);
    pub const CapStyleAttribute: Self = Self(40003i32);
    pub const CultureAttribute: Self = Self(40004i32);
    pub const FontNameAttribute: Self = Self(40005i32);
    pub const FontSizeAttribute: Self = Self(40006i32);
    pub const FontWeightAttribute: Self = Self(40007i32);
    pub const ForegroundColorAttribute: Self = Self(40008i32);
    pub const HorizontalTextAlignmentAttribute: Self = Self(40009i32);
    pub const IndentationFirstLineAttribute: Self = Self(40010i32);
    pub const IndentationLeadingAttribute: Self = Self(40011i32);
    pub const IndentationTrailingAttribute: Self = Self(40012i32);
    pub const IsHiddenAttribute: Self = Self(40013i32);
    pub const IsItalicAttribute: Self = Self(40014i32);
    pub const IsReadOnlyAttribute: Self = Self(40015i32);
    pub const IsSubscriptAttribute: Self = Self(40016i32);
    pub const IsSuperscriptAttribute: Self = Self(40017i32);
    pub const MarginBottomAttribute: Self = Self(40018i32);
    pub const MarginLeadingAttribute: Self = Self(40019i32);
    pub const MarginTopAttribute: Self = Self(40020i32);
    pub const MarginTrailingAttribute: Self = Self(40021i32);
    pub const OutlineStylesAttribute: Self = Self(40022i32);
    pub const OverlineColorAttribute: Self = Self(40023i32);
    pub const OverlineStyleAttribute: Self = Self(40024i32);
    pub const StrikethroughColorAttribute: Self = Self(40025i32);
    pub const StrikethroughStyleAttribute: Self = Self(40026i32);
    pub const TabsAttribute: Self = Self(40027i32);
    pub const TextFlowDirectionsAttribute: Self = Self(40028i32);
    pub const UnderlineColorAttribute: Self = Self(40029i32);
    pub const UnderlineStyleAttribute: Self = Self(40030i32);
    pub const AnnotationTypesAttribute: Self = Self(40031i32);
    pub const AnnotationObjectsAttribute: Self = Self(40032i32);
    pub const StyleNameAttribute: Self = Self(40033i32);
    pub const StyleIdAttribute: Self = Self(40034i32);
    pub const LinkAttribute: Self = Self(40035i32);
    pub const IsActiveAttribute: Self = Self(40036i32);
    pub const SelectionActiveEndAttribute: Self = Self(40037i32);
    pub const CaretPositionAttribute: Self = Self(40038i32);
    pub const CaretBidiModeAttribute: Self = Self(40039i32);
}
impl ::core::marker::Copy for AutomationTextAttributesEnum {}
impl ::core::clone::Clone for AutomationTextAttributesEnum {
    fn clone(&self) -> Self {
        *self
    }
}
pub type BindingFailedEventArgs = *mut ::core::ffi::c_void;
pub type BindingFailedEventHandler = *mut ::core::ffi::c_void;
pub type BringIntoViewOptions = *mut ::core::ffi::c_void;
pub type BringIntoViewRequestedEventArgs = *mut ::core::ffi::c_void;
pub type BrushTransition = *mut ::core::ffi::c_void;
pub type ColorPaletteResources = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"UI_Xaml\"`*"]
pub struct CornerRadius {
    pub TopLeft: f64,
    pub TopRight: f64,
    pub BottomRight: f64,
    pub BottomLeft: f64,
}
impl ::core::marker::Copy for CornerRadius {}
impl ::core::clone::Clone for CornerRadius {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CornerRadiusHelper = *mut ::core::ffi::c_void;
pub type CreateDefaultValueCallback = *mut ::core::ffi::c_void;
pub type DataContextChangedEventArgs = *mut ::core::ffi::c_void;
pub type DataTemplate = *mut ::core::ffi::c_void;
pub type DataTemplateKey = *mut ::core::ffi::c_void;
pub type DebugSettings = *mut ::core::ffi::c_void;
pub type DependencyObject = *mut ::core::ffi::c_void;
pub type DependencyObjectCollection = *mut ::core::ffi::c_void;
pub type DependencyProperty = *mut ::core::ffi::c_void;
pub type DependencyPropertyChangedCallback = *mut ::core::ffi::c_void;
pub type DependencyPropertyChangedEventArgs = *mut ::core::ffi::c_void;
pub type DependencyPropertyChangedEventHandler = *mut ::core::ffi::c_void;
pub type DispatcherTimer = *mut ::core::ffi::c_void;
pub type DragEventArgs = *mut ::core::ffi::c_void;
pub type DragEventHandler = *mut ::core::ffi::c_void;
pub type DragOperationDeferral = *mut ::core::ffi::c_void;
pub type DragStartingEventArgs = *mut ::core::ffi::c_void;
pub type DragUI = *mut ::core::ffi::c_void;
pub type DragUIOverride = *mut ::core::ffi::c_void;
pub type DropCompletedEventArgs = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"UI_Xaml\"`, `\"Foundation\"`*"]
#[cfg(feature = "Foundation")]
pub struct Duration {
    pub TimeSpan: super::super::Foundation::TimeSpan,
    pub Type: DurationType,
}
#[cfg(feature = "Foundation")]
impl ::core::marker::Copy for Duration {}
#[cfg(feature = "Foundation")]
impl ::core::clone::Clone for Duration {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DurationHelper = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct DurationType(pub i32);
impl DurationType {
    pub const Automatic: Self = Self(0i32);
    pub const TimeSpan: Self = Self(1i32);
    pub const Forever: Self = Self(2i32);
}
impl ::core::marker::Copy for DurationType {}
impl ::core::clone::Clone for DurationType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type EffectiveViewportChangedEventArgs = *mut ::core::ffi::c_void;
pub type ElementFactoryGetArgs = *mut ::core::ffi::c_void;
pub type ElementFactoryRecycleArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct ElementHighContrastAdjustment(pub u32);
impl ElementHighContrastAdjustment {
    pub const None: Self = Self(0u32);
    pub const Application: Self = Self(2147483648u32);
    pub const Auto: Self = Self(4294967295u32);
}
impl ::core::marker::Copy for ElementHighContrastAdjustment {}
impl ::core::clone::Clone for ElementHighContrastAdjustment {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct ElementSoundKind(pub i32);
impl ElementSoundKind {
    pub const Focus: Self = Self(0i32);
    pub const Invoke: Self = Self(1i32);
    pub const Show: Self = Self(2i32);
    pub const Hide: Self = Self(3i32);
    pub const MovePrevious: Self = Self(4i32);
    pub const MoveNext: Self = Self(5i32);
    pub const GoBack: Self = Self(6i32);
}
impl ::core::marker::Copy for ElementSoundKind {}
impl ::core::clone::Clone for ElementSoundKind {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct ElementSoundMode(pub i32);
impl ElementSoundMode {
    pub const Default: Self = Self(0i32);
    pub const FocusOnly: Self = Self(1i32);
    pub const Off: Self = Self(2i32);
}
impl ::core::marker::Copy for ElementSoundMode {}
impl ::core::clone::Clone for ElementSoundMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ElementSoundPlayer = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct ElementSoundPlayerState(pub i32);
impl ElementSoundPlayerState {
    pub const Auto: Self = Self(0i32);
    pub const Off: Self = Self(1i32);
    pub const On: Self = Self(2i32);
}
impl ::core::marker::Copy for ElementSoundPlayerState {}
impl ::core::clone::Clone for ElementSoundPlayerState {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct ElementSpatialAudioMode(pub i32);
impl ElementSpatialAudioMode {
    pub const Auto: Self = Self(0i32);
    pub const Off: Self = Self(1i32);
    pub const On: Self = Self(2i32);
}
impl ::core::marker::Copy for ElementSpatialAudioMode {}
impl ::core::clone::Clone for ElementSpatialAudioMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct ElementTheme(pub i32);
impl ElementTheme {
    pub const Default: Self = Self(0i32);
    pub const Light: Self = Self(1i32);
    pub const Dark: Self = Self(2i32);
}
impl ::core::marker::Copy for ElementTheme {}
impl ::core::clone::Clone for ElementTheme {
    fn clone(&self) -> Self {
        *self
    }
}
pub type EnteredBackgroundEventHandler = *mut ::core::ffi::c_void;
pub type EventTrigger = *mut ::core::ffi::c_void;
pub type ExceptionRoutedEventArgs = *mut ::core::ffi::c_void;
pub type ExceptionRoutedEventHandler = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct FlowDirection(pub i32);
impl FlowDirection {
    pub const LeftToRight: Self = Self(0i32);
    pub const RightToLeft: Self = Self(1i32);
}
impl ::core::marker::Copy for FlowDirection {}
impl ::core::clone::Clone for FlowDirection {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct FocusState(pub i32);
impl FocusState {
    pub const Unfocused: Self = Self(0i32);
    pub const Pointer: Self = Self(1i32);
    pub const Keyboard: Self = Self(2i32);
    pub const Programmatic: Self = Self(3i32);
}
impl ::core::marker::Copy for FocusState {}
impl ::core::clone::Clone for FocusState {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct FocusVisualKind(pub i32);
impl FocusVisualKind {
    pub const DottedLine: Self = Self(0i32);
    pub const HighVisibility: Self = Self(1i32);
    pub const Reveal: Self = Self(2i32);
}
impl ::core::marker::Copy for FocusVisualKind {}
impl ::core::clone::Clone for FocusVisualKind {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct FontCapitals(pub i32);
impl FontCapitals {
    pub const Normal: Self = Self(0i32);
    pub const AllSmallCaps: Self = Self(1i32);
    pub const SmallCaps: Self = Self(2i32);
    pub const AllPetiteCaps: Self = Self(3i32);
    pub const PetiteCaps: Self = Self(4i32);
    pub const Unicase: Self = Self(5i32);
    pub const Titling: Self = Self(6i32);
}
impl ::core::marker::Copy for FontCapitals {}
impl ::core::clone::Clone for FontCapitals {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct FontEastAsianLanguage(pub i32);
impl FontEastAsianLanguage {
    pub const Normal: Self = Self(0i32);
    pub const HojoKanji: Self = Self(1i32);
    pub const Jis04: Self = Self(2i32);
    pub const Jis78: Self = Self(3i32);
    pub const Jis83: Self = Self(4i32);
    pub const Jis90: Self = Self(5i32);
    pub const NlcKanji: Self = Self(6i32);
    pub const Simplified: Self = Self(7i32);
    pub const Traditional: Self = Self(8i32);
    pub const TraditionalNames: Self = Self(9i32);
}
impl ::core::marker::Copy for FontEastAsianLanguage {}
impl ::core::clone::Clone for FontEastAsianLanguage {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct FontEastAsianWidths(pub i32);
impl FontEastAsianWidths {
    pub const Normal: Self = Self(0i32);
    pub const Full: Self = Self(1i32);
    pub const Half: Self = Self(2i32);
    pub const Proportional: Self = Self(3i32);
    pub const Quarter: Self = Self(4i32);
    pub const Third: Self = Self(5i32);
}
impl ::core::marker::Copy for FontEastAsianWidths {}
impl ::core::clone::Clone for FontEastAsianWidths {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct FontFraction(pub i32);
impl FontFraction {
    pub const Normal: Self = Self(0i32);
    pub const Stacked: Self = Self(1i32);
    pub const Slashed: Self = Self(2i32);
}
impl ::core::marker::Copy for FontFraction {}
impl ::core::clone::Clone for FontFraction {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct FontNumeralAlignment(pub i32);
impl FontNumeralAlignment {
    pub const Normal: Self = Self(0i32);
    pub const Proportional: Self = Self(1i32);
    pub const Tabular: Self = Self(2i32);
}
impl ::core::marker::Copy for FontNumeralAlignment {}
impl ::core::clone::Clone for FontNumeralAlignment {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct FontNumeralStyle(pub i32);
impl FontNumeralStyle {
    pub const Normal: Self = Self(0i32);
    pub const Lining: Self = Self(1i32);
    pub const OldStyle: Self = Self(2i32);
}
impl ::core::marker::Copy for FontNumeralStyle {}
impl ::core::clone::Clone for FontNumeralStyle {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct FontVariants(pub i32);
impl FontVariants {
    pub const Normal: Self = Self(0i32);
    pub const Superscript: Self = Self(1i32);
    pub const Subscript: Self = Self(2i32);
    pub const Ordinal: Self = Self(3i32);
    pub const Inferior: Self = Self(4i32);
    pub const Ruby: Self = Self(5i32);
}
impl ::core::marker::Copy for FontVariants {}
impl ::core::clone::Clone for FontVariants {
    fn clone(&self) -> Self {
        *self
    }
}
pub type FrameworkElement = *mut ::core::ffi::c_void;
pub type FrameworkTemplate = *mut ::core::ffi::c_void;
pub type FrameworkView = *mut ::core::ffi::c_void;
pub type FrameworkViewSource = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"UI_Xaml\"`*"]
pub struct GridLength {
    pub Value: f64,
    pub GridUnitType: GridUnitType,
}
impl ::core::marker::Copy for GridLength {}
impl ::core::clone::Clone for GridLength {
    fn clone(&self) -> Self {
        *self
    }
}
pub type GridLengthHelper = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct GridUnitType(pub i32);
impl GridUnitType {
    pub const Auto: Self = Self(0i32);
    pub const Pixel: Self = Self(1i32);
    pub const Star: Self = Self(2i32);
}
impl ::core::marker::Copy for GridUnitType {}
impl ::core::clone::Clone for GridUnitType {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct HorizontalAlignment(pub i32);
impl HorizontalAlignment {
    pub const Left: Self = Self(0i32);
    pub const Center: Self = Self(1i32);
    pub const Right: Self = Self(2i32);
    pub const Stretch: Self = Self(3i32);
}
impl ::core::marker::Copy for HorizontalAlignment {}
impl ::core::clone::Clone for HorizontalAlignment {
    fn clone(&self) -> Self {
        *self
    }
}
pub type IDataTemplateExtension = *mut ::core::ffi::c_void;
pub type IElementFactory = *mut ::core::ffi::c_void;
pub type LeavingBackgroundEventHandler = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct LineStackingStrategy(pub i32);
impl LineStackingStrategy {
    pub const MaxHeight: Self = Self(0i32);
    pub const BlockLineHeight: Self = Self(1i32);
    pub const BaselineToBaseline: Self = Self(2i32);
}
impl ::core::marker::Copy for LineStackingStrategy {}
impl ::core::clone::Clone for LineStackingStrategy {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MediaFailedRoutedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct OpticalMarginAlignment(pub i32);
impl OpticalMarginAlignment {
    pub const None: Self = Self(0i32);
    pub const TrimSideBearings: Self = Self(1i32);
}
impl ::core::marker::Copy for OpticalMarginAlignment {}
impl ::core::clone::Clone for OpticalMarginAlignment {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PointHelper = *mut ::core::ffi::c_void;
pub type PropertyChangedCallback = *mut ::core::ffi::c_void;
pub type PropertyMetadata = *mut ::core::ffi::c_void;
pub type PropertyPath = *mut ::core::ffi::c_void;
pub type RectHelper = *mut ::core::ffi::c_void;
pub type ResourceDictionary = *mut ::core::ffi::c_void;
pub type RoutedEvent = *mut ::core::ffi::c_void;
pub type RoutedEventArgs = *mut ::core::ffi::c_void;
pub type RoutedEventHandler = *mut ::core::ffi::c_void;
pub type ScalarTransition = *mut ::core::ffi::c_void;
pub type Setter = *mut ::core::ffi::c_void;
pub type SetterBase = *mut ::core::ffi::c_void;
pub type SetterBaseCollection = *mut ::core::ffi::c_void;
pub type SizeChangedEventArgs = *mut ::core::ffi::c_void;
pub type SizeChangedEventHandler = *mut ::core::ffi::c_void;
pub type SizeHelper = *mut ::core::ffi::c_void;
pub type StateTrigger = *mut ::core::ffi::c_void;
pub type StateTriggerBase = *mut ::core::ffi::c_void;
pub type Style = *mut ::core::ffi::c_void;
pub type SuspendingEventHandler = *mut ::core::ffi::c_void;
pub type TargetPropertyPath = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct TextAlignment(pub i32);
impl TextAlignment {
    pub const Center: Self = Self(0i32);
    pub const Left: Self = Self(1i32);
    pub const Start: Self = Self(1i32);
    pub const Right: Self = Self(2i32);
    pub const End: Self = Self(2i32);
    pub const Justify: Self = Self(3i32);
    pub const DetectFromContent: Self = Self(4i32);
}
impl ::core::marker::Copy for TextAlignment {}
impl ::core::clone::Clone for TextAlignment {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct TextLineBounds(pub i32);
impl TextLineBounds {
    pub const Full: Self = Self(0i32);
    pub const TrimToCapHeight: Self = Self(1i32);
    pub const TrimToBaseline: Self = Self(2i32);
    pub const Tight: Self = Self(3i32);
}
impl ::core::marker::Copy for TextLineBounds {}
impl ::core::clone::Clone for TextLineBounds {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct TextReadingOrder(pub i32);
impl TextReadingOrder {
    pub const Default: Self = Self(0i32);
    pub const UseFlowDirection: Self = Self(0i32);
    pub const DetectFromContent: Self = Self(1i32);
}
impl ::core::marker::Copy for TextReadingOrder {}
impl ::core::clone::Clone for TextReadingOrder {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct TextTrimming(pub i32);
impl TextTrimming {
    pub const None: Self = Self(0i32);
    pub const CharacterEllipsis: Self = Self(1i32);
    pub const WordEllipsis: Self = Self(2i32);
    pub const Clip: Self = Self(3i32);
}
impl ::core::marker::Copy for TextTrimming {}
impl ::core::clone::Clone for TextTrimming {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct TextWrapping(pub i32);
impl TextWrapping {
    pub const NoWrap: Self = Self(1i32);
    pub const Wrap: Self = Self(2i32);
    pub const WrapWholeWords: Self = Self(3i32);
}
impl ::core::marker::Copy for TextWrapping {}
impl ::core::clone::Clone for TextWrapping {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"UI_Xaml\"`*"]
pub struct Thickness {
    pub Left: f64,
    pub Top: f64,
    pub Right: f64,
    pub Bottom: f64,
}
impl ::core::marker::Copy for Thickness {}
impl ::core::clone::Clone for Thickness {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ThicknessHelper = *mut ::core::ffi::c_void;
pub type TriggerAction = *mut ::core::ffi::c_void;
pub type TriggerActionCollection = *mut ::core::ffi::c_void;
pub type TriggerBase = *mut ::core::ffi::c_void;
pub type TriggerCollection = *mut ::core::ffi::c_void;
pub type UIElement = *mut ::core::ffi::c_void;
pub type UIElementWeakCollection = *mut ::core::ffi::c_void;
pub type UnhandledExceptionEventArgs = *mut ::core::ffi::c_void;
pub type UnhandledExceptionEventHandler = *mut ::core::ffi::c_void;
pub type Vector3Transition = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct Vector3TransitionComponents(pub u32);
impl Vector3TransitionComponents {
    pub const X: Self = Self(1u32);
    pub const Y: Self = Self(2u32);
    pub const Z: Self = Self(4u32);
}
impl ::core::marker::Copy for Vector3TransitionComponents {}
impl ::core::clone::Clone for Vector3TransitionComponents {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct VerticalAlignment(pub i32);
impl VerticalAlignment {
    pub const Top: Self = Self(0i32);
    pub const Center: Self = Self(1i32);
    pub const Bottom: Self = Self(2i32);
    pub const Stretch: Self = Self(3i32);
}
impl ::core::marker::Copy for VerticalAlignment {}
impl ::core::clone::Clone for VerticalAlignment {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml\"`*"]
#[repr(transparent)]
pub struct Visibility(pub i32);
impl Visibility {
    pub const Visible: Self = Self(0i32);
    pub const Collapsed: Self = Self(1i32);
}
impl ::core::marker::Copy for Visibility {}
impl ::core::clone::Clone for Visibility {
    fn clone(&self) -> Self {
        *self
    }
}
pub type VisualState = *mut ::core::ffi::c_void;
pub type VisualStateChangedEventArgs = *mut ::core::ffi::c_void;
pub type VisualStateChangedEventHandler = *mut ::core::ffi::c_void;
pub type VisualStateGroup = *mut ::core::ffi::c_void;
pub type VisualStateManager = *mut ::core::ffi::c_void;
pub type VisualTransition = *mut ::core::ffi::c_void;
pub type Window = *mut ::core::ffi::c_void;
pub type WindowActivatedEventHandler = *mut ::core::ffi::c_void;
pub type WindowClosedEventHandler = *mut ::core::ffi::c_void;
pub type WindowCreatedEventArgs = *mut ::core::ffi::c_void;
pub type WindowSizeChangedEventHandler = *mut ::core::ffi::c_void;
pub type WindowVisibilityChangedEventHandler = *mut ::core::ffi::c_void;
pub type XamlRoot = *mut ::core::ffi::c_void;
pub type XamlRootChangedEventArgs = *mut ::core::ffi::c_void;

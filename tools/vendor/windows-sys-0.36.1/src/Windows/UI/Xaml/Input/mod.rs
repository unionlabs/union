pub type AccessKeyDisplayDismissedEventArgs = *mut ::core::ffi::c_void;
pub type AccessKeyDisplayRequestedEventArgs = *mut ::core::ffi::c_void;
pub type AccessKeyInvokedEventArgs = *mut ::core::ffi::c_void;
pub type AccessKeyManager = *mut ::core::ffi::c_void;
pub type CanExecuteRequestedEventArgs = *mut ::core::ffi::c_void;
pub type CharacterReceivedRoutedEventArgs = *mut ::core::ffi::c_void;
pub type ContextRequestedEventArgs = *mut ::core::ffi::c_void;
pub type DoubleTappedEventHandler = *mut ::core::ffi::c_void;
pub type DoubleTappedRoutedEventArgs = *mut ::core::ffi::c_void;
pub type ExecuteRequestedEventArgs = *mut ::core::ffi::c_void;
pub type FindNextElementOptions = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Input\"`*"]
#[repr(transparent)]
pub struct FocusInputDeviceKind(pub i32);
impl FocusInputDeviceKind {
    pub const None: Self = Self(0i32);
    pub const Mouse: Self = Self(1i32);
    pub const Touch: Self = Self(2i32);
    pub const Pen: Self = Self(3i32);
    pub const Keyboard: Self = Self(4i32);
    pub const GameController: Self = Self(5i32);
}
impl ::core::marker::Copy for FocusInputDeviceKind {}
impl ::core::clone::Clone for FocusInputDeviceKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type FocusManager = *mut ::core::ffi::c_void;
pub type FocusManagerGotFocusEventArgs = *mut ::core::ffi::c_void;
pub type FocusManagerLostFocusEventArgs = *mut ::core::ffi::c_void;
pub type FocusMovementResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Input\"`*"]
#[repr(transparent)]
pub struct FocusNavigationDirection(pub i32);
impl FocusNavigationDirection {
    pub const Next: Self = Self(0i32);
    pub const Previous: Self = Self(1i32);
    pub const Up: Self = Self(2i32);
    pub const Down: Self = Self(3i32);
    pub const Left: Self = Self(4i32);
    pub const Right: Self = Self(5i32);
    pub const None: Self = Self(6i32);
}
impl ::core::marker::Copy for FocusNavigationDirection {}
impl ::core::clone::Clone for FocusNavigationDirection {
    fn clone(&self) -> Self {
        *self
    }
}
pub type GettingFocusEventArgs = *mut ::core::ffi::c_void;
pub type HoldingEventHandler = *mut ::core::ffi::c_void;
pub type HoldingRoutedEventArgs = *mut ::core::ffi::c_void;
pub type ICommand = *mut ::core::ffi::c_void;
pub type InertiaExpansionBehavior = *mut ::core::ffi::c_void;
pub type InertiaRotationBehavior = *mut ::core::ffi::c_void;
pub type InertiaTranslationBehavior = *mut ::core::ffi::c_void;
pub type InputScope = *mut ::core::ffi::c_void;
pub type InputScopeName = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Input\"`*"]
#[repr(transparent)]
pub struct InputScopeNameValue(pub i32);
impl InputScopeNameValue {
    pub const Default: Self = Self(0i32);
    pub const Url: Self = Self(1i32);
    pub const EmailSmtpAddress: Self = Self(5i32);
    pub const PersonalFullName: Self = Self(7i32);
    pub const CurrencyAmountAndSymbol: Self = Self(20i32);
    pub const CurrencyAmount: Self = Self(21i32);
    pub const DateMonthNumber: Self = Self(23i32);
    pub const DateDayNumber: Self = Self(24i32);
    pub const DateYear: Self = Self(25i32);
    pub const Digits: Self = Self(28i32);
    pub const Number: Self = Self(29i32);
    pub const Password: Self = Self(31i32);
    pub const TelephoneNumber: Self = Self(32i32);
    pub const TelephoneCountryCode: Self = Self(33i32);
    pub const TelephoneAreaCode: Self = Self(34i32);
    pub const TelephoneLocalNumber: Self = Self(35i32);
    pub const TimeHour: Self = Self(37i32);
    pub const TimeMinutesOrSeconds: Self = Self(38i32);
    pub const NumberFullWidth: Self = Self(39i32);
    pub const AlphanumericHalfWidth: Self = Self(40i32);
    pub const AlphanumericFullWidth: Self = Self(41i32);
    pub const Hiragana: Self = Self(44i32);
    pub const KatakanaHalfWidth: Self = Self(45i32);
    pub const KatakanaFullWidth: Self = Self(46i32);
    pub const Hanja: Self = Self(47i32);
    pub const HangulHalfWidth: Self = Self(48i32);
    pub const HangulFullWidth: Self = Self(49i32);
    pub const Search: Self = Self(50i32);
    pub const Formula: Self = Self(51i32);
    pub const SearchIncremental: Self = Self(52i32);
    pub const ChineseHalfWidth: Self = Self(53i32);
    pub const ChineseFullWidth: Self = Self(54i32);
    pub const NativeScript: Self = Self(55i32);
    pub const Text: Self = Self(57i32);
    pub const Chat: Self = Self(58i32);
    pub const NameOrPhoneNumber: Self = Self(59i32);
    pub const EmailNameOrAddress: Self = Self(60i32);
    pub const Private: Self = Self(61i32);
    pub const Maps: Self = Self(62i32);
    pub const NumericPassword: Self = Self(63i32);
    pub const NumericPin: Self = Self(64i32);
    pub const AlphanumericPin: Self = Self(65i32);
    pub const FormulaNumber: Self = Self(67i32);
    pub const ChatWithoutEmoji: Self = Self(68i32);
}
impl ::core::marker::Copy for InputScopeNameValue {}
impl ::core::clone::Clone for InputScopeNameValue {
    fn clone(&self) -> Self {
        *self
    }
}
pub type KeyEventHandler = *mut ::core::ffi::c_void;
pub type KeyRoutedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Input\"`*"]
#[repr(transparent)]
pub struct KeyTipPlacementMode(pub i32);
impl KeyTipPlacementMode {
    pub const Auto: Self = Self(0i32);
    pub const Bottom: Self = Self(1i32);
    pub const Top: Self = Self(2i32);
    pub const Left: Self = Self(3i32);
    pub const Right: Self = Self(4i32);
    pub const Center: Self = Self(5i32);
    pub const Hidden: Self = Self(6i32);
}
impl ::core::marker::Copy for KeyTipPlacementMode {}
impl ::core::clone::Clone for KeyTipPlacementMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type KeyboardAccelerator = *mut ::core::ffi::c_void;
pub type KeyboardAcceleratorInvokedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Input\"`*"]
#[repr(transparent)]
pub struct KeyboardAcceleratorPlacementMode(pub i32);
impl KeyboardAcceleratorPlacementMode {
    pub const Auto: Self = Self(0i32);
    pub const Hidden: Self = Self(1i32);
}
impl ::core::marker::Copy for KeyboardAcceleratorPlacementMode {}
impl ::core::clone::Clone for KeyboardAcceleratorPlacementMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Input\"`*"]
#[repr(transparent)]
pub struct KeyboardNavigationMode(pub i32);
impl KeyboardNavigationMode {
    pub const Local: Self = Self(0i32);
    pub const Cycle: Self = Self(1i32);
    pub const Once: Self = Self(2i32);
}
impl ::core::marker::Copy for KeyboardNavigationMode {}
impl ::core::clone::Clone for KeyboardNavigationMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type LosingFocusEventArgs = *mut ::core::ffi::c_void;
pub type ManipulationCompletedEventHandler = *mut ::core::ffi::c_void;
pub type ManipulationCompletedRoutedEventArgs = *mut ::core::ffi::c_void;
pub type ManipulationDeltaEventHandler = *mut ::core::ffi::c_void;
pub type ManipulationDeltaRoutedEventArgs = *mut ::core::ffi::c_void;
pub type ManipulationInertiaStartingEventHandler = *mut ::core::ffi::c_void;
pub type ManipulationInertiaStartingRoutedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Input\"`*"]
#[repr(transparent)]
pub struct ManipulationModes(pub u32);
impl ManipulationModes {
    pub const None: Self = Self(0u32);
    pub const TranslateX: Self = Self(1u32);
    pub const TranslateY: Self = Self(2u32);
    pub const TranslateRailsX: Self = Self(4u32);
    pub const TranslateRailsY: Self = Self(8u32);
    pub const Rotate: Self = Self(16u32);
    pub const Scale: Self = Self(32u32);
    pub const TranslateInertia: Self = Self(64u32);
    pub const RotateInertia: Self = Self(128u32);
    pub const ScaleInertia: Self = Self(256u32);
    pub const All: Self = Self(65535u32);
    pub const System: Self = Self(65536u32);
}
impl ::core::marker::Copy for ManipulationModes {}
impl ::core::clone::Clone for ManipulationModes {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ManipulationPivot = *mut ::core::ffi::c_void;
pub type ManipulationStartedEventHandler = *mut ::core::ffi::c_void;
pub type ManipulationStartedRoutedEventArgs = *mut ::core::ffi::c_void;
pub type ManipulationStartingEventHandler = *mut ::core::ffi::c_void;
pub type ManipulationStartingRoutedEventArgs = *mut ::core::ffi::c_void;
pub type NoFocusCandidateFoundEventArgs = *mut ::core::ffi::c_void;
pub type Pointer = *mut ::core::ffi::c_void;
pub type PointerEventHandler = *mut ::core::ffi::c_void;
pub type PointerRoutedEventArgs = *mut ::core::ffi::c_void;
pub type ProcessKeyboardAcceleratorEventArgs = *mut ::core::ffi::c_void;
pub type RightTappedEventHandler = *mut ::core::ffi::c_void;
pub type RightTappedRoutedEventArgs = *mut ::core::ffi::c_void;
pub type StandardUICommand = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Input\"`*"]
#[repr(transparent)]
pub struct StandardUICommandKind(pub i32);
impl StandardUICommandKind {
    pub const None: Self = Self(0i32);
    pub const Cut: Self = Self(1i32);
    pub const Copy: Self = Self(2i32);
    pub const Paste: Self = Self(3i32);
    pub const SelectAll: Self = Self(4i32);
    pub const Delete: Self = Self(5i32);
    pub const Share: Self = Self(6i32);
    pub const Save: Self = Self(7i32);
    pub const Open: Self = Self(8i32);
    pub const Close: Self = Self(9i32);
    pub const Pause: Self = Self(10i32);
    pub const Play: Self = Self(11i32);
    pub const Stop: Self = Self(12i32);
    pub const Forward: Self = Self(13i32);
    pub const Backward: Self = Self(14i32);
    pub const Undo: Self = Self(15i32);
    pub const Redo: Self = Self(16i32);
}
impl ::core::marker::Copy for StandardUICommandKind {}
impl ::core::clone::Clone for StandardUICommandKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type TappedEventHandler = *mut ::core::ffi::c_void;
pub type TappedRoutedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Input\"`*"]
#[repr(transparent)]
pub struct XYFocusKeyboardNavigationMode(pub i32);
impl XYFocusKeyboardNavigationMode {
    pub const Auto: Self = Self(0i32);
    pub const Enabled: Self = Self(1i32);
    pub const Disabled: Self = Self(2i32);
}
impl ::core::marker::Copy for XYFocusKeyboardNavigationMode {}
impl ::core::clone::Clone for XYFocusKeyboardNavigationMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Input\"`*"]
#[repr(transparent)]
pub struct XYFocusNavigationStrategy(pub i32);
impl XYFocusNavigationStrategy {
    pub const Auto: Self = Self(0i32);
    pub const Projection: Self = Self(1i32);
    pub const NavigationDirectionDistance: Self = Self(2i32);
    pub const RectilinearDistance: Self = Self(3i32);
}
impl ::core::marker::Copy for XYFocusNavigationStrategy {}
impl ::core::clone::Clone for XYFocusNavigationStrategy {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Xaml_Input\"`*"]
#[repr(transparent)]
pub struct XYFocusNavigationStrategyOverride(pub i32);
impl XYFocusNavigationStrategyOverride {
    pub const None: Self = Self(0i32);
    pub const Auto: Self = Self(1i32);
    pub const Projection: Self = Self(2i32);
    pub const NavigationDirectionDistance: Self = Self(3i32);
    pub const RectilinearDistance: Self = Self(4i32);
}
impl ::core::marker::Copy for XYFocusNavigationStrategyOverride {}
impl ::core::clone::Clone for XYFocusNavigationStrategyOverride {
    fn clone(&self) -> Self {
        *self
    }
}
pub type XamlUICommand = *mut ::core::ffi::c_void;

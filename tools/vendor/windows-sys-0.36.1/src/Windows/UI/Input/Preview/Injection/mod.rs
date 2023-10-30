#[doc = "*Required features: `\"UI_Input_Preview_Injection\"`*"]
#[repr(transparent)]
pub struct InjectedInputButtonChangeKind(pub i32);
impl InjectedInputButtonChangeKind {
    pub const None: Self = Self(0i32);
    pub const FirstButtonDown: Self = Self(1i32);
    pub const FirstButtonUp: Self = Self(2i32);
    pub const SecondButtonDown: Self = Self(3i32);
    pub const SecondButtonUp: Self = Self(4i32);
    pub const ThirdButtonDown: Self = Self(5i32);
    pub const ThirdButtonUp: Self = Self(6i32);
    pub const FourthButtonDown: Self = Self(7i32);
    pub const FourthButtonUp: Self = Self(8i32);
    pub const FifthButtonDown: Self = Self(9i32);
    pub const FifthButtonUp: Self = Self(10i32);
}
impl ::core::marker::Copy for InjectedInputButtonChangeKind {}
impl ::core::clone::Clone for InjectedInputButtonChangeKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type InjectedInputGamepadInfo = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Input_Preview_Injection\"`*"]
#[repr(transparent)]
pub struct InjectedInputKeyOptions(pub u32);
impl InjectedInputKeyOptions {
    pub const None: Self = Self(0u32);
    pub const ExtendedKey: Self = Self(1u32);
    pub const KeyUp: Self = Self(2u32);
    pub const ScanCode: Self = Self(8u32);
    pub const Unicode: Self = Self(4u32);
}
impl ::core::marker::Copy for InjectedInputKeyOptions {}
impl ::core::clone::Clone for InjectedInputKeyOptions {
    fn clone(&self) -> Self {
        *self
    }
}
pub type InjectedInputKeyboardInfo = *mut ::core::ffi::c_void;
pub type InjectedInputMouseInfo = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Input_Preview_Injection\"`*"]
#[repr(transparent)]
pub struct InjectedInputMouseOptions(pub u32);
impl InjectedInputMouseOptions {
    pub const None: Self = Self(0u32);
    pub const Move: Self = Self(1u32);
    pub const LeftDown: Self = Self(2u32);
    pub const LeftUp: Self = Self(4u32);
    pub const RightDown: Self = Self(8u32);
    pub const RightUp: Self = Self(16u32);
    pub const MiddleDown: Self = Self(32u32);
    pub const MiddleUp: Self = Self(64u32);
    pub const XDown: Self = Self(128u32);
    pub const XUp: Self = Self(256u32);
    pub const Wheel: Self = Self(2048u32);
    pub const HWheel: Self = Self(4096u32);
    pub const MoveNoCoalesce: Self = Self(8192u32);
    pub const VirtualDesk: Self = Self(16384u32);
    pub const Absolute: Self = Self(32768u32);
}
impl ::core::marker::Copy for InjectedInputMouseOptions {}
impl ::core::clone::Clone for InjectedInputMouseOptions {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Input_Preview_Injection\"`*"]
#[repr(transparent)]
pub struct InjectedInputPenButtons(pub u32);
impl InjectedInputPenButtons {
    pub const None: Self = Self(0u32);
    pub const Barrel: Self = Self(1u32);
    pub const Inverted: Self = Self(2u32);
    pub const Eraser: Self = Self(4u32);
}
impl ::core::marker::Copy for InjectedInputPenButtons {}
impl ::core::clone::Clone for InjectedInputPenButtons {
    fn clone(&self) -> Self {
        *self
    }
}
pub type InjectedInputPenInfo = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Input_Preview_Injection\"`*"]
#[repr(transparent)]
pub struct InjectedInputPenParameters(pub u32);
impl InjectedInputPenParameters {
    pub const None: Self = Self(0u32);
    pub const Pressure: Self = Self(1u32);
    pub const Rotation: Self = Self(2u32);
    pub const TiltX: Self = Self(4u32);
    pub const TiltY: Self = Self(8u32);
}
impl ::core::marker::Copy for InjectedInputPenParameters {}
impl ::core::clone::Clone for InjectedInputPenParameters {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"UI_Input_Preview_Injection\"`*"]
pub struct InjectedInputPoint {
    pub PositionX: i32,
    pub PositionY: i32,
}
impl ::core::marker::Copy for InjectedInputPoint {}
impl ::core::clone::Clone for InjectedInputPoint {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"UI_Input_Preview_Injection\"`*"]
pub struct InjectedInputPointerInfo {
    pub PointerId: u32,
    pub PointerOptions: InjectedInputPointerOptions,
    pub PixelLocation: InjectedInputPoint,
    pub TimeOffsetInMilliseconds: u32,
    pub PerformanceCount: u64,
}
impl ::core::marker::Copy for InjectedInputPointerInfo {}
impl ::core::clone::Clone for InjectedInputPointerInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Input_Preview_Injection\"`*"]
#[repr(transparent)]
pub struct InjectedInputPointerOptions(pub u32);
impl InjectedInputPointerOptions {
    pub const None: Self = Self(0u32);
    pub const New: Self = Self(1u32);
    pub const InRange: Self = Self(2u32);
    pub const InContact: Self = Self(4u32);
    pub const FirstButton: Self = Self(16u32);
    pub const SecondButton: Self = Self(32u32);
    pub const Primary: Self = Self(8192u32);
    pub const Confidence: Self = Self(16384u32);
    pub const Canceled: Self = Self(32768u32);
    pub const PointerDown: Self = Self(65536u32);
    pub const Update: Self = Self(131072u32);
    pub const PointerUp: Self = Self(262144u32);
    pub const CaptureChanged: Self = Self(2097152u32);
}
impl ::core::marker::Copy for InjectedInputPointerOptions {}
impl ::core::clone::Clone for InjectedInputPointerOptions {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"UI_Input_Preview_Injection\"`*"]
pub struct InjectedInputRectangle {
    pub Left: i32,
    pub Top: i32,
    pub Bottom: i32,
    pub Right: i32,
}
impl ::core::marker::Copy for InjectedInputRectangle {}
impl ::core::clone::Clone for InjectedInputRectangle {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Input_Preview_Injection\"`*"]
#[repr(transparent)]
pub struct InjectedInputShortcut(pub i32);
impl InjectedInputShortcut {
    pub const Back: Self = Self(0i32);
    pub const Start: Self = Self(1i32);
    pub const Search: Self = Self(2i32);
}
impl ::core::marker::Copy for InjectedInputShortcut {}
impl ::core::clone::Clone for InjectedInputShortcut {
    fn clone(&self) -> Self {
        *self
    }
}
pub type InjectedInputTouchInfo = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Input_Preview_Injection\"`*"]
#[repr(transparent)]
pub struct InjectedInputTouchParameters(pub u32);
impl InjectedInputTouchParameters {
    pub const None: Self = Self(0u32);
    pub const Contact: Self = Self(1u32);
    pub const Orientation: Self = Self(2u32);
    pub const Pressure: Self = Self(4u32);
}
impl ::core::marker::Copy for InjectedInputTouchParameters {}
impl ::core::clone::Clone for InjectedInputTouchParameters {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Input_Preview_Injection\"`*"]
#[repr(transparent)]
pub struct InjectedInputVisualizationMode(pub i32);
impl InjectedInputVisualizationMode {
    pub const None: Self = Self(0i32);
    pub const Default: Self = Self(1i32);
    pub const Indirect: Self = Self(2i32);
}
impl ::core::marker::Copy for InjectedInputVisualizationMode {}
impl ::core::clone::Clone for InjectedInputVisualizationMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type InputInjector = *mut ::core::ffi::c_void;

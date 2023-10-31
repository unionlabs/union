#[cfg(feature = "Devices_Input_Preview")]
pub mod Preview;
pub type KeyboardCapabilities = *mut ::core::ffi::c_void;
pub type MouseCapabilities = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"Devices_Input\"`*"]
pub struct MouseDelta {
    pub X: i32,
    pub Y: i32,
}
impl ::core::marker::Copy for MouseDelta {}
impl ::core::clone::Clone for MouseDelta {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MouseDevice = *mut ::core::ffi::c_void;
pub type MouseEventArgs = *mut ::core::ffi::c_void;
pub type PenButtonListener = *mut ::core::ffi::c_void;
pub type PenDevice = *mut ::core::ffi::c_void;
pub type PenDockListener = *mut ::core::ffi::c_void;
pub type PenDockedEventArgs = *mut ::core::ffi::c_void;
pub type PenTailButtonClickedEventArgs = *mut ::core::ffi::c_void;
pub type PenTailButtonDoubleClickedEventArgs = *mut ::core::ffi::c_void;
pub type PenTailButtonLongPressedEventArgs = *mut ::core::ffi::c_void;
pub type PenUndockedEventArgs = *mut ::core::ffi::c_void;
pub type PointerDevice = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Input\"`*"]
#[repr(transparent)]
pub struct PointerDeviceType(pub i32);
impl PointerDeviceType {
    pub const Touch: Self = Self(0i32);
    pub const Pen: Self = Self(1i32);
    pub const Mouse: Self = Self(2i32);
}
impl ::core::marker::Copy for PointerDeviceType {}
impl ::core::clone::Clone for PointerDeviceType {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Devices_Input\"`*"]
pub struct PointerDeviceUsage {
    pub UsagePage: u32,
    pub Usage: u32,
    pub MinLogical: i32,
    pub MaxLogical: i32,
    pub MinPhysical: i32,
    pub MaxPhysical: i32,
    pub Unit: u32,
    pub PhysicalMultiplier: f32,
}
impl ::core::marker::Copy for PointerDeviceUsage {}
impl ::core::clone::Clone for PointerDeviceUsage {
    fn clone(&self) -> Self {
        *self
    }
}
pub type TouchCapabilities = *mut ::core::ffi::c_void;

pub type HidBooleanControl = *mut ::core::ffi::c_void;
pub type HidBooleanControlDescription = *mut ::core::ffi::c_void;
pub type HidCollection = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_HumanInterfaceDevice\"`*"]
#[repr(transparent)]
pub struct HidCollectionType(pub i32);
impl HidCollectionType {
    pub const Physical: Self = Self(0i32);
    pub const Application: Self = Self(1i32);
    pub const Logical: Self = Self(2i32);
    pub const Report: Self = Self(3i32);
    pub const NamedArray: Self = Self(4i32);
    pub const UsageSwitch: Self = Self(5i32);
    pub const UsageModifier: Self = Self(6i32);
    pub const Other: Self = Self(7i32);
}
impl ::core::marker::Copy for HidCollectionType {}
impl ::core::clone::Clone for HidCollectionType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type HidDevice = *mut ::core::ffi::c_void;
pub type HidFeatureReport = *mut ::core::ffi::c_void;
pub type HidInputReport = *mut ::core::ffi::c_void;
pub type HidInputReportReceivedEventArgs = *mut ::core::ffi::c_void;
pub type HidNumericControl = *mut ::core::ffi::c_void;
pub type HidNumericControlDescription = *mut ::core::ffi::c_void;
pub type HidOutputReport = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_HumanInterfaceDevice\"`*"]
#[repr(transparent)]
pub struct HidReportType(pub i32);
impl HidReportType {
    pub const Input: Self = Self(0i32);
    pub const Output: Self = Self(1i32);
    pub const Feature: Self = Self(2i32);
}
impl ::core::marker::Copy for HidReportType {}
impl ::core::clone::Clone for HidReportType {
    fn clone(&self) -> Self {
        *self
    }
}

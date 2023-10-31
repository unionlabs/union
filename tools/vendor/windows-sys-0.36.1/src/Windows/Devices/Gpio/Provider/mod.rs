pub type GpioPinProviderValueChangedEventArgs = *mut ::core::ffi::c_void;
pub type IGpioControllerProvider = *mut ::core::ffi::c_void;
pub type IGpioPinProvider = *mut ::core::ffi::c_void;
pub type IGpioProvider = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Gpio_Provider\"`*"]
#[repr(transparent)]
pub struct ProviderGpioPinDriveMode(pub i32);
impl ProviderGpioPinDriveMode {
    pub const Input: Self = Self(0i32);
    pub const Output: Self = Self(1i32);
    pub const InputPullUp: Self = Self(2i32);
    pub const InputPullDown: Self = Self(3i32);
    pub const OutputOpenDrain: Self = Self(4i32);
    pub const OutputOpenDrainPullUp: Self = Self(5i32);
    pub const OutputOpenSource: Self = Self(6i32);
    pub const OutputOpenSourcePullDown: Self = Self(7i32);
}
impl ::core::marker::Copy for ProviderGpioPinDriveMode {}
impl ::core::clone::Clone for ProviderGpioPinDriveMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Devices_Gpio_Provider\"`*"]
#[repr(transparent)]
pub struct ProviderGpioPinEdge(pub i32);
impl ProviderGpioPinEdge {
    pub const FallingEdge: Self = Self(0i32);
    pub const RisingEdge: Self = Self(1i32);
}
impl ::core::marker::Copy for ProviderGpioPinEdge {}
impl ::core::clone::Clone for ProviderGpioPinEdge {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Devices_Gpio_Provider\"`*"]
#[repr(transparent)]
pub struct ProviderGpioPinValue(pub i32);
impl ProviderGpioPinValue {
    pub const Low: Self = Self(0i32);
    pub const High: Self = Self(1i32);
}
impl ::core::marker::Copy for ProviderGpioPinValue {}
impl ::core::clone::Clone for ProviderGpioPinValue {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Devices_Gpio_Provider\"`*"]
#[repr(transparent)]
pub struct ProviderGpioSharingMode(pub i32);
impl ProviderGpioSharingMode {
    pub const Exclusive: Self = Self(0i32);
    pub const SharedReadOnly: Self = Self(1i32);
}
impl ::core::marker::Copy for ProviderGpioSharingMode {}
impl ::core::clone::Clone for ProviderGpioSharingMode {
    fn clone(&self) -> Self {
        *self
    }
}

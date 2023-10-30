#[cfg(feature = "Devices_Adc_Provider")]
pub mod Provider;
pub type AdcChannel = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Adc\"`*"]
#[repr(transparent)]
pub struct AdcChannelMode(pub i32);
impl AdcChannelMode {
    pub const SingleEnded: Self = Self(0i32);
    pub const Differential: Self = Self(1i32);
}
impl ::core::marker::Copy for AdcChannelMode {}
impl ::core::clone::Clone for AdcChannelMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type AdcController = *mut ::core::ffi::c_void;

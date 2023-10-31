pub type IAdcControllerProvider = *mut ::core::ffi::c_void;
pub type IAdcProvider = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Adc_Provider\"`*"]
#[repr(transparent)]
pub struct ProviderAdcChannelMode(pub i32);
impl ProviderAdcChannelMode {
    pub const SingleEnded: Self = Self(0i32);
    pub const Differential: Self = Self(1i32);
}
impl ::core::marker::Copy for ProviderAdcChannelMode {}
impl ::core::clone::Clone for ProviderAdcChannelMode {
    fn clone(&self) -> Self {
        *self
    }
}

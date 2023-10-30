#[cfg(feature = "Devices_Spi_Provider")]
pub mod Provider;
pub type ISpiDeviceStatics = *mut ::core::ffi::c_void;
pub type SpiBusInfo = *mut ::core::ffi::c_void;
pub type SpiConnectionSettings = *mut ::core::ffi::c_void;
pub type SpiController = *mut ::core::ffi::c_void;
pub type SpiDevice = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Spi\"`*"]
#[repr(transparent)]
pub struct SpiMode(pub i32);
impl SpiMode {
    pub const Mode0: Self = Self(0i32);
    pub const Mode1: Self = Self(1i32);
    pub const Mode2: Self = Self(2i32);
    pub const Mode3: Self = Self(3i32);
}
impl ::core::marker::Copy for SpiMode {}
impl ::core::clone::Clone for SpiMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Devices_Spi\"`*"]
#[repr(transparent)]
pub struct SpiSharingMode(pub i32);
impl SpiSharingMode {
    pub const Exclusive: Self = Self(0i32);
    pub const Shared: Self = Self(1i32);
}
impl ::core::marker::Copy for SpiSharingMode {}
impl ::core::clone::Clone for SpiSharingMode {
    fn clone(&self) -> Self {
        *self
    }
}

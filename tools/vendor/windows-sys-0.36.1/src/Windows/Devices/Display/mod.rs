#[cfg(feature = "Devices_Display_Core")]
pub mod Core;
pub type DisplayMonitor = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Display\"`*"]
#[repr(transparent)]
pub struct DisplayMonitorConnectionKind(pub i32);
impl DisplayMonitorConnectionKind {
    pub const Internal: Self = Self(0i32);
    pub const Wired: Self = Self(1i32);
    pub const Wireless: Self = Self(2i32);
    pub const Virtual: Self = Self(3i32);
}
impl ::core::marker::Copy for DisplayMonitorConnectionKind {}
impl ::core::clone::Clone for DisplayMonitorConnectionKind {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Devices_Display\"`*"]
#[repr(transparent)]
pub struct DisplayMonitorDescriptorKind(pub i32);
impl DisplayMonitorDescriptorKind {
    pub const Edid: Self = Self(0i32);
    pub const DisplayId: Self = Self(1i32);
}
impl ::core::marker::Copy for DisplayMonitorDescriptorKind {}
impl ::core::clone::Clone for DisplayMonitorDescriptorKind {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Devices_Display\"`*"]
#[repr(transparent)]
pub struct DisplayMonitorPhysicalConnectorKind(pub i32);
impl DisplayMonitorPhysicalConnectorKind {
    pub const Unknown: Self = Self(0i32);
    pub const HD15: Self = Self(1i32);
    pub const AnalogTV: Self = Self(2i32);
    pub const Dvi: Self = Self(3i32);
    pub const Hdmi: Self = Self(4i32);
    pub const Lvds: Self = Self(5i32);
    pub const Sdi: Self = Self(6i32);
    pub const DisplayPort: Self = Self(7i32);
}
impl ::core::marker::Copy for DisplayMonitorPhysicalConnectorKind {}
impl ::core::clone::Clone for DisplayMonitorPhysicalConnectorKind {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Devices_Display\"`*"]
#[repr(transparent)]
pub struct DisplayMonitorUsageKind(pub i32);
impl DisplayMonitorUsageKind {
    pub const Standard: Self = Self(0i32);
    pub const HeadMounted: Self = Self(1i32);
    pub const SpecialPurpose: Self = Self(2i32);
}
impl ::core::marker::Copy for DisplayMonitorUsageKind {}
impl ::core::clone::Clone for DisplayMonitorUsageKind {
    fn clone(&self) -> Self {
        *self
    }
}

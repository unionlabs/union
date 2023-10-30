#[cfg(feature = "Devices_Lights_Effects")]
pub mod Effects;
pub type Lamp = *mut ::core::ffi::c_void;
pub type LampArray = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Lights\"`*"]
#[repr(transparent)]
pub struct LampArrayKind(pub i32);
impl LampArrayKind {
    pub const Undefined: Self = Self(0i32);
    pub const Keyboard: Self = Self(1i32);
    pub const Mouse: Self = Self(2i32);
    pub const GameController: Self = Self(3i32);
    pub const Peripheral: Self = Self(4i32);
    pub const Scene: Self = Self(5i32);
    pub const Notification: Self = Self(6i32);
    pub const Chassis: Self = Self(7i32);
    pub const Wearable: Self = Self(8i32);
    pub const Furniture: Self = Self(9i32);
    pub const Art: Self = Self(10i32);
}
impl ::core::marker::Copy for LampArrayKind {}
impl ::core::clone::Clone for LampArrayKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type LampAvailabilityChangedEventArgs = *mut ::core::ffi::c_void;
pub type LampInfo = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Lights\"`*"]
#[repr(transparent)]
pub struct LampPurposes(pub u32);
impl LampPurposes {
    pub const Undefined: Self = Self(0u32);
    pub const Control: Self = Self(1u32);
    pub const Accent: Self = Self(2u32);
    pub const Branding: Self = Self(4u32);
    pub const Status: Self = Self(8u32);
    pub const Illumination: Self = Self(16u32);
    pub const Presentation: Self = Self(32u32);
}
impl ::core::marker::Copy for LampPurposes {}
impl ::core::clone::Clone for LampPurposes {
    fn clone(&self) -> Self {
        *self
    }
}

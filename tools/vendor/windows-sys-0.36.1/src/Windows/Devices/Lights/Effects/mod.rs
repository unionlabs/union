pub type ILampArrayEffect = *mut ::core::ffi::c_void;
pub type LampArrayBitmapEffect = *mut ::core::ffi::c_void;
pub type LampArrayBitmapRequestedEventArgs = *mut ::core::ffi::c_void;
pub type LampArrayBlinkEffect = *mut ::core::ffi::c_void;
pub type LampArrayColorRampEffect = *mut ::core::ffi::c_void;
pub type LampArrayCustomEffect = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Lights_Effects\"`*"]
#[repr(transparent)]
pub struct LampArrayEffectCompletionBehavior(pub i32);
impl LampArrayEffectCompletionBehavior {
    pub const ClearState: Self = Self(0i32);
    pub const KeepState: Self = Self(1i32);
}
impl ::core::marker::Copy for LampArrayEffectCompletionBehavior {}
impl ::core::clone::Clone for LampArrayEffectCompletionBehavior {
    fn clone(&self) -> Self {
        *self
    }
}
pub type LampArrayEffectPlaylist = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Lights_Effects\"`*"]
#[repr(transparent)]
pub struct LampArrayEffectStartMode(pub i32);
impl LampArrayEffectStartMode {
    pub const Sequential: Self = Self(0i32);
    pub const Simultaneous: Self = Self(1i32);
}
impl ::core::marker::Copy for LampArrayEffectStartMode {}
impl ::core::clone::Clone for LampArrayEffectStartMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Devices_Lights_Effects\"`*"]
#[repr(transparent)]
pub struct LampArrayRepetitionMode(pub i32);
impl LampArrayRepetitionMode {
    pub const Occurrences: Self = Self(0i32);
    pub const Forever: Self = Self(1i32);
}
impl ::core::marker::Copy for LampArrayRepetitionMode {}
impl ::core::clone::Clone for LampArrayRepetitionMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type LampArraySolidEffect = *mut ::core::ffi::c_void;
pub type LampArrayUpdateRequestedEventArgs = *mut ::core::ffi::c_void;

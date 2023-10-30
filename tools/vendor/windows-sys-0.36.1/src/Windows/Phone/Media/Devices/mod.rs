#[doc = "*Required features: `\"Phone_Media_Devices\"`*"]
#[repr(transparent)]
pub struct AudioRoutingEndpoint(pub i32);
impl AudioRoutingEndpoint {
    pub const Default: Self = Self(0i32);
    pub const Earpiece: Self = Self(1i32);
    pub const Speakerphone: Self = Self(2i32);
    pub const Bluetooth: Self = Self(3i32);
    pub const WiredHeadset: Self = Self(4i32);
    pub const WiredHeadsetSpeakerOnly: Self = Self(5i32);
    pub const BluetoothWithNoiseAndEchoCancellation: Self = Self(6i32);
    pub const BluetoothPreferred: Self = Self(7i32);
}
impl ::core::marker::Copy for AudioRoutingEndpoint {}
impl ::core::clone::Clone for AudioRoutingEndpoint {
    fn clone(&self) -> Self {
        *self
    }
}
pub type AudioRoutingManager = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Phone_Media_Devices\"`*"]
#[repr(transparent)]
pub struct AvailableAudioRoutingEndpoints(pub u32);
impl AvailableAudioRoutingEndpoints {
    pub const None: Self = Self(0u32);
    pub const Earpiece: Self = Self(1u32);
    pub const Speakerphone: Self = Self(2u32);
    pub const Bluetooth: Self = Self(4u32);
}
impl ::core::marker::Copy for AvailableAudioRoutingEndpoints {}
impl ::core::clone::Clone for AvailableAudioRoutingEndpoints {
    fn clone(&self) -> Self {
        *self
    }
}

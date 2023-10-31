pub type VoiceCommand = *mut ::core::ffi::c_void;
pub type VoiceCommandCompletedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_VoiceCommands\"`*"]
#[repr(transparent)]
pub struct VoiceCommandCompletionReason(pub i32);
impl VoiceCommandCompletionReason {
    pub const Unknown: Self = Self(0i32);
    pub const CommunicationFailed: Self = Self(1i32);
    pub const ResourceLimitsExceeded: Self = Self(2i32);
    pub const Canceled: Self = Self(3i32);
    pub const TimeoutExceeded: Self = Self(4i32);
    pub const AppLaunched: Self = Self(5i32);
    pub const Completed: Self = Self(6i32);
}
impl ::core::marker::Copy for VoiceCommandCompletionReason {}
impl ::core::clone::Clone for VoiceCommandCompletionReason {
    fn clone(&self) -> Self {
        *self
    }
}
pub type VoiceCommandConfirmationResult = *mut ::core::ffi::c_void;
pub type VoiceCommandContentTile = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_VoiceCommands\"`*"]
#[repr(transparent)]
pub struct VoiceCommandContentTileType(pub i32);
impl VoiceCommandContentTileType {
    pub const TitleOnly: Self = Self(0i32);
    pub const TitleWithText: Self = Self(1i32);
    pub const TitleWith68x68Icon: Self = Self(2i32);
    pub const TitleWith68x68IconAndText: Self = Self(3i32);
    pub const TitleWith68x92Icon: Self = Self(4i32);
    pub const TitleWith68x92IconAndText: Self = Self(5i32);
    pub const TitleWith280x140Icon: Self = Self(6i32);
    pub const TitleWith280x140IconAndText: Self = Self(7i32);
}
impl ::core::marker::Copy for VoiceCommandContentTileType {}
impl ::core::clone::Clone for VoiceCommandContentTileType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type VoiceCommandDefinition = *mut ::core::ffi::c_void;
pub type VoiceCommandDisambiguationResult = *mut ::core::ffi::c_void;
pub type VoiceCommandResponse = *mut ::core::ffi::c_void;
pub type VoiceCommandServiceConnection = *mut ::core::ffi::c_void;
pub type VoiceCommandUserMessage = *mut ::core::ffi::c_void;

pub type IMidiMessage = *mut ::core::ffi::c_void;
pub type IMidiOutPort = *mut ::core::ffi::c_void;
pub type MidiActiveSensingMessage = *mut ::core::ffi::c_void;
pub type MidiChannelPressureMessage = *mut ::core::ffi::c_void;
pub type MidiContinueMessage = *mut ::core::ffi::c_void;
pub type MidiControlChangeMessage = *mut ::core::ffi::c_void;
pub type MidiInPort = *mut ::core::ffi::c_void;
pub type MidiMessageReceivedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Midi\"`*"]
#[repr(transparent)]
pub struct MidiMessageType(pub i32);
impl MidiMessageType {
    pub const None: Self = Self(0i32);
    pub const NoteOff: Self = Self(128i32);
    pub const NoteOn: Self = Self(144i32);
    pub const PolyphonicKeyPressure: Self = Self(160i32);
    pub const ControlChange: Self = Self(176i32);
    pub const ProgramChange: Self = Self(192i32);
    pub const ChannelPressure: Self = Self(208i32);
    pub const PitchBendChange: Self = Self(224i32);
    pub const SystemExclusive: Self = Self(240i32);
    pub const MidiTimeCode: Self = Self(241i32);
    pub const SongPositionPointer: Self = Self(242i32);
    pub const SongSelect: Self = Self(243i32);
    pub const TuneRequest: Self = Self(246i32);
    pub const EndSystemExclusive: Self = Self(247i32);
    pub const TimingClock: Self = Self(248i32);
    pub const Start: Self = Self(250i32);
    pub const Continue: Self = Self(251i32);
    pub const Stop: Self = Self(252i32);
    pub const ActiveSensing: Self = Self(254i32);
    pub const SystemReset: Self = Self(255i32);
}
impl ::core::marker::Copy for MidiMessageType {}
impl ::core::clone::Clone for MidiMessageType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MidiNoteOffMessage = *mut ::core::ffi::c_void;
pub type MidiNoteOnMessage = *mut ::core::ffi::c_void;
pub type MidiOutPort = *mut ::core::ffi::c_void;
pub type MidiPitchBendChangeMessage = *mut ::core::ffi::c_void;
pub type MidiPolyphonicKeyPressureMessage = *mut ::core::ffi::c_void;
pub type MidiProgramChangeMessage = *mut ::core::ffi::c_void;
pub type MidiSongPositionPointerMessage = *mut ::core::ffi::c_void;
pub type MidiSongSelectMessage = *mut ::core::ffi::c_void;
pub type MidiStartMessage = *mut ::core::ffi::c_void;
pub type MidiStopMessage = *mut ::core::ffi::c_void;
pub type MidiSynthesizer = *mut ::core::ffi::c_void;
pub type MidiSystemExclusiveMessage = *mut ::core::ffi::c_void;
pub type MidiSystemResetMessage = *mut ::core::ffi::c_void;
pub type MidiTimeCodeMessage = *mut ::core::ffi::c_void;
pub type MidiTimingClockMessage = *mut ::core::ffi::c_void;
pub type MidiTuneRequestMessage = *mut ::core::ffi::c_void;

#[link(name = "windows")]
extern "system" {
    #[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn XInputEnable(enable: super::super::super::Foundation::BOOL);
    #[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
    pub fn XInputGetAudioDeviceIds(dwuserindex: u32, prenderdeviceid: ::windows_sys::core::PWSTR, prendercount: *mut u32, pcapturedeviceid: ::windows_sys::core::PWSTR, pcapturecount: *mut u32) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
    pub fn XInputGetBatteryInformation(dwuserindex: u32, devtype: u8, pbatteryinformation: *mut XINPUT_BATTERY_INFORMATION) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
    pub fn XInputGetCapabilities(dwuserindex: u32, dwflags: u32, pcapabilities: *mut XINPUT_CAPABILITIES) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
    pub fn XInputGetKeystroke(dwuserindex: u32, dwreserved: u32, pkeystroke: *mut XINPUT_KEYSTROKE) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
    pub fn XInputGetState(dwuserindex: u32, pstate: *mut XINPUT_STATE) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
    pub fn XInputSetState(dwuserindex: u32, pvibration: *const XINPUT_VIBRATION) -> u32;
}
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const BATTERY_DEVTYPE_GAMEPAD: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const BATTERY_DEVTYPE_HEADSET: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const BATTERY_LEVEL_EMPTY: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const BATTERY_LEVEL_FULL: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const BATTERY_LEVEL_LOW: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const BATTERY_LEVEL_MEDIUM: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const BATTERY_TYPE_ALKALINE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const BATTERY_TYPE_DISCONNECTED: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const BATTERY_TYPE_NIMH: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const BATTERY_TYPE_UNKNOWN: u32 = 255u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const BATTERY_TYPE_WIRED: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub struct XINPUT_BATTERY_INFORMATION {
    pub BatteryType: u8,
    pub BatteryLevel: u8,
}
impl ::core::marker::Copy for XINPUT_BATTERY_INFORMATION {}
impl ::core::clone::Clone for XINPUT_BATTERY_INFORMATION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub struct XINPUT_CAPABILITIES {
    pub Type: u8,
    pub SubType: u8,
    pub Flags: u16,
    pub Gamepad: XINPUT_GAMEPAD,
    pub Vibration: XINPUT_VIBRATION,
}
impl ::core::marker::Copy for XINPUT_CAPABILITIES {}
impl ::core::clone::Clone for XINPUT_CAPABILITIES {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_CAPS_FFB_SUPPORTED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_CAPS_NO_NAVIGATION: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_CAPS_PMD_SUPPORTED: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_CAPS_VOICE_SUPPORTED: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_CAPS_WIRELESS: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_DEVSUBTYPE_ARCADE_PAD: u32 = 19u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_DEVSUBTYPE_ARCADE_STICK: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_DEVSUBTYPE_DANCE_PAD: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_DEVSUBTYPE_DRUM_KIT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_DEVSUBTYPE_FLIGHT_STICK: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_DEVSUBTYPE_GAMEPAD: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_DEVSUBTYPE_GUITAR: u32 = 6u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_DEVSUBTYPE_GUITAR_ALTERNATE: u32 = 7u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_DEVSUBTYPE_GUITAR_BASS: u32 = 11u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_DEVSUBTYPE_UNKNOWN: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_DEVSUBTYPE_WHEEL: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_DEVTYPE_GAMEPAD: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_DLL: &str = "xinput1_4.dll";
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_DLL_A: &str = "xinput1_4.dll";
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_DLL_W: &str = "xinput1_4.dll";
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_FLAG_GAMEPAD: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub struct XINPUT_GAMEPAD {
    pub wButtons: u16,
    pub bLeftTrigger: u8,
    pub bRightTrigger: u8,
    pub sThumbLX: i16,
    pub sThumbLY: i16,
    pub sThumbRX: i16,
    pub sThumbRY: i16,
}
impl ::core::marker::Copy for XINPUT_GAMEPAD {}
impl ::core::clone::Clone for XINPUT_GAMEPAD {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_GAMEPAD_A: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_GAMEPAD_B: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_GAMEPAD_BACK: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_GAMEPAD_DPAD_DOWN: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_GAMEPAD_DPAD_LEFT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_GAMEPAD_DPAD_RIGHT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_GAMEPAD_DPAD_UP: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_GAMEPAD_LEFT_SHOULDER: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_GAMEPAD_LEFT_THUMB: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE: u32 = 7849u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_GAMEPAD_RIGHT_SHOULDER: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_GAMEPAD_RIGHT_THUMB: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_GAMEPAD_RIGHT_THUMB_DEADZONE: u32 = 8689u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_GAMEPAD_START: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_GAMEPAD_TRIGGER_THRESHOLD: u32 = 30u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_GAMEPAD_X: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_GAMEPAD_Y: u32 = 32768u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub struct XINPUT_KEYSTROKE {
    pub VirtualKey: XINPUT_VIRTUAL_KEY,
    pub Unicode: u16,
    pub Flags: u16,
    pub UserIndex: u8,
    pub HidCode: u8,
}
impl ::core::marker::Copy for XINPUT_KEYSTROKE {}
impl ::core::clone::Clone for XINPUT_KEYSTROKE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_KEYSTROKE_KEYDOWN: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_KEYSTROKE_KEYUP: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XINPUT_KEYSTROKE_REPEAT: u32 = 4u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub struct XINPUT_STATE {
    pub dwPacketNumber: u32,
    pub Gamepad: XINPUT_GAMEPAD,
}
impl ::core::marker::Copy for XINPUT_STATE {}
impl ::core::clone::Clone for XINPUT_STATE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub struct XINPUT_VIBRATION {
    pub wLeftMotorSpeed: u16,
    pub wRightMotorSpeed: u16,
}
impl ::core::marker::Copy for XINPUT_VIBRATION {}
impl ::core::clone::Clone for XINPUT_VIBRATION {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub type XINPUT_VIRTUAL_KEY = u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_A: XINPUT_VIRTUAL_KEY = 22528u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_B: XINPUT_VIRTUAL_KEY = 22529u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_X: XINPUT_VIRTUAL_KEY = 22530u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_Y: XINPUT_VIRTUAL_KEY = 22531u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_RSHOULDER: XINPUT_VIRTUAL_KEY = 22532u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_LSHOULDER: XINPUT_VIRTUAL_KEY = 22533u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_LTRIGGER: XINPUT_VIRTUAL_KEY = 22534u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_RTRIGGER: XINPUT_VIRTUAL_KEY = 22535u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_DPAD_UP: XINPUT_VIRTUAL_KEY = 22544u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_DPAD_DOWN: XINPUT_VIRTUAL_KEY = 22545u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_DPAD_LEFT: XINPUT_VIRTUAL_KEY = 22546u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_DPAD_RIGHT: XINPUT_VIRTUAL_KEY = 22547u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_START: XINPUT_VIRTUAL_KEY = 22548u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_BACK: XINPUT_VIRTUAL_KEY = 22549u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_LTHUMB_PRESS: XINPUT_VIRTUAL_KEY = 22550u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_RTHUMB_PRESS: XINPUT_VIRTUAL_KEY = 22551u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_LTHUMB_UP: XINPUT_VIRTUAL_KEY = 22560u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_LTHUMB_DOWN: XINPUT_VIRTUAL_KEY = 22561u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_LTHUMB_RIGHT: XINPUT_VIRTUAL_KEY = 22562u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_LTHUMB_LEFT: XINPUT_VIRTUAL_KEY = 22563u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_LTHUMB_UPLEFT: XINPUT_VIRTUAL_KEY = 22564u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_LTHUMB_UPRIGHT: XINPUT_VIRTUAL_KEY = 22565u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_LTHUMB_DOWNRIGHT: XINPUT_VIRTUAL_KEY = 22566u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_LTHUMB_DOWNLEFT: XINPUT_VIRTUAL_KEY = 22567u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_RTHUMB_UP: XINPUT_VIRTUAL_KEY = 22576u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_RTHUMB_DOWN: XINPUT_VIRTUAL_KEY = 22577u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_RTHUMB_RIGHT: XINPUT_VIRTUAL_KEY = 22578u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_RTHUMB_LEFT: XINPUT_VIRTUAL_KEY = 22579u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_RTHUMB_UPLEFT: XINPUT_VIRTUAL_KEY = 22580u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_RTHUMB_UPRIGHT: XINPUT_VIRTUAL_KEY = 22581u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_RTHUMB_DOWNRIGHT: XINPUT_VIRTUAL_KEY = 22582u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const VK_PAD_RTHUMB_DOWNLEFT: XINPUT_VIRTUAL_KEY = 22583u16;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XUSER_INDEX_ANY: u32 = 255u32;
#[doc = "*Required features: `\"Win32_UI_Input_XboxController\"`*"]
pub const XUSER_MAX_COUNT: u32 = 4u32;

#[repr(C)]
#[doc = "*Required features: `\"Win32_Media_Audio_Endpoints\"`*"]
pub struct AUDIO_ENDPOINT_SHARED_CREATE_PARAMS {
    pub u32Size: u32,
    pub u32TSSessionId: u32,
    pub targetEndpointConnectorType: EndpointConnectorType,
    pub wfxDeviceFormat: super::WAVEFORMATEX,
}
impl ::core::marker::Copy for AUDIO_ENDPOINT_SHARED_CREATE_PARAMS {}
impl ::core::clone::Clone for AUDIO_ENDPOINT_SHARED_CREATE_PARAMS {
    fn clone(&self) -> Self {
        *self
    }
}
pub const DEVINTERFACE_AUDIOENDPOINTPLUGIN: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2670689126, data2: 26028, data3: 20390, data4: [138, 228, 18, 60, 120, 184, 147, 19] };
#[doc = "*Required features: `\"Win32_Media_Audio_Endpoints\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const DEVPKEY_AudioEndpointPlugin2_FactoryCLSID: super::super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 316160983, data2: 53010, data3: 18110, data4: [133, 64, 129, 39, 16, 211, 2, 28] }, pid: 4u32 };
#[doc = "*Required features: `\"Win32_Media_Audio_Endpoints\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const DEVPKEY_AudioEndpointPlugin_DataFlow: super::super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 316160983, data2: 53010, data3: 18110, data4: [133, 64, 129, 39, 16, 211, 2, 28] }, pid: 2u32 };
#[doc = "*Required features: `\"Win32_Media_Audio_Endpoints\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const DEVPKEY_AudioEndpointPlugin_FactoryCLSID: super::super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 316160983, data2: 53010, data3: 18110, data4: [133, 64, 129, 39, 16, 211, 2, 28] }, pid: 1u32 };
#[doc = "*Required features: `\"Win32_Media_Audio_Endpoints\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const DEVPKEY_AudioEndpointPlugin_PnPInterface: super::super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 316160983, data2: 53010, data3: 18110, data4: [133, 64, 129, 39, 16, 211, 2, 28] }, pid: 3u32 };
#[doc = "*Required features: `\"Win32_Media_Audio_Endpoints\"`*"]
pub type EndpointConnectorType = i32;
#[doc = "*Required features: `\"Win32_Media_Audio_Endpoints\"`*"]
pub const eHostProcessConnector: EndpointConnectorType = 0i32;
#[doc = "*Required features: `\"Win32_Media_Audio_Endpoints\"`*"]
pub const eOffloadConnector: EndpointConnectorType = 1i32;
#[doc = "*Required features: `\"Win32_Media_Audio_Endpoints\"`*"]
pub const eLoopbackConnector: EndpointConnectorType = 2i32;
#[doc = "*Required features: `\"Win32_Media_Audio_Endpoints\"`*"]
pub const eKeywordDetectorConnector: EndpointConnectorType = 3i32;
#[doc = "*Required features: `\"Win32_Media_Audio_Endpoints\"`*"]
pub const eConnectorCount: EndpointConnectorType = 4i32;
pub type IAudioEndpointFormatControl = *mut ::core::ffi::c_void;
pub type IAudioEndpointLastBufferControl = *mut ::core::ffi::c_void;
pub type IAudioEndpointOffloadStreamMeter = *mut ::core::ffi::c_void;
pub type IAudioEndpointOffloadStreamMute = *mut ::core::ffi::c_void;
pub type IAudioEndpointOffloadStreamVolume = *mut ::core::ffi::c_void;
pub type IAudioEndpointVolume = *mut ::core::ffi::c_void;
pub type IAudioEndpointVolumeCallback = *mut ::core::ffi::c_void;
pub type IAudioEndpointVolumeEx = *mut ::core::ffi::c_void;
pub type IAudioLfxControl = *mut ::core::ffi::c_void;
pub type IAudioMeterInformation = *mut ::core::ffi::c_void;
pub type IHardwareAudioEngineBase = *mut ::core::ffi::c_void;
